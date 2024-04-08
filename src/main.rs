#![feature(type_alias_impl_trait)]

use std::fs;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use ariadne::{Report, Span};



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


pub (crate) type LReport<'a> = Report<'a,(&'a str, Range<usize>)>;
pub(crate) type LResult<'a, T> = Result<T, LReport<'a>>;
pub(crate) type LResults<'a, T> = Result<T, Vec<LReport<'a>>>;


fn main() {
    test_file("./examples.ap/if.ap", true);
}

fn test_file<P: AsRef<Path>>(path: P, parse: bool) {
    let contents: Arc<str> = fs::read_to_string(path).unwrap().into();
    let source = Lexer::scan(contents.clone(), "file.ap").unwrap();
    
    print_tokens(source.clone());
    
    // if !parse { return }
    // let mut parser = Parser2::new(source.0, source.1);
    // let ast = parser.parse();
    // 
    // let ast = match ast {
    //     Ok(ast) => {ast}
    //     Err(e) => {
    //         println!();
    //         display_errors(contents.clone(), e);
    //         return
    //     }
    // };
    // println!();
    // println!();
    // println!("{:}", ast.print_tree());
    // println!("{}",expr.print_tree());
}