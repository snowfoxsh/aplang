use std::fs;
use logos::Logos;
use crate::lexer::SyntaxKind;

mod lexer;
mod parser;
mod syntax;

fn main() {
    let input = read_from_file("src/test.ap");

    print_lexer(&input);
}

fn read_from_file(file_path: &str) -> String {
    fs::read_to_string(file_path)
        .expect("Should have been able to read the file")
}

fn print_lexer(input: &str) {
    let mut lex = SyntaxKind::lexer(input);
    while let Some(tok) = lex.next() {
        println!("{:?}", tok)
    }
}