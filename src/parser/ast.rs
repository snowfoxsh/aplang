use logos::{Span, SpannedIter};
use crate::lexer::token::Token;

pub trait AstNode {
    fn kind(&self) -> Self;
    fn span(&self) -> Span;
    fn spanned<'source>(&self) -> SpannedIter<'source, Token>;
}

// pub enum Node {
//
// }
