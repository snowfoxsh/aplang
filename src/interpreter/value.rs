use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::cell::RefCell;

// variable value types
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Number(f64),
    Bool(bool),
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    NativeFunction(),
    Function(),
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
                Value::Bool(b) => write!(f, "{b}", ),
                _ => { write!(f, "FUNCTION")}
        }
    }
}