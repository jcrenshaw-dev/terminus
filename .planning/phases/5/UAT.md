---
status: testing
phase: 5
fields_version: 1
started: 2026-10-23
updated: 2026-10-23
---

## Items

### 1. One driver and one last-run reader
expected: `git grep -nE '\bRunRow\b|fn record_run|fn record_pass' crates/` prints nothing; `pass::run_with`, `backfill::run_with` and `ingest::run_with` each call the same driver function; `git grep -n 'FROM runs ORDER BY id DESC' crates/terminus/src` prints nothing, and `status` and `doctor` read the last run through one core function
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: Old-name greps and both `FROM runs` greps over crates/terminus/src print nothing. driver::drive is called only from pass.rs:147, backfill.rs:112 and ingest/mod.rs:136. Lock and recovery appear only at driver.rs:71/81, the one INSERT is at runs.rs:98, and status.rs:57 and doctor.rs:832 both read through runs::last.

### 2. A backfill leaves the row a tree pass does
expected: A tree pass and `terminus backfill`, each on its own copy of one store with the same damaged watermark, leave last `runs` rows with the same set of non-null columns, and both rows' `notes` hold recovery's line
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: crates/terminus/tests/backfill.rs a_backfill_leaves_the_same_runs_row_a_tree_pass_does passed in the full run. It uses equal non-null column sets, error NULL and a `<key>: ... lowered` note in both rows, from a copied store with one watermark pushed past its archive, running `terminus ingest` against `backfill --work`.

### 3. A named file writes one row whatever it did
expected: `terminus ingest <path>` writes one `runs` row when the file commits, when it is already up to date, and when it fails; on failure it exits non-zero and the row's `error` names the file
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: status.rs and core ingest.rs three-run tests passed. A scratch spot-check gave exits 0, 0, 1 and rows (1,1,0), (1,0,0), (1,0,1), with error naming the file. status printed it under `  error`, and transcript_diverged was not set.

### 4. A recovery-only pass is no error
expected: On a real store whose watermark recovery must lower, with no failing file, after `terminus ingest`: `terminus status` prints no `error` line and shows recovery's line under notes, `terminus status --json` has `error: null`, and `terminus doctor` exits 0
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: crates/terminus/tests/doctor.rs a_pass_that_only_lowered_a_watermark_is_not_an_error passed with real watermark damage. A scratch spot-check showed a status notes line ending `lowered`, no error block, `--json` error null, and doctor `ok last_run`.

### 5. A shrunk transcript is still an error
expected: On a store with one shrunk transcript, after `terminus ingest`, `terminus status` prints that file under `error` and `terminus doctor` exits 1
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: crates/terminus/tests/doctor.rs a_pass_that_skipped_a_shrunk_transcript_is_a_problem passed. Status names the transcript under `  error`, and doctor exits 1 with last_run as its only problem.

### 6. Sessions waiting to be observed are a note
expected: A pass that finds more than 100 newly finished sessions and fails nothing records the "waiting to be observed" line in `notes`, not `error`, and `terminus doctor` exits 0
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: Observed::errors/notes split (observe/mod.rs) is wired at runs.rs:85/90. The unit, core and process-level tests passed. sessions_left_waiting_to_be_observed_are_not_an_error, with 101 sessions, shows error null, files_failed 0, the waiting line in notes, and doctor exit 0.

### 7. The suite passes and the hook test is untouched
expected: `cargo test --workspace --all-features` passes, and the phase's diff leaves `reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs` unchanged
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: cargo test --workspace --all-features exited 0 with 1037 passed and 0 failed. `git diff --quiet f1f55af -- crates/terminus/tests/hook.rs` exits 0, and the hook stdout test passed.

### 8. A recovery-only pass on a copy of the live store
expected: On a `VACUUM INTO` copy of the live store with `TERMINUS_DATA_DIR` pointed at it and one session's watermark pushed past its archive, this branch's build runs `terminus ingest` with exit 0, `terminus status` prints no `error` line and shows recovery's line under notes, and `terminus doctor` reports `last_run` ok
status: pass
first_pass: pass
source: model
evidence: Build: cargo build --release -p terminus at ce5c967. Copy: sqlite3 ~/.local/share/terminus/terminus.db "VACUUM INTO '<copy>/terminus.db'" -> exit 0; sessions 3537 and turns 727385 match the live store. Damage: PLAN-2.md Notes step 4 UPDATE pushed 2a81eb34-700f-4c54-a345-bdf81669b94a.jsonl's watermark from 1143914 by 4096, changes() 1. TERMINUS_DATA_DIR=<copy> ./target/release/terminus ingest -> exit 0. status -> '3546 walked, 2 committed, 0 failed', no error block, notes '2a81eb34-...jsonl: watermark 1148010 was past the 1143914 bytes the archive holds; lowered'. status --json .data.last_run.error -> null. doctor -> 'ok last_run ... 0 failed', 'no problems', exit 0. Copy removed.

### 9. A recovery-only pass on a copy of the live store: `cargo build --release -p terminus`; `copy=$(mktemp -d "$HOME/terminus-copy.XXXXXX")`; `sqlite3 "$(terminus status --json | jq -r .data.store)" "VACUUM INTO '$copy/terminus.db'"`; push one full session's watermark past its archive on the copy (the UPDATE in PLAN-2.md Notes step 4); then `TERMINUS_DATA_DIR="$copy" ./target/release/terminus ingest`, `... status`, `... status --json | jq .data.last_run.error`, `... doctor`; finally `rm -rf "$copy"`
expected: ingest exits 0. status shows a `notes` block with a line ending `lowered`, and no `error` block unless a real transcript on the machine failed, in which case the block names only that file. `.data.last_run.error` is null when nothing failed, and doctor reports last_run ok.
origin: verifier
why_human: No probe exercised it. It is not irreversible, and the live store is on this machine, so it can be run instead of asked. The verifier left it out because its contract forbids mutating state and caps a check at about 10 seconds, while this check reads John's live store and writes a copy of about 3 GiB under $HOME. The commands are fully scripted in PLAN-2.md Notes, so whoever is cleared to touch the live store can run them.
status: pass
first_pass: pass
source: model
evidence: Build: cargo build --release -p terminus at ce5c967. Copy: sqlite3 ~/.local/share/terminus/terminus.db "VACUUM INTO '<copy>/terminus.db'" -> exit 0; sessions 3537 and turns 727385 match the live store. Damage: PLAN-2.md Notes step 4 UPDATE pushed 2a81eb34-700f-4c54-a345-bdf81669b94a.jsonl's watermark from 1143914 by 4096, changes() 1. TERMINUS_DATA_DIR=<copy> ./target/release/terminus ingest -> exit 0. status -> '3546 walked, 2 committed, 0 failed', no error block, notes '2a81eb34-...jsonl: watermark 1148010 was past the 1143914 bytes the archive holds; lowered'. status --json .data.last_run.error -> null. doctor -> 'ok last_run ... 0 failed', 'no problems', exit 0. Copy removed.

## Summary

total: 9
passed: 9
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 0
