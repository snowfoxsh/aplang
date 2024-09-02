use crate::ast::{BinaryOp, LogicalOp, UnaryOp};
use crate::lexer::LiteralValue;
use miette::{miette, LabeledSpan, SourceSpan};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

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
    
    // Modules
    Import, Export, 

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<LiteralValue>,
    pub span: SourceSpan,
    pub line_number: usize,
    pub source: Arc<str>,
}

// Implement Display for Token
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.lexeme)
    }
}

impl Token {
    pub fn is_soft_semi(&self) -> bool {
        matches!(self.token_type, TokenType::SoftSemi)
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
        "import" => Import, "IMPORT" => Import,
        "export" => Export, "EXPORT" => Export,
    }
}

impl Token {
    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }
    pub fn label(&self, label: impl Into<String>) -> LabeledSpan {
        LabeledSpan::at(self.span, label)
    }

    pub fn span_to_label(&self, other: SourceSpan, label: impl Into<String>) -> LabeledSpan {
        LabeledSpan::at(self.span_to(other), label)
    }

    pub fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn span_to(&self, other: SourceSpan) -> SourceSpan {
        join_spans(self.span(), other)
    }
    
    pub fn span_until(&self, other: SourceSpan) -> SourceSpan {
        span_between(self.span(), other)
    }

    pub fn span_to_token(&self, other: &Token) -> SourceSpan {
        self.span_to(other.span())
    }
    
    pub fn span_until_token(&self, other: &Token) -> SourceSpan {
        self.span_until(other.span())
    }
}

pub fn join_spans(left: SourceSpan, right: SourceSpan) -> SourceSpan {
    let length = right.offset() - left.offset() + right.len();
    SourceSpan::from(left.offset()..length)
}

pub fn span_between(left: SourceSpan, right: SourceSpan) -> SourceSpan {
    SourceSpan::from(left.offset() + left.len() .. right.offset())
}

impl Token {
    pub fn to_binary_op(&self) -> miette::Result<BinaryOp> {
        match self.token_type {
            TokenType::EqualEqual => Ok(BinaryOp::EqualEqual),
            TokenType::BangEqual => Ok(BinaryOp::NotEqual),
            TokenType::Less => Ok(BinaryOp::Less),
            TokenType::LessEqual => Ok(BinaryOp::LessEqual),
            TokenType::Greater => Ok(BinaryOp::Greater),
            TokenType::GreaterEqual => Ok(BinaryOp::GreaterEqual),
            TokenType::Plus => Ok(BinaryOp::Plus),
            TokenType::Minus => Ok(BinaryOp::Minus),
            TokenType::Star => Ok(BinaryOp::Star),
            TokenType::Slash => Ok(BinaryOp::Slash),
            // todo: improve this message
            _ => Err(miette!(
                "Conversion to Binary Op Error, Token is not binary Op"
            )),
        }
    }

    pub fn to_unary_op(&self) -> miette::Result<UnaryOp> {
        match self.token_type {
            TokenType::Minus => Ok(UnaryOp::Minus),
            TokenType::Not => Ok(UnaryOp::Not),
            // todo: improve this message
            _ => Err(miette!(
                "Conversion to Binary Unary Error, Token is not Unary op"
            )),
        }
    }

    pub fn to_logical_op(&self) -> miette::Result<LogicalOp> {
        match self.token_type {
            TokenType::Or => Ok(LogicalOp::Or),
            TokenType::And => Ok(LogicalOp::And),
            // todo: improve this message
            _ => Err(miette!(
                "Conversion to Binary Logical Error, Token is not Logical op"
            )),
        }
    }
}
