use miette::SourceSpan;
use parser::Parser;
use scanner::Scanner;

mod errors;
mod expr;
mod parser;
mod parser2;
mod scanner;
mod stmt;

fn main() {
    let mut scanner = Scanner::new("1 + 3 * 2");
    let tokens = scanner.scan_tokens().unwrap();

    let mut parser = Parser::new(tokens.clone());
    let ast = parser.expression();

    println!("{:#?}\n{:#?}", ast, tokens);
}
