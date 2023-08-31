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
    use expect_test::{Expect, expect};
    use super::*;

    fn check(input: &str, expected_tree: Expect) {
        let parse = Parser::new(input).parse();
        let syntax_node = SyntaxNode::new_root(parse.green_node);

        let actual_tree = format!("{:#?}", syntax_node);

        // We cut off the last byte because formatting the SyntaxNode adds on a newline at the end.
        expected_tree.assert_eq(&actual_tree[0..actual_tree.len() - 1]);
    }

    #[test]
    fn parse_nothing() {
        check("", expect![[r#"Root@0..0"#]]);
    }
}