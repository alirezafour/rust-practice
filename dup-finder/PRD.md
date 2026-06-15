# PRD — Duplicate File Finder (dup-finder)

## Problem

Disk fills with duplicate files — copies of photos, downloaded zips, build artifacts, backups of backups. Finding them manually is impossible at scale. Naive approaches either miss duplicates (compare names) or are too slow (hash every file sequentially, single-threaded). A good dedup tool must be **fast** (parallel) and **correct** (content-based, not name-based), and ideally **space-aware** (report wasted bytes, offer to reclaim them).

This project is also a **focused vehicle for learning Rust concurrency**: OS threads, shared state (`Arc`/`Mutex`), lock-free counters (`AtomicU64`/`AtomicBool`), and message-passing (`mpsc`). The domain is chosen because hashing files is *embarrassingly parallel* — every concurrency primitive has a natural, non-forced place.

## Goal

Build a Rust binary that:

1. **Walks** one or more directory trees recursively
2. **Prefilters** by file size (cheap) to skip ~90%+ of files without hashing
3. **Hashes** only same-size candidates **in parallel** across N worker threads
4. **Groups** identical files by content hash → duplicate sets
5. **Reports** duplicates (wasted bytes, paths) as text or JSON
6. **Optionally reclaims space** — delete, hardlink, or symlink duplicates (with confirmation)
7. **Bonus (Phase 8):** block-level dedup via content-defined chunking (CDC), restic/borg-style — finds duplicates *within* files, not just whole files.

## Core Insight: Three-Stage Filter

Hashing is expensive (CPU + I/O). Don't hash everything. Filter cheaply first:

| Stage | Cost | What it catches |
|---|---|---|
| 1. Size | ~free (stat only) | Any file with a unique size → not a duplicate. Drops ~90%+ |
| 2. Partial hash | low (first+last N KB) | Same-size files differing early/late |
| 3. Full hash (sha256) | high (full read) | Only true candidates remaining |

MVP implements stages 1 + 3. Stage 2 (partial hash) is a nice-to-have optimization. CDC (Phase 8) replaces whole-file hashing with block-level chunking.

## Architecture

```
path(s)
  │
  ▼
┌──────────────────┐
│  Walker          │  walkdir, recursive → Vec<FileInfo{path,size}>
└────────┬─────────┘
         │ Vec<FileInfo>
         ▼
┌──────────────────┐
│  SizeGrouper     │  HashMap<size, Vec<Path>>; drop unique sizes
└────────┬─────────┘
         │ candidate groups (size occurs ≥2×)
         ▼
┌──────────────────┐
│  Parallel Hasher │  thread::scope, N workers
│                  │  Arc<Mutex<HashMap<Hash, Vec<Path>>>>  (collector)
│                  │  Arc<AtomicU64> (bytes+files done)     (progress)
│                  │  Arc<AtomicBool> (stop flag)           (Ctrl-C)
└────────┬─────────┘
         │ DuplicateGroup { hash, size, paths }
         ▼
┌──────────────────┐
│  Reporter        │  print text / JSON; --action list|delete|hardlink|symlink
└──────────────────┘
```

### Core Data Model

```rust
struct FileInfo {
    path: PathBuf,
    size: u64,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct FileHash([u8; 32]);  // sha256 digest; hex Display

struct DuplicateGroup {
    hash: FileHash,
    size: u64,
    paths: Vec<PathBuf>,  // len ≥ 2
}

enum Action { List, Delete, Hardlink, Symlink }

struct ScanConfig {
    paths: Vec<PathBuf>,
    min_size: u64,           // skip files smaller than this (default 1)
    follow_symlinks: bool,
    threads: usize,          // worker count (default = num_cpus)
    action: Action,
    dry_run: bool,           // default true for destructive actions
}
```

### Pipeline

1. `Walker::scan(&config)` → `Vec<FileInfo>`
2. `SizeGrouper::group(files)` → `Vec<(u64, Vec<PathBuf>)>` (size-ambiguous groups only)
3. `Hasher::hash_parallel(groups, &config)` → `Vec<DuplicateGroup>` (concurrency core)
4. `Reporter::report(groups, &config)` → print/JSON + optional reclaim

## Concurrency Concepts (the learning focus)

This is the heart of the project. Every primitive below maps to a real need, not a contrived exercise:

| Primitive | Where it's used | Why |
|---|---|---|
| `std::thread::scope` | Worker pool in hasher | Spawn threads that borrow local data; no `'static` needed |
| `Arc<T>` | Shared collector, counters, stop flag | Multiple owners across threads |
| `Mutex<HashMap>` | Result collector (`Hash → Vec<Path>`) | One writer at a time; safe aggregation |
| `mpsc` channel *(alt)* | Stream results to a single collector | Message-passing alternative to Mutex — feel the tradeoff |
| `AtomicU64` | Bytes hashed, files hashed counters | Lock-free progress; no mutex contention |
| `AtomicBool` | Stop flag | Ctrl-C signal visible to all workers |
| `Ordering::Relaxed/AcqRel` | Atomic ops | Learn when memory ordering matters (mostly `Relaxed` here) |
| `Mutex<VecDeque> + Condvar` *(stretch)* | Hand-rolled bounded job queue | Max mutex/condvar learning; replaces `thread::scope` partitioning |

**MVP path:** `thread::scope` + `Arc<Mutex<HashMap>>` collector + atomics for progress. The mpsc and Condvar-job-queue variants are documented as alternative / stretch within Phase 4.

## Requirements

### Must Have (MVP — Phases 1-7)

- [ ] Recursive directory walk (walkdir), follow-symlinks flag
- [ ] Size-based prefilter (drop unique sizes)
- [ ] Parallel sha256 hashing with worker pool (`thread::scope` + `Arc<Mutex>`)
- [ ] Progress reporting via atomics (bytes + files)
- [ ] Graceful Ctrl-C stop via `AtomicBool`
- [ ] Duplicate report: text + JSON output
- [ ] CLI (clap): paths, `--min-size`, `--threads`, `--format`, `--action`, `--dry-run`
- [ ] Reclaim actions: delete / hardlink / symlink (with `--dry-run` default + confirmation)
- [ ] Robustness: symlink-loop protection, permission errors skipped, empty files handled
- [ ] Error handling via thiserror (`DupError`)
- [ ] Unit + integration tests (TDD, temp-dir fixtures)

### Nice to Have

- [ ] Partial-hash prefilter (stage 2) — hash first+last 64KB before full hash
- [ ] Exclude patterns (gitignore-style) via `ignore` crate
- [ ] Progress bar (indicatif)
- [ ] `--min-bytes-wasted` filter (only show groups wasting > N bytes)
- [ ] Resume / cache hashes to disk (content-addressable cache)
- [ ] Benchmarks (criterion): parallel vs sequential hashing speedup

### Bonus — Phase 8: Block-level dedup (CDC)

- [ ] Content-defined chunking (FastCDC algorithm + Gear rolling hash)
- [ ] Per-file: split into variable-size chunks at content boundaries
- [ ] Content-addressable chunk store (`HashMap<ChunkHash, ChunkMeta>`)
- [ ] Dedup ratio report (logical bytes vs unique bytes)
- [ ] Detect duplicates *within* a single file and across files
- *Detailed in the section below; not started until MVP complete.*

## Tech Stack

| Concern | Crate |
|---|---|
| Directory traversal | `walkdir` (sequential; we parallelize hashing ourselves) |
| Hashing | `sha2` (SHA-256) |
| CLI | `clap` (derive) |
| Error handling | `thiserror` + `anyhow` |
| Serialization (JSON output) | `serde` + `serde_json` |
| Progress bar (nice-to-have) | `indicatif` |
| Logging | `tracing` + `tracing-subscriber` |
| Testing | built-in `#[test]` + `tempfile` for fixtures |
| Concurrency | **`std::thread`, `std::sync`, `std::sync::atomic`** — no `rayon` (defeats the learning goal) |

> **No `rayon`.** The whole point is to build the parallelism by hand. Add rayon only *after* understanding the manual version, as a refactor comparison.

## Bonus: Block-Level Dedup (Phase 8)

**Status: not started. Detailed only when MVP (Phases 1-7) is complete.**

Whole-file hashing misses duplicates *inside* files — e.g. two VM images sharing 80% of blocks, or a backup containing a large file that changed by one byte. CDC splits files at content-defined boundaries (not fixed offsets), so identical regions produce identical chunks regardless of position.

- **FastCDC** — fast content-defined chunking algorithm (normalized chunking)
- **Gear / rolling hash** — slides a window, emits a boundary when the hash hits a mask
- **Chunk store** — content-addressable; identical chunks stored once
- **Dedup ratio** — `logical_bytes / unique_bytes`; the headline number restic/borg report

Scope: ~13-18h additional. Mostly hashing-algorithm depth, marginal new concurrency. Do it *after* MVP, when the hash pipeline already exists to reuse.
