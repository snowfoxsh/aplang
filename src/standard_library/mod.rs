use std::collections::HashMap;
use rand::Rng;
use crate::interpreter::FunctionMap;
use crate::std_function;
use crate::standard_library::io::input;
use crate::interpreter::Value;

mod time;
mod std_macros;
mod file_system;
mod math;
mod io;
mod strings;
mod style;


#[derive(Debug, Clone, Default)]
pub struct Modules {
    modules: HashMap<String, fn() -> FunctionMap>
}

impl Modules {
    fn inject(&mut self) {
        self.register("CORE", std_core);
        self.register("FS", file_system::file_system);
        self.register("TIME", time::time);
        self.register("MATH", math::std_math);
        self.register("IO", io::std_io);
        self.register("STRING", strings::std_strings);
        self.register("STYLE", style::std_style);
    }
    pub fn init() -> Self {
        // create bland hashmap of modules
        let mut modules = Self::default();
        // load in the module functions
        modules.inject();
        // return handle
        modules
    }

    pub fn lookup(&self, module: &str) -> Option<&fn() -> FunctionMap> {
        self.modules.get(module)
    }

    pub fn register(&mut self, module_name: &str, injector: fn() -> FunctionMap) {

        // if a module is defined again with the same name, then the prev will be discarded
        let _ = self.modules.insert(module_name.to_string(), injector);
    }
}

fn std_core() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    std_function!(functions => fn DISPLAY(value: Value) {
        println!("{}", value);

        return Ok(Value::Null)
    });
    
    std_function!(functions => fn DISPLAY_NOLN(value: Value) {
        print!("{}", value);

        return Ok(Value::Null)
    });

    std_function!(functions => fn INPUT() {
        let result = input("").expect("Failed to get user input! Critical Failure");
        Ok(Value::String(result))
    });

    std_function!(functions => fn INSERT(list: Value::List, i: Value::Number, value: Value) {
        // subtract one because indexed at one
        list.borrow_mut().insert(i as usize - 1, value.clone());

        return Ok(Value::Null)
    });
    
    std_function!(functions => fn APPEND(list: Value::List, value: Value) {
        list.borrow_mut().push(value.clone());
        
        return Ok(Value::Null)
    });
    
    std_function!(functions => fn REMOVE(list: Value::List, i: Value::Number) {
        // todo instead of panic with default hook make this return a nice error
        let poped = list.borrow_mut().remove(i as usize - 1);
        return Ok(poped);
    });
    
     std_function!(functions => fn LENGTH(list: Value::List) {
        let len = list.borrow().len() as f64;

        return Ok(Value::Number(len))
    });

    // return a random integer from a to b including a and b
    std_function!(functions => fn RANDOM(a: Value::Number, b: Value::Number) {
        let mut rng = rand::thread_rng();
        let result = rng.gen_range(a as i64..=b as i64);

        return Ok(Value::Number(result as f64))
    });

    functions
}