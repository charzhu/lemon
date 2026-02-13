# Lemon 🍋

A modern, expressive programming language with Rust-like syntax, built-in graphics, and a rich standard library.

## Features

- **Rust-inspired syntax** - Familiar syntax with `fn`, `let`, `match`, `struct`, `impl`
- **Object-oriented programming** - Classes, interfaces, inheritance, and delegation
- **Module system** - Organize code with `mod` and `use` statements
- **Type system** - Static typing with generics, `Result<T, E>`, and `Option<T>`
- **Built-in graphics** - 2D graphics library using minifb for games and visualizations
- **Networking** - TCP sockets and HTTP client/server support
- **File I/O** - Complete filesystem operations

## Quick Start

### Build from Source

```bash
# Clone the repository
git clone https://github.com/charzhu/lemon.git
cd lemon

# Build
cargo build --release

# Run a program
cargo run -- run examples/hello.lemon
```

### Hello World

```lemon
fn main() {
    println("Hello, Lemon!");
}
```

### Run Examples

```bash
# Simple hello world
cargo run -- run examples/hello.lemon

# Space Invaders game
cargo run -- run examples/space_invaders.lemon

# Graphics demo
cargo run -- run examples/ui_demo.lemon

# HTTP server
cargo run -- run examples/http_server_test.lemon
```

## Language Overview

### Variables and Types

```lemon
let name = "Lemon";           // String
let age = 1;                  // Int
let pi = 3.14159;             // Float
let active = true;            // Bool
let numbers = [1, 2, 3, 4];   // Array
let point = (10, 20);         // Tuple
```

### Functions

```lemon
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn greet(name: String) {
    println("Hello, " + name + "!");
}
```

### Control Flow

```lemon
// If-else
if score > 100 {
    println("High score!");
} else if score > 50 {
    println("Good job!");
} else {
    println("Keep trying!");
}

// While loop
let i = 0;
while i < 10 {
    println(i);
    i = i + 1;
}

// For loop
for item in items {
    println(item);
}

// Match expression
match result {
    Ok(value) => println("Success: " + str(value)),
    Err(e) => println("Error: " + e),
}
```

### Classes and Interfaces

```lemon
interface Drawable {
    fn draw(this);
}

class Button implements Drawable {
    label: String;
    x: Int;
    y: Int;

    pub fn new(label: String, x: Int, y: Int) -> Self {
        Self { label, x, y }
    }

    pub fn draw(this) {
        println("Drawing button: " + this.label);
    }
}
```

### Modules

```lemon
// Import standard library
use lemon::ui;
use lemon::net;
use lemon::fs;

// Use module functions
let window = ui::create(800, 600, "My App");
```

## Standard Library

### I/O (`lemon::io`)
- `println(value)` - Print with newline
- `print(value)` - Print without newline
- `input(prompt)` - Read user input
- `len(value)` - Get length of string/array
- `str(value)` - Convert to string
- `int(value)` - Convert to integer
- `random(min, max)` - Random integer in range

### File System (`lemon::fs`)
- `read_string(path)` - Read file as string
- `write_string(path, content)` - Write string to file
- `exists(path)` - Check if path exists
- `mkdir(path)` - Create directory
- `list_dir(path)` - List directory contents

### Networking (`lemon::net`)
- `tcp_listen(addr)` - Create TCP server
- `tcp_connect(addr)` - Connect to TCP server
- `tcp_read(stream)` - Read from connection
- `tcp_write(stream, data)` - Write to connection

### HTTP (`lemon::http`)
- `get(url)` - HTTP GET request
- `post(url, body)` - HTTP POST request

### Graphics (`lemon::ui`)
- `create(width, height, title)` - Create window
- `is_open(window)` - Check if window is open
- `clear(window, color)` - Clear screen
- `rect(window, x, y, w, h, color)` - Draw rectangle
- `circle(window, cx, cy, r, color)` - Draw circle
- `line(window, x1, y1, x2, y2, color)` - Draw line
- `key_down(window, key)` - Check key press
- `mouse_pos(window)` - Get mouse position

## Examples

### Space Invaders

A complete playable game demonstrating graphics, input handling, and game logic:

```bash
cargo run -- run examples/space_invaders.lemon
```

Controls: Arrow keys to move, Space to shoot, ESC to quit.

### Graphics Demo

```lemon
use lemon::ui;

fn main() {
    match ui::create(800, 600, "Demo") {
        Ok(window) => {
            while ui::is_open(window) {
                ui::clear(window, ui::BLACK);
                ui::rect(window, 100, 100, 200, 150, ui::RED);
                ui::circle(window, 400, 300, 50, ui::BLUE);
                ui::update(window);

                if ui::key_down(window, "escape") {
                    break;
                }
            }
        },
        Err(e) => println("Error: " + e),
    }
}
```

### HTTP Server

```lemon
use lemon::net;

fn main() {
    match net::tcp_listen("127.0.0.1:8080") {
        Ok(server) => {
            println("Server running on http://127.0.0.1:8080");
            match net::tcp_accept(server) {
                Ok(client) => {
                    let request = net::tcp_read(client);
                    let response = "HTTP/1.1 200 OK\r\n\r\nHello!";
                    net::tcp_write(client, response);
                    net::tcp_close(client);
                },
                Err(e) => println("Error: " + e),
            }
        },
        Err(e) => println("Failed to start: " + e),
    }
}
```

## Project Structure

```
lemon/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lexer/           # Tokenizer
│   ├── parser/          # Parser (expressions, items)
│   ├── ast/             # Abstract syntax tree
│   ├── types/           # Type checker
│   ├── runtime/         # Interpreter and builtins
│   └── codegen/         # Code generation (WIP)
├── lemon/               # Standard library
│   ├── io.lemon
│   ├── fs.lemon
│   ├── net.lemon
│   ├── http.lemon
│   └── ui.lemon
└── examples/            # Example programs
    ├── space_invaders.lemon
    ├── ui_demo.lemon
    └── ...
```

## Documentation

See [GUIDE.md](GUIDE.md) for a comprehensive programming guide.

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
