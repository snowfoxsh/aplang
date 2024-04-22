use std::collections::HashMap;
use std::sync::Arc;
use crate::ast::{Expr, Stmt, Variable};


// variable value types
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    List(Vec<Value>)
}

// context structure, contains variables

// behaviour:
// declaration and assignment are the same
// therefore values will be overwritten
// when declared multiple times

// methods:
// - get variable
// - update variable
// - lookup variable
// do the same for functions
struct Env {
    venv: Vec<Context>,
}

#[derive(Default)]
struct Context {
    variables: HashMap<String, (Value, Arc<Variable>)>,
    //              |^^^^^   |^^^       ^^^^^^^^|> Source code pointer
    //              |        |> Value of symbol
    //              |> Name of symbol

    // functions: HashMap<String, ~~Function~~ >
}

impl Env {
    pub fn layer(&mut self) {
        self.venv.push(Context::default())
    }

    pub fn scrape(&mut self) {
        self.venv.pop().expect("attempted to remove context but failed");
    }

    fn activate(&mut self) -> &mut Context {
        &mut self.venv[0]
    }

    pub fn define(&mut self, variable: Arc<Variable>, value: Value) {
        // add the variable into the context
        self.activate().variables.insert(variable.ident.clone(), (value, variable));
    }
    
    pub fn lookup(&mut self, var: &str) -> Option<&(Value, Arc<Variable>)> {
        self.activate().variables.get(var)
    }

    pub fn lookup_value(&mut self, var: &str) -> Option<&Value> {
        Some(&self.lookup(var)?.0)
    }
    
    pub fn edit(&mut self, var: &str, value: Value) -> Option<Arc<Variable>> {
        // retrieve variable, if not found |-> None
        let (found_value, location) = self.activate().variables.get_mut(var)?;
        
        // 
        *found_value = value;

        Some(location.clone())
    }
}

