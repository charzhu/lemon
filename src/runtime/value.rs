//! Runtime values for the Lemon interpreter

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::{Block, Expr, Function, Ident, Pattern};
use crate::lexer::Span;

/// Runtime value
#[derive(Clone)]
pub enum Value {
    /// Unit value (void)
    Unit,
    /// Boolean
    Bool(bool),
    /// Integer (i64)
    Int(i64),
    /// Float (f64)
    Float(f64),
    /// String
    String(String),
    /// Array
    Array(Rc<RefCell<Vec<Value>>>),
    /// Tuple
    Tuple(Vec<Value>),
    /// Struct instance
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    /// Function reference
    Function(Rc<LemonFunction>),
    /// Closure (function with captured environment)
    Closure {
        func: Rc<LemonFunction>,
        env: Rc<RefCell<Environment>>,
    },
    /// Window handle for UI
    Window(Rc<RefCell<UiWindowState>>),
    /// File handle
    File(Rc<RefCell<std::fs::File>>),
    /// TCP listener handle
    TcpListener(Rc<RefCell<TcpListener>>),
    /// TCP stream handle
    TcpStream(Rc<RefCell<TcpStream>>),
    /// Result::Ok variant
    Ok(Box<Value>),
    /// Result::Err variant
    Err(Box<Value>),
    /// Option::Some variant
    Some(Box<Value>),
    /// Option::None variant
    None,
    /// Module namespace
    Module {
        name: String,
        path: PathBuf,
        exports: HashMap<String, Value>,
    },
}

/// A Lemon function (user-defined)
#[derive(Clone)]
pub struct LemonFunction {
    pub name: String,
    pub params: Vec<Ident>,
    pub body: Box<Expr>,
}

impl LemonFunction {
    pub fn from_ast(func: &Function) -> Self {
        // Extract parameter names from patterns
        let params: Vec<Ident> = func
            .params
            .iter()
            .filter_map(|p| {
                match &p.pattern {
                    Pattern::Ident { name, .. } => Some(name.clone()),
                    _ => None, // Skip complex patterns for now
                }
            })
            .collect();

        // The body is the function's body block as an Expr::Block
        let body = func.body.clone().map(|block| {
            Box::new(Expr::Block(block))
        }).unwrap_or_else(|| {
            Box::new(Expr::Block(Block {
                stmts: vec![],
                span: Span { start: 0, end: 0 },
            }))
        });

        Self {
            name: func.name.name.clone(),
            params,
            body,
        }
    }
}

/// UI Window state for graphics
pub struct UiWindowState {
    pub window: minifb::Window,
    pub buffer: Vec<u32>,
    pub width: usize,
    pub height: usize,
}

/// Variable environment with scoping
#[derive(Clone, Default)]
pub struct Environment {
    /// Variable bindings in current scope
    bindings: HashMap<String, Value>,
    /// Parent scope (if any)
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            false
        }
    }
}

impl Value {
    /// Create an Ok result value
    pub fn result_ok(value: Value) -> Value {
        Value::Ok(Box::new(value))
    }

    /// Create an Err result value
    pub fn result_err(value: Value) -> Value {
        Value::Err(Box::new(value))
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::type_error("bool", self.type_name())),
        }
    }

    /// Try to get as int
    pub fn as_int(&self) -> Result<i64, RuntimeError> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(RuntimeError::type_error("int", self.type_name())),
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            _ => Err(RuntimeError::type_error("float", self.type_name())),
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Result<&str, RuntimeError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(RuntimeError::type_error("String", self.type_name())),
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Unit => "()",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "String",
            Value::Array(_) => "Array",
            Value::Tuple(_) => "Tuple",
            Value::Struct { .. } => "Struct",
            Value::Function(_) => "Function",
            Value::Closure { .. } => "Closure",
            Value::Window(_) => "Window",
            Value::File(_) => "File",
            Value::TcpListener(_) => "TcpListener",
            Value::TcpStream(_) => "TcpStream",
            Value::Ok(_) => "Ok",
            Value::Err(_) => "Err",
            Value::Some(_) => "Some",
            Value::None => "None",
            Value::Module { .. } => "Module",
        }
    }

    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Unit => false,
            Value::None => false,
            Value::Err(_) => false,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Unit, Value::Unit) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                let arr = arr.borrow();
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            Value::Struct { name, fields } => {
                write!(f, "{} {{ ", name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Closure { func, .. } => write!(f, "<closure {}>", func.name),
            Value::Window(_) => write!(f, "<Window>"),
            Value::File(_) => write!(f, "<File>"),
            Value::TcpListener(_) => write!(f, "<TcpListener>"),
            Value::TcpStream(_) => write!(f, "<TcpStream>"),
            Value::Ok(v) => write!(f, "Ok({})", v),
            Value::Err(v) => write!(f, "Err({})", v),
            Value::Some(v) => write!(f, "Some({})", v),
            Value::None => write!(f, "None"),
            Value::Module { name, .. } => write!(f, "<module {}>", name),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Control flow signal kind
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// A real runtime error
    Error,
    /// break [value] inside a loop
    Break,
    /// continue inside a loop
    Continue,
    /// return [value] from a function
    Return,
}

/// Runtime error (also used for control flow signals)
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub kind: ErrorKind,
    /// Optional value carried by break/return
    pub value: Option<Box<Value>>,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ErrorKind::Error,
            value: None,
        }
    }

    pub fn break_signal(value: Value) -> Self {
        Self {
            message: "break".into(),
            kind: ErrorKind::Break,
            value: Some(Box::new(value)),
        }
    }

    pub fn continue_signal() -> Self {
        Self {
            message: "continue".into(),
            kind: ErrorKind::Continue,
            value: None,
        }
    }

    pub fn return_signal(value: Value) -> Self {
        Self {
            message: "return".into(),
            kind: ErrorKind::Return,
            value: Some(Box::new(value)),
        }
    }

    pub fn type_error(expected: &str, got: &str) -> Self {
        Self::new(format!("type error: expected {}, got {}", expected, got))
    }

    pub fn undefined_variable(name: &str) -> Self {
        Self::new(format!("undefined variable: {}", name))
    }

    pub fn undefined_function(name: &str) -> Self {
        Self::new(format!("undefined function: {}", name))
    }

    pub fn arity_mismatch(expected: usize, got: usize) -> Self {
        Self::new(format!(
            "wrong number of arguments: expected {}, got {}",
            expected, got
        ))
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
