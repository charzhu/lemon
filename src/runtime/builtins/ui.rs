//! UI/Graphics builtins for Lemon

use std::cell::RefCell;
use std::rc::Rc;

use minifb::{Key, Window, WindowOptions};

use super::super::value::{RuntimeError, UiWindowState, Value};

/// Create a window: _ui_create(width, height, title) -> Result<Window, String>
pub fn builtin_ui_create(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::arity_mismatch(3, args.len()));
    }

    let width = args[0].as_int()? as usize;
    let height = args[1].as_int()? as usize;
    let title = args[2].as_string()?;

    let window = Window::new(
        title,
        width,
        height,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    );

    match window {
        Ok(mut win) => {
            // Limit frame rate to ~60fps
            win.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

            let buffer = vec![0u32; width * height];
            let state = UiWindowState {
                window: win,
                buffer,
                width,
                height,
            };

            let handle = Value::Window(Rc::new(RefCell::new(state)));
            Ok(Value::Ok(Box::new(handle)))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::String(e.to_string())))),
    }
}

/// Check if window is open: _ui_is_open(window) -> Bool
pub fn builtin_ui_is_open(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Window(window) => {
            let state = window.borrow();
            Ok(Value::Bool(state.window.is_open()))
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Update window (swap buffer): _ui_update(window)
pub fn builtin_ui_update(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            let width = state.width;
            let height = state.height;
            let buffer = state.buffer.clone();
            state
                .window
                .update_with_buffer(&buffer, width, height)
                .map_err(|e| RuntimeError::new(e.to_string()))?;
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Clear buffer with color: _ui_clear(window, color)
pub fn builtin_ui_clear(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let color = args[1].as_int()? as u32;

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            state.buffer.fill(color);
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Draw filled rectangle: _ui_rect(window, x, y, w, h, color)
pub fn builtin_ui_rect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 6 {
        return Err(RuntimeError::arity_mismatch(6, args.len()));
    }

    let x = args[1].as_int()? as i64;
    let y = args[2].as_int()? as i64;
    let w = args[3].as_int()? as i64;
    let h = args[4].as_int()? as i64;
    let color = args[5].as_int()? as u32;

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            let buf_width = state.width as i64;
            let buf_height = state.height as i64;

            for py in y.max(0)..(y + h).min(buf_height) {
                for px in x.max(0)..(x + w).min(buf_width) {
                    let idx = (py * buf_width + px) as usize;
                    if idx < state.buffer.len() {
                        state.buffer[idx] = color;
                    }
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Draw filled circle: _ui_circle(window, cx, cy, r, color)
pub fn builtin_ui_circle(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 5 {
        return Err(RuntimeError::arity_mismatch(5, args.len()));
    }

    let cx = args[1].as_int()?;
    let cy = args[2].as_int()?;
    let r = args[3].as_int()?;
    let color = args[4].as_int()? as u32;

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            let buf_width = state.width as i64;
            let buf_height = state.height as i64;

            // Simple filled circle using midpoint algorithm bounds
            for py in (cy - r).max(0)..(cy + r + 1).min(buf_height) {
                for px in (cx - r).max(0)..(cx + r + 1).min(buf_width) {
                    let dx = px - cx;
                    let dy = py - cy;
                    if dx * dx + dy * dy <= r * r {
                        let idx = (py * buf_width + px) as usize;
                        if idx < state.buffer.len() {
                            state.buffer[idx] = color;
                        }
                    }
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Draw line: _ui_line(window, x1, y1, x2, y2, color)
pub fn builtin_ui_line(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 6 {
        return Err(RuntimeError::arity_mismatch(6, args.len()));
    }

    let x1 = args[1].as_int()?;
    let y1 = args[2].as_int()?;
    let x2 = args[3].as_int()?;
    let y2 = args[4].as_int()?;
    let color = args[5].as_int()? as u32;

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            // Bresenham's line algorithm
            let mut x = x1;
            let mut y = y1;
            let dx = (x2 - x1).abs();
            let dy = -(y2 - y1).abs();
            let sx = if x1 < x2 { 1 } else { -1 };
            let sy = if y1 < y2 { 1 } else { -1 };
            let mut err = dx + dy;

            let buf_width = state.width as i64;
            let buf_height = state.height as i64;

            loop {
                if x >= 0 && x < buf_width && y >= 0 && y < buf_height {
                    let idx = (y * buf_width + x) as usize;
                    if idx < state.buffer.len() {
                        state.buffer[idx] = color;
                    }
                }

                if x == x2 && y == y2 {
                    break;
                }

                let e2 = 2 * err;
                if e2 >= dy {
                    err += dy;
                    x += sx;
                }
                if e2 <= dx {
                    err += dx;
                    y += sy;
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Set pixel: _ui_pixel(window, x, y, color)
pub fn builtin_ui_pixel(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::arity_mismatch(4, args.len()));
    }

    let x = args[1].as_int()?;
    let y = args[2].as_int()?;
    let color = args[3].as_int()? as u32;

    match &args[0] {
        Value::Window(window) => {
            let mut state = window.borrow_mut();
            let buf_width = state.width as i64;
            let buf_height = state.height as i64;

            if x >= 0 && x < buf_width && y >= 0 && y < buf_height {
                let idx = (y * buf_width + x) as usize;
                if idx < state.buffer.len() {
                    state.buffer[idx] = color;
                }
            }
            Ok(Value::Unit)
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Create RGB color: _ui_rgb(r, g, b) -> Int
pub fn builtin_ui_rgb(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(RuntimeError::arity_mismatch(3, args.len()));
    }

    let r = (args[0].as_int()? & 0xFF) as u32;
    let g = (args[1].as_int()? & 0xFF) as u32;
    let b = (args[2].as_int()? & 0xFF) as u32;

    // minifb uses 0x00RRGGBB format
    let color = (r << 16) | (g << 8) | b;
    Ok(Value::Int(color as i64))
}

/// Check if key is pressed: _ui_key_down(window, key) -> Bool
pub fn builtin_ui_key_down(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::arity_mismatch(2, args.len()));
    }

    let key_name = args[1].as_string()?;

    match &args[0] {
        Value::Window(window) => {
            let state = window.borrow();
            let key = string_to_key(key_name);
            if let Some(k) = key {
                Ok(Value::Bool(state.window.is_key_down(k)))
            } else {
                Ok(Value::Bool(false))
            }
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Get mouse position: _ui_mouse_pos(window) -> (Int, Int)
pub fn builtin_ui_mouse_pos(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::arity_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Window(window) => {
            let state = window.borrow();
            let pos = state.window.get_mouse_pos(minifb::MouseMode::Clamp);
            match pos {
                Some((x, y)) => Ok(Value::Tuple(vec![
                    Value::Int(x as i64),
                    Value::Int(y as i64),
                ])),
                None => Ok(Value::Tuple(vec![Value::Int(0), Value::Int(0)])),
            }
        }
        _ => Err(RuntimeError::type_error("Window", args[0].type_name())),
    }
}

/// Convert string key name to minifb Key
fn string_to_key(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "escape" | "esc" => Some(Key::Escape),
        "space" => Some(Key::Space),
        "enter" | "return" => Some(Key::Enter),
        "up" => Some(Key::Up),
        "down" => Some(Key::Down),
        "left" => Some(Key::Left),
        "right" => Some(Key::Right),
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),
        "0" => Some(Key::Key0),
        "1" => Some(Key::Key1),
        "2" => Some(Key::Key2),
        "3" => Some(Key::Key3),
        "4" => Some(Key::Key4),
        "5" => Some(Key::Key5),
        "6" => Some(Key::Key6),
        "7" => Some(Key::Key7),
        "8" => Some(Key::Key8),
        "9" => Some(Key::Key9),
        _ => None,
    }
}
