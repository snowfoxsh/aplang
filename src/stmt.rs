use crate::expr::Expr;
use crate::scanner::Token;

pub struct Ident(String);

pub enum Stmt {
    Expression {
        expression: Expr
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Ident,
        initializer: Expr
    },
    Block {
        stmts: Vec<Stmt>
    },
    Procedure {
        name: Ident,
        params: Vec<Token>, // vec ident
        body: Vec<Stmt>
    },
    If {
        predicate: Expr,
        body: Box<Stmt>, // should this be vec?
        alt: Option<Box<Stmt>>
    },
    RepeatLoop {
        count: Expr,
        body: Box<Stmt> // should this be vec?
    },
    RepeatUntilLoop {
        predicate: Expr,
        body: Box<Stmt>
    },
    ForEachLoop {
        item: Ident, // ident
        list: Expr,
        body: Box<Stmt>
    },
    Return {
        keyword: Token,
        value: Option<Expr>
    }
}

