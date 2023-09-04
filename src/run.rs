use logos::Logos;
use crate::parser::Parser;
use crate::syntax_kind::SyntaxKind;

pub fn run(input: String) {
    todo!()
}

pub fn debug_parser(input: String) {
    let parse = Parser::new(input.as_str()).parse();

    println!("{}", parse.debug_tree());
}

pub fn debug_lexer(input: String) {
    let mut lex = SyntaxKind::lexer(input.as_str());
    while let Some(tok) = lex.next() {
        println!("{:?}", tok)
    }
}