use crate::interpreter::FunctionMap;
use crate::interpreter::Value;
use crate::std_function;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn time() -> FunctionMap {
    let mut functions = FunctionMap::new();

    // gets the current time in milliseconds
    std_function!(functions=> fn TIME() {

        let now = SystemTime::now();
        let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");

        return Ok(Value::Number(unix_time_ms.as_millis() as f64))
    });

    std_function!(functions => fn SLEEP(duration: Value::Number) {
        let duration = std::time::Duration::from_millis(duration as u64);
        std::thread::sleep(duration);
        Ok(Value::Null)
    });

    functions
}
