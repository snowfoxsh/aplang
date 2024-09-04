use std::time::{SystemTime, UNIX_EPOCH};

use crate::interpreter::{Env, FunctionMap, Value};
use crate::std_function;

pub(super) fn time() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    // gets the current time in milliseconds
    std_function!(functions=> fn TIME() {
        
        let now = SystemTime::now();
        let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");
        
        return Ok(Value::Number(unix_time_ms.as_millis() as f64))
    });
    
    functions
}