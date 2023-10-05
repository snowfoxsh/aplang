use logos::Logos;
use crate::lexer::syntax_kind::SyntaxKind;

#[derive(Debug)]
pub struct Lexer<'a> {
    pub inner: logos::Lexer<'a, SyntaxKind>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: SyntaxKind::lexer(input)
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (SyntaxKind, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.inner.next()?.ok()?;
        let text = self.inner.slice();

        Some((kind, text))
    }
}
