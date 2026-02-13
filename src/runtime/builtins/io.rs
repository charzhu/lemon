//! I/O built-in functions

use std::io::{self, BufRead, Write};

use crate::runtime::value::{RuntimeError, Value};

/// print(value...) - Print values without newline
pub fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    io::stdout().flush().ok();
    Ok(Value::Unit)
}

/// println(value...) - Print values with newline
pub fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Unit)
}

/// eprint(value...) - Print to stderr without newline
pub fn builtin_eprint(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            eprint!(" ");
        }
        eprint!("{}", arg);
    }
    io::stderr().flush().ok();
    Ok(Value::Unit)
}

/// eprintln(value...) - Print to stderr with newline
pub fn builtin_eprintln(args: &[Value]) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            eprint!(" ");
        }
        eprint!("{}", arg);
    }
    eprintln!();
    Ok(Value::Unit)
}

/// input() - Read a line from stdin
pub fn builtin_input(args: &[Value]) -> Result<Value, RuntimeError> {
    // Optional prompt
    if !args.is_empty() {
        print!("{}", args[0]);
        io::stdout().flush().ok();
    }

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line).map_err(|e| {
        RuntimeError::new(format!("failed to read input: {}", e))
    })?;

    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(Value::String(line))
}

/// len(value) - Get length of string or array
pub fn builtin_len(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Int(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Int(arr.borrow().len() as i64)),
        Value::Tuple(items) => Ok(Value::Int(items.len() as i64)),
        v => Err(RuntimeError::new(format!(
            "len() not supported for {}",
            v.type_name()
        ))),
    }
}

/// str(value) - Convert value to string
pub fn builtin_str(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }
    Ok(Value::String(format!("{}", args[0])))
}

/// int(value) - Convert value to integer
pub fn builtin_int(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Int(i) => Ok(Value::Int(*i)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::String(s) => {
            s.parse::<i64>()
                .map(Value::Int)
                .map_err(|_| RuntimeError::new(format!("cannot convert '{}' to int", s)))
        }
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        v => Err(RuntimeError::new(format!(
            "cannot convert {} to int",
            v.type_name()
        ))),
    }
}

/// float(value) - Convert value to float
pub fn builtin_float(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Int(i) => Ok(Value::Float(*i as f64)),
        Value::String(s) => {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| RuntimeError::new(format!("cannot convert '{}' to float", s)))
        }
        v => Err(RuntimeError::new(format!(
            "cannot convert {} to float",
            v.type_name()
        ))),
    }
}

/// type_of(value) - Get type name of value
pub fn builtin_type_of(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

/// random(min, max) - Generate random integer in range [min, max]
pub fn builtin_random(args: &[Value]) -> Result<Value, RuntimeError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let min = args[0].as_int()?;
    let max = args[1].as_int()?;

    if min > max {
        return Err(RuntimeError::new("random: min must be <= max"));
    }

    // Get high-resolution time for randomness
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Use hasher to improve distribution
    let mut hasher = DefaultHasher::new();
    nanos.hash(&mut hasher);
    let hash = hasher.finish();

    let range = (max - min + 1) as u64;
    let value = min + ((hash % range) as i64);

    Ok(Value::Int(value))
}
