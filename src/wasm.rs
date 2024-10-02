//! Handles the wasm features for the library

use wasm_bindgen::prelude::wasm_bindgen;
use std::cell::RefCell;
use std::sync::Arc;
use js_sys::Function;
use crate::{display_error, ApLang};

thread_local! {
    pub static OUT: RefCell<Option<Function>> = const { RefCell::new(None) };
    pub static IN: RefCell<Option<Function>> = const { RefCell::new(None) };
}


/// Provide a callback to javascript to handle input and output
#[wasm_bindgen]
pub fn bind_io(stdout: Function, stdin: Function) {
    // make a call to javascript
    OUT.with(|output| {
        *output.borrow_mut() = Some(stdout)
    });

    IN.with(|input| {
        *input.borrow_mut() = Some(stdin)
    })
}


#[wasm_bindgen]
pub fn aplang(source_code: &str) {
    // make sure source can escape
    let source_code: Arc<str> = source_code.into();
    
        let aplang = ApLang::new_from_stdin(source_code);

        let lexed = match aplang.lex() {
            Ok(lexed) => lexed,
            Err(reports) => {
                for report in reports {
                    display_error!("{:?}", report)
                }
                return;
            }
        };

        let parsed = match lexed.parse() {
            Ok(parsed) => parsed,
            Err(reports) => {
                for report in reports {
                    display_error!("{:?}", report)
                }
                return;
            }
        };


        match parsed.execute() {
            Err(report) => {
                display_error!("{:?}", report)
            }
            Ok(_) => { /* nop */}
        }    
}
