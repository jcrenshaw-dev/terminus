---
phase: 6
status: complete
completed: 2026-10-23
---

# Phase 6: Archive - Summary

A tree-pass sweep fills a NULL `session_meta.continues_from` once the predecessor is archived (#11), retention reads the stored transcript path again just before each delete and keeps a session whose transcript is back (#43), and the observe entity query drives from the session's turns with a test that fails if it goes back to the kind index (#70).

## What shipped

- One `parentUuid` lookup shared by ingest and the sweep (`parent_session_id`) - `crates/terminus-core/src/ingest/mod.rs`
- The continuation sweep (`fill_continuations`), first in `Tree::after_walk`, its failure one line in `runs.error` - `crates/terminus-core/src/ingest/mod.rs`, `pass.rs`, `runs.rs`, `driver.rs` (module doc)
- Two lineage tests: a link filled once the predecessor is archived, and a link already set survives the sweep - `crates/terminus-core/tests/lineage.rs`
- Retention's delete clears derived rows through `reindex::clear_derived`, now `pub(crate)`, so the list exists once - `crates/terminus-core/src/reindex.rs`, `crates/terminus-core/src/retention/apply.rs`
- The re-check before each delete and `Applied::kept`, reported through `Applied::notes()` - `crates/terminus-core/src/retention/apply.rs`, `retention/mod.rs`
- A test that a transcript coming back before the delete keeps its session - `crates/terminus-core/tests/retention.rs`, `crates/terminus/tests/lifecycle.rs`
- The entity query as `ENTITY_VALUES_SQL`, `turns t CROSS JOIN entities e`, with an EXPLAIN QUERY PLAN guard test - `crates/terminus-core/src/observe/mechanical.rs`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | a14a5c6 | Resolve a parentUuid predecessor in one function |
| 1 | 2 | db9ec3f | Fill a missing continuation link on every tree pass |
| 2 | 1 | a17b467 | Clear a deleted session's derived rows through reindex::clear_derived |
| 2 | 2 | f6a9e3d | Check the transcript again just before each retention delete |
| 3 | 1 | 575b906 | Drive the observe entity query from the session's turns |

The plans ran in parallel worktrees and merged into `mule` as 144a56c (plan 1, fast-forward), 52efd9d (plan 2) and 0260413 (plan 3).

## Deviations

None - plans executed as written.

## Open items

- #116 (Docs, sev: low): `Applied::kept`'s doc says kept sessions are "not in `notes`", while `Applied::notes()` prints a line for them. Filed from plan 2's risk review. (plan 2)
- Plan 2's risk review found the transcript can still come back between the re-check and the delete. That is the window D-08 accepted, and `delete_one`'s doc says so (`apply.rs:168-171`). Declined to `.planning/DECLINED.md`. (plan 2)
- Each executor ran `cargo test --workspace --all-features` green in its own worktree. The merged tree has not been run as one, since `workflow.test_command` is unset. (all plans)

## Goal check

The commits plausibly deliver the goal. Continuation links: `fill_continuations` (`ingest/mod.rs:1559`) runs first in `Tree::after_walk` (`pass.rs:345`), and its failure reaches `runs.error` through `runs.rs:84`. Plan 1's report says `a_continuing_session_is_linked_once_its_predecessor_is_archived` failed under its mutation check and `show --json` reported the predecessor in `context.continues_from` on a scratch store. Retention: `delete_one` reads `session_meta.transcript_path` in its own transaction and returns before any delete when `transcript_is_gone` is false (`apply.rs:195-202`). `delete_session` calls `reindex::clear_derived` (`apply.rs:212`), and `grep 'DELETE FROM (paths|entities|compactions|turns_fts|turns)'` over `apply.rs` prints nothing. The new test `a_transcript_that_comes_back_before_the_delete_keeps_its_session` failed with the check disabled, per plan 2's report. Observations: `entity_values` prepares `ENTITY_VALUES_SQL` (`mechanical.rs:252-271`), and the guard test `a_plan_that_starts_from_the_sessions_turns` (`mechanical.rs:469`) failed on the old kind-driven text, per plan 3's report. `git diff --stat ccb3bf2..HEAD -- crates/terminus-core/src/store crates/terminus-core/tests/observe.rs` prints nothing, so no index and no `DERIVED_SCHEMA` bump. The one gap is that the merged tree's suite has not been run as a whole.
