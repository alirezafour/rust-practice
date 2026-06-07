# CLAUDE.md

This is a **Rust learning project** — building an RSS/Atom feed aggregator.
The goal is learning new Rust concepts, not just shipping features.

**Full requirements:** See `PRD.md`.

## Learner Profile

Completed `lox_interpreter` — knows: ownership, borrowing, lifetimes, enums, pattern matching, `Result`/`Option`, `Box`, `Rc<RefCell<>>`, traits, closures, `HashMap`, `Vec`.

**Gaps this project fills:** async/await, tokio runtime, serde, thiserror, external crates, `dyn Trait`, concurrent fetching, streaming parsers.

## Teaching Guidelines

### How to teach

1. **Introduce concepts before code.** When a task needs a new concept (e.g. `async fn`, `#[derive(Serialize)]`, `Arc<Mutex<>>`), explain what it is and *why* it exists before showing how. 2-4 sentences max, then code.
2. **Don't over-explain known concepts.** The user understands ownership, borrowing, enums, `Result`, traits. No need to re-explain these.
3. **Point out ecosystem idioms.** When using serde/tokio/thiserror/clap, note the convention — e.g. "thiserror is how Rust projects derive Error traits instead of manual impl."
4. **Ask before simplifying.** If the user writes something verbose but correct, ask if they want the idiomatic version before rewriting it.
5. **Explain compile errors.** When the user hits a confusing borrow checker / lifetime / async error, decode it in plain English before suggesting a fix.

### Code review

When reviewing code, check:
- **Idiomatic Rust**: Prefer `?` over `match` for error propagation, use `if let` when one variant matters, iterator chains over manual loops when clearer.
- **Async correctness**: No blocking inside async fns, proper `Send` bounds, no unnecessary `.clone()`.
- **Serde hygiene**: Proper `#[serde(rename_all)]`, `#[serde(default)]` for optional fields, no untagged enums unless needed.
- **Error handling**: Use `thiserror` derives, propagate with `?`, don't swallow errors with `.unwrap()` outside tests.

When you spot an issue, point out the smell, explain *why*, ask how they'd fix it.

### Phase progression

Guide the user through phases. Each phase introduces 1-2 new concepts. Don't skip ahead — the user should learn each concept in context.

**Phase 1: Model + Error types** (serde, thiserror, Option-heavy structs)
- Define `Feed`, `Entry`, `Author` in `model.rs` with serde derives
- Define error types in `error.rs` with thiserror
- Write tests for serialization round-trips
- **New concepts:** `#[derive(Serialize, Deserialize)]`, `thiserror`, serde attributes

**Phase 2: Format detection** (enum dispatch, string matching)
- Define `FeedFormat` enum in `detect.rs`
- Implement `FeedDetector` that sniffs XML root element
- Write tests with fixture XML snippets
- **New concepts:** enum as state machine, basic string/bytes inspection

**Phase 3: XML parsing — RSS** (quick-xml, streaming iteration)
- Define `FeedParser` trait in `parser/mod.rs`
- Implement `RssParser` in `parser/rss.rs`
- Write tests with RSS fixture XML
- **New concepts:** `quick-xml` streaming API, `Reader::read_event()`, match on XML events

**Phase 4: XML parsing — Atom** (trait implementations, date parsing)
- Implement `AtomParser` in `parser/atom.rs`
- Add date normalization (RFC 2822 vs ISO 8601 → `DateTime<Utc>`)
- Write tests with Atom fixture XML
- **New concepts:** `chrono` date parsing, trait-based polymorphism

**Phase 5: Async HTTP fetching** (tokio, reqwest, async/await)
- Define `FeedFetcher` trait in `fetch.rs`
- Implement with reqwest
- Write tests with `tokio::test` and mocked responses
- **New concepts:** `async fn`, `#[tokio::main]`, futures, `.await`, `tokio::test`

**Phase 6: CLI + pipeline wiring** (clap, async main, pulling it together)
- Add clap CLI in `main.rs`
- Wire the full pipeline: fetch → detect → parse → serialize → print
- **New concepts:** `clap` derive API, async pipeline composition

**Phase 7: Concurrent fetching** (tokio::spawn, futures::join!, Arc)
- Fetch multiple feeds concurrently
- Aggregate results
- **New concepts:** `tokio::spawn`, `JoinHandle`, `futures::join!`, `Arc` for shared state

**Phase 8: XML round-trip serialization** (reverse direction, edge cases)
- Implement `FeedSerializer` for XML output
- Verify `parse(serialize(feed))` round-trip property
- Handle malformed feeds gracefully
- **New concepts:** builder pattern, error recovery, property-based testing

## Commands

```bash
cargo build                  # compile
cargo test                   # run all tests
cargo test test_name         # run single test (substring match)
cargo test -- --nocapture    # run tests with stdout visible
cargo run                    # run binary
cargo run -- <feed-url>      # fetch and parse a single feed
cargo clippy                 # lint
cargo fmt                    # format
cargo fmt -- --check         # check formatting
```

## Architecture

### Trait Hierarchy

- **`FeedFetcher`** — async HTTP GET, returns raw bytes. Default impl via reqwest.
- **`FeedDetector`** — sniffs XML root element (`<rss>` vs `<feed>`), returns `FeedFormat` enum.
- **`FeedParser`** — parses XML string → `Feed`. Two impls: `RssParser`, `AtomParser`.
- **`FeedSerializer`** — serializes `Feed` → JSON or XML bytes. Impl per output format.

### Core Types

- `Feed` — title, link, description, updated timestamp, list of `Entry`.
- `Entry` — id, title, link, summary, content, `Author`, published/updated timestamps.
- `Author` — name, optional email, optional uri.
- `FeedFormat` — enum: `Rss`, `Atom`, `Unknown`.

### Pipeline Flow

```
URL string
  → FeedFetcher::fetch(url) → raw bytes
  → FeedDetector::detect(&bytes) → FeedFormat
  → match format → RssParser::parse() | AtomParser::parse() → Feed
  → FeedSerializer::to_json() → print to console
```

### Directory Layout (target)

```
src/
  lib.rs              — re-exports, crate root
  main.rs             — binary entry, tokio main, CLI
  fetch.rs            — FeedFetcher trait + reqwest impl
  detect.rs           — FeedDetector + FeedFormat enum
  parser/
    mod.rs            — FeedParser trait
    rss.rs            — RSS 2.0 parser
    atom.rs           — Atom parser
  serializer/
    mod.rs            — FeedSerializer trait
    json.rs           — JSON output via serde
    xml.rs            — XML round-trip serializer
  model.rs            — Feed, Entry, Author structs (Deserialize/Serialize)
  error.rs            — thiserror-based error types
tests/
  integration_test.rs — round-trip and real-feed fixture tests
  fixtures/           — sample .xml files (RSS + Atom)
```

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` (full features) | async runtime |
| `reqwest` | async HTTP client |
| `quick-xml` | streaming XML parser |
| `serde` + `serde_json` | JSON serialization |
| `chrono` | date/time parsing (RFC 2822 + ISO 8601) |
| `clap` | CLI argument parsing |
| `tracing` + `tracing-subscriber` | structured logging |
| `thiserror` | error type derivation |

## Key Domain Concepts

- **RSS 2.0**: root `<rss version="2.0">`, items in `<channel><item>`. Dates RFC 2822. No namespaced author (just `<author>` email string).
- **Atom**: root `<feed xmlns="http://www.w3.org/2005/Atom">`. Entries in `<entry>`. Dates ISO 8601. Author is `<author><name>` element. Content can be inline text/html/xhtml.
- **Date normalization**: RSS uses RFC 2822, Atom uses ISO 8601 — both must parse into `DateTime<Utc>`.
- **Malformed feeds**: real-world feeds have missing fields, inconsistent namespaces, HTML in text fields. Parser must be lenient — missing optional fields become `None`, not errors.

## Development Approach

- **TDD**: write failing test first, then implement. All tests must pass before marking work done.
- **Traits first**: define trait in `mod.rs`, implement in separate file.
- **Round-trip property**: `parse(serialize(feed))` must produce equivalent `Feed`.
- **Fixture-driven**: keep real-world XML snippets in `tests/fixtures/` for integration tests.

## Current Status

**Phase 0: Not started** — empty `main.rs`, no dependencies added yet.
