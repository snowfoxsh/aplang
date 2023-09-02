use std::fs;
use logos::Logos;
use crate::syntax_kind::SyntaxKind;
use crate::parser::Parser;

mod syntax_kind;
mod parser;
mod syntax;
mod lexer;

fn main() {
    let _file = fs::read_to_string("src/test.ap")
        .expect("file not found!");

    // print_syntax_kind(file);
    print_ast("hello <- 3".to_string());
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