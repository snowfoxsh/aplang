use std::path::PathBuf;
use std::mem;
use miette::NamedSource;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::ops::Deref;
use crate::aplang::ApLang;
use crate::standard_library::Modules;
use crate::parser::ast::{Ast, Binary, Expr, Literal, ProcCall, Stmt, Unary};
use crate::interpreter::errors::{Reports, RuntimeError};
use crate::interpreter::procedure::FunctionMap;
use crate::interpreter::env::{Env, LoopControl};
use crate::interpreter::procedure::Procedure;
use crate::interpreter::value::Value;
use crate::lexer::token::LiteralValue;

// we're using this weird error type because miette! slows down the execution
// of recursive code by a HUGE amount
// we profiled and couldn't figure out how to solve the issue
// we don't know why.
// we would like to use miette! macro it would
// make reports better and easier to write
// increment this counter if you try to solve this and fail
// COLLECTIVE HOURS WASTED: 20
#[derive(Clone)]
pub struct Interpreter {
    pub(super) venv: Env,
    ast: Ast,

    file_path: Option<PathBuf>,

    pub(super) return_value: Option<Value>,
    loop_stack: Vec<LoopControl>,

    modules: Modules,
}

impl Interpreter {
    pub fn new(ast: Ast, file_path: Option<PathBuf>) -> Self {
        let mut interpreter = Self {
            venv: Env::default(),
            ast,
            file_path: file_path.clone(),
            return_value: None,

            loop_stack: vec![], // *
            modules: Modules::init(),
        };
        //* we start in no loops
        //* if the stack is empty then we are not in a loop anymore

        // initiate the core std functions
        interpreter.venv.functions.extend(
            interpreter.modules.lookup("CORE").unwrap()()
        );

        interpreter
    }

    pub fn get_return_value(&self) -> &Option<Value> {
        &self.return_value
    }

    pub fn get_file_path(&self) -> String {
        if let Some(file_path) = &self.file_path {
            file_path.to_string_lossy().into_owned()
        } else {
            "stdin".to_string()
        }
    }

    pub fn interpret_module(mut self) -> Result<FunctionMap, RuntimeError> {
        // temporarily take the program to avoid borrow error
        let program = mem::take(&mut self.ast.program);

        for stmt in &program {
            match stmt {
                Stmt::Expr(expr) => {
                    self.expr(expr.deref())?;
                }
                stmt => self.stmt(stmt)?,
            }
        }

        self.ast.program = program; // restore program
        Ok(self.venv.exports)
    }

    pub fn interpret(&mut self) -> Result<(), RuntimeError> {
        // temporarily take the program to avoid borrow error
        let program = mem::take(&mut self.ast.program);

        for stmt in &program {
            match stmt {
                Stmt::Expr(expr) => {
                    self.expr(expr.deref())?;
                }
                stmt => self.stmt(stmt)?,
            }
        }

        self.ast.program = program; // restore program
        Ok(())
    }

    pub fn interpret_debug(&mut self) -> Result<Vec<Value>, RuntimeError> {
        let mut values = vec![];

        let program = mem::take(&mut self.ast.program); // Temporarily take the program

        for stmt in &program {

            match stmt {
                Stmt::Expr(expr) => {
                    let value = self.expr(expr.deref())?;
                    values.push(value);
                }
                stmt => self.stmt(stmt)?,
            }
        }

        self.ast.program = program; // Restore the program
        Ok(values)
    }

    // a stmt by definition returns nothing
    pub(super) fn stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => self.expr(expr.as_ref()).map(|_| ()),
            Stmt::IfStmt(if_stmt) => {
                // evaluate the conditional
                if Self::is_truthy(&self.expr(&if_stmt.condition)?) {
                    self.stmt(&if_stmt.then_branch)
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    self.stmt(else_branch)
                } else {
                    Ok(())
                }
            }
            Stmt::RepeatTimes(repeat_times) => {
                match self.expr(&repeat_times.count)? {
                    Value::Number(count) => {
                        // we are now in a loop so keep track of the loop_stack
                        self.loop_stack.push(LoopControl::default());

                        // floor the value into an int so we can iterate
                        let count = count as usize;
                        for _ in 1..=count {
                            // if the BREAK stmt was called handle it
                            if self.loop_stack.last().unwrap().should_break {
                                self.loop_stack.last_mut().unwrap().should_break = false;
                                break;
                            }

                            // if the CONTINUE stmt was called handle it
                            if self.loop_stack.last().unwrap().should_continue {
                                self.loop_stack.last_mut().unwrap().should_continue = false;
                                continue;
                            }

                            self.stmt(&repeat_times.body)?;
                        }

                        // exit the loop
                        assert!(self.loop_stack.pop().is_some());

                        Ok(())
                    } // format!("cannot do count for value {value:?}")
                    value => Err(
                        RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), repeat_times.count_token.source.clone()),
                            span: repeat_times.count_token.span,
                            message: "Invalid Value for nTIMES".to_string(),
                            help: format!("Make sure `{value:?}` is a NUMBER"),
                            label: "Invalid Value here".to_string()
                        }
                    ),
                }
            }
            Stmt::RepeatUntil(repeat_until) => {
                // enter a loop
                self.loop_stack.push(LoopControl::default());

                while !Self::is_truthy(&self.expr(&repeat_until.condition)?) {
                    // if the BREAK stmt was called handle it
                    if self.loop_stack.last().unwrap().should_break {
                        self.loop_stack.last_mut().unwrap().should_break = false;
                        break;
                    }

                    // if the CONTINUE stmt was called handle it
                    if self.loop_stack.last().unwrap().should_continue {
                        self.loop_stack.last_mut().unwrap().should_continue = false;
                        continue;
                    }

                    self.stmt(&repeat_until.body)?;
                }

                // exit the loop
                assert!(self.loop_stack.pop().is_some());

                Ok(())
            }
            Stmt::ForEach(for_each) => {
                let values = match self.expr(&for_each.list)? {
                    Value::List(list) => list,
                    Value::String(string) => Rc::new(RefCell::new(string
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<Value>>())),
                    value => Err(
                        RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), for_each.list_token.source.clone()),
                            span: for_each.list_token.span,
                            message: "Invalid Iterator".to_string(),
                            help: format!("Cannot iterate over {value:?}. This should be a LIST or a STRING"),
                            label: "Invalid Iterator Here".to_string()
                        }
                    )?,
                };

                let element = Arc::new(for_each.item.clone());

                // if the variable already exists temperately remove it so doesn't get lost
                let maybe_cached = self.venv.remove(element.clone());

                // enter into the loop
                self.loop_stack.push(LoopControl::default());

                let len = values.borrow().len();
                for i in 0..len {
                    // inserting temporary value into env
                    self.venv.define(element.clone(), values.borrow()[i].clone());
                    // execute body

                    // handle break and continue

                    // if the BREAK stmt was called handle it
                    if self.loop_stack.last().unwrap().should_break {
                        self.loop_stack.last_mut().unwrap().should_break = false;
                        break;
                    }

                    // if the CONTINUE stmt was called then we need to deal with that
                    if self.loop_stack.last().unwrap().should_continue {
                        self.loop_stack.last_mut().unwrap().should_continue = false;
                        continue;
                    }


                    // todo possible bug: confirm that this doesnt have any weird value errors
                    self.stmt(&for_each.body)?;
                    // get temp val out and change it in vec
                    (*values.borrow_mut())[i] = self.venv.remove(element.clone()).unwrap().0;
                }

                assert!(self.loop_stack.pop().is_some());

                // put it back if it was originally defined
                if let Some((cached_value, cached_variable)) = maybe_cached {
                    self.venv.define(cached_variable, cached_value)
                }

                Ok(())
            }
            Stmt::ProcDeclaration(proc_dec) => {
                // create a new non-native aplang function

                let procedure = Rc::new(Procedure {
                    name: proc_dec.name.to_string(),
                    params: proc_dec.params.clone(),
                    body: proc_dec.body.clone(),
                });

                self.venv.functions.insert(procedure.name.clone(), (procedure.clone(), Some(proc_dec.clone())));

                if proc_dec.exported {
                    self.venv.exports.insert(procedure.name.clone(), (procedure.clone(), Some(proc_dec.clone())));
                }

                Ok(())
            },
            Stmt::Return(ret_val) => {
                // deal with the return value inside the procedure

                self.return_value = match &ret_val.data {
                    None => Some(Value::Null),
                    Some(expr) => {
                        Some(self.expr(expr)?)
                    }
                };

                Ok(())
            }
            Stmt::Continue(_cont) => {
                // we should be in a loop scope here
                // if not, uh oh
                // *should* be insured by the parser
                // todo: write an actual error message instead of panicking here
                self.loop_stack.last_mut().unwrap().should_continue = true;

                Ok(())
            },
            Stmt::Break(_brk) => {
                // we should be in a loop scope here
                // if not, uh oh
                // *should* be insured by the parser
                // todo: write an actual error message instead of panicking here
                self.loop_stack.last_mut().unwrap().should_break = true;

                Ok(())
            }
            Stmt::Block(block) => {
                self.venv.create_nested_layer();

                for stmt in block.statements.iter() {
                    if self.loop_stack.last().is_some_and(|lc| lc.should_break || lc.should_continue) {
                        // if we are in a loop then we need to STOP execution
                        break;
                    }

                    self.stmt(stmt)?
                }

                self.venv.flatten_nested_layer();

                Ok(())
            }
            Stmt::Import(import) => {
                // get a ref to the module name to be imported/activated
                let Some(LiteralValue::String(module_name)) = import.module_name.literal.as_ref() else {
                    unreachable!()
                };

                let mut module = if let Some(injector) = self.modules.lookup(module_name) {
                    // if the module is a native standard library module, get it
                    injector()
                } else {
                    // the module must be a user module or invalid

                    let Some(mut current_module_path) = self.file_path.clone() else {
                        return Err(RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), import.module_name.source.clone()),
                            span: import.module_name.span,
                            message: "user modules cannot be called when evaluating from stdin".to_string(),
                            label: "cannot use module".to_string(),
                            help: "put your code in a file to use user modules".to_string(),
                        })
                    };

                    // strip the filename from the path
                    current_module_path.pop();
                    // let maybe_path = self
                    // let maybe_path = current_module_path.join(module_name);
                    let maybe_path = current_module_path.join(module_name);

                    // check if the file has a dot ap extension.
                    // if it does then continue
                    // if not, then try to import an invalid std
                    if maybe_path.extension().map(|os_str| os_str
                        .to_string_lossy()
                        .eq_ignore_ascii_case("ap"))
                        .is_some_and(|res| res) {
                    } else {
                        Err(RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), import.module_name.source.clone()),
                            span: import.module_name.span,
                            label: "invalid std module".to_string(),
                            message: format!("std module not found {}", module_name),
                            help: "if you meant to import a user module please enter the path to the .ap file in question".to_string()
                            // maybe do a fuzzy module find?
                        })?;
                    }

                    // we need to make sure the file is actually there!
                    if !maybe_path.is_file() {
                        Err(RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), import.module_name.source.clone()),
                            span: import.module_name.span,
                            message: format!("file {} does not exist, or is a directory. could not import user module", module_name),
                            label: "invalid file path".to_string(),
                            help: "specify a valid path to '.ap' file to import an std module".to_string(),
                        })?;
                    }

                    // TODO: BUG: Only can accept an absolute path. work on relative paths
                    // // attempt to read module
                    // let (Ok(module_source_code), Some(file_name)) = (fs::read_to_string(maybe_path), maybe_path.file_name()) else {
                    //     Err(RuntimeError {
                    //         named_source: NamedSource::new(self.get_file_path(), import.module_name.source.clone()),
                    //         span: import.module_name.span,
                    //         message: format!("user module {} exists but could not read source", module_name),
                    //         label: "failed to read".to_string(),
                    //         help: "specify a valid path to '.ap' file to import an std module".to_string(),
                    //     })?
                    // };

                    // package source code
                    // let module_source_code: Arc<str> = module_source_code.into();

                    // convert filename into regular string
                    // let file_name = file_name.to_string_lossy().into_owned();


                    // init the module interpreter
                    let aplang = ApLang::new_from_file(maybe_path.to_path_buf()).map_err(|_err| {
                        RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), import.module_name.source.clone()),
                            span: import.module_name.span,
                            message: format!("user module {} exists but could not read source", module_name),
                            label: "failed to read module".to_string(),
                            help: "specify a valid path to '.ap' file to import an std module".to_string(),
                        }
                    })?;

                    // lex
                    let lexed = aplang.lex().map_err(Reports::from).unwrap();
                    // parseRun
                    let parsed = lexed.parse().map_err(Reports::from).unwrap();
                    // execute the module, get the exports
                    parsed.execute_as_module()?
                };

                // before actually adding the function, we might have to trim the module
                // if we're using IMPORT "x" FROM MOD "y"
                if let Some(functions) = import.only_functions.clone() {
                    let mut trimmed_module = FunctionMap::new();
                    // generated functions need to be removed
                    // we trim the hashmap down to only specify the specified keys
                    for function in &functions {
                        let Some(LiteralValue::String(function_name)) = function.literal.as_ref() else {
                            unreachable!()
                        };

                        let Some(function) = module.remove(function_name) else {
                            return Err(RuntimeError {
                                named_source: NamedSource::new("", function.source.clone()),
                                span: function.span,
                                message: "Invalid Function".to_string(),
                                help: format!("Function {function_name} does not exist in module {module_name}"),
                                label: "Does not exist".to_string(),
                            });
                        };

                        trimmed_module.insert(function_name.clone(), function);
                    }

                    module = trimmed_module;
                }

                // finally, add it
                self.venv.functions.extend(module);

                Ok(())
            },
        }
    }

    pub fn interpret_expr_temp(&mut self) -> Result<Vec<Value>, RuntimeError> {
        let expressions: Vec<Expr> = self
            .ast
            .program
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::Expr(expr) => Some((**expr).clone()), // Dereference Arc and clone Expr
                _ => None,
            })
            .collect(); // Collects into Vec<Expr>

        expressions
            .iter()
            .map(|expr| self.expr(expr)) // Directly use Expr reference
            .collect()
    }
    fn expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        use crate::parser::ast::Expr::*;
        use crate::parser::ast::LogicalOp;
        let value = match expr {
            Grouping(inside) => self.expr(&inside.expr),
            Literal(lit) => Ok(Self::literal(&lit.value)),
            Binary(binary) => self.binary(binary.as_ref()),
            Unary(unary) => self.unary(unary.as_ref()),
            ProcCall(proc) => {
                self.call(proc.as_ref())
            },
            Access(access) => {
                self.access(access.as_ref())
            },
            List(list) => self.list(list.as_ref()),
            Variable(v) => self.venv
                .lookup_name(v.ident.clone().as_str(), v.token.clone(), self.get_file_path())
                .cloned()
                .map(|(value, _)| value),
            Assign(assignment) => {
                // execute the expression
                let result = self.expr(&assignment.value)?;
                match &result {
                    Value::List(list) => {
                        match self.venv.lookup_var(&assignment.target.clone(), self.get_file_path()) {
                            Ok(Value::List(target_list)) => {
                                target_list.swap(list);
                            },
                            _ => self.venv.define(assignment.target.clone(), result.clone()),
                        }
                    }
                    _ => self.venv.define(assignment.target.clone(), result.clone()),
                }

                Ok(result)
            }
            Set(set) => {
                self.set(set.as_ref())
            },
            Logical(log) => {
                let left = self.expr(&log.left)?;
                let short_circuit = match log.operator {
                    LogicalOp::Or => Self::is_truthy(&left),
                    LogicalOp::And => !Self::is_truthy(&left),
                };

                if short_circuit {
                    Ok(left)
                } else {
                    Ok(self.expr(&log.right)?)
                }
            },
        };
        // println!("{value:?}");
        value
    }

    fn call(&mut self, proc: &ProcCall) -> Result<Value, RuntimeError> {
        // todo: look into callee expr

        let mut argument_evaluations = Vec::new();

        for arg in &proc.arguments {
            argument_evaluations.push(self.expr(arg)?)
        }

        let callable = self.venv.lookup_function(proc.ident.clone(), proc.token.clone(), self.get_file_path())?;

        if callable.arity() as usize != argument_evaluations.len() {
            return Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), proc.token.source.clone()),
                    span: (proc.parens.0.span.offset() + proc.parens.0.span.len() .. proc.parens.1.span.offset()).into(),
                    message: "Incorrect Number Of Args".to_string(),
                    help: "Make sure the you are passing in the correct number of arguments to the PROCEDURE".to_string(),
                    label: format!("There should be {} arg{}; Found {}", callable.arity(), if callable.arity() == 1 {""} else {"s"}, argument_evaluations.len())
                }
            ) // todo make this error message better -- use source proc pointer
        }

        callable.call(self, argument_evaluations.as_ref(), proc.arguments_spans.as_ref(), proc.token.source.clone())
    }

    // help: a string can be thought of a list of chars
    fn list(&mut self, list: &crate::parser::ast::List) -> Result<Value, RuntimeError> {
        list.items
            .iter()
            .map(|expr: &Expr| self.expr(expr))
            .collect::<Result<Vec<Value>, RuntimeError>>()
            .map(|x|Value::List(RefCell::new(x).into()))
    }

    fn access(&mut self, access: &crate::parser::ast::Access) -> Result<Value, RuntimeError> {
        let list = self.expr(&access.list)?;
        let idx = self.expr(&access.key)?;

        let Value::Number(idx) = idx else {
            return Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), access.list_token.source.clone()),
                    span: (access.brackets.0.span.offset() + access.brackets.0.span.len() .. access.brackets.1.span.offset()).into(),
                    message: "Invalid Index".to_string(),
                    help: format!("Make sure index {idx:?} is a NUMBER!"),
                    label: "Index must be a NUMBER!".to_string()
                }
            )
        };

        let target = match &list {
            Value::String(string) => {
                string.chars().nth((idx - 1.0) as usize).map(|ch| Value::String(ch.to_string())).ok_or_else(||
                    RuntimeError {
                        named_source: NamedSource::new(self.get_file_path(), access.brackets.0.source.clone()),
                        span: (access.brackets.0.span.offset() + access.brackets.0.span.len()..access.brackets.1.span.offset()).into(),
                        message: "Invalid List Index".to_string(),
                        help: format!("Make sure index `{idx}` is less than {}", string.len()),
                        label: "Index must be less than the length of the STRING".to_string()
                    }
                )
            }
            Value::List(list) => {
                list.borrow().get((idx - 1.0) as usize).cloned().ok_or_else(|| {
                    RuntimeError {
                        named_source: NamedSource::new(self.get_file_path(), access.brackets.0.source.clone()),
                        span: (access.brackets.0.span.offset() + access.brackets.0.span.len()..access.brackets.1.span.offset()).into(),
                        message: "Invalid List Index".to_string(),
                        help: format!("Make sure index `{idx}` is less than {}", list.borrow().len()),
                        label: "Index must be less than the length of the LIST".to_string()
                    }
                })
            }
            _ => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), access.list_token.source.clone()),
                    span: access.list_token.span,
                    message: "Invalid Type".to_string(),
                    help: "You can only access STRINGS and LISTS this way".to_string(),
                    label: "This has the wrong type".to_string()
                }
            )
        };

        target
    }

    fn set(&mut self, set: &crate::parser::ast::Set) -> Result<Value, RuntimeError> {
        let list = self.expr(&set.list)?;
        let idx = self.expr(&set.idx)?;
        let value = self.expr(&set.value)?;

        let Value::List(ref list) = list else {
            return Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), set.list_token.source.clone()),
                    span: set.list_token.span,
                    message: "Invalid Type".to_string(),
                    help: "You can only SET LISTS this way".to_string(),
                    label: "This should be a LIST".to_string()
                }
            )
        };

        let Value::Number(idx) = idx else {
            return Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), set.brackets.0.source.clone()),
                    span: (set.brackets.0.span.offset() + set.brackets.0.span.len() .. set.brackets.1.span.offset()).into(),
                    message: "Invalid Index".to_string(),
                    help: format!("Make sure index {idx:?} is a NUMBER!"),
                    label: "Index must be a NUMBER!".to_string()
                }
            )
        };

        let mut list_borrowed = list.borrow_mut();
        if let Some(target) = list_borrowed.get_mut((idx - 1.0) as usize) {
            *target = value.clone();
        } else {
            return Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), set.brackets.0.source.clone()),
                    span: (set.brackets.0.span.offset() + set.brackets.0.span.len() .. set.brackets.1.span.offset()).into(),
                    message: "Invalid List Index".to_string(),
                    help: format!("Make sure index `{idx}` is less than {}", list_borrowed.len()),
                    label: "Index must be less than the length of the LIST".to_string()
                }
            )
        }

        Ok(value)
    }

    fn binary(&mut self, node: &Binary) -> Result<Value, RuntimeError> {
        let lhs = self.expr(&node.left)?;
        let rhs = self.expr(&node.right)?;

        use crate::parser::ast::BinaryOp::*;
        use crate::interpreter::value::Value::*;
        match (&lhs, &node.operator, &rhs) {
            (_, EqualEqual, _) => Ok(Bool(Self::equals(&lhs, &rhs))),
            (_, NotEqual, _) => Ok(Bool(!Self::equals(&lhs, &rhs))),
            (Number(a), Less, Number(b)) => Ok(Bool(a < b)),
            (Number(a), LessEqual, Number(b)) => Ok(Bool(a <= b)),
            (Number(a), Greater, Number(b)) => Ok(Bool(a > b)),
            (Number(a), GreaterEqual, Number(b)) => Ok(Bool(a >= b)),
            (Number(a), Plus, Number(b)) => Ok(Number(a + b)),
            (Number(a), Minus, Number(b)) => Ok(Number(a - b)),
            (Number(a), Star, Number(b)) => Ok(Number(a * b)),
            (Number(a), Slash, Number(b)) => {
                if *b != 0.0 {
                    Ok(Number(a / b))
                } else {
                    Err(
                        RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                            span: node.token.span,
                            message: "Division by Zero".to_string(),
                            help: "Remember not to divide by zero".to_string(),
                            label: "Cannot divide by zero".to_string()
                        }
                    )
                }
            },
            (Number(a), Modulo, Number(b)) => {
                if *b != 0.0 {
                    Ok(Number(a % b))
                } else {
                    Err(
                        RuntimeError {
                            named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                            span: node.token.span,
                            message: "Modulo by Zero".to_string(),
                            help: "Remember not to take a modulo by zero".to_string(),
                            label: "Cannot modulo by zero".to_string()
                        }
                    )
                }
            }
            // if we add to a string implicitly cast the other thing to a string for convenience
            (String(a), Plus, b) => Ok(String(format!("{a}{b}"))),
            (List(a), Plus, List(b)) => {
                // adding two lists
                // todo: consider using try_borrow?
                let new_list: Vec<_> = a.borrow().iter().cloned().chain(b.borrow().iter().cloned()).collect();
                Ok(List(RefCell::new(new_list).into()))
            },
            _ => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Incomparable Values".to_string(),
                    help: format!("Cannot compare {:?} and {:?}", &lhs, &rhs),
                    label: "Cannot compare these two values".to_string()
                }
            ),
        }
    }

    fn literal(lit: &Literal) -> Value {
        match lit {
            Literal::Number(num) => Value::Number(*num),
            Literal::String(string) => Value::String(string.clone()),
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Null => Value::Null,
        }
    }
    fn unary(&mut self, node: &Unary) -> Result<Value, RuntimeError> {
        let value = self.expr(&node.right)?;

        use crate::parser::ast::UnaryOp::*;
        use crate::interpreter::value::Value::*;
        match (&node.operator, value) {
            (Minus, Number(num)) => Ok(Number(-num)),
            (Not, value) => Ok(Bool(!Self::is_truthy(&value))),
            (op, String(_)) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to String type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, NativeFunction()) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to NativeFunction type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, Function()) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to Function type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (Minus, Bool(b)) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op Minus to Bool type (value) {b}"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, Null) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to Null type"),
                    label: "Cannot do operand here".to_string()
                }
            ),
            (op, List(_l)) => Err(
                RuntimeError {
                    named_source: NamedSource::new(self.get_file_path(), node.token.source.clone()),
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to List type"),
                    label: "Cannot do operand here".to_string()
                }
            ),
        }
    }

    fn equals(lhs: &Value, rhs: &Value) -> bool {
        match (lhs, rhs) {
            (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Null, Value::Null) => true,
            (_, _) => false,
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Number(n) if *n == 0.0 => false,
            Value::Null => false,
            _ => true,
        }
    }
}