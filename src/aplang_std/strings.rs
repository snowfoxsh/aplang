use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::{Env, Value};
use crate::std_function;

pub(super) fn std_strings(env: &mut Env) {
    std_function!(env.functions => fn NUMBER(raw: Value::String) {
        let Ok(parsed) = raw.parse::<f64>() else {
            return Ok(Value::Null)
        }

        Ok(Value::Number(parsed))
    });

    std_function!(env.functions => fn BOOL(raw: Value::String) {
        let Ok(parsed) = raw.parse::<bool>() else {
            return Ok(Value::Null)
        }

        Ok(Value::Bool(parsed))
    });
    
    std_function!(env.functions => fn SPLIT(raw: Value::String, patern: Value::String) {
        let split: Vec<_> = raw.split(patern.as_str()).map(|slice| Value::String(slice.to_string())).collect();

        Ok(Value::List(Rc::new(RefCell::new(split))))
    });
}