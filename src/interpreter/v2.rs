use std::any::Any;
use std::fmt::{Display, Formatter};
use cowvert::Data;


pub trait Object: Any + Display {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn Object>;
}

// Blanket impl for Clone on Box<dyn Object>
impl Clone for Box<dyn Object> {
    fn clone(&self) -> Box<dyn Object> {
        self.clone_box()
    }
}



#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Data<Value>>),
    Object(Box<dyn Object>),
    Callable(())
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(true) => write!(f, "TRUE"),
            Value::Bool(false) => write!(f, "FALSE"),
            // Value::Number(v) | Value::String(v) | Value::Object(v) => write!(f, "{v}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Object(o) => write!(f, "{o}"),
            
            Value::List(list) => {
                write!(f, "[")?;
                
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    
                    write!(f, "{}", *item.borrow())?;
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