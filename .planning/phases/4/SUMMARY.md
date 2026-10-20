---
phase: 4
status: complete
completed: 2026-10-20
---

# Phase 4: Steady-state ingest pass - Summary

A steady-state pass now reads no block rows for a file that has not grown, reads every file's row in one query at pass start, parses a grown tail once, compiles `derive`'s statements once per connection and opens the store for writing once, cutting the median pass on a copy of the live store from 318 ms to 164.5 ms (#85).

## What shipped

- Tests that pin what an unchanged file still gets: the held-key, missing `session_meta`, checksum, shrunk-file and moved-file outcomes, on both the walk and `ingest::run_with` - `crates/terminus-core/tests/pass.rs`
- A `testkit`-only block-read log, `archive::block_reads` - `crates/terminus-core/src/store/archive.rs`
- The length gate: `Existing::read` stats the file and fetches the next turn number, the hashes and the tail row only when the file is longer than its watermark or the session needs its whole archive - `crates/terminus-core/src/ingest/mod.rs`
- The resume offset is found by a search for the tail's last newline, leaving one `parse::scan_from` call per grown file - `crates/terminus-core/src/ingest/mod.rs`
- One pass-start query, `ROWS` (`sessions` LEFT JOIN `session_meta` LEFT JOIN `watermarks`), that `claim` answers from in the walk and in backfill - `crates/terminus-core/src/ingest/mod.rs`, `pass.rs`, `backfill.rs`
- The snapshot gate answered on the pass's own store inside the lock, with a writable open after release only when a snapshot is due - `crates/terminus-core/src/ingest/pass.rs`
- `prepare_cached` for `derive`'s nine statements - `crates/terminus-core/src/derive.rs`
- The before-and-after measurement - `.planning/phases/4/MEASUREMENT.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 34071a6 | Pin an unchanged file's five outcomes in tests, passing at 57c036e |
| 1 | 2 | 0fb7422 | Log every block-row read under `testkit` |
| 1 | 3 | 8c50128 | Skip an unchanged file's block rows and turn count |
| 1 | 4 | 54ab34f | Find a grown file's resume offset without a parse |
| 2 | 1 | 19e062b | Read every file's row once at pass start |
| 2 | 2 | 64c8852 | Ask whether a snapshot is due on the pass's own store |
| 2 | 3 | fb88658 | Compile `derive`'s statements once per connection |
| 2 | 4 | d727ff8 | Record steady-state passes before and after on a copy of the live store |

## Deviations

- [deviation] PLAN-1 Task 4's Verify expected `grep -c 'scan_from'` to print 1, and the first run printed 2. The second hit was the new comment naming `scan_from`, not a call. The comment was reworded to name the rule through `parse`, and the committed file counts 1 (54ab34f).

## Open items

- `recover.rs`'s `hashes` and `block` queries (the doubled-archive repair in `rekey`) read `blocks` directly rather than through `store::archive`, so the block-read log does not see them. They run only when recovery finds a doubled key, never in a steady-state pass, and the plan named only `archive.rs`'s readers.
- `/home/john/terminus-measure-4` (the 4.5 GB copy, including one 1.6 GB snapshot, plus the baseline source and build) is left in place for UAT to re-run AC4. Removing it afterwards is John's call.
- The PLAN-1 risk review raised a race that predates this phase: an unflagged file truncated and rewritten past its watermark between the length check and the tail read is appended, and its watermark is committed from bytes the file may no longer hold. It was refuted as a regression because `read_tail` and `open_at_least` are unchanged and the length-only test is D-13's design. No issue tracks it yet.
- The PLAN-2 risk review raised duplicate snapshots and an out-of-order `last_snapshot_at` stamp from two adjacent passes. That was already possible at 54ab34f and is #89, which D-15 keeps open.

## Goal check

The commits plausibly deliver the goal. Every item the goal names is addressed. An unchanged file no longer reads its block hashes and tail: 8c50128 gates `archive::append_state` behind the length check, and its test fails first with 2 reads and passes with none. The five per-file lookups become one pass-start query in 19e062b, with the next-turn number still read per grown file, as D-11 decided. The triple parse is down to the stored scan plus `derive_turn`'s own parse in 54ab34f, which D-13 keeps. Statements are compiled once per connection in fb88658 (`grep -c prepare_cached` prints 9 and no `tx.execute` is left). With no snapshot due, `strace` on the copy shows 1 writable open of `terminus.db` against 2 at baseline (64c8852, MEASUREMENT.md "AC4 on the copy"). MEASUREMENT.md records the median wall clock falling from 318 ms to 164.5 ms, `runs.duration_ms` from 310 to 157 and syscalls from 372,448 to about 92,756. All of those were measured with a warm page cache, which MEASUREMENT.md states understates what a cold pass saves. `cargo test --workspace --all-features` passed after each plan (1014 passed, 0 failed), and `git diff 57c036e` under the test directories adds lines only. Nothing in the goal looks missing. The one fact the measurement does not cover is a cold-cache pass, because dropping caches needs root (D-06).
