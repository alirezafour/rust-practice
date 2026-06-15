# CLAUDE.md

This is a **Rust learning project** — building a parallel duplicate file finder.
The goal is learning Rust **concurrency** concepts (threads, `Arc`/`Mutex`, atomics, channels), not just shipping a dedup tool.

**Full requirements:** See `PRD.md`.

## Learner Profile

Completed `lox_interpreter` (ownership, borrowing, lifetimes, enums, pattern matching, `Result`/`Option`, `Box`, `Rc<RefCell<>>`, traits, closures, `HashMap`, `Vec`) **and** `rss-atom-parser` (async/await, tokio, serde, thiserror, external crates, `dyn Trait`, concurrent *task* fetching via `tokio::spawn` + `Arc`).

**Gaps this project fills:** OS **threads** (not async tasks), `Arc`/`Mutex`/`RwLock` shared state, **atomics** (`AtomicU64`/`AtomicBool`, `Ordering`), `Condvar`, `mpsc` channels, `thread::scope`, `Send`/`Sync`, graceful shutdown.

> Key distinction from rss-atom-parser: there you used *async tasks* (`tokio::spawn`) on a runtime. Here you use *OS threads* (`std::thread::spawn`/`scope`) directly — no runtime, no `await`. Shared mutable state needs real locks, not `await` points.

## Teaching Guidelines

### How to teach

1. **Introduce concepts before code.** When a task needs a new concept (e.g. `Arc<Mutex<>>`, `AtomicU64::fetch_add`, `thread::scope`, `Ordering`), explain what it is and *why* it exists before showing how. 2-4 sentences max, then code.
2. **Don't over-explain known concepts.** The user understands ownership, borrowing, enums, `Result`, traits, serde, async. No need to re-explain these.
3. **Concurrency is the lesson.** When two designs work, prefer the one that teaches the primitive (e.g. `Arc<Mutex<HashMap>>` collector over a single-threaded fold) — *unless* it's clearly worse. Explain the tradeoff either way.
4. **Explain data races & `Send`/`Sync`.** When the borrow checker rejects a thread spawn ("`*Mutex` cannot be sent"), decode it: which trait is missing, why, what `Arc`/`Mutex` add.
5. **Explain `Ordering`.** When using atomics, say why `Relaxed` is enough here (counters) vs when `AcqRel`/`SeqCst` would matter.
6. **Ask before simplifying.** If the user writes verbose-but-correct thread code, ask if they want the idiomatic version before rewriting it.
7. **Explain compile errors.** Decode borrow-checker / lifetime / `Send`-bound errors in plain English before suggesting a fix.

### Code review

When reviewing code, check:
- **Concurrency correctness:** no data races, `Arc` where sharing across threads, `Mutex`/`RwLock` around shared mutable state, correct `Ordering` on atomics, no holding locks across I/O.
- **No lock-amplification:** don't clone-then-lock when a scoped borrow works; don't lock per-iteration when once outside the loop works.
- **Graceful shutdown:** long-running workers check a stop flag; Ctrl-C doesn't corrupt output.
- **Idiomatic Rust:** `?` over `match`, `if let`, iterator chains when clearer, `scope` over `'static` `Arc` when borrow is local.
- **Error handling:** `thiserror` derives, propagate with `?`, no `.unwrap()` outside tests, I/O errors per-file (skip + log) not fatal.
- **Safety:** destructive actions (`delete`/`hardlink`) default to `--dry-run`, require explicit confirmation.

When you spot an issue, point out the smell, explain *why*, ask how they'd fix it.

### Phase progression

Guide the user through phases. Each phase introduces 1-2 new concepts. Don't skip ahead — learn each concept in context. **No `rayon`** — build parallelism by hand; that's the point.

**Phase 1: Model + Error types** (thiserror, serde, newtypes)
- Define `FileInfo`, `FileHash` (newtype around `[u8;32]`), `DuplicateGroup`, `Action`, `ScanConfig` in `model.rs`
- Define `DupError` in `error.rs` with thiserror
- serde derives for JSON output + `FileHash` hex (De)Serialize
- **New concepts:** newtype pattern, `Hash`/`Eq`/`PartialEq` derives for use as HashMap key

**Phase 2: Directory walker** (walkdir, filesystem)
- `Walker::scan(&ScanConfig) -> Vec<FileInfo>` in `walker.rs`
- Recursive, follow-symlinks flag, regular-files-only, min-size filter
- Tests with `tempfile` fixtures
- **New concepts:** `walkdir` iterator API, `fs::metadata`, symlink vs regular file

**Phase 3: Size grouping** (cheap prefilter)
- `SizeGrouper::group(Vec<FileInfo>) -> Vec<(u64, Vec<PathBuf>)>` in `grouper.rs`
- Drop sizes occurring once; keep candidate groups
- Tests
- **New concepts:** `HashMap` grouping/fold, the three-stage-filter insight (why prefiltering matters)

**Phase 4: Parallel hashing — the core** (`thread::scope`, `Arc`, `Mutex`, `Send`/`Sync`)
- `Hasher::hash_parallel(groups, &ScanConfig) -> Vec<DuplicateGroup>` in `hasher.rs`
- Spawn N worker threads via `thread::scope`
- Shared collector: `Arc<Mutex<HashMap<FileHash, Vec<PathBuf>>>>`
- Workers pull jobs from a shared `Arc<Mutex<VecDeque<Job>>>` (or partition groups)
- sha256 each candidate file
- **New concepts:** `std::thread::scope`, `Arc`, `Mutex`, `lock().unwrap()`, `Send`/`Sync`, ownership across threads
- *Documented alternatives:* mpsc channel collector (message-passing), hand-rolled `Condvar` job queue — try at least one as a refactor to feel the tradeoff

**Phase 5: Progress + graceful stop** (atomics, signals)
- `Arc<AtomicU64>` for bytes-hashed and files-hashed counters
- Progress output (plain eprintln, or `indicatif` bar) polling the atomics
- `Arc<AtomicBool>` stop flag + Ctrl-C handler (`ctrlc` crate or `signal_hook`)
- Workers check stop flag, drain early
- **New concepts:** `AtomicU64`, `AtomicBool`, `fetch_add`, `load`/`store`, `Ordering::Relaxed` vs `AcqRel`, signal handling

**Phase 6: CLI + reporter** (clap, output formats)
- clap derive CLI in `main.rs`: paths, `--min-size`, `--threads`, `--format text|json`, `--action list|delete|hardlink|symlink`, `--dry-run`
- `Reporter::report(groups, &ScanConfig)` in `reporter.rs`: text table (paths, size, wasted bytes) + JSON via serde
- Wasted-bytes calculation: `(paths.len() - 1) * size` per group
- **New concepts:** clap derive (positional + flags + `ValueEnum`), serde JSON output, CLI ergonomics

**Phase 7: Reclaim actions + robustness** (filesystem mutation, safety)
- `--action delete` / `hardlink` / `symlink`: implement in `reporter.rs`/`actions.rs`
- `--dry-run` default ON for destructive actions; confirmation prompt
- Symlink-loop protection in walker, per-file permission errors skipped+logged, empty files handled (size 0 → skip or group-all-empty)
- Integration test: create dups in tempdir, scan, assert groups
- **New concepts:** `fs::hard_link`, `fs::remove_file`, destructive-op safety patterns, robust error isolation

**Phase 8 — CDC bonus:** *Not detailed here to keep focus on MVP.* When Phases 1-7 are complete and tests are green, read the **"Bonus: Block-Level Dedup (Phase 8)"** section in `PRD.md`, then we'll expand this CLAUDE.md with CDC phases (FastCDC, rolling hash, content-addressable chunk store).

## Commands

```bash
cargo build                  # compile
cargo test                   # run all tests
cargo test test_name         # run single test (substring match)
cargo test -- --nocapture    # run tests with stdout visible
cargo run -- <dir>           # scan a directory for duplicates
cargo run -- --format json --threads 8 ~/photos
cargo clippy                 # lint
cargo fmt                    # format
cargo fmt -- --check         # check formatting
cargo doc -p <crate> --no-deps   # build rustdoc HTML for a dependency → target/doc/<crate>/
```

## Looking up crate APIs

- **Signatures/types (fastest):** grep crate source directly in
  `~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/`. Already on disk,
  no build step, exact types the compiler sees.
- **Prose/examples/traits:** `cargo doc -p <crate> --no-deps`, then read
  `target/doc/<crate>/index.html` (replaces `/` in name with `_`, e.g. `walkdir`, `sha2`).
- **std concurrency:** `std::sync::atomic`, `std::sync::Mutex`, `std::thread` are all in std;
  read the source/docs locally via `cargo doc` or the std book.
- Prefer offline source/docs over docs.rs/web; web only when neither has the answer.

## Architecture

### Component Hierarchy

- **`Walker`** — recursive dir traversal (walkdir) → `Vec<FileInfo>`.
- **`SizeGrouper`** — group files by size, drop unique sizes → candidate groups.
- **`Hasher`** — parallel sha256 of candidates → `Vec<DuplicateGroup>`. *Concurrency core.*
- **`Progress`** — atomic counters + stop flag, drives progress output.
- **`Reporter`** — output (text/JSON) + optional reclaim actions (delete/hardlink/symlink).

### Core Types

- `FileInfo { path: PathBuf, size: u64 }`
- `FileHash([u8;32])` — sha256 digest newtype; `Hash`/`Eq` for use as key, hex `Display`/serde.
- `DuplicateGroup { hash: FileHash, size: u64, paths: Vec<PathBuf> }` — len ≥ 2.
- `Action { List, Delete, Hardlink, Symlink }` — what to do with found duplicates.
- `ScanConfig { paths, min_size, follow_symlinks, threads, action, dry_run }`.
- `DupError` — thiserror enum (walk, hash, io, action errors).

### Pipeline Flow

```
path(s)
  → Walker::scan()              → Vec<FileInfo>
  → SizeGrouper::group()        → Vec<(size, Vec<PathBuf>)>   [candidates only]
  → Hasher::hash_parallel()     → Vec<DuplicateGroup>         [threads + Arc/Mutex + atomics]
  → Reporter::report()          → text/JSON + optional reclaim
```

### Directory Layout (target)

```
src/
  lib.rs          — crate root, re-exports
  main.rs         — binary entry, clap CLI
  model.rs        — FileInfo, FileHash, DuplicateGroup, Action, ScanConfig
  error.rs        — DupError (thiserror)
  walker.rs       — Walker (walkdir traversal)
  grouper.rs      — SizeGrouper (size prefilter)
  hasher.rs       — parallel hashing — thread::scope, Arc<Mutex>, atomics
  progress.rs     — atomic counters + stop flag (+ indicatif bar)
  reporter.rs     — output (text/json) + reclaim actions
tests/
  integration_test.rs — end-to-end: tempdir dups → scan → assert
```

## Dependencies

Added per-phase during TDD (start from the `cargo init` scaffold). Planned:

| Crate | Purpose | Phase |
|---|---|---|
| `thiserror` | error type derivation | 1 |
| `serde` + `serde_json` | JSON output | 1 |
| `walkdir` | recursive directory traversal | 2 |
| `sha2` | SHA-256 file hashing | 4 |
| `indicatif` | progress bar | 5 |
| `clap` | CLI argument parsing (derive) | 6 |
| `anyhow` | application-level error convenience in `main` | 6 |
| `tempfile` (dev) | test fixtures | 2 |

> `std::thread`, `std::sync` (`Arc`, `Mutex`, `Condvar`), `std::sync::atomic` come from std — no crate needed. **No `rayon`** (see Teaching Guidelines).

## Key Domain Concepts

- **Content-based, not name-based:** two files are duplicates iff their bytes are identical, regardless of name/location. Hashing proves this.
- **Three-stage filter:** size (free) → partial hash (cheap) → full hash (expensive). Prefiltering by size alone drops ~90%+ of files before any hashing. MVP does stage 1 + 3.
- **Embarrassingly parallel:** each file's hash is independent → ideal for a worker pool. The only shared state is the result collector + progress counters.
- **Destructive actions need safety:** `delete`/`hardlink`/`symlink` mutate the filesystem. Default to `--dry-run`, require explicit `--action` + confirmation. Never delete the "first" copy of a group without keeping ≥1.
- **Empty files:** size-0 files all "match" each other. Decide: skip entirely (default) or report as a special group. Document the choice.
- **Symlinks:** loops (`a → b → a`) must not infinite-loop the walker; broken symlinks skipped, not fatal.

## Development Approach

- **TDD:** write failing test first, then implement. All tests must pass before marking work done.
- **Concurrency in context:** introduce each primitive (`Arc`, `Mutex`, `AtomicU64`, `thread::scope`) exactly when the phase needs it — never all at once.
- **Sequential first, then parallel:** Phase 4 may start single-threaded (correctness), then add `thread::scope` + shared state. Measure the speedup.
- **Think Before Acting:**
  - State assumptions before implementing
  - Ask questions if anything is unclear
  - Do not guess silently
  - Prefer correctness over speed
- **Simplicity First:**
  - Use the simplest solution that works
  - Avoid unnecessary abstractions
  - Don't over-engineer
  - Minimize code changes outside the target area
- **Goal-Oriented Execution:**
  - Define what "done" means before starting
  - Work toward the outcome, not the steps
  - If the goal changes, stop and re-evaluate
  - Prefer end-to-end working solutions over partial improvements
- **Before You Act / Explain Plan:**
  - Briefly describe what you are about to do
  - List steps before execution
  - Wait for confirmation only if needed (otherwise proceed)
  - Keep planning short and concrete

## Progress Tracker

### Phase 1: Model + Error types — thiserror, serde, newtypes
- [ ] Add deps: `thiserror`, `serde`, `serde_json`
- [ ] Create `model.rs`: `FileInfo`, `FileHash` newtype (Hash/Eq + hex Display/serde), `DuplicateGroup`, `Action`, `ScanConfig`
- [ ] Create `error.rs`: `DupError` (thiserror) — variants for walk, hash, io, action
- [ ] Create `lib.rs` with `pub mod` declarations
- [ ] Test: `FileHash` serde round-trip (hex)
- [ ] Test: `FileHash` equality / use as HashMap key

### Phase 2: Directory walker — walkdir, filesystem
- [ ] Add dep: `walkdir` (+ `tempfile` dev)
- [ ] Create `walker.rs`, add `pub mod walker;`
- [ ] `Walker::scan(&ScanConfig) -> Result<Vec<FileInfo>, DupError>`
- [ ] Recursive walk, follow-symlinks flag, regular-files-only
- [ ] Apply `min_size` filter
- [ ] Test: tempdir with nested files → correct `FileInfo` list
- [ ] Test: symlink loop does not infinite-loop
- [ ] Test: permission-denied file skipped, not fatal

### Phase 3: Size grouping — cheap prefilter
- [ ] Create `grouper.rs`, add `pub mod grouper;`
- [ ] `SizeGrouper::group(Vec<FileInfo>) -> Vec<(u64, Vec<PathBuf>)>`
- [ ] Drop sizes occurring once; return only candidate groups
- [ ] Test: mixed sizes → only ambiguous sizes returned
- [ ] Test: all-unique sizes → empty result

### Phase 4: Parallel hashing — thread::scope, Arc, Mutex, Send/Sync
- [ ] Add dep: `sha2`
- [ ] Create `hasher.rs`, add `pub mod hasher;`
- [ ] (Optional) single-threaded `hash_sequential()` first for correctness baseline
- [ ] `hash_parallel(groups, &ScanConfig) -> Vec<DuplicateGroup>`
- [ ] Spawn N workers via `thread::scope`
- [ ] Shared job queue `Arc<Mutex<VecDeque<PathBuf>>>` + shared collector `Arc<Mutex<HashMap<FileHash, Vec<PathBuf>>>>`
- [ ] sha256 each file; insert into collector
- [ ] Test: two identical files → one `DuplicateGroup` with both paths
- [ ] Test: distinct files → separate groups
- [ ] Test: parallel result equals sequential result (property test)
- [ ] _Stretch:_ refactor collector to `mpsc` channel; compare tradeoffs
- [ ] _Stretch:_ hand-rolled `Mutex<VecDeque> + Condvar` job queue

### Phase 5: Progress + graceful stop — atomics, signals
- [ ] Add dep: `indicatif` (optional bar)
- [ ] Create `progress.rs`, add `pub mod progress;`
- [ ] `Arc<AtomicU64>` bytes-hashed + files-hashed counters (updated in workers)
- [ ] Progress output polling atomics (plain loop or `indicatif`)
- [ ] `Arc<AtomicBool>` stop flag + Ctrl-C handler
- [ ] Workers check stop flag, drain early
- [ ] Test: counters reflect work done
- [ ] Test: stop flag halts further hashing

### Phase 6: CLI + reporter — clap, output formats
- [ ] Add deps: `clap`, `anyhow`
- [ ] clap derive `Cli` in `main.rs`: paths, `--min-size`, `--threads`, `--format text|json`, `--action`, `--dry-run`
- [ ] Create `reporter.rs`, add `pub mod reporter;`
- [ ] `Reporter::report(&[DuplicateGroup], &ScanConfig)` → text + JSON
- [ ] Wasted-bytes calc + summary (total groups, total wasted)
- [ ] Wire full pipeline in `main.rs`
- [ ] Test: `cargo run -- <tempdir>` prints duplicate groups
- [ ] Test: `--format json` parses back via serde

### Phase 7: Reclaim actions + robustness — filesystem mutation, safety
- [ ] Implement `Action::Delete` / `Hardlink` / `Symlink` in reporter/actions
- [ ] `--dry-run` default ON; confirmation prompt for destructive actions
- [ ] Keep ≥1 copy per group (never delete all)
- [ ] Symlink-loop protection in walker; per-file errors skipped + logged
- [ ] Empty-file policy (skip or special group) — documented
- [ ] Integration test: tempdir dups → scan → `--action hardlink --dry-run` → no mutation; real run → hardlinked

### Phase 8: CDC bonus — *see PRD.md when MVP done*
- [ ] Read "Bonus: Block-Level Dedup (Phase 8)" in `PRD.md`
- [ ] Expand this CLAUDE.md with CDC phases (FastCDC, rolling hash, CAS store)
- [ ] *Do not start until Phases 1-7 complete and tests green.*

## Current Status

**Phase 1: Not started.** Project scaffolded via `cargo init --vcs none` (binary crate, Hello World `main.rs`, minimal `Cargo.toml`). `PRD.md` + `CLAUDE.md` written. Awaiting Phase 1.
