use num_traits::{FromPrimitive, ToPrimitive};
use rowan::Language;
use crate::syntax_kind::SyntaxKind;

pub type SyntaxNode = rowan::SyntaxNode<ApLang>;
pub type SyntaxElement = rowan::SyntaxElement<ApLang>;
pub type SyntaxToken = rowan::SyntaxToken<ApLang>;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ApLang;

impl Language for ApLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.to_u16().unwrap())
    }
}