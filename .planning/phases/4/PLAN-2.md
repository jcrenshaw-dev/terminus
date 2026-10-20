---
phase: 4
plan: 2
requirements:
  - "#85"
files:
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/ingest/backfill.rs
  - crates/terminus-core/src/derive.rs
  - .planning/phases/4/MEASUREMENT.md
---

# Phase 4: Steady-state ingest pass - Plan 2

## Goal

A steady-state pass reads its per-file rows once, opens the store for writing once, compiles `derive`'s statements once per connection, and the phase holds a before-and-after measurement of that pass on a copy of the live store (#85).

## Must be true when done

- Each file's `sessions`, `session_meta` and `watermarks` row is read once at pass start, after recovery, in one query, and `claim` answers from it. No per-file lookup of those three tables is left in the walk or in backfill.
- A pass with no snapshot due opens `terminus.db` for writing once, as `strace -e trace=openat` shows. With a snapshot due, the snapshot is still taken, and `last_snapshot_at` is written only after the copy lands.
- `derive`'s nine statements go through `prepare_cached`, and the statement cache holds every statement used in one apply loop and in one `reindex` loop at its default size.
- `.planning/phases/4/MEASUREMENT.md` holds, for the build at `57c036e` and for this phase's final build, at least 5 timed steady-state passes each on a `sqlite3 .backup` copy of the live store after one discarded warm-up. Each pass is recorded with its wall clock, `runs.duration_ms`, `files_committed` and an `strace -c` summary.
- `git diff 57c036e` under `crates/*/tests` shows only added lines, and `cargo test --workspace --all-features` passes.

## Context

- Runs after PLAN-1, which it depends on: it edits `Existing` in `ingest/mod.rs` after PLAN-1's gate is in, and the measurement measures the final code.
- Locked decisions from `.planning/phases/4/CONTEXT.md`: D-11 (one pass-start query joining `sessions`, `session_meta` and `watermarks`, with the turn number left out of it), D-14 (`prepare_cached` in `derive`, cache size set only if a loop needs more than 16), D-15 (the snapshot gate is answered on the pass's own store inside `locked`, a writable open happens after release only when a snapshot is due, and #89 stays open), D-01 to D-06 (how the measurement is taken), D-12 (existing tests untouched).
- Outside the phase: making the writable open take the ingest lock (#89), a `--timing` flag or any timing code in the binary, `derive_turn`'s own parse, and #70.

## Tasks

### Task 1: Read every file's row once at pass start

- **Files:** crates/terminus-core/src/ingest/mod.rs, crates/terminus-core/src/ingest/pass.rs, crates/terminus-core/src/ingest/backfill.rs
- **Action:** Write one query that reads, for every archived session, the per-file row `Existing::read` assembles today from three lookups. That is `session_no` and `checksum`, plus the `session_meta` columns it reads (`agent_meta IS NOT NULL`, `transcript_diverged`, `is_evicted`, `uncompressed_len`, `capture_mode`, `block_size`, `transcript_path`) and the `watermarks.byte_offset` for the key. The query runs from `sessions`, LEFT JOIN `session_meta`, LEFT JOIN `watermarks` on `transcript_path = session_key`, and its rows are held in memory by session key (D-11).
  - The LEFT JOIN on `session_meta` is required. A session that lost its row must still be found and refused, and an inner join reads it as brand new, which is the overwrite `Existing::read`'s LEFT JOIN comment describes.
  - A key with no row in the result is a never-archived file with watermark 0. That holds because recovery's `sweep_watermarks` (`recover.rs`) removes every non-zero watermark whose key has no `sessions` row before any file is read. So the read must come after `recover::recover`.
  - Call it in `pass::locked` after recovery and the feedback drain and before `walk`, and in `backfill::run_with` after recovery and before `pipeline`. Never call it before recovery: the re-key and twin repair rewrite these rows (`hardin/phases/4/UAT.md`, phase 4: one pass repairs a planted doubled store before the walk).
  - `claim` takes the in-memory rows instead of the connection and builds `Holder::of` from the row's `transcript_path`. Delete `stored_holder`.
  - In the walk and in backfill's `dispatch_round` and `read_existing`, `Existing` is built from the file's row instead of running its watermark, `session_meta` and `sessions` lookups. PLAN-1's gated `append_state` and next-turn reads stay per file. The no-`session_meta` and 32-byte checksum refusals are raised from the row in the same order as before (D-09).
  - The named-file path (`ingest::run_with` into `ingest_locked`) reads only its own key's row, using the same query restricted to that key, so the row has one definition and ingesting one file does not read 5,090 rows. This is the planner's choice where CONTEXT is silent.
  - Rows read once cannot go stale inside a pass. `claim`'s `taken` map lets each key through once per walk, so no apply in the walk changes a row a later file reads. This is the argument `dispatch_round`'s comment already makes per round, now made per run.
  - Rewrite the comments that name the old order: `run_with`'s "asked in `Existing::read`, the one place...", `Existing::read`'s per-file query comment, and `claim`'s "the store is asked before the walk is".
- **Verify:** `grep -rn 'stored_holder' crates/terminus-core/src` prints nothing. Outside PLAN-1's `#[cfg(test)]` module, `grep -n 'SELECT' crates/terminus-core/src/ingest/mod.rs` lists only these: the one per-file row query, the next-turn query, `next_session_no`'s and `continues_from`'s. `cargo test --workspace --all-features` passes, including PLAN-1's AC2 and AC3 tests, the #69 key-holder tests in `tests/pass.rs`, `tests/backfill.rs`, `tests/recover.rs`, `tests/session_keys.rs` and `crates/terminus/tests/crash.rs`. `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.

### Task 2: Answer whether a snapshot is due on the pass's own store

- **Files:** crates/terminus-core/src/ingest/pass.rs
- **Action:** Move `roll`'s gate into `locked` (D-15). It goes after `record_pass` has committed and before `locked` returns, which is where the ingest guard drops. When `config.snapshots_enabled()`, read SQLite's `strftime('%s', 'now')` and `META_LAST_SNAPSHOT` from the pass's own store, and decide whether a snapshot is due with the interval comparison `roll` uses today.
  - Carry the decision and that `now` reading out to `run_with` without adding a field to the public `Summary` or `Snapshotted`, which the CLI and four test files read.
  - `run_with` calls the snapshot step only when one is due. That step opens `Store::open`, runs `snapshot::take` then `snapshot::prune`, and calls `set_meta_int(META_LAST_SNAPSHOT, now)` only after `take` returned `Ok`. The order and every note's wording stay as they are today.
  - A failed `now` or `meta` read is a note in `summary.snapshot.notes` with today's text, never a failed pass. The `runs` row has already committed, so it cannot reach `runs.error`, which is what `Summary::snapshot`'s doc already says.
  - Not due, or snapshots off: no second store is opened.
  - Do not take the snapshot before release, which D-15 rejected because it holds the lock through a `VACUUM INTO` of a 3 GB store. A snapshot taken after release while another pass holds the lock is still a valid store (`v0.1.0/phases/8/UAT.md`, phase 8).
  - Leave `Store::open`'s signature alone: #89 stays open.
  - Rewrite `roll`'s and `run_with`'s doc comments to describe the split.
- **Verify:** `cargo test -p terminus-core --all-features --test snapshot` passes unchanged. That covers the first pass snapshotting and the next one inside the interval not doing so, a backdated stamp taking the next snapshot, the prune to the configured count, snapshots off writing nothing, and `a_snapshot_that_cannot_be_written_is_a_note_and_not_a_failed_pass` leaving the stamp unwritten.
  - Run `cargo build -p terminus` and set up a temp tree. Put `roots = ['<tmp>/claude']` in `<tmp>/cfg/terminus.toml`, and copy `tests/fixtures/session-basic.jsonl` to `<tmp>/claude/projects/-tmp-p/11111111-1111-4111-8111-111111111111.jsonl`.
  - Run `TERMINUS_CONFIG_DIR=<tmp>/cfg TERMINUS_DATA_DIR=<tmp>/data target/debug/terminus ingest` once, which creates the store and takes its first snapshot.
  - Then run the same command under `strace -f -e trace=openat -o <tmp>/openat.txt`. The trace holds exactly one successful `openat` of `<tmp>/data/terminus.db` with `O_RDWR`. Count only the path ending in `terminus.db"` (not `-wal` or `-shm`) and only calls whose return is not `-1`.
  - Then set `last_snapshot_at` to 0 on that temp store with `sqlite3` and run once more. A second `terminus-*.db` appears in `<tmp>/data/snapshots`, and the stamp is now non-zero.
  - `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.

### Task 3: Compile derive's statements once per connection

- **Files:** crates/terminus-core/src/derive.rs
- **Action:** Switch each of `derive.rs`'s nine statements from `tx.execute` to a statement taken from `prepare_cached` on the connection it is handed (D-14). The nine are: the `turns` upsert; the `turns_fts` delete and insert; the `entities` and `paths` deletes; the `entities` and `paths` inserts in the entity loop; and `set_boundary`'s `compactions` delete and insert. The SQL text stays byte-for-byte as it is, so `reindex` and ingest still write through one seam and nothing a rebuild compares can drift.
  - The cache count D-14 asks for, from the code: rusqlite 0.40.2's default cache holds 16 and evicts the least recently used (`lib.rs:168`). Across one walk, the cached statements in use are `archive::write` (1), `append_state`'s hash-list, tail and every-row queries (3), and `derive` (9). That is 13, or 14 if PLAN-1's next-turn read is cached. One `reindex` loop uses `layout` and `numbered_rows` (2) and `derive` (9), which is 11. `recover.rs`'s one cached statement runs once, before either loop.
  - Every loop fits inside 16, so `Store::open` is not changed and no cache size is set. If the executor's recount finds a loop above 16, that is a stop-and-report, not a silent edit to `store/open.rs`, which this plan does not lease.
- **Verify:** `grep -c 'prepare_cached' crates/terminus-core/src/derive.rs` prints 9, and `grep -n 'tx.execute' crates/terminus-core/src/derive.rs` prints nothing. `git diff --quiet 57c036e -- crates/terminus-core/src/store/open.rs` exits 0. `cargo test --workspace --all-features` passes, including `tests/derive.rs`, `tests/reindex.rs`, `tests/index.rs` and `tests/recall_golden.rs` against their unchanged goldens. `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.

### Task 4: Measure steady-state passes before and after on a copy of the live store

- **Files:** .planning/phases/4/MEASUREMENT.md
- **Action:** Take the measurement D-01 to D-06 describe and record it in `.planning/phases/4/MEASUREMENT.md`. No timing code is added to either build (D-01).
  1. Builds. For the baseline, extract `git -C /code/terminus archive 57c036e` into a directory outside the repository and run `cargo build --release --locked -p terminus` there. For the final build, run the same command in `/code/terminus` at the commit after Task 3, and record `git rev-parse HEAD` and both binary paths. Use `git archive` and not `git worktree`, so the measurement leaves no git state behind.
  2. Copy. Make a new directory on the same disk as the live store, not under `/tmp`, which is a tmpfs on this machine. Copy the store with `sqlite3 -readonly ~/.local/share/terminus/terminus.db ".backup '<copy>/terminus.db'"`. That is D-03's method; `-readonly` keeps even the `sqlite3` connection to the live file read-only. If `-readonly` refuses `.backup`, use D-03's exact command, which still never runs terminus's writable open on the live store. No terminus command from either build ever runs with `TERMINUS_DATA_DIR` unset or pointing at the live directory. No `terminus.toml` exists on this machine, so both builds run with the defaults (roots under `~/.claude`, daily snapshots), reading the live transcripts in place (D-05).
  3. Warm-up. Run `TERMINUS_DATA_DIR=<copy> <final binary> ingest` once and discard it. It rebuilds about 1.15 million turns from `derived_schema` 4 to 5 (D-04), and it may take the copy's first snapshot, because the live `last_snapshot_at` is from 2026-10-19. Then set the copy's `last_snapshot_at` to SQLite's current unix time with `sqlite3` on the copy, so no timed pass falls due mid-series, and say so in the record.
  4. Timed passes. Alternate the two builds on the same copy (D-04), at least 5 of each.
     - Each timed pass runs with no tracer and records the wall clock of the whole process, for example from `date +%s%N` before and after (D-02).
     - Each pass is then recorded with `duration_ms`, `files_seen`, `files_committed` and `files_failed` from the copy's newest `runs` row, read with `sqlite3 -readonly <copy>/terminus.db` ordered by `id`.
     - Each untraced pass is followed at once by one pass of the same build under `strace -c -f -o <file>`. Its summary is recorded beside the timed pass, with its own `files_committed`. The summary covers total calls and seconds, plus the `openat`, `statx`/`newfstatat`/`fstat`, `lseek`, `read`, `pread64`, `pwrite64`, `fdatasync`/`fsync` and `close` lines.
     - The wall clock is never taken under strace. `strace -c` stops the process on every system call, and the two builds make different numbers of calls, so a traced wall clock would inflate the old build more than the new one.
  5. AC4 on the copy, with no snapshot due. Run each build once under `strace -f -e trace=openat -o <file>`. Count the successful `openat` calls of `<copy>/terminus.db` with `O_RDWR`, using the same rule as Task 2 (exact file name, return not `-1`). Record both counts; the final build's must be 1. Then set the copy's `last_snapshot_at` to 0 and run the final build once. Record that a new snapshot file appeared under `<copy>/snapshots` and that the stamp afterwards is non-zero. The stamp-after-copy order is proven by `tests/snapshot.rs`.
  6. The record states:
     - the date, and RAM and page cache from `free -g`, with the warm-cache caveat (D-06);
     - the live store's session count;
     - both commit ids and binary paths, the copy's path and size, and every command used, verbatim;
     - one table per build with each timed pass's wall-clock ms, `duration_ms`, `files_seen`, `files_committed`, `files_failed` and its paired strace totals, then the median and mean per build;
     - the AC4 counts and the snapshot-due check.
     - It also notes that `files_committed` varies because the transcripts are live, that hook-triggered passes on the live store run beside the series, and that a session going idle during the series gets its one observation read in whichever pass marks it final. The live store had 0 sessions waiting for observation on 2026-10-20.
  7. Leave the copy where it is and name it in the record, so UAT can re-run AC4 on it. Removing it is John's call.
- **Verify:**
  - `.planning/phases/4/MEASUREMENT.md` names `57c036e` and the final commit id. It shows one discarded warm-up, and at least 5 untraced timed passes per build, each with a wall clock, `runs.duration_ms`, `files_committed` and a paired `strace -c` summary.
  - Its AC4 section shows the final build's no-snapshot-due trace with exactly one successful `O_RDWR` `openat` of `terminus.db`, and the snapshot-due run producing a new snapshot file.
  - Every command it lists that names `~/.local/share/terminus` is the read-only `.backup`.
  - At the final commit, `cargo test --workspace --all-features` passes, and `git diff 57c036e -- crates/terminus-core/tests crates/terminus/tests | grep '^-[^-]'` prints nothing (AC5).

## Notes

- Task 1's single-key read for a named file, the paired traced run in Task 4, `-readonly` on the `.backup`, the stamp after the warm-up, the copy's location on disk, and leaving the copy for UAT are all the planner's choices where CONTEXT is silent. Each is stated in its task with the reason.
- D-14's count: the flagged assumption said 14 or 15, and the code gives 13 (14 at most) per walk and 11 per `reindex` loop. So the explicit cache size D-14 allows for is not needed, and `store/open.rs` is not in this plan.
- AC6 (#85 closed by the merged change) is for the orchestrator's PR: its description carries "Closes #85". After the merge it is a human-verify: on GitHub, #85 shows as closed by that PR.
