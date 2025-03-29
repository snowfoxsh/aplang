use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use cowvert::Data;


pub trait Object: Any + Display {
    fn as_any(&self) -> &dyn Any;
    fn clone_object(&self) -> Box<dyn Object>;
    fn eq(&self, other: &dyn Object) -> bool { false }
    fn iter(&self) -> Option<Box<dyn Iterator<Item=&Data<Value>>>> { None }
}

// blanket impl for T
impl<T: Any + Display + Clone + PartialEq> Object for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_object(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn eq(&self, other: &dyn Object) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |o| o == self)
    }
}

// blanket impl for Clone on Box<dyn Object>
impl Clone for Box<dyn Object> {
    fn clone(&self) -> Box<dyn Object> {
        self.clone_object()
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

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) =>
                a.iter().zip(b.iter()).all(|(a, b)| *a.borrow() == *b.borrow()),
            (Value::Object(a), Value::Object(b)) => a.eq(b.as_ref()),
            (Value::Callable(a), Value::Callable(b)) => todo!(),
            _ => false,
        }
    }
}

// fn equals() 

impl Eq for Value {}


impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) if *n == 0.0 => false,
            Value::Null => false,
            _ => true,
        }
    }
}

pub trait SmartClone {
    fn smart_clone(&mut self) -> Self;
}

impl SmartClone for Data<Value> {
    fn smart_clone(&mut self) -> Self {
        let use_val = {
            let val = self.borrow();
            matches!(val.deref(), Value::Null | Value::Bool(_) | Value::Number(_))
        };

        if use_val {
            self.by_val()
        } else {
            self.by_cow()
        }
    }
}

pub trait IterValue {
    fn iter_value(&self) -> Option<Box<dyn Iterator<Item=&Data<Value>>>>;

}