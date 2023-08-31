use logos::{Logos};

#[derive(Logos, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
#[logos(skip "[ \t]+")]
pub enum SyntaxKind {
    #[regex("[A-Za-z][A-Za-z0-9_]*")]
    Ident,

    // i hate windows this took so long
    #[token("\r\n")]
    #[token("\n")]
    #[token("\r")]
    NewLine,

    #[regex("[0-9]+")]
    Number,

    // booleans
    #[token("true")]
    #[token("TRUE")]
    True,

    #[token("false")]
    #[token("FALSE")]
    False,

    #[token("\"")]
    Quote,

    #[token(".")]
    Dot,

    #[token("<-")]
    Assign,

    #[token(",")]
    Comma,

    // blocks
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    // math operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    // #[token("%")] // not recommended
    #[token("mod")]
    #[token("MOD")]
    Mod,

    // logical operators
    #[token("==")]
    Equals,

    #[token("!=")]
    NotEquals,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEquals,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEquals,

    // selection keywords
    #[token("if")]
    #[token("IF")]

    If,
    #[token("else")]
    #[token("ELSE")]
    Else,

    #[token("repeat")]
    #[token("REPEAT")]
    Repeat,

    #[token("times")]
    #[token("TIMES")]
    Times,

    #[token("until")]
    #[token("UNTIL")]
    Until,

    #[token("for")]
    #[token("FOR")]
    For,

    #[token("each")]
    #[token("EACH")]
    Each,

    #[token("in")]
    #[token("IN")]
    In,

    // procedure keywords
    #[token("procedure")]
    #[token("PROCEDURE")]
    Procedure,

    #[token("return")]
    #[token("RETURN")]
    Return,

    // cmp keywords
    #[token("not")]
    #[token("NOT")]
    Not,

    #[token("and")]
    #[token("AND")]
    And,

    #[token("or")]
    #[token("OR")]
    Or,

    #[token("//")]
    #[token("#")]
    Comment,

    // lexer ignore
    // for the parser
    Root,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}
