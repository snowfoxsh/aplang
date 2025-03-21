use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use cowvert::Data;
use crate::interpreter::v2::Value;

pub type ValueRef = Data<Value>;
pub type EnvRef = Rc<RefCell<Environment>>;

pub struct Environment {
    values: HashMap<String, ValueRef>,
    parent: Option<EnvRef>,
}

impl Environment {
    /// Creates a new empty [Environment]
    pub fn new() -> EnvRef {
        let env = Environment {
            values: HashMap::new(),
            parent: None,
        };

        Rc::new(RefCell::new(env))
    }

    /// Creates a new layer with self as the parent
    pub fn layer(this: EnvRef) -> EnvRef {
        let env = Environment {
            values: HashMap::new(),
            parent: Some(this),
        };

        Rc::new(RefCell::new(env))
    }

    /// Defines a new variable in the current environment
    /// Returns the previous value if it existed
    pub fn define(&mut self, name: String, value: ValueRef) -> Option<ValueRef> {
        self.values.insert(name, value)
    }

    /// Gets a value from the environment
    /// It is a Data<Value> so you can choose to take it by ref (assign) or value (clone)
    /// Searches for a variable and returns a `Ref` to the inner value.
    // Change the return type to not be tied to the input reference lifetime
    pub fn search(this: &EnvRef, name: &str) -> Option<ValueRef> {
        let env_borrow = this.borrow();

        // If found in current environment, clone the ValueRef and return it
        if let Some(value) = env_borrow.values.get(name) {
            return Some(value.clone());
        }

        // If there's a parent, search recursively
        if let Some(parent) = &env_borrow.parent {
            return Environment::search(parent, name);
        }

        None
    }
}
