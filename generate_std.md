Write a documentation page in markdown for a given standard library module. The module will contain various functions following a specific format. The documentation should include a brief introduction, a table of contents listing all the functions, and detailed descriptions for each function, including:

- **Function Name**
- **Description** (brief explanation of what the function does)
- **Parameters** (each parameter name, type, and a description)
- **Returns** (the return type and value explanation)
- **Example Usage** (a sample code block showing how to use the function)

Here is the format for the module definition:

```rust
pub(super) fn <module_name>() -> FunctionMap {
    let mut functions = FunctionMap::new();

    std_function!(functions => fn <function_name>(<params>) {
        <function_body>
    });

    // additional functions...

    functions
}
```

Example input:

```rust
pub(super) fn file_system() -> FunctionMap {
    let mut functions = FunctionMap::new();

    std_function!(functions => fn PATH_EXISTS(path: Value::String) {
        let exists = Path::new(&path).exists();
        return Ok(Value::Bool(exists))
    });

    std_function!(functions => fn FILE_CREATE(file_path: Value::String) {
        return match File::create_new(file_path) {
            Ok(_) => Ok(Value::Bool(true)),
            Err(_) => Ok(Value::Bool(false)),
        }
    });

    functions
}
```

Expected markdown output:

```markdown
# file_system Module Documentation

This module provides various file system-related functions for interacting with files and directories.

## Table of Contents

- [PATH_EXISTS](#path_exists)
- [FILE_CREATE](#file_create)

## PATH_EXISTS

**Description:**  
Checks if the given path exists in the file system.

**Parameters:**  
- `path` (Value::String): The path to check for existence.

**Returns:**  
- `Value::Bool`: `true` if the path exists, `false` otherwise.

**Example Usage:**
```rust
exists <- PATH_EXISTS("/path/to/file");
```

---

## FILE_CREATE

**Description:**  
Creates a new file at the specified path. Returns true if successful, false otherwise.

**Parameters:**
- `file_path` (Value::String): The path where the file should be created.

**Returns:**
- `Value::Bool`: `true` if the file was created successfully, `false` otherwise.

**Example Usage:**
```aplang
success <- FILE_CREATE("/path/to/newfile.txt");
```

--- 

...


When you are ready to receive the module respond to this message with ONLY the word "continue"