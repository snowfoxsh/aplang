use std::fs;
use logos::Logos;
use crate::syntax_kind::SyntaxKind;
use crate::parser::Parser;

mod syntax_kind;
mod parser;
mod syntax;
mod lexer;

// flags i want
// -d --debug-mode
// ex)
// -d parser
// -d -d parser

/*
flags i want:



prints output of lexer or whatever
-d --debug-mode
    -d parser
    -d ast

noflags
run integrated interprtor

 */

fn main() {
    print_syntax_kind("3MOD 3".to_string());
    // let _file = fs::read_to_string("src/test.ap")
    //     .expect("file not found!");

    // // print_syntax_kind(file);
    // print_ast("1+3".to_string());
    // println!("----------------------");
    // print_ast("1+2+3+4".to_string());
    // println!("----------------------");
    // print_ast("32 + 3 * 10".to_string());
    // println!("----------------------");
    // print_ast("3 * 10 + 32".to_string());
    // println!("----------------------");
    // print_ast("ad + b".to_string());
    // println!("----------------------");
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
