use std::any::Any;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

// variable value types

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Number(f64),
    Bool(bool),
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    NativeObject(Rc<RefCell<dyn Any>>),
    NativeFunction(), // Assuming some representation
    Function(),       // Assuming some representation
}

impl Eq for Value {}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => *a.borrow() == *b.borrow(),
            (Value::NativeObject(a), Value::NativeObject(b)) => Rc::ptr_eq(a, b),
            (Value::NativeFunction(), Value::NativeFunction()) => false, // Define better comparison if needed
            (Value::Function(), Value::Function()) => false,             // Define better comparison if needed
            _ => false,
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => state.write_u8(0),
            Value::Number(n) => state.write_u64(n.to_bits()), // Ensures consistent hashing
            Value::Bool(b) => state.write_u8(if *b { 1 } else { 0 }),
            Value::String(s) => s.hash(state),
            Value::List(list) => {
                state.write_u8(4);
                for item in list.borrow().iter() {
                    item.hash(state);
                }
            }
            Value::NativeObject(obj) => {
                state.write_u8(5);
                let ptr = Rc::as_ptr(obj) as * const ();
                ptr.hash(state);
            }
            Value::NativeFunction() => state.write_u8(6), // Adjust if needed
            Value::Function() => state.write_u8(7),       // Adjust if needed
        }
    }
}


impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::List(l) => {
                // Borrow the list to access its elements
                let list = l.borrow();

                // Begin the list with an opening bracket
                write!(f, "[")?;

                // Iterate over the elements, formatting each one
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        // Add a comma and space before all elements except the first
                        write!(f, ", ")?;
                    }
                    // Write the current element using its Display implementation
                    write!(f, "{}", item)?;
                }

                // Close the list with a closing bracket
                write!(f, "]")
            }
            Value::String(s) => write!(f, "{s}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Bool(true) => write!(f, "TRUE"),
            Value::Bool(false) => write!(f, "FALSE"),
            _ => {
                write!(f, "NATIVE")
            }
        }
    }
}
