use std::cell::{RefCell};
use crate::ast::{Ast, Binary, BinaryOp, Expr, Literal, LogicalOp, ProcCall, ProcDeclaration, Stmt, Unary, UnaryOp, Variable};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::{fs, mem};
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use miette::SourceSpan;
use crate::aplang::ApLang;
use crate::errors::{Reports, RuntimeError};
use crate::aplang_std::Modules;
use crate::lexer::LiteralValue;
use crate::token::Token;

// we're using this weird error type because miette! slows down the execution
// of recursive code by a HUGE amount
// we profiled and couldn't figure out how to solve the issue
// we don't know why.
// we would like to use miette! macro it would
// make reports better and easier to write
// increment this counter if you try to solve this and fail
// COLLECTIVE HOURS WASTED: 20

// variable value types
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Number(f64),
    Bool(bool),
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    NativeFunction(),
    Function(),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
                Value::Null => write!(f, "NULL"),
                Value::List(l) => {
                    // Borrow the list to access its elements
                    let list = l.borrow();

                    // Begin the list with an opening bracket
                    write!(f, "[")?;

                    // Iterate over the elements, formatting each one
                    for (i, item) in list.iter().enumerate() {
                        if i > 0 {
                            // Add a comma and space before all elements except the first
                            write!(f, ", ")?;
                        }
                        // Write the current element using its Display implementation
                        write!(f, "{}", item)?;
                    }

                    // Close the list with a closing bracket
                    write!(f, "]")
                }
                Value::String(s) => write!(f, "{s}"),
                Value::Number(v) => write!(f, "{v}"),
                Value::Bool(b) => write!(f, "{b}", ),
                _ => { write!(f, "FUNCTION")}
        }
    }
}

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], args_toks: &[SourceSpan]) -> Result<Value, RuntimeError>;
    fn arity(&self) -> u8;
}

pub struct Procedure {
    pub name: String,
    pub params: Vec<Variable>,
    pub body: Stmt,
}

impl Callable for Procedure {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], args_toks: &[SourceSpan]) -> Result<Value, RuntimeError> {
        // save the retval
        let cached_retval = interpreter.ret_val.clone();

        // todo: consider allowing variables to be taken into context
        // ignore the global env
        interpreter.venv.initialize_empty_scope();

        // copy in the arguments
        // assigns them to their appropriate name parameter
        self.params.iter().zip(args.iter().cloned())
            .for_each(|(param, arg)| {
                interpreter.venv.define(Arc::new(param.clone()), arg)
            });
        
        // execute the function
        interpreter.stmt(&self.body)?;
        
        let retval = interpreter.ret_val.clone();
        // todo implement backtrace
        interpreter.ret_val = cached_retval;

        // restore the previous env
        interpreter.venv.scrape();
        
        match retval {
            None => Ok(Value::Null),
            Some(value) =>Ok(value),
        }
    }

    fn arity(&self) -> u8 {
        self.params.len().try_into().unwrap()
    }
}

pub struct NativeProcedure {
    pub name: String,
    pub arity: u8,
    pub callable: fn(&mut Interpreter, &[Value], args_toks: &[SourceSpan]) -> Result<Value, RuntimeError>
}

impl Callable for NativeProcedure {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value], args_toks: &[SourceSpan]) -> Result<Value, RuntimeError> {
        (self.callable)(interpreter, args, args_toks)
    }
    
    fn arity(&self) -> u8 {
        self.arity
    }
}



pub type FunctionMap = HashMap<String, (Rc<dyn Callable>, Option<Arc<ProcDeclaration>>)>;
/*                            |^^^^^^  |^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^|> Maybe pointer to function def
                              |        |                                                  If None: it is native function
                              |        |> Pointer to the function
                              |> Function name (symbol)
*/

// context structure, contains variables
//
// behavior:
// declaration and assignment are the same,
// therefore, values will be overwritten
// when declared multiple times
//
// methods:
// - get variable
// - update variable
// - lookup variable
// does the same for functions
#[derive(Clone)]
pub struct Env {
    /// private functions
    pub functions: FunctionMap,

    /// public functions
    pub exports: FunctionMap,

    venv: Vec<Context>,
}

#[derive(Default, Clone, Debug)]
struct Context {
    variables: HashMap<String, (Value, Arc<Variable>)>,
    //              |^^^^^   |^^^       ^^^^^^^^|> Source code pointer
    //              |        |> Value of symbol
    //              |> Name of symbol

    // functions: HashMap<String, ~~Function~~ >
}

impl Env {
    /// used for creating function (or the base env)
    pub fn initialize_empty_scope(&mut self) {
        self.venv.push(Context::default())
    }

    /// creates a new block scope.
    /// used for something like a For Loop, or an If Stmt
    pub fn create_nested_layer(&mut self) {
        let enclosing = self.activate().clone();
        self.venv.push(enclosing)
    }

    /// replace the previous layer with the current layer.
    pub fn flatten_nested_layer(&mut self) {
        let context = self.scrape();

        *self.activate() = context;
    }

    /// pops off the current layer of the venv
    fn scrape(&mut self) -> Context {
        self.venv
            .pop()
            .expect("attempted to remove context but failed")
    }

    /// gets mutable ref to the current layer
    fn activate(&mut self) -> &mut Context {
        let len = self.venv.len();
        &mut self.venv[len - 1]
    }


    /// creates a variable with some value
    pub fn define(&mut self, variable: Arc<Variable>, value: Value) {
        // add the variable into the context
        self.activate()
            .variables
            .insert(variable.ident.clone(), (value, variable));
    }
    
    /// look up a variable based on the symbol
    pub fn lookup_name(&mut self, var: &str, tok: Token) -> Result<&(Value, Arc<Variable>), RuntimeError> {
        self.activate()
            .variables
            .get(var)
            .ok_or(
                RuntimeError {
                    span: tok.span,
                    message: "Invalid Variable".to_string(),
                    help: format!("Make sure to create the variable `{var}` before you use it"),
                    label: "Could not find variable".to_string()
                }
            )
    }


    /// looks up the variable by comparing the entire variable object
    pub fn lookup_var(&mut self, var: &Variable) -> Result<&Value, RuntimeError> {
        Ok(&self.lookup_name(var.ident.as_str(), var.token.clone())?.0)
    }

    pub fn lookup_function(&self, function_name: String, tok: Token) -> Result<Rc<dyn Callable>, RuntimeError> {
        let (a, b) = self.functions.get(&function_name).ok_or(
            RuntimeError {
                span: tok.span,
                message: "Invalid PROCEDURE".to_string(),
                help: format!("Make sure to create the PROCEDURE `{function_name}` before you call it"),
                label: "This PROCEDURE doesn't exist".to_string()
            }
        )?.clone();
        Ok(a)
    }

    /// removes a variable
    pub fn remove(&mut self, variable: Arc<Variable>) -> Option<(Value, Arc<Variable>)> {
        self.activate().variables.remove(&variable.ident)
    }

    pub fn contains(&mut self, variable: Arc<Variable>) -> bool {
        self.activate().variables.contains_key(&variable.ident)
    }
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self { functions: Default::default(), exports: Default::default(), venv: vec![] };
        // push the base context layer into env so we don't panic
        env.initialize_empty_scope();
        env
    }
    
}

#[derive(Clone)]
pub struct Interpreter {
    venv: Env,
    ast: Ast,
    
    ret_val: Option<Value>,
    in_loop_scope: bool,
    should_loop_break: bool,
    should_loop_continue: bool,
    
    modules: Modules,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        let mut interpreter = Self {
            venv: Env::default(),
            ast,
            ret_val: None,
            in_loop_scope: false,
            should_loop_break: false,
            should_loop_continue: false,
            modules: Modules::init(),
        };

        // initiate the core std functions
        interpreter.venv.functions.extend(
            interpreter.modules.lookup("CORE").unwrap()()
        );
        
        interpreter
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
    fn stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
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
                        // floor the value into an int so we can iterate
                        let count = count as usize;
                        for _ in 1..=count {
                            // if the BREAK stmt was called then we need to deal with that
                            if self.should_loop_break {
                                self.should_loop_break = false;
                                break;
                            }

                            // if the CONTINUE stmt was called then we need to deal with that
                            if self.should_loop_continue {
                                self.should_loop_continue = false;
                                continue;
                            }

                            self.stmt(&repeat_times.body)?;
                        }
                        Ok(())
                    } // format!("cannot do count for value {value:?}")
                    value => Err(
                        RuntimeError {
                            span: repeat_times.count_token.span,
                            message: "Invalid Value for nTIMES".to_string(),
                            help: format!("Make sure `{value:?}` is a NUMBER"),
                            label: "Invalid Value here".to_string()
                        }
                    ),
                }
            }
            Stmt::RepeatUntil(repeat_until) => {
                while Self::is_truthy(&self.expr(&repeat_until.condition)?) {
                    // if the BREAK stmt was called then we need to deal with that
                    if self.should_loop_break {
                        self.should_loop_break = false;
                        break;
                    }

                    // if the CONTINUE stmt was called then we need to deal with that
                    if self.should_loop_continue {
                        self.should_loop_continue = false;
                        continue;
                    }

                    self.stmt(&repeat_until.body)?;
                }
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

                let len = values.borrow().len();
                for i in 0..len {
                    // inserting temporary value into env
                    self.venv.define(element.clone(), values.borrow()[i].clone());
                    // execute body
                    // if the BREAK stmt was called then we need to deal with that
                    if self.should_loop_break {
                        self.should_loop_break = false;
                        break;
                    }

                    // if the CONTINUE stmt was called then we need to deal with that
                    if self.should_loop_continue {
                        self.should_loop_continue = false;
                        continue;
                    }

                    // todo possible bug: confirm that this doesnt have any weird value errors
                    self.stmt(&for_each.body)?;
                    // get temp val out and change it in vec
                    (*values.borrow_mut())[i] = self.venv.remove(element.clone()).unwrap().0;
                }

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
                
                self.ret_val = match &ret_val.data {
                    None => Some(Value::Null),
                    Some(expr) => {
                        Some(self.expr(expr)?)
                    }
                };
                
                Ok(())
            }
            Stmt::Continue(cont) => {

                // we should be in a loop scope here
                // if not, uh oh
                self.should_loop_continue = true;

                Ok(())
            },
            Stmt::Break(brk) => {
                // we should be in a loop scope here
                // if not, uh oh
                self.should_loop_break = true;

                Ok(())
            }
            Stmt::Block(block) => {
                self.venv.create_nested_layer();

                for stmt in block.statements.iter() {
                    if self.should_loop_continue || self.should_loop_break {
                        // we should be in a loop scope now
                        // if we aren't, uh oh

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
                    let maybe_path = Path::new(module_name);

                    // check if the file has a dot ap extension.
                    // if it does then continue
                    // if not, then try to import an invalid std
                    if maybe_path.extension().map(|os_str| os_str
                        .to_string_lossy()
                        .eq_ignore_ascii_case("ap"))
                        .is_some_and(|res| res) {
                    } else {
                        Err(RuntimeError {
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
                            span: import.module_name.span,
                            message: format!("file {} does not exist, or is a directory. could not import user module", module_name),
                            label: "invalid file path".to_string(),
                            help: "specify a valid path to '.ap' file to import an std module".to_string(),
                        })?;
                    }

                    // TODO: BUG: Only can accept an absolute path. work on relative paths
                    // attempt to read module
                    let (Ok(module_source_code), Some(file_name)) = (fs::read_to_string(maybe_path), maybe_path.file_name()) else {
                        Err(RuntimeError {
                            span: import.module_name.span,
                            message: format!("user module {} exists but could not read source", module_name),
                            label: "failed to read".to_string(),
                            help: "specify a valid path to '.ap' file to import an std module".to_string(),
                        })?
                    };

                    // package source code
                    let module_source_code: Arc<str> = module_source_code.into();

                    // convert filename into regular string
                    let file_name = file_name.to_string_lossy().into_owned();


                    // init the module interpreter
                    let aplang = ApLang::new(module_source_code, file_name);

                    // todo: pass up the errors dont just explode right away

                    // lex
                    let lexed = aplang.lex().map_err(Reports::from).unwrap();
                    // parse
                    let parsed = lexed.parse().map_err(Reports::from).unwrap();
                    // execute the module, get the exports
                    parsed.execute_as_module().unwrap()
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
        use Expr::*;
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
                .lookup_name(v.ident.clone().as_str(), v.token.clone())
                .cloned()
                .map(|(value, _)| value),
            Assign(assignment) => {
                // execute the expression
                let result = self.expr(&assignment.value)?;
                match &result {
                    Value::List(list) => {
                        match self.venv.lookup_var(&assignment.target.clone()) {
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

        let callable = self.venv.lookup_function(proc.ident.clone(), proc.token.clone())?;

        if callable.arity() as usize != argument_evaluations.len() {
            return Err(
                RuntimeError {
                    span: (proc.parens.0.span.offset() + proc.parens.0.span.len() .. proc.parens.1.span.offset()).into(),
                    message: "Incorrect Number Of Args".to_string(),
                    help: "Make sure the you are passing in the correct number of arguments to the PROCEDURE".to_string(),
                    label: format!("There should be {} arg{}; Found {}", callable.arity(), if callable.arity() == 1 {""} else {"s"}, argument_evaluations.len())
                }
            ) // todo make this error message better -- use source proc pointer
        }

        callable.call(self, argument_evaluations.as_ref(), proc.arguments_spans.as_ref())
    }

    // help: a string can be thought of a list of chars
    fn list(&mut self, list: &crate::ast::List) -> Result<Value, RuntimeError> {
        list.items
            .iter()
            .map(|expr: &Expr| self.expr(expr))
            .collect::<Result<Vec<Value>, RuntimeError>>()
            .map(|x|Value::List(RefCell::new(x).into()))
    }

    fn access(&mut self, access: &crate::ast::Access) -> Result<Value, RuntimeError> {
        let list = self.expr(&access.list)?;
        let idx = self.expr(&access.key)?;
        
        let Value::Number(idx) = idx else {
            return Err(
                RuntimeError {
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
                        span: (access.brackets.0.span.offset() + access.brackets.0.span.len() .. access.brackets.1.span.offset()).into(),
                        message: "Invalid List Index".to_string(),
                        help: format!("Make sure index `{idx}` is less than {}", string.len()),
                        label: "Index must be less than the length of the STRING".to_string()
                    }
                )
            }
            Value::List(list) => {
                list.borrow().get((idx - 1.0) as usize).cloned().ok_or_else(||
                    RuntimeError {
                        span: (access.brackets.0.span.offset() + access.brackets.0.span.len() .. access.brackets.1.span.offset()).into(),
                        message: "Invalid List Index".to_string(),
                        help: format!("Make sure index `{idx}` is less than {}", list.borrow().len()),
                        label: "Index must be less than the length of the LIST".to_string()
                    }
                )
            }
            _ => Err(
                RuntimeError {
                    span: access.list_token.span,
                    message: "Invalid Type".to_string(),
                    help: "You can only access STRINGS and LISTS this way".to_string(),
                    label: "This has the wrong type".to_string()
                }
            )
        };
        
        target
    }

    fn set(&mut self, set: &crate::ast::Set) -> Result<Value, RuntimeError> {
        let list = self.expr(&set.list)?;
        let idx = self.expr(&set.idx)?;
        let value = self.expr(&set.value)?;

        let Value::List(ref list) = list else {
            return Err(
                RuntimeError {
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
                    span: (set.brackets.0.span.offset() + set.brackets.0.span.len() .. set.brackets.1.span.offset()).into(),
                    message: "Invalid List Index".to_string(),
                    help: format!("Make sure index `{idx}` is less than {}", list.borrow().len()),
                    label: "Index must be a Number!".to_string()
                }
            )
        };

        if let Some(target) = list.borrow_mut().get_mut((idx - 1.0) as usize) {
            *target = value.clone();
        }

        Ok(value)
    }

    fn binary(&mut self, node: &Binary) -> Result<Value, RuntimeError> {
        let lhs = self.expr(&node.left)?;
        let rhs = self.expr(&node.right)?;

        use BinaryOp::*;
        use Value::*;
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
                            span: node.token.span,
                            message: "Division by Zero".to_string(),
                            help: "Remember not to divide by zero".to_string(),
                            label: "Cannot divide by zero".to_string()
                        }
                    )
                }
            }
            (String(a), Plus, String(b)) => Ok(String(format!("{a}{b}"))),
            (List(a), Plus, List(b)) => {
                // adding two lists
                // todo: consider using try_borrow?
                let new_list: Vec<_> = a.borrow().iter().cloned().chain(b.borrow().iter().cloned()).collect();
                Ok(List(RefCell::new(new_list).into()))
            }
            _ => Err(
                RuntimeError {
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

        use UnaryOp::*;
        use Value::*;
        match (&node.operator, value) {
            (Minus, Number(num)) => Ok(Number(-num)),
            (Not, value) => Ok(Bool(!Self::is_truthy(&value))),
            (op, String(_)) => Err(
                RuntimeError {
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to String type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, NativeFunction()) => Err(
                RuntimeError {
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to NativeFunction type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, Function()) => Err(
                RuntimeError {
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to Function type"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (Minus, Bool(b)) => Err(
                RuntimeError {
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op Minus to Bool type (value) {b}"),
                    label: "Cannot do operand here".to_string(),
                }
            ),
            (op, Null) => Err(
                RuntimeError {
                    span: node.token.span,
                    message: "Invalid Unary Op".to_string(),
                    help: format!("Invalid application of unary op {op} to Null type"),
                    label: "Cannot do operand here".to_string()
                }
            ),
            (op, List(l)) => Err(
                RuntimeError {
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
