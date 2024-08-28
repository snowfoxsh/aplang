use std::cell::{RefCell};
use crate::ast::BinaryOp::{
    EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, NotEqual, Plus, Star,
};
use crate::ast::Expr::List;
use crate::ast::{Ast, Binary, BinaryOp, Expr, Literal, LogicalOp, ProcCall, ProcDeclaration, Stmt, Unary, UnaryOp, Variable};
use crate::aplang_std;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::mem;
use std::ops::Deref;
use std::slice::Iter;
use std::process::id;
use std::rc::Rc;
use std::sync::Arc;
use miette::miette;

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
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, String>;
    fn arity(&self) -> u8;
}

pub struct Procedure {
    pub name: String,
    pub params: Vec<Variable>,
    pub body: Stmt,
}

impl Callable for Procedure {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, String> {
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
    pub callable: fn(&mut Interpreter, &[Value]) -> Result<Value, String>
}

impl Callable for NativeProcedure {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, String> {
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
    pub fn lookup_name(&mut self, var: &str) -> Result<&(Value, Arc<Variable>), String> {
        self.activate()
            .variables
            .get(var)
            .ok_or("could not find variable".to_string())
    }


    /// looks up the variable by comparing the entire variable object
    pub fn lookup_var(&mut self, var: &Variable) -> Result<&Value, String> {
        Ok(&self.lookup_name(var.ident.as_str())?.0)
    }

    pub fn lookup_function(&self, function_name: String) -> Result<Rc<dyn Callable>, String> {
        let (a, b) = self.functions.get(&function_name).ok_or("could not find function".to_string())?.clone();
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
        // todo: push the std native function on here
        // push the base context layer into env so we dont panic
        env.initialize_empty_scope();
        env.inject_std_default();
        env
    }
    
}

#[derive(Clone)]
pub struct Interpreter {
    venv: Env,
    ast: Ast,
    ret_val: Option<Value>,
    idx: usize,
    program: Vec<Stmt>,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        Self {
            venv: Env::default(),
            ast,
            ret_val: None,
            idx: 0,
            program: Default::default()
        }
    }

    pub fn interpret_debug(&mut self) -> Result<Vec<Value>, String> {
        let mut values = vec![];

        self.program = self.ast.program.clone(); // todo: get rid of the clone here somehow
        let program = mem::take(&mut self.ast.program); // Temporarily take the program

        for (i, stmt) in program.iter().enumerate() {
            self.idx = i;

            match stmt {
                Stmt::Expr(expr) => {
                    let value = self.expr(expr.deref())?;
                    values.push(value);
                }
                stmt => self.stmt(stmt)?,
            }
        }

        // self.iter = None;
        self.ast.program = program; // Restore the program
        Ok(values)
    }

    fn peek_next_stmt(&mut self) -> Option<&Stmt> {
        self.program.get(self.idx)
    }

    // a stmt by definition returns nothing
    fn stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
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
                    }
                    value => Err(format!("cannot do count for value {value:?}")),
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
                    value => Err(format!("cannot make iterator over value {value:?}"))?,
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
        }
    }

    pub fn interpret_expr_temp(&mut self) -> Result<Vec<Value>, String> {
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
    fn expr(&mut self, expr: &Expr) -> Result<Value, String> {
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
            Variable(v) => self.venv.lookup_name(v.ident.clone().as_str()).cloned().map(|(value, _)| value),
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

    fn call(&mut self, proc: &ProcCall) -> Result<Value, String> {
        // todo: look into callee expr

        // run the argument expressions before the actual call
        let argument_evaluations: Result<Vec<_>, _> = proc.arguments // todo: figure out why this type conversion works
            .iter()
            .map(|arg| self.expr(arg)) // todo write a better error message here
            .collect();

        let Ok(argument_evaluations) = argument_evaluations else {
            // todo: write better error message -- use individual expr source pointers
            return Err(
                "could not evaluate arguments".to_string()
            )
        };

        let callable = self.venv.lookup_function(proc.ident.clone())?;

        // todo make the source pointer error message better

        if callable.arity() as usize != argument_evaluations.len() {
            return Err("function called with incorrect number of args".to_string()) // todo make this error message better -- use source proc pointer
        }

        callable.call(self, argument_evaluations.as_ref())
    }

    // help: a string can be thought of a list of chars
    fn list(&mut self, list: &crate::ast::List) -> Result<Value, String> {
        list.items
            .iter()
            .map(|expr: &Expr| self.expr(expr))
            .collect::<Result<Vec<Value>, String>>()
            .map(|x|Value::List(RefCell::new(x).into()))
    }

    fn access(&mut self, access: &crate::ast::Access) -> Result<Value, String> {
        let list = self.expr(&access.list)?;
        let idx = self.expr(&access.key)?;

        // Access is only called when accessing an element in a list by index
        if let Value::List(ref l) = list {

            // Unwrapping the index and checking that it's a Number
            let Value::Number(idx) = idx else {
                // todo: make better error message
                return Err("Invalid Index. Index Must Be A Number".to_string());
            };

            // Checking if the next Statement is a Set()
            // If it's a set, we need to feed it the list and index we will be assigning the new value to
            if matches!(
                self.peek_next_stmt(),
                Some(Stmt::Expr(expr)) if matches!(expr.as_ref(), Expr::Set(_))
            ) {
                return Ok(Value::List(Rc::new(RefCell::new(vec![
                    Value::Number(idx),
                    list, // we disguise this essentially mutable copy as a List
                ]))));
            }

            // Returning the accessed value if it's not a set
            // We subtract the idx by 1 to account for lists being indexed at 1, not 0
            l.borrow().get((idx - 1.0) as usize).cloned().ok_or_else(|| "Index out of Bounds".to_string())
        } else {
            // todo: make better error message
            Err("Invalid types for access".to_string())
        }
    }

    fn set(&mut self, set: &crate::ast::Set) -> Result<Value, String> {
        let target = self.expr(&set.target)?;
        let value = self.expr(&set.value)?;

        // The previous statement to this should be Access, which returns what is now our target
        // All we need to do is unpack it
        match target {
            Value::List(list) if list.borrow().len() == 2 => {
                let list = list.borrow();
                if let (Value::Number(i), Value::List(ref l)) = (&list[0], &list[1]) {
                    // Mutably editing the list given to us
                    // Subtracting index by 1 to account for lists being indexed at 1, not 0
                    if let Some(t) = l.borrow_mut().get_mut((i - 1.0) as usize) {
                        *t = value;
                    } else {
                        return Err("Index out of Bounds".to_string())
                    }
                }
                Ok(Value::Bool(true)) // returning TRUE because target was set properly
            }
            _ => Err("Invalid types for set".to_string()),
        }
    }

    fn binary(&mut self, node: &Binary) -> Result<Value, String> {
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
                    Err("dev by zero error".to_string())
                }
            }
            (String(a), Plus, String(b)) => Ok(String(format!("{a}{b}"))),
            (List(a), Plus, List(b)) => {
                // adding two lists
                // todo: consider using try_borrow?
                let new_list: Vec<_> = a.borrow().iter().cloned().chain(b.borrow().iter().cloned()).collect();
                Ok(List(RefCell::new(new_list).into()))
            }
            _ => Err("invalid operands in binary op not equal".to_string()),
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
    fn unary(&mut self, node: &Unary) -> Result<Value, String> {
        let value = self.expr(&node.right)?;

        use UnaryOp::*;
        use Value::*;
        match (&node.operator, value) {
            (Minus, Number(num)) => Ok(Number(-num)),
            (Not, value) => Ok(Bool(!Self::is_truthy(&value))),
            (op, String(_)) => Err(format!(
                "Invalid application of unary op {op} to String type"
            )),
            (op, NativeFunction()) => Err(format!(
                "Invalid application of unary op {op} to NativeFunction type"
            )),
            (op, Function()) => Err(format!(
                "Invalid application of unary op {op} to Function type"
            )),
            (Minus, Bool(b)) => Err(format!(
                "Invalid application of unary op Minus to Bool type (value) {b}"
            )),
            (op, Null) => Err(format!("Invalid application of unary op {op} to Null type")),
            (op, List(l)) => Err(format!("Invalid application of unary op {op} to List type")),
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
