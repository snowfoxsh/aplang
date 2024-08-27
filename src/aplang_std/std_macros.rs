use std::rc::Rc;
use crate::interpreter::{Env, Interpreter, NativeProcedure, Value};

#[macro_export]
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

#[macro_export]
macro_rules! arity {
    ($arg:ident $($tail:tt)*) => {
        1u8 + arity!($($tail)*)
    };
    () => {
        0u8
    };
}

#[macro_export]
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