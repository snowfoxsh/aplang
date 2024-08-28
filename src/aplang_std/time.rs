use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::interpreter::{Env, NativeProcedure, Value, Interpreter};
use crate::{std_function, arity, unwrap_arg_type};
use miette::miette;

impl Env {
    pub(crate) fn inject_std_time(&mut self) {
        // gets the current time in milliseconds
        std_function!(self.functions=> fn TIME() {
            
            let now = SystemTime::now();
            let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");
            
            return Ok(Value::Number(unix_time_ms.as_millis() as f64))
        });
    }
}