use std::cell::{RefCell};
use crate::ast::BinaryOp::{
    EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, NotEqual, Plus, Star,
};
use crate::ast::Expr::List;
use crate::ast::{Ast, Binary, BinaryOp, Expr, Literal, LogicalOp, ProcCall, ProcDeclaration, Stmt, Unary, UnaryOp, Variable};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use crate::errors::RuntimeError;
use crate::aplang_std::Modules;
use crate::lexer::LiteralValue;


// todo convert all the boxes to 'a

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
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, Box<RuntimeError>>;
    fn arity(&self) -> u8;
}

pub struct Procedure {
    pub name: String,
    pub params: Vec<Variable>,
    pub body: Stmt,
}

impl Callable for Procedure {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
        // save the retval
        let cached_retval = interpreter.ret_val.clone();

        // todo: consider allowing variables to be taken into context
        // ignore the global env
        interpreter.venv.initialize_empty_scope();

        // copy in the arguments
        // assign them to their appropirate name parameter
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
    pub callable: fn(&mut Interpreter, &[Value]) -> Result<Value, Box<RuntimeError>>
}

impl Callable for NativeProcedure {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, Box<RuntimeError>> {
        (self.callable)(interpreter, args)
    }
}



// context structure, contains variables
//
// behaviour:
// declaration and assignment are the same
// therefore values will be overwritten
// when declared multiple times
//
// methods:
// - get variable
// - update variable
// - lookup variable
// do the same for functions
#[derive(Clone)]
pub struct Env {
    pub functions: HashMap<String, (Rc<dyn Callable>, Option<Arc<ProcDeclaration>>)>,
    //                |^^^^^^  |^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^|> Maybe pointer to function def
    //                |        |                                                  If None: it is native function
    //                |        |> Pointer to the function
    //                |> Function name (symbol)
    pub venv: Vec<Context>,
}

enum NativeFunction {}

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

    /// crates a new block scope.
    /// Used for something like a For Loop, or an If Stmt
    pub fn create_nested_layer(&mut self) {
        let enclosing = self.activate().clone();
        self.venv.push(enclosing)
    }

    /// replace the previous layer with the current layer.
    pub fn flatten_nested_layer(&mut self) {
        let context = self.scrape();

        *self.activate() = context;
    }

    /// pops of the current layer of the venv
    pub fn scrape(&mut self) -> Context {
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
    pub fn lookup_name(&mut self, var: &str) -> Result<&(Value, Arc<Variable>), Box<RuntimeError>> {
        self.activate()
            .variables
            .get(var)
            .ok_or(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Invalid variable here".to_string(),
                    help: "Could not find variable".to_string(),
                    label: "Invalid Variable".to_string()
                })
            )
    }


    /// looks up the variable by comparing the entire variable object
    pub fn lookup_var(&mut self, var: &Variable) -> Result<&Value, Box<RuntimeError>> {
        Ok(&self.lookup_name(var.ident.as_str())?.0)
    }

    pub fn lookup_function(&self, function_name: String) -> Result<Rc<dyn Callable>, Box<RuntimeError>> {
        let (a, b) = self.functions.get(&function_name).ok_or(
            Box::new(RuntimeError {
                src: Arc::from("... code here".to_string()),
                span: (0..2).into(),
                message: "This function doesn't exist".to_string(),
                help: "Could not find function".to_string(),
                label: "Invalid Function".to_string()
            })
        )?.clone();
        Ok(a)
    }

    // pub fn create_function(&mut self, function_name: String, )

    /// removes a variable
    pub fn remove(&mut self, variable: Arc<Variable>) -> Option<(Value, Arc<Variable>)> {
        self.activate().variables.remove(&variable.ident)
    }

    pub fn contains(&mut self, variable: Arc<Variable>) -> bool {
        self.activate().variables.contains_key(&variable.ident)
    }

    // pub fn edit(&mut self, var: &str, value: Value) -> Option<Arc<Variable>> {
    //     // retrieve variable, if not found |-> None
    //     let (found_value, location) = self.activate().variables.get_mut(var)?;
    //
    //     //
    //     *found_value = value;
    //
    //     Some(location.clone())
    // }
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self { functions: Default::default(), venv: vec![] };
        // push the base context layer into env so we dont panic
        env.initialize_empty_scope();
        env
    }
    
}

#[derive(Clone)]
pub struct Interpreter {
    venv: Env,
    ast: Ast,
    ret_val: Option<Value>,
    modules: Modules,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        let mut interpreter = Self {
            venv: Env::default(),
            ast,
            ret_val: None,
            modules: Modules::init(),
        };

        // initiate the core std functions
        interpreter.modules.lookup("core").unwrap()(&mut interpreter.venv);
        interpreter
    }
    
    pub fn interpret(&mut self) -> Result<(), Box<RuntimeError>> {
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
    
    pub fn interpret_debug(&mut self) -> Result<Vec<Value>, Box<RuntimeError>> {
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
    fn stmt(&mut self, stmt: &Stmt) -> Result<(), Box<RuntimeError>> {
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
                            self.stmt(&repeat_times.body)?;
                        }
                        Ok(())
                    } // format!("cannot do count for value {value:?}")
                    value => Err(
                        Box::new(RuntimeError {
                            src: Arc::from("... code here".to_string()),
                            span: (0..2).into(),
                            message: "Invalid variable here".to_string(),
                            help: format!("Cannot do count for value {value:?}"),
                            label: "Invalid Variable".to_string()
                        })
                    ),
                }
            }
            Stmt::RepeatUntil(repeat_until) => {
                while Self::is_truthy(&self.expr(&repeat_until.condition)?) {
                    self.stmt(&repeat_until.body)?;
                }
                Ok(())
            }
            Stmt::ForEach(for_each) => {
                let mut values = match self.expr(&for_each.list)? {
                    Value::List(mut list) => list,
                    Value::String(string) => Rc::new(RefCell::new(string
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect::<Vec<Value>>())),
                    value => Err(
                        Box::new(RuntimeError {
                            src: Arc::from("... code here".to_string()),
                            span: (0..2).into(),
                            message: "Cannot make iterator here".to_string(),
                            help: format!("Cannot make iterator over value {value:?}"),
                            label: "Invalid Iterator".to_string()
                        })
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
                
                let procedure = Procedure {
                    name: proc_dec.name.to_string(),
                    params: proc_dec.params.clone(),
                    body: proc_dec.body.clone(),
                };
                
                self.venv.functions.insert(procedure.name.clone(), (Rc::new(procedure), Some(proc_dec.clone())));
                
                Ok(())
            },
            Stmt::Return(ret_val) => {
                // deal with the return value inside the procedure...
                
                self.ret_val = match &ret_val.data {
                    None => Some(Value::Null),
                    Some(expr) => {
                        Some(self.expr(expr)?)
                    }
                };
                
                Ok(())
            }
            Stmt::Block(block) => {
                self.venv.create_nested_layer();

                for stmt in block.statements.iter() {
                    self.stmt(stmt)?
                }

                self.venv.flatten_nested_layer();

                Ok(())
            }
            Stmt::Import(import) => {
                // get a ref to the module name to be imported / activated
                let Some(LiteralValue::String(module_name)) = import.import_string.literal.as_ref() else {
                    unreachable!() //
                };
                
                // load the module into the venv / activate it
                self.modules.lookup(module_name)
                    .ok_or(
                        Box::new(RuntimeError {
                            src: Arc::from("... code here".to_string()),
                            span: (0..2).into(),
                            message: "Cannot find this module".to_string(),
                            help: "Module not found (todo)".to_string(),
                            label: "Invalid Module".to_string()
                        })
                    )?(&mut self.venv);

                Ok(())
            },
        }
    }

    pub fn interpret_expr_temp(&mut self) -> Result<Vec<Value>, Box<RuntimeError>> {
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
    fn expr(&mut self, expr: &Expr) -> Result<Value, Box<RuntimeError>> {
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
                .lookup_name(v.ident.clone().as_str())
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

    fn call(&mut self, proc: &ProcCall) -> Result<Value, Box<RuntimeError>> {
        // todo: look into callee expr

        // run the argument expressions before the actual call
        let argument_evaluations: Result<Vec<_>, _> = proc.arguments // todo: figure out why this type conversion works
            .iter()
            .map(|arg| self.expr(arg)) // todo write a better error message here
            .collect();

        let Ok(argument_evaluations) = argument_evaluations else {
            // todo: write better error message -- use individual expr source pointers
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot evaluate these arguments".to_string(),
                    help: "Could not evaluate arguments".to_string(),
                    label: "Invalid Arguments".to_string()
                })
            )
        };

        let callable = self.venv.lookup_function(proc.ident.clone())?;

        // todo make the source pointer error message better

        if callable.arity() as usize != argument_evaluations.len() {
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Wrong number of arguments here".to_string(),
                    help: "Function called with incorrect number of args".to_string(),
                    label: "Incorrect Number Of Args".to_string()
                })
            ) // todo make this error message better -- use source proc pointer
        }

        callable.call(self, argument_evaluations.as_ref())
    }

    // help: a string can be thought of a list of chars
    fn list(&mut self, list: &crate::ast::List) -> Result<Value, Box<RuntimeError>> {
        list.items
            .iter()
            .map(|expr: &Expr| self.expr(expr))
            .collect::<Result<Vec<Value>, Box<RuntimeError>>>()
            .map(|x|Value::List(RefCell::new(x).into()))
    }

    fn access(&mut self, access: &crate::ast::Access) -> Result<Value, Box<RuntimeError>> {
        let list = self.expr(&access.list)?;
        let idx = self.expr(&access.key)?;

        let Value::List(list) = list else {
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Invalid Type Here".to_string(),
                    help: "This should be a LIST".to_string(),
                    label: "Invalid Type".to_string()
                })
            )
        };

        let Value::Number(idx) = idx else {
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Invalid List Index Here".to_string(),
                    help: "Index must be a Number!".to_string(),
                    label: "Invalid Index".to_string()
                })
            )
        };

       let target = list.borrow().get((idx - 1.0) as usize).cloned().ok_or_else(||
           Box::new(RuntimeError {
                src: Arc::from("... code here".to_string()),
                span: (0..2).into(),
                message: "Invalid List Index Here".to_string(),
                help: "Index must be less than the length of the LIST".to_string(),
                label: "Invalid Index".to_string()
           })
       );
        target
    }

    fn set(&mut self, set: &crate::ast::Set) -> Result<Value, Box<RuntimeError>> {
        let list = self.expr(&set.list)?;
        let idx = self.expr(&set.idx)?;
        let value = self.expr(&set.value)?;

        let Value::List(ref list) = list else {
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Invalid Type Here".to_string(),
                    help: "This should be a LIST".to_string(),
                    label: "Invalid Type".to_string()
                })
            )
        };

        let Value::Number(idx) = idx else {
            return Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Invalid List Index Here".to_string(),
                    help: "Index must be a Number!".to_string(),
                    label: "Invalid Index".to_string()
                })
            )
        };

        if let Some(mut target) = list.borrow_mut().get_mut((idx - 1.0) as usize) {
            *target = value.clone();
        }

        Ok(value)
    }

    fn binary(&mut self, node: &Binary) -> Result<Value, Box<RuntimeError>> {
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
                        Box::new(RuntimeError {
                            src: Arc::from("... code here".to_string()),
                            span: (0..2).into(),
                            message: "Division by Zero Here".to_string(),
                            help: "Cannot divide by zero".to_string(),
                            label: "Divide by Zero".to_string()
                        })
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
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Internal Bug".to_string(),
                    help: "Invalid operands in binary op not equal".to_string(),
                    label: "Invalid Operands".to_string()
                })
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
    fn unary(&mut self, node: &Unary) -> Result<Value, Box<RuntimeError>> {
        let value = self.expr(&node.right)?;

        use UnaryOp::*;
        use Value::*;
        match (&node.operator, value) {
            (Minus, Number(num)) => Ok(Number(-num)),
            (Not, value) => Ok(Bool(!Self::is_truthy(&value))),
            (op, String(_)) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op {op} to String type"),
                    label: "Invalid Application".to_string()
                })
            ),
            (op, NativeFunction()) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op {op} to NativeFunction type"),
                    label: "Invalid Application".to_string()
                })
            ),
            (op, Function()) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op {op} to Function type"),
                    label: "Invalid Application".to_string()
                })
            ),
            (Minus, Bool(b)) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op Minus to Bool type (value) {b}"),
                    label: "Invalid Application".to_string()
                })
            ),
            (op, Null) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op {op} to Null type"),
                    label: "Invalid Application".to_string()
                })
            ),
            (op, List(l)) => Err(
                Box::new(RuntimeError {
                    src: Arc::from("... code here".to_string()),
                    span: (0..2).into(),
                    message: "Cannot do operand here".to_string(),
                    help: format!("Invalid application of unary op {op} to List type"),
                    label: "Invalid Application".to_string()
                })
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
