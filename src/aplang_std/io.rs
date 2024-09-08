use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::rc::Rc;
use std::sync::Arc;
use crate::interpreter::{FunctionMap, Value};
use crate::std_function;

pub(super) fn input(prompt: &str) -> Option<String> {
    print!("{}", prompt);
    io::stdout().flush().ok()?;


    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok()?;
    Some(buf.trim_end().to_string())
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