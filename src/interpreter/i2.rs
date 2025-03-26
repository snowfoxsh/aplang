use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use cowvert::Data;
use crate::interpreter::env2::{EnvRef, Layer};
use crate::interpreter::env2::ValueRef;
use crate::interpreter::errors::Error;
use crate::interpreter::v2::{Object, Value};
use crate::parser::ast::{Destructor, Variable, Grouping, RepeatTimes, Access, Assignment, Ast, Binary, Block, Continue, Expr, ExprLiteral, ForEach, If, Import, List, Literal, Logical, ProcCall, ProcDeclaration, RepeatUntil, Return, Set, Stmt, Unary, Break};

// #[derive(Debug)]
enum Flow {
    // Normal(ValueRef),
    Normal(ValueRef),
    Return(ValueRef),
    Break,    // broke out of loop
    Continue, // continued loop iteration
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

/// Stmt
impl Interpreter {
    fn stmt(&mut self, stmt: &Stmt) -> Result<Flow, Error> {
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
    fn block_stmt(&mut self, block_stmt: &Arc<Block>) -> Result<Flow, Error> {
        self.env = self.env.layer();

        for stmt in block_stmt.statements.iter() {
            let flow = self.stmt(stmt)?;

            match flow {
                Flow::Normal(_) => { /* nop */ },
                _ => return Ok(flow),
            }
        }
        
        self.env = self.env.scrape();

        Ok(Flow::default())
    }

    // IF <cond> { ]
    fn if_stmt(&mut self, if_stmt: &Arc<If>) -> Result<Flow, Error> {
        if self.expr(&if_stmt.condition)?.borrow().deref().is_truthy() {
            self.stmt(&if_stmt.then_branch)
        } else if let Some(else_branch) = &if_stmt.else_branch {
            self.stmt(else_branch)
        } else {
            Ok(Flow::default())
        }
    }

    /// REPEAT <expr> TIMES { }
    fn repeat_times_stmt(&mut self, repeat_times: &Arc<RepeatTimes>) -> Result<Flow, Error> {
        let count = self.expr(&repeat_times.count)?.borrow().deref();
        
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
    fn repeat_until_stmt(&mut self, repeat_until: &Arc<RepeatUntil>) -> Result<Flow, Error> {
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

    // FOR EACH <var> IN <list> { }
    fn for_each_stmt(&mut self, for_each_stmt: &Arc<ForEach>) -> Result<Flow, Error> {
        let list = self.expr(&for_each_stmt.list)?;

        let list = match list.borrow().deref() {
            Value::List(list) => list.iter(),
            Value::String(s) =>
                s.chars()
                .map(|ch| Data::value(Value::String(ch.to_string())))
                .collect::<Vec<_>>()
                .iter(),
            Value::Object(obj) => {
                if let Some(iter) = obj.iter() {
                    return iter
                } else {
                    return Err(Error::todo())
                }
            },
            _ => return Err(Error::todo()),
        };

        Ok(Flow::default())
    }

    fn proc_decl_stmt(&mut self, proc_decl_stmt: &Arc<ProcDeclaration>) -> Result<Flow, Error> {
        todo!()
    }

    fn ret_stmt(&mut self, ret_stmt: &Arc<Return>) -> Result<Flow, Error> {
        if let Some(return_value) = &ret_stmt.data {
            Ok(Flow::Return(self.expr(return_value)?))
        } else {
            Ok(Flow::Return(Data::value(Value::Null)))
        }
    }

    fn cont_stmt(&mut self, cont_stmt: &Arc<Continue>) -> Result<Flow, Error> {
        Ok(Flow::Continue)
    }
    
    fn break_stmt(&mut self, break_stmt: &Arc<Break>) -> Result<Flow, Error> {
        Ok(Flow::Break)
    }

    fn import_stmt(&mut self, import_stmt: &Arc<Import>) -> Result<Flow, Error> {
        todo!()
    }

    fn destructure_stmt(&self, p0: &Arc<Destructor>) -> Result<Flow, Error> {
        todo!()
    }
}

/// Expr
impl Interpreter {
    fn expr(&mut self, expr: &Expr) -> Result<ValueRef, Error> {
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

    fn literal_expr(&mut self, literal_expr: &Arc<ExprLiteral>) -> Result<ValueRef, Error> {
        let lit = match &literal_expr.value {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()), // value clone here
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Null => Value::Null,
        };
        
        Ok(Data::value(lit))
    }
    
    fn binary_expr(&mut self, binary: &Arc<Binary>) -> Result<ValueRef, Error> {
        use crate::interpreter::v2::Value::*;
        use crate::parser::ast::BinaryOp::*;
        
        let lhs = self.expr(&binary.left)?;
        let rhs = self.expr(&binary.right)?;
        
        match (lhs, &binary.operator, rhs) {
            (_, EqualEqual, _) => Ok(Data::value())
        }
    }

    fn unary_expr(&mut self, unary: &Arc<Unary>) -> Result<ValueRef, Error> {
        todo!()
    }

    fn grouping_expr(&mut self, grouping_expr: &Arc<Grouping>) -> Result<ValueRef, Error> {
        self.expr(&grouping_expr.expr)
    }

    fn logical_expr(&mut self, logical_expr: &Arc<Logical>) -> Result<ValueRef, Error> {
        todo!()
    }

    
    fn variable_expr(&mut self, variable_expr: &Arc<Variable>) -> Result<ValueRef, Error> {
        todo!()
    }

    fn call_expr(&mut self, call_expr: &Arc<ProcCall>) -> Result<ValueRef, Error> {
        todo!()
    }

    fn access_expr(&mut self, access_expr: &Arc<Access>) -> Result<ValueRef, Error> {
        todo!()
    }

    fn list_expr(&mut self, list_expr: &Arc<List>) -> Result<ValueRef, Error> {
        todo!()
    }
    
    fn assign_expr(&mut self, assign_expr: &Arc<Assignment>) -> Result<ValueRef, Error> {
        todo!()
    }
    
    fn set_expr(&mut self, set_expr: &Arc<Set>) -> Result<ValueRef, Error> {
        todo!()
    }
}