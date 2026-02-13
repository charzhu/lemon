//! File system built-in functions

use std::fs;
use std::path::Path;

use crate::runtime::value::{RuntimeError, Value};

/// fs::read_string(path: String) -> Result<String, Error>
pub fn builtin_fs_read_string(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    match fs::read_to_string(path) {
        Ok(content) => Ok(Value::Ok(Box::new(Value::String(content)))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::write_string(path: String, content: String) -> Result<(), Error>
pub fn builtin_fs_write_string(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let path = args[0].as_string()?;
    let content = args[1].as_string()?;

    match fs::write(path, content) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::read_bytes(path: String) -> Result<Array<int>, Error>
pub fn builtin_fs_read_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    match fs::read(path) {
        Ok(bytes) => {
            let values: Vec<Value> = bytes.into_iter().map(|b| Value::Int(b as i64)).collect();
            Ok(Value::Ok(Box::new(Value::Array(std::rc::Rc::new(
                std::cell::RefCell::new(values),
            )))))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::write_bytes(path: String, data: Array<int>) -> Result<(), Error>
pub fn builtin_fs_write_bytes(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let path = args[0].as_string()?;

    let bytes = match &args[1] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let mut bytes = Vec::with_capacity(arr.len());
            for val in arr.iter() {
                match val {
                    Value::Int(i) => bytes.push(*i as u8),
                    _ => return Err(RuntimeError::type_error("int", val.type_name())),
                }
            }
            bytes
        }
        _ => return Err(RuntimeError::type_error("Array", args[1].type_name())),
    };

    match fs::write(path, bytes) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::exists(path: String) -> bool
pub fn builtin_fs_exists(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    Ok(Value::Bool(Path::new(path).exists()))
}

/// fs::is_file(path: String) -> bool
pub fn builtin_fs_is_file(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    Ok(Value::Bool(Path::new(path).is_file()))
}

/// fs::is_dir(path: String) -> bool
pub fn builtin_fs_is_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    Ok(Value::Bool(Path::new(path).is_dir()))
}

/// fs::remove(path: String) -> Result<(), Error>
pub fn builtin_fs_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    let path = Path::new(path);

    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match result {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::mkdir(path: String) -> Result<(), Error>
pub fn builtin_fs_mkdir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    match fs::create_dir_all(path) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::list_dir(path: String) -> Result<Array<String>, Error>
pub fn builtin_fs_list_dir(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut names = Vec::new();
            for entry in entries {
                match entry {
                    Ok(e) => {
                        if let Some(name) = e.file_name().to_str() {
                            names.push(Value::String(name.to_string()));
                        }
                    }
                    Err(e) => {
                        return Ok(Value::Err(Box::new(Value::String(e.to_string()))));
                    }
                }
            }
            Ok(Value::Ok(Box::new(Value::Array(std::rc::Rc::new(
                std::cell::RefCell::new(names),
            )))))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::copy(src: String, dst: String) -> Result<(), Error>
pub fn builtin_fs_copy(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let src = args[0].as_string()?;
    let dst = args[1].as_string()?;

    match fs::copy(src, dst) {
        Ok(_) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::rename(src: String, dst: String) -> Result<(), Error>
pub fn builtin_fs_rename(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let src = args[0].as_string()?;
    let dst = args[1].as_string()?;

    match fs::rename(src, dst) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::append(path: String, content: String) -> Result<(), Error>
pub fn builtin_fs_append(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let path = args[0].as_string()?;
    let content = args[1].as_string()?;

    use std::fs::OpenOptions;
    use std::io::Write;

    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(content.as_bytes()));

    match result {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// fs::cwd() -> String
pub fn builtin_fs_cwd(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::arity_mismatch(0, args.len()));
    }

    match std::env::current_dir() {
        Ok(path) => Ok(Value::String(path.to_string_lossy().to_string())),
        Err(e) => Err(RuntimeError::new(format!("failed to get cwd: {}", e))),
    }
}

/// fs::absolute(path: String) -> Result<String, Error>
pub fn builtin_fs_absolute(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    let path = args[0].as_string()?;
    match fs::canonicalize(path) {
        Ok(abs_path) => Ok(Value::Ok(Box::new(Value::String(
            abs_path.to_string_lossy().to_string(),
        )))),
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}
