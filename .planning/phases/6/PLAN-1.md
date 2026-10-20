---
phase: 6
plan: 1
requirements:
  - "#90"
files:
  - crates/terminus-core/src/parse/mod.rs
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/backfill.rs
  - crates/terminus-core/src/ingest/pass.rs
---

# Phase 6: Simplify the 1.0.0 changes - Plan 1 (ingest)

## Goal

A simplify pass over the code the 1.0.0 fixes change, before the README and the docs sweep describe it (#90). It removes code that reinvents something the codebase already has, code that can be simpler, and wasted work, and it changes no behavior. This plan applies SIMPLIFY.md findings 1, 3, 5 and 8, which all sit in the ingest pass.

## Must be true when done

- `cargo test --workspace --all-features --no-fail-fast` reports 0 failed, and `git diff` from the commit this plan started from adds, removes or changes no line containing `assert`.
- The rule for where a pass stops in a file (just past the last `\n`) is written once, in `parse`. `prepare` and `scan_from` both call it, and `grep -rn "consumed(" crates` prints nothing.
- `Existing::read` takes the pass's row map and looks up its own row. `Row` derives `Default`, `Holder::of` takes the row and the path, and `grep -rn "rows.get(" crates/terminus-core/src/ingest` prints only the line in `claim` and the one in `Existing::read`.
- In `Existing::read`, the file's length is checked only for a session that is neither evicted nor `whole`.
- `Gate` is gone. The snapshot check returns a `Result`, and `grep -c "no snapshot was taken" crates/terminus-core/src/ingest/pass.rs` prints 2: the one in `run_with` and the one in `roll`.
- `git diff --name-only` from the commit this plan started from lists only the four files in this plan's frontmatter.
- Issue #90 is closed by the merged change, and the landing PR body carries `Fixes #90` (human-verify: needs the merged PR on GitHub).

## Context

- `.planning/phases/6/SIMPLIFY.md` stands in for CONTEXT.md. Each task applies one finding under `## Apply` exactly as written. Nothing under `## Skip` is planned, and its reasons already meet criterion 1.
- Binding on every task: no behavior change, every existing test passes, and no assertion changes. The `## Criterion 2` paragraph of SIMPLIFY.md is the locked reading of criterion 2.
- Phase 4 decisions this plan keeps: D-08 (a stat, not an open, decides whether a file grew, and the refusals run first), D-13 (a flagged session is compared in full), and D-15 (the snapshot gate is answered on the pass's own store inside the lock), all in `.planning/phases/4/CONTEXT.md`.
- Out of scope: `recover.rs`, which calls `scan_from` and holds the second next-turn query (skip 4). `read_tail` and `open_at_least`. The `rposition` inside the unit test `a_file_that_grows_after_its_check_is_left_for_the_next_pass` (skip 18).

## Tasks

### Task 1: One resume rule, in `parse` (finding 1)

- **Files:** crates/terminus-core/src/parse/mod.rs (`Scan::consumed`, `scan_from`), crates/terminus-core/src/ingest/mod.rs (`prepare`)
- **Action:** Add one public function to `parse`, beside `scan` and `scan_from`. It returns how many leading bytes of a byte slice end at the slice's last `\n`, or 0 when the slice holds no `\n`. This is the D-14 resume rule the module comment states. Give it a doc comment that says so. `scan_from` sets `resume_offset` to `start_offset` plus that function's answer over `bytes`, and its loop stops tracking its own `resume` index. The loop still splits and parses lines exactly as it does now. In `prepare`, `consumed` comes from the new function over `tail` in place of the inline `tail.iter().rposition(...)`. The comment above that line says the resume point comes from `parse`'s function, and it keeps the rest of what it says now: the watermark is a file offset, so it is measured against the file and not the stored bytes, and `consumed` is what `runs.bytes_read` and `status` report. Delete `Scan::consumed`, which nothing calls. Do not edit `recover.rs`, which calls `scan_from` and gets the same `resume_offset` (criterion 3). Leave the `rposition` in the unit test `a_file_that_grows_after_its_check_is_left_for_the_next_pass` alone (skip 18).
- **Verify:** `grep -rn "consumed(" crates` prints nothing. `grep -n "rposition" crates/terminus-core/src/ingest/mod.rs` prints only the line inside `mod tests`. `grep -n "let mut resume" crates/terminus-core/src/parse/mod.rs` prints nothing. `cargo test -p terminus-core --all-features --test parse --test ingest --test pass --test recover --test capture --test compaction` reports 0 failed.

### Task 2: No stat where its answer is not read (finding 3)

- **Files:** crates/terminus-core/src/ingest/mod.rs (`Existing::read`)
- **Action:** In `Existing::read`, run the `std::fs::metadata(path)` length check only when its answer can change the result, which is for a session that is neither evicted nor `whole`. Fold the check into the condition that decides `fetched`, after `!evicted` and after `whole`, so the condition short-circuits before the stat for an evicted or a `whole` session. Keep the length test exactly as it is (`is_ok_and(|m| m.len() > watermark)`), so a failed stat still counts as not grown, and keep it after `watermark` is resolved. The D-08 comment above it keeps every reason it gives now: refusals first, a stat and not an open because opens are counted, no fetch for a file no longer than its watermark or a failed stat, and no rows for an evicted session. Add that the stat is skipped where its answer would not be read. It stays a stat (phase 4 D-08, skip 2).
- **Verify:** In `Existing::read`, the one `std::fs::metadata(path)` call sits inside the `fetched` condition, to the right of both `!evicted` and `whole` in a short-circuiting `&&`/`||` expression, and no separate `let grown` statement runs it before that condition. `cargo test -p terminus-core --all-features --lib a_file_that_grows_after_its_check_is_left_for_the_next_pass` passes. `cargo test -p terminus-core --all-features --test pass --test ingest --test backfill --test retention` reports 0 failed.

### Task 3: `Existing::read` reads its row once (finding 5)

- **Files:** crates/terminus-core/src/ingest/mod.rs (`Row`, `read_rows`, `Holder::of`, `claim`, `ingest_locked`, `Existing::read`, `mod tests`), crates/terminus-core/src/ingest/backfill.rs (`read_existing`)
- **Action:** Five changes, all from finding 5.
  - `Row` derives `Default`. In `Existing::read`, the eight per-field reads (`watermark`, `agent_meta_stored`, `diverged`, `evicted`, `uncompressed_len`, `stored_capture_mode`, `block_size`, `transcript_path`) all go through one fallback: the row when there is one, `Row::default()` when there is not. Each value must equal what its own default gives today: no watermark, every flag false, a length of 0 after the existing `.max(0)` clamp, and no capture mode, block size or path.
  - Build the session in one step, from whether the row exists and never from the fallback. No row means no session. A row with no checksum is the existing `Error::NotAStore` refusal with its `path` and `detail` text unchanged. A row with a checksum becomes `ExistingSession` after the existing 32-byte conversion, which fails with the same `Error::BlobChecksumMismatch` `expected` and `actual` text. The two `match`es become one. The checksum check may now run before `Holder::of`. That changes only which error comes first when both could apply, and `Holder::of` can return no error. Its only other effect is two `canonicalize` reads.
  - `Holder::of` takes the row (`Option<&Row>`) and the path, and reads the stored `transcript_path` off the row. Its `archived` argument goes, because it is true whenever it is read: a stored path exists only when there is a row, and `Existing::read` has already refused a row with no checksum. The comparison is unchanged: string equality, then `canonicalize`, then `NotFound` as `Gone`. `claim` calls it with `rows.get(&session_key)`, and `Existing::read` calls it with its row.
  - `Existing::read` takes the `Rows` map in place of `Option<&Row>` and does the `session_key` lookup itself. Its four callers pass the map: `ingest_locked`, `backfill::read_existing`, and the two calls in the unit test `a_file_that_grows_after_its_check_is_left_for_the_next_pass`. The unit test changes at those two call sites and nowhere else. No assertion in it changes. Update `Existing::read`'s doc comment for the new argument.
  - `read_rows` stops binding a `key` that is only `only` again. It picks the SQL from `only` and passes `only` to `params_from_iter`.
- **Verify:** `grep -rn "Existing::read(" crates/terminus-core/src` shows no call passing `rows.get(`. `grep -rn "rows.get(" crates/terminus-core/src/ingest` prints exactly two lines, one in `claim` and one in `Existing::read`. `grep -n "fn of(" crates/terminus-core/src/ingest/mod.rs` shows two parameters. `git diff HEAD -- crates/terminus-core/src/ingest | grep -E '^[-+][^-+]' | grep assert` prints nothing. `cargo test -p terminus-core --all-features --lib ingest::` passes. `cargo test -p terminus-core --all-features --test ingest --test pass --test backfill --test session_keys --test recover --test retention` reports 0 failed.

### Task 4: The snapshot gate returns a `Result` (finding 8)

- **Files:** crates/terminus-core/src/ingest/pass.rs (`Gate`, `due`, `locked`, `run_with`)
- **Action:** Delete `Gate`. `due` returns `std::result::Result<Option<i64>, String>`. Write the std type out in full, because the module imports `crate::error::Result`, which takes one argument. `due` returns `Ok(None)` where it returned `Gate::NotDue` and `Ok(Some(now))` where it returned `Gate::Due { now }`. Each of the three `Gate::Note` arms returns `Err` carrying only the failure's own text (`e.to_string()`), with no prefix. `locked` returns that `Result` in its tuple. On the lock-held path it returns `Ok(None)`, which `run_with` never reads because it returns `PassOutcome::LockHeld` first. `run_with` maps `Ok(None)` to `Snapshotted::default()` and `Ok(Some(now))` to `roll(data_dir, config, now)`. It maps `Err(detail)` to a `Snapshotted` whose one note is `no snapshot was taken: ` followed by the detail, so every note is byte for byte what it is now. Leave `roll`'s own `no snapshot was taken: {e}` for a failed `Store::open` as it is, because it is not one of the gate's three. Update the doc comments. `due`'s "A failed read is a note" now says the failure comes back as `Err` and `run_with` writes the note. Check that `run_with`'s and `locked`'s mentions of the gate are still true. Phase 4 D-15 is unchanged: the gate is answered on the pass's own store inside the lock, and a writable open after release happens only when a snapshot is due.
- **Verify:** `grep -n "Gate" crates/terminus-core/src/ingest/pass.rs` prints nothing. `grep -c "no snapshot was taken" crates/terminus-core/src/ingest/pass.rs` prints 2. `cargo test -p terminus-core --all-features --test snapshot --test pass --test config` reports 0 failed. Then, for the plan as a whole: `cargo test --workspace --all-features --no-fail-fast` reports 0 failed. `git diff --name-only <the commit this plan started from>` lists only this plan's four frontmatter files. `git diff <that commit> -- crates | grep -E '^[-+][^-+]' | grep assert` prints nothing. `cargo fmt --all --check` names none of the four files.

## Notes

- Criterion 3 and finding 1: `parse/mod.rs` is not in `git diff main...traders`. Finding 1 names it, and John adjudicated finding 1 as Apply. `Scan::consumed` lost its last caller in the traders diff, and the diff's `rposition` copied `scan_from`'s rule. So this is code the diff made redundant, not unrelated code like skip 4's `recover.rs` change. It is planned as written.
- Criterion 2 says finding 16 is "the one test change". Finding 5 itself lists the unit test's two `Existing::read` calls (`ingest/mod.rs:1689`, `:1716`) among the four callers to change. So Task 3 edits those two call sites in `mod tests` and touches no assertion. This is the only test-code change in this plan.
- Finding 8: the lock-held path still returns a value, `Ok(None)`, because `locked` keeps its tuple. The finding's fix is the `Result` type and the single prefix. Restructuring `locked` so the lock-held path carries no gate at all is not part of it, so it is not planned.
- Prior art: phase 4's SUMMARY records a race in `read_tail` and `open_at_least` that predates the phase (recalled from `phases/4/SUMMARY.md`, phase 4). No task here touches either function, so that race is unchanged.
- Line numbers in SIMPLIFY.md were checked against `traders` at 4c078c8 and all match: `ingest/mod.rs:407`, `:1173`, `:1116-1163`, `:1007`, `:209`, `:1143`, `:256`, `:1689`, `:1716`, `:1076-1086`, `backfill.rs:432`, `parse/mod.rs:44`, `:61-88`, `pass.rs:150-218`.
