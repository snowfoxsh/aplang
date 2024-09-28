#![allow(dead_code, unused_variables, clippy::module_inception)]

//! # Hello
//! if you are looking to use the interpreter
//! please install it as a binary.
//! for more information please see [aplang.org](https://aplang.org).
//! ---
//! this for if you want to run ApLang as a library.
//! although this is not officially supported
//! i still provide it as an option.
//! everything is public because of that.
//! use with care
//! ---
//! <3

pub mod aplang;
pub mod arguments;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod standard_library;

pub use aplang::*;


#[test]
pub fn test() {
    let aplang = ApLang::new_from_stdin("3 + 3");
    let lexed = aplang.lex().unwrap();
    let parsed = lexed.parse().unwrap();
    let result = parsed.execute_with_debug().unwrap();

    let mut buf = String::new();
    result.debug_output(&mut buf).unwrap();
    println!("{buf}");
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    pub fn aplang(source_code: &str) -> String {
        let aplang = crate::ApLang::new_from_stdin(source_code);
        let lexed = aplang.lex().unwrap();
        let parsed = lexed.parse().unwrap();
        let result = parsed.execute_with_debug().unwrap();

        let mut buf = String::new();
        result.debug_output(&mut buf).unwrap();
        buf
    }
}
