use logos::Logos;
use rowan::{GreenNode, GreenNodeBuilder};
use crate::lexer::SyntaxKind;

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
        self.builder.start_node(SyntaxKind::Root.into());
        self.builder.finish_node();

        Parse {
            green_node: self.builder.finish()
        }
    }
}

pub struct Parse {
    green_node: GreenNode,
}