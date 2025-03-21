use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
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

    pub fn get<'a, 'b>(&'b self, name: impl Into<&'a str>) -> Option<&'b ValueRef> {
        let name = name.into();

        
        if let Some(value) = self.values.get(name) {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
        
        // let mut env = self;

        // loop {
        //     if let Some(value) = env.values.get(name) {
        //         return Some(value);
        //     }
        // 
        //     if let Some(parent) = &env.parent {
        //         env = parent.as_ref();
        //     } else {
        //         return None;
        //     }
        // }
    }
}

pub trait SearchEnv {
    fn get_ref<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef>;
    fn get_val<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef>;
}

impl SearchEnv for EnvRef {
    fn get_ref<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();
        let mut env_borrow = self.borrow();

        loop {
            if let Some(mut value) = env_borrow.values.get(name) {
                return Some(value.by_ref());
            }

            if let Some(parent) = &env_borrow.parent {
                env_borrow = parent.borrow();
            } else {
                return None
            }
        }
    }

    fn get_val<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();
        let mut env_borrow = self.borrow();

        loop {
            if let Some(mut value) = env_borrow.values.get(name) {
                return Some(match value.borrow().deref() {
                    Value::Null | Value::Bool(_) | Value::Number(_) => value.by_val(),
                    _ => value.by_cow()
                })
            }

            if let Some(parent) = &env_borrow.parent {
                env_borrow = parent.borrow();
            } else {
                return None;
            }
        }
    }
}