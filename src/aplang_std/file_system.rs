use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::io::Write;
use crate::errors::RuntimeError;
use crate::interpreter::{Env, Value, NativeProcedure, Interpreter};
use miette::SourceSpan;
use crate::{std_function, arity, unwrap_arg_type};


pub(super) fn file_system(env: &mut Env) {
    
    // checks if path given exists
    std_function!(env.functions => fn PATH_EXISTS(path: Value::String) {
        let exists = Path::new(&path).exists();

        return Ok(Value::Bool(exists))
    });

    // returns True if successful
    std_function!(env.functions => fn FILE_CREATE(file_path: Value::String) {
        return match File::create_new(file_path) {
            Ok(_) => Ok(Value::Bool(true)),
            Err(_) => Ok(Value::Bool(false)),
        }
    });

    // returns File as String
    std_function!(env.functions => fn FILE_READ(file_path: Value::String) {
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

    // Writes to end of file
    std_function!(env.functions => fn FILE_APPEND(file_path: Value::String, contents: Value) {
        let Ok(mut file) = OpenOptions::new().append(true).open(file_path) else {
            return Ok(Value::Bool(false))
        };

        if let Err(_e) = write!(file, "{}" , contents) {
            return Ok(Value::Bool(false))
        };

        return Ok(Value::Bool(true))
    });

    // Writes to beginning of file, overwriting in the process
    std_function!(env.functions => fn FILE_OVERWRITE(file_path: Value::String, contents: Value) {
        let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(file_path) else {
            return Ok(Value::Bool(false))
        };

        if let Err(_e) = write!(file, "{}", contents){
            return Ok(Value::Bool(false))
        };

        return Ok(Value::Bool(true))
    });
}