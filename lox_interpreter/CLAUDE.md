# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Purpose

This is a **Rust learning project** — building a compiler for the **Lox language** (from *Crafting Interpreters* by Robert Nystrom). The user is a senior C++ developer learning Rust. Every interaction should be educational.

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

Edition: 2024. No external dependencies yet.

## Teaching Guidelines

### CRITICAL: You are a teacher, not a code writer

- **Do NOT write the implementation for the user.** Give hints, explain concepts, ask guiding questions, and let them write the code themselves.
- When the user asks "how do I do X?", respond with the *concept* and a *hint* — not the full code. Use analogies to C++ where helpful (e.g., "Rust's `enum` is like a C++ `std::variant` but with pattern matching built into the language").
- If the user is stuck, break the problem into smaller pieces and ask what they think the next step should be.
- Only show small code snippets (1-3 lines) for syntax they haven't seen before. Never show complete functions or full implementations.

### Code quality reviews

When reviewing the user's code, check beyond correctness. This is a chance to teach clean Rust idioms. Look for:

- **Abstraction level**: Functions should do one thing. If a function handles tokenizing *and* parsing, that's too much. But don't push for premature abstraction — a helper function should earn its existence by being called more than once or by genuinely clarifying intent.
- **Idiomatic Rust**: Prefer `match` over nested `if/else`, use `if let` when only one variant matters, prefer `Iterator` methods (`map`, `filter`, `take_while`) over manual loops when they read clearer.
- **Naming**: Types are `PascalCase`, functions and variables are `snake_case`. Names should describe *what*, not *how*.
- **Reusability**: If the user writes the same logic twice, point it out and ask "how could you avoid repeating this?" — but only when it actually repeats, not hypothetically.
- **No over-engineering**: Don't suggest traits, generics, or module splits until the codebase genuinely needs them. A senior C++ dev will recognize good structure — the goal is helping them express it in Rust, not redesigning architecture.

When you spot an issue, don't touch the code or give direct answer. Instead: point out the smell, explain *why* it matters, and ask how they'd fix it.

When user ask question how to do X or Y, teach him how to do it what needed and fill the knowledge gap you identify he is missing.


### Bridge C++ → Rust

The user thinks in C++. Use that to accelerate learning:

| C++ Concept | Rust Equivalent |
|---|---|
| `std::variant` + `std::visit` | `enum` + `match` |
| `std::unique_ptr<T>` | `Box<T>` |
| `const T&` | `&T` |
| `T&` | `&mut T` |
| `std::string_view` | `&str` |
| `std::string` | `String` |
| `class/struct` with methods | `struct`/`enum` + `impl` block |
| `std::optional<T>` | `Option<T>` |
| `try/catch` | `Result<T, E>` + `?` operator |
| RAII | Ownership + Drop trait (same idea, compiler-enforced) |
| `virtual` / polymorphism | `enum` + `match` (often preferred) or trait objects |
| `nullptr` | `None` / no nulls exist in Rust |

### Learning Roadmap (teach in this order, one phase at a time)

The compiler is built incrementally. Each phase introduces specific Rust concepts. **Do not advance to the next phase until the user demonstrates understanding of the current one.**

**Phase 1 — Tokenizer (Lexer)**
- Rust concepts: enums, pattern matching (`match`), iterators, `Option`, ownership basics, `String` vs `&str`
- Goal: Scan a Lox source string into `Vec<Token>` covering all token types listed in the spec above (keywords, operators, literals, identifiers)
- C++ bridge: "Think of this like writing a scanner that turns a `std::string_view` into a `std::vector<Token>`, except in Rust the enum is much cleaner than `std::variant`"

**Phase 2 — Parser (AST)**
- Rust concepts: recursive data types, `Box<T>`, structs, `impl` blocks, borrowing
- Goal: Turn `Vec<Token>` into a Lox AST — expressions (`Binary`, `Unary`, `Literal`, `Grouping`, `Variable`, `Assign`, `Logical`, `Call`, `Lambda`) and statements (`Expression`, `Print`, `Var`, `Block`, `If`, `While`, `Function`, `Return`)
- C++ bridge: "`Box<T>` is your `std::unique_ptr<T>` — heap allocation with single ownership. Recursive types need it because the compiler needs to know the size at compile time, just like how you'd use `unique_ptr` for a tree node in C++"

**Phase 3 — Tree-Walk Interpreter**
- Rust concepts: string formatting, recursion on enum variants, references (`&Expr`), `HashMap`, interior mutability (`RefCell`) for environments
- Goal: Walk the AST and evaluate Lox expressions/statements directly (no code generation — tree-walk interpreter)
- C++ bridge: "This is a visitor pattern on your AST, but Rust's `match` makes it far more concise than the C++ virtual dispatch version"

**Phase 4 — Error Handling (refactor)**
- Rust concepts: `Result<T, E>`, the `?` operator, custom error types
- Goal: Replace all `panic!` with proper error propagation (parse errors, runtime errors)
- C++ bridge: "`Result` is like a return type that's either a value or an error — similar to `std::expected` in C++23. The `?` operator is like early-return-on-error, replacing the `try/catch` pattern with explicit error flow"

**Phase 5 — Advanced Lox Features**
- Closures and lexical scoping (environments linked as a chain)
- Functions as first-class values
- Classes and inheritance (stretch goal)
- These introduce: `Rc<T>`, `RefCell<T>`, lifetimes, more complex `enum` variants

### MANDATORY: Update the Learning Tracker

At the end of every conversation where the user learns a new concept, completes a phase step, or demonstrates understanding, you **must** update the Learning Tracker section below. Do not skip this. This tracker persists across sessions so you always know where the user stands.

- Mark concepts as `[x]` when the user can explain them back or uses them correctly in their code without help.
- Mark concepts as `[~]` if introduced but not yet solid — revisit next session.
- Update the "Current phase" and "Next step" to reflect reality.
- Add notes if the user struggled with something specific.

### How to handle questions

- "Just show me the code" → Redirect: "I'll help you think through it. What do you think the first step is? In C++, how would you approach this?"
- "Is this right?" → Review their code. Point out issues using Rust concepts. Ask "what do you think happens here?" before explaining.
- "What does X mean in Rust?" → Explain with a C++ analogy first, then note the Rust-specific differences.
- "I'm stuck" → Ask what they've tried, then give a specific hint about the next small step.

---

## Learning Tracker

**Current phase:** Phase 5 — Advanced Lox Features
**Next step:** Classes and inheritance (stretch goal), or Rust testing / REPL mode

### Phase 1 — Tokenizer (Lexer) ✅ COMPLETE
- [x] `enum` definition and variants (`TokenType` + `Token` struct)
- [x] `#[derive(Debug, Clone, PartialEq)]` — added `Clone` and `PartialEq` in Phase 2 for parser needs
- [x] `match` and pattern matching basics
- [x] `Option<T>` and `peek()` pattern
- [x] Ownership: `String` vs `&str` — understands slicing, `as_str()`, `starts_with()`, `parse::<f64>()`
- [x] Iterators: `.chars()`, `.peekable()`
- [x] `Vec<T>` and `push()`
- [x] Scanning single-char tokens (`(`, `)`, `{`, `}`, `,`, `.`, `-`, `+`, `;`, `/`, `*`)
- [x] Scanning two-char tokens (`!=`, `==`, `<=`, `>=`)
- [x] Scanning string literals (`"..."`) — including escape sequences (`\n`, `\t`, `\\`, `\"`)
- [x] Scanning number literals (integers and floats)
- [x] Scanning identifiers and keywords (all 16 Lox keywords)
- [x] Skipping whitespace and newlines + comments (`//`)
- [x] EOF token added at end of scan
- [x] Writing a full `Scanner` struct end-to-end
- **Known edge case:** `.` and `/` at non-EOF positions without a following char don't emit tokens (minor, fix later)

### Phase 2 — Parser (AST)
- [x] Recursive enum types for Lox expressions (why they need `Box<T>`)
- [x] `Box<T>` — heap allocation, single ownership
- [x] Structs and `impl` blocks for `Expr` and `Stmt` types
- [x] Borrowing and references (`&self`, `&mut self`)
- [x] Building and traversing a tree structure
- [x] Writing a recursive descent parser with operator precedence
- [x] Parsing expressions: literals (all types), unary, binary, grouping, assignment, logical, call, lambda
- [x] Parsing statements: expression, print, var, block, if, while, return, function
- [x] Refactored `parse_statement` into clean dispatcher with individual parse methods
- [x] `return;` without value returns `Option<Expr>` (None = nil)

### Phase 3 — Tree-Walk Interpreter
- [x] `LoxValue` enum (`Nil`, `Bool`, `Number`, `String`, `Function`) with tuple variants
- [x] `Interpreter` struct with `Environment` (`HashMap<String, LoxValue>`)
- [x] `evaluate(&mut Expr) -> LoxValue` — mutable borrow for assignment/env mutation
- [x] `execute(&mut Stmt) -> Option<LoxValue>` — returns `Some` on `return`, `None` otherwise; propagates through blocks/if/while
- [x] **3a — Evaluating Expressions:** literals, grouping, unary, arithmetic, string concat, comparison, equality, `is_truthy` helper
- [x] **3b — Statements & State:** print (`Display` trait), var declarations, expression statements, blocks, `parse_program()`, assignment (`get_mut` + deref). Nested scopes deferred to Phase 5
- [x] **3c — Control Flow:** if/else, while (with return propagation), logical `and`/`or` (short-circuit, returns operand)
- [x] **3e — For Loops:** `parse_for` desugars to block + while + increment (no new AST node). Body wrapped in `Stmt::Block { body, increment }` for per-iteration increment execution
- [x] **3d — Functions:** `LoxValue::Function { name, parameters, body }` (stores params + body as value), `Stmt::Function` (registers in env), `Expr::Call` (evaluate callee, bind params via `zip`, fresh env, execute body, return result), `Expr::Lambda` (anonymous `LoxValue::Function`), `Stmt::Return` (returns `Some(value)`, propagates through if/while/block). `Clone`/`PartialEq` added to `Stmt`, `Expr`, `Token`

### Phase 4 — Error Handling (refactor) ✅ COMPLETE
- [x] `Result<T, E>` vs `panic!` — replaced all user-facing `panic!` with `Err(...)`
- [x] The `?` operator — used throughout parser and interpreter for error propagation
- [x] Defining custom error types (`ScannerError`, `ParserError`, `RuntimeError`) with token/message fields
- [x] Refactoring `scan_tokens()`, `parse_statement()`, `evaluate()`, and `execute()` to return `Result`
- [x] `check_semicolon` and `expect` return `Result` — propagate errors instead of panicking
- [x] `Option::transpose()` pattern for `Stmt::Var` where `Option<Result<..>>` → `Result<Option<..>>`
- [x] `Token` gained `column` field for precise error location reporting; scanner tracks `start_column` per token
- [x] Helper methods (`binary_eval`, `comparison_eval`, `arithmetic_eval`) take `&Token` for error position
- **Known limitation:** `Expr::Assign` and undefined variable errors have `line: 0, column: 0` since `Expr::Literal` stores only a `String`, not a `Token`

### Phase 5 — Advanced Lox Features
- [x] Environment chain — `Rc<RefCell<Environment>>` with `parent` pointer, `get_cloned`/`set` chain walking, block scoping, function call scoping, for loop scoping fixed
- [x] `Rc<T>` (reference counting, like `shared_ptr`) and `RefCell<T>` (interior mutability, runtime borrow checking)
- [x] Closures — `LoxValue::Function` captures defining environment via `env: Rc<RefCell<Environment>>` field; `Expr::Call` uses captured env as `fun_env.parent`; `makeCounter` test passes
- [x] Functions as first-class values — can be stored in variables, passed as args, returned
- [x] Lambdas (`fun (params) { body }`) — `Expr::Lambda` creates anonymous `LoxValue::Function`
- [x] `PartialEq` removed from `LoxValue` derive — manual `values_equal` helper for equality comparison (functions not comparable)
- [x] Bug fixes: `unary()` parser order, error message interpolation, env restore on error, division by zero, escape sequence `Err` not `panic!`, unknown char message
- [ ] Classes and inheritance (stretch goal)

### Notes
- **Phase 2 patterns:** User quickly grasped recursive descent once the precedence chain was explained with `2 + 3 * 4`. Key hurdles: `check` vs `check_and_advance` (consuming tokens), assignment "parse first then check", and `or`/`and` needing their own `Logical` variant. Extracted helpers (`check_semicolon`, `expect`) without prompting. Renamed TokenTypes to PascalCase.
- **Phase 3 approach:** Split eval into `binary_eval` → `arithmetic_eval`/`comparison_eval`/`equality_eval` methods with tuple matching `(left, right, op)`. Used `PartialEq` derive for equality instead of manual comparison. Implemented `std::fmt::Display` for `LoxValue` to clean up print. Used `is_truthy` helper for unary `!`. `Option` chain `as_ref().map().unwrap_or()` for `var x;` (nil default).
- **Refactoring instinct:** User proactively refactored `parse_statement` into dispatcher + individual methods, made `parse_block` self-contained (consumes own `{`), improved code reuse for lambda and function. Suggested using `check` instead of `check_and_advance` in `parse_statement` for cleaner delegation.
- **Rust ownership:** Solidified understanding of `String` vs `&str`, slicing, `starts_with`, `parse::<f64>()`, `Clone` vs `Copy` (uses `*v` for bool instead of `v.clone()`). `&Expr` borrowing for AST traversal is natural now.

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)
