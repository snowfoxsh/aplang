use crate::ast::BinaryOp::{
    EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, NotEqual, Plus, Star,
};
use crate::ast::Expr::List;
use crate::ast::LogicalOp::Or;
use crate::ast::{Ast, Binary, BinaryOp, Expr, Literal, LogicalOp, ProcCall, ProcDeclaration, Stmt, Unary, UnaryOp, Variable};
use crate::interpreter::Value::{Bool, Null};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{format};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::Arc;

// variable value types
#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    NativeFunction(),
    Function(),
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
                // (param.clone(), arg)
            });
        
        // execute the function
        interpreter.stmt(&self.body)?;
        
        let retval = interpreter.ret_val.clone();
        // todo implement backtrace
        interpreter.ret_val = cached_retval;
        
        match retval {
            None => Ok(Value::Null),
            Some(value) =>Ok(value),
        }
    }

    fn arity(&self) -> u8 {
        self.params.len().try_into().unwrap()
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
struct Env {
    functions: HashMap<String, (Rc<dyn Callable>, Option<Arc<ProcDeclaration>>)>,
    //                |^^^^^^  |^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^|> Maybe pointer to function def
    //                |        |                                                  If None: it is native function
    //                |        |> Pointer to the function
    //                |> Function name (symbol)
    venv: Vec<Context>,
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
        env
    }
}

#[derive(Clone)]
pub struct Interpreter {
    venv: Env,
    ast: Ast,
    ret_val: Option<Value>,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        Self {
            venv: Env::default(),
            ast,
            ret_val: None,
        }
    }

    pub fn interpret_debug(&mut self) -> Result<Vec<Value>, String> {
        let mut values = vec![];

        let program = mem::take(&mut self.ast.program); // Temporarily take the program
        for stmt in &program {
            // self.stmt(stmt)?; // Process each statement.

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
                let values: Vec<Value> = match self.expr(&for_each.list)? {
                    Value::List(list) => list,
                    Value::String(string) => string
                        .chars()
                        .map(|ch| Value::String(ch.to_string()))
                        .collect(),
                    value => Err(format!("cannot make iterator over value {value:?}"))?,
                };

                let element = Arc::new(for_each.item.clone());

                // if the variable already exists temperately remove it so doesn't get lost
                let maybe_cached = self.venv.remove(element.clone());

                for value in values {
                    // add the variable to the context for this block
                    self.venv.define(element.clone(), value);
                    self.stmt(&for_each.body)?; // run the block
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
            s => {
                todo!()
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
            Access(_) => todo!(),
            List(list) => self.list(list.as_ref()),
            Variable(v) => self.venv.lookup_name(v.ident.clone().as_str()).cloned().map(|(value, _)| value),
            Assign(assignment) => {
                // assign to variable
                let result = self.expr(&assignment.value)?;
                self.venv.define(assignment.target.clone(), result.clone());
                Ok(result)
            }
            Set(set) => todo!(),
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
            }
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
            .collect::<Result<Vec<_>, _>>()
            .map(Value::List)
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
                Ok(List(a.iter().cloned().chain(b.iter().cloned()).collect()))
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

//
//
// do nothing
fn by_value(mut thing: i32) {
    thing += 1;
}

//
fn by_ref(thing: &mut i32) {
    *thing += 1;
}