//! Handles the wasm features for the library

use wasm_bindgen::prelude::wasm_bindgen;
use std::cell::RefCell;
use std::sync::Arc;
use web_sys::{window, Element};
use log::{error, Level, Metadata, Record};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use crate::{ApLang};

thread_local! {
    static DOM: RefCell<Option<Element>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn init_console_logging() {
    console_log::init_with_level(Level::Info)
        .expect("CRITICAL FAILURE: Failed to initialize logging in browser")
}

#[wasm_bindgen]
pub fn init_dom_logging(parent: Element) -> Result<(), JsValue> {
    DOM.with(|out|  {
        *out.borrow_mut() = Some(parent)
    });
    
    log::set_boxed_logger(Box::new(DOMLogger)).unwrap();
    log::set_max_level(log::LevelFilter::Info);
    
    Ok(())
}

pub struct DOMLogger;

impl log::Log for DOMLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());
            DOM.with(|out| {
                if let Some(ref element)  = *out.borrow() {
                    let document = window().unwrap().document().unwrap();
                    let p =  document.create_element("p").unwrap();
                    
                    let mut classes = Vec::new();
                    classes.push("log-line");
                    classes.push(match record.metadata().level() {
                        Level::Info => "info",
                        Level::Error => "error",
                        _ => "other"
                    });
                    
                    p.set_class_name(&classes.join(" "));
                    p.set_text_content(Some(&message));
                    
                    element.append_child(&p).unwrap();
                }
            })
        }
    }

    fn flush(&self) {}
}

#[wasm_bindgen]
pub fn aplang(source_code: &str) {
    // make sure source can escape
    let source_code: Arc<str> = source_code.into();
    
    spawn_local(async {
        let aplang = ApLang::new_from_stdin(source_code);

        let lexed = match aplang.lex() {
            Ok(lexed) => lexed,
            Err(reports) => {
                for report in reports {
                    error!("{:?}", report)
                }
                return;
            }
        };

        let parsed = match lexed.parse() {
            Ok(parsed) => parsed,
            Err(reports) => {
                for report in reports {
                    error!("{:?}", report)
                }
                return;
            }
        };


        match parsed.execute() {
            Err(report) => {
                error!("{:?}", report)
            }
            Ok(_) => { /* nop */}
        }    
    });
}
