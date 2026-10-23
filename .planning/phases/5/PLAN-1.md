---
phase: 5
plan: 1
requirements: [PASS-01, PASS-02]
files:
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/ingest/backfill.rs
  - crates/terminus-core/src/ingest/driver.rs
  - crates/terminus-core/src/ingest/runs.rs
  - crates/terminus-core/src/recover.rs
  - crates/terminus-core/src/observe/mod.rs
  - crates/terminus-core/src/reindex.rs
  - crates/terminus-core/src/feedback/mod.rs
  - crates/terminus/src/cmd/status.rs
  - crates/terminus/src/cmd/doctor.rs
  - crates/terminus-core/tests/ingest.rs
  - crates/terminus-core/tests/recover.rs
  - crates/terminus-core/tests/observe.rs
  - crates/terminus-core/tests/pass.rs
---

# Phase 5: One ingest pass - Plan 1 of 2 (the driver, the row and the reader)

## Goal

One pass driver runs every pass, and a pass that only recovered is not reported as an error (#31, #96). This plan does the code: the tree pass, the backfill and single-file ingest run through one driver that writes one `runs` row the same way for all three; recovery's lines and observe's "waiting to be observed" line go to `notes`; and `status` and `doctor` read the last run through one core function. Plan 2 adds the process-level proofs and the markdown docs.

## Must be true when done

- `git grep -nE '\bRunRow\b|fn record_run|fn record_pass' crates/` prints nothing. `pass::run_with`, `backfill::run_with` and `ingest::run_with` each call the one driver function in `crates/terminus-core/src/ingest/driver.rs`. In `crates/terminus-core/src/ingest/`, only that module takes the ingest lock or runs recovery.
- A tree pass, a backfill and `terminus ingest <path>` each write one `runs` row, through one INSERT, after the walk, with `notes` written by all three.
- A named file writes a row whether it commits (1 seen, 1 committed, 0 failed), is up to date (1, 0, 0) or fails (1, 0, 1, with its path and reason under `error`). A failure still returns the typed error and never sets the divergence flag.
- A pass that only recovered leaves `runs.error` null with recovery's lines in `runs.notes`. A pass that finds more than 100 newly final sessions and fails nothing leaves `error` null with the "waiting to be observed" line in `notes`.
- `git grep -n 'FROM runs' crates/terminus/src` prints nothing. `status` and `doctor` read the last run through one core function, and what they print is unchanged.
- `cargo test --workspace --all-features` passes, and `crates/terminus/tests/hook.rs` is byte for byte what it was at `f1f55af`.

## Context

- Locked: every decision in `.planning/phases/5/CONTEXT.md` (D-01 to D-14). This plan does D-01 to D-12 and the code-comment half of D-14. Plan 2 does D-13, the markdown half of D-14, and the process-level criteria.
- Patterns to follow: `pass::record_pass` is the shape of the one row; `feedback::Labeled` and `retention::Applied` are the shape of an `errors()`/`notes()` split; `config::visible::counts(conn, config)` is the shape of a core read the binary calls without naming `rusqlite`.
- Out of scope (CONTEXT): giving the backfill or single-file ingest any of the tree pass's post-walk steps; any change to the signatures or return types of `pass::run_with`, `backfill::run_with` and `ingest::run_with` (`PassOutcome`, `Report`, `Outcome`); a `note` state for `doctor`'s `last_run`; any edit to `crates/terminus/tests/hook.rs`; running `cargo fmt` over the workspace. Rewrap comments by hand. New comments and tests may cite `#31` and `#96`, never a requirement id or a decision tag (`crates/terminus-core/tests/planning_tags.rs` fails on those).
- Plans run in order 1 then 2. They share no file, but plan 2's tests prove this plan's behaviour and fail without it, so they never run in parallel.

## Tasks

### Task 1: Give the per-file ingest a result with no lock or exclusion arm

- **Files:** crates/terminus-core/src/ingest/mod.rs (`Outcome`, `ingest_locked`, `apply`, `run_with`, the `tests` module), crates/terminus-core/src/ingest/pass.rs (`walk`), crates/terminus-core/src/ingest/backfill.rs (`commit_one`)
- **Action:** `apply` and `ingest_locked` return a crate-internal result with exactly two arms: committed, carrying the existing `Pass`, and up to date. `LockHeld` and `Excluded` stay only on the public `Outcome`, which `ingest::run_with` still returns. It maps the new result onto `Outcome::Committed` and `Outcome::UpToDate`, so its signature and every caller of it stay as they are (D-03, D-06). Drop the `Ok(Outcome::UpToDate | Outcome::LockHeld | Outcome::Excluded(_))` arms in `pass::walk` and `backfill::commit_one` in favour of the up-to-date arm. Rewrite the `Excluded` doc comment ("Only [`run`] produces this...") to say the per-file result cannot carry it. Update the unit test `a_file_that_grows_after_its_check_is_left_for_the_next_pass` to match on the new result; give the type whatever derives that test needs. Change nothing else about `apply`'s writes in this task. The `started` and `run_row` parameters go in task 4.
- **Verify:** `git grep -nE 'Outcome::(LockHeld|Excluded)' crates/terminus-core/src/ingest/pass.rs crates/terminus-core/src/ingest/backfill.rs` prints nothing. `cargo test -p terminus-core --features testkit --lib ingest::` passes, and so does `cargo test -p terminus-core --features testkit --test pass --test backfill --test ingest --test recover`.

### Task 2: Run the tree pass through one driver that writes the one row

- **Files:** crates/terminus-core/src/ingest/driver.rs (new), crates/terminus-core/src/ingest/runs.rs (new), crates/terminus-core/src/ingest/mod.rs (module declarations), crates/terminus-core/src/ingest/pass.rs (`run_with`, `locked`, `walk`, `record_pass`, `due`)
- **Action:** Add a crate-internal driver module and a public `runs` module. The `runs` module is public now because task 7 adds the reader the binary calls. Declare both in `ingest/mod.rs`.
  - **The row.** In `runs.rs`, one crate-internal function does the single `runs` INSERT. It is moved from `pass::record_pass` with its content unchanged: the same ten columns, the same `strftime` arithmetic for `started_at`, the same `error` list in the same order (failures and unreadable directories through `pass::note`, `recovery.lines()`, `feedback.lines()`, `outcomes.errors()`, `observations.lines()`, `retention.errors()`, then `pass failed: …` for a fatal error), and the same `notes` list (`outcomes.notes()`, then `retention.notes()`). Recovery and observe move to `notes` in tasks 5 and 6, not here. Do not name it `record_pass` or `record_run`, because `git grep -nE 'fn record_run|fn record_pass' crates/` must end up empty.
  - **The driver.** One function in `driver.rs` runs every pass in one order (D-02). It takes the lock with `lock::try_acquire`, and when the lock is held it returns `PassOutcome::LockHeld` having written nothing. It runs `recover::recover`, whose error propagates with no row, as today. It runs the walk's before-the-rows step, reads the rows with `read_rows` filtered as the walk asks (a failed read is the walk failing, as today), and runs the walk. It runs the walk's after-the-walk step, whether or not the walk failed, as the tree pass's post-walk steps run today. It sets `duration`, then does the one INSERT (D-05: one INSERT at the end, never a row inserted early and updated, so `fault::AFTER_BLOB` stays ahead of every `runs` write). It returns the walk's error when there is one, and otherwise propagates a failed INSERT. Only then does it run the walk's after-the-row step, on the pass's own store, before the guard drops, and it returns `PassOutcome::Ran(summary)`. Writing a fatal error as `pass failed: …` is the tree pass's and the backfill's choice. Shape the seam so the named file in task 4 can write its own failure as one failed file instead.
  - **How the walk is taken (planner's choice, flagged in CONTEXT).** A trait in `driver.rs`. Its methods are: the step before the rows, the row filter (every row, or one key's), the walk itself, the step after the walk, and the step after the row. The before, after-walk and after-row steps default to doing nothing. A trait is chosen over a closure or an enum of walks because each walk carries state the driver never reads: the tree pass's snapshot gate, the backfill's worker count and thread set, the named file's own result. Default no-op steps keep the backfill and the named file out of the tree pass's steps by construction (D-04).
  - **The per-file handler (D-06).** One function in `driver.rs` takes a file's path and its per-file result from task 1 and updates the `Summary`. On commit it counts files, bytes and turns and fires `fault::stall_after_files`. On up to date it does nothing. On an error it records the path and reason, and on `Error::TranscriptDiverged` or `Error::TranscriptRewritten` it sets the flag with `flag_divergence`, keeping the " (the flag could not be set: …)" suffix and matching on the variant, never the message. Route claim failures (`Claim::Held`, an `Err` from `claim`) through its failure half; an io error never matches the flag.
  - **The tree pass.** `pass::run_with` keeps its signature and return type. It hands the driver the tree pass's walk. The before step is `feedback::drain`, with a failed drain turned into a discarded entry as today, then `discover::discover` filling `unreadable` and `excluded`. The walk is today's `walk`, with `fault::pass_fails_after` where it is, and it calls the per-file handler. The after-walk step is `feedback::outcomes`, then `observe::observe_new`, then `retention::retain`. The after-row step is `due`, kept on the walk value. Once the driver returns, `run_with` runs `roll` as it does now, after the lock has dropped. `locked` and `record_pass` are deleted. Every comment in `pass.rs` that names `record_pass`, or says this file takes the lock, recovers or writes the row, now says the driver does it. Keep each comment's reason.
- **Verify:** `git grep -nE 'fn record_pass|fn locked|lock::try_acquire\(|recover::recover\(' crates/terminus-core/src/ingest/pass.rs` prints nothing. `cargo test -p terminus-core --features testkit` passes, and so does `cargo test -p terminus --all-features --test status --test doctor --test crash --test hook --test cli`. `git diff --quiet f1f55af -- crates/terminus/tests/hook.rs` exits 0.

### Task 3: Run the backfill through the driver

- **Files:** crates/terminus-core/src/ingest/backfill.rs (module doc, `run_with`, `pipeline`, `commit_one`, `record_pass`), crates/terminus-core/src/ingest/driver.rs
- **Action:** `backfill::run_with` keeps its signature and `Report`, and hands the driver the backfill's walk. The before step is `discover::discover` only, with no drain (D-04). The walk is `pipeline` over every row. There is no after-walk or after-row step. `Report.workers` is still `workers.max(1)`. `threads_used` comes from the walk's thread set, and is 0 when the lock was held. `commit_one`'s match and its copy of the divergence-flag code give way to a call to the driver's per-file handler. `backfill::record_pass` is deleted, so a backfill's row now comes from the same INSERT as a tree pass's and fills `notes` too. Threads are still started only inside `pipeline`; the driver starts none, which keeps the module doc's "no thread on the hook path" true. Update the module doc's "Everything that makes a pass resumable is unchanged" paragraph and the comments in `run_with` to say the driver takes the lock, recovers and writes the one row.
- **Verify:** `git grep -nE 'lock::try_acquire\(|recover::recover\(|fn record_pass|flag_divergence\(|INSERT INTO runs' crates/terminus-core/src/ingest/backfill.rs` prints nothing. `git grep -n 'fn record_pass' crates/` prints nothing. `cargo test -p terminus-core --features testkit --test backfill --test pass --test recover` passes, and so does `cargo test -p terminus --all-features --test backfill`.

### Task 4: Run a named file through the driver, with its row outside the file's transaction

- **Files:** crates/terminus-core/src/ingest/mod.rs (module doc, `Outcome::UpToDate` doc, `run_with`, `RunRow`, `ingest_locked`, `apply`, `record_run`, the `tests` module), crates/terminus-core/src/ingest/driver.rs, crates/terminus-core/src/ingest/pass.rs (module doc, `walk`), crates/terminus-core/src/ingest/backfill.rs (`commit_one`), crates/terminus-core/tests/ingest.rs (`a_rerun_on_an_unchanged_file_writes_nothing`)
- **Action:**
  - **The entry point.** `ingest::run_with` keeps its signature and `Outcome`. Canonicalizing and the exclusion check stay before the lock, so an excluded file still opens and writes nothing (D-07). It then hands the driver a one-file walk. The row filter is that file's key only (D-07). The walk is `ingest_locked` on the canonical path. There is no before, after-walk or after-row step (D-04). A held lock is still `Outcome::LockHeld` with nothing written. `crates/terminus/tests/lock_race.rs` depends on that.
  - **The row (D-07).** Every run that holds the lock writes one row through the driver's INSERT. Committed is 1 seen, 1 committed, 0 failed, with `bytes_read` and `turns_added` from the `Pass`. Up to date is 1, 0, 0. Failed is 1, 0, 1, with one `error` line holding the file's path and reason through `pass::note`. That line is not also written as a `pass failed:` line, because `files_failed` already counts it. Recovery's lines, discarded today as `_recovered`, go into the row like every pass's; task 5 moves them to `notes`.
  - **The failure (D-08).** Anything that fails for the named file after recovery is that file's failure: its key, its row read, its read, `prepare`, `apply`. The driver writes it into the row, then returns the typed `Error`, so `terminus ingest <path>` still exits non-zero on a damaged file and `matches!(err, Error::TranscriptDiverged { .. })` in `crates/terminus-core/tests/pass.rs` still holds. A named file never sets the divergence flag, so it does not go through the per-file handler's failure half. `crates/terminus/src/cmd/ingest.rs` does not change.
  - **D-01.** `apply` no longer writes a `runs` row. Delete `RunRow` and `record_run`. `apply` and `ingest_locked` lose their `started` and `run_row` parameters, and the dummy `Instant::now()` and `RunRow::PassOwns` arguments in `pass::walk` and `backfill::commit_one` go with them. A kill between the file's commit and the row now leaves a committed file with no row. CONTEXT accepted that trade, and `crates/terminus/tests/crash.rs` already leaves `runs` out of its comparison.
  - **Doc comments.** The `ingest/mod.rs` module doc says everything a run writes commits in one transaction. Rewrite it: the file's bytes, turns and watermark commit in one transaction, and the run's row is written after it. Rewrite the `UpToDate` doc's "not even to `runs`". The recovery comment in `run_with` either moves with recovery into the driver or goes. The `pass.rs` module doc lines on "`ingest::run`'s single-file behaviour" now state what is true: the walk records a file's failure and goes on, and a named file's failure comes back to its caller.
  - **Tests.** Update the unit test `a_file_that_grows_after_its_check_is_left_for_the_next_pass` to `apply`'s new parameters. Rewrite `a_rerun_on_an_unchanged_file_writes_nothing` in `crates/terminus-core/tests/ingest.rs`: an up-to-date rerun moves no table except `runs`, which gains exactly one row reading 1, 0, 0. Add a test there that runs `ingest::run` three times on one file: when it commits, when it is up to date, and after it is cut to `{}\n`. Each run adds exactly one row with the counts above. The third returns `Err` matching `Error::TranscriptDiverged`, and its row's `error` contains the file's path. A survey at planning found no other test that pins "an up-to-date or failed named file writes no row". Every other `runs` count in the suites follows a tree pass or a refused open. If one turns up anyway, rewrite it to the rule above in this task, and report its file as outside this task's lease.
- **Verify:** `git grep -nE '\bRunRow\b|fn record_run|fn record_pass' crates/` prints nothing. `git grep -nE 'lock::try_acquire\(|recover::recover\(' crates/terminus-core/src/ingest/` lists only `driver.rs`. The driver's entry function, grepped by name across `crates/`, has exactly three call sites: one each in `pass::run_with`, `backfill::run_with` and `ingest::run_with`. `cargo test --workspace --all-features` passes.

### Task 5: Write recovery's lines under notes

- **Files:** crates/terminus-core/src/ingest/runs.rs, crates/terminus-core/src/recover.rs (`Recovered::lines` doc), crates/terminus-core/tests/recover.rs
- **Action:** The row's INSERT puts `Recovered::lines()` in the `notes` list, first, because recovery runs first, instead of in the `error` list (D-09). A pass whose only lines come from recovery then leaves `runs.error` null. Every caller gets this from the one INSERT, so `status` and `doctor` change no logic: `status` prints `error` only when the column is non-null, and `doctor`'s `(None, 0)` arm already calls the run ok (D-11). Rewrite the `Recovered::lines` doc ("for a `runs.error` or a `status` report") to say the lines are notes. Since #105 their only content is lowered or removed watermarks (phases/1/CONTEXT.md, phase 1). Add a test to `crates/terminus-core/tests/recover.rs` with one fresh `bench()` and `damage()` per caller: the tree pass (`pass::run_with`), the backfill (`backfill::run_with` with two workers) and a named file (`ingest::run` on the bench transcript). In all three, the last `runs` row has `error` NULL and `notes` holding the lowered line naming the bench's key and the removed line naming `ORPHAN`.
- **Verify:** `cargo test -p terminus-core --features testkit --test recover --test pass --test backfill --test ingest` passes. The new test asserts `error IS NULL`, so it fails if the INSERT puts recovery's lines back under `error`.

### Task 6: Split observe's waiting line from its failures

- **Files:** crates/terminus-core/src/observe/mod.rs (module doc, `Observed`, `observe_new`), crates/terminus-core/src/ingest/runs.rs, crates/terminus-core/tests/observe.rs
- **Action:** `Observed` reports its failures and its overflow apart, the way `feedback::Labeled` and `retention::Applied` do (D-10).
  - **The overflow.** Keep it as a count on `Observed`, as `Applied` keeps `over`, not as a pre-formatted entry in the failure list. Planner's choice, flagged in CONTEXT.
  - **The two methods.** `errors()` returns the failure lines: the sessions could not be listed, the finalized sessions could not be read, and a session whose observation could not be written, which stays an error. `notes()` returns the "N more finalized session(s) are waiting to be observed; the next ingest takes the next 100" line, worded as now, when the count is above zero. `Observed::lines` goes. The row's INSERT puts `errors()` in `error` and `notes()` in `notes`.
  - **Doc comments.** Rewrite the `Observed` doc comments and the module doc's "folded into `runs.error`" to say failures go to `error` and the waiting line to `notes`.
  - **Tests.** A unit test in `observe/mod.rs` builds an `Observed` with one failure and an overflow, and asserts `errors()` holds only the failure and `notes()` only the waiting line. A test in `crates/terminus-core/tests/observe.rs` writes `MAX_PER_PASS + 1` transcripts through `pass_bench()` and `PassBench::transcript`, each with its own session uuid and a `ts("-10 hours")` clock. One pass writes `MAX_PER_PASS` observations, and its `runs` row has `error` NULL and `notes` containing "1 more finalized session(s) are waiting to be observed".
- **Verify:** `git grep -n 'fn lines' crates/terminus-core/src/observe/mod.rs` prints nothing. `cargo test -p terminus-core --features testkit --lib observe::` passes, and so does `cargo test -p terminus-core --features testkit --test observe --test pass --test label --test feedback --test compaction`.

### Task 7: Read the last run through one core function

- **Files:** crates/terminus-core/src/ingest/runs.rs, crates/terminus/src/cmd/status.rs (`run`, `document`, `Run`), crates/terminus/src/cmd/doctor.rs (`Run`, `Doctor::store`, `Doctor::last_run`)
- **Action:** Add a public function to `runs.rs` that reads the newest `runs` row by id into a public struct (D-12). It carries the nine values `status` prints: `started_at`, `duration_ms` with null read as 0, `files_seen`, `files_committed`, `files_failed`, `bytes_read`, `turns_added`, `error` and `notes`. An empty table is "no row" and a failed read is an error, so "no run yet" and "the query failed" stay two answers without a separate count.
  - **`status`.** It drops its `Run` struct, its `count(*)` probe and its last-row query, and reads through the core function. Its human output and `--json` document stay field for field what they are.
  - **`doctor`.** It drops its `Run` struct and both of its queries. In `Doctor::store`, a failed read is the `problem` "the runs table could not be read: …", no row is the existing `note`, and a row goes to `last_run`, whose judgement does not change (D-11).
  - **Constraint.** The binary crate names no `rusqlite` type; the connection comes from `Store::conn` as now. Rewrite `status.rs`'s "Counted first rather than reached for with an optional row" comment to match.
- **Verify:** `git grep -n 'FROM runs' crates/terminus/src` prints nothing. `git grep -n 'rusqlite::' crates/terminus/src/cmd/status.rs crates/terminus/src/cmd/doctor.rs` prints nothing. `cargo test -p terminus --all-features --test status --test doctor` passes with its tests unchanged, the planted-row rendering tests `status_prints_a_passs_routine_notes_apart_from_its_errors` and `a_last_run_with_only_notes_is_ok` among them.

### Task 8: Rewrite the comments that still name the old row writers

- **Files:** crates/terminus-core/src/reindex.rs (`reindex`), crates/terminus-core/src/feedback/mod.rs (`Drained::lines`), crates/terminus-core/tests/pass.rs (`one_pass_over_five_transcripts_writes_exactly_one_runs_row`)
- **Action:** Rewrite each comment that names `record_pass` or `record_run` to state its rule without the name (D-14). In `reindex.rs`, a failed rebuild "takes the pass with it before `record_pass` can say so": it takes the pass with it before the pass's `runs` row can say so. `Drained::lines` gives one line per discarded file, in the shape the `runs` row's `error` list takes. The doc of `one_pass_over_five_transcripts_writes_exactly_one_runs_row` states the rule it pins: one row per pass whatever the file count, so the last run `status` reports is the pass and not one file. Only comment lines change. Rewrap by hand.
- **Verify:** `git grep -nE 'RunRow|record_run|record_pass' crates/` prints nothing. `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning. `git diff --quiet f1f55af -- crates/terminus/tests/hook.rs` exits 0.

## Notes

- **The planner's choices**, all where CONTEXT left the call open:
  - The driver takes its walk as a trait with default no-op steps.
  - The driver is a crate-internal `ingest/driver.rs`. The `runs` row's INSERT and reader share a public `ingest/runs.rs`, so the column list lives in one file.
  - `Observed` keeps its overflow as a count.
  - A named file's failure is one `error` line, not repeated as `pass failed:`.
  - A named file whose key cannot be made fails after the lock, so it leaves a failed row.
  - `status` and `doctor` lose their `count(*)` probes, because the reader's "no row" answer replaces them.
- **Plan shape.** The CONTEXT directive asks for multiple plans, and this phase has two. They touch disjoint files, but they are sequential: plan 2's process-level tests fail until this plan's behaviour lands.
- **Recalled prior art.** CONTEXT's D-01 (phases/5/CONTEXT.md, phase 5) is task 4. The live-store check from phases/1/UAT.md (phase 1) carries forward as plan 2's human-verify item.
