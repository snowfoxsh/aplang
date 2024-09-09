use std::cell::RefCell;
use std::fs;
use std::fs::{OpenOptions};
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use crate::interpreter::{FunctionMap, Value};
use crate::std_function;

pub(super) fn file_system() -> FunctionMap {
    let mut functions = FunctionMap::new();
    
    // checks if path given exists
    std_function!(functions => fn PATH_EXISTS(path: Value::String) {
        let exists = Path::new(&path).exists();

        return Ok(Value::Bool(exists))
    });

    std_function!(functions => fn PATH_IS_FILE(path: Value::String) {
        let path = Path::new(&path);

        return Ok(Value::Bool(path.is_file()))
    });

    std_function!(functions => fn PATH_IS_DIRECTORY(path: Value::String) {
        let path = Path::new(&path);

        return Ok(Value::Bool(path.is_dir()))
    });

    // returns True if successful
    std_function!(functions => fn FILE_CREATE(file_path: Value::String) {
        return match fs::File::create_new(file_path) {
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

    std_function!(functions => fn READ_DIRECTORY(path: Value::String) {
        let paths = fs::read_dir(path).expect("Failed to read directory");

        let mut dir_list = Vec::new();
        for path in paths {
            let path = path.unwrap().path();
            let path = path.to_str().unwrap();
            dir_list.push(Value::String(path.to_string()));
        }

        return Ok(Value::List(Rc::new(RefCell::new(dir_list))))
    });

    functions
}