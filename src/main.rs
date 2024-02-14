use miette::SourceSpan;

mod scanner;
mod parser;
mod stmt;
mod expr;
mod errors;
mod parser2;

fn main() {
    SourceSpan::from(3);
}
