use logos::Logos;
use crate::debug::LangDebug;
use crate::lexer::token::Token;

pub struct DebugSyntaxKind;

impl LangDebug for DebugSyntaxKind {
    fn debug(input: String) {
        let mut sk = Token::lexer(input.as_str());

        while let Some(token) = sk.next() {
            let slice = sk.slice();
            let span = sk.span();

            match token {
                Ok(t) => {
                    let t = format!("{:?}", t);

                    println!("{:<25} {}", t, slice);
                }

                Err(_) => {
                    eprintln!("{:<25} {}", "Unknown [Error]", slice)
                }
            }

            println!("^ {:?}", span);
        }
    }
}

pub struct Echo;

impl LangDebug for Echo {
    fn debug(input: String) {
        let mut sk = Token::lexer(input.as_str());

        while let Some(_) = sk.next() {
            print!("{}", sk.slice())
        }
    }
}