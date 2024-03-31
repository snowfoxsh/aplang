use std::process::Termination;
use miette::{Context, IntoDiagnostic, Report, SourceSpan, Result, miette, diagnostic, LabeledSpan, ErrorHook, MietteHandlerOpts};
use owo_colors::OwoColorize;
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
mod token;


fn main() -> Result<()> {
    let input = "(!a = b) {\n\
    string\n\
    line1\n\
    line2\n\
    line3\n\
    line4\n\
    line5\n\
    line6\n\
    line7\n\
    line8 \n\
    line9\n\
    line10\n\
    line11\n\
    line12\n\
    line13\n\
    line14\n\
    line15\n\
    line16\n\
    line17\n\
    line18\n\
    line19\n\
    line20\n\
    line21\n\
    line22\n\
    line23\n\
    line24\n\
    line25\n\
    \
    ";


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
