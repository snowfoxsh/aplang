use num_traits::FromPrimitive;
use rowan::Language;
use crate::lexer::SyntaxKind;

pub type SyntaxNode = rowan::SyntaxNode<ApLang>;


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ApLang;

impl Language for ApLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}