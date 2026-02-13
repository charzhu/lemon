//! Tree-walking interpreter for Lemon

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ast::{
    BinOp, Block, Expr, ExprPath, FieldInit, Ident, Item, Literal, NewArgs,
    Pattern, SourceFile, Stmt, UnaryOp, UseDecl, UseTree, ModDecl, Visibility,
};
use crate::lexer::Span;

use super::builtins::BuiltinRegistry;
use super::value::{Environment, ErrorKind, LemonFunction, RuntimeError, Value};

/// The Lemon interpreter
pub struct Interpreter {
    /// Global environment (top-level functions and variables)
    globals: Rc<RefCell<Environment>>,
    /// Current local environment
    env: Rc<RefCell<Environment>>,
    /// Built-in functions
    builtins: BuiltinRegistry,
    /// Loaded modules cache
    loaded_modules: HashMap<PathBuf, Value>,
    /// Module search paths (lemon lib directory, current working directory)
    module_search_paths: Vec<PathBuf>,
    /// Current file being executed (for relative imports)
    current_file: Option<PathBuf>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));
        let mut interp = Self {
            globals: globals.clone(),
            env: globals,
            builtins: BuiltinRegistry::new(),
            loaded_modules: HashMap::new(),
            module_search_paths: Vec::new(),
            current_file: None,
        };
        interp.init_lemon_lib_path();
        interp.import_prelude();
        interp
    }

    /// Set the current file being executed (for relative imports)
    pub fn set_current_file(&mut self, path: PathBuf) {
        self.current_file = Some(path);
    }

    /// Import the lemon prelude (lemon/mod.lemon) into globals
    fn import_prelude(&mut self) {
        // Load lemon/mod.lemon and import all exports into globals
        if let Some(path) = self.find_lemon_file("mod.lemon") {
            if let Ok(module) = self.load_module(&path) {
                if let Value::Module { exports, .. } = module {
                    for (name, value) in exports {
                        self.globals.borrow_mut().define(name, value);
                    }
                }
            }
        }
    }

    /// Initialize the lemon standard library search paths
    fn init_lemon_lib_path(&mut self) {
        // 1. Check LEMON_LIB environment variable
        if let Ok(path) = std::env::var("LEMON_LIB") {
            self.module_search_paths.push(PathBuf::from(path));
        }

        // 2. Check relative to executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                self.module_search_paths.push(dir.join("lemon"));
            }
        }

        // 3. Check current working directory
        if let Ok(cwd) = std::env::current_dir() {
            self.module_search_paths.push(cwd.join("lemon"));
        }

        // 4. Fallback to ./lemon
        self.module_search_paths.push(PathBuf::from("lemon"));
    }

    /// Find a file in the lemon lib paths
    fn find_lemon_file(&self, name: &str) -> Option<PathBuf> {
        for search_path in &self.module_search_paths {
            let path = search_path.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Resolve a module path to a file path
    /// "lemon" -> lemon/mod.lemon
    /// "lemon.fs" -> lemon/fs.lemon
    /// "foo" -> foo.lemon or foo/mod.lemon
    fn resolve_module_path(&self, segments: &[String]) -> Result<PathBuf, RuntimeError> {
        if segments.is_empty() {
            return Err(RuntimeError::new("empty module path"));
        }

        // Check if it's a lemon stdlib module
        if segments[0] == "lemon" {
            if segments.len() == 1 {
                // Just "lemon" -> lemon/mod.lemon
                if let Some(path) = self.find_lemon_file("mod.lemon") {
                    return Ok(path);
                }
            } else {
                // "lemon.fs" -> lemon/fs.lemon
                let relative = segments[1..].join("/") + ".lemon";
                if let Some(path) = self.find_lemon_file(&relative) {
                    return Ok(path);
                }
            }
        }

        // Check relative to current file
        if let Some(current) = &self.current_file {
            if let Some(dir) = current.parent() {
                // Try as file: foo.lemon
                let file_path = dir.join(segments.join("/") + ".lemon");
                if file_path.exists() {
                    return Ok(file_path);
                }

                // Try as directory: foo/mod.lemon
                let mod_path = dir.join(segments.join("/")).join("mod.lemon");
                if mod_path.exists() {
                    return Ok(mod_path);
                }
            }
        }

        // Try in current working directory
        if let Ok(cwd) = std::env::current_dir() {
            let file_path = cwd.join(segments.join("/") + ".lemon");
            if file_path.exists() {
                return Ok(file_path);
            }

            let mod_path = cwd.join(segments.join("/")).join("mod.lemon");
            if mod_path.exists() {
                return Ok(mod_path);
            }
        }

        Err(RuntimeError::new(format!(
            "module not found: {}",
            segments.join(".")
        )))
    }

    /// Load a module from a file path
    fn load_module(&mut self, path: &PathBuf) -> Result<Value, RuntimeError> {
        // Check cache
        if let Some(cached) = self.loaded_modules.get(path) {
            return Ok(cached.clone());
        }

        // Read and parse the file
        let source = std::fs::read_to_string(path).map_err(|e| {
            RuntimeError::new(format!("cannot read module '{}': {}", path.display(), e))
        })?;

        let ast = crate::parser::parse(&source).map_err(|e| {
            RuntimeError::new(format!("parse error in module '{}': {:?}", path.display(), e))
        })?;

        // Save current state
        let old_file = self.current_file.take();
        let old_env = self.env.clone();

        // Set up module context
        self.current_file = Some(path.clone());
        self.env = Rc::new(RefCell::new(Environment::with_parent(self.globals.clone())));

        // Collect exports from the module
        let mut exports = HashMap::new();

        for item in &ast.items {
            match item {
                Item::Function(func) => {
                    let lemon_func = LemonFunction::from_ast(func);
                    let value = Value::Function(Rc::new(lemon_func));

                    // Add to module environment
                    self.env.borrow_mut().define(func.name.name.clone(), value.clone());

                    // If public, add to exports
                    if matches!(func.visibility, Visibility::Public) {
                        exports.insert(func.name.name.clone(), value);
                    }
                }
                Item::Const(const_def) => {
                    let value = self.eval_expr(&const_def.value)?;

                    // Add to module environment
                    self.env.borrow_mut().define(const_def.name.name.clone(), value.clone());

                    // If public, add to exports
                    if matches!(const_def.visibility, Visibility::Public) {
                        exports.insert(const_def.name.name.clone(), value);
                    }
                }
                Item::Use(use_decl) => {
                    self.process_use(use_decl)?;
                }
                Item::Mod(mod_decl) => {
                    self.process_mod(mod_decl)?;
                }
                _ => {}
            }
        }

        // Restore state
        self.current_file = old_file;
        self.env = old_env;

        // Create module value
        let module_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let module = Value::Module {
            name: module_name,
            path: path.clone(),
            exports,
        };

        // Cache it
        self.loaded_modules.insert(path.clone(), module.clone());

        Ok(module)
    }

    /// Process a use declaration
    fn process_use(&mut self, use_decl: &UseDecl) -> Result<(), RuntimeError> {
        self.import_use_tree(&use_decl.tree, &[])
    }

    /// Import items from a use tree
    fn import_use_tree(&mut self, tree: &UseTree, prefix: &[String]) -> Result<(), RuntimeError> {
        match tree {
            UseTree::Path(ident, rest) => {
                let mut new_prefix = prefix.to_vec();
                new_prefix.push(ident.name.clone());
                self.import_use_tree(rest, &new_prefix)
            }
            UseTree::Name(ident) => {
                // use foo::bar -> import bar from foo module
                let mut full_path = prefix.to_vec();
                full_path.push(ident.name.clone());

                // Try to resolve as a module first
                if let Ok(module_path) = self.resolve_module_path(&full_path) {
                    let module = self.load_module(&module_path)?;
                    self.env.borrow_mut().define(ident.name.clone(), module);
                } else if !prefix.is_empty() {
                    // Try to get item from parent module
                    let module_path = self.resolve_module_path(prefix)?;
                    let module = self.load_module(&module_path)?;
                    if let Value::Module { exports, .. } = module {
                        if let Some(value) = exports.get(&ident.name) {
                            self.env.borrow_mut().define(ident.name.clone(), value.clone());
                        } else {
                            return Err(RuntimeError::new(format!(
                                "'{}' not found in module '{}'",
                                ident.name,
                                prefix.join(".")
                            )));
                        }
                    }
                } else {
                    return Err(RuntimeError::new(format!(
                        "cannot resolve '{}'",
                        full_path.join(".")
                    )));
                }
                Ok(())
            }
            UseTree::Rename(ident, alias) => {
                let mut full_path = prefix.to_vec();
                full_path.push(ident.name.clone());

                if let Ok(module_path) = self.resolve_module_path(&full_path) {
                    let module = self.load_module(&module_path)?;
                    self.env.borrow_mut().define(alias.name.clone(), module);
                } else if !prefix.is_empty() {
                    let module_path = self.resolve_module_path(prefix)?;
                    let module = self.load_module(&module_path)?;
                    if let Value::Module { exports, .. } = module {
                        if let Some(value) = exports.get(&ident.name) {
                            self.env.borrow_mut().define(alias.name.clone(), value.clone());
                        } else {
                            return Err(RuntimeError::new(format!(
                                "'{}' not found in module '{}'",
                                ident.name,
                                prefix.join(".")
                            )));
                        }
                    }
                }
                Ok(())
            }
            UseTree::Glob => {
                // use foo::* -> import all exports from foo
                let module_path = self.resolve_module_path(prefix)?;
                let module = self.load_module(&module_path)?;
                if let Value::Module { exports, .. } = module {
                    for (name, value) in exports {
                        self.env.borrow_mut().define(name, value);
                    }
                }
                Ok(())
            }
            UseTree::Group(items) => {
                // use foo::{bar, baz}
                for item in items {
                    self.import_use_tree(item, prefix)?;
                }
                Ok(())
            }
        }
    }

    /// Process a mod declaration
    fn process_mod(&mut self, mod_decl: &ModDecl) -> Result<(), RuntimeError> {
        let module = if let Some(items) = &mod_decl.items {
            // Inline module: mod foo { ... }
            let mut exports = HashMap::new();

            // Save current env
            let old_env = self.env.clone();
            self.env = Rc::new(RefCell::new(Environment::with_parent(self.globals.clone())));

            for item in items {
                match item {
                    Item::Function(func) => {
                        let lemon_func = LemonFunction::from_ast(func);
                        let value = Value::Function(Rc::new(lemon_func));
                        self.env.borrow_mut().define(func.name.name.clone(), value.clone());
                        if matches!(func.visibility, Visibility::Public) {
                            exports.insert(func.name.name.clone(), value);
                        }
                    }
                    Item::Const(const_def) => {
                        let value = self.eval_expr(&const_def.value)?;
                        self.env.borrow_mut().define(const_def.name.name.clone(), value.clone());
                        if matches!(const_def.visibility, Visibility::Public) {
                            exports.insert(const_def.name.name.clone(), value);
                        }
                    }
                    _ => {}
                }
            }

            // Restore env
            self.env = old_env;

            Value::Module {
                name: mod_decl.name.name.clone(),
                path: PathBuf::new(),
                exports,
            }
        } else {
            // External module: mod foo; -> load foo.lemon
            let path = self.resolve_module_path(&[mod_decl.name.name.clone()])?;
            self.load_module(&path)?
        };

        self.env.borrow_mut().define(mod_decl.name.name.clone(), module);
        Ok(())
    }

    /// Execute a source file
    pub fn execute(&mut self, file: &SourceFile) -> Result<Value, RuntimeError> {
        // First pass: process use/mod declarations and collect functions
        for item in &file.items {
            match item {
                Item::Use(use_decl) => self.process_use(use_decl)?,
                Item::Mod(mod_decl) => self.process_mod(mod_decl)?,
                _ => self.define_item(item)?,
            }
        }

        // Call main() if it exists
        let main_fn = self.globals.borrow().get("main");
        if let Some(main_fn) = main_fn {
            self.call_value(&main_fn, vec![])
        } else {
            // No main function - just return unit
            Ok(Value::Unit)
        }
    }

    /// Define a top-level item
    fn define_item(&mut self, item: &Item) -> Result<(), RuntimeError> {
        match item {
            Item::Function(func) => {
                let lemon_func = LemonFunction::from_ast(func);
                self.globals
                    .borrow_mut()
                    .define(func.name.name.clone(), Value::Function(Rc::new(lemon_func)));
            }
            Item::Const(const_def) => {
                let value = self.eval_expr(&const_def.value)?;
                self.globals
                    .borrow_mut()
                    .define(const_def.name.name.clone(), value);
            }
            // TODO: Handle other items (structs, enums, etc.)
            _ => {}
        }
        Ok(())
    }

    /// Evaluate a block
    fn eval_block(&mut self, block: &Block) -> Result<Value, RuntimeError> {
        let mut result = Value::Unit;

        for stmt in &block.stmts {
            result = self.eval_stmt(stmt)?;

            // Check for early return
            if matches!(result, Value::Ok(_) | Value::Err(_)) {
                // Propagate return values
            }
        }

        Ok(result)
    }

    /// Evaluate a statement
    fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Let {
                pattern,
                value,
                ..
            } => {
                let val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    Value::Unit
                };

                self.bind_pattern(pattern, val)?;
                Ok(Value::Unit)
            }
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::Item(item) => {
                self.define_item(item)?;
                Ok(Value::Unit)
            }
        }
    }

    /// Bind a pattern to a value
    fn bind_pattern(&mut self, pattern: &Pattern, value: Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Ident { name, .. } => {
                self.env.borrow_mut().define(name.name.clone(), value);
                Ok(())
            }
            Pattern::Wildcard(_) => Ok(()), // Ignore
            Pattern::Tuple(elements, _) => {
                if let Value::Tuple(values) = value {
                    if elements.len() != values.len() {
                        return Err(RuntimeError::new("tuple pattern length mismatch"));
                    }
                    for (pat, val) in elements.iter().zip(values) {
                        self.bind_pattern(pat, val)?;
                    }
                    Ok(())
                } else {
                    Err(RuntimeError::type_error("tuple", value.type_name()))
                }
            }
            _ => Err(RuntimeError::new(format!(
                "unsupported pattern: {:?}",
                pattern
            ))),
        }
    }

    /// Evaluate an expression
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit, _) => self.eval_literal(lit),

            Expr::Path(path, _) => self.eval_path(path),

            Expr::Binary {
                op, left, right, ..
            } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binary(*op, left_val, right_val)
            }

            Expr::Unary { op, expr, .. } => {
                let val = self.eval_expr(expr)?;
                self.eval_unary(*op, val)
            }

            Expr::Call { func, args, .. } => {
                let func_val = self.eval_expr(func)?;
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                self.call_value(&func_val, arg_values)
            }

            Expr::Block(block) => self.eval_block(block),

            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond = self.eval_expr(condition)?;
                if cond.is_truthy() {
                    self.eval_block(then_branch)
                } else if let Some(else_expr) = else_branch {
                    self.eval_expr(else_expr)
                } else {
                    Ok(Value::Unit)
                }
            }

            Expr::While {
                condition, body, ..
            } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    match self.eval_block(body) {
                        Ok(_) => {}
                        Err(e) if e.kind == ErrorKind::Break => {
                            return Ok(e.value.map(|v| *v).unwrap_or(Value::Unit));
                        }
                        Err(e) if e.kind == ErrorKind::Continue => {
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ok(Value::Unit)
            }

            Expr::Loop { body, .. } => {
                loop {
                    match self.eval_block(body) {
                        Ok(_) => {}
                        Err(e) if e.kind == ErrorKind::Break => {
                            return Ok(e.value.map(|v| *v).unwrap_or(Value::Unit));
                        }
                        Err(e) if e.kind == ErrorKind::Continue => {
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            Expr::For {
                pattern,
                iter,
                body,
                ..
            } => {
                let iterable = self.eval_expr(iter)?;

                match iterable {
                    Value::Array(arr) => {
                        let items: Vec<Value> = arr.borrow().clone();
                        for item in items {
                            // Create new scope for each iteration
                            let old_env = self.env.clone();
                            self.env = Rc::new(RefCell::new(Environment::with_parent(old_env.clone())));

                            self.bind_pattern(pattern, item)?;
                            match self.eval_block(body) {
                                Ok(_) => {}
                                Err(e) if e.kind == ErrorKind::Break => {
                                    self.env = old_env;
                                    return Ok(e.value.map(|v| *v).unwrap_or(Value::Unit));
                                }
                                Err(e) if e.kind == ErrorKind::Continue => {
                                    self.env = old_env;
                                    continue;
                                }
                                Err(e) => {
                                    self.env = old_env;
                                    return Err(e);
                                }
                            }

                            self.env = old_env;
                        }
                        Ok(Value::Unit)
                    }
                    Value::String(s) => {
                        for ch in s.chars() {
                            let old_env = self.env.clone();
                            self.env = Rc::new(RefCell::new(Environment::with_parent(old_env.clone())));

                            self.bind_pattern(pattern, Value::String(ch.to_string()))?;
                            match self.eval_block(body) {
                                Ok(_) => {}
                                Err(e) if e.kind == ErrorKind::Break => {
                                    self.env = old_env;
                                    return Ok(e.value.map(|v| *v).unwrap_or(Value::Unit));
                                }
                                Err(e) if e.kind == ErrorKind::Continue => {
                                    self.env = old_env;
                                    continue;
                                }
                                Err(e) => {
                                    self.env = old_env;
                                    return Err(e);
                                }
                            }

                            self.env = old_env;
                        }
                        Ok(Value::Unit)
                    }
                    _ => Err(RuntimeError::new(format!(
                        "cannot iterate over {}",
                        iterable.type_name()
                    ))),
                }
            }

            Expr::Return { value, .. } => {
                let val = if let Some(v) = value {
                    self.eval_expr(v)?
                } else {
                    Value::Unit
                };
                Err(RuntimeError::return_signal(val))
            }

            Expr::Break { value, .. } => {
                let val = if let Some(v) = value {
                    self.eval_expr(v)?
                } else {
                    Value::Unit
                };
                Err(RuntimeError::break_signal(val))
            }

            Expr::Continue { .. } => {
                Err(RuntimeError::continue_signal())
            }

            Expr::Assign { target, value, .. } => {
                let val = self.eval_expr(value)?;
                self.assign_to(target, val)?;
                Ok(Value::Unit)
            }

            Expr::AssignOp {
                op, target, value, ..
            } => {
                let current = self.eval_expr(target)?;
                let rhs = self.eval_expr(value)?;
                let new_val = self.eval_binary(*op, current, rhs)?;
                self.assign_to(target, new_val)?;
                Ok(Value::Unit)
            }

            Expr::Array(elements, _) => {
                let values: Vec<Value> = elements
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Array(Rc::new(RefCell::new(values))))
            }

            Expr::Tuple(elements, _) => {
                let values: Vec<Value> = elements
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                Ok(Value::Tuple(values))
            }

            Expr::Index { expr, index, .. } => {
                let array = self.eval_expr(expr)?;
                let idx = self.eval_expr(index)?;
                match (array, idx) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let arr = arr.borrow();
                        let idx = if i < 0 {
                            (arr.len() as i64 + i) as usize
                        } else {
                            i as usize
                        };
                        arr.get(idx).cloned().ok_or_else(|| {
                            RuntimeError::new(format!("index {} out of bounds", i))
                        })
                    }
                    (Value::String(s), Value::Int(i)) => {
                        let idx = if i < 0 {
                            (s.len() as i64 + i) as usize
                        } else {
                            i as usize
                        };
                        s.chars()
                            .nth(idx)
                            .map(|c| Value::String(c.to_string()))
                            .ok_or_else(|| RuntimeError::new(format!("index {} out of bounds", i)))
                    }
                    (arr, idx) => Err(RuntimeError::new(format!(
                        "cannot index {} with {}",
                        arr.type_name(),
                        idx.type_name()
                    ))),
                }
            }

            Expr::Field { expr, field, .. } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Struct { fields, .. } => {
                        fields.get(&field.name).cloned().ok_or_else(|| {
                            RuntimeError::new(format!("no field named '{}'", field.name))
                        })
                    }
                    _ => Err(RuntimeError::new(format!(
                        "cannot access field on {}",
                        val.type_name()
                    ))),
                }
            }

            Expr::MethodCall {
                receiver,
                method,
                args,
                ..
            } => {
                let recv = self.eval_expr(receiver)?;
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;

                // Check for built-in methods
                self.call_method(recv, &method.name, arg_values)
            }

            Expr::Struct { path, fields, .. } => {
                let mut field_values = HashMap::new();
                for FieldInit { name, value, .. } in fields {
                    let val = if let Some(expr) = value {
                        self.eval_expr(expr)?
                    } else {
                        // Shorthand: use variable with same name
                        self.env
                            .borrow()
                            .get(&name.name)
                            .ok_or_else(|| RuntimeError::undefined_variable(&name.name))?
                    };
                    field_values.insert(name.name.clone(), val);
                }
                Ok(Value::Struct {
                    name: path.segments.last().map(|s| s.ident.name.clone()).unwrap_or_default(),
                    fields: field_values,
                })
            }

            Expr::Closure {
                params, body, ..
            } => {
                let param_idents: Vec<Ident> = params.iter().filter_map(|p| {
                    match &p.pattern {
                        Pattern::Ident { name, .. } => Some(name.clone()),
                        _ => None,
                    }
                }).collect();
                let func = LemonFunction {
                    name: "<closure>".to_string(),
                    params: param_idents,
                    body: body.clone(),
                };
                Ok(Value::Closure {
                    func: Rc::new(func),
                    env: self.env.clone(),
                })
            }

            // OOP expressions - basic support
            Expr::This { .. } => {
                self.env
                    .borrow()
                    .get("this")
                    .ok_or_else(|| RuntimeError::new("'this' outside of method"))
            }

            Expr::New { class, args, .. } => {
                let class_name = class
                    .segments
                    .last()
                    .map(|s| s.ident.name.clone())
                    .unwrap_or_default();

                // Look for constructor
                let constructor_name = format!("{}::new", class_name);
                let ctor = self.globals.borrow().get(&constructor_name);

                if let Some(ctor) = ctor {
                    let arg_values = match args {
                        NewArgs::Args(exprs) => exprs
                            .iter()
                            .map(|e| self.eval_expr(e))
                            .collect::<Result<Vec<_>, _>>()?,
                        NewArgs::Fields(fields) => {
                            // Convert to struct and call default constructor
                            let mut field_values = HashMap::new();
                            for FieldInit { name, value, .. } in fields {
                                let val = if let Some(expr) = value {
                                    self.eval_expr(expr)?
                                } else {
                                    self.env
                                        .borrow()
                                        .get(&name.name)
                                        .ok_or_else(|| RuntimeError::undefined_variable(&name.name))?
                                };
                                field_values.insert(name.name.clone(), val);
                            }
                            return Ok(Value::Struct {
                                name: class_name,
                                fields: field_values,
                            });
                        }
                    };
                    self.call_value(&ctor, arg_values)
                } else {
                    // No constructor - create struct directly
                    match args {
                        NewArgs::Fields(fields) => {
                            let mut field_values = HashMap::new();
                            for FieldInit { name, value, .. } in fields {
                                let val = if let Some(expr) = value {
                                    self.eval_expr(expr)?
                                } else {
                                    self.env
                                        .borrow()
                                        .get(&name.name)
                                        .ok_or_else(|| RuntimeError::undefined_variable(&name.name))?
                                };
                                field_values.insert(name.name.clone(), val);
                            }
                            Ok(Value::Struct {
                                name: class_name,
                                fields: field_values,
                            })
                        }
                        NewArgs::Args(_) => Err(RuntimeError::new(format!(
                            "no constructor found for '{}'",
                            class_name
                        ))),
                    }
                }
            }

            Expr::Match { expr, arms, .. } => {
                let value = self.eval_expr(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern, &value)? {
                        // Check guard if present
                        if let Some(guard) = &arm.guard {
                            let guard_env = Rc::new(RefCell::new(Environment::with_parent(self.env.clone())));
                            let old_env = std::mem::replace(&mut self.env, guard_env);
                            for (name, val) in &bindings {
                                self.env.borrow_mut().define(name.clone(), val.clone());
                            }
                            let guard_result = self.eval_expr(guard)?;
                            self.env = old_env;
                            if !matches!(guard_result, Value::Bool(true)) {
                                continue;
                            }
                        }
                        // Execute arm body with bindings
                        let arm_env = Rc::new(RefCell::new(Environment::with_parent(self.env.clone())));
                        let old_env = std::mem::replace(&mut self.env, arm_env);
                        for (name, val) in bindings {
                            self.env.borrow_mut().define(name, val);
                        }
                        let result = self.eval_expr(&arm.body);
                        self.env = old_env;
                        return result;
                    }
                }
                Err(RuntimeError::new("no match arm matched"))
            }

            Expr::Try { expr, .. } => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Ok(inner) => Ok(*inner),
                    Value::Err(e) => Err(RuntimeError::new(format!("propagated error: {}", e))),
                    Value::Some(inner) => Ok(*inner),
                    Value::None => Err(RuntimeError::new("unwrapped None")),
                    other => Ok(other), // Pass through non-Result/Option types
                }
            }

            _ => Err(RuntimeError::new(format!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expr)
            ))),
        }
    }

    /// Match a pattern against a value, returning bindings if it matches
    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<Option<HashMap<String, Value>>, RuntimeError> {
        let mut bindings = HashMap::new();

        match pattern {
            Pattern::Wildcard(_) => Ok(Some(bindings)),

            Pattern::Ident { name, .. } => {
                bindings.insert(name.name.clone(), value.clone());
                Ok(Some(bindings))
            }

            Pattern::Literal(lit, _) => {
                let pattern_val = self.eval_literal(lit)?;
                if self.values_equal(&pattern_val, value) {
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            }

            Pattern::Tuple(patterns, _) => {
                if let Value::Tuple(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(None);
                    }
                    for (p, v) in patterns.iter().zip(values.iter()) {
                        if let Some(sub_bindings) = self.match_pattern(p, v)? {
                            bindings.extend(sub_bindings);
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(Some(bindings))
                } else {
                    Ok(None)
                }
            }

            Pattern::TupleStruct { path, fields, .. } => {
                // Handle Ok(x), Err(e), Some(x), None patterns
                let variant_name = path.segments.last()
                    .map(|s| s.ident.name.as_str())
                    .unwrap_or("");

                match (variant_name, value) {
                    ("Ok", Value::Ok(inner)) => {
                        if fields.len() == 1 {
                            if let Some(sub_bindings) = self.match_pattern(&fields[0], &*inner)? {
                                bindings.extend(sub_bindings);
                                Ok(Some(bindings))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    ("Err", Value::Err(inner)) => {
                        if fields.len() == 1 {
                            // Err contains a string
                            let err_val = Value::String(inner.to_string());
                            if let Some(sub_bindings) = self.match_pattern(&fields[0], &err_val)? {
                                bindings.extend(sub_bindings);
                                Ok(Some(bindings))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    ("Some", Value::Some(inner)) => {
                        if fields.len() == 1 {
                            if let Some(sub_bindings) = self.match_pattern(&fields[0], &*inner)? {
                                bindings.extend(sub_bindings);
                                Ok(Some(bindings))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            }

            Pattern::Path(path, _) => {
                // Handle unit variants like None
                let variant_name = path.segments.last()
                    .map(|s| s.ident.name.as_str())
                    .unwrap_or("");

                match (variant_name, value) {
                    ("None", Value::None) => Ok(Some(bindings)),
                    _ => Ok(None),
                }
            }

            Pattern::Or(patterns, _) => {
                for p in patterns {
                    if let Some(sub_bindings) = self.match_pattern(p, value)? {
                        return Ok(Some(sub_bindings));
                    }
                }
                Ok(None)
            }

            _ => Err(RuntimeError::new(format!(
                "unsupported pattern: {:?}",
                std::mem::discriminant(pattern)
            ))),
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Unit, Value::Unit) => true,
            (Value::None, Value::None) => true,
            _ => false,
        }
    }

    /// Evaluate a literal
    fn eval_literal(&self, lit: &Literal) -> Result<Value, RuntimeError> {
        Ok(match lit {
            Literal::Int(i) => Value::Int(*i),
            Literal::Float(f) => Value::Float(*f),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Char(c) => Value::String(c.to_string()),
            Literal::Bool(b) => Value::Bool(*b),
        })
    }

    /// Evaluate a path (variable lookup)
    fn eval_path(&self, path: &ExprPath) -> Result<Value, RuntimeError> {
        if path.segments.len() == 1 {
            let name = &path.segments[0].name;

            // Check builtins first
            if self.builtins.has(name) {
                return Ok(Value::Function(Rc::new(LemonFunction {
                    name: name.clone(),
                    params: vec![],
                    body: Box::new(Expr::Literal(
                        Literal::Int(0),
                        Span { start: 0, end: 0 },
                    )),
                })));
            }

            // Then local environment
            if let Some(val) = self.env.borrow().get(name) {
                return Ok(val);
            }

            // Then globals
            if let Some(val) = self.globals.borrow().get(name) {
                return Ok(val);
            }

            Err(RuntimeError::undefined_variable(name))
        } else {
            // Multi-segment path like `fs::read_string` or `math::square`
            let full_path = path
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::");

            // Check builtins with full path first
            if self.builtins.has(&full_path) {
                return Ok(Value::Function(Rc::new(LemonFunction {
                    name: full_path,
                    params: vec![],
                    body: Box::new(Expr::Literal(
                        Literal::Int(0),
                        Span { start: 0, end: 0 },
                    )),
                })));
            }

            // Try module member lookup: first segment is module name
            let module_name = &path.segments[0].name;

            // Check in local env first, then globals
            let module_value = self.env.borrow().get(module_name)
                .or_else(|| self.globals.borrow().get(module_name));

            if let Some(module) = module_value {
                if let Value::Module { exports, .. } = module {
                    // Navigate through the path to find the target
                    let mut current_exports = exports;
                    for (i, segment) in path.segments.iter().enumerate().skip(1) {
                        if i == path.segments.len() - 1 {
                            // Last segment - get the value
                            if let Some(value) = current_exports.get(&segment.name) {
                                return Ok(value.clone());
                            } else {
                                return Err(RuntimeError::new(format!(
                                    "'{}' not found in module '{}'",
                                    segment.name, module_name
                                )));
                            }
                        } else {
                            // Intermediate segment - must be another module
                            if let Some(Value::Module { exports: nested, .. }) = current_exports.get(&segment.name) {
                                current_exports = nested.clone();
                            } else {
                                return Err(RuntimeError::new(format!(
                                    "'{}' is not a module in path '{}'",
                                    segment.name, full_path
                                )));
                            }
                        }
                    }
                }
            }

            Err(RuntimeError::undefined_variable(&full_path))
        }
    }

    /// Assign a value to a target expression
    fn assign_to(&mut self, target: &Expr, value: Value) -> Result<(), RuntimeError> {
        match target {
            Expr::Path(path, _) if path.segments.len() == 1 => {
                let name = &path.segments[0].name;
                if !self.env.borrow_mut().set(name, value.clone()) {
                    self.env.borrow_mut().define(name.clone(), value);
                }
                Ok(())
            }
            Expr::Index { expr, index, .. } => {
                let array = self.eval_expr(expr)?;
                let idx = self.eval_expr(index)?.as_int()?;
                match array {
                    Value::Array(arr) => {
                        let mut arr = arr.borrow_mut();
                        let idx = if idx < 0 {
                            (arr.len() as i64 + idx) as usize
                        } else {
                            idx as usize
                        };
                        if idx < arr.len() {
                            arr[idx] = value;
                            Ok(())
                        } else {
                            Err(RuntimeError::new(format!("index {} out of bounds", idx)))
                        }
                    }
                    _ => Err(RuntimeError::new("cannot index non-array")),
                }
            }
            Expr::Field { expr, field, .. } => {
                let target_val = self.eval_expr(expr)?;
                match target_val {
                    Value::Struct { name, mut fields } => {
                        fields.insert(field.name.clone(), value);
                        // We need to update the original - this is a limitation of the current design
                        // For now, just error
                        Err(RuntimeError::new("mutable field access not yet supported"))
                    }
                    _ => Err(RuntimeError::new("cannot access field on non-struct")),
                }
            }
            _ => Err(RuntimeError::new("invalid assignment target")),
        }
    }

    /// Evaluate a binary operation
    fn eval_binary(&self, op: BinOp, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot add {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Sub => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot subtract {} from {}",
                    b.type_name(),
                    a.type_name()
                ))),
            },
            BinOp::Mul => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot multiply {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Div => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if b == 0 {
                        Err(RuntimeError::new("division by zero"))
                    } else {
                        Ok(Value::Int(a / b))
                    }
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot divide {} by {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Rem => match (left, right) {
                (Value::Int(a), Value::Int(b)) => {
                    if b == 0 {
                        Err(RuntimeError::new("modulo by zero"))
                    } else {
                        Ok(Value::Int(a % b))
                    }
                }
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot modulo {} by {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Eq => Ok(Value::Bool(left == right)),
            BinOp::Ne => Ok(Value::Bool(left != right)),
            BinOp::Lt => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Le => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Gt => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Ge => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
                (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::And => Ok(Value::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Bool(left.is_truthy() || right.is_truthy())),
            BinOp::BitAnd => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot bitwise AND {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::BitOr => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot bitwise OR {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::BitXor => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot bitwise XOR {} and {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Shl => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a << b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot shift {} by {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
            BinOp::Shr => match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a >> b)),
                (a, b) => Err(RuntimeError::new(format!(
                    "cannot shift {} by {}",
                    a.type_name(),
                    b.type_name()
                ))),
            },
        }
    }

    /// Evaluate a unary operation
    fn eval_unary(&self, op: UnaryOp, val: Value) -> Result<Value, RuntimeError> {
        match op {
            UnaryOp::Not => Ok(Value::Bool(!val.is_truthy())),
            UnaryOp::Neg => match val {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                v => Err(RuntimeError::new(format!("cannot negate {}", v.type_name()))),
            },
            _ => Err(RuntimeError::new(format!("unsupported unary operator: {:?}", op))),
        }
    }

    /// Call a value as a function
    fn call_value(&mut self, func: &Value, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match func {
            Value::Function(lemon_func) => {
                // Check if it's a builtin
                if self.builtins.has(&lemon_func.name) {
                    return self.builtins.call(&lemon_func.name, args);
                }

                // User-defined function
                self.call_function(lemon_func, args)
            }
            Value::Closure { func, env } => {
                // Create new environment with closure's captured environment as parent
                let old_env = self.env.clone();
                self.env = Rc::new(RefCell::new(Environment::with_parent(env.clone())));

                // Bind parameters
                if func.params.len() != args.len() {
                    return Err(RuntimeError::arity_mismatch(func.params.len(), args.len()));
                }
                for (param, arg) in func.params.iter().zip(args) {
                    self.env.borrow_mut().define(param.name.clone(), arg);
                }

                // Execute body
                let result = match self.eval_expr(&func.body) {
                    Ok(val) => Ok(val),
                    Err(e) if e.kind == ErrorKind::Return => {
                        Ok(e.value.map(|v| *v).unwrap_or(Value::Unit))
                    }
                    Err(e) => Err(e),
                };

                // Restore environment
                self.env = old_env;

                result
            }
            _ => Err(RuntimeError::new(format!(
                "cannot call {}",
                func.type_name()
            ))),
        }
    }

    /// Call a user-defined function
    fn call_function(
        &mut self,
        func: &LemonFunction,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // Check arity
        if func.params.len() != args.len() {
            return Err(RuntimeError::arity_mismatch(func.params.len(), args.len()));
        }

        // Create new environment
        let old_env = self.env.clone();
        self.env = Rc::new(RefCell::new(Environment::with_parent(self.globals.clone())));

        // Bind parameters
        for (param, arg) in func.params.iter().zip(args) {
            self.env.borrow_mut().define(param.name.clone(), arg);
        }

        // Execute body
        let result = match self.eval_expr(&func.body) {
            Ok(val) => Ok(val),
            Err(e) if e.kind == ErrorKind::Return => {
                Ok(e.value.map(|v| *v).unwrap_or(Value::Unit))
            }
            Err(e) => Err(e),
        };

        // Restore environment
        self.env = old_env;

        result
    }

    /// Call a method on a value
    fn call_method(
        &mut self,
        receiver: Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // Built-in methods for arrays
        match (&receiver, method) {
            (Value::Array(arr), "len") => Ok(Value::Int(arr.borrow().len() as i64)),
            (Value::Array(arr), "push") => {
                if args.len() != 1 {
                    return Err(RuntimeError::arity_mismatch(1, args.len()));
                }
                arr.borrow_mut().push(args.into_iter().next().unwrap());
                Ok(Value::Unit)
            }
            (Value::Array(arr), "pop") => {
                let val = arr.borrow_mut().pop();
                Ok(val.map(|v| Value::Some(Box::new(v))).unwrap_or(Value::None))
            }
            (Value::String(s), "len") => Ok(Value::Int(s.len() as i64)),
            (Value::String(s), "trim") => Ok(Value::String(s.trim().to_string())),
            (Value::String(s), "to_upper") => Ok(Value::String(s.to_uppercase())),
            (Value::String(s), "to_lower") => Ok(Value::String(s.to_lowercase())),
            (Value::String(s), "split") => {
                if args.len() != 1 {
                    return Err(RuntimeError::arity_mismatch(1, args.len()));
                }
                let delim = args[0].as_string()?;
                let parts: Vec<Value> = s.split(delim).map(|p| Value::String(p.to_string())).collect();
                Ok(Value::Array(Rc::new(RefCell::new(parts))))
            }
            (Value::String(s), "contains") => {
                if args.len() != 1 {
                    return Err(RuntimeError::arity_mismatch(1, args.len()));
                }
                let needle = args[0].as_string()?;
                Ok(Value::Bool(s.contains(needle)))
            }
            _ => Err(RuntimeError::new(format!(
                "no method '{}' on {}",
                method,
                receiver.type_name()
            ))),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
