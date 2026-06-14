# network-parse

A command-line RSS/Atom feed aggregator written in Rust. Fetch one or many feed URLs concurrently, auto-detect the format, parse into a normalized model, and re-serialize as JSON, RSS, or Atom.

This is a **Rust learning project**. The goal is learning language and ecosystem concepts in context — async/await, tokio, serde, thiserror, quick-xml streaming, trait-based polymorphism — rather than shipping a production aggregator. See [`CLAUDE.md`](CLAUDE.md) for the phase-by-phase teaching plan and [`PRD.md`](PRD.md) for full requirements.

## Features

- **Format auto-detection** — sniffs the XML root element (`<rss>` vs `<feed>`), no manual hinting.
- **RSS 2.0 + Atom parsing** — streaming parsers built on `quick-xml`'s pull API. Lenient: missing optional fields become `None`, not errors.
- **Date normalization** — RSS RFC 2822 and Atom ISO 8601 dates both parse into `DateTime<Utc>`.
- **Three output formats** — re-serialize a parsed feed as JSON, RSS, or Atom, including **cross-format conversion** (e.g. fetch an RSS feed, emit Atom).
- **Concurrent fetching** — multiple URLs fetched in parallel via `tokio::spawn`; partial failures don't abort the batch.
- **Round-trip property** — `parse(serialize(feed))` holds for both XML formats, with tests.

## Install / build

Requires a recent Rust toolchain (edition 2024).

```bash
cargo build                    # compile
cargo test                     # run the test suite (23 tests)
cargo clippy                   # lint
cargo fmt                      # format
```

## Usage

```bash
# Fetch a single feed, default output (JSON, 10 entries)
cargo run -- https://hnrss.org/frontpage

# Convert an RSS feed to Atom output, limit 5 entries
cargo run -- --format atom --limit 5 https://hnrss.org/frontpage

# Fetch multiple feeds concurrently
cargo run -- --format rss https://hnrss.org/frontpage https://www.theverge.com/rss/index.xml
```

### CLI flags

| Flag | Description | Default |
|---|---|---|
| `<urls...>` | One or more feed URLs (positional) | — |
| `-f, --format <FORMAT>` | Output format: `json`, `rss`, `atom` | `json` |
| `-l, --limit <N>` | Max entries per feed | `10` |

## Architecture

A staged pipeline, each stage a trait with format-specific implementations:

```
URL string
  → FeedFetcher::fetch(url)         → raw bytes
  → FeedDetector::detect(&bytes)    → FeedFormat { Rss | Atom | Unknown }
  → RssParser | AtomParser .parse() → Feed
  → FeedSerializer::serialize()     → JSON / RSS / Atom bytes
```

### Trait hierarchy

- **`FeedFetcher`** — async HTTP GET, returns raw bytes. Default impl via `reqwest`.
- **`FeedDetector`** — sniffs XML root element, returns `FeedFormat` enum.
- **`FeedParser`** — parses XML string → `Feed`. Two impls: `RssParser`, `AtomParser`.
- **`FeedSerializer`** — serializes `Feed` → bytes. Three impls: `JsonSerializer`, `RssSerializer`, `AtomSerializer`.

### Core model

- `Feed` — title, link, description, updated, list of `Entry`.
- `Entry` — id, title, link, summary, content, `Author`, published/updated timestamps.
- `Author` — name, optional email, optional uri.

### Source layout

```
src/
  lib.rs              — crate root, re-exports
  main.rs             — binary entry, tokio main, clap CLI
  fetch.rs            — FeedFetcher trait + reqwest impl
  detect.rs           — FeedDetector + FeedFormat enum
  model.rs            — Feed, Entry, Author (serde derives)
  error.rs            — thiserror-based error types
  parser/
    mod.rs            — FeedParser trait
    rss.rs            — RSS 2.0 parser
    atom.rs           — Atom parser
  serializer/
    mod.rs            — FeedSerializer trait
    json.rs           — JSON via serde
    rss.rs            — RSS 2.0 output via quick-xml Writer
    atom.rs           — Atom output via quick-xml Writer
```

## Concepts learned

Built incrementally, each phase introducing one or two ideas:

- serde derives (`Serialize`/`Deserialize`), `#[serde(default)]` for lenient deserialization
- `thiserror` for derived error enums with `#[from]` / `#[error(transparent)]`
- Enum dispatch + root-element sniffing for format detection
- `quick-xml` streaming — `Reader::read_event()` pull parsing, matching on `Event` variants
- quick-xml `Writer` — the inverse push API, automatic XML escaping
- `chrono` date parsing (RFC 2822 + ISO 8601 → `DateTime<Utc>`)
- `async fn`, `#[tokio::main]`, `#[tokio::test]`, futures, `.await`
- `tokio::spawn`, `JoinHandle`, `futures::future::join_all` for concurrent fetching
- clap derive API, `ValueEnum` for typed CLI flags
- trait-based polymorphism (`dyn FeedSerializer`)

## Notable implementation detail

Entity-encoded text (`&amp;`, `&lt;`, …) is split by `quick-xml` into separate `Text` and `GeneralRef` events. The parsers accumulate fragments (`push_str`), decode each general reference, and trim at the element's close event — so text like `Tom &amp; Jerry &lt;cartoon&gt;` round-trips correctly through parse and serialize. Tests cover this in both parsers.

## Dependencies

| Crate | Purpose |
|---|---|
| `tokio` | async runtime |
| `reqwest` | async HTTP client |
| `quick-xml` | streaming XML reader + writer |
| `serde` + `serde_json` | serialization |
| `chrono` | date/time parsing |
| `clap` | CLI argument parsing |
| `thiserror` | error type derivation |
| `futures` | `join_all` for concurrent fetch aggregation |

## Status

All eight planned phases complete. See [Current Status](CLAUDE.md#current-status) in `CLAUDE.md`.
