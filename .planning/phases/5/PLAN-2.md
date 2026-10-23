---
phase: 5
plan: 2
requirements: [PASS-01, PASS-02]
files:
  - crates/terminus/tests/status.rs
  - crates/terminus/tests/doctor.rs
  - crates/terminus/tests/backfill.rs
  - docs/commands.md
  - docs/json-shapes.md
  - docs/design/0003-ingest.md
---

# Phase 5: One ingest pass - Plan 2 of 2 (the proofs and the docs)

## Goal

One pass driver runs every pass, and a pass that only recovered is not reported as an error (#31, #96). Plan 1 made it so in the code. This plan proves it by running real `terminus` processes against real stores, and rewrites the docs that still describe the old split.

## Must be true when done

- `terminus ingest <path>` writes one `runs` row when the file commits, when it is up to date and when it fails. When it fails, it exits non-zero and the row's `error` names the file.
- On a store whose watermark recovery must lower, with no failing file: after `terminus ingest`, `terminus status` prints no `error` block and shows recovery's line under `notes`, `terminus status --json` has `error: null`, and `terminus doctor` exits 0.
- On a store with one shrunk transcript: after `terminus ingest`, `terminus status` prints that file under `error`, and `terminus doctor` exits 1.
- One `terminus ingest` over 101 newly finished sessions that fails nothing writes the "waiting to be observed" line in `notes`, not `error`, and `terminus doctor` exits 0.
- A tree pass and `terminus backfill`, each run on its own copy of one store with the same damaged watermark, leave last `runs` rows with the same set of non-null columns. Both rows' `notes` hold recovery's line.
- `docs/commands.md`, `docs/json-shapes.md` and `docs/design/0003-ingest.md` no longer say recovery prints under `error`. They describe the named file's row and the one driver.
- `cargo test --workspace --all-features` passes, and the phase's diff leaves `crates/terminus/tests/hook.rs`, and with it `reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`, unchanged.

## Context

- Runs after plan 1, never beside it. These tests prove plan 1's behaviour and fail without it.
- Locked: D-13. A recovery-only pass and a failed file are each produced on a real store, by a watermark pushed past its archive or a shrunk transcript, and then `terminus ingest`, `status` and `doctor` run as processes. A `notes` or `error` value planted with UPDATE proves only how a row is displayed and does not count. The existing planted-row tests stay as rendering tests. D-14 governs the docs.
- Patterns: `crates/terminus-core/tests/recover.rs` (`damage()`) for the watermark push. `crates/terminus/tests/doctor.rs` (`fixture()`, `Fixture::install`, `after_one_ingest_the_store_and_the_last_run_are_reported`) for a machine where every check but the one under test is ok. `crates/terminus/tests/status.rs` (`bench()`, `Bench::write`) for a spawned `terminus` and a one-record transcript.
- Out of scope: any production code (plan 1 owns it); any edit to `crates/terminus/tests/hook.rs`; running `cargo fmt` over the workspace. New tests may cite `#31` and `#96`, never a requirement id or a decision tag.

## Tasks

### Task 1: Prove a named file's row at the process boundary

- **Files:** crates/terminus/tests/status.rs
- **Action:** Add a test beside `status_counts_the_file_a_single_file_ingest_committed`, using `bench()`, `Bench::place`, `Bench::run` and `Bench::conn`. Place `session-basic.jsonl` under a uuid name and run `terminus ingest <path>` three times:
  1. The file commits; the run exits 0.
  2. The file is unchanged and up to date; the run exits 0.
  3. Cut the file to `{}\n` first. The run exits non-zero.

  After each run, `runs` holds exactly one more row than before. The newest row's files seen, committed and failed are 1, 1, 0 after the first run, then 1, 0, 0, then 1, 0, 1. The third row's `error` contains the file's path. A last `terminus status` prints that path under `  error`. The test reads the rows with its own SQL through `Bench::conn`; AC1's grep covers only `crates/terminus/src`.
- **Verify:** `cargo test -p terminus --all-features --test status` passes. The new test asserts the row count rises by exactly one on each run, so it fails on a build where an up-to-date or failed named file writes no row.

### Task 2: Prove a recovery-only pass and a failed-file pass end to end

- **Files:** crates/terminus/tests/doctor.rs
- **Action:** Add two tests that each start from `fixture()` and `Fixture::install`, so every doctor check except `last_run` is ok. Place one transcript the way `after_one_ingest_the_store_and_the_last_run_are_reported` does: a `<uuid>.jsonl` in `projects/-data-code-x`, written from `terminus_core::testkit::fixture_bytes("session-basic.jsonl")`. Then run `terminus ingest`.
  - **Recovery only.** Open the store with `rusqlite` and push that session's watermark past its archive, to its `session_meta.uncompressed_len` plus 4096, as `damage()` in `crates/terminus-core/tests/recover.rs` does. This is real damage recovery must lower, not a planted `notes` or `error` value. Drop the connection and run `terminus ingest` again; it exits 0. Then:
    - `terminus status` prints no `  error` line, and prints a `  notes` block whose line names the session key and ends in "lowered".
    - `terminus status --json` has `data.last_run.error` null and `data.last_run.notes` containing that line.
    - `terminus doctor` exits 0 with `last_run` ok.
  - **Failed file.** After the first ingest, cut the transcript to `{}\n` and run `terminus ingest` again. It exits 0, because a skipped file is not a failed pass. Then `terminus status` prints the transcript's path under `  error`, and `terminus doctor` exits 1 with `last_run` as its only problem.
- **Verify:** `cargo test -p terminus --all-features --test doctor` passes. The recovery test asserts that `status` prints no `  error` and that `doctor` exits 0, so it fails on a build that still files recovery's lines under `error`.

### Task 3: Prove the waiting line is a note at the process boundary

- **Files:** crates/terminus/tests/doctor.rs
- **Action:** Add a test that starts from `fixture()` and `Fixture::install`. It writes `terminus_core::observe::MAX_PER_PASS + 1` one-record transcripts into one project directory, each a `<uuid>.jsonl` with its own uuid and `sessionId`. Use the record shape `Bench::write` in `crates/terminus/tests/status.rs` writes (`type`, `uuid`, `timestamp`, `sessionId`, `cwd`, a user message). Give every record a fixed timestamp months in the past, such as August 2026, so the six-hour idle rule finalizes every session on the first pass. Run `terminus ingest` once. Then:
  - `terminus status --json` has `data.last_run.error` null, `files_failed` 0, and `data.last_run.notes` containing "1 more finalized session(s) are waiting to be observed".
  - `terminus doctor` exits 0 with `last_run` ok.
- **Verify:** `cargo test -p terminus --all-features --test doctor` passes. The test asserts `error` is null with the waiting line in `notes`, so it fails on a build that files that line under `error`.

### Task 4: Prove a backfill's row matches a tree pass's

- **Files:** crates/terminus/tests/backfill.rs
- **Action:** Add a test that starts from `Dirs::new()` and places two fixtures with `place` in one project directory under the bench's Claude directory. Build one store with `Dirs::command_in(first, &["ingest"])`, and push one session's watermark past its archive the way task 2 does. Copy that store's database file into a second data directory, with any `-wal` or `-shm` file beside it, so both runs start from one store. Then run `terminus ingest` against the first and `terminus backfill --work` against the second through `work_fully`. `backfill --work` is the process `terminus backfill` detaches, and the one this file's kill tests already drive. From each store, read the newest `runs` row's every column. The sets of column names whose value is non-null are equal. Both rows have `error` null, and both rows' `notes` contain recovery's line naming that session key and ending in "lowered".
- **Verify:** `cargo test -p terminus --all-features --test backfill` passes. The test compares the non-null column sets, so it fails on a build where the backfill's row leaves `notes` null or files recovery under `error`.

### Task 5: Rewrite the docs that state the old split

- **Files:** docs/commands.md (`terminus status`), docs/json-shapes.md (the `status` section), docs/design/0003-ingest.md (the module table, `terminus ingest <path.jsonl>`, `terminus backfill`, section 6 "One pass")
- **Action:** D-14 governs these passages; rewrite only them.
  - **`docs/commands.md`.** In the `terminus status` paragraph, the sentence "The lines recovery leaves, such as a watermark it lowered, print under `error` even when nothing failed." now says those lines print under notes, so a pass that only repaired a watermark shows no error.
  - **`docs/json-shapes.md`.** In the `status` bullets, the `error` sentence drops "a watermark repaired". The `notes` sentence names a watermark recovery lowered or removed among its examples.
  - **`docs/design/0003-ingest.md`, the named file.** The outputs of `terminus ingest <path.jsonl>` no longer say the file's own transaction writes its `runs` row only when there are new bytes. The file archives in one transaction. Then the run writes one `runs` row whether the file committed, was up to date or failed: recovery's lines go in `notes`, and a failed file is named under `error`. The command exits 1 on such a file and does not flag it diverged.
  - **`docs/design/0003-ingest.md`, the modules.** The `ingest/pass.rs` row in the module table stops claiming the `runs` row. Add rows for `crates/terminus-core/src/ingest/driver.rs` (the one driver every ingest runs through: the lock, recovery, the rows, the walk, each file's failure, and the `runs` row) and `crates/terminus-core/src/ingest/runs.rs` (the `runs` row's insert, and the reader `status` and `doctor` share).
  - **`docs/design/0003-ingest.md`, the backfill and section 6.** The backfill's outputs say its `runs` row is the one a tree pass writes, `notes` included. Section 6's lead-in names the driver beside `pass.rs`. Step 2 says recovery's repairs become lines in the row's `notes`.
  - **Style.** Docs describe the current design only. No history, no "used to".
- **Verify:**
  - `git grep -n 'even when nothing failed' docs/` prints nothing.
  - `git grep -nE 'RunRow|record_run|record_pass' crates/ docs/` prints nothing.
  - The `error` sentence in the `status` section of `docs/json-shapes.md` names no watermark.
  - `cargo test -p terminus-core --test planning_tags` passes.
  - Phase gate: `cargo test --workspace --all-features` passes, and `git diff --quiet f1f55af -- crates/terminus/tests/hook.rs` exits 0.

## Notes

- **The live-store check (human-verify, needs John's machine).** D-13 keeps a recovery-only pass on a copy of the live store as a check John runs. The `terminus` on PATH is the installed release, used only to find the store. Never `cp` the live store while hooks may write to it.
  1. `cd /code/terminus && cargo build --release -p terminus`
  2. `copy=$(mktemp -d "$HOME/terminus-copy.XXXXXX")`. Under `$HOME`, because the store is about 3 GiB.
  3. `sqlite3 "$(terminus status --json | jq -r .data.store)" "VACUUM INTO '$copy/terminus.db'"`
  4. Push one `full` session's watermark past its archive on the copy: `sqlite3 "$copy/terminus.db" "UPDATE watermarks SET byte_offset = byte_offset + 4096 WHERE transcript_path = (SELECT m.session_key FROM session_meta m JOIN watermarks w ON w.transcript_path = m.session_key WHERE coalesce(m.capture_mode, 'full') = 'full' AND w.byte_offset = m.uncompressed_len LIMIT 1)"`
  5. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus ingest; echo "exit $?"` ends `exit 0`.
  6. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus status` shows a `notes` block with a line ending "lowered". It has no `error` block, unless a real transcript on the machine failed, in which case the `error` block names only that file. `... status --json | jq .data.last_run.error` prints `null` when nothing failed.
  7. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus doctor` reports `last_run` as `ok`.
  8. `rm -rf "$copy"`
  This carries forward the phase 1 live-store check (phases/1/UAT.md, phases/1/SUMMARY.md, phase 1).
- **Planner's choice.** Task 4 drives `backfill --work` rather than the detaching `backfill`. The detached process runs exactly that command, and waiting on it would add a poll for a row that is already the thing under test.
