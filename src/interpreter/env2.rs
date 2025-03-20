use std::cell::RefCell;
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
    pub fn layer(self: EnvRef) -> EnvRef {
        let env = Environment {
            values: HashMap::new(),
            parent: Some(self),
        };

        Rc::new(RefCell::new(env))
    }

    /// Defines a new variable in the current environment
    /// Returns the previous value if it existed
    pub fn define(&mut self, name: String, value: ValueRef) -> Option<ValueRef> {
        self.values.insert(name, value)
    }

    /// Gets a variable from the environment
    /// It is a Data<Value> so you can choose to take it by ref (assign) or value (clone)
    pub fn search(&self, name: impl Into<&str>) -> Option<&ValueRef> {
        let name: &str = name.into();
        if let Some(val) = self.values.get(name) {
            Some(val)
        } else if let Some(ref parent) = self.parent {
            parent.borrow().search(name)
        } else {
            None
        }
    }
}
