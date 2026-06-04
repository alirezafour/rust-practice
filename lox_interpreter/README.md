# Lox Interpreter

A tree-walk interpreter for the [Lox language] written in Rust.

Built as a learning project to pick up Rust, coming from a C++ background. Covers the full pipeline: scanner → parser → interpreter.

## Features

- Dynamic typing, lexical scoping, closures
- Classes with inheritance, `this`, and `super`
- First-class functions and lambdas
- Control flow: if/else, while, for (desugared to while)
- 128 tests passing

## Build & Run

```sh
cargo build          # compile
cargo run            # run (currently REPL mode)
cargo test           # run all 128 tests
cargo test test_name # run a single test
```

No external dependencies. Edition 2024.

## Example

```lox
class Shape {
    area() { return 0; }
}

class Circle < Shape {
    init(r) { this.r = r; }
    area() { return 3.14159 * this.r * this.r; }
}

var c = Circle(5);
print c.area(); // 78.53975
```

## Lox Quick Reference

```
// Variables
var x = 10;
x = 20;

// Functions
fun add(a, b) { return a + b; }
print add(3, 4); // 7

// Closures
fun makeCounter() {
    var count = 0;
    fun counter() { count = count + 1; return count; }
    return counter;
}
var counter = makeCounter();
print counter(); // 1
print counter(); // 2

// Control flow
if (x > 5) { print "big"; } else { print "small"; }
while (x > 0) { print x; x = x - 1; }
for (var i = 0; i < 10; i = i + 1) { print i; }
```

## Project Structure

```
src/
  main.rs         # entry point, REPL
  scanner.rs      # lexer (implements Iterator trait)
  parser.rs       # recursive descent parser → AST
  interpreter.rs  # tree-walk interpreter
```
