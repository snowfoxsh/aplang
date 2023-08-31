use rowan::Language;
use crate::lexer::SyntaxKind;



#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct ApLang;

impl Language for ApLang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        todo!()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        todo!()
    }
}