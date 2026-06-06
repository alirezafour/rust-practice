# Progress Backup — Lox Interpreter Learning Journey

Complete record of all phases, concepts learned, and notes. Kept as reference for teaching others.

## Phase 1 — Tokenizer (Lexer) ✅ COMPLETE
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

## Phase 2 — Parser (AST)
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

## Phase 3 — Tree-Walk Interpreter
- [x] `LoxValue` enum (`Nil`, `Bool`, `Number`, `String`, `Function`) with tuple variants
- [x] `Interpreter` struct with `Environment` (`HashMap<String, LoxValue>`)
- [x] `evaluate(&mut Expr) -> LoxValue` — mutable borrow for assignment/env mutation
- [x] `execute(&mut Stmt) -> Option<LoxValue>` — returns `Some` on `return`, `None` otherwise; propagates through blocks/if/while
- [x] **3a — Evaluating Expressions:** literals, grouping, unary, arithmetic, string concat, comparison, equality, `is_truthy` helper
- [x] **3b — Statements & State:** print (`Display` trait), var declarations, expression statements, blocks, `parse_program()`, assignment (`get_mut` + deref). Nested scopes deferred to Phase 5
- [x] **3c — Control Flow:** if/else, while (with return propagation), logical `and`/`or` (short-circuit, returns operand)
- [x] **3e — For Loops:** `parse_for` desugars to block + while + increment (no new AST node). Body wrapped in `Stmt::Block { body, increment }` for per-iteration increment execution
- [x] **3d — Functions:** `LoxValue::Function { name, parameters, body }` (stores params + body as value), `Stmt::Function` (registers in env), `Expr::Call` (evaluate callee, bind params via `zip`, fresh env, execute body, return result), `Expr::Lambda` (anonymous `LoxValue::Function`), `Stmt::Return` (returns `Some(value)`, propagates through if/while/block). `Clone`/`PartialEq` added to `Stmt`, `Expr`, `Token`

## Phase 4 — Error Handling (refactor) ✅ COMPLETE
- [x] `Result<T, E>` vs `panic!` — replaced all user-facing `panic!` with `Err(...)`
- [x] The `?` operator — used throughout parser and interpreter for error propagation
- [x] Defining custom error types (`ScannerError`, `ParserError`, `RuntimeError`) with token/message fields
- [x] Refactoring `scan_tokens()`, `parse_statement()`, `evaluate()`, and `execute()` to return `Result`
- [x] `check_semicolon` and `expect` return `Result` — propagate errors instead of panicking
- [x] `Option::transpose()` pattern for `Stmt::Var` where `Option<Result<..>>` → `Result<Option<..>>`
- [x] `Token` gained `column` field for precise error location reporting; scanner tracks `start_column` per token
- [x] Helper methods (`binary_eval`, `comparison_eval`, `arithmetic_eval`) take `&Token` for error position

## Phase 5 — Advanced Lox Features
- [x] Environment chain — `Rc<RefCell<Environment>>` with `parent` pointer, `get_cloned`/`set` chain walking, block scoping, function call scoping, for loop scoping fixed
- [x] `Rc<T>` (reference counting, like `shared_ptr`) and `RefCell<T>` (interior mutability, runtime borrow checking)
- [x] Closures — `LoxValue::Function` captures defining environment via `env: Rc<RefCell<Environment>>` field; `Expr::Call` uses captured env as `fun_env.parent`; `makeCounter` test passes
- [x] Functions as first-class values — can be stored in variables, passed as args, returned
- [x] Lambdas (`fun (params) { body }`) — `Expr::Lambda` creates anonymous `LoxValue::Function`
- [x] `PartialEq` removed from `LoxValue` derive — manual `values_equal` helper for equality comparison (functions not comparable)
- [x] Bug fixes: `unary()` parser order, error message interpolation, env restore on error, division by zero, escape sequence `Err` not `panic!`, unknown char message
- [x] **5a — Classes and Instances:** `LoxValue::Class { name, methods: HashMap }`, `LoxValue::Instance { fields: Rc<RefCell<HashMap>>, class_name }`, `Stmt::Class` (stores methods in env), `Expr::Get` (property access: fields → methods lookup), `Expr::Set` (field mutation via `Environment::set_field`), `Expr::Call` on `Class` creates `Instance`. Field/method storage separation, instance state isolation.
- [x] **5b — `this` keyword:** `this` bound in method env via `fun_env.map.insert("this", obj_val)`. `set_field` walks parent chain for `this`. `Rc<RefCell<HashMap>>` for shared instance fields — `this` and variable point to same instance.
- [x] **5c — Inheritance:** `class Sub < Super` syntax (`<` in parser, `superclass: Option<Token>` in `Stmt::Class`). `LoxValue::Class` stores `superclass`. Recursive `lookup_class` walks superclass chain for method resolution. `bind_method` helper extracts this-binding + param-binding + execute logic. Superclass validated at class declaration time. Method override supported (subclass method takes precedence). Multi-level inheritance tested (3-level chain).
- [x] **5d — `super` keyword:** `Expr::Super { identifier: Token }` parsed as dedicated AST node (not `Expr::Get`). `Expr::Call` intercepts `Super` callee, reads `__super` (superclass name as `LoxValue::String`) and `this` (current instance) from environment, calls `lookup_class` + `lookup_superclass_of` + `bind_this_method`. `__super` bound in `bind_this_method` alongside `this`. Multi-level super chain tested (3 levels). Parser rejects `super` without dot. 128 tests passing.

## Phase 6 — Idiomatic Rust: Traits
- [x] `#[derive(Debug, Clone, PartialEq)]` — auto-implemented traits (Phase 1-2)
- [x] `impl std::fmt::Display for LoxValue` — first manual trait impl (Phase 3)
- [x] `std::error::Error` — implemented for `RuntimeError`, `ParserError`, `ScannerError` with `Display` + empty `impl std::error::Error`. `main.rs` updated to use `println!("{err}")` instead of manual field access.
- [x] `Iterator` trait — implemented for `Scanner<'a>`. Struct stores `Peekable<Chars<'a>>` instead of `source_code: String`. Lifetimes learned: `Scanner<'a>` borrows source via `&'a str`. `next()` skips whitespace/comments in loop, passes first real char to `get_next_token(c, start_column)`. `scan_tokens()` delegates to `collect()`. All callers updated to `Scanner::new(source)`. 128 tests pass.
- [x] `From`/`Into` — implemented `From<f64>`, `From<bool>`, `From<String>` for `LoxValue`. Enables `.into()` conversion from Rust primitives. `?` operator uses `From` internally for `Box<dyn Error>` conversions.
- [x] Custom trait definitions — `SourceLocation` trait in scanner.rs with `line()`, `column()` required methods and `format_location()` default method. Implemented for `Token`, `ScannerError`. Used in `Display` impls.
- [x] Trait bounds on generics — `format_error<T: SourceLocation>` with `where` clause. Compile-time monomorphization (like C++ templates with concepts).
- [x] `dyn Trait` — understood conceptually via `Box<dyn std::error::Error>` already in use. Runtime vtable dispatch, same as C++ virtual. No separate exercise needed.
- [x] Default trait methods — `format_location()` in `SourceLocation` provides default impl, types only override `line()`/`column()`
- [x] Operator overloading — skipped. `LoxValue` is dynamically typed, runtime type checks make operator overloading low-value. Concept understood.

## Design Notes

### Refactoring instinct
Proactively refactors into helpers (`check_semicolon`, `expect`, `bind_this_method`, `call_get_handle`, `lookup_superclass_of`). Suggests cleaner patterns without prompting.

### Rust ownership
Solid on `String`/`&str`, `Clone`/`Copy`, `&Expr` borrowing. `Rc<RefCell<>>` for shared state understood via `shared_ptr` analogy.

### Classes
`HashMap<String, LoxValue>` for methods/fields. `Rc<RefCell<HashMap>>` for shared instance fields (key bug: cloning `Instance` created separate objects). `Expr::Get` lookup: fields → class methods → superclass chain.

### Inheritance
`lookup_class` recursive walk (same pattern as env chain). `bind_this_method` handles `this` + `__super` binding. `Expr::Call` intercepts both `Get` and `Super` callees before regular function call path.

### Super keyword
Key design insight — `super` can't be a value stored as a regular variable. It needs its own `Expr` variant because the method must be found on the superclass and bound to current `this`. `Expr::Super` is never evaluated standalone; always handled inside `Expr::Call`. `__super` stored as `LoxValue::String` (class name) in method env, not as `LoxValue::Class`.

## C++ → Rust Reference Table

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
| pure virtual class / interface | `trait` (no data, only method signatures) |
| `template <typename T> requires X` | generics with trait bounds: `fn foo<T: Display>(...)` |
| `operator+` overloading | `impl std::ops::Add for Type` |
| implicit conversion constructor | `impl From<Other> for Type` / `.into()` |
