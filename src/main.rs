use std::fs;
use std::path::Path;

use miette::Result;

use lexer::Lexer;

use crate::ast::pretty::TreePrinter;
use crate::errors::display_errors;
use crate::interpreter::Interpreter;
use crate::parser2::Parser2;
use crate::token::print_tokens;

mod ast;
mod errors;
mod interpreter;
mod lexer;
mod parser2;
mod token;
mod aplang_std;

fn main() -> Result<()> {
    test_file("./examples.ap/scope.ap", true);

    Ok(())
}

fn test_file<P: AsRef<Path>>(path: P, parse: bool) {
    let contents = fs::read_to_string(path).unwrap();
    let source = Lexer::scan(contents, "file.ap".to_string()).unwrap();

    print_tokens(source.0.clone());

    if !parse {
        return;
    }
    let mut parser = Parser2::new(source.0, source.1, "main.ap");
    let ast = parser.parse();

    let ast = match ast {
        Ok(ast) => ast,
        Err(e) => {
            println!();
            display_errors(e, true);
            return;
        }
    };
    println!();
    println!();
    println!("{:}", ast.print_tree());

    let mut interpreter = Interpreter::new(ast);

    let now = std::time::Instant::now();
    let results = interpreter.interpret_debug().unwrap();
    let duration = now.elapsed();

    results.iter().for_each(|value| println!("{value:?}"));

    println!("runtime: {duration:?}");
    // println!("{:?}", results);

    // println!("{}",expr.print_tree());
}
