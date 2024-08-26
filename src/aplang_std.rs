use std::fmt::format;
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
    ($value:ident => Value::Null) => {
        let $value = match $value {
            Value::Null => Value::Null,
            // todo make this a better message
            _ => return Err("Argument cannot be cast into null".to_string())
        }
    };
    ($value:ident => Value::Number) => {
        let Value::Number(mut $value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type Number", stringify!($value)));
        };
    };
    ($value:ident => Value::String) => {
        let Value::String(mut $value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type String", stringify!($value)));
        };
    };
    ($value:ident => Value::Bool) => {
        let Value::Bool(mut $value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type Bool", stringify!($value)));
        };
    };
    ($value:ident => Value::List) => {
        let Value::List(mut $value) = $value.clone() else {
            return Err(format!("Argument Value ({}) is not of type List<Value>", stringify!($value)));
        };
    };
}

impl Env {
    pub(crate) fn inject_std(&mut self) {
        std_function!(self.functions => fn DISPLAY(value: Value) {
            println!("PRINT OUTPUT: {}", value);

            return Ok(Value::Null)
        });

        std_function!(self.functions => fn INSERT(list: Value, i: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            unwrap_arg_type!(i => Value::Number);
            
            // add one because indexed at one
            list.borrow_mut().insert(i as usize - 1, value.clone());

            return Ok(Value::List(list))
        }) ;

        std_function!(self.functions => fn APPEND(list: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            list.borrow_mut().push(value.clone());
            
            return Ok(Value::List(list))
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