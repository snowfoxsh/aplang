use std::cell::RefCell;
use crate::interpreter::{Env, Interpreter, NativeProcedure, Value};
use turtle::Turtle;
use crate::{std_function, arity};
use std::rc::Rc;
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use crate::interpreter::Value::Bool;

impl Env {

    pub(crate) fn inject_std_turtle(&mut self) {

        // TURTLE RELATED FUNCTIONS
        self.functions.insert(
            String::from("TURTLE_INIT"),
            (Rc::new(NativeProcedure {
                name: String::from("TURTLE_INIT"),
                arity: 0,
                callable: | interpreter, args | {

                    interpreter.turtle = Some(Rc::new(RefCell::new(Turtle::new())));

                    return Ok(Value::Null);
                },
            }), None)
        );
        
        self.functions.insert(
            String::from("TURTLE_KILL"),
            (Rc::new(NativeProcedure {
                name: String::from("TURTLE_KILL"),
                arity: 0,
                callable: | interpreter, args | { 
                    // todo: this does nothing
                    //  turtle does not exit gracefully (this is an understatement)
                    interpreter.turtle = None;
                    
                    return Ok(Value::Null);
                },
            }), None)
        );
        
        self.functions.insert(
            String::from("MOVE_FORWARD"),
            (Rc::new(NativeProcedure {
                name: String::from("MOVE_FORWARD"),
                arity: 0,
                callable: | interpreter, args | {

                    if let Some(mut turtle) = &interpreter.turtle.as_ref() {
                        // move turtle forward by 100 (as default for now)
                        turtle.borrow_mut().forward(100.0);
                    } else {
                        return Err(String::from("Turtle Not Initialized!"));
                    };

                    return Ok(Value::Null);
                },
            }), None)
        );
    }
}