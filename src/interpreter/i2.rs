use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use cowvert::Data;
use std::fmt::{Debug, Formatter};
use std::{fmt, mem};
use std::collections::HashMap;
use aplang_lib::parser::ast::BinaryOp;
use crate::parser::ast::BinaryOp::{EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, Plus, Slash, Star};
use crate::interpreter::env2::{Env, EnvRef, Environment, Layer};
use crate::interpreter::env2::ValueRef;
use crate::interpreter::errors::Error;
use crate::interpreter::v2::{Object, SmartClone, Value};
use crate::parser::ast::{Destructor, Variable, Grouping, RepeatTimes, Access, Assignment, Ast, Binary, Block, Continue, Expr, ExprLiteral, ForEach, If, Import, List, Literal, Logical, ProcCall, ProcDeclaration, RepeatUntil, Return, Set, Stmt, Unary, Break, LogicalOp};

type IResult<T> = Result<T, Box<Error>>;

// #[derive(Debug)]
enum Flow {
    Normal(ValueRef),
    Return(ValueRef),
    Break,    // broke out of loop
    Continue, // continued loop iteration
}

impl Debug for Flow {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Flow::Normal(d) => {
                d.with(|v| {
                    write!(f, "Normal({})", v)
                })
            },
            Flow::Return(d) => {
                d.with(|v| {
                    write!(f, "Return({})", v)
                })
            },
            Flow::Break => write!(f, "Break"),
            Flow::Continue => write!(f, "Continue"),
        }
    }
}

impl Default for Flow {
    fn default() -> Self {
        Flow::Normal(Data::value(Value::Null))
    }
}

pub struct Interpreter {
    env: EnvRef,
    
    file_path: Option<PathBuf>,
    ast: Ast,
    
    // modules: Modules
}

impl Interpreter {
    pub fn new(ast: Ast, file_path: Option<PathBuf>) -> Self {
        let env = Environment::new();

        Self {
            env,
            file_path,
            ast,
        }
    }

    pub fn execute(&mut self) -> IResult<()> {
        let program = mem::take(&mut self.ast.program);

        for ref stmt in program {
            let s = self.stmt(stmt)?;
            println!("{:?}", s);
        };

        Ok(())
    }
}

/// Stmt
impl Interpreter {
    fn stmt(&mut self, stmt: &Stmt) -> IResult<Flow> {
        match stmt {
            Stmt::Expr(expr) => Ok(Flow::Normal(self.expr(expr)?)),
            Stmt::Block(block) => self.block_stmt(block),
            Stmt::If(ifs) => self.if_stmt(ifs),
            Stmt::RepeatTimes(repeat_times) => self.repeat_times_stmt(repeat_times),
            Stmt::RepeatUntil(repeat_until) => self.repeat_until_stmt(repeat_until),
            Stmt::ForEach(for_each) => self.for_each_stmt(for_each),
            Stmt::ProcDeclaration(proc_decl) => self.proc_decl_stmt(proc_decl),
            Stmt::Return(ret) => self.ret_stmt(ret),
            Stmt::Continue(cont) => self.cont_stmt(cont),
            Stmt::Import(import) => self.import_stmt(import),
            Stmt::Break(brk) => self.break_stmt(brk),
            Stmt::Destructure(de) => self.destructure_stmt(de),
        }
    }

    // { <stmts> }
    fn block_stmt(&mut self, block_stmt: &Arc<Block>) -> IResult<Flow> {
        self.env = self.env.layer();

        for stmt in block_stmt.statements.iter() {
            let flow = self.stmt(stmt)?;

            match flow {
                Flow::Normal(_) => { /* nop */ },
                _ => return Ok(flow),
            }
        }
        
        self.env = self.env.clone().scrape();

        Ok(Flow::default())
    }

    // IF <cond> { ]
    fn if_stmt(&mut self, if_stmt: &Arc<If>) -> IResult<Flow> {
        if self.expr(&if_stmt.condition)?.borrow().deref().is_truthy() {
            self.stmt(&if_stmt.then_branch)
        } else if let Some(else_branch) = &if_stmt.else_branch {
            self.stmt(else_branch)
        } else {
            Ok(Flow::default())
        }
    }

    /// REPEAT <expr> TIMES { }
    fn repeat_times_stmt(&mut self, repeat_times: &Arc<RepeatTimes>) -> IResult<Flow> {
        let mut binding = self.expr(&repeat_times.count)?;
        let binding = binding.borrow();
        let count = binding.deref();
        
        // error if type cannot become a number
        let count = match count {
            Value::Number(n) => *n as u64,
            value => {
                return Err(Error::todo())
            },
        };
        
        for _ in 1..=count {
            let flow = self.stmt(&repeat_times.body)?;
            
            match flow {
                Flow::Normal(_) => {/* nop */},
                Flow::Continue => continue,
                Flow::Break => break,
                Flow::Return(r) => return Ok(Flow::Return(r))
            }
        }
        
        Ok(Flow::default())
    }

    // REPEAT UNTIL ( <cond> ) { }
    fn repeat_until_stmt(&mut self, repeat_until: &Arc<RepeatUntil>) -> IResult<Flow> {
        while self.expr(&repeat_until.condition)?.borrow().deref().is_truthy() {
            let flow = self.stmt(&repeat_until.body)?;
            
            match flow {
                Flow::Normal(_) => {/* nop */},
                Flow::Continue => continue,
                Flow::Break => break,
                Flow::Return(r) => return Ok(Flow::Return(r))
            }
        }
        
        Ok(Flow::default())
    }

    // FOR EACH <item> IN <iter> { }
    fn for_each_stmt(&mut self, for_each_stmt: &Arc<ForEach>) -> IResult<Flow> {
        let mut list = self.expr(&for_each_stmt.list)?;

        let binding = list.borrow();

        let temp_storage: Vec<Data<Value>>;
        let iter: Box<dyn Iterator<Item = &Data<Value>>> = match binding.deref() {
            Value::List(list) => {
                Box::new(list.iter())
            },
            Value::String(s) => {
                let vec: Vec<Data<Value>> = s
                    .chars()
                    .map(|ch| Data::value(Value::String(ch.to_string())))
                    .collect();
                temp_storage = vec;
                Box::new(temp_storage.iter())
            },
            Value::Object(obj) => {
                if let Some(iter) = obj.iter() {
                    iter
                } else {
                    return Err(Error::todo())
                }
            },
            _ => return Err(Error::todo()),
        };

        for item in iter {
        }

        Ok(Flow::default())
    }


    // EXPORT? PROCEDURE <name>( <params> ) { }
    fn proc_decl_stmt(&mut self, proc_decl_stmt: &Arc<ProcDeclaration>) -> IResult<Flow> {
        todo!()
    }

    // RETURN
    fn ret_stmt(&mut self, ret_stmt: &Arc<Return>) -> IResult<Flow> {
        if let Some(return_value) = &ret_stmt.data {
            Ok(Flow::Return(self.expr(return_value)?))
        } else {
            Ok(Flow::Return(Data::value(Value::Null)))
        }
    }

    // CONTINUE
    fn cont_stmt(&mut self, cont_stmt: &Arc<Continue>) -> IResult<Flow> {
        Ok(Flow::Continue)
    }

    // BREAK
    fn break_stmt(&mut self, break_stmt: &Arc<Break>) -> IResult<Flow> {
        Ok(Flow::Break)
    }

    // IMPORT
    fn import_stmt(&mut self, import_stmt: &Arc<Import>) -> IResult<Flow> {
        todo!()
    }

    // [ <binds> ] <- <expr>
    fn destructure_stmt(&mut self, destructor: &Arc<Destructor>) -> IResult<Flow> {
        let mut list = self.expr(&destructor.right)?;
        let mut list = list.borrow_mut();
        eprintln!("bindings: {:?}", destructor.bindings);
        eprintln!("list: {}", list.deref());
        
        let mut temp_storage: Option<Vec<Data<Value>>> = None;
        let mut_iter: Box<dyn ExactSizeIterator<Item = &mut Data<Value>>> = match list.deref_mut() {
            Value::List(list) => {
                Box::new(list.iter_mut())
            },
            Value::String(s) => {
                // convert the string to a vector of Data<Value>
                let vec: Vec<Data<Value>> = s
                    .chars()
                    .map(|ch| Data::value(Value::String(ch.to_string())))
                    .collect();
                temp_storage = Some(vec);
                // safe to unwrap since we just set it
                Box::new(temp_storage.as_mut().unwrap().iter_mut())
            },
            _ => return Err(Error::todo()),
        };
        
        if mut_iter.len() != destructor.bindings.len() {
            return Err(Error::todo())
        }
        
        // eprintln!()

        for (binding, value) in destructor.bindings.iter().zip(mut_iter) {
            let Some(binding) = binding else { continue };
            self.env.define(&binding.ident, value.by_ref());
        }
        
        Ok(Flow::default())
    }

}

/// Expr
impl Interpreter {
    fn expr(&mut self, expr: &Expr) -> IResult<ValueRef> {
        match expr {
            Expr::Literal(literal) => self.literal_expr(literal),
            Expr::Binary(binary) => self.binary_expr(binary),
            Expr::Unary(unary) => self.unary_expr(unary),
            Expr::Grouping(grouping) => self.grouping_expr(grouping),
            Expr::Logical(logical) => self.logical_expr(logical),
            Expr::Variable(var) => self.variable_expr(var),
            Expr::ProcCall(call) => self.call_expr(call),
            Expr::Access(access) => self.access_expr(access),
            Expr::List(list) => self.list_expr(list),
            Expr::Assign(assign) => self.assign_expr(assign),
            Expr::Set(set) => self.set_expr(set),
        }
    }

    fn literal_expr(&mut self, literal_expr: &Arc<ExprLiteral>) -> IResult<ValueRef> {
        let lit = match &literal_expr.value {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()), // value clone here
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Null => Value::Null,
        };
        
        Ok(Data::value(lit))
    }
    
    fn binary_expr(&mut self, binary: &Arc<Binary>) -> IResult<ValueRef> {
        use crate::interpreter::v2::Value::*;
        use crate::parser::ast::BinaryOp::*;
        let value = Data::value;
    
        let mut lhs_binding = self.expr(&binary.left)?;
        let mut rhs_binding = self.expr(&binary.right)?;
        
        rhs_binding.try_with_other_mut(&mut lhs_binding, |lhs, rhs| {
            Ok(match (lhs, rhs) {
                // normal case
                (mut a, Some(mut b)) => match (a.deref_mut(), &binary.operator, b.deref_mut()) {
                    // comparison
                    (a, EqualEqual, b) => value(Bool(todo!())),
                    (a, NotEqual, b) => value(Bool(todo!())),
                    (Number(a), Less, Number(b)) => value(Bool(a < b)),
                    (Number(a), LessEqual, Number(b)) => value(Bool(a <= b)),
                    (Number(a), Greater, Number(b)) => value(Bool(a > b)),
                    (Number(a), GreaterEqual, Number(b)) => value(Bool(a >= b)),

                    // arithmatic
                    (Number(a), Plus, Number(b)) => value(Number(*a + *b)),
                    (Number(a), Minus, Number(b)) => value(Number(*a - *b)),
                    (Number(a), Star, Number(b)) => value(Number(*a * *b)),
                    (&mut Number(a), Slash, &mut Number(b)) => {
                        if b != 0.0 {
                            value(Number(a / b))
                        } else {
                            return Err(Error::todo())
                        }
                    }
                    (&mut Number(a), Modulo, &mut Number(b)) => {
                        if b != 0.0 {
                            value(Number(a % b))
                        } else {
                            return Err(Error::todo())
                        }
                    }

                    // string
                    (String(a), Plus, b) => value(String(format!("{a}{b}"))),
                    // list
                    (List(a), Plus, List(b)) => {
                        let new: Vec<_> = a.iter_mut()
                            .map(|v| v.smart_clone())
                            .chain(b.iter_mut().map(|v| v.smart_clone()))
                            .collect();

                        value(List(new))
                    }

                    _ => return Err(Error::todo())
                },
                
                // a is b
                (mut v, None) => {
                    match (v.deref(), &binary.operator) {
                        // comparison
                        (_, EqualEqual) => value(Bool(true)),
                        (_, NotEqual) => value(Bool(false)),
                        (Number(_), LessEqual) => value(Bool(true)),
                        (Number(_), GreaterEqual) => value(Bool(true)),
                        (Number(_), Less) => value(Bool(false)),
                        (Number(_), Greater) => value(Bool(false)),

                        // arithmatic
                        (Number(a), Plus) => value(Number(*a + *a)),
                        (Number(a), Minus) => value(Number(*a - *a)),
                        (Number(a), Star) => value(Number(*a * *a)),
                        (Number(a), Slash) => {
                            if *a != 0.0 {
                                value(Number(1.0))
                            } else {
                                return Err(Error::todo())
                            }
                        }
                    }
                }
            })
        }).expect("lhs is already being borrowed, unrecoverable")
    }

    fn unary_expr(&mut self, unary: &Arc<Unary>) -> IResult<ValueRef> {
        use crate::interpreter::v2::Value::*;
        use crate::parser::ast::UnaryOp::*;

        let mut operand_binding = self.expr(&unary.right)?;
        let operand = operand_binding.borrow();
        Ok(match (&unary.operator, operand.deref()) {
            (Minus, Number(num)) => Data::value(Number(-num)),
            (Not, value) => Data::value(Bool(!value.is_truthy())),

            // todo add specific errors here
            _ => return Err(Error::todo())
        })
    }

    fn grouping_expr(&mut self, grouping_expr: &Arc<Grouping>) -> IResult<ValueRef> {
        self.expr(&grouping_expr.expr)
    }

    fn logical_expr(&mut self, logical_expr: &Arc<Logical>) -> IResult<ValueRef> {
        let mut left = self.expr(&logical_expr.left)?;
        let short_circuit = match logical_expr.operator {
            LogicalOp::Or => left.borrow().deref().is_truthy(),
            LogicalOp::And => !left.borrow().deref().is_truthy(),
        };

        if short_circuit {
            Ok(left)
        } else {
            Ok(self.expr(&logical_expr.right)?)
        }
    }
    
    fn variable_expr(&mut self, variable_expr: &Arc<Variable>) -> IResult<ValueRef> {
        if let Some(var) = self.env.get_ref(&*variable_expr.ident) {
            Ok(var)
        } else {
            Err(Error::todo())
        }
    }
    fn call_expr(&mut self, call_expr: &Arc<ProcCall>) -> IResult<ValueRef> {
        let args: Vec<ValueRef> = call_expr
            .arguments
            .iter()
            .map(|arg| self.expr(arg))
            .collect::<Result<_, _>>()?;
        
        let arg_strings: Vec<String> = args
            .iter()
            .map(|val| {
                let mut s = String::new();
                val.with(|x| s = format!("{x}"));
                s
            })
            .collect();

        println!("{}", arg_strings.join(", "));

        Ok(Data::value(Value::Null))
    }

    fn access_expr(&mut self, access_expr: &Arc<Access>) -> IResult<ValueRef> {
        let mut list = self.expr(&access_expr.list)?;
        
        let mut key = self.expr(&access_expr.key)?;
        let key = key.borrow();
        let key = key.deref();
        let Value::Number(key) = key else {
            return Err(Error::todo())
        };
        
        let key = if *key < 1.0 {
            // cannot index less than 1.0
            return Err(Error::todo())
        } else {
            // index starting at 1
            (key - 1.0) as usize
        };
        
        let mut list = list.borrow_mut();
        let list = list.deref_mut();
        match list {
            Value::List(list) => {
                if let Some(elm) = list.get_mut(key) {
                    Ok(elm.by_ref())
                } else {
                    // out of bounds
                    Err(Error::todo())
                }
            },
            Value::String(s) => {
                if let Some(ch) = s.chars().nth(key) {
                    Ok(Data::value(Value::String(ch.to_string())))
                } else {
                    // out of bounds
                    Err(Error::todo())
                }
            },
            _ => {
                Err(Error::todo())
            }
        }
    }

    fn list_expr(&mut self, list_expr: &Arc<List>) -> IResult<ValueRef> {
        let mut list = Vec::with_capacity(list_expr.items.len());
        
        for expr in list_expr.items.iter() {
            list.push(self.expr(expr)?);
        }

        Ok(Data::value(Value::List(list)))
    }
    
    fn assign_expr(&mut self, assign_expr: &Arc<Assignment>) -> IResult<ValueRef> {
        let mut result = self.expr(&assign_expr.value)?;
        let handle = result.by_ref();
        
        let _exists = self.env.define(&assign_expr.target.ident, result);
        
        // also return a ref to the value
        // this maybe should be a Cow?
        Ok(handle)
    }
    
    fn set_expr(&mut self, set_expr: &Arc<Set>) -> IResult<ValueRef> {
        // get the list
        let mut list = self.expr(&set_expr.list)?;
        
        let mut list_binding = list.borrow_mut();
        let Value::List(list) = list_binding.deref_mut() else  {
            return Err(Error::todo())
        };

        // get the key
        let mut key = self.expr(&set_expr.key)?;
        let key = key.borrow();
        let key = key.deref();

        let Value::Number(key) = key else {
            return Err(Error::todo())
        };

        let key = if *key < 1.0 {
            // cannot index less than 1.0
            return Err(Error::todo())
        } else {
            // index starting at 1
            (key - 1.0) as usize
        };

        if key >= list.len() {
            // out of bounds
            return Err(Error::todo())
        }

        // get the value
        let value = self.expr(&set_expr.value)?;
        let Some(elm) = list.get_mut(key) else {
            return Err(Error::todo())
        };
        let handle = elm.by_ref();

        // set the value
        *elm = value;
        
        Ok(handle)
    }
}

#[cfg(test)]
mod interpreter_tests {
    use crate::aplang::ApLang;

    #[test]
    fn run_new_interpreter() {
        // A simple source code snippet for testing.
        let source = include_str!("../../examples.ap/test.ap");

        // Create a new ApLang instance from source.
        let lang = ApLang::new_from_stdin(source);

        // Run the lexing phase.
        let lexed = lang.lex().expect("Lexing failed");

        // Run the parsing phase.
        let parsed = lexed.parse().expect("Parsing failed");

        // Execute the code (using the new interpreter in the execution phase).
        parsed.execute_dev().expect("Execution failed");
    }
}