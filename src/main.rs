use std::fs;
use logos::Logos;
use crate::syntax_kind::SyntaxKind;
use crate::parser::{Parser, SyntaxNode};

mod syntax_kind;
mod parser;
mod syntax;
mod lexer;

fn main() {
    let file = fs::read_to_string("src/test.ap")
        .expect("file not found!");

    print_syntax_kind(file);
    print_ast("423".to_string());
}

fn print_syntax_kind(input: String) {
    let mut lex = SyntaxKind::lexer(input.as_str());
    while let Some(tok) = lex.next() {
        println!("{:?}", tok)
    }
}

fn print_ast(input: String) {
    let parse = Parser::new(input.as_str()).parse();

    println!("{}", parse.debug_tree());
}