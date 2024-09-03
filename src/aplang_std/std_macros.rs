#[macro_export]
macro_rules! std_function {
    ($location:expr => fn $name:ident ($($arg:ident:  Value $(:: $arg_type:ident)?),*) {$($body:tt)*}) => {
        $location.insert(
            String::from(stringify!($name)),
            (std::rc::Rc::new(crate::interpreter::NativeProcedure {
                name: String::from(stringify!($name)),
                arity: crate::arity!($($arg)*),
                callable: |_interpreter: &mut crate::interpreter::Interpreter,  args: &[crate::interpreter::Value], args_toks: &[miette::SourceSpan]| {
                    #[allow(unused_mut, unused_variables)]
                    let mut iter = args.into_iter();
                    #[allow(unused_mut)]
                    let mut iter_toks = iter.zip(args_toks.into_iter());

                    $(
                        let $arg = iter_toks.next().unwrap();
                        crate::unwrap_arg_type!($arg => Value $(::$arg_type)?);
                    )*

                    $($body)*
                }
            }), None)
        )
    };
}

#[macro_export]
macro_rules! arity {
    ($arg:ident $($tail:tt)*) => {
        1u8 + crate::arity!($($tail)*)
    };
    () => {
        0u8
    };
}

#[macro_export]
macro_rules! unwrap_arg_type {
    ($value:ident => Value::Null) => {
        #[allow(unused_mut)]
        let mut $value = match $value.0 {
            crate::interpreter::Value::Null => crate::interpreter::Value::Null,
            // todo make this a better message
            _ => return Err(
                crate::errors::RuntimeError {
                    // src: Arc::from("... code here".to_string()),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: "".to_string(),
                    label: "Argument cannot be cast into null".to_string(),
                }
            )
        }
    };
    ($value:ident => Value::Number) => {
        #[allow(unused_mut)]
        let crate::interpreter::Value::Number(mut $value) = $value.0.clone() else {
            return Err(
                crate::errors::RuntimeError {
                    // src: Arc::from("... code here".to_string()),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: "".to_string(),
                    label: format!("Argument Value ({}) is not of type Number", stringify!($value)),
                }
            );
       };
    };
    ($value:ident => Value::String) => {
        #[allow(unused_mut)]
        let crate::interpreter::Value::String(mut $value) = $value.0.clone() else {
            return Err(
                crate::errors::RuntimeError {
                    // src: Arc::from("... code here".to_string()),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: "".to_string(),
                    label: format!("Argument Value ({}) is not of type String", stringify!($value)),
                }
            );
        };
    };
    ($value:ident => Value::Bool) => {
        #[allow(unused_mut)]
        let crate::interpreter::Value::Bool(mut $value) = $value.0.clone() else {
            return Err(
                crate::errors::RuntimeError {
                    // src: Arc::from("... code here".to_string()),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: "".to_string(),
                    label: format!("Argument Value ({}) is not of type Bool", stringify!($value)),
                }
            );
        };
    };
    ($value:ident => Value::List) => {
        #[allow(unused_mut)]
        let crate::interpreter::Value::List(mut $value) = $value.0.clone() else {
            return Err(
                crate::errors::RuntimeError {
                    // src: Arc::from("... code here".to_string()),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: "".to_string(),
                    label: format!("Argument Value ({}) is not of type List<Value>", stringify!($value)),
                }
            );
        };
    };
    ($value:ident => Value) => {
        #[allow(unused_mut)]
        let mut $value = $value.0;
    };
}