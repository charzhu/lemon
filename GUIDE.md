# Lemon Programming Guide

A comprehensive guide to programming in Lemon.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Syntax](#basic-syntax)
3. [Data Types](#data-types)
4. [Variables](#variables)
5. [Operators](#operators)
6. [Control Flow](#control-flow)
7. [Functions](#functions)
8. [Arrays and Tuples](#arrays-and-tuples)
9. [Structs](#structs)
10. [Classes and OOP](#classes-and-oop)
11. [Interfaces](#interfaces)
12. [Modules](#modules)
13. [Error Handling](#error-handling)
14. [Standard Library](#standard-library)
15. [Graphics Programming](#graphics-programming)
16. [Networking](#networking)
17. [Best Practices](#best-practices)

---

## Getting Started

### Installation

```bash
git clone https://github.com/charzhu/lemon.git
cd lemon
cargo build --release
```

### Running Programs

```bash
# Run a Lemon file
cargo run -- run myprogram.lemon

# Or use the built binary
./target/release/lemon run myprogram.lemon
```

### Your First Program

Create a file `hello.lemon`:

```lemon
fn main() {
    println("Hello, World!");
}
```

Run it:

```bash
cargo run -- run hello.lemon
```

---

## Basic Syntax

### Comments

```lemon
// Single-line comment

/*
   Multi-line
   comment
*/

/// Documentation comment
fn documented_function() {
    // ...
}
```

### Statements

Statements end with semicolons or are expressions:

```lemon
let x = 5;              // Statement with semicolon
println("Hello");       // Function call statement

// Expression as last statement (implicit return)
fn double(x: Int) -> Int {
    x * 2               // No semicolon = return value
}
```

### Blocks

Blocks are enclosed in curly braces and create new scopes:

```lemon
{
    let x = 10;
    println(x);         // x is visible here
}
// x is not visible here
```

---

## Data Types

### Primitive Types

| Type | Description | Example |
|------|-------------|---------|
| `Int` | 64-bit signed integer | `42`, `-17`, `0xFF` |
| `Float` | 64-bit floating point | `3.14`, `-0.5` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"Hello"` |
| `()` | Unit type (void) | `()` |

### Compound Types

| Type | Description | Example |
|------|-------------|---------|
| `[T]` | Array | `[1, 2, 3]` |
| `(T, U)` | Tuple | `(10, "hello")` |
| `Result<T, E>` | Success or error | `Ok(42)`, `Err("failed")` |
| `Option<T>` | Some value or none | `Some(5)`, `None` |

### Type Annotations

```lemon
let x: Int = 42;
let name: String = "Lemon";
let numbers: [Int] = [1, 2, 3];
let pair: (Int, String) = (1, "one");

fn add(a: Int, b: Int) -> Int {
    a + b
}
```

---

## Variables

### Declaration

```lemon
// Immutable by default
let x = 10;

// Mutable variable
let mut counter = 0;
counter = counter + 1;

// With type annotation
let name: String = "Lemon";
```

### Constants

```lemon
// Module-level constants
pub let PI = 3.14159;
pub let MAX_SIZE = 1000;
```

### Shadowing

```lemon
let x = 5;
let x = x + 1;      // New variable shadows old one
let x = "hello";    // Can even change type
```

---

## Operators

### Arithmetic

```lemon
let sum = a + b;        // Addition
let diff = a - b;       // Subtraction
let prod = a * b;       // Multiplication
let quot = a / b;       // Division
let rem = a % b;        // Remainder (modulo)
```

### Comparison

```lemon
a == b      // Equal
a != b      // Not equal
a < b       // Less than
a <= b      // Less than or equal
a > b       // Greater than
a >= b      // Greater than or equal
```

### Logical

```lemon
a && b      // Logical AND
a || b      // Logical OR
!a          // Logical NOT
```

### String Concatenation

```lemon
let greeting = "Hello, " + name + "!";
```

---

## Control Flow

### If-Else

```lemon
if condition {
    // do something
} else if other_condition {
    // do something else
} else {
    // fallback
}

// If as expression
let result = if x > 0 { "positive" } else { "non-positive" };
```

### While Loop

```lemon
let i = 0;
while i < 10 {
    println(i);
    i = i + 1;
}
```

### For Loop

```lemon
// Iterate over array
for item in items {
    println(item);
}

// Iterate over range (if implemented)
for i in 0..10 {
    println(i);
}
```

### Match Expression

```lemon
match value {
    0 => println("zero"),
    1 => println("one"),
    n => println("other: " + str(n)),
}

// Match with Result
match some_operation() {
    Ok(result) => {
        println("Success: " + str(result));
    },
    Err(error) => {
        println("Error: " + error);
    },
}
```

### Break and Continue

```lemon
while true {
    if should_stop {
        break;
    }
    if should_skip {
        continue;
    }
    // do work
}
```

---

## Functions

### Basic Functions

```lemon
fn greet() {
    println("Hello!");
}

fn add(a: Int, b: Int) -> Int {
    a + b
}

fn main() {
    greet();
    let sum = add(3, 4);
    println(sum);  // 7
}
```

### Return Values

```lemon
// Explicit return
fn abs(x: Int) -> Int {
    if x < 0 {
        return -x;
    }
    return x;
}

// Implicit return (last expression without semicolon)
fn square(x: Int) -> Int {
    x * x
}
```

### Multiple Parameters

```lemon
fn create_point(x: Int, y: Int, label: String) -> Point {
    Point { x, y, label }
}
```

### Closures (Lambda Functions)

```lemon
let double = |x| x * 2;
let result = double(5);  // 10

let add = |a, b| a + b;
let sum = add(3, 4);     // 7

// With explicit types
let multiply: fn(Int, Int) -> Int = |a, b| a * b;
```

---

## Arrays and Tuples

### Arrays

```lemon
// Create array
let numbers = [1, 2, 3, 4, 5];

// Access elements
let first = numbers[0];
let third = numbers[2];

// Get length
let size = len(numbers);

// Modify (if mutable)
let mut items = [1, 2, 3];
items[0] = 10;

// Array methods
items.push(4);           // Add to end
let last = items.pop();  // Remove from end
```

### Tuples

```lemon
// Create tuple
let point = (10, 20);
let person = ("Alice", 30, true);

// Destructuring
let (x, y) = point;
let (name, age, active) = person;

// Access by index (via destructuring)
let (first, second) = pair;
```

---

## Structs

### Definition

```lemon
struct Point {
    x: Int;
    y: Int;
}

struct Person {
    name: String;
    age: Int;
    email: String;
}
```

### Creating Instances

```lemon
let origin = Point { x: 0, y: 0 };
let p = Point { x: 10, y: 20 };

// Shorthand when variable names match
let x = 5;
let y = 10;
let point = Point { x, y };
```

### Accessing Fields

```lemon
let px = point.x;
let py = point.y;
println("Point: (" + str(px) + ", " + str(py) + ")");
```

---

## Classes and OOP

### Class Definition

```lemon
class Counter {
    value: Int;

    // Constructor
    pub fn new() -> Self {
        Self { value: 0 }
    }

    // Methods
    pub fn increment(mut this) {
        this.value = this.value + 1;
    }

    pub fn get(this) -> Int {
        this.value
    }

    pub fn reset(mut this) {
        this.value = 0;
    }
}
```

### Using Classes

```lemon
fn main() {
    let counter = new Counter();
    counter.increment();
    counter.increment();
    println(counter.get());  // 2
    counter.reset();
    println(counter.get());  // 0
}
```

### Static Members

```lemon
class Config {
    static instance_count: Int = 0;
    name: String;

    pub fn new(name: String) -> Self {
        Config::instance_count = Config::instance_count + 1;
        Self { name }
    }

    static fn get_count() -> Int {
        Config::instance_count
    }
}
```

### Inheritance

```lemon
abstract class Shape {
    x: Int;
    y: Int;

    pub abstract fn area(this) -> Int;

    pub fn move_to(mut this, x: Int, y: Int) {
        this.x = x;
        this.y = y;
    }
}

class Rectangle extends Shape {
    width: Int;
    height: Int;

    pub fn new(x: Int, y: Int, w: Int, h: Int) -> Self {
        Self { x, y, width: w, height: h }
    }

    pub override fn area(this) -> Int {
        this.width * this.height
    }
}
```

### Visibility

```lemon
class Example {
    pub field: Int;           // Public
    protected internal: Int;  // Protected (subclasses)
    private_field: Int;       // Private (default)

    pub fn public_method(this) { }
    fn private_method(this) { }
}
```

---

## Interfaces

### Definition

```lemon
interface Drawable {
    fn draw(this);
    fn get_bounds(this) -> (Int, Int, Int, Int);
}

interface Clickable {
    fn on_click(this, x: Int, y: Int);
}
```

### Implementation

```lemon
class Button implements Drawable, Clickable {
    x: Int;
    y: Int;
    width: Int;
    height: Int;
    label: String;

    pub fn new(x: Int, y: Int, label: String) -> Self {
        Self { x, y, width: 100, height: 30, label }
    }

    // Implement Drawable
    pub fn draw(this) {
        println("Drawing button: " + this.label);
    }

    pub fn get_bounds(this) -> (Int, Int, Int, Int) {
        (this.x, this.y, this.width, this.height)
    }

    // Implement Clickable
    pub fn on_click(this, x: Int, y: Int) {
        println("Button clicked at (" + str(x) + ", " + str(y) + ")");
    }
}
```

---

## Modules

### Module Declaration

```lemon
// In mymodule.lemon
mod mymodule {
    pub fn public_function() {
        println("I'm public!");
    }

    fn private_function() {
        println("I'm private!");
    }

    pub struct PublicStruct {
        pub value: Int;
    }
}
```

### Using Modules

```lemon
// Import entire module
use lemon::ui;
use lemon::net;

// Use module items
fn main() {
    let window = ui::create(800, 600, "My App");
    ui::clear(window, ui::BLACK);
}
```

### Standard Library Modules

```lemon
use lemon::io;      // I/O operations
use lemon::fs;      // File system
use lemon::net;     // Networking
use lemon::http;    // HTTP client/server
use lemon::ui;      // Graphics
```

---

## Error Handling

### Result Type

```lemon
fn divide(a: Int, b: Int) -> Result<Int, String> {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println("Result: " + str(result)),
        Err(error) => println("Error: " + error),
    }
}
```

### Option Type

```lemon
fn find_item(items: [String], target: String) -> Option<Int> {
    let i = 0;
    while i < len(items) {
        if items[i] == target {
            return Some(i);
        }
        i = i + 1;
    }
    None
}

fn main() {
    let items = ["apple", "banana", "cherry"];
    match find_item(items, "banana") {
        Some(index) => println("Found at index: " + str(index)),
        None => println("Not found"),
    }
}
```

### Try Operator

```lemon
fn read_and_parse(path: String) -> Result<Int, String> {
    let content = fs::read_string(path)?;  // Propagate error if failed
    let number = int(content)?;
    Ok(number)
}
```

---

## Standard Library

### I/O Functions

```lemon
// Output
println("Hello");           // Print with newline
print("No newline");        // Print without newline
eprintln("Error message");  // Print to stderr

// Input
let name = input("Enter name: ");

// Conversion
let s = str(42);            // "42"
let n = int("42");          // 42
let f = float("3.14");      // 3.14

// Utility
let length = len(array);    // Array/string length
let t = type_of(value);     // Type name as string
let r = random(1, 100);     // Random int in range
```

### File System

```lemon
use lemon::fs;

// Read file
match fs::read_string("file.txt") {
    Ok(content) => println(content),
    Err(e) => println("Error: " + e),
}

// Write file
fs::write_string("output.txt", "Hello, file!");

// Check existence
if fs::exists("myfile.txt") {
    println("File exists!");
}

// Directory operations
fs::mkdir("new_folder");
let files = fs::list_dir(".");

// File info
if fs::is_file("test.txt") { }
if fs::is_dir("folder") { }
```

### Networking

```lemon
use lemon::net;

// TCP Server
match net::tcp_listen("127.0.0.1:8080") {
    Ok(listener) => {
        match net::tcp_accept(listener) {
            Ok(client) => {
                let data = net::tcp_read(client);
                net::tcp_write(client, "Response");
                net::tcp_close(client);
            },
            Err(e) => println("Accept error: " + e),
        }
    },
    Err(e) => println("Listen error: " + e),
}

// TCP Client
match net::tcp_connect("127.0.0.1:8080") {
    Ok(conn) => {
        net::tcp_write(conn, "Hello server!");
        let response = net::tcp_read(conn);
        net::tcp_close(conn);
    },
    Err(e) => println("Connect error: " + e),
}
```

### HTTP

```lemon
use lemon::http;

// GET request
match http::get("http://example.com/api") {
    Ok(body) => println(body),
    Err(e) => println("Error: " + e),
}

// POST request
match http::post("http://example.com/api", "data") {
    Ok(response) => println(response),
    Err(e) => println("Error: " + e),
}
```

---

## Graphics Programming

### Window Creation

```lemon
use lemon::ui;

fn main() {
    match ui::create(800, 600, "My Game") {
        Ok(window) => game_loop(window),
        Err(e) => println("Failed to create window: " + e),
    }
}

fn game_loop(window: Window) {
    while ui::is_open(window) {
        // Clear screen
        ui::clear(window, ui::BLACK);

        // Draw stuff
        // ...

        // Update display
        ui::update(window);

        // Check for exit
        if ui::key_down(window, "escape") {
            break;
        }
    }
}
```

### Drawing Primitives

```lemon
// Colors (predefined)
ui::BLACK       // 0x000000
ui::WHITE       // 0xFFFFFF
ui::RED         // 0xFF0000
ui::GREEN       // 0x00FF00
ui::BLUE        // 0x0000FF
ui::YELLOW      // 0xFFFF00
ui::CYAN        // 0x00FFFF
ui::MAGENTA     // 0xFF00FF

// Custom color
let orange = ui::rgb(255, 165, 0);

// Drawing
ui::rect(window, x, y, width, height, color);   // Rectangle
ui::circle(window, cx, cy, radius, color);       // Circle
ui::line(window, x1, y1, x2, y2, color);         // Line
ui::pixel(window, x, y, color);                   // Single pixel
```

### Input Handling

```lemon
// Keyboard
if ui::key_down(window, "left") {
    player_x = player_x - 5;
}
if ui::key_down(window, "right") {
    player_x = player_x + 5;
}
if ui::key_down(window, "space") {
    shoot();
}

// Available keys:
// "escape", "space", "enter", "up", "down", "left", "right"
// "a" through "z", "0" through "9"

// Mouse
let (mx, my) = ui::mouse_pos(window);
```

### Game Loop Pattern

```lemon
use lemon::ui;

fn main() {
    match ui::create(800, 600, "Game") {
        Ok(window) => run(window),
        Err(e) => println("Error: " + e),
    }
}

fn run(window: Window) {
    // Game state
    let player_x = 400;
    let player_y = 500;
    let score = 0;

    while ui::is_open(window) {
        // 1. Handle input
        if ui::key_down(window, "left") {
            player_x = player_x - 5;
        }
        if ui::key_down(window, "right") {
            player_x = player_x + 5;
        }

        // 2. Update game state
        // (move enemies, check collisions, etc.)

        // 3. Render
        ui::clear(window, ui::BLACK);
        ui::rect(window, player_x, player_y, 50, 30, ui::GREEN);
        ui::update(window);

        // 4. Check exit
        if ui::key_down(window, "escape") {
            break;
        }
    }

    println("Final score: " + str(score));
}
```

---

## Best Practices

### Code Organization

1. **One file per module** - Keep modules focused and cohesive
2. **Use meaningful names** - Variables, functions, and types should be self-documenting
3. **Keep functions small** - Each function should do one thing well

### Error Handling

1. **Use Result for fallible operations** - Don't silently fail
2. **Handle errors at appropriate levels** - Propagate when needed, handle when you can
3. **Provide meaningful error messages**

```lemon
// Good
fn read_config(path: String) -> Result<Config, String> {
    match fs::read_string(path) {
        Ok(content) => parse_config(content),
        Err(e) => Err("Failed to read config file: " + e),
    }
}

// Bad - ignores errors
fn read_config_bad(path: String) -> Config {
    // This might crash!
    let content = fs::read_string(path);
    parse_config(content)
}
```

### Performance Tips

1. **Avoid unnecessary allocations** - Reuse arrays when possible
2. **Use appropriate data structures** - Arrays for ordered data, consider the access patterns
3. **Profile before optimizing** - Don't guess where the bottleneck is

### Graphics Programming

1. **Clear before drawing** - Always clear the screen at the start of each frame
2. **Update once per frame** - Call `ui::update()` once at the end of your render loop
3. **Use frame-based timing** - The window is limited to ~60fps by default

---

## Example: Complete Game

Here's a simplified version of Space Invaders demonstrating many concepts:

```lemon
use lemon::ui;

fn main() {
    println("Space Invaders - Arrow keys to move, Space to shoot, ESC to quit");

    match ui::create(800, 600, "Space Invaders") {
        Ok(window) => run_game(window),
        Err(e) => println("Error: " + e),
    }
}

fn run_game(window: Window) {
    let player_x = 370;
    let bullets = [];
    let enemies = [];
    let score = 0;

    // Create enemies
    let row = 0;
    while row < 3 {
        let col = 0;
        while col < 8 {
            enemies.push(100 + col * 70);  // x
            enemies.push(50 + row * 50);   // y
            enemies.push(1);                // alive
            col = col + 1;
        }
        row = row + 1;
    }

    while ui::is_open(window) {
        // Input
        if ui::key_down(window, "left") {
            player_x = player_x - 5;
            if player_x < 0 { player_x = 0; }
        }
        if ui::key_down(window, "right") {
            player_x = player_x + 5;
            if player_x > 740 { player_x = 740; }
        }
        if ui::key_down(window, "space") {
            bullets.push(player_x + 25);
            bullets.push(550);
        }
        if ui::key_down(window, "escape") {
            break;
        }

        // Update bullets
        let new_bullets = [];
        let i = 0;
        while i < len(bullets) {
            let bx = bullets[i];
            let by = bullets[i + 1] - 8;
            if by > 0 {
                new_bullets.push(bx);
                new_bullets.push(by);
            }
            i = i + 2;
        }
        bullets = new_bullets;

        // Render
        ui::clear(window, ui::BLACK);

        // Draw player
        ui::rect(window, player_x, 560, 60, 20, ui::GREEN);

        // Draw bullets
        i = 0;
        while i < len(bullets) {
            ui::rect(window, bullets[i], bullets[i + 1], 4, 10, ui::YELLOW);
            i = i + 2;
        }

        // Draw enemies
        i = 0;
        while i < len(enemies) {
            if enemies[i + 2] == 1 {
                ui::rect(window, enemies[i], enemies[i + 1], 40, 30, ui::RED);
            }
            i = i + 3;
        }

        ui::update(window);
    }

    println("Game Over! Score: " + str(score));
}
```

---

## Further Reading

- Check the `examples/` directory for more sample programs
- Read the source code in `lemon/` for standard library implementations
- Explore `src/` to understand the language implementation

Happy coding with Lemon! 🍋
