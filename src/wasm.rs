//! Handles the wasm features for the library

use wasm_bindgen::prelude::wasm_bindgen;
use std::cell::RefCell;
use std::io::Stdin;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use js_sys::Function;
use web_sys::{window, Element};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use crate::{display, display_error, ApLang};

// thread_local! {
//     static DOM: RefCell<Option<Element>> = const { RefCell::new(None) };
// }

// #[wasm_bindgen]
// pub fn init_console_logging() {
//     console_log::init_with_level(Level::Info)
//         .expect("CRITICAL FAILURE: Failed to initialize logging in browser")
// }

// #[wasm_bindgen]
// pub fn init_dom_logging(parent: Element) -> Result<(), JsValue> {
//     DOM.with(|out|  {
//         *out.borrow_mut() = Some(parent)
//     });
//
//     log::set_boxed_logger(Box::new(DOMLogger)).unwrap();
//     log::set_max_level(log::LevelFilter::Info);
//
//     Ok(())
// }

// out: Function(this, output: String, is_error: bool)
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
pub fn aplang(source_code: &str, stdout: Function, stdin: Function) {
    bind_io(stdout, stdin);

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
