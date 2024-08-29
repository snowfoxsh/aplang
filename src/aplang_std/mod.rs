use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use crate::interpreter::{Env, Value, Interpreter, NativeProcedure};
use crate::{std_function, arity, unwrap_arg_type};
use miette::miette;
use crate::aplang_std::file_system::file_system;

mod time;
mod std_macros;
mod file_system;

fn std_core(env: &mut Env) {
    todo!()
}

impl Env {
    pub(crate) fn inject_std_default(&mut self) {
        // All std functions injected by default
        
        std_function!(self.functions => fn DISPLAY(value: Value) {
            println!("PRINT OUTPUT: {}", value);

            return Ok(Value::Null)
        });
        
        std_function!(self.functions => fn INSERT(list: Value, i: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            unwrap_arg_type!(i => Value::Number);
            
            // subtract one because indexed at one
            list.borrow_mut().insert(i as usize - 1, value.clone());

            return Ok(Value::Null)
        });
        
        std_function!(self.functions => fn APPEND(list: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            
            list.borrow_mut().push(value.clone());
            
            return Ok(Value::Null)
        });
        
        std_function!(self.functions => fn REMOVE(list: Value, i: Value) {
            unwrap_arg_type!(list => Value::List);
            unwrap_arg_type!(i => Value::Number);

            // todo instead of panic with default hook make this return a nice error
            let poped = list.borrow_mut().remove(i as usize - 1);

            return Ok(poped);
        });
        
         std_function!(self.functions => fn LENGTH(list: Value) {
            unwrap_arg_type!(list => Value::List);

            let len = list.borrow().len() as f64;

            return Ok(Value::Number(len))
        });
    }
}

#[derive(Debug, Clone, Default)]
pub struct Modules {
    modules: HashMap<String, fn(&mut Env)>
}


impl Modules {
    fn inject(&mut self) {
        self.register("core", std_core);
        self.register("fs", file_system::file_system);
        self.register("time", time::time);
    }
    pub fn init() -> Self {
        // create bland hashmap of modules
        let mut modules = Self::default();
        // load in the module functions
        modules.inject();
        // return handle
        modules
    }

    pub fn lookup(&self, module: &str) -> Option<&fn(&mut Env)> {
        self.modules.get(module)
    }

    pub fn register(&mut self, module_name: &str, injector: fn(&mut Env)) {

        // if a module is defined again with the same name then the prev will be discarded
        let _ = self.modules.insert(module_name.to_string(), injector);
    }
}
