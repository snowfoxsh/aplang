use std::process::Termination;
use miette::{Context, IntoDiagnostic, Report, SourceSpan, Result, miette, diagnostic, LabeledSpan};
use parser::Parser;
use lexer::Lexer;
use crate::errors::{display_errors};

mod errors;
mod expr;
mod parser;
mod parser2;
mod lexer;
mod stmt;
mod source;

fn main() -> Result<()> {
    let input = "\"(!a = b) ";


    let source = Lexer::scan(input, "hello.ap".to_string()).unwrap_err();
   
    display_errors(source, true);
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
