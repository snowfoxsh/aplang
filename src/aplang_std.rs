use std::ops::Deref;
use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::interpreter::{Env, Interpreter, NativeProcedure, Value};

macro_rules! std_function {
    ($location:expr => fn $name:ident ($($arg:ident: Value),*) {$($body:tt)*}) => {
        $location.insert(
            String::from(stringify!($name)),
            (Rc::new(NativeProcedure {
                name: String::from(stringify!($name)),
                arity: arity!($($arg)*),
                callable: |_interpreter: &mut Interpreter,  args: &[Value]| {
                    let mut iter = args.into_iter(); 
                    $( let $arg = iter.next().unwrap();)*

                    $($body)*
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

macro_rules! unwrap_arg_type {
    ($value:expr => Value::Null) => {
        let $value = match $value {
            Value::Null => Value::Null,
            // todo make this a better message
            _ => return Err("Argument cannot be cast into null".to_string())
        }
    };
    ($value:expr => Value::Number) => {
        let Value::Number($value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type Number", stringify!($value)));
        };
    };
    ($value:ident => Value::String) => {
        let Value::String($value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type String", stringify!($value)));
        };
    };
    ($value:expr => Value::Bool) => {
        let Value::Bool($value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type Bool", stringify!($value)));
        };
    };
    ($value:expr => Value::List) => {
        let Value::List($value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type List<Value>", stringify!($value)));
        };
    };
}



// count!(hello: Value);

// fn test() {
//     count!(hello: Value);
// }

impl Env {
    pub(crate) fn inject_std(&mut self) {

        std_function!(self.functions => fn test(hello: Value) {
            unwrap_arg_type!(hello => Value::String);

            return Ok(Value::String(hello))

        });
        
        self.functions.insert(
            "TIME".to_string(),
            (Rc::new(NativeProcedure {
                name: "TIME".to_string(),
                arity: 0,
                callable: |s, args: &[Value]| {
                    let now = SystemTime::now();
                    let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");
                    Ok(Value::Number(unix_time_ms.as_millis() as f64))
                },
            }), None),
        );
    }
}