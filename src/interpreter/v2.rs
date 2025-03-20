use std::any::Any;
use std::fmt::{Display, Formatter};
use cowvert::Data;

pub trait Object: Any + Display {
}


// #[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Data<Value>>),
    Object(dyn Object),
    Callable(())
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(true) => write!(f, "TRUE"),
            Value::Bool(false) => write!(f, "FALSE"),
            Value::Number(v) | Value::String(v) | Value::Object(v) => write!(f, "{v}"),
            Value::List(list) => {
                write!(f, "[")?;
                
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    
                    write!(f, "{item}")?;
                }
                
                write!(f, "]")
            },
            Value::Callable(_) => todo!(),
        }
    }
}

impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,

            // todo
            (Value::List(a), Value::List(b)) => todo!(),
            (Value::Object(a), Value::Object(b)) => todo!(),
            (Value::Callable(a), Value::Callable(b)) => todo!(),
            
            // (Value::List(a), Value::List(b)) => a.borrow() == *b.borrow(),
            // (Value::NativeObject(a), Value::NativeObject(b)) => Rc::ptr_eq(a, b),
            // (Value::NativeFunction(), Value::NativeFunction()) => false, // Define better comparison if needed
            // (Value::Function(), Value::Function()) => false,             // Define better comparison if needed
            _ => false,
        }
    }
}

impl Eq for Value {}