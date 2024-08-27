use std::rc::Rc;
use crate::interpreter::{Env, Value, Interpreter, NativeProcedure};
use crate::{std_function, arity, unwrap_arg_type};

mod time;
mod std_macros;
mod file_system;

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