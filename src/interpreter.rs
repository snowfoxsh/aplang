use crate::ast::{Expr, Stmt};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Type {
    Number,
    String,
    Bool,
    Nil,
    NativeFunction,
    ApFunction,
    List,
}

#[derive(Clone, Debug)]
pub struct ApFunction {
    pub id: usize,
    pub name: String,
    pub params: Vec<String>,
}


pub struct Enviernment {
    
}


pub struct Interpreter {
}
