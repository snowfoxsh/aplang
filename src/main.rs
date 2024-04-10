use std::fs;
use std::mem::size_of;
use std::path::Path;


use miette::{Result};


use lexer::Lexer;

use crate::ast::pretty::TreePrinter;
use crate::ast::Stmt;
use crate::errors::display_errors;
use crate::parser2::Parser2;
use crate::token::print_tokens;

mod errors;
mod parser2;
mod lexer;
mod token;
mod ast;
mod interpreter;


fn main() -> Result<()> {
    test_file("./examples.ap/if.ap", true);

    Ok(())
}

fn test_file<P: AsRef<Path>>(path: P, parse: bool) {
    let contents = fs::read_to_string(path).unwrap();
    let source = Lexer::scan(contents, "file.ap".to_string()).unwrap();
    
    print_tokens(source.0.clone());
    
    if !parse { return }
    let mut parser = Parser2::new(source.0, source.1);
    let ast = parser.parse();
    
    let ast = match ast {
        Ok(ast) => {ast}
        Err(e) => {
            println!();
            display_errors(e, true);
            return
        }
    };
    println!();
    println!();
    println!("{:}", ast.print_tree());
    // println!("{}",expr.print_tree());
}