---
phase: 5
status: complete
completed: 2026-10-23
---

# Phase 5: One ingest pass - Summary

The tree pass, the backfill and `terminus ingest <path>` now run through one driver (`driver::drive`) that takes the lock, runs recovery and writes one `runs` row through one INSERT, with recovery's lines and observe's "waiting to be observed" line under `notes` instead of `error`.

## What shipped

- One pass driver with a `Walk` trait and one per-file handler - `crates/terminus-core/src/ingest/driver.rs`
- One `runs` INSERT and one last-run reader (`runs::insert`, `runs::Run`, `runs::last`) - `crates/terminus-core/src/ingest/runs.rs`
- A per-file result with only committed and up-to-date arms (`ingest::Ingested`) - `crates/terminus-core/src/ingest/mod.rs`
- The tree pass, the backfill and the named file as walks (`pass::Tree`, `backfill::Backfill`, `ingest::Named`) - `crates/terminus-core/src/ingest/pass.rs`, `backfill.rs`, `mod.rs`
- A named file writes a row whether it commits, is up to date or fails, outside the file's own transaction - `crates/terminus-core/src/ingest/mod.rs`
- Recovery's lines under `notes` - `crates/terminus-core/src/ingest/runs.rs`, `crates/terminus-core/src/recover.rs`
- Observe's waiting line split from its failures (`Observed::errors`, `Observed::notes`) - `crates/terminus-core/src/observe/mod.rs`
- `status` and `doctor` read the last run through `runs::last` - `crates/terminus/src/cmd/status.rs`, `crates/terminus/src/cmd/doctor.rs`
- Process-level proofs for the named file's row, a recovery-only pass, a skipped file, the waiting line and a backfill's row - `crates/terminus/tests/status.rs`, `doctor.rs`, `backfill.rs`
- Docs for the named file's row and the one driver - `docs/commands.md`, `docs/json-shapes.md`, `docs/design/0003-ingest.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 10e941e | Give the per-file ingest a result with no lock or exclusion arm |
| 1 | 2 | da826b2 | Run the tree pass through one driver that writes the one row |
| 1 | 3 | 4e217cb | Run the backfill through the driver |
| 1 | 4 | 708df13 | Run a named file through the driver, its row outside the file's transaction |
| 1 | 5 | 0bba6af | Write recovery's lines under notes, not error |
| 1 | 6 | 8677b76 | Split observe's waiting line from its failures |
| 1 | 7 | 08d259e | Read the last run through one core function |
| 1 | 8 | 6119cca | Rewrite the comments that still name the old row writers |
| 2 | 1 | de74bc5 | Prove a named file's run leaves one row whatever it did |
| 2 | 2 | 12cfc8d | Prove a recovery-only pass is no error and a skipped file is |
| 2 | 3 | bb4ee5a | Prove sessions left waiting to be observed are a note |
| 2 | 4 | f7698eb | Prove a backfill leaves the row a tree pass does |
| 2 | 5 | 16b07bd | Describe the named file's row and the one driver |

## Deviations

- [deviation] Plan 1 task 1's Verify, `git grep -nE 'Outcome::(LockHeld|Excluded)'` over `pass.rs` and `backfill.rs`, also matches `PassOutcome::LockHeld`, which the plan keeps. The executor checked with a word boundary (`\bOutcome::`) instead, which prints nothing and is the criterion's intent (10e941e).

## Open items

- `ingest/driver.rs`'s recovery comment dropped the second reason the old `pass::locked` and `ingest::run_with` comments gave: recovery also brings an older store's derived tables forward (`reindex::up_to_date` inside `recover::recover`), so a walk never appends turn rows in two shapes. A comment-only restore. (plan 1)
- `rustfmt --check` on `ingest/mod.rs` or `observe/mod.rs` also walks into `observe/egress.rs`, which already has formatting diffs this phase did not touch. Left alone, since `cargo fmt` over the workspace is out of scope. (plan 1)
- `feedback::Drained::discarded`'s doc says its lines go into `runs.error` "with the pass's other notes", which uses "notes" loosely now that `runs.notes` is its own list. (plan 1)
- The `terminus status` paragraph in `docs/commands.md` still says it reports "the last ingest pass", while doctor's list now says "the last ingest run", and a named-file run is now what status may show. (plan 2)
- `docs/design/0003-ingest.md`'s backfill passage keeps "Only the `runs` row each writes differs", which reads as the rows' values and stays true beside the new "the row a tree pass writes". (plan 2)

## Goal check

The commits plausibly deliver the goal. Only `ingest/driver.rs` takes the lock or runs recovery inside `crates/terminus-core/src/ingest/` (`driver.rs:71`, `driver.rs:81`), and the tree pass, the backfill and the named file each call `driver::drive` (`pass.rs:147`, `backfill.rs:112`, `mod.rs:136`). The one `INSERT INTO runs` in `src` is `runs.rs:98`, and `git grep -nE '\bRunRow\b|fn record_run|fn record_pass' crates/` and `git grep -n 'FROM runs' crates/terminus/src` both print nothing. `status` and `doctor` read through `runs::last` (`status.rs:57`, `doctor.rs:832`). The recovery-only and waiting-line cases are proved at the process boundary by 12cfc8d and bb4ee5a, each with a mutation run that failed when the line was moved back under `error`, per plan 2's report. Both executors report `cargo test --workspace --all-features` exit 0 with 0 failed, and `git diff --quiet f1f55af -- crates/terminus/tests/hook.rs` exits 0. Nothing in the goal looks missing; the open items are comment and doc wording, not behaviour.
