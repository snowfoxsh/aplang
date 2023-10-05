use logos::Logos;
use crate::debug::LangDebug;
use crate::lexer::syntax_kind::SyntaxKind;

pub struct DebugSyntaxKind;

impl LangDebug for DebugSyntaxKind {
    fn debug(input: String) {
        let mut sk = SyntaxKind::lexer(input.as_str());

        while let Some(token) = sk.next() {
            let slice = sk.slice();
            let span = sk.span();
            println!("{:?}", span);

            match token {
                Ok(t) => {
                    let t = format!("{:?}", t);

                    println!("{:<10} {}", t, slice);
                }

                Err(_) => {
                    eprintln!("{:<10} {}", "Unknown", slice)
                }
            }
        }
    }
}

pub struct Echo;

impl LangDebug for Echo {
    fn debug(input: String) {
        let mut sk = SyntaxKind::lexer(input.as_str());

        while let Some(_) = sk.next() {
            print!("{}", sk.slice())
        }
    }
}