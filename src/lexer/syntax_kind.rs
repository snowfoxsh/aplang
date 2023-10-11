use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, PartialOrd, Clone)]
// #[logos(skip "[ \t]+")]
pub enum Token {
    #[regex("[ \t]+")]
    Whitespace,

    #[regex("//.*")]
    #[regex("#.*")]
    Comment,

    #[regex("[A-Za-z_][A-Za-z0-9_]*", as_string)]
    Ident(String),

    // this took me so long for some reason????
    #[regex(r"\d*\.?\d*", as_number)]
    Number(f64),

    // i hate windows this took so long
    #[token("\r\n")]
    #[token("\n")]
    #[token("\r")]
    NewLine,

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

    #[cfg(not(feature = "c_syntax"))]
    #[token("mod")]
    #[token("MOD")]
    Mod,

    #[cfg(feature = "c_syntax")]
    #[token("%")]
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
    #[cfg(not(feature = "c_syntax"))]
    #[token("not")]
    #[token("NOT")]
    Not,

    #[cfg(feature = "c_syntax")]
    #[token("!")]
    #[token("not")]
    #[token("NOT")]
    Not,

    #[cfg(not(feature = "c_syntax"))]
    #[token("and")]
    #[token("AND")]
    And,

    #[cfg(feature = "c_syntax")]
    #[token("&&")]
    #[token("and")]
    #[token("AND")]
    And,

    #[cfg(not(feature = "c_syntax"))]
    #[token("or")]
    #[token("OR")]
    Or,

    #[cfg(feature = "c_syntax")]
    #[token("||")]
    #[token("or")]
    #[token("OR")]
    Or,
}

fn as_string(lex: &Lexer<Token>) -> String {
    lex.slice().to_string()
}

fn as_number(lex: &Lexer<Token>) -> Option<f64> {
    lex.slice().parse().ok()
}

#[cfg(test)]
mod tests {
    use crate::lexer::syntax_kind::Token::*;
    use super::*;
    fn check(input: &str, kind: Token) {
        let mut lexer = Token::lexer(input);
        let span = lexer.span();

        assert_eq!(lexer.next(), Some(Ok(kind)));
        assert_eq!(lexer.slice(), input);
    }

    #[test]
    fn test_newline() {
        check("\n", NewLine);
        check("\r\n", NewLine);
        check("\r", NewLine);
    }

    #[test]
    fn test_number() {
        check("23", Number);
        check("4444", Number);
        check("1", Number);
    }

    #[test]
    fn test_true() {
        check("true", True);
        check("TRUE", True);
    }

    #[test]
    fn test_false() {
        check("false", False);
        check("FALSE", False);
    }

    #[test]
    fn test_quote() {
        check("\"", Quote);
    }

    #[test]
    fn test_dot() {
        check(".", Dot);
    }

    #[test]
    fn test_assign() {
        check("<-", Assign);
    }

    #[test]
    fn test_comma() {
        check(",", Comma);
    }

    #[test]
    fn test_operators() {
        check("+", Plus);
        check("-", Minus);
        check("*", Star);
        check("/", Slash);
        check("mod", Mod);
        check("MOD", Mod);
    }

    #[test]
    fn test_logic_operators() {
        check("==", Equals);
        check("!=", NotEquals);
        check(">", Greater);
        check(">=", GreaterEquals);
        check("<", Less);
        check("<=", LessEquals);
    }

    #[test]
    fn test_keywords() {
        check("if", If);
        check("IF", If);
        check("else", Else);
        check("ELSE", Else);
        check("procedure", Procedure);
        check("PROCEDURE", Procedure);
    }

    #[test]
    fn test_comments() {
        check("//", Comment);
        check("#", Comment);
    }

    #[test]
    fn test_ident() {
        check("variableName", Ident);
        check("variable_another", Ident);
        check("Var123", Ident);
    }

    #[test]
    fn test_selection_keywords() {
        check("if", If);
        check("IF", If);
        check("else", Else);
        check("ELSE", Else);
        check("repeat", Repeat);
        check("REPEAT", Repeat);
        check("times", Times);
        check("TIMES", Times);
        check("until", Until);
        check("UNTIL", Until);
        check("for", For);
        check("FOR", For);
        check("each", Each);
        check("EACH", Each);
        check("in", In);
        check("IN", In);
    }

    #[test]
    fn test_parentheses() {
        check("(", LeftParen);
        check(")", RightParen);
    }

    #[test]
    fn test_braces() {
        check("{", LeftBrace);
        check("}", RightBrace);
    }

    #[test]
    fn test_brackets() {
        check("[", LeftBracket);
        check("]", RightBracket);
    }
    // fill in the rest of the tests
}

// the better the function is preformed the more excellent the thing
// rootless is trying to explain how to be a good person.
// he thinks that it is a thing that is how ur raised