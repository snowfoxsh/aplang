#![allow(dead_code, unused_variables)]

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
//! <3

pub mod aplang;
pub mod arguments;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod standard_library;

pub use aplang::*;
