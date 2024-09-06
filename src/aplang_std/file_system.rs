use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use crate::interpreter::{FunctionMap, Value};
use crate::std_function;

pub(super) fn file_system() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    // checks if path given exists
    std_function!(functions => fn PATH_EXISTS(path: Value::String) {
        let exists = Path::new(&path).exists();

        return Ok(Value::Bool(exists))
    });

    // returns True if successful
    std_function!(functions => fn FILE_CREATE(file_path: Value::String) {
        return match File::create_new(file_path) {
            Ok(_) => Ok(Value::Bool(true)),
            Err(_) => Ok(Value::Bool(false)),
        }
    });

    // returns File as String
    std_function!(functions => fn FILE_READ(file_path: Value::String) {
        return match fs::read_to_string(file_path) {
            Ok(s) => {
                Ok(Value::String(s))
            }
            Err(_) => {
                // return NULL if the file can't be read
                Ok(Value::Null)
            }
        }
    });

    // Writes to end of file
    std_function!(functions => fn FILE_APPEND(file_path: Value::String, contents: Value) {
        let Ok(mut file) = OpenOptions::new().append(true).open(file_path) else {
            return Ok(Value::Bool(false))
        };

        if let Err(_e) = write!(file, "{}" , contents) {
            return Ok(Value::Bool(false))
        };

        return Ok(Value::Bool(true))
    });

    // Writes to beginning of file, overwriting in the process
    std_function!(functions => fn FILE_OVERWRITE(file_path: Value::String, contents: Value) {
        let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(file_path) else {
            return Ok(Value::Bool(false))
        };

        if let Err(_e) = write!(file, "{}", contents){
            return Ok(Value::Bool(false))
        };

        return Ok(Value::Bool(true))
    });
    
    functions
}