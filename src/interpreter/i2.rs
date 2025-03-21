use std::path::PathBuf;
use std::sync::Arc;
use crate::interpreter::env2::EnvRef;
use crate::interpreter::env2::ValueRef;
use crate::interpreter::errors::Error;
use crate::parser::ast::{Variable, Grouping, RepeatTimes, Access, Assignment, Ast, Binary, Block, Continue, Expr, ExprLiteral, ForEach, If, Import, List, Literal, Logical, ProcCall, ProcDeclaration, RepeatUntil, Return, Set, Stmt, Unary, Break};

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
        match stmt {
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::Block(block) => self.block(block),
            Stmt::If(ifs) => self.if_stmt(ifs),
            Stmt::RepeatTimes(repeat_times) => self.repeat_times_stmt(repeat_times),
            Stmt::RepeatUntil(repeat_until) => self.repeat_until_stmt(repeat_until),
            Stmt::ForEach(for_each) => self.for_each_stmt(for_each),
            Stmt::ProcDeclaration(proc_decl) => self.proc_decl_stmt(proc_decl),
            Stmt::Return(ret) => self.ret_stmt(ret),
            Stmt::Continue(cont) => self.cont_stmt(cont),
            Stmt::Import(import) => self.import_stmt(import),
            Stmt::Break(brk) => self.break_stmt(brk),
        }
    }

    fn block(&mut self, block_stmt: &Arc<Block>) -> Result<Flow, Error> {
        todo!()
    }

    fn if_stmt(&mut self, if_stmt: &Arc<If>) -> Result<Flow, Error> {
        todo!()
    }

    fn repeat_times_stmt(&mut self, repeat_times_stmt: &Arc<RepeatTimes>) -> Result<Flow, Error> {
        todo!()
    }

    fn repeat_until_stmt(&mut self, repeat_until_stmt: &Arc<RepeatUntil>) -> Result<Flow, Error> {
        todo!()
    }

    fn for_each_stmt(&mut self, for_each_stmt: &Arc<ForEach>) -> Result<Flow, Error> {
        todo!()
    }

    fn proc_decl_stmt(&mut self, proc_decl_stmt: &Arc<ProcDeclaration>) -> Result<Flow, Error> {
        todo!()
    }

    fn ret_stmt(&mut self, ret_stmt: &Arc<Return>) -> Result<Flow, Error> {
        todo!()
    }

    fn cont_stmt(&mut self, cont_stmt: &Arc<Continue>) -> Result<Flow, Error> {
        todo!()
    }
    
    fn break_stmt(&mut self, break_stmt: &Arc<Break>) -> Result<Flow, Error> {
        todo!()
    }

    fn import_stmt(&mut self, import_stmt: &Arc<Import>) -> Result<Flow, Error> {
        todo!()
    }
}

/// Expr
impl Interpreter {
    fn expr(&mut self, expr: &Expr) -> Result<Flow, Error> {
        match expr {
            Expr::Binary(binary) => self.binary_expr(binary),
            Expr::Unary(unary) => self.unary_expr(unary),
            Expr::Grouping(grouping) => self.grouping_expr(grouping),
            Expr::Logical(logical) => self.logical_expr(logical),
            Expr::Literal(literal) => self.literal_expr(literal),
            Expr::Variable(var) => self.variable_expr(var),
            Expr::ProcCall(call) => self.call_expr(call),
            Expr::Access(access) => self.access_expr(access),
            Expr::List(list) => self.list_expr(list),
            Expr::Assign(assign) => self.assign_expr(assign),
            Expr::Set(set) => self.set_expr(set),
        }
    }

    fn binary_expr(&mut self, binary: &Arc<Binary>) -> Result<Flow, Error> {
        todo!()
    }

    fn unary_expr(&mut self, unary: &Arc<Unary>) -> Result<Flow, Error> {
        todo!()
    }

    fn grouping_expr(&mut self, grouping_expr: &Arc<Grouping>) -> Result<Flow, Error> {
        todo!()
    }

    fn logical_expr(&mut self, logical_expr: &Arc<Logical>) -> Result<Flow, Error> {
        todo!()
    }

    fn literal_expr(&mut self, literal_expr: &Arc<ExprLiteral>) -> Result<Flow, Error> {
        todo!()
    }
    
    fn variable_expr(&mut self, variable_expr: &Arc<Variable>) -> Result<Flow, Error> {
        todo!()
    }

    fn call_expr(&mut self, call_expr: &Arc<ProcCall>) -> Result<Flow, Error> {
        todo!()
    }

    fn access_expr(&mut self, access_expr: &Arc<Access>) -> Result<Flow, Error> {
        todo!()
    }

    fn list_expr(&mut self, list_expr: &Arc<List>) -> Result<Flow, Error> {
        todo!()
    }
    
    fn assign_expr(&mut self, assign_expr: &Arc<Assignment>) -> Result<Flow, Error> {
        todo!()
    }
    
    fn set_expr(&mut self, set_expr: &Arc<Set>) -> Result<Flow, Error> {
        todo!()
    }
}