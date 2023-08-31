use std::ops::Range;
use logos::Logos;
use rowan::{TextRange, TextSize};
use crate::syntax_kind::SyntaxKind;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, SyntaxKind>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: SyntaxKind::lexer(input)
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?.ok()?;
        let text = self.inner.slice();
        let range = {
            let Range { start, end } = self.inner.span();
            let start = TextSize::try_from(start).unwrap();
            let end = TextSize::try_from(end).unwrap();

            TextRange::new(start, end)
        };

        Some(Self::Item {
            kind, text, range
        })
    }
}


#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: SyntaxKind,
    pub text: &'a str,
    pub range: TextRange
}