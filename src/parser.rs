use std::iter::Peekable;
use rowan::{Checkpoint, GreenNode, GreenNodeBuilder, Language};
use crate::lexer::Lexer;
use crate::syntax_kind::SyntaxKind;
use crate::syntax::ApLang;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    builder: GreenNodeBuilder<'static> // consider modding this lifetime
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input).peekable(),
            builder: GreenNodeBuilder::new(),
        }
    }

    // important
    pub fn parse(mut self) -> Parse {
        self.start_node(SyntaxKind::Root);

        self.expr();

        self.finish_node();

        Parse {
            green_node: self.builder.finish(),
        }
    }

    fn expr(&mut self) {
        self.expr_binding_power( 0);
    }

    fn peek(&mut self) -> Option<SyntaxKind> {
        self.lexer.peek().map(|(kind, _)| *kind)
    }

    fn bump(&mut self) {
        let (kind, text) = self.lexer.next().unwrap();

        self.builder.token(ApLang::kind_to_raw(kind), text.into())
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(ApLang::kind_to_raw(kind));
    }

    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint, ApLang::kind_to_raw(kind));
    }

    fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
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
        let formatted = format!("{:#?}", syntax_node);

        formatted[0..formatted.len() - 1].to_string()
    }
}


// ops
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod
}

impl Op {
    fn binding_power(&self) -> (u8, u8) {
        match self {
            Self::Add | Self::Sub => (1, 2),
            Self::Mul | Self::Div | Self::Mod => (3, 4),
        }
    }
}

// exprs
impl<'a> Parser<'a> {
    fn expr_binding_power(&mut self, min_binding_power: u8) {
        let checkpoint = self.checkpoint();

        match self.peek() {
            Some(SyntaxKind::Number | SyntaxKind::Ident) => self.bump(),
            _ => {}
        }

        loop {
            let op = match self.peek() {
                Some(SyntaxKind::Plus) => Op::Add,
                Some(SyntaxKind::Minus) => Op::Sub,
                Some(SyntaxKind::Star) => Op::Mul,
                Some(SyntaxKind::Slash) => Op::Div,
                Some(SyntaxKind::Mod) => Op::Mod,
                _ => return, // error state
            };

            let (lbp, rbp) = op.binding_power();

            if lbp < min_binding_power {
                return;
            }

            self.bump();

            self.start_node_at(checkpoint, SyntaxKind::BinOp);
            self.expr_binding_power(rbp);
            self.finish_node();
        }
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

    #[test]
    fn parse_number() {
        check(
            "123",
            expect![[r#"
Root@0..3
  Number@0..3 "123""#]],
        );
    }
}