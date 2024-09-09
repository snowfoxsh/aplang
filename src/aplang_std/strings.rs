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

    std_function!(functions => fn TRIM(raw: Value::String) {
        Ok(Value::String(raw.trim().to_string()))
    });

    std_function!(functions => fn CONTAINS(raw: Value::String, pattern: Value::String) {
        Ok(Value::Bool(raw.contains(pattern.as_str())))
    });

    std_function!(functions => fn REPLACE(raw: Value::String, from: Value::String, to: Value::String) {
        Ok(Value::String(raw.replace(from.as_str(), to.as_str())))
    });

    std_function!(functions => fn STARTS_WITH(raw: Value::String, prefix: Value::String) {
        Ok(Value::Bool(raw.starts_with(prefix.as_str())))
    });

    std_function!(functions => fn ENDS_WITH(raw: Value::String, suffix: Value::String) {
        Ok(Value::Bool(raw.ends_with(suffix.as_str())))
    });

    std_function!(functions => fn JOIN(list: Value::List, separator: Value::String) {
        let list = list.borrow();
        let joined = list.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(separator.as_str());
        Ok(Value::String(joined))
    });

    std_function!(functions => fn SUBSTRING(raw: Value::String, start: Value::Number, length: Value::Number) {
        let start = start as usize;
        let length = length as usize;
        let substring = &raw[start..std::cmp::min(start + length, raw.len())];
        Ok(Value::String(substring.to_string()))
    });

    std_function!(functions => fn TO_CHAR_ARRAY(raw: Value::String) {
        let char_array: Vec<_> = raw.chars().map(|c| Value::String(c.to_string())).collect();

        Ok(Value::List(Rc::new(RefCell::new(char_array))))
    });

    functions
}