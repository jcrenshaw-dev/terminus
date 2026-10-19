---
phase: 4
status: complete
completed: 2026-10-18
---

# Phase 4: Session keys - Summary

A session is keyed on its transcript's path below `projects`, so a moved or symlinked `.claude` keeps every key, and the first run after upgrading re-keys the old absolute-path sessions and merges each provable twin into the older session (#57).

## What shipped

- The key is the path below `projects`, found from the shape of the path, with a last-component link keyed on its target - `path_key` in `crates/terminus-core/src/ingest/mod.rs`
- Each key is ingested once per walk, and a second root's different file under the same key is refused, unread, in the pass, backfill and single-file ingest - `ingest::claim`, `ingest::run_with`
- The repair runs inside recovery in one transaction: drain waiting decision files, re-key every unpaired session in place, merge each proven twin into the oldest session, report and keep every pair that is not provably one stream, then point labels and decision documents at the survivor - `crates/terminus-core/src/recover.rs`
- `delete_session` factored out of retention's delete, so the merge deletes a twin the way retention does - `crates/terminus-core/src/retention/apply.rs`
- Regression tests: `a_moved_claude_dir_behind_a_symlink_archives_nothing_twice` (`tests/pass.rs`), `a_second_roots_file_never_appends_onto_the_first_roots_session` (`tests/ingest.rs`), and the planted-store test `one_pass_repairs_a_store_keyed_on_absolute_paths` with `recovery_with_nothing_to_repair_leaves_decision_files_waiting` (`tests/session_keys.rs`, testkit)
- The docs that define the key describe the new one: `THREAT-MODEL.md`, `docs/design/0003-ingest.md`, `docs/design/0004-archive-and-block-rows.md`, `docs/json-shapes.md`, and a `CHANGELOG.md` `[Unreleased]` `### Fixed` entry for #57 that records the `session_key` output change

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 6fe9190 | Key every session on its path below `projects` |
| 1 | 2 | 45420d8 | Ingest each session key once per walk |
| 1 | 3 | 66b5e1f | Prove a moved `.claude` behind a symlink archives nothing twice |
| 2 | 1 | 1685971 | Re-key every unpaired session inside recovery |
| 2 | 2 | fb08cdf | Report the twins that are not provably one stream |
| 2 | 3 | 3ece63e | Merge each proven twin into the older session |
| 2 | 4 | 0c2fa88 | Point labels and decisions at the surviving session |
| 3 | 1 | 0f29f3d | Describe the session key as the path below `projects`, and the repair recovery makes |
| 3 | 2 | 07f8ecd | Record the doubled archive and the new `session_key` in the changelog |

## Deviations

- [deviation] Plan 1: Task 1's Verify says `cargo fmt --check` is clean, but it already failed at the plan's base on four files outside the lease (`crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `release_workflows.rs`, `sign_release.rs`). Every file the plan changed is rustfmt-clean; the four were left alone. (6fe9190)
- [deviation] Plan 2: the plan said a second run's recovery report is empty, and also that every pair that cannot merge is reported again on every run, while Task 2's failing pairs live in the same test store. The executor followed the Action: the second run changes no row, and its `runs.error` holds only the same `not merged into` lines. On a store with no unmergeable twin the second run reports nothing. (3ece63e)
- [deviation] Plan 3: the fallback sentence D-01 requires (a file under no `projects` folder keeps its canonical path) is the only match left by Task 1's pattern search, which said nothing should match. The sentence was kept. (0f29f3d)
- [deviation] Plan 3: `THREAT-MODEL.md:17` said the session is keyed by the link's path, a line D-15 did not name. It now says the session is keyed by the file the link points at. Figure 3 in 0003 said recovery removes a deleted session's watermark; it now says retention deletes the session and its watermark. (0f29f3d)
- [deviation] Plan 2's `risk_surface` review raised three findings, all `high`. Two were refuted against the code and one was downgraded to low; all three were declined, so nothing was filed. Recorded in `ADJUDICATION-risk_surface-plan-2.json` and `.planning/DECLINED.md`.

## Open items

- AC4 has not been run: on a copy of the live store, one ingest pass with the new binary, then check that `recall_search` returns each turn once, `terminus status` counts each session once, the `-code-cadence/388a000f` orphan is still archived, and no label or decision turn id dangles. This is the check against the real 2,733 transcripts.
- AC7 at landing: the pull request carries `Fixes #57`, and #57 closes once it merges.
- John reads each changed doc in RustRover (plan 3, Task 1).
- `cargo fmt --check` fails on four files outside this phase, predating it.
- A longer twin with no `watermarks` row leaves the survivor's own, shorter watermark. No writer produces that state; it was the downgraded review finding.
- A merged twin's `observations` row is deleted with it, not carried; `terminus observations regenerate` rebuilds it from the survivor's stream.
- Decisions recovery drains are not counted anywhere; only a discard or a failed drain is reported.
- Single-file `terminus ingest <path>` drops recovery's report (`ingest/mod.rs:119`), so an unmergeable twin or failed drain is named only by the next pass or backfill. Predates this phase.
- `docs/design/0003-ingest.md:93` and `:136` do not mention plan 1's refusal of a second file that reduces to a held key.
- The Mermaid figures in 0003 were not rendered; no Mermaid CLI is installed.

## Goal check

The commits plausibly deliver the goal. The key no longer depends on the root (6fe9190), and `a_moved_claude_dir_behind_a_symlink_archives_nothing_twice` (`tests/pass.rs:848`) fails at the pre-phase tree with the session count moving from 3 to 6 and passes now. The repair re-keys in place and merges only proven twins into the oldest session, keeping every archive-only session and both halves of any pair it cannot prove (1685971, fb08cdf, 3ece63e), and the planted-store test in `tests/session_keys.rs:340` checks block rows, checksums, turn ids, watermarks, `verify` and an empty `foreign_key_check`, failing at the pre-phase tree at 15 sessions against 11. Labels and decision documents follow the survivor (0c2fa88). The full suite passed 976 of 976 after the last plan. Two things are unproven here: success criterion 3 (`recall_search` once, `status` once) is shown only by plan 1's by-hand check on a small tree, and AC4 against a copy of the live store has not been run, so the real archive's repair is still untested.
