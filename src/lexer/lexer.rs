use crate::lexer::token::Token;
use crate::lexer::token::TokenType::*;
use crate::lexer::token::{LiteralValue, TokenType};
use miette::{miette, LabeledSpan, Report, SourceSpan};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::convert::From;
use std::fmt::Display;
use std::sync::Arc;

pub struct Lexer {
    file_name: String,
    source: Arc<str>,

    pub(super) tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,

    keywords: HashMap<&'static str, TokenType>,
}

impl Lexer {
    pub fn new(input: impl Into<Arc<str>>, file_name: String) -> Self {
        Self {
            file_name,
            source: input.into(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords: crate::lexer::token::get_keywords_hashmap(),
        }
    }

    pub fn scan(
        input: impl Into<Arc<str>>,
        file_name: String,
    ) -> miette::Result<Vec<Token>, Vec<Report>> {
        let mut lexer = Self::new(input, file_name);
        let tokens = lexer.scan_tokens()?;

        Ok(tokens)
    }

    pub fn scan_tokens(&mut self) -> miette::Result<Vec<Token>, Vec<Report>> {
        let mut errors: Vec<Report> = vec![];
        while !self.is_at_end() {
            // println!("({}..{})", self.start, self.current);
            self.start = self.current;
            match self.scan_token() {
                Ok(_) => (),
                Err(msg) => errors.push(msg),
            }
        }

        // push eof token onto token stack
        self.tokens.push(Token {
            token_type: Eof,
            lexeme: "<EOF>".to_string(),
            literal: None,
            span: SourceSpan::new(self.start.into(), 0usize),
            line_number: self.line,
            source: self.source.clone(), // pass a source ptr to each token
        });

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(self.tokens.clone())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> miette::Result<()> {
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
                    let labels = vec![LabeledSpan::at(
                        self.current_span(),
                        "operator `!` (bang) not allowed in syntax",
                    )];
                    let error = miette!(
                        labels = labels,
                        code = "lexer::unknown_symbol::bang",
                        help = "for logical not write `NOT` instead of `!`",
                        "{} unknown symbol `!`",
                        self.location_string()
                    )
                    .with_source_code(self.source.clone());

                    return Err(error);
                }
            }
            '=' => {
                if self.char_match('=') {
                    self.add_token(EqualEqual)
                } else {
                    let labels = vec![LabeledSpan::at(
                        self.current_span(),
                        "operator `=` (equals) not allowed in syntax",
                    )];
                    let error = miette!(
                        labels = labels,
                        code = "lexer::unknown_symbol::equals",
                        help = "for logical equals write `==` instead of `=`\n\
                        to assign to a variable write `<-` instead of `=`",
                        "{} unknown symbol `=`",
                        self.location_string()
                    )
                    .with_source_code(self.source.clone());

                    return Err(error);
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
                let labels = vec![LabeledSpan::at(
                    self.current_span(),
                    format!("symbol `{ch}` is not allowed in syntax"),
                )];

                let error = miette!(
                    labels = labels,
                    code = "lexer::unknown_symbol",
                    "{} unknown symbol `{ch}`",
                    self.location_string()
                )
                .with_source_code(self.source.clone());

                return Err(error);
            }
        }

        Ok(())
    }

    fn string(&mut self) -> miette::Result<()> {
        let mut result = String::new();

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            // escape codes
            if self.peek() == '\\' {
                self.advance(); // consume the backslash

                match self.peek() {
                    'n' => {
                        result.push('\n');
                        self.advance(); // consume 'n'
                    }
                    'r' => {
                        result.push('\r');
                        self.advance(); // consume 'r'
                    }
                    't' => {
                        result.push('\t');
                        self.advance(); // consume 't'
                    }
                    '\\' => {
                        result.push('\\');
                        self.advance(); // consume another '\'
                    }
                    '"' => {
                        result.push('"');
                        self.advance(); // consume the double quote
                    }
                    _ => {
                        // invalid escape sequence
                        return Err(miette!("Invalid escape sequence: \\{}", self.peek()));
                    }
                }
            } else {
                result.push(self.advance()); // add normal characters to the result
            }
        }

        // reaching the end without closing the string should throw an error
        if self.is_at_end() {
            let labels = vec![
                LabeledSpan::at_offset(self.start, "unmatched quote"),
                LabeledSpan::at(self.current_span(), "unmatched quote"),
            ];

            let error = miette!(
                labels = labels,
                code = "lexer::unterminated_string",
                help = "A string literal must end with a matching quote",
                "{} unterminated string",
                self.location_string()
            )
            .with_source_code(self.source.clone());

            return Err(error);
        }

        self.advance(); // consume the closing quote

        // store the parsed string literal in the token list
        self.add_token_lit(StringLiteral, Some(LiteralValue::String(result)));

        Ok(())
    }

    fn number(&mut self) -> miette::Result<()> {
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
                let labels = vec![LabeledSpan::at(self.current_span(), "could not parse")];

                let error = miette!(
                    labels = labels,
                    code = "lexer::unknown_token",
                    help = "this token might not be a valid number",
                    "{} failed to parse `{}` into number",
                    self.location_string(),
                    substring
                )
                .with_source_code(self.source.clone());

                return Err(error);
            }
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
            return false;
        }

        let mut i = 1;
        loop {
            let next_char = self.source.chars().nth(self.current + i);

            match next_char {
                // if we're at the end, then return false
                None => {
                    break false;
                }
                Some(next_char) => {
                    if next_char.is_whitespace() {
                        i += 1;
                    } else {
                        return next_char == ch;
                    }
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_lit(token_type, None)
    }

    fn add_token_lit(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        let text = self
            .source
            .get(self.start..self.current)
            .expect("Internal Compiler Error, This is a BUG")
            .to_string();

        let span_len = self.current - self.start;

        self.tokens.push(Token {
            token_type,
            lexeme: text,
            literal,
            line_number: self.line,
            span: SourceSpan::new(self.start.into(), span_len),
            source: self.source.clone(), // pass a pointer to source
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
