use std::fmt::Display;
use crate::scanner::{LiteralValue, Token, TokenType};



#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        value: Literal
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Parens {
        expr: Box<Expr>
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Null,
}


impl From<Token> for Literal {
    fn from(value: Token) -> Self {
       match value.token_type {
           TokenType::Number => Self::Number(value.literal.unwrap().try_into().unwrap() ),
           TokenType::StringLiteral => Self::String(value.literal.unwrap().try_into().unwrap() ),
           TokenType::True => Self::True,
           TokenType::False => Self::False,
           TokenType::Null => Self::Null,
           _ => panic!("could not create LiteralValue from {:?}",value)
       }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::True => write!(f, "TRUE"),
            Literal::False => write!(f, "FALSE"),
            Literal::Null => write!(f, "NULL"),
        }
    }
}


impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Expr::Binary {
                left,
                operator,
                right
            } => format!("({} {} {})",
                         operator.lexeme,
                         left,
                         right,
            ),
            Expr::Parens {
                expr
            } => format!("(group {})", expr),
            Expr::Literal {
                value
            } => format!("({})", value),
            Expr::Unary {
                operator,
                right
            } => format!("({} {})", operator.lexeme, right)
        };
        write!(f, "{}", str)
    }
}