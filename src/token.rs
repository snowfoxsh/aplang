use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, write};
use std::ops::Range;
use std::sync::Arc;
use ariadne::{Label, Report, Span};
use crate::ast::{BinaryOp, LogicalOp, UnaryOp};
use crate::lexer::LiteralValue;
use crate::LResult;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Smart
    SoftSemi,

    // Single-char tokens
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,

    // Mixed
    Arrow,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    Number,
    StringLiteral,

    // Keywords
    Mod,
    If,
    Else,
    Repeat,
    Times,
    Until,
    For,
    Each,
    Continue,
    Break,
    In,
    Procedure,
    Return,
    Print,
    Not,
    And,
    Or,

    True,
    False,
    Null,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub span: Range<usize>,
    pub line_number: usize,
    pub source: Arc<str>,
}

// Implement Display for Token
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.lexeme)
    }
}

pub fn print_tokens(tokens: Vec<Token>) {
    for token in tokens {
        
        if matches!(token.token_type, TokenType::SoftSemi) {
            print!(" ;")
        }
        print!("{token}")
    }
}

pub fn get_keywords_hashmap() -> HashMap<&'static str, TokenType> {
    use mapro::map;
    use TokenType::*;
    map! {
        "mod" => Mod, "MOD" => Mod,
        "if" => If, "IF" => If,
        "else" => Else, "ELSE" => Else,
        "repeat" => Repeat, "REPEAT" => Repeat,
        "times" => Times, "TIMES" => Times,
        "until" => Until, "UNTIL" => Until,
        "for" => For, "FOR" => For,
        "each" => Each, "EACH" => Each,
        "continue" => Continue, "CONTINUE" => Continue,
        "break" => Break, "BREAK" => Break,
        "in" => In, "IN" => In,
        "procedure" => Procedure, "PROCEDURE" => Procedure,
        "return" => Return, "RETURN" => Return,
        "print" => Print, "PRINT" => Print,
        "not" => Not, "NOT" => Not,
        "and" => And, "AND" => And,
        "or" => Or, "OR" => Or,
        "true" => True, "TRUE" => True,
        "false" => False, "FALSE" => False,
        "null" => Null, "NULL" => Null,
    }
}

impl Token {
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }
    
    pub fn label(&self, file_name: &String ) -> Label {
        Label::new(self.span().clone())
    }
    pub fn span(&self) -> &Range<usize> {
        &self.span
    }
    
    pub fn span_to(&self, other: &Range<usize>) -> Range<usize> {
        join_spans(self.span(), other)
    }
}

pub fn join_spans(left: &Range<usize>, right: &Range<usize>) -> Range<usize> {
    left.start..right.end
}


impl Token {
    pub fn to_binary_op<'a>(&self) ->  LResult<'a, BinaryOp> {
        Ok(match self.token_type {
            TokenType::EqualEqual => BinaryOp::EqualEqual,
            TokenType::BangEqual => BinaryOp::NotEqual,
            TokenType::Less => BinaryOp::Less,
            TokenType::LessEqual => BinaryOp::LessEqual,
            TokenType::Greater => BinaryOp::Greater,
            TokenType::GreaterEqual => BinaryOp::GreaterEqual,
            TokenType::Plus => BinaryOp::Plus,
            TokenType::Minus => BinaryOp::Minus,
            TokenType::Star => BinaryOp::Star,
            TokenType::Slash => BinaryOp::Slash,
            _ =>
                // miette!("Conversion to Binary Op Error, Token is not binary Op")
                // todo: improve this message but this should not happen anywyas
                panic!("conversion to binary op error, token is not a binary op")
             
        })
    }

    pub fn to_unary_op<'a>(&self) -> LResult<'a, UnaryOp> {
        Ok(match self.token_type {
            TokenType::Minus => UnaryOp::Minus,
            TokenType::Not => UnaryOp::Not,
            // todo: improve this message
            _ => panic!("Conversion to Binary Unary Error, Token is not Unary op"),
        })
    }
    
    pub fn to_logical_op<'a>(&self) -> LResult<'a, LogicalOp> {
        match self.token_type {
            TokenType::Or => Ok(LogicalOp::Or),
            TokenType::And => Ok(LogicalOp::And),
            // todo: improve this message
            _ => panic!("Conversion to Binary Logical Error, Token is not Logical op"),
        }
    }
}