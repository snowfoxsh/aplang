use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::{FunctionMap, Value};
use crate::std_function;

pub(super) fn std_strings() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    // casts String to Number, returns NULL if not possible
    std_function!(functions => fn TO_NUMBER(raw: Value::String) {
        let Ok(parsed) = raw.parse::<f64>() else {
            return Ok(Value::Null)
        };

        Ok(Value::Number(parsed))
    });

    // casts String to Bool, returns NULL if not possible
    std_function!(functions => fn TO_BOOL(raw: Value::String) {
        let Ok(parsed) = raw.parse::<bool>() else {
            return Ok(Value::Null)
        };

        Ok(Value::Bool(parsed))
    });
    
    // splits a string into a list of strings based on a pattern string
    std_function!(functions => fn SPLIT(raw: Value::String, pattern: Value::String) {
        let split: Vec<_> = raw.split(pattern.as_str()).map(|slice| Value::String(slice.to_string())).collect();

        Ok(Value::List(Rc::new(RefCell::new(split))))
    });
    
    // String to Upper Case
    std_function!(functions => fn TO_UPPER(raw: Value::String) {
        Ok(Value::String(raw.to_uppercase()))
    });
    
    // String to Lower Case
    std_function!(functions => fn TO_LOWER(raw: Value::String) {
        Ok(Value::String(raw.to_lowercase()))
    });
    
    functions
}