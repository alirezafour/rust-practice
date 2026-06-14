# PRD — RSS/Atom Feed Aggregator

## Problem

RSS and Atom feeds use different XML schemas. Aggregating them means parsing two formats, normalizing into one model, and exposing results as JSON. Real-world feeds are messy — malformed XML, missing fields, varying date formats, mixed encodings.

## Goal

Build a Rust library + binary that:

1. **Fetches** RSS/Atom feeds over HTTP asynchronously
2. **Detects** feed format (RSS 2.0 vs Atom) automatically
3. **Parses** XML into a unified `Feed` / `Entry` model
4. **Normalizes** fields (dates, authors, content) across formats
5. **Outputs** structured JSON to console (MVP)
6. **Serializes** the unified model back to XML (round-trip capable)

All parsing is trait-based — format detection and parsing are swappable at compile time.

## Architecture

```
HTTP (reqwest)
  │
  ▼
┌──────────────────┐
│  FeedFetcher     │  async GET, return XML bytes
└────────┬─────────┘
         │ bytes
         ▼
┌──────────────────┐
│  FeedDetector    │  sniff root element → Rss | Atom | Unknown
└────────┬─────────┘
         │ format tag
         ▼
┌──────────────────┐
│  FeedParser      │  trait: parse XML → Feed
│  ├─ RssParser    │  RSS 2.0 impl
│  └─ AtomParser   │  Atom impl
└────────┬─────────┘
         │ Feed
         ▼
┌──────────────────┐
│  FeedSerializer  │  trait: Feed → JSON or XML
└──────────────────┘
```

### Trait Abstractions

| Trait | Responsibility |
|---|---|
| `FeedFetcher` | Async HTTP GET, return raw bytes |
| `FeedDetector` | Inspect XML root element, return format enum |
| `FeedParser` | Parse XML string → `Feed` |
| `FeedSerializer` | Serialize `Feed` → JSON or XML bytes |

Concrete implementations selectable via generics or enum dispatch.

### Core Data Model

```rust
struct Feed {
    title: String,
    link: String,
    description: Option<String>,
    updated: Option<DateTime<Utc>>,
    entries: Vec<Entry>,
}

struct Entry {
    id: String,
    title: String,
    link: Option<String>,
    summary: Option<String>,
    content: Option<String>,
    author: Option<Author>,
    published: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
}

struct Author {
    name: String,
    email: Option<String>,
    uri: Option<String>,
}
```

### Pipeline

1. `FeedFetcher::fetch(url)` → raw bytes
2. `FeedDetector::detect(&bytes)` → `FeedFormat::Rss | FeedFormat::Atom`
3. Match format → `RssParser::parse()` or `AtomParser::parse()` → `Feed`
4. Print as JSON to console
5. (Optional) `FeedSerializer::to_json()` / `FeedSerializer::to_xml()` for output

## Requirements

### Must Have

- [ ] Async HTTP fetching with reqwest + tokio
- [ ] XML parsing with roxmltree (or quick-xml streaming)
- [ ] Auto-detect RSS 2.0 vs Atom from root element
- [ ] Unified `Feed` / `Entry` model
- [ ] Date normalization (RFC 2822, ISO 8601, loose parsing)
- [ ] JSON output via serde_json
- [ ] Trait-based parser — swappable via generics or enum dispatch
- [ ] Console output of parsed feeds
- [ ] Unit tests (TDD approach)
- [ ] Integration tests against real-world feed fixtures
- [ ] Round-trip: parse → serialize → parse for both formats

### Nice to Have

- [ ] CLI with clap (URL arg, --format json/xml, --limit N)
- [ ] Parallel fetching of multiple feeds
- [ ] Feed caching (conditional GET with ETag/Last-Modified)
- [ ] Logging with tracing
- [ ] Error recovery on malformed feeds (best-effort parsing)

## Tech Stack

| Concern | Crate |
|---|---|
| Async runtime | `tokio` |
| HTTP client | `reqwest` |
| XML parsing | `quick-xml` (streaming, low allocation) |
| JSON serialization | `serde` + `serde_json` |
| Date/time parsing | `chrono` |
| CLI (nice-to-have) | `clap` |
| Logging | `tracing` + `tracing-subscriber` |
| Error handling | `thiserror` |
| Testing | built-in `#[test]` + `tokio::test` |

## Feed Format Reference

### RSS 2.0

Root element: `<rss version="2.0"><channel>...</channel></rss>`

Key elements: `<title>`, `<link>`, `<description>`, `<item>` (with `<title>`, `<link>`, `<description>`, `<pubDate>`, `<author>`)

Dates: RFC 2822 (e.g. `Mon, 01 Jan 2024 12:00:00 +0000`)

### Atom

Root element: `<feed xmlns="http://www.w3.org/2005/Atom">`

Key elements: `<title>`, `<link href="...">`, `<subtitle>`, `<entry>` (with `<title>`, `<link>`, `<summary>`, `<content>`, `<published>`, `<updated>`, `<author><name>`)

Dates: ISO 8601 / RFC 3339 (e.g. `2024-01-01T12:00:00Z`)
