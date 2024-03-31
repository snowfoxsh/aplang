use miette::{bail, LabeledSpan, miette, Report, Result};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use mapro::map;
use miette::{Diagnostic, SourceSpan};
use owo_colors::{FgColorDisplay, OwoColorize};
use owo_colors::colors::Red;
use owo_colors::styles::BoldDisplay;
use TokenType::*;
use crate::source::Source;


pub struct Scanner {
    file_name: String,
    source: Arc<str>,

    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(input: impl Into<Arc<str>>, file_name: String) -> Self {
        Self {
            file_name,
            source: input.into(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: get_keywords_hashmap(),
        }
    }

    pub fn scan(input: impl Into<Arc<str>>, file_name: String) -> Result<Source, Vec<Report>> {
        let mut scanner = Self::new(input, file_name);
        let tokens = scanner.scan_tokens()?;
        let raw = scanner.source; // move the source pointer out of the scanner

        Ok(Source::new(tokens, raw))
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Vec<Report>> {
        let mut errors: Vec<Report> = vec![];
        while !self.is_at_end() {
            // println!("({}..{})", self.start, self.current);
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => errors.push(msg)
            }
        }

        // push eof token onto token stack
        self.tokens.push(
            Token {
                token_type: Eof,
                lexeme: "".to_string(),
                literal: None,
                span: SourceSpan::new(self.start.into(), 0usize),
                line_number: self.line,
                source: self.source.clone() // pass a source ptr to each token
            }
        );
        
        if !errors.is_empty() {
            return Err(errors)
        }

        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();

        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '[' => self.add_token(LeftBracket),
            ']' => self.add_token(RightBracket),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            '*' => self.add_token(Star),
            ';' => self.add_token(SoftSemi),
            '!' => {
                if self.char_match('=') {
                    self.add_token(BangEqual)
                } else {
                    let labels = vec![
                        LabeledSpan::at(self.current_span(), "operator `!` (bang) not allowed in syntax")
                    ];
                    let error = miette!(
                        labels = labels,
                        code = "lexer::unknown_symbol::bang",
                        help = "for logical not write `NOT` instead of `!`",
                        "{} unknown symbol `!`", self.location_string()
                    ).with_source_code(self.source.clone());
                    
                    return Err(error)
                }
            }
            '=' => {
                if self.char_match('=') {
                    self.add_token(EqualEqual)
                } else {
                    let labels = vec![
                        LabeledSpan::at(self.current_span(), "operator `=` (equals) not allowed in syntax")
                    ];
                    let error = miette!(
                        labels = labels,
                        code = "lexer::unknown_symbol::equals",
                        help = "for logical equals write `==` instead of `=`\n\
                        to assign to a variable write `<-` instead of `=`",
                        "{} unknown symbol `=`", self.location_string()
                    ).with_source_code(self.source.clone());

                    return Err(error)
                }
            }
            '<' => {
                let token = if self.char_match('=') {
                    LessEqual
                } else if self.char_match('-') {
                    Arrow
                } else {
                    Less
                };

                self.add_token(token)
            }
            '>' => {
                let token = if self.char_match('=') {
                    GreaterEqual
                } else {
                    Greater
                };

                self.add_token(token)
            }
            '/' => {
                if self.char_match('/') {
                    // comment
                    loop {
                        if self.peek() == '\n' || self.is_at_end() {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(Slash)
                }
            }
            ' ' | '\r' | '\t' => { /* nop */ }
            '\n' => {
                if let Some(prev) = self.tokens.last() {
                    // use go's method of implicit semicolons
                    // see: https://go.dev/ref/spec#Semicolons
                    match prev.token_type {
                        Identifier | // ident
                        Number | StringLiteral | // literal
                        Break | Continue | Return |
                        RightParen | RightBracket | RightBrace
                        => {
                            self.add_token(SoftSemi)
                        }
                        // otherwise ignore
                        _ => {}
                    }
                };
            }
            '"' => self.string()?,
            ch if ch.is_ascii_digit() => self.number()?,
            ch if ch.is_alphanumeric() => self.identifier(),
            ch => {
                let labels = vec![
                    LabeledSpan::at(self.current_span(), format!("symbol `{ch}` is not allowed in syntax"))
                ];
                
                let error = miette!(
                    labels = labels,
                    code = "lexer::unknown_symbol",
                    "{} unknown symbol `{ch}`", self.location_string()
                ).with_source_code(self.source.clone());
                
                return Err(error)
            }
        }

        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // reaching the end without closing the string should throw an error
        if self.is_at_end() {
            let labels = vec![
                LabeledSpan::at_offset(self.start, "unmatched quote")
            ];
            
            let error = miette!(
                labels = labels,
                code = "lexer::unterminated_string",
                help = "a string literal must end with a matching quote",
                "{} unterminated string", self.location_string()
            ).with_source_code(self.source.clone());
            
            return Err(error)
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];

        self.add_token_lit(StringLiteral, Some(LiteralValue::String(value.to_string())));

        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_advance().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        let substring = &self.source[self.start..self.current];
        // let value = substring.parse::<f64>();
        let value = Err("e");
        match value {
            Ok(value) => self.add_token_lit(Number, Some(LiteralValue::Number(value))),
            Err(_) => {
                let labels = vec![
                    LabeledSpan::at(self.current_span(), "could not parse")
                ];
                
                let error = miette!(
                    labels = labels,
                    code = "lexer::unknown_token",
                    help = "this token might not be a valid number",
                    "{} failed to parse `{}` into number", self.location_string(), substring
                ).with_source_code(self.source.clone());
                
                return Err(error)
            },
            
        }

        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let substring = &self.source[self.start..self.current];
        if let Some(keyword_token_type) = self.keywords.get(substring) {
            self.add_token(keyword_token_type.clone());
        } else {
            self.add_token(Identifier)
        }
    }

    fn peek_advance(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }
    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None)
    }

    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self.source.get(self.start..self.current)
            .expect("Internal Compiler Error, This is a BUG")
            .to_string();
        

        let span_len = self.current - self.start;

        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line_number: self.line,
            span: SourceSpan::new(self.start.into(), span_len),
            source: self.source.clone() // pass a pointer to source
        });
    }

    fn char_match(&mut self, ch: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != ch {
            false
        } else {
            self.current += 1;
            true
        }
    }
    
    
    fn current_span(&self) -> SourceSpan {
        SourceSpan::from(self.start..self.current)
    }

    /// generate the location string for errors
    fn location_string(&self) -> impl Display {
        let string = format!("{}:{}:{}", self.file_name, self.line, self.start);
        let string = string.bold();
        let string = string.red();
        format!("{string}")
    }
}


#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
}

impl TryInto<f64> for LiteralValue {
    type Error = (String);

    fn try_into(self) -> Result<f64, Self::Error> {
        let Self::Number(num) = self else {
            return Err("Trying to convert to number when literal is not of type number".to_string())
        };

        Ok(num)
    }
}

impl TryInto<String> for LiteralValue {
    type Error = (String);

    fn try_into(self) -> Result<String, Self::Error> {
        let Self::String(string) = self else {
            return Err("Trying to convert to string when literal is not of type string".to_string())
        };

        Ok(string)
    }
}


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
    pub span: SourceSpan,
    pub line_number: usize,
    pub source: Arc<str>
}

fn get_keywords_hashmap() -> HashMap<&'static str, TokenType> {
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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::process::Termination;
    use miette::{Diagnostic, IntoDiagnostic, MietteDiagnostic, WrapErr};
    use super::{LiteralValue, Scanner};
    use super::TokenType::*;

    #[test]
    fn handle_one_char_tokens() {
        let source = "(( )) }{ []";
        let mut scanner = Scanner::new(source, String::default());
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 9);
        assert_eq!(scanner.tokens[0].token_type, LeftParen);
        assert_eq!(scanner.tokens[1].token_type, LeftParen);
        assert_eq!(scanner.tokens[2].token_type, RightParen);
        assert_eq!(scanner.tokens[3].token_type, RightParen);
        assert_eq!(scanner.tokens[4].token_type, RightBrace);
        assert_eq!(scanner.tokens[5].token_type, LeftBrace);
        assert_eq!(scanner.tokens[6].token_type, LeftBracket);
        assert_eq!(scanner.tokens[7].token_type, RightBracket);
        assert_eq!(scanner.tokens[8].token_type, Eof);
    }

    #[test]
    fn handle_two_char_tokens() {
        let source = "<- != == >=";
        let mut scanner = Scanner::new(source, String::default());
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 5);
        assert_eq!(scanner.tokens[0].token_type, Arrow);
        assert_eq!(scanner.tokens[1].token_type, BangEqual);
        assert_eq!(scanner.tokens[2].token_type, EqualEqual);
        assert_eq!(scanner.tokens[3].token_type, GreaterEqual);
        assert_eq!(scanner.tokens[4].token_type, Eof);
    }

    #[test]
    fn handle_string_lit() {
        let source = r#""ABC""#;
        let mut scanner = Scanner::new(source, String::default());
        scanner.scan_tokens().unwrap();
        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, StringLiteral);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::String(val) => assert_eq!(val, "ABC"),
            _ => panic!("Incorrect literal type"),
        }
    }

    #[test]
    fn handle_string_lit_unterminated() {
        let source = r#""ABC"#;
        let mut scanner = Scanner::new(source, "".to_string());
        let result = scanner.scan_tokens();
        match result {
            Err(_) => (),
            _ => panic!("Should have failed"),
        }
    }

    #[test]
    fn handle_string_lit_multiline() {
        let source = "\"ABC\ndef\"";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();
        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, StringLiteral);
        match scanner.tokens[0].literal.as_ref().unwrap() {
            LiteralValue::String(val) => assert_eq!(val, "ABC\ndef"),
            _ => panic!("Incorrect literal type"),
        }
    }

    #[test]
    fn handle_number_literals() {
        let source = "123.123\n321.0\n5";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 6);

        match scanner.tokens[0].literal {
            Some(LiteralValue::Number(val)) => assert_eq!(val, 123.123),
            _ => panic!("Incorrect literal type"),
        }
        assert_eq!(scanner.tokens[1].token_type, SoftSemi);
        match scanner.tokens[2].literal {
            Some(LiteralValue::Number(val)) => assert_eq!(val, 321.0),
            _ => panic!("Incorrect literal type"),
        }
        assert_eq!(scanner.tokens[3].token_type, SoftSemi);
        match scanner.tokens[4].literal {
            Some(LiteralValue::Number(val)) => assert_eq!(val, 5.0),
            _ => panic!("Incorrect literal type"),
        }
        assert_eq!(scanner.tokens[5].token_type, Eof)
    }

    #[test]
    fn handle_keywords() {
        let keywords = vec![
            ("mod", Mod),
            ("if", If),
            ("else", Else),
            ("repeat", Repeat),
            ("times", Times),
            ("until", Until),
            ("for", For),
            ("each", Each),
            ("continue", Continue),
            ("break", Break),
            ("in", In),
            ("procedure", Procedure),
            ("return", Return),
            ("not", Not),
            ("and", And),
            ("or", Or),
            ("true", True),
            ("false", False),
            ("null", Null),
        ];

        for (keyword, token_type) in keywords {
            // Test lowercase version
            let mut scanner = Scanner::new(keyword);
            let result = scanner.scan_tokens().expect("Scanner failed on lowercase");
            assert_eq!(result.len(), 2, "Failed on keyword length: {}", keyword); // Expecting keyword token and EOF token
            assert_eq!(result[0].token_type, token_type, "Failed on lowercase keyword: {}", keyword);

            // Test uppercase version
            let upper_keyword = keyword.to_uppercase();
            let mut scanner_upper = Scanner::new(upper_keyword.to_owned());
            let result_upper = scanner_upper.scan_tokens().expect("Scanner failed on uppercase");
            assert_eq!(result_upper.len(), 2, "Failed on keyword length: {}", upper_keyword); // Expecting keyword token and EOF token
            assert_eq!(result_upper[0].token_type, token_type, "Failed on uppercase keyword: {}", upper_keyword);
        }
    }

    #[test]
    fn handle_identifer() {
        let source = "this_is_a_3_var <- 12;";
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens().unwrap();

        assert_eq!(scanner.tokens.len(), 5);

        assert_eq!(scanner.tokens[0].token_type, Identifier);
        assert_eq!(scanner.tokens[1].token_type, Arrow);
        assert_eq!(scanner.tokens[2].token_type, Number);
        assert_eq!(scanner.tokens[3].token_type, SoftSemi);
        assert_eq!(scanner.tokens[4].token_type, Eof);
    }

    #[test]
    fn handle_implicit_semicolon() {
        let test_cases = vec![
            ("varName\n", true), // Identifier ends with newline
            ("123\n", true), // Number ends with newline
            ("\"string\"\n", true), // StringLiteral ends with newline
            (")\n", true), // RightParen ends with newline
            ("]\n", true), // RightBracket ends with newline
            ("}\n", true), // RightBrace ends with newline
            ("+\n", false), // Plus does not end a statement
            ("varName", false), // No newline, no implicit semicolon
        ];

        for (source, should_have_semicolon) in test_cases {
            let mut scanner = Scanner::new(source);
            let result = scanner.scan_tokens().unwrap();

            let has_semicolon = result.iter().any(|token| token.token_type == SoftSemi);
            assert_eq!(has_semicolon, should_have_semicolon, "Failed on source: {}", source);
        }
    }

    #[test]
    fn test_spans() {
        let input = "IF (a == 3) {\
            a <- a + 1\
            }";
        
        // let num: i32 = input.parse().into_diagnostic().wrap_err("something here")

        // let source = Scanner::scan(input).unwrap();
        
        

        // let error = MietteDiagnostic::new("There was an error").with_code("hell");

        // println!("{source:#?}");
    }
}