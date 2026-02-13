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
17. [Coming from Other Languages](#coming-from-other-languages)
18. [Best Practices](#best-practices)

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

### Program Structure

Every Lemon program needs a `main` function as its entry point. Top-level code outside functions is limited to constant declarations (`let`), function definitions (`fn`), struct/class/interface definitions, and module declarations.

```lemon
// Top-level: constants and functions
let MAX_SIZE = 100;

fn helper() {
    println("I'm a helper");
}

fn main() {
    // Program starts here
    helper();
    println("Max size is " + str(MAX_SIZE));
}
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

| Type | Description | Examples |
|------|-------------|---------|
| `Int` | 64-bit signed integer | `42`, `-17`, `0xFF`, `0b1010` |
| `Float` | 64-bit floating point | `3.14`, `-0.5` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"Hello"` |
| `()` | Unit type (void) | `()` |

Integer literals support decimal, hexadecimal (`0x` prefix), and binary (`0b` prefix) notation. Underscores can be used as separators for readability: `1_000_000`, `0xFF_FF`.

### Compound Types

| Type | Description | Example |
|------|-------------|---------|
| `[T]` | Array | `[1, 2, 3]` |
| `(T, U)` | Tuple | `(10, "hello")` |
| `Result<T, E>` | Success or error | `Ok(42)`, `Err("failed")` |
| `Option<T>` | Some value or none | `Some(5)`, `None` |

### Type Annotations

Type annotations are optional in most cases — the interpreter infers types at runtime. They are required for function parameters and useful for documentation:

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

### Top-Level Constants

Variables declared at the top level (outside functions) are constants. They must be initialized with literal values or simple expressions. They cannot be reassigned.

```lemon
let PI = 3.14159;
let MAX_SIZE = 1000;
let GREETING = "Hello";

// Public constants (visible to importers)
pub let VERSION = 1;
```

> **Note:** Top-level `let` declarations cannot use function calls like `ui::rgb(255, 0, 0)`. Move those into a function body.

### Function-Level Variables

Inside functions, variables are declared with `let` and can be reassigned:

```lemon
fn main() {
    let x = 10;
    x = x + 1;       // Reassignment is allowed
    println(x);       // 11

    let name = "Lemon";
    println(name);
}
```

### Shadowing

You can declare a new variable with the same name, which shadows the previous one:

```lemon
let x = 5;
let x = x + 1;      // New variable shadows old one
let x = "hello";    // Can even change type
```

---

## Operators

### Arithmetic

```lemon
let sum = a + b;        // Addition (also string concatenation)
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
a && b      // Logical AND (short-circuit)
a || b      // Logical OR (short-circuit)
!a          // Logical NOT
```

### Bitwise

```lemon
a & b       // Bitwise AND
a | b       // Bitwise OR
a ^ b       // Bitwise XOR
a << n      // Left shift
a >> n      // Right shift
```

### String Concatenation

The `+` operator concatenates strings. Use `str()` to convert other types:

```lemon
let greeting = "Hello, " + name + "!";
let message = "Score: " + str(42);
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
```

`if` can also be used as an expression:

```lemon
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

### Loop (Infinite)

`loop` creates an infinite loop. Use `break` to exit:

```lemon
let count = 0;
loop {
    count = count + 1;
    if count >= 10 {
        break;
    }
}
println("Counted to " + str(count));
```

### For Loop

Iterate over arrays:

```lemon
let items = ["apple", "banana", "cherry"];
for item in items {
    println(item);
}
```

Iterate over characters in a string:

```lemon
for ch in "hello" {
    println(ch);  // prints h, e, l, l, o on separate lines
}
```

### Match Expression

Pattern matching with `match`:

```lemon
match value {
    0 => println("zero"),
    1 => println("one"),
    n => println("other: " + str(n)),
}
```

Match with `Result` and `Option`:

```lemon
match some_operation() {
    Ok(result) => {
        println("Success: " + str(result));
    },
    Err(error) => {
        println("Error: " + error);
    },
}

match find_something() {
    Some(value) => println("Found: " + str(value)),
    None => println("Not found"),
}
```

### Break and Continue

`break` exits the enclosing loop. `continue` skips to the next iteration:

```lemon
let i = 0;
while i < 20 {
    i = i + 1;
    if i % 2 == 0 {
        continue;    // Skip even numbers
    }
    if i > 15 {
        break;       // Stop at 15
    }
    println(i);      // Prints 1, 3, 5, 7, 9, 11, 13, 15
}
```

Both work in `while`, `loop`, and `for` loops.

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

Functions return the last expression (without semicolon) implicitly, or use `return` for early exits:

```lemon
// Implicit return (last expression)
fn square(x: Int) -> Int {
    x * x
}

// Explicit return for early exit
fn abs(x: Int) -> Int {
    if x < 0 {
        return -x;
    }
    x
}

// Return from nested control flow
fn find_first_negative(numbers: [Int]) -> Option<Int> {
    for n in numbers {
        if n < 0 {
            return Some(n);
        }
    }
    None
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

Closures capture variables from their enclosing scope.

---

## Arrays and Tuples

### Arrays

Arrays are ordered, mutable collections:

```lemon
// Create
let numbers = [1, 2, 3, 4, 5];

// Access by index (0-based)
let first = numbers[0];
let third = numbers[2];

// Negative indexing (from the end)
let last = numbers[-1];       // 5
let second_last = numbers[-2]; // 4

// Length
let size = len(numbers);

// Modify elements
numbers[0] = 10;

// Add and remove
numbers.push(6);              // Add to end
let popped = numbers.pop();   // Remove from end
```

### Tuples

Tuples are fixed-size, ordered collections of potentially different types:

```lemon
// Create
let point = (10, 20);
let person = ("Alice", 30, true);

// Destructuring
let (x, y) = point;
let (name, age, active) = person;
println(name + " is " + str(age));
```

Tuples are commonly used for returning multiple values from functions:

```lemon
fn min_max(arr: [Int]) -> (Int, Int) {
    let lo = arr[0];
    let hi = arr[0];
    for v in arr {
        if v < lo { lo = v; }
        if v > hi { hi = v; }
    }
    (lo, hi)
}

fn main() {
    let (lo, hi) = min_max([3, 1, 4, 1, 5, 9]);
    println("Min: " + str(lo) + ", Max: " + str(hi));
}
```

---

## Structs

### Definition and Usage

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

// Shorthand when variable names match field names
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

Classes combine data and behavior. Methods receive `this` (immutable) or `mut this` (mutable) as the first parameter:

```lemon
class Counter {
    value: Int;

    // Constructor — returns Self
    pub fn new() -> Self {
        Self { value: 0 }
    }

    // Mutable method — can modify fields
    pub fn increment(mut this) {
        this.value = this.value + 1;
    }

    // Immutable method — read-only access
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

Static fields and methods belong to the class itself, not instances:

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

Use `extends` for class inheritance. Abstract classes define methods that subclasses must implement:

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

class Circle extends Shape {
    radius: Int;

    pub fn new(x: Int, y: Int, r: Int) -> Self {
        Self { x, y, radius: r }
    }

    pub override fn area(this) -> Int {
        // Approximation using integer math
        3 * this.radius * this.radius
    }
}
```

### Visibility

Fields and methods have three visibility levels:

```lemon
class Example {
    pub field: Int;           // Public — accessible everywhere
    protected internal: Int;  // Protected — accessible in subclasses
    private_field: Int;       // Private — default, class only

    pub fn public_method(this) { }
    fn private_method(this) { }
}
```

---

## Interfaces

### Definition

Interfaces define a contract that classes must fulfill:

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

A class can implement multiple interfaces:

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

    pub fn draw(this) {
        println("Drawing button: " + this.label);
    }

    pub fn get_bounds(this) -> (Int, Int, Int, Int) {
        (this.x, this.y, this.width, this.height)
    }

    pub fn on_click(this, x: Int, y: Int) {
        println("Button '" + this.label + "' clicked");
    }
}
```

### Combining Inheritance and Interfaces

```lemon
class ImageButton extends Button implements Drawable {
    image_path: String;

    pub fn new(x: Int, y: Int, label: String, image: String) -> Self {
        Self { x, y, width: 100, height: 30, label, image_path: image }
    }

    pub override fn draw(this) {
        println("Drawing image button: " + this.image_path);
    }
}
```

---

## Modules

### Module Declaration

Modules group related code together:

```lemon
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
use lemon::ui;
use lemon::net;

fn main() {
    let window = ui::create(800, 600, "My App");
    ui::clear(window, ui::BLACK);
}
```

### Standard Library Modules

```lemon
use lemon::io;      // I/O operations
use lemon::fs;      // File system
use lemon::net;     // TCP networking
use lemon::http;    // HTTP client/server
use lemon::ui;      // 2D graphics
```

---

## Error Handling

Lemon uses `Result<T, E>` and `Option<T>` types for error handling. There are no exceptions — errors are values that must be explicitly handled.

### Result Type

`Result<T, E>` represents either success (`Ok(value)`) or failure (`Err(error)`):

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

`Option<T>` represents a value that may or may not exist:

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

### Try Operator (`?`)

The `?` operator propagates errors concisely. If the value is `Err`, the function returns that error immediately. If `Ok`, it unwraps the value:

```lemon
fn read_and_parse(path: String) -> Result<Int, String> {
    let content = fs::read_string(path)?;  // Returns Err early if file read fails
    let number = int(content)?;            // Returns Err early if parse fails
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
eprint("Error no newline"); // Print to stderr without newline

// Input
let name = input("Enter name: ");

// Conversion
let s = str(42);            // "42"
let n = int("42");          // 42
let f = float("3.14");      // 3.14

// Utility
let length = len(array);    // Array/string length
let t = type_of(value);     // Type name as string
let r = random(1, 100);     // Random int in [1, 100]
```

### File System (`lemon::fs`)

```lemon
use lemon::fs;

// Read and write files
match fs::read_string("config.txt") {
    Ok(content) => println(content),
    Err(e) => println("Error: " + e),
}
fs::write_string("output.txt", "Hello, file!");
fs::append("log.txt", "New log entry\n");

// Check paths
if fs::exists("myfile.txt") {
    println("File exists!");
}
if fs::is_file("test.txt") { }
if fs::is_dir("folder") { }

// Directory operations
fs::mkdir("new_folder");
let files = fs::list_dir(".");

// File operations
fs::copy("src.txt", "dst.txt");
fs::rename("old.txt", "new.txt");
fs::remove("temp.txt");

// Path utilities
let cwd = fs::cwd();
let abs = fs::absolute("relative/path");
```

### Networking (`lemon::net`)

```lemon
use lemon::net;

// TCP Server
match net::tcp_listen("127.0.0.1:8080") {
    Ok(listener) => {
        println("Listening on port 8080");
        match net::tcp_accept(listener) {
            Ok(client) => {
                net::tcp_set_timeout(client, 5000);  // 5 second timeout
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
        println(response);
        net::tcp_close(conn);
    },
    Err(e) => println("Connect error: " + e),
}

// DNS resolution
match net::resolve("example.com") {
    Ok(ip) => println("IP: " + ip),
    Err(e) => println("DNS error: " + e),
}
```

### HTTP (`lemon::http`)

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

Lemon includes a built-in 2D graphics library for creating games and visualizations.

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
        ui::clear(window, ui::BLACK);
        // ... draw things ...
        ui::update(window);

        if ui::key_down(window, "escape") {
            break;
        }
    }
}
```

### Drawing Primitives

```lemon
// Predefined colors
ui::BLACK       // 0x000000
ui::WHITE       // 0xFFFFFF
ui::RED         // 0xFF0000
ui::GREEN       // 0x00FF00
ui::BLUE        // 0x0000FF
ui::YELLOW      // 0xFFFF00
ui::CYAN        // 0x00FFFF
ui::MAGENTA     // 0xFF00FF

// Custom color from RGB (0-255 each)
let orange = ui::rgb(255, 165, 0);
let dark_gray = ui::rgb(60, 60, 60);

// Drawing functions
ui::rect(window, x, y, width, height, color);   // Filled rectangle
ui::circle(window, cx, cy, radius, color);       // Filled circle
ui::line(window, x1, y1, x2, y2, color);         // Line
ui::pixel(window, x, y, color);                   // Single pixel
```

### Input Handling

```lemon
// Keyboard — returns true while key is held down
if ui::key_down(window, "left")   { player_x = player_x - 5; }
if ui::key_down(window, "right")  { player_x = player_x + 5; }
if ui::key_down(window, "up")     { player_y = player_y - 5; }
if ui::key_down(window, "down")   { player_y = player_y + 5; }
if ui::key_down(window, "space")  { shoot(); }
if ui::key_down(window, "escape") { break; }

// Available keys:
// "escape", "space", "enter", "tab"
// "up", "down", "left", "right"
// "a" through "z"
// "0" through "9"

// Mouse position
let (mx, my) = ui::mouse_pos(window);
```

### Game Loop Pattern

The standard pattern for a Lemon game:

```lemon
use lemon::ui;

let SCREEN_W = 800;
let SCREEN_H = 600;

fn main() {
    match ui::create(SCREEN_W, SCREEN_H, "My Game") {
        Ok(window) => run(window),
        Err(e) => println("Error: " + e),
    }
}

fn run(window: Window) {
    // All mutable game state lives here
    let player_x = SCREEN_W / 2;
    let player_y = SCREEN_H - 60;
    let score = 0;

    while ui::is_open(window) {
        // 1. Handle input
        if ui::key_down(window, "left")  { player_x = player_x - 5; }
        if ui::key_down(window, "right") { player_x = player_x + 5; }
        if ui::key_down(window, "escape") { break; }

        // 2. Update game state
        score = score + 1;

        // 3. Render
        ui::clear(window, ui::BLACK);
        ui::rect(window, player_x, player_y, 50, 30, ui::GREEN);
        ui::update(window);
    }

    println("Final score: " + str(score));
}
```

> **Important:** All mutable state must be declared inside a function. Top-level `let` creates constants. Colors from `ui::rgb()` must also be created inside functions since they are function calls.

### Collision Detection

A common pattern for AABB (axis-aligned bounding box) collision:

```lemon
fn collides(ax: Int, ay: Int, aw: Int, ah: Int,
            bx: Int, by: Int, bw: Int, bh: Int) -> Bool {
    if ax + aw <= bx { return false; }
    if ax >= bx + bw { return false; }
    if ay + ah <= by { return false; }
    if ay >= by + bh { return false; }
    return true;
}

// Usage
if collides(player_x, player_y, 40, 60, enemy_x, enemy_y, 40, 60) {
    game_over = true;
}
```

### Complete Example: Car Racing

See `examples/car_racing.lemon` for a full game with:
- 4-lane scrolling road with animated lane dividers
- 4 obstacle cars with collision detection
- Fuel pickup system with a HUD fuel bar
- Speed control and score tracking
- Game over and restart with Space

```bash
cargo run -- run examples/car_racing.lemon
```

See `examples/space_invaders.lemon` for another complete game with bullets, enemies, bombs, lives, and wave progression.

---

## Networking

### TCP Echo Server

A minimal server that echoes back whatever clients send:

```lemon
use lemon::net;

fn main() {
    match net::tcp_listen("127.0.0.1:9000") {
        Ok(listener) => {
            println("Echo server on port 9000");
            loop {
                match net::tcp_accept(listener) {
                    Ok(client) => {
                        let data = net::tcp_read(client);
                        net::tcp_write(client, "Echo: " + data);
                        net::tcp_close(client);
                    },
                    Err(e) => {
                        println("Error: " + e);
                        break;
                    }
                }
            }
        },
        Err(e) => println("Failed to listen: " + e),
    }
}
```

### Simple HTTP Server

```lemon
use lemon::net;

fn main() {
    match net::tcp_listen("127.0.0.1:8080") {
        Ok(server) => {
            println("HTTP server on http://127.0.0.1:8080");
            match net::tcp_accept(server) {
                Ok(client) => {
                    let request = net::tcp_read(client);
                    let body = "<html><body><h1>Hello from Lemon!</h1></body></html>";
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n" + body;
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

---

## Coming from Other Languages

### From Rust

Lemon borrows much of Rust's syntax but drops the complexity:

| Rust | Lemon | Notes |
|------|-------|-------|
| `let mut x = 5;` | `let x = 5;` | Variables inside functions are always reassignable |
| `fn foo(x: &str)` | `fn foo(x: String)` | No references or lifetimes |
| `impl Trait for Type` | `class Type implements Trait` | Classes instead of impl blocks |
| `struct` + `impl` | `class` | Data and methods together |
| `match` with exhaustive check | `match` | Same syntax, familiar semantics |
| `Result<T, E>`, `Option<T>` | Same | Identical error handling model |
| `vec![1,2,3]` | `[1, 2, 3]` | Array literals directly |
| `cargo build` | `cargo run -- run file.lemon` | Lemon is built with Cargo |

### From Go

| Go | Lemon | Notes |
|----|-------|-------|
| `x := 5` | `let x = 5;` | `let` keyword required |
| `func foo()` | `fn foo()` | `fn` keyword |
| `if err != nil` | `match result { Err(e) => ... }` | Result types instead of error returns |
| Interfaces (implicit) | `implements` keyword | Explicit interface implementation |
| No classes | `class` with inheritance | Full OOP support |
| No generics (until 1.18) | Generics with `<T>` | Generic types supported |
| Goroutines | Not yet | Concurrency is planned |
| `fmt.Println()` | `println()` | Built-in, no import needed |

### From Java

| Java | Lemon | Notes |
|------|-------|-------|
| `int x = 5;` | `let x = 5;` | Type inference, no type keyword needed |
| `try/catch` | `match` on `Result` | No exceptions |
| `null` | `None` via `Option<T>` | Null-safe by design |
| `class Foo extends Bar` | Same syntax | Familiar inheritance |
| `interface` | Same keyword | Familiar interfaces |
| `public/private` | `pub` / default private | Simpler visibility |
| `System.out.println()` | `println()` | Built-in |
| `new Foo()` | `new Foo()` | Same constructor syntax |
| Verbose lambdas | `\|x\| x * 2` | Concise closures |

### From Python

| Python | Lemon | Notes |
|--------|-------|-------|
| Dynamic typing | Static typing | Types checked, but inferred |
| `def foo():` | `fn foo() {` | Curly braces, not indentation |
| `try/except` | `match` on `Result` | No exceptions |
| `None` | `None` via `Option<T>` | Explicit optional values |
| `class Foo:` | `class Foo {` | Similar but with typed fields |
| `self` | `this` | Explicit receiver |
| `import os` | `use lemon::fs;` | Module imports |
| `lambda x: x*2` | `\|x\| x * 2` | More capable closures |
| `list.append(x)` | `array.push(x)` | Different method name |
| tkinter (separate) | `ui::*` built-in | Graphics included |

---

## Best Practices

### Code Organization

1. **Constants at the top** — Declare `let` constants at module level, mutable state inside functions
2. **Small functions** — Break logic into focused helper functions
3. **Separate concerns** — Use the `main` → `run_game(window)` pattern for graphics programs

```lemon
// Good: constants at top, logic in functions
let SPEED = 5;
let MAX_ENEMIES = 10;

fn main() {
    match ui::create(800, 600, "Game") {
        Ok(window) => run(window),
        Err(e) => println("Error: " + e),
    }
}

fn run(window: Window) {
    let player_x = 400;
    // ...
}
```

### Error Handling

1. **Use `Result` for operations that can fail** — file I/O, networking, parsing
2. **Use `?` to propagate errors** up the call stack
3. **Handle errors at the right level** — propagate when the caller should decide, handle when you can recover
4. **Provide context** in error messages

```lemon
// Good: descriptive error handling
fn load_config(path: String) -> Result<String, String> {
    match fs::read_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Err("Failed to load config '" + path + "': " + e),
    }
}
```

### Arrays as Data Stores

Since Lemon doesn't have hashmaps, flat arrays with stride access are the standard pattern for structured collections:

```lemon
// Store enemies as [x, y, alive, x, y, alive, ...]
let enemies = [];
let col = 0;
while col < 8 {
    enemies.push(100 + col * 60);  // x
    enemies.push(50);               // y
    enemies.push(1);                // alive (1=yes, 0=no)
    col = col + 1;
}

// Access enemy i
let i = 0;
while i < len(enemies) {
    let ex = enemies[i];
    let ey = enemies[i + 1];
    let alive = enemies[i + 2];
    if alive == 1 {
        ui::rect(window, ex, ey, 40, 30, ui::RED);
    }
    i = i + 3;  // stride of 3
}
```

### Graphics Programming

1. **Clear before drawing** — always call `ui::clear()` at the start of each frame
2. **Update once per frame** — call `ui::update()` once at the end of the render loop
3. **Create colors once** — store `ui::rgb()` results in variables rather than calling every frame
4. **Check keys after drawing** — check `ui::key_down()` for exit after `ui::update()` or at the start of the loop

```lemon
fn run(window: Window) {
    // Create colors once, not every frame
    let bg_color = ui::rgb(34, 139, 34);
    let player_color = ui::rgb(0, 120, 255);

    let player_x = 400;

    while ui::is_open(window) {
        if ui::key_down(window, "escape") { break; }
        if ui::key_down(window, "left")  { player_x = player_x - 5; }
        if ui::key_down(window, "right") { player_x = player_x + 5; }

        ui::clear(window, bg_color);
        ui::rect(window, player_x, 500, 40, 60, player_color);
        ui::update(window);
    }
}
```

---

## Further Reading

- Browse `examples/` for sample programs covering basics, OOP, networking, and games
- Read the source in `lemon/` for standard library implementations
- See [README.md](README.md) for the language comparison and rationale
