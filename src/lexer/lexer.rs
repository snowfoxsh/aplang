use logos::Logos;
use crate::lexer::syntax_kind::Token;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub inner: logos::Lexer<'a, Token>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input)
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?.ok()?;
        let text = self.inner.slice();

        Some((kind, text))
    }
}
