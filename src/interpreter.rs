use crate::ast::BinaryOp::{
    EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, NotEqual, Plus, Star,
};
use crate::ast::Expr::List;
use crate::ast::LogicalOp::Or;
use crate::ast::{Ast, Binary, BinaryOp, Expr, Literal, LogicalOp, Stmt, Unary, UnaryOp, Variable};
use crate::interpreter::Value::{Bool, Null};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::format;
use std::mem;
use std::ops::Deref;
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
struct Env {
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
    pub fn layer(&mut self) {
        self.venv.push(Context::default())
    }

    pub fn enclosing_layer(&mut self) {
        let enclosing = self.activate().clone();
        self.venv.push(enclosing)
    }
    
    pub fn merge_down(&mut self) {
        let context = self.scrape();
        
        *self.activate() = context;
    }

    pub fn scrape(&mut self) -> Context {
        self.venv
            .pop()
            .expect("attempted to remove context but failed")
    }

    fn activate(&mut self) -> &mut Context {
        let len = self.venv.len();
        &mut self.venv[len - 1]
    }

    pub fn define(&mut self, variable: Arc<Variable>, value: Value) {
        // add the variable into the context
        self.activate()
            .variables
            .insert(variable.ident.clone(), (value, variable));
    }

    pub fn lookup_name(&mut self, var: &str) -> Result<&(Value, Arc<Variable>), String> {
        self.activate()
            .variables
            .get(var)
            .ok_or("could not find variable".to_string())
    }

    pub fn lookup_var(&mut self, var: &Variable) -> Result<&Value, String> {
        Ok(&self.lookup_name(var.ident.as_str())?.0)
    }
    
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
        let mut env = Self {
            venv: vec![],
        };
        // push the base context layer into env so we dont panic
        env.layer();
        env
    }
}

pub struct Interpreter {
    venv: Env,
    ast: Ast,
}

impl Interpreter {
    pub fn new(ast: Ast) -> Self {
        Self {
            venv: Env::default(),
            ast,
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
                stmt => self.stmt(stmt)?
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
                    },
                    value => Err(format!("cannot do count for value {value:?}"))
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
                    Value::String(string) => string.chars().map(|ch| Value::String(ch.to_string())).collect(),
                    value=> Err(format!("cannot make iterator over value {value:?}"))?
                };

                let element = Arc::new(for_each.item.clone());
                
                // if the variable already exists temperately remove it so doesn't get lost
                let maybe_cached= self.venv.remove(element.clone());

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
            // Stmt::ProcDeclaration(_) => {}
            // Stmt::Return(_) => {}

            Stmt::Block(block) => {
                self.venv.enclosing_layer();

                for stmt in block.statements.iter() {
                    self.stmt(stmt)?
                }
                
                self.venv.merge_down();

                Ok(())
            },
            s => {
                println!("{s:#?}");
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
            ProcCall(_) => todo!(),
            Access(_) => todo!(),
            List(list) => self.list(list.as_ref()),
            Variable(v) => self.venv.lookup_var(v.as_ref()).cloned(),
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

    // help: a string can be thought of a list of chars
    fn list(&mut self, list: &crate::ast::List) -> Result<Value, String> {
        list.items.iter()
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
