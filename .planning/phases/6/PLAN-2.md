---
phase: 6
plan: 2
requirements: [ARCV-02]
files:
  - crates/terminus-core/src/reindex.rs
  - crates/terminus-core/src/retention/apply.rs
  - crates/terminus-core/src/retention/mod.rs
  - crates/terminus-core/tests/retention.rs
  - crates/terminus/tests/lifecycle.rs
---

# Phase 6: Archive - Plan 2 of 3 (retention's delete)

## Goal

Retention re-checks before it deletes (#43). A transcript that comes back between selection and delete keeps its session archived. The derived-table delete list is written once, in `reindex::clear_derived`.

## Must be true when done

- Take a session `retention::evaluate` selected for deletion, whose transcript is back on disk when `retention::apply` reaches it. It keeps every row it owns, `apply` deletes nothing, and the kept session is counted in `Applied::notes()`, which goes to `runs.notes`. It never appears in `Applied::errors()`, which goes to `runs.error`, and never in `deleted`.
- The check reads that session's stored `session_meta.transcript_path`, not its key. It runs once per session, inside that session's transaction, just before its delete. Only `NotFound` counts as gone.
- `crates/terminus-core/src/retention/apply.rs` has no `DELETE FROM` for `paths`, `entities`, `compactions`, `turns_fts` or `turns`. `delete_session` calls `reindex::clear_derived`, and the existing deletion tests pass with no assertion changed.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-07 to D-12 and D-17's retention and `reindex` half from `.planning/phases/6/CONTEXT.md`.
  - `Selection.delete` keeps carrying keys only.
  - The check is per session, inside the loop.
  - A kept session is a note, never an error.
  - `delete_session` calls `clear_derived`, which becomes `pub(crate)`.
  - D-11 adds no new test. D-10 adds one.
- Out of scope (CONTEXT): the user docs, and `docs/commands.md`. CONTEXT flagged that file as possibly listing the notes `status` prints. It does not: its `status` entry names only recovery's lines.
- Patterns: `Applied::errors()` and `Applied::notes()` already split what failed from what a healthy pass did (#96). `Bench::archive`, `Bench::rows`, `Bench::matches` and `Bench::remove_transcript` in `tests/retention.rs` are the fixtures to reuse.
- Comments and tests cite `#43` only, never a requirement id or decision tag (`crates/terminus-core/tests/planning_tags.rs`). Rewrap comments by hand. Do not run `cargo fmt`.

## Tasks

### Task 1: Clear a deleted session's derived rows through `reindex::clear_derived`

- **Files:** crates/terminus-core/src/reindex.rs (module doc, `clear_derived`), crates/terminus-core/src/retention/apply.rs (`delete_one`'s doc, `delete_session`)
- **Action:**
  - **The code (D-11).**
    - `clear_derived` becomes `pub(crate)`, with its body unchanged: the same five statements in the same order.
    - In `delete_session`, the five-statement loop and its `TURN_IDS` constant give way to one call to `crate::reindex::clear_derived(tx, session_key)`.
    - After that call, the rest is unchanged: the `observations` delete whose count is returned, then `session_meta`, `archive::remove`, `sessions` and the watermark, in that order.
  - **Docs (D-17).**
    - `clear_derived`'s doc names both callers: the preserving rebuild and retention's delete. Keep its child-first reason and its reason for clearing `compactions` by `session_key`.
    - The `reindex.rs` module doc says the per-session clear the preserving path uses is also the derived half of retention's delete, so the child-first list of derived tables is written once. Its statement that `sessions` and `session_meta` are never touched is about `reindex` itself, so keep it true.
    - `delete_one`'s "Child first" paragraph points to `reindex::clear_derived` for the derived tables and their order, instead of restating them. It keeps only what is retention's own: `observations` and `session_meta` hang off `sessions(session_key)` and the block rows off `sessions(session_no)`, so they go after the derived rows and before `sessions`.
    - `delete_session`'s doc stops saying "in `delete_one`'s order" and says what is now true.
    - The `decisions` and `labels` paragraph and the watermark paragraph stay as they are.
- **Verify:**
  - `grep -nE 'DELETE FROM (paths|entities|compactions|turns_fts|turns)' crates/terminus-core/src/retention/apply.rs` prints nothing.
  - `grep -n 'clear_derived' crates/terminus-core/src/retention/apply.rs` shows the call in `delete_session`.
  - `grep -n 'pub(crate) fn clear_derived' crates/terminus-core/src/reindex.rs` prints one line.
  - `cargo test -p terminus-core --features testkit --test retention --test reindex` passes, with `git diff --exit-code -- crates/terminus-core/tests` exiting 0. The existing deletion tests are the proof (D-12).
  - `cargo test -p terminus --all-features --test lifecycle` passes.

### Task 2: Check the transcript again just before each delete

- **Files:** crates/terminus-core/src/retention/apply.rs (`Applied`, `Applied::is_silent`, `Applied::notes`, `apply`, `delete_one` and its doc), crates/terminus-core/src/retention/mod.rs (the `RetentionAction::Delete` arm's comment in `evaluate`, `transcript_is_gone`), crates/terminus-core/tests/retention.rs (`a_deletion_removes_every_row_the_session_owns`, `a_decision_naming_the_deleted_session_survives_it`, `a_failure_on_one_session_does_not_stop_the_next`, one new test), crates/terminus/tests/lifecycle.rs (`Bench::delete`)
- **Action:**
  - **The check (D-07, D-08).** Inside `delete_one`'s per-session transaction, before `delete_session`, read that session's `session_meta.transcript_path` and pass it to `super::transcript_is_gone`.
    - `transcript_is_gone` is already reachable from the child module. Do not widen its visibility or copy its `NotFound` rule.
    - If the transcript is gone, delete the session as today.
    - If it is not gone, write nothing for that session, end its transaction without changes, and let the loop go on.
    - If the read fails, that is an `Err`. A missing `session_meta` row counts as a failed read. `apply` already records an `Err` as `{session_key}: the deletion failed: {e}`, and that session is not deleted.
    - Use the stored path, never the key. After a moved `.claude` directory, the key names a path that is gone (`a_moved_claude_dir_keeps_every_session_whose_transcript_is_still_on_disk`).
    - Check per session, never once at the top of `apply`. A transcript that comes back partway through a batch of up to `MAX_PER_PASS` is still caught (D-08).
    - `Selection` does not change (D-07). `delete_session` stays the unconditional delete; the check belongs to retention's `apply` alone.
  - **Reporting (D-09).** `Applied` gains a count of the sessions the check kept.
    - A kept session is never in `deleted`, never adds to `observations_lost`, and never goes in `Applied.notes`. That field is what `errors()` returns for `runs.error`.
    - When the count is not zero, `Applied::notes()` gains one line. It says how many sessions were kept because their transcript was back on disk when the delete came, so they are still archived.
    - `is_silent` is false when the count is not zero.
    - `errors()` does not change.
  - **Docs (D-17).**
    - `delete_one`'s doc says that it reads the stored transcript path again and keeps a session whose transcript is back (#43), and why the check is per session. It also says the check narrows the window between selection and delete without closing it, because the ingest lock guards the store and not the transcript tree.
    - The comment in `evaluate`'s `RetentionAction::Delete` arm says `apply` asks again just before each delete.
  - **Existing tests.** Three tests in `tests/retention.rs` and the `Bench::delete` helper in `crates/terminus/tests/lifecycle.rs` apply hand-built delete selections to sessions whose transcript is still on disk. With the check, those sessions are kept.
    - The three tests are `a_deletion_removes_every_row_the_session_owns`, `a_decision_naming_the_deleted_session_survives_it` and `a_failure_on_one_session_does_not_stop_the_next`.
    - Give each one the precondition the product requires by removing the transcript before `apply`: `bench.remove_transcript(&key)` in `retention.rs`, and `std::fs::remove_file` on each placed path in the lifecycle helper.
    - The lifecycle helper's doc gains a line saying why. The tests after it only run `compact`, `sessions`, `status` and `usage`, never another ingest.
    - Change no assertion and no expected value (D-12). Without the removal, `a_decision_naming_the_deleted_session_survives_it` would pass without deleting anything.
  - **The new test (D-10, AC2).** Add it in `tests/retention.rs`.
    - **Setup.**
      - `bench.archive` a fixture, and record `bench.rows(&key)` and `bench.matches(testkit::UNIQUE_TOKEN)`.
      - Plant `is_final = 1` and `last_turn_at = ANCIENT` on its `session_meta` row. This is setup only; it is what `evaluate` reads.
      - Keep the transcript's bytes, then call `bench.remove_transcript(&key)`.
      - Run `retention::evaluate` with `config(DELETE_30)` at `NOW`, and assert that its `delete` is exactly that key. This is the test's premise.
      - Write the bytes back to the same path and run `bench.apply(&selection)`.
    - **Assertions.** `deleted` is empty, `notes` and `errors()` are empty, and `notes()` carries the kept line. `rows(&key)` and `matches(UNIQUE_TOKEN)` equal what they were before.
    - Comments cite #43 only.
- **Verify:**
  - **Retention tests.** `cargo test -p terminus-core --features testkit --test retention` passes, the new test included.
  - **Mutation check.** With `delete_one` changed to delete without the check, the new test fails. Put the check back.
  - **Callers.** `cargo test -p terminus --all-features --test lifecycle --test retention` passes, and so does `cargo test -p terminus-core --features testkit --test pass`.
  - **No assertion changed.** `git diff e8b4d8a -- crates/terminus-core/tests/retention.rs crates/terminus/tests/lifecycle.rs | grep -E '^-[^-].*assert'` prints nothing.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices**, all where CONTEXT left the call open:
  - The check sits inside `delete_one`'s transaction, so the path it reads and the delete it guards see the same row.
  - A path that cannot be read is a deletion failure (in `runs.error`), never a delete.
  - A kept session is counted, not listed by key, as D-09 says.
- **Beyond CONTEXT, and required by AC5:** `crates/terminus/tests/lifecycle.rs`'s `Bench::delete`, and three tests in `tests/retention.rs`, delete sessions whose transcript is still on disk. They need the transcript removed first. This changes setup only; D-07's "the hand-built selections do not change" and D-12's "no new test" both still hold.
- **Plan shape:** this is plan 2 of 3. It shares no file with plans 1 and 3, and none depends on another's behaviour.
