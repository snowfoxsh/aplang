use std::fs;
use logos::Logos;
use crate::lexer::SyntaxKind;

mod lexer;
mod parser;
mod syntax;

fn main() {
    let file = fs::read_to_string("src/test.ap")
        .expect("file not found!");

    print_syntax_kind(file);
}

fn print_syntax_kind(input: String) {
    let mut lex = SyntaxKind::lexer(input.as_str());
    while let Some(tok) = lex.next() {
        println!("{:?}", tok)
    }
}