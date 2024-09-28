//! Handles the wasm features for the library

use wasm_bindgen::prelude::wasm_bindgen;
use std::cell::RefCell;
use web_sys::{window, Element};
use log::{info, Level, Metadata, Record};
use wasm_bindgen::JsValue;

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

#[wasm_bindgen]
pub fn test_logger() {
    info!("hello from the logger )1");
    info!("hello from the logger )2");
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
                    p.set_class_name("log-line");
                    p.set_text_content(Some(&message));
                    element.append_child(&p).unwrap();
                }
            })
        }
    }

    fn flush(&self) {}
}