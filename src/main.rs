use std::fs;
use std::mem::size_of;
use std::path::Path;
use std::process::Termination;
use std::rc::Rc;
use std::sync::Arc;
use miette::{Context, IntoDiagnostic, Report, SourceSpan, Result, miette, diagnostic, LabeledSpan, ErrorHook, MietteHandlerOpts};
use owo_colors::OwoColorize;
use parser::Parser;
use lexer::Lexer;
use crate::ast::Ast;
use crate::ast::pretty::TreePrinter;
use crate::errors::{display_errors};
use crate::parser2::Parser2;
use crate::token::{print_tokens, Token};

mod errors;
mod expr;
mod parser;
mod parser2;
mod lexer;
mod stmt;
mod source;
mod token;
mod ast;


fn main() -> Result<()> {
    println!("{}", size_of::<Token>());
    println!("{}", size_of::<Arc<Token>>());
    println!("{}", size_of::<Rc<Token>>());
    // let input = "NOT (3 + hello == 3)".to_string();
    // let expr = "myProc(1, 2, 3) OR NOT (3 + 4 * (hello - 4) == 7 OR (5 - 2) > 0) AND (TRUE == FALSE) OR (\"sampleString\" != \"otherString\" AND 9 >= 3 * 2)";
    // let expr = "myProc(1, 2, 3) OR NOT (3 + 4 * (hello - 4) == 7 OR (5 - 2) > 0) AND (TRUE == FALSE) OR (\"sampleString\" != \"otherString\" AND 9 >= 3 * 2)";
    
    // let expr = "NOT NULL + 2";
    
    // let expr = "[1, 2, 3, 4, 5, 6, 7, 8, func(a, b, c, d, e, 1 + 2)]";
    
    // let expr = "[1, 2, 3, 4, 5, 6][1][1]";


    // let source = Lexer::scan(expr.to_string(), "hello.ap".to_string()).unwrap();
    
    // print_tokens(source.0.clone());

    // println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n");
    // display_errors(source, true);
    // let mut parser = Parser2::new(source.0, source.1);
    // 
    // let expr = parser.expression().unwrap();
    // 
    // println!("{}", expr.print_tree());
    
    test_file("./examples.ap/if.ap", true);

    // display_errors(source, true);
    // println!("{}", source.with_source_code(input));
    // let span = source.tokens[5 + 3].span;
    
    // Err(ParserError::TokenExpected {
    //     src: source,
    //     span: span,
    //     expected: ";".into()
    // })?;
    // let ls = LabeledSpan::at(span, "Unexpected Identifier");
    //
    // let ident = &source.tokens[5 + 3].lexeme;
    //
    // Err(miette!(
    //     code = "scanner::expected::rparen",
    //     labels = vec![ls],
    //     help = "Did you mean `if`",
    //
    //     "unexpected identifier `{}`", ident
    // ).with_source_code(source))?;
    //

    Ok(())


    // let error = MietteDiagnostic::new("There was an error").with_code("hell");
    // 
    // println!("{source:#?}");
    // 
    // println!("{:#?}\n{:#?}", ast, tokens);
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