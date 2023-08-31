use std::fmt::format;
use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder, Language};
use crate::syntax_kind::SyntaxKind;
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

        if self.lexer.next() == Some(Ok(SyntaxKind::Number)) {
            self.builder.token(
                ApLang::kind_to_raw(SyntaxKind::Number),
                self.lexer.slice().into(),
            );
        }

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
    pub(crate) green_node: GreenNode,
}

impl Parse {
    pub fn debug_tree(&self) -> String {
        let syntax_node = SyntaxNode::new_root(self.green_node.clone());
        let formated = format!("{:#?}", syntax_node);

        formated[0..formated.len() - 1].to_string()
    }
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