# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Purpose

This is a **Rust learning project** — building a compiler for the **Lox language**.
**Detailed progress history:** See `progress-backup.md` for complete phase-by-phase learning record.

## Lox Language Spec (reference for validation)

### Types
`nil`, `bool`, `number` (f64), `string`

### Expressions
- Literals: `true`, `false`, `nil`, `42`, `3.14`, `"hello"`
- Arithmetic: `+  -  *  /` (numbers only; `+` also concatenates strings)
- Comparison: `<  <=  >  >=` (numbers)
- Equality: `==  !=` (any type, no implicit coercion)
- Unary: `-expr`, `!expr`
- Grouping: `(expr)`
- Variables: `identifier`
- Assignment: `identifier = expr`
- Logical: `and`, `or` (short-circuit)
- Call: `function(args)`
- Lambda: `fun (params) { body }`

### Statements
- Expression statement: `expr;`
- Variable declaration: `var name = expr;` (initializer optional, defaults to `nil`)
- Block: `{ stmt; stmt; ... }`
- Print: `print expr;`
- If/else: `if (cond) { ... } else { ... }`
- While: `while (cond) { ... }`
- For: `for (var i = 0; i < 10; i = i + 1) { ... }` (desugars to while)
- Function: `fun name(params) { body }`
- Return: `return expr;`
- Class: `class Name { methods }` (stretch goal)

### Semantics
- Dynamic typing, no type declarations
- Lexical (static) scoping — closures capture environment
- Everything is an expression to the interpreter, but top-level forms are statements
- `nil` is the only "null" value
- No semicolons required after block `}`

### Tokens the lexer must produce
`LEFT_PAREN  RIGHT_PAREN  LEFT_BRACE  RIGHT_BRACE  COMMA  DOT  MINUS  PLUS  SEMICOLON  SLASH  STAR  BANG  BANG_EQUAL  EQUAL  EQUAL_EQUAL  GREATER  GREATER_EQUAL  LESS  LESS_EQUAL  IDENTIFIER  STRING  NUMBER  AND  CLASS  ELSE  FALSE  FUN  FOR  IF  NIL  OR  PRINT  RETURN  SUPER  THIS  TRUE  VAR  WHILE  EOF`

## Build & Run

```sh
cargo build        # compile
cargo run          # run
cargo test         # run tests
cargo test test_name  # run a single test
```

## Teaching Guidelines

### Code quality reviews

When reviewing the user's code, check beyond correctness. Look for:
- **Abstraction level**: Functions should do one thing. No premature abstraction.
- **Idiomatic Rust**: Prefer `match` over nested `if/else`, use `if let` when only one variant matters, prefer `Iterator` methods over manual loops when clearer.
- **Naming**: Types `PascalCase`, functions/variables `snake_case`. Describe *what*, not *how*.
- **No over-engineering**: Don't suggest traits, generics, or module splits until the codebase genuinely needs them.

When you spot an issue, point out the smell, explain *why*, and ask how they'd fix it.

When user asks how to do X or Y, teach what's needed and fill knowledge gaps.

---

## Current Status

**Phases 1–6: COMPLETE ✅** (128 tests passing)

### Rust Concepts Learned
- `enum`, `match`, `Option`, `Result`, `?` operator
- `String` vs `&str`, ownership, borrowing, lifetimes
- `Box<T>`, `Rc<RefCell<T>>`, interior mutability
- `struct`, `impl` blocks, `HashMap`, `Vec`
- `Iterator` trait, `From`/`Into`, `Display`, `std::error::Error`
- Custom traits (`SourceLocation`), trait bounds, `dyn Trait`, default methods

### Next Goals
- **REPL** — Fix the existing `looped()` REPL in `main.rs` (Read-Eval-Print Loop)
- **File execution** — `cargo run file.lox` to read and execute a Lox source file
- **String execution** — `cargo run -e "print 1 + 2;"` to run a Lox string directly
- **Code cleanup** — fix known issues, remove dead code, consistent error messages

### Known Issues
- `__super` not blocked as user variable name
- `.` and `/` at non-EOF without following char don't emit tokens
- `Expr::Assign` errors have `line: 0, column: 0`
