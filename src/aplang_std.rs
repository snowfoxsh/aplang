use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::interpreter::{Env, NativeProcedure, Value};

impl Env {
    pub(crate) fn inject_std(&mut self) {
        // self.functions.insert(String::from("TIME"), (Rc::new(NativeProcedure {
        //     name: "".to_string(),
        //     arity: 0,
        //     callable: (),
        // }), None))

        self.functions.insert(
            "TIME".to_string(),
            (Rc::new(NativeProcedure {
                name: "TIME".to_string(),
                arity: 0,
                callable: |_, _| {
                    let now = SystemTime::now();
                    let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");
                    Ok(Value::Number(unix_time_ms.as_millis() as f64))
                },
            }), None),
        );
    }
}