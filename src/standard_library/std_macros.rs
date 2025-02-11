#[macro_export]
macro_rules! std_function {
    ($location:expr => fn $name:ident ($($arg:ident:  Value $(:: $arg_type:ident)? $(<$ot:ty>)?),*) {$($body:tt)*}) => {
        $location.insert(
            String::from(stringify!($name)),
            (std::rc::Rc::new($crate::interpreter::NativeProcedure {
                name: String::from(stringify!($name)),
                arity: $crate::arity!($($arg)*),
                callable: |_interpreter: &mut $crate::interpreter::Interpreter,  args: &[$crate::interpreter::Value], args_toks: &[miette::SourceSpan], _source: std::sync::Arc<str>| {
                    #[allow(unused_mut, unused_variables)]
                    let mut iter = args.into_iter();
                    #[allow(unused_mut)]
                    let mut __iter_toks = iter.zip(args_toks.into_iter());

                    $(
                        let $arg = __iter_toks.next().unwrap();
                        $crate::unwrap_arg_type!($arg => Value $(::$arg_type)? $(<$ot>)?, _interpreter, _source);
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
        1u8 + $crate::arity!($($tail)*)
    };
    () => {
        0u8
    };
}

#[macro_export]
macro_rules! unwrap_arg_type {
    ($value:ident => Value::Null, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let mut $value = match $value.0 {
            $crate::interpreter::value::Value::Null => $crate::interpreter::Value::Null,
            _ => return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type NULL", stringify!($value)),
                    label: "This argument cannot be cast into null".to_string(),
                }
            )
        }
    };
    ($value:ident => Value::Number, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let $crate::interpreter::Value::Number(mut $value) = $value.0.clone() else {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type NUMBER", stringify!($value)),
                    label: "This argument cannot be cast into NUMBER".to_string(),
                }
            );
       };
    };
    ($value:ident => Value::String, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let $crate::interpreter::Value::String(mut $value) = $value.0.clone() else {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type STRING", stringify!($value)),
                    label: "This argument cannot be cast into STRING".to_string(),
                }
            );
        };
    };
    ($value:ident => Value::Bool, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let $crate::interpreter::value::Value::Bool(mut $value) = $value.0.clone() else {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type BOOL", stringify!($value)),
                    label: "This argument cannot be cast into BOOL".to_string(),
                }
            );
        };
    };
    ($value:ident => Value::List, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let $crate::interpreter::Value::List(mut $value) = $value.0.clone() else {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: *$value.1,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type LIST<Value>", stringify!($value)),
                    label: "This argument cannot be cast into LIST".to_string(),
                }
            );
        };
    };
    ($value:ident => Value::NativeObject<$ot:ty>, $interpreter:ident, $source:ident) => {
        
        let __span = *$value.1;
        #[allow(unused_mut)]
        let $crate::interpreter::Value::NativeObject(mut $value) = $value.0.clone() else {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: __span,
                    message: "Invalid Argument Cast".to_string(),
                    help: format!("Argument Value ({}) is not of type NATIVE_OBJECT<A>", stringify!($value)),
                    label: "This argument cannot be cast into NATIVE_OBJECT".to_string(),
                }
            );
        };
        
        /* we check to make sure it is the right struct instance */
        if $value.as_ref().borrow().downcast_ref::<$ot>().is_none() {
            return Err(
                $crate::interpreter::errors::RuntimeError {
                    named_source: miette::NamedSource::new($interpreter.get_file_path(), $source),
                    span: __span,
                    message: "Invalid NATIVE_OBJECT variety for function".to_string(),
                    help: format!("THe function cannot accept this type"),
                    label: "This argument is a NATIVE_OBJECT but not the correct variety".to_string(),
                }
            )
        }
    };
    ($value:ident => Value, $interpreter:ident, $source:ident) => {
        #[allow(unused_mut)]
        let mut $value = $value.0;
    };
}

#[macro_export]
macro_rules! downcast {
    ($any:ident => $ty:ty) => {
        #[allow(clippy::mutable_key_type)]
        let mut __any_ref = $any.as_ref().borrow_mut();
        let $any =  __any_ref.downcast_mut::<$ty>().unwrap();
    };
}