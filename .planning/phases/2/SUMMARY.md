---
phase: 2
status: complete
completed: 2026-10-20
---

# Phase 2: Read-only status and verify - Summary

`terminus status` and `terminus verify` open the store through `cmd/read.rs` instead of the writable `Store::open`, so a missing store is reported and nothing is created, an older store is reported and keeps its schema, and a database file with no tables at all reads as no store yet (#77).

## What shipped

- `store::holds_no_tables`, a read-only check that a database file holds no tables, asked by `read::open_in` only after a `NotAStore` - `crates/terminus-core/src/store/open.rs`, `crates/terminus/src/cmd/read.rs`
- `status` opens through `read::open()` and answers no store with a zero-and-null document - `crates/terminus/src/cmd/status.rs`
- `verify` opens through `read::open_in` with `Config::default()`, so a malformed `terminus.toml` cannot stop the integrity check - `crates/terminus/src/cmd/verify.rs`
- Regression tests for no store, a no-table database, an older store and a malformed config, plus the existing tests D-09 named - `crates/terminus/tests/status.rs`, `crates/terminus/tests/cli.rs`, `crates/terminus/tests/tighten.rs`, `crates/terminus/tests/uninstall.rs`
- The docs, comments and CHANGELOG the change made false - `THREAT-MODEL.md`, `docs/commands.md`, `docs/json-shapes.md`, `CHANGELOG.md`, and comments in `read.rs`, `open.rs`, `schema.rs`, `stats.rs`, `tests/schema.rs`, `cmd/uninstall.rs`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | ad520cb | Read a database with no tables as no store yet |
| 1 | 2 | 6f0836f | Open status read-only, so no store is an answer and creates nothing |
| 1 | 3 | 06dd2c4 | Open verify read-only with the default config |
| 1 | 4 | 1425f82 | Say status and verify open read-only and create nothing |

## Deviations

None - plans executed as written.

## Open items

- The `holds_no_tables` doc says it "creates, writes and repairs nothing", but a read-only open of a WAL file in a writable directory can leave `terminus.db-shm` and an empty `-wal` behind, which D-10 measured and the uninstall test now accounts for (risk_surface review, medium, survived and not fixed; `ADJUDICATION-risk_surface-plan-1.json`).
- John reviews `THREAT-MODEL.md`, `docs/commands.md`, `docs/json-shapes.md` and `CHANGELOG.md` himself; the executor opened them in RustRover.
- #77 closes on the merged PR, which needs `Closes #77` (human-verify).
- macOS `size_bytes` parity is unmeasured; the macos-15 CI runners settle it.
- `cargo fmt --check` reports diffs in files this phase did not touch (`crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `crates/terminus/tests/release_workflows.rs`, possibly others), left alone.

## Goal check

The four commits deliver the goal. Neither command calls `Store::open` any more: `crates/terminus/src/cmd/status.rs:41` opens through `super::read::open()` and `crates/terminus/src/cmd/verify.rs:14` through `read::open_in(&super::data_dir()?, Config::default())`. The store being created is covered by `crates/terminus/src/cmd/read.rs:112-113`, which turns a `NotAStore` into no store only when `holds_no_tables` answers true, so a foreign SQLite file still fails as not a store. The executor's report records that the new no-store, no-table and older-store tests fail against the pre-phase code and pass after, that the before and after binaries print byte-identical `status` and `verify` output on an ingested store (`size_bytes` 204800), and that `cargo test --workspace --all-features` passed with 1001 tests and none failing. What remains is #77's close on the merged PR and the doc comment on `holds_no_tables` overstating what a read-only open leaves on disk.
