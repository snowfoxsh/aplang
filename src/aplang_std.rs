use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::interpreter::{Env, NativeProcedure, Value};

macro_rules! std_function {
    ($location:expr => fn $name:ident ($($arg:ident: Value),*) {
        
        {
            
        }
    }) => {
        $location.insert(
            String::from(stringify!($name)),
            (Rc::new(NativeProcedure {
                name: String::from(stringify!($name)),
                arity: arity!($($arg)*),
                callable: |_, _| {
                    Ok(Value::Null)
                }
            }), None)
        )
    };
}

macro_rules! arity {
    ($arg:ident $($tail:tt)*) => {
        1u8 + arity!($($tail)*) 
    };
    () => {
        0u8
    };
}

// count!(hello: Value);

// fn test() {
//     count!(hello: Value);
// }

impl Env {
    pub(crate) fn inject_std(&mut self) {
        // self.functions.insert(String::from("TIME"), (Rc::new(NativeProcedure {
        //     name: "".to_string(),
        //     arity: 0,
        //     callable: (),
        // }), None))
        
        std_function!(self.functions => fn test(hello: Value) {
            
        });

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