use std::collections::HashMap;
use crate::interpreter::{Env, Value, Interpreter, NativeProcedure};
use crate::{std_function, arity, unwrap_arg_type};
use crate::errors::RuntimeError;

mod time;
mod std_macros;
mod file_system;


#[derive(Debug, Clone, Default)]
pub struct Modules {
    modules: HashMap<String, fn(&mut Env)>
}

impl Modules {
    fn inject(&mut self) {
        self.register("core", std_core);
        self.register("fs", file_system::file_system);
        self.register("time", time::time);
        // math lib, sin cos etc. rounding
        // casting to int
        // input functions
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

fn std_core(env: &mut Env) {
    std_function!(env.functions => fn DISPLAY(value: Value) {
        println!("{}", value);

        return Ok(Value::Null)
    });
    
    std_function!(env.functions => fn INSERT(list: Value, i: Value, value: Value) {
        unwrap_arg_type!(list => Value::List);
        unwrap_arg_type!(i => Value::Number);
        
        // subtract one because indexed at one
        list.borrow_mut().insert(i as usize - 1, value.clone());

        return Ok(Value::Null)
    });
    
    std_function!(env.functions => fn APPEND(list: Value, value: Value) {
        unwrap_arg_type!(list => Value::List);
        
        list.borrow_mut().push(value.clone());
        
        return Ok(Value::Null)
    });
    
    std_function!(env.functions => fn REMOVE(list: Value, i: Value) {
        unwrap_arg_type!(list => Value::List);
        unwrap_arg_type!(i => Value::Number);

        // todo instead of panic with default hook make this return a nice error
        let poped = list.borrow_mut().remove(i as usize - 1);

        return Ok(poped);
    });
    
     std_function!(env.functions => fn LENGTH(list: Value) {
        unwrap_arg_type!(list => Value::List);

        let len = list.borrow().len() as f64;

        return Ok(Value::Number(len))
    });
}