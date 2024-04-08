use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;
use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Span};
use crate::token::TokenType::*;
use crate::{LReport, LResult, LResults, token};
use crate::token::{Token, TokenType};

pub enum LexerCodes {
    UnknownSymbol,
    UnterminatedString,
    NumberParseError,
}

impl Display for LexerCodes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       let code = match self {
           LexerCodes::UnknownSymbol => "unknown_symbol",
           LexerCodes::UnterminatedString => "unterminated_string",
           LexerCodes::NumberParseError => "number_parse_error"
       };

        write!(f, "{code}")
    }
}

pub struct Lexer<'l> {
    file_name: &'l str,
    source: Arc<str>,

    tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
    _report_marker : PhantomData<&'l ()>
}


impl<'l> Lexer<'l> {
    pub fn new(input: impl Into<Arc<str>>, file_name: &'l str) -> Lexer<'l> {
        Lexer {
            file_name,
            source: input.into(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: token::get_keywords_hashmap(),
            _report_marker: Default::default(),
        }
    }

    pub fn scan(input: impl Into<Arc<str>>, file_name: &'l str) -> LResults<'l, Vec<Token>> {
        let tokens = {
            let mut lexer = Self::new(input, file_name);
            lexer.scan_tokens()?
        };

        Ok(tokens)
    }

    pub fn scan_tokens(&mut self) -> LResults<'l, Vec<Token>> {
        let mut errors: Vec<_> = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => {
                    errors.push(msg);
                }
            }
        }

        // push eof token onto token stack
        self.tokens.push(
            Token {
                token_type: Eof,
                lexeme: "<EOF>".to_string(),
                literal: None,
                // span: SourceSpan::new(self.start.into(), 0usize),
                span: self.start..self.start,
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

    fn scan_token(&mut self) -> LResult<'l, ()> {
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
                    let mut colors = ColorGenerator::new();

                    let a = colors.next();
                    let b = colors.next();
                    let c = colors.next();

                    let report  = Report::build(ReportKind::Error, self.file_name, self.offset())
                        .with_code(LexerCodes::UnknownSymbol)
                        .with_message(format!("unrecognised symbol `{}`", "!".fg(a)))
                        .with_label(
                            Label::new((self.file_name, self.current_span()))
                                .with_message("this symbol is not allowed")
                                .with_color(a)
                        )
                        .with_help(
                            format!(
                                "for {} write `{}` instead of `{}`\nfor {} write {} instead of `{}`",
                                "logical not".fg(b), "NOT".fg(b), "!".fg(a), "logical not equals".fg(c), "!=".fg(c), "!".fg(a)))
                        .with_note(format!("aplang doesnt use `{}` in its syntax", "!".fg(a)))
                        .finish();
                    return Err(report)
                }
            }
            '=' => {
                if self.char_match('=') {
                    self.add_token(EqualEqual)
                } else {
                    let mut colors = ColorGenerator::new();

                    let a = colors.next();
                    let b = colors.next();
                    let c = colors.next();

                    let report: LReport = Report::build(ReportKind::Error, self.file_name, self.offset().clone())
                        .with_code(LexerCodes::UnknownSymbol)
                        .with_message(format!("unrecognised symbol `{}`", "=".fg(a)))
                        .with_label(
                            Label::new((self.file_name, self.current_span().clone()))
                                .with_message("this symbol is not allowed")
                                .with_color(a)
                        )
                        .with_help(
                            format!(
                                "for logical equals write `{}` instead of `{}`\nto assign to a variable write `{}` instead of `{}`",
                                "==".fg(b), "=".fg(a), "<-".fg(c), "=".fg(a)
                            )
                        )
                        .finish();

                    return Err(report)
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
                    self.line += 1;
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
                let mut colors = ColorGenerator::new();
                let a = colors.next();
                
                let report = Report::build(ReportKind::Error, self.file_name, self.offset())
                    .with_code(LexerCodes::UnterminatedString)
                    .with_message(format!("unrecognised character {}", ch.fg(a)))
                    .with_label(
                        Label::new((self.file_name, self.current_span()))
                            .with_message("this char is not allowed")
                            .with_color(a)
                    )
                    .with_help(format!("you cannot use {}", ch.fg(a)))
                    .finish();
                
                let report = Report::build(ReportKind::Error, self.file_name, self.offset()).finish();
                
                return Err(report)
            }
        }

        Ok(())
    }

    fn string(&mut self) -> LResult<'l, ()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        // reaching the end without closing the string should throw an error
        if self.is_at_end() {
            let mut colors = ColorGenerator::new();

            let a = colors.next();
            let b = colors.next();
            let c = colors.next();

            let report = Report::build(ReportKind::Error, self.file_name, self.offset())
                .with_code(LexerCodes::UnterminatedString)
                .with_message("unterminated string")
                .with_label(
                    Label::new((self.file_name, self.current_span()))
                        .with_message("this string is not terminated")
                        .with_color(a)
                )
                .with_help(
                    "a string literal must end with a matching quote"
                )
                .finish();
            
            return Err(report)
        }

        self.advance();

        let value = &self.source[self.start + 1..self.current - 1];

        self.add_token_lit(StringLiteral, Some(LiteralValue::String(value.to_string())));

        Ok(())
    }

    fn number(&mut self) -> LResult<'l, ()> {
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
        let value = substring.parse::<f64>();
        
        match value {
            Ok(value) => self.add_token_lit(Number, Some(LiteralValue::Number(value))),
            Err(_) => {
                let mut colors = ColorGenerator::new();

                let a = colors.next();
                let b = colors.next();
                let c = colors.next();

                let report = Report::build(ReportKind::Error, self.file_name.clone(), self.offset())
                    .with_code(LexerCodes::NumberParseError)
                    .with_message(format!("could not parse `{}` into type {}", self.current_slice().fg(a), "number".fg(b)))
                    .with_label(
                        Label::new((self.file_name, self.current_span()))
                            .with_message(format!("this is not a {}", "number".fg(b)))
                            .with_color(a)
                    )
                    .with_help(format!("try making literal `{}` into an {} (3) or {} (3.14)",
                                       self.current_slice().fg(a), "int".fg(b), "float".fg(b)))
                    .finish();
                
                return Err(report)
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
    
    fn check_next(&self, ch: char) -> bool {
        if self.is_at_end() {
            return false
        }
        
        let mut i = 1;
        loop {
            let next_char = self.source.chars().nth(self.current + i);
            
            match next_char {
                // if we are at the end then return false
                None => {
                    break false;
                }
                Some(next_char) => {
                    if next_char.is_whitespace() {
                        i += 1;
                    } else {
                        return next_char == ch
                    }
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None)
    }

    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self.source.get(self.start..self.current)
            .expect("Internal Compiler Error, This is a BUG")
            .to_string();

        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line_number: self.line,
            span: self.start..self.current,
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

    /// generate the location string for errors
    fn current_slice(&'l self) -> &'l str {
         &self.source[self.start..self.current]
    }
    
    
    fn current_span(&self) -> Range<usize> {
        self.start..self.current
    }

    fn offset(&self) -> usize {
        self.start
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

#[cfg(test)]
mod tests {
    use super::{Lexer, LiteralValue};
    use crate::token::TokenType::*;

    #[test]
    fn handle_one_char_tokens() {
        let source = "(( )) }{ []";
        let mut scanner = Lexer::new(source, String::default());
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
        let mut scanner = Lexer::new(source, String::default());
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
        let mut scanner = Lexer::new(source, String::default());
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
        let mut scanner = Lexer::new(source, "".to_string());
        let result = scanner.scan_tokens();
        match result {
            Err(_) => (),
            _ => panic!("Should have failed"),
        }
    }

    #[test]
    fn handle_string_lit_multiline() {
        let source = "\"ABC\ndef\"";
        let mut scanner = Lexer::new(source, String::default());
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
        let mut scanner = Lexer::new(source, String::default());
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
            let mut scanner = Lexer::new(keyword, String::default());
            let result = scanner.scan_tokens().expect("Scanner failed on lowercase");
            assert_eq!(result.len(), 2, "Failed on keyword length: {}", keyword); // Expecting keyword token and EOF token
            assert_eq!(result[0].token_type, token_type, "Failed on lowercase keyword: {}", keyword);

            // Test uppercase version
            let upper_keyword = keyword.to_uppercase();
            let mut scanner_upper = Lexer::new(upper_keyword.to_owned(), String::default());
            let result_upper = scanner_upper.scan_tokens().expect("Scanner failed on uppercase");
            assert_eq!(result_upper.len(), 2, "Failed on keyword length: {}", upper_keyword); // Expecting keyword token and EOF token
            assert_eq!(result_upper[0].token_type, token_type, "Failed on uppercase keyword: {}", upper_keyword);
        }
    }

    #[test]
    fn handle_identifer() {
        let source = "this_is_a_3_var <- 12;";
        let mut scanner = Lexer::new(source, String::default());
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
            let mut scanner = Lexer::new(source, String::new());
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