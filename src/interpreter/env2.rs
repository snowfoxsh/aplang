use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
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
    pub fn define(&mut self, name: String, mut value: ValueRef) -> Option<ValueRef> {
        #[allow(clippy::map_entry)]
        if self.values.contains_key(&name) {
            self.values.insert(name, value)
        } else if let Some(parent) = &self.parent {
            
            // Attempt to update the parent first
            let result = parent.borrow_mut().define(name.clone(), value.clone());

            // Cache a pointer locally (by_ref avoids copying for simple types)
            self.values.insert(name, value.by_ref());

            result
        } else {
            self.values.insert(name, value)
        }
    }
    
    fn get_ref<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();

        if let Some(value) = self.values.get_mut(name) {
            Some(value.by_ref())
        } else if let Some(parent) = &self.parent {
            parent.get_ref(name)
        } else {
            None
        }
    }

    fn get_val<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();

        if let Some(value) = self.values.get_mut(name) {
            let by_val = match value.borrow().deref() {
                Value::Null | Value::Bool(_) | Value::Number(_) => true,
                _ => false,
            };

            Some(if by_val {
                value.by_val()
            } else {
                value.by_ref()
            })
        } else if let Some(parent) = &self.parent {
            parent.get_val(name)
        } else {
            None
        }
    }
}

pub trait Env {
    fn define(&mut self, name: impl Into<String>, value: ValueRef) -> Option<ValueRef>;
    fn get_ref<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef>;
    fn get_val<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef>;
}

impl Env for EnvRef {
    fn define(&mut self, name: impl Into<String>, value: ValueRef) -> Option<ValueRef> {
        let name = name.into();
        let mut binding = self.borrow_mut();
        let env = binding.deref_mut();
        env.define(name, value)
    }

    fn get_ref<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let mut binding = self.borrow_mut();
        let env = binding.deref_mut();
        env.get_ref(name)
    }

    fn get_val<'a>(&mut self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let mut binding = self.borrow_mut();
        let env = binding.deref_mut();
        env.get_val(name)
    }
}

pub trait Layer {
    fn layer(&self) -> EnvRef;
    fn scrape(&self) -> EnvRef;
}

impl Layer for EnvRef {
    fn layer(&self) -> EnvRef {
        Environment::layer(Rc::clone(self))
    }

    fn scrape(&self) -> EnvRef {
        self.borrow().parent.clone().unwrap()
    }
}

pub trait SearchEnv {
    fn get_ref<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef>;
    fn get_val<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef>;
}

impl SearchEnv for EnvRef {
    fn get_ref<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();

        if let Some(value) = self.borrow_mut().values.get_mut(name) {
            Some(value.by_ref())
        } else if let Some(parent) = &self.borrow().parent {
            parent.get_ref(name)
        } else {
            None
        }
    }

    fn get_val<'a>(&self, name: impl Into<&'a str>) -> Option<ValueRef> {
        let name = name.into();

        if let Some(value) = self.borrow_mut().values.get_mut(name) {
            let by_val = match value.borrow().deref() {
                Value::Null | Value::Bool(_) | Value::Number(_) => true,
                _ => false,
            };

            Some(if by_val {
                value.by_val()
            } else {
                value.by_ref()
            })
        } else if let Some(parent) = &self.borrow().parent {
            parent.get_val(name)
        } else {
            None
        }
    }
}
