use logos::{Span, SpannedIter};
use crate::lexer::token::Token;

pub trait AstNode {
    fn kind(&self) -> Self;
    fn span(&self) -> Span;
    fn spanned<'source>(&self) -> SpannedIter<'source, Token>;
}

pub type Program = Vec<Statement>;

pub enum Statement {
    Assign(Ident, Expr),
    Return(Expr),
    Expr(Expr),
}
pub enum Expr {
    Ident(Ident),
    Literal(Literal),
    Prefix(Prefix, Box<Expr>),
    Infix(Box<Expr>, Infix, Box<Expr>),
    If {
        condition: Box<Expr>,
        consequence: Program,
        alternative: Option<Program>,
    },
    Repeat {
        count: Box<Expr>,
        body: Program,
    },
    RepeatUntil {
        condition: Box<Expr>,
    },
    ForEach {
        item: Ident,
        list: Box<Expr>,
        body: Program,
    },
    Procedure {
        name: Ident,
        parameters: Vec<Ident>,
        body: Program,
    },
    ProcedureCall {
        arguments: Vec<Expr>,
        procedure: Box<Expr>
    },
    List(Vec<Expr>),
    Index {
        list: Box<Expr>,
        index: Box<Expr>,
    },
}

pub enum Literal {
    Number(f64),
    Bool(bool),
    String(String),
}

pub struct Ident(String);

pub enum Prefix {
    Plus,
    Minus,
    Not
}

pub enum Infix {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
}

