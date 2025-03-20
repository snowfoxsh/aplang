use std::path::PathBuf;
use std::sync::Arc;
use crate::interpreter::env2::EnvRef;
use crate::interpreter::env2::ValueRef;
use crate::interpreter::errors::Error;
use crate::parser::ast::{Ast, Stmt};

// #[derive(Debug)]
enum Flow {
    Normal(ValueRef),
    Return(ValueRef),
    Break,    // broke out of loop
    Continue, // continued loop iteration
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
        match self {
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::If(if_stmt) => self.if_stmt(if_stmt),
            _ => todo!(),
        }
        todo!()
    }
    
    fn if_stmt(&mut self, if_stmt: &mut Arc<Stmt::If>) -> Flow {
        todo!()
    }
}

/// Expr
impl Interpreter {
    fn expr(&mut self, expr) -> Flow {
        todo!()
    }
}