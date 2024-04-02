use std::sync::Arc;
use crate::token::Token;


// To facilitate better error handling down the line,  
// we are going to store the tokens that the thing came from
// so we can report back to them later


struct Ast {
    source: Arc<str>,
    
}

#[derive(Debug)]
pub enum Expr {
    Literal {
        value: Literal,
        token: Token,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        token: Token,
    },
    Logical {
        left: Box<Expr>,
        operator: LogicalOp,
        right: Box<Expr>,
        token: Token,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
        token: Token
    },
    Grouping {
        expr: Box<Expr>,
        parens: (Token, Token),
    },
    ProcCall {
        ident: String,
        arguments: Arguments,
        
        token: Token,
        parens: (Token, Token)
    },
    Access {
        list: Box<Expr>,
        accessor: Box<Expr>,
        brackets: (Token, Token),
    },
    List {
        items: Vec<Expr>,
        brackets: (Token, Token),
    },
    Variable {
        ident: String,
        token: Token
    }
}

#[derive(Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Null
}

#[derive(Debug)]
pub struct Arguments {
    arguments: Vec<Expr>,

    tokens: Vec<Token>,
}

#[derive(Debug)]
pub enum BinaryOp {
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(Debug)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug)]
pub enum LogicalOp {
    Or,
    And,
}
