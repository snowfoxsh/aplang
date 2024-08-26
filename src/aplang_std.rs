use std::fmt::format;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use owo_colors::OwoColorize;
use crate::interpreter::{Env, Interpreter, NativeProcedure, Value};
use crate::interpreter::Value::Bool;
use crate::{std_function, arity, unwrap_arg_type};

impl Env {
    pub(crate) fn inject_std(&mut self) {
        std_function!(self.functions => fn DISPLAY(value: Value) {
            println!("PRINT OUTPUT: {}", value);

            return Ok(Value::Null)
        });

        std_function!(self.functions => fn INSERT(list: Value, i: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            unwrap_arg_type!(i => Value::Number);
            
            // add one because indexed at one
            list.borrow_mut().insert(i as usize - 1, value.clone());

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

        std_function!(self.functions => fn APPEND(list: Value, value: Value) {
            unwrap_arg_type!(list => Value::List);
            list.borrow_mut().push(value.clone());
            
            return Ok(Value::Null)
        });

        /// TIME related functions
        self.functions.insert(
            "TIME".to_string(),
            (Rc::new(NativeProcedure {
                name: "TIME".to_string(),
                arity: 0,
                callable: |s, args: &[Value]| {
                    let now = SystemTime::now();
                    let unix_time_ms = now.duration_since(UNIX_EPOCH).expect("TIME WENT BACKWARDS???");
                    Ok(Value::Number(unix_time_ms.as_millis() as f64))
                },
            }), None),
        );

        /// FILE SYSTEM INTERACTIONS
        std_function!(self.functions => fn PATH_EXISTS(path: Value) {
            unwrap_arg_type!(path => Value::String);

            let exists = Path::new(&path).exists();

            return Ok(Value::Bool(exists))
        });

        // returns a of if it was sucessful or not
        std_function!(self.functions => fn FILE_CREATE(file_path: Value) {
            unwrap_arg_type!(file_path => Value::String);

            return match File::create_new(file_path) {
                Ok(_) => Ok(Value::Bool(true)),
                Err(_) => Ok(Value::Bool(false)),
            }
        });

        std_function!(self.functions => fn FILE_READ(file_path: Value) {
            unwrap_arg_type!(file_path => Value::String);

            return match fs::read_to_string(file_path) {
                Ok(s) => {
                    Ok(Value::String(s))
                }
                Err(_) => {
                    // return NULL if the file cannot be read
                    Ok(Value::Null)
                }
            }
        });

        std_function!(self.functions => fn FILE_APPEND(file_path: Value, contents: Value) {
            unwrap_arg_type!(file_path => Value::String);

            let Ok(mut file) = OpenOptions::new().write(true).append(true).open(file_path) else {
                return Ok(Value::Bool(false))
            };

            if let Err(e) = write!(file, "{}" , contents) {
                return Ok(Value::Bool(false))
            };

            return Ok(Value::Bool(true))
        });

        std_function!(self.functions => fn FILE_OVERWRITE(file_path: Value, contents: Value) {
            unwrap_arg_type!(file_path => Value::String);

            let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(file_path) else {
                return Ok(Value::Bool(false))
            };

            if let Err(e) = write!(file, "{}", contents){
                return Ok(Value::Bool(false))
            };

            return Ok(Value::Bool(true))
        });
        // todo implement directory functions
        // todo: sort function
    }
}