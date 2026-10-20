---
phase: 4
plan: 1
requirements:
  - "#85"
files:
  - crates/terminus-core/tests/pass.rs
  - crates/terminus-core/src/store/archive.rs
  - crates/terminus-core/src/ingest/mod.rs
---

# Phase 4: Steady-state ingest pass - Plan 1

## Goal

A pass over a transcript that has not grown reads none of its block rows, still reports everything about that file it reports today, and parses a grown file's tail once instead of twice (#85).

## Must be true when done

- With the `testkit` block-read log on, a pass over a session whose transcript is unchanged records no read of that session's `blocks` rows, the hash list included. A grown file, a diverged session and a session whose file is gone still record their reads.
- An unchanged file still gets, exactly as at `57c036e`, the refusal when another file holds its key, the refusal of a session with no `session_meta` row, the 32-byte checksum refusal, the flag for a file shorter than its watermark, and the `transcript_path` update for a moved file. The tests that show it pass at `57c036e` and after the change.
- A file that grows between the length check and the tail read is reported up to date with its watermark unmoved, and the next pass appends it. Nothing is ever appended without its stored hashes.
- `prepare` calls `parse::scan_from` once per grown file, and the watermark it writes is still the byte just past the tail's last newline.
- `git diff 57c036e -- crates/terminus-core/tests` shows only added lines, and `cargo test --workspace --all-features` passes.

## Context

- Locked decisions from `.planning/phases/4/CONTEXT.md`: D-08 (the gate is the file's length against its watermark, on the writer thread, before `archive::append_state`), D-09 (only the block-row reads and the turn count are skipped), D-10 (a `testkit`-only read log built like `discover::opened`), D-12 (existing tests and goldens untouched, lines only added), D-13 (the first scan becomes a last-newline search, the second scan and `derive_turn`'s parse stay).
- Test patterns to follow: the `Bench` helpers in `crates/terminus-core/tests/pass.rs` (`bench`, `place`, `pass`, `conn`, `config`, `append`, `last_line`, `stream_of`, `diverged`, `key_of`, `uuid`), and `opens_of` in `crates/terminus-core/tests/compaction.rs`: a shared log is counted for one target before and after a pass and never `reset`, because the tests in one binary run in parallel.
- PLAN-2 owns the pass-start query (D-11), the snapshot check (D-15), `prepare_cached` (D-14) and the measurement. Outside the phase: #89, any `--timing` flag, `derive_turn`'s own JSON parse, #70.

## Tasks

### Task 1: Pin an unchanged file's five outcomes in tests

- **Files:** crates/terminus-core/tests/pass.rs
- **Action:** Append new tests to `tests/pass.rs`, written and run before any source file changes, so they record what the code at `57c036e` does (AC3). Each scenario is built on a fresh bench for each entry point and run through both `bench.pass` (the tree walk, `pass::run_with`) and `ingest::run_with` on the one file, so neither entry point runs on the other's result. In every scenario the transcript's bytes are the ones the first pass archived, except where the scenario itself changes the file.
  1. Key held: archive a file under one project folder, then place a second file with the same name and the same bytes (so its length equals the watermark) under a project folder that sorts first, with the first file still on disk. The pass records exactly one failure, for the second file, whose reason contains "already held" and names both paths. `ingest::run_with` on the second file errs with "already held". The archived stream, the watermark and the session count are unchanged.
  2. No `session_meta` row: archive a file, delete its `session_meta` row, and run again on the unchanged file. The pass records one failure for that file whose reason names `session_meta`. `ingest::run_with` errs naming `session_meta`. The block rows (`testkit::stored`), the `sessions` row and its `session_no`, and the watermark are unchanged.
  3. Checksum: archive a file, set its `session_meta.checksum` to a blob that is not 32 bytes, and run again on the unchanged file. Both entry points report a failure containing "32 bytes", the archive is unchanged, and `transcript_diverged` stays null. `tests/compaction.rs` `another_per_file_failure_flags_no_session` covers the walk on its own tree; this test puts the same case on the pass bench next to the other four.
  4. Shorter than the watermark: archive a file, then rewrite it shorter than its watermark. The pass records the failure and sets `transcript_diverged`. `ingest::run_with` errs with the `TranscriptDiverged` message and, because only the walk calls `flag_divergence`, leaves the flag unset. The archived stream is unchanged either way.
  5. Moved: archive a file, then rename its project folder with no link left behind. The pass commits nothing, records no failure and adds no session, and `session_meta.transcript_path` names the file at its new path. `ingest::run_with` on the new path returns `Outcome::UpToDate`, and the row names the new path. The stored stream is unchanged.

  Where the description above and the code at HEAD disagree on a detail, the test asserts what HEAD does: the goal is a record of today's behavior, not a new rule. Do not edit any existing test.
- **Verify:** `git diff --quiet 57c036e -- crates ':(exclude)crates/terminus-core/tests/pass.rs'` exits 0, which shows the source under test is `57c036e`'s (`git diff --stat 57c036e HEAD` touches only `.planning/`). `cargo test -p terminus-core --all-features --test pass` passes and lists every new test. `git diff 57c036e -- crates/terminus-core/tests | grep '^-[^-]'` prints nothing.

### Task 2: Log every block-row read under testkit

- **Files:** crates/terminus-core/src/store/archive.rs, crates/terminus-core/tests/pass.rs
- **Action:** In `store/archive.rs`, add a log of `blocks` reads that exists only under the `testkit` feature, built the way `discover::opened` is (D-10). It is a `#[cfg(feature = "testkit")]` public module holding a static mutex-guarded list that recovers from a poisoned lock, a reset, and readers. Its `#[cfg(not(feature = "testkit"))]` twin's record function is an `#[inline(always)]` no-op, so the shipped binary takes no branch for it.
  - Record one entry for each query that reads one session's `blocks` rows, in each of these: `append_state`'s hash-list query (counted as a read, D-10), its tail query and its every-row query, then `ranges`, `rows`, and `stream`'s `numbered_rows`.
  - Each entry names the database file, as rusqlite's `Connection::path` reports it for the connection that ran the read, and the session's `session_no`. Every temp store numbers its sessions from 0 and the tests in one binary run in parallel, so an entry carrying only `session_no` would credit one test with another's reads.
  - `stored_bytes` is not logged: it sums lengths over every session for `terminus usage`, no pass calls it, and it reads no single session's rows.
  - `Pass.block_rows_read` and `Outcome::UpToDate` keep their shape (D-10).

  Then append to `tests/pass.rs` the half of AC2 that already holds today: three tests, each counting the log's entries for one session's `session_no` in its own store before and after a pass, comparing database paths canonicalized on both sides and never calling the reset. A file grown by one appended record (`last_line`) records at least one read. A session flagged diverged by setting `transcript_diverged` to 1, with its file unchanged, records reads. A session whose file is gone because its project folder was renamed records reads.
- **Verify:** `cargo test -p terminus-core --all-features --test pass` passes, including the three new tests. `cargo build --release -p terminus` succeeds without the feature. `grep -n 'cfg(not(feature = "testkit"))' crates/terminus-core/src/store/archive.rs` shows the inert twin. `cargo test --workspace --all-features` passes, and `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.

### Task 3: Skip an unchanged file's block rows and turn count

- **Files:** crates/terminus-core/src/ingest/mod.rs, crates/terminus-core/tests/pass.rs
- **Action:** First, append the other half of AC2 to `tests/pass.rs` and watch it fail. One pass archives two sessions. Then one transcript grows by a record (`last_line`) and the other does not. Across the next pass, the unchanged session has zero new log entries and the grown one has at least one. Today it fails, because `Existing::read` calls `archive::append_state` for every session that is not evicted. One step outside ingest also reads blocks: `observe::observe_new` reads a session's stream once (through `archive::stream` in `observe/mechanical.rs`), in the pass where the session becomes final. If the unchanged session is still waiting for that read when the counted window opens, run one settling pass first, so the window holds only the steady-state pass.

  Then change `Existing::read` (`ingest/mod.rs`), which runs on the writer thread for the walk, for backfill's `read_existing` and for the single-file `ingest_locked`:
  - The refusals keep their place and run first, exactly as today (D-09). That covers the archived session with no `session_meta` row, then the checksum that is not 32 bytes.
  - Read the file's length with a metadata call on `path`. Do not use `discover::open_transcript`: a stat is not an open, and `tests/compaction.rs` asserts one counted open per flagged file per pass.
  - Call `archive::append_state` and read the next turn number only when the session is not evicted and at least one of these holds: the file is longer than its watermark, the session is flagged diverged, or its holder is `Holder::Gone`. That matches today's `whole` condition, plus growth (D-08). An evicted session keeps `AppendState::empty()` as today.
  - The next turn number is `coalesce(max(turn_seq) + 1, 0)` over the session's `turns`, in place of `count(*)`. The two are equal in all 5,083 live sessions with turns, and `max` is a seek on the `UNIQUE (session_key, turn_seq)` index (D-11).
  - A length equal to the watermark, a shorter file, or a metadata call that fails all fetch nothing. A shorter file still reaches `open_at_least` in `prepare` and fails with `TranscriptDiverged`, which the walk flags. A vanished file fails at `prepare`'s open with today's io error.
  - When nothing was fetched, `Existing` must say so in a form `prepare` cannot mistake for an empty archive. Reusing `AppendState::empty()` for this is wrong: its empty hash list would reach `blob::append`, fail against the checksum, and turn a file that grew after the check into a per-file failure instead of leaving it for the next pass.
  - `prepare` keeps its order: `Holder::Other` refused first, evicted returns no work, then `read_tail` or `read_flagged`, so a file is still opened once per pass. When there is a complete record past the watermark but `Existing` fetched nothing, `prepare` returns no work, with the watermark unmoved and `moved` honored, exactly like the `consumed == 0` arm. The file is left for the next pass and never appended without its hashes (D-08).
  - Update the comments that describe the old cost: `Existing`'s "a handful of queries, and the bytes of at most one block row", and `Outcome::UpToDate`'s, which now also covers a file that grew after its check.

  This narrows the v0.0.1 phase 7 UAT rule ("the pass reads at most the last block row per walked file, except a session flagged diverged", `v0.0.1/phases/7/UAT.md`) to no rows at all for an unchanged file. The tests behind that UAT must still pass unchanged.

  Add a unit test inside `ingest/mod.rs` under `#[cfg(test)]`, since `Existing` and `prepare` are crate-private. It reads `Existing` for an archived, unchanged transcript, appends a complete record to the file, then runs `prepare` and `apply`. The result is `Outcome::UpToDate`, with the watermark and every block row byte-identical. A fresh `Existing::read`, `prepare` and `apply` then commits the record.
- **Verify:** The new `tests/pass.rs` test fails before the `ingest/mod.rs` change and passes after it. `cargo test -p terminus-core --all-features --lib ingest` runs the grow-after-check unit test, and it passes. `cargo test --workspace --all-features` passes, including the ingest, recover, compaction (divergence), backfill and `crates/terminus/tests/crash.rs` suites and Task 1's tests. `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean. `git diff 57c036e -- crates/terminus-core/tests crates/terminus/tests | grep '^-[^-]'` prints nothing.

### Task 4: Find a grown file's resume offset without a parse

- **Files:** crates/terminus-core/src/ingest/mod.rs
- **Action:** In `prepare`, replace the first `parse::scan_from` over the raw tail (the scan whose only output used is the resume offset) with a search for the tail's last `\n` (D-13). `consumed` is the index just past it, or 0 when the tail holds none, and the watermark written is the old watermark plus `consumed`. That is what `scan_from` computes as `resume_offset` (the rule `parse/mod.rs`'s module comment states: the byte just past the last newline, never end of file), so blank lines and a trailing partial record give the same answer as before.
  - `bytes_read` stays `consumed`, a count of file bytes.
  - Keep the second `scan_from` over `stored`. Turns are derived from it, and under `lean` and `minimal` the stored bytes differ from the file's, so its offsets are the only ones that address the archived stream (D-13). `derive_turn` keeps its own parse.
  - Do not add a second definition of the newline rule in `parse`: the search stays in `prepare`, with a comment naming the rule it matches.
  - Rewrite the "FIRST scan" and "SECOND scan" comments to describe one search and one scan.
- **Verify:** `grep -c 'scan_from' crates/terminus-core/src/ingest/mod.rs` prints 1. `cargo test --workspace --all-features` passes, including `tests/ingest.rs`'s `resuming_from_a_mid_line_watermark_converges_on_a_single_pass`, the `lean` and `minimal` cases in `tests/capture.rs`, backfill's byte-identity tests and Tasks 1 to 3's tests. `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.

## Notes

- Plan shape: two plans, as CONTEXT directs. They run in sequence because both edit `crates/terminus-core/src/ingest/mod.rs`, and PLAN-2's measurement needs this plan's code.
- AC3's "passes at `57c036e`": `git diff --stat 57c036e HEAD` touches only `.planning/STATE.md` and `.planning/phases/4/CONTEXT.md`, so Task 1's tests passing before any source change is a pass at `57c036e`'s code.
- Issue #85 also asks for the tail to be moved instead of copied (`stored.into_owned()` in `prepare`). CONTEXT's scope does not carry that item, so no task does it.
