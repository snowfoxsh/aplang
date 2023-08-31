use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, Language};
use crate::lexer::SyntaxKind;
use crate::syntax::ApLang;

pub struct Parser<'a> {
    lexer: logos::Lexer<'a, SyntaxKind>,
    builder: GreenNodeBuilder<'static> // consider modding this lifetime
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SyntaxKind::lexer(input),
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn parse(mut self) -> Parse {
        self.start_node(SyntaxKind::Root.into());
        self.finish_node();

        Parse {
            green_node: self.builder.finish()
        }
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(ApLang::kind_to_raw(kind));
    }

    fn finish_node(&mut self) {
        self.builder.finish_node();
    }
}

pub struct Parse {
    green_node: GreenNode,
}

pub type SyntaxNode = rowan::SyntaxNode<ApLang>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_root() {
        let output = Parser::new("").parse();

        assert_eq!(
            format!("{:#?}", SyntaxNode::new_root(output.green_node)),
            r#"Root@0..0
"#,
        );
    }
}