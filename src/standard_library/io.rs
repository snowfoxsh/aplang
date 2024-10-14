use crate::interpreter::FunctionMap;
use crate::interpreter::Value;
use crate::{display, std_function};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(not(feature = "wasm"))]
pub(super) fn input(prompt: &str) -> Option<String> {
    use std::io;
    use std::io::Write;

    display!("{}", prompt);
    io::stdout().flush().ok()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok()?;
    Some(buf.trim_end().to_string())
}

#[cfg(feature = "wasm")]
pub(super) fn input(prompt: &str) -> Option<String> {
    use wasm_bindgen::prelude::*;
    use crate::wasm::IN;

    display!("begin input");
    // let output = IN.with(|input| {
    //     if let Some(ref js) = *input.borrow() {
    //        let this = JsValue::NULL;
    //
    //        let input_result = js.call1(
    //            &this,
    //            &JsValue::from_str(prompt), // prompt
    //        ).ok()?;
    //
    //        input_result.as_string()
    //     } else {
    //         None
    //     }
    // });

    IN.with(|input| {
        if let Some(ref callback) = *input.borrow() {
            let this = JsValue::NULL;

            callback.call1(&this, &JsValue::from_str("   |hello|    ")).unwrap();
        }
    });

    let output = Some("output".to_string());
    display!("end input");

    display!("{prompt}{}\n", output.clone().unwrap_or_default());

    output
}

fn format(fstring: String, args: Rc<RefCell<Vec<Value>>>) -> Option<String> {
    use std::fmt::Write; // need for write! to string
    let segments = fstring.split("{}").collect::<Vec<&str>>();

    // build the string
    let mut builder = String::new();
    for (i, segment) in segments.iter().enumerate() {
        write!(builder, "{}", segment).unwrap();

        // if we're last one there is no format arg
        if i + 1 < segments.len() {
            write!(builder, "{}", args.borrow()[i]).unwrap()
        }
    }

    Some(builder)
}

pub(super) fn std_io() -> FunctionMap {
    let mut functions = FunctionMap::new();
    std_function!(functions => fn INPUT_PROMPT(prompt: Value::String) {
        let result = input(prompt.as_str()).expect("Failed to get user input! Critical Failure");
        Ok(Value::String(result))
    });

    std_function!(functions => fn FORMAT(fstring: Value::String, args: Value::List) {
        let builder= format(fstring, args).expect("Incorrect number of format arguments. Failed to format");
        Ok(Value::String(builder))
    });

    std_function!(functions => fn DISPLAYF(fstring: Value::String, args: Value::List) {
        let builder= format(fstring, args).expect("Incorrect number of format arguments. Failed to format");
        println!("{}", builder);

        Ok(Value::Null)
    });

    functions
}
