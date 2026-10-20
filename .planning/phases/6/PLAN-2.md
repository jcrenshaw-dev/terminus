---
phase: 6
plan: 2
requirements:
  - "#90"
files:
  - crates/terminus-core/src/store/open.rs
  - crates/terminus-core/src/derive.rs
  - crates/terminus-core/src/store/archive.rs
  - crates/terminus-core/src/store/mod.rs
  - crates/terminus/src/cmd/read.rs
---

# Phase 6: Simplify the 1.0.0 changes - Plan 2 (store)

## Goal

A simplify pass over the code the 1.0.0 fixes change, before the README and the docs sweep describe it (#90). It removes code that reinvents something the codebase already has, code that can be simpler, and wasted work, and it changes no behavior. This plan applies SIMPLIFY.md findings 6, 7 and 9, which all sit in the store.

## Must be true when done

- `cargo test --workspace --all-features --no-fail-fast` reports 0 failed, and `git diff` from the commit this plan started from adds, removes or changes no line containing `assert`.
- `Store::open` sets the statement cache's capacity itself, and `derive_turn`'s doc comment no longer carries hand counts of statements.
- `block_reads` has no `reset` and no `record_key`, and `append_state` records each block read from one place per query.
- A read command on a database file with no tables still answers "no terminus store at ...", exit 0, with no table created. It now gets that answer from `Store::open_read_only`'s first statement, without a second connection, and `holds_no_tables` is gone.
- `git diff --name-only` from the commit this plan started from lists only the five files in this plan's frontmatter.

## Context

- `.planning/phases/6/SIMPLIFY.md` stands in for CONTEXT.md. Each task applies one finding under `## Apply` exactly as written. Nothing under `## Skip` is planned.
- Binding on every task: no behavior change, every existing test passes, and no assertion changes.
- Earlier decisions kept: phase 2 D-03 (a database with no tables is no store, while one with tables but no `meta` is somebody else's file) and D-05 (the no-store answer), both in `.planning/phases/2/CONTEXT.md`. Phase 4 D-14 (`prepare_cached` in `derive`) is in `.planning/phases/4/CONTEXT.md`.
- Out of scope: `crates/terminus-core/src/error.rs`, `inject/mod.rs` and `cmd/install/mod.rs`. Task 3 depends on how those files handle the error, and none of them is edited.

## Tasks

### Task 1: Set the statement cache's capacity in `Store::open` (finding 6)

- **Files:** crates/terminus-core/src/store/open.rs (`Store::open`, `BUSY_TIMEOUT_MS`), crates/terminus-core/src/derive.rs (`derive_turn`'s doc comment)
- **Action:** In `Store::open`, right after the writable `Connection::open` and next to its `busy_timeout`, call rusqlite 0.40.2's `Connection::set_prepared_statement_cache_capacity` (at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rusqlite-0.40.2/src/cache.rs:48`). Pass a named constant declared next to `BUSY_TIMEOUT_MS`, set to 32. Its doc comment gives the reason. `derive_turn` and the archive reads go through `prepare_cached`, and a loop compiles each statement once only while all of the loop's statements fit in the cache. 32 is twice rusqlite's default of 16, so the statements of one ingest or `reindex` loop fit with room to spare, and nobody has to count them to know it. The comment states no statement counts. Leave `Store::open_read_only` alone, because the finding names `Store::open` only. In `derive_turn`'s doc comment, delete "That holds only while a loop's statements fit the cache's default 16: an ingest walk uses 13 and a `reindex` loop 11." In its place, say this holds while a loop's statements fit the capacity `Store::open` sets.
- **Verify:** `grep -rn "set_prepared_statement_cache_capacity" crates` prints exactly one line, in `Store::open` in `store/open.rs`. `grep -n "default 16\|uses 13\|loop 11" crates/terminus-core/src/derive.rs` prints nothing. `cargo test -p terminus-core --all-features --test store_open --test reindex --test ingest --test derive --test pass` reports 0 failed.

### Task 2: Trim the `block_reads` log (finding 7)

- **Files:** crates/terminus-core/src/store/archive.rs (`block_reads`, `append_state`, `rows`)
- **Action:** Delete `block_reads::reset`, both `block_reads::record_key` functions (the `testkit` one and the no-op stub), and the `record_key` call at the top of `archive::rows`. Nothing counts `rows`' reads. Its one caller is `verify` (`crates/terminus-core/src/verify.rs:213`), and the log is read only by `reads_of` in `crates/terminus-core/tests/pass.rs`, which runs no verify. In `append_state`, call `block_reads::record(conn, session_no)` once, after `tail_no` is computed and before `if every_row`, and remove the two calls that open each branch now. Move the remaining `if tail_no.is_some()` record into the `Some(n)` arm of the `match tail_no` below it, ahead of the tail query. Each path then logs exactly what it logs now: one entry for the hash or every-row query, plus one more when a tail row is read. Fix the doc comments that name the deleted function. The module doc's "never call [`block_reads::reset`] mid-binary" and `entries`' "since the last [`reset`]" must say instead that the log is never cleared.
- **Verify:** `grep -n "record_key\|reset" crates/terminus-core/src/store/archive.rs` prints nothing. `grep -c "block_reads::record(" crates/terminus-core/src/store/archive.rs` prints 4. `cargo build -p terminus-core` (testkit off) succeeds. `cargo test -p terminus-core --all-features --test pass --test recall --test backfill --test verify` reports 0 failed. `cargo doc -p terminus-core --all-features --no-deps` prints no unresolved-link warning for `archive.rs`.

### Task 3: "No store yet" is answered by the read-only open itself (finding 9)

- **Files:** crates/terminus-core/src/store/open.rs (`Store::open_read_only`, `holds_no_tables`), crates/terminus-core/src/store/mod.rs (the `pub use open::{...}` list), crates/terminus/src/cmd/read.rs (`open_in`)
- **Action:** In `Store::open_read_only`, the first statement counts tables (`SELECT count(*) FROM sqlite_master WHERE type = 'table'`) instead of every `sqlite_master` row. It keeps its `StoreUnreadable` mapping, because this statement is still where a lazy open fails. When the count is 0, return `Error::StoreNotFound { path }` before `read_versions` runs. Move the reason that sits in `holds_no_tables`' doc comment and in `open_in`'s match-arm comment to that line: `Store::open` stamps `journal_mode=wal` before `initialize` commits, so a non-empty file with no tables is a store another process is still creating, which counts as no store yet. A file with tables but no `meta` is still `NotAStore` (phase 2 D-03). Extend the doc's "A missing data directory or database file is [`Error::StoreNotFound`]" to cover a database with no tables. Delete `holds_no_tables` and remove its name from `store/mod.rs`'s re-export, which puts that line back to its text on `main`. In `read::open_in`, delete the `Err(Error::NotAStore { path, .. }) if ... holds_no_tables ...` arm and its comment. Every caller gets the same answer as before. `read::open_in` already maps `StoreNotFound` to `no_store(&path)`, and the `path` is the same `data_dir.join(DB_FILE_NAME)` that `read_versions` was given. `inject::open` drops the error with `.ok()?`. `start_backfill` in `cmd/install/mod.rs` matches only `ArchiveFormatTooOld`. Edit neither of those files, nor `error.rs`.
- **Verify:** `grep -rn "holds_no_tables" crates` prints only the unit test name `a_database_with_no_tables_is_a_reason_and_still_holds_no_tables` in `cmd/read.rs`. `git diff main -- crates/terminus-core/src/store/mod.rs` prints nothing. `cargo test -p terminus --all-features --lib cmd::read::tests` reports 0 failed, and its passes include `a_database_with_no_tables_is_a_reason_and_still_holds_no_tables`, `a_database_with_a_foreign_table_is_operational` and `a_meta_table_with_no_versions_is_operational`. `cargo test -p terminus --all-features --test status --test cli --test doctor --test mcp` and `cargo test -p terminus-core --all-features --test store_open --test inject_brief` report 0 failed. `status_on_a_database_with_no_tables_says_no_store_and_adds_no_table` is among the passes. Then, for the plan as a whole: `cargo test --workspace --all-features --no-fail-fast` reports 0 failed. `git diff --name-only <the commit this plan started from>` lists only this plan's five frontmatter files. `git diff <that commit> -- crates | grep -E '^[-+][^-+]' | grep assert` prints nothing. `cargo fmt --all --check` names none of the five files.

## Notes

- The cache size of 32 in Task 1 is the planner's choice. Finding 6 says to set the capacity and does not give a number. 32 is twice rusqlite's default, which lets the counts come out of the comment honestly. Setting it to 16 would change nothing at run time, but then the comment's claim would still rest on the counts. A cache size is not something a test or a user can see, so either value keeps criterion 2. If John wants a different number, the constant is the only thing that changes.
- Phase 4 D-14 said to set the cache size only if a loop needed more than 16, and none did. Finding 6, which John adjudicated after phase 4, sets it anyway. Phase 2 D-01 said `Store::open_read_only` would not change. Finding 9 changes it, and the answer each caller gets stays the same.
- Criterion 3: `Store::open` (Task 1) and `open_read_only`'s count statement (Task 3) sit in `open.rs`, a file the traders diff changes, but the diff did not change those lines. Both findings name those lines, and both remove something the diff added: the cache comment's counts, and the `holds_no_tables` second connection.
- Prior art: the read-only path keeps the rule that a missing store is an empty result with a reason (recalled D-10, `v0.1.0/phases/3/CONTEXT.md`, phase 3), and `doctor` still reaches it through `cmd/read.rs` (recalled D-12, `v0.1.0/phases/4/CONTEXT.md`, phase 4).
- SIMPLIFY.md's line numbers were checked against `traders` at 4c078c8 and all match: `derive.rs:77-79`, `archive.rs:425`, `:436`, `:455`, `:278`, `:300`, `:308-310`, `verify.rs:213`, `open.rs:269`, `:424-436`, `read.rs:112-116`, `inject/mod.rs:130`, `cmd/install/mod.rs:351`.
- Criterion 4 (`Fixes #90` in the landing PR body) is carried as a truth in PLAN-1.
