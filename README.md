# Lemon

A modern programming language that combines Rust's safety-oriented syntax with the approachability of Python and Go. Built in Rust, Lemon is designed for developers who want expressive, readable code without the steep learning curve of systems languages or the runtime overhead of dynamic ones.

## Why Lemon?

Most programming languages force a choice: either you get safety and performance (Rust, C++) but pay with complexity, or you get simplicity (Python, JavaScript) but sacrifice type safety and speed. Languages in the middle like Go and Java each come with their own trade-offs — Go lacks generics expressiveness and OOP, Java carries decades of boilerplate.

Lemon aims for a different point in the design space:

- **Readable like Python, structured like Rust.** Familiar keywords (`fn`, `let`, `match`, `struct`) without lifetimes, borrowing rules, or trait bounds. Code reads naturally from top to bottom.
- **Object-oriented when you need it.** Classes, interfaces, inheritance, and abstract methods are first-class features — not bolted on (Go) or buried under ceremony (Rust traits + impl blocks).
- **Errors as values, not exceptions.** `Result<T, E>` and `Option<T>` with pattern matching, like Rust. No hidden control flow from exceptions (Java, Python) or ignored error returns (Go).
- **Batteries included.** Built-in graphics, networking, HTTP, and file I/O libraries. Write a game, a web server, or a CLI tool without pulling in external dependencies.
- **Fast feedback loop.** Interpreted execution for development, with JIT/AOT compilation support for production. No waiting for long compile times during iteration.

### How does Lemon compare?

| | Lemon | Rust | Go | Java | Python |
|---|---|---|---|---|---|
| Learning curve | Low | High | Low | Medium | Low |
| Type safety | Static | Static | Static | Static | Dynamic |
| Error handling | `Result`/`Option` | `Result`/`Option` | Multiple returns | Exceptions | Exceptions |
| OOP support | Classes, interfaces, inheritance | Traits only (no inheritance) | Interfaces only (no classes) | Full OOP | Full OOP |
| Pattern matching | `match` expressions | `match` expressions | None | Switch (limited) | `match` (3.10+) |
| Generics | Yes | Yes (complex) | Yes (basic) | Yes (type erasure) | No (typing hints) |
| Built-in graphics | Yes | No | No | No (Swing/JavaFX separate) | No (tkinter separate) |
| Memory management | GC / reference counted | Ownership system | GC | GC | GC |
| Null safety | `Option<T>` | `Option<T>` | Nil (unsafe) | Null (unsafe) | None (unsafe) |
| Closures | Yes | Yes | Yes | Yes (verbose) | Yes (limited lambda) |
| Module system | `mod` / `use` | `mod` / `use` | Packages | Packages | Modules |

### When to use Lemon

- **Learning programming** — clear syntax, immediate visual feedback with built-in graphics
- **Prototyping** — express ideas quickly without boilerplate
- **Games and visualizations** — built-in 2D graphics with keyboard and mouse input
- **Small to medium tools** — CLI utilities, file processing, network services
- **Teaching language design** — the implementation itself is a readable Rust codebase

### When not to use Lemon

- Systems programming requiring manual memory control (use Rust or C++)
- Large-scale production services with strict performance requirements (use Go or Java)
- Data science and ML ecosystems (use Python)

## Quick Start

### Build from Source

```bash
git clone https://github.com/charzhu/lemon.git
cd lemon
cargo build --release
```

### Hello World

```lemon
fn main() {
    println("Hello, Lemon!");
}
```

```bash
cargo run -- run examples/hello.lemon
```

### Run Examples

```bash
# Games
cargo run -- run examples/space_invaders.lemon
cargo run -- run examples/car_racing.lemon

# Graphics demo
cargo run -- run examples/ui_demo.lemon

# Algorithms
cargo run -- run examples/fib.lemon

# Networking
cargo run -- run examples/http_server.lemon

# OOP
cargo run -- run examples/oop.lemon
```

## Language Overview

### Variables and Types

```lemon
let name = "Lemon";           // String (immutable)
let age = 1;                  // Int (64-bit signed)
let pi = 3.14159;             // Float (64-bit)
let active = true;            // Bool
let numbers = [1, 2, 3, 4];   // Array
let point = (10, 20);         // Tuple
```

Inside functions, variables can be reassigned:

```lemon
fn main() {
    let count = 0;
    count = count + 1;  // Reassignment is allowed inside functions
}
```

### Functions

```lemon
fn add(a: Int, b: Int) -> Int {
    a + b  // Last expression is the return value
}

fn greet(name: String) {
    println("Hello, " + name + "!");
}

// Closures
let double = |x| x * 2;
println(double(5));  // 10
```

### Control Flow

```lemon
// If-else
if score > 100 {
    println("High score!");
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

// Pattern matching
match result {
    Ok(value) => println("Success: " + str(value)),
    Err(e) => println("Error: " + e),
}
```

### Error Handling

Lemon uses `Result<T, E>` and `Option<T>` types — no exceptions, no null.

```lemon
fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 0) {
        Ok(result) => println("Result: " + str(result)),
        Err(error) => println("Error: " + error),
    }
}
```

The `?` operator propagates errors concisely:

```lemon
fn read_and_parse(path: String) -> Result<Int, String> {
    let content = fs::read_string(path)?;
    let number = int(content)?;
    Ok(number)
}
```

### Classes and Interfaces

```lemon
interface Drawable {
    fn draw(this);
}

abstract class Shape {
    x: Int;
    y: Int;

    pub abstract fn area(this) -> Int;

    pub fn move_to(mut this, x: Int, y: Int) {
        this.x = x;
        this.y = y;
    }
}

class Rectangle extends Shape implements Drawable {
    width: Int;
    height: Int;

    pub fn new(x: Int, y: Int, w: Int, h: Int) -> Self {
        Self { x, y, width: w, height: h }
    }

    pub override fn area(this) -> Int {
        this.width * this.height
    }

    pub fn draw(this) {
        println("Rect at (" + str(this.x) + ", " + str(this.y) + ")");
    }
}
```

### Modules

```lemon
use lemon::ui;    // Graphics
use lemon::fs;    // File system
use lemon::net;   // TCP networking
use lemon::http;  // HTTP client/server
```

## Standard Library

### I/O

| Function | Description |
|---|---|
| `println(value)` | Print with newline |
| `print(value)` | Print without newline |
| `input(prompt)` | Read user input |
| `len(value)` | Length of string, array, or tuple |
| `str(value)` | Convert any value to string |
| `int(value)` | Convert to integer |
| `float(value)` | Convert to float |
| `type_of(value)` | Type name as string |
| `random(min, max)` | Random integer in range [min, max] |

### File System (`lemon::fs`)

| Function | Description |
|---|---|
| `fs::read_string(path)` | Read file contents as string |
| `fs::write_string(path, content)` | Write string to file |
| `fs::exists(path)` | Check if path exists |
| `fs::is_file(path)` / `fs::is_dir(path)` | Check path type |
| `fs::mkdir(path)` | Create directory |
| `fs::list_dir(path)` | List directory contents |
| `fs::remove(path)` | Delete file or directory |
| `fs::copy(src, dst)` | Copy a file |
| `fs::rename(from, to)` | Rename/move a file |
| `fs::append(path, content)` | Append to file |

### Networking (`lemon::net`)

| Function | Description |
|---|---|
| `net::tcp_listen(addr)` | Create TCP server |
| `net::tcp_accept(listener)` | Accept incoming connection |
| `net::tcp_connect(addr)` | Connect to TCP server |
| `net::tcp_read(stream)` | Read from connection |
| `net::tcp_write(stream, data)` | Write to connection |
| `net::tcp_close(stream)` | Close connection |
| `net::tcp_set_timeout(stream, ms)` | Set read timeout |

### HTTP (`lemon::http`)

| Function | Description |
|---|---|
| `http::get(url)` | HTTP GET request |
| `http::post(url, body)` | HTTP POST request |

### Graphics (`lemon::ui`)

| Function | Description |
|---|---|
| `ui::create(w, h, title)` | Create a window |
| `ui::is_open(window)` | Check if window is still open |
| `ui::clear(window, color)` | Clear screen with color |
| `ui::update(window)` | Flush frame to display |
| `ui::rect(window, x, y, w, h, color)` | Draw filled rectangle |
| `ui::circle(window, cx, cy, r, color)` | Draw filled circle |
| `ui::line(window, x1, y1, x2, y2, color)` | Draw line |
| `ui::pixel(window, x, y, color)` | Set single pixel |
| `ui::key_down(window, key)` | Check if key is pressed |
| `ui::mouse_pos(window)` | Get mouse position as `(x, y)` |
| `ui::rgb(r, g, b)` | Create color from RGB components |

Predefined colors: `ui::BLACK`, `ui::WHITE`, `ui::RED`, `ui::GREEN`, `ui::BLUE`, `ui::YELLOW`, `ui::CYAN`, `ui::MAGENTA`

## Example: Car Racing Game

A complete game demonstrating graphics, input, collision detection, and game state management:

```lemon
use lemon::ui;

let SCREEN_W = 640;
let SCREEN_H = 480;
let ROAD_LEFT = 160;
let ROAD_RIGHT = 480;
let CAR_W = 40;
let CAR_H = 60;

fn main() {
    match ui::create(SCREEN_W, SCREEN_H, "Car Racing") {
        Ok(window) => run_game(window),
        Err(e) => println("Error: " + e),
    }
}

fn run_game(window: Window) {
    let player_x = 300;
    let speed = 3;
    let score = 0;
    let obstacle_y = -100;
    let obstacle_x = 250;

    while ui::is_open(window) {
        // Input
        if ui::key_down(window, "left")  { player_x = player_x - 4; }
        if ui::key_down(window, "right") { player_x = player_x + 4; }
        if ui::key_down(window, "escape") { break; }

        // Update
        obstacle_y = obstacle_y + speed;
        score = score + speed;
        if obstacle_y > SCREEN_H {
            obstacle_y = 0 - CAR_H;
            obstacle_x = ROAD_LEFT + random(0, 240);
        }

        // Draw
        ui::clear(window, ui::rgb(34, 139, 34));
        ui::rect(window, ROAD_LEFT, 0, 320, SCREEN_H, ui::rgb(60, 60, 60));
        ui::rect(window, obstacle_x, obstacle_y, CAR_W, CAR_H, ui::RED);
        ui::rect(window, player_x, 380, CAR_W, CAR_H, ui::rgb(0, 120, 255));
        ui::update(window);
    }

    println("Score: " + str(score));
}
```

Run the full version: `cargo run -- run examples/car_racing.lemon`

## Project Structure

```
lemon/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lexer/           # Tokenizer (logos)
│   ├── parser/          # Parser (chumsky)
│   ├── ast/             # Abstract syntax tree
│   ├── types/           # Type checker
│   ├── runtime/         # Tree-walking interpreter
│   └── codegen/         # Native code generation (cranelift, WIP)
├── lemon/               # Standard library modules
│   ├── io.lemon
│   ├── fs.lemon
│   ├── net.lemon
│   ├── http.lemon
│   └── ui.lemon
└── examples/            # Example programs
    ├── hello.lemon
    ├── fib.lemon
    ├── space_invaders.lemon
    ├── car_racing.lemon
    ├── oop.lemon
    └── ...
```

## Documentation

See [GUIDE.md](GUIDE.md) for the full language reference covering data types, control flow, functions, classes, modules, error handling, graphics programming, and best practices.

## License

MIT License

## Contributing

Contributions are welcome. Please open issues and pull requests on GitHub.
