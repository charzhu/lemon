//! Built-in functions for the Lemon runtime

pub mod fs;
pub mod io;
pub mod net;
pub mod ui;

use std::collections::HashMap;

use super::value::{RuntimeError, Value};

/// Registry of built-in functions
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFn>,
}

/// A built-in function definition
pub struct BuiltinFn {
    pub name: &'static str,
    pub handler: fn(&[Value]) -> Result<Value, RuntimeError>,
}

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            functions: HashMap::new(),
        };
        reg.register_io();
        reg.register_fs();
        reg.register_net();
        reg.register_ui();
        reg
    }

    /// Check if a builtin exists
    pub fn has(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Call a builtin function
    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if let Some(builtin) = self.functions.get(name) {
            (builtin.handler)(&args)
        } else {
            Err(RuntimeError::undefined_function(name))
        }
    }

    /// Register a builtin function
    fn register(&mut self, name: &'static str, handler: fn(&[Value]) -> Result<Value, RuntimeError>) {
        self.functions.insert(
            name.to_string(),
            BuiltinFn { name, handler },
        );
    }

    /// Register I/O builtins (both legacy names and underscore primitives)
    fn register_io(&mut self) {
        // Legacy names (for backward compatibility)
        self.register("print", io::builtin_print);
        self.register("println", io::builtin_println);
        self.register("eprint", io::builtin_eprint);
        self.register("eprintln", io::builtin_eprintln);
        self.register("input", io::builtin_input);
        self.register("len", io::builtin_len);
        self.register("str", io::builtin_str);
        self.register("int", io::builtin_int);
        self.register("float", io::builtin_float);
        self.register("type_of", io::builtin_type_of);
        self.register("random", io::builtin_random);

        // Underscore primitives (for Lemon stdlib)
        self.register("_print", io::builtin_print);
        self.register("_println", io::builtin_println);
        self.register("_eprint", io::builtin_eprint);
        self.register("_eprintln", io::builtin_eprintln);
        self.register("_input", io::builtin_input);
        self.register("_len", io::builtin_len);
        self.register("_to_string", io::builtin_str);
        self.register("_to_int", io::builtin_int);
        self.register("_to_float", io::builtin_float);
        self.register("_type_of", io::builtin_type_of);
        self.register("_random", io::builtin_random);
    }

    /// Register file system builtins
    fn register_fs(&mut self) {
        // Legacy names (for backward compatibility)
        self.register("fs::read_string", fs::builtin_fs_read_string);
        self.register("fs::write_string", fs::builtin_fs_write_string);
        self.register("fs::read_bytes", fs::builtin_fs_read_bytes);
        self.register("fs::write_bytes", fs::builtin_fs_write_bytes);
        self.register("fs::exists", fs::builtin_fs_exists);
        self.register("fs::is_file", fs::builtin_fs_is_file);
        self.register("fs::is_dir", fs::builtin_fs_is_dir);
        self.register("fs::remove", fs::builtin_fs_remove);
        self.register("fs::mkdir", fs::builtin_fs_mkdir);
        self.register("fs::list_dir", fs::builtin_fs_list_dir);
        self.register("fs::copy", fs::builtin_fs_copy);
        self.register("fs::rename", fs::builtin_fs_rename);
        self.register("fs::append", fs::builtin_fs_append);
        self.register("fs::cwd", fs::builtin_fs_cwd);
        self.register("fs::absolute", fs::builtin_fs_absolute);

        // Underscore primitives (for Lemon stdlib)
        self.register("_fs_read_string", fs::builtin_fs_read_string);
        self.register("_fs_write_string", fs::builtin_fs_write_string);
        self.register("_fs_read_bytes", fs::builtin_fs_read_bytes);
        self.register("_fs_write_bytes", fs::builtin_fs_write_bytes);
        self.register("_fs_exists", fs::builtin_fs_exists);
        self.register("_fs_is_file", fs::builtin_fs_is_file);
        self.register("_fs_is_dir", fs::builtin_fs_is_dir);
        self.register("_fs_remove", fs::builtin_fs_remove);
        self.register("_fs_mkdir", fs::builtin_fs_mkdir);
        self.register("_fs_list_dir", fs::builtin_fs_list_dir);
        self.register("_fs_copy", fs::builtin_fs_copy);
        self.register("_fs_rename", fs::builtin_fs_rename);
        self.register("_fs_append", fs::builtin_fs_append);
        self.register("_fs_cwd", fs::builtin_fs_cwd);
        self.register("_fs_absolute", fs::builtin_fs_absolute);
    }

    /// Register networking builtins
    fn register_net(&mut self) {
        // Legacy names (for backward compatibility)
        // TCP operations
        self.register("net::tcp_listen", net::builtin_net_tcp_listen);
        self.register("net::tcp_accept", net::builtin_net_tcp_accept);
        self.register("net::tcp_connect", net::builtin_net_tcp_connect);
        self.register("net::tcp_read", net::builtin_net_tcp_read);
        self.register("net::tcp_read_line", net::builtin_net_tcp_read_line);
        self.register("net::tcp_write", net::builtin_net_tcp_write);
        self.register("net::tcp_close", net::builtin_net_tcp_close);
        self.register("net::tcp_set_timeout", net::builtin_net_tcp_set_timeout);
        self.register("net::resolve", net::builtin_net_resolve);
        // HTTP operations
        self.register("http::get", net::builtin_http_get);
        self.register("http::post", net::builtin_http_post);
        self.register("http::serve_once", net::builtin_http_serve_once);
        self.register("http::respond", net::builtin_http_respond);

        // Underscore primitives (for Lemon stdlib)
        self.register("_net_tcp_listen", net::builtin_net_tcp_listen);
        self.register("_net_tcp_accept", net::builtin_net_tcp_accept);
        self.register("_net_tcp_connect", net::builtin_net_tcp_connect);
        self.register("_net_tcp_read", net::builtin_net_tcp_read);
        self.register("_net_tcp_read_line", net::builtin_net_tcp_read_line);
        self.register("_net_tcp_write", net::builtin_net_tcp_write);
        self.register("_net_tcp_close", net::builtin_net_tcp_close);
        self.register("_net_tcp_set_timeout", net::builtin_net_tcp_set_timeout);
        self.register("_net_resolve", net::builtin_net_resolve);
    }

    /// Register UI/graphics builtins
    fn register_ui(&mut self) {
        self.register("_ui_create", ui::builtin_ui_create);
        self.register("_ui_is_open", ui::builtin_ui_is_open);
        self.register("_ui_update", ui::builtin_ui_update);
        self.register("_ui_clear", ui::builtin_ui_clear);
        self.register("_ui_rect", ui::builtin_ui_rect);
        self.register("_ui_circle", ui::builtin_ui_circle);
        self.register("_ui_line", ui::builtin_ui_line);
        self.register("_ui_pixel", ui::builtin_ui_pixel);
        self.register("_ui_rgb", ui::builtin_ui_rgb);
        self.register("_ui_key_down", ui::builtin_ui_key_down);
        self.register("_ui_mouse_pos", ui::builtin_ui_mouse_pos);
    }
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}
