use std::fmt::Debug;
use logos::{Logos, Span, SpannedIter};
use crate::lexer::syntax_kind::SyntaxKind;

pub trait AstNode: Debug {
    fn kind(&self) -> Self;
    fn span(&self) -> Span;
    fn spanned<'source>(&self) -> SpannedIter<'source, SyntaxKind>;
}

pub enum Node {

}
