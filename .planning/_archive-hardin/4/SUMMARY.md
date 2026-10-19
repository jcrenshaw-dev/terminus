---
phase: 4
status: complete
completed: 2026-10-19
---

# Phase 4: Session keys - Summary

A session is keyed on its transcript's path below its project folder, so moving `.claude`, reaching it through a symlink, or moving a project folder keeps every key. The first run after upgrading re-keys the old absolute-path sessions and merges each provable twin into the older session, and a pass over an unchanged file records where it now lives, so retention never deletes a moved session (#57).

## What shipped

- The key is the path below the project folder (`<uuid>.jsonl`, or `<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar), found from the shape of the path, with a last-component link keyed on its target - `path_key` and `below_project_folder` in `crates/terminus-core/src/ingest/mod.rs`
- Each key is ingested once per walk, and a second file under a held key is refused, unread, in the pass, backfill and single-file ingest - `ingest::claim`, `ingest::run_with`
- The repair runs inside recovery in one transaction: drain waiting decision files, re-key every unpaired session in place, merge each proven twin into the oldest session, report and keep every pair that is not provably one stream, then point labels and decision documents at the survivor - `crates/terminus-core/src/recover.rs`
- `delete_session` factored out of retention's delete, so the merge deletes a twin the way retention does - `crates/terminus-core/src/retention/apply.rs`
- A pass that reaches an archived session and writes no turn updates `session_meta.transcript_path` when the file now lives somewhere else, and writes nothing when it has not moved - `Existing::read`, `Prepared::moved` and `apply` in `crates/terminus-core/src/ingest/mod.rs`
- Regression tests: `a_moved_claude_dir_behind_a_symlink_archives_nothing_twice` and `a_moved_project_folder_archives_nothing_twice` (`tests/pass.rs`), `a_second_roots_file_never_appends_onto_the_first_roots_session` (`tests/ingest.rs`), the planted-store tests `one_pass_repairs_a_store_keyed_on_absolute_paths` and `one_pass_merges_the_copies_a_moved_project_folder_left` with `recovery_with_nothing_to_repair_leaves_decision_files_waiting` (`tests/session_keys.rs`, testkit), and `a_moved_claude_dir_keeps_every_session_whose_transcript_is_still_on_disk` (`tests/retention.rs`)
- The docs that define the key describe the new one: `THREAT-MODEL.md`, `docs/design/0003-ingest.md`, `docs/design/0004-archive-and-block-rows.md`, `docs/json-shapes.md`, and a `CHANGELOG.md` `[Unreleased]` `### Fixed` entry for #57 that records the `session_key` output change, the moved-project-folder repair and the retention fix

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
| 4 | 1 | b48c8c3 | Key a session on its path below the project folder |
| 4 | 2 | ebf7b65 | Prove a moved project folder, and the copies one left in the archive, are one session |
| 4 | 3 | 95411fa | Record where an unchanged file was read from |
| 4 | 4 | a24a04b | Describe the session key as the path below the project folder |

## Deviations

- [deviation] Plan 1: Task 1's Verify says `cargo fmt --check` is clean, but it already failed at the plan's base on four files outside the lease (`crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `release_workflows.rs`, `sign_release.rs`). Every file the plan changed is rustfmt-clean; the four were left alone. (6fe9190)
- [deviation] Plan 2: the plan said a second run's recovery report is empty, and also that every pair that cannot merge is reported again on every run, while Task 2's failing pairs live in the same test store. The executor followed the Action: the second run changes no row, and its `runs.error` holds only the same `not merged into` lines. On a store with no unmergeable twin the second run reports nothing. (3ece63e)
- [deviation] Plan 3: the fallback sentence D-01 requires (a file under no `projects` folder keeps its canonical path) is the only match left by Task 1's pattern search, which said nothing should match. The sentence was kept. (0f29f3d)
- [deviation] Plan 3: `THREAT-MODEL.md:17` said the session is keyed by the link's path, a line D-15 did not name. It now says the session is keyed by the file the link points at. Figure 3 in 0003 said recovery removes a deleted session's watermark; it now says retention deletes the session and its watermark. (0f29f3d)
- [deviation] Plan 2's `risk_surface` review raised three findings, all `high`. Two were refuted against the code and one was downgraded to low; all three were declined, so nothing was filed. Recorded in `ADJUDICATION-risk_surface-plan-2.json` and `.planning/DECLINED.md`.
- Plan 4 ran as written, with no deviation.

## Open items

- AC4 has not been run: on a fresh copy of the live store, one ingest pass with this branch's release build, then the checks in PLAN-4's Notes (each transcript archived once, the `388a000f` orphan still session 2000 with 857,565 bytes, `terminus verify` clean, no dangling label or decision turn id). This is the check against the real archive.
- John reads the docs plan 3 and plan 4 changed in RustRover (plan 3 Task 1, plan 4 Task 4).
- CONTEXT D-01 still states the key as `<encoded project dir>/<uuid>.jsonl`, and D-13 still says ingest overwrites `transcript_path` on every pass. John's ruling overrode D-01 and plan 4 Task 3 made D-13's claim true, but neither line is annotated.
- AC7 at landing: the pull request carries `Fixes #57`, and #57 closes once it merges.
- `cargo fmt --check` fails on four files outside this phase, predating it.
- A longer twin with no `watermarks` row leaves the survivor's own, shorter watermark. No writer produces that state; it was the downgraded review finding.
- A merged twin's `observations` row is deleted with it, not carried; `terminus observations regenerate` rebuilds it from the survivor's stream.
- Decisions recovery drains are not counted anywhere; only a discard or a failed drain is reported.
- Single-file `terminus ingest <path>` drops recovery's report (`ingest/mod.rs:119`), so an unmergeable twin or failed drain is named only by the next pass or backfill. Predates this phase.
- `docs/design/0003-ingest.md:93` and `:136` do not mention plan 1's refusal of a second file that reduces to a held key.
- The Mermaid figures in 0003 were not rendered; no Mermaid CLI is installed.

## Goal check

The commits plausibly deliver the goal, including the two gaps UAT found. The key no longer depends on the root or the project folder (b48c8c3, `path_key` at `crates/terminus-core/src/ingest/mod.rs:1112`). `a_moved_project_folder_archives_nothing_twice` (`tests/pass.rs:927`) failed at 742e352 with the session count moving from 2 to 4 and passes now, beside the earlier symlink test at `tests/pass.rs:848`. `one_pass_merges_the_copies_a_moved_project_folder_left` (`tests/session_keys.rs:719`) plants UAT item 5's shape, including the prefix pair whose gone copy is shorter and a `wf_*` sidecar pair, and failed at 742e352 with 6 sessions where there should be 3. `a_moved_claude_dir_keeps_every_session_whose_transcript_is_still_on_disk` (`tests/retention.rs:817`) closes UAT item 10: at 742e352 retention deleted every row of the moved session, and now the rows stay and `transcript_path` names the new place (95411fa). The full suite passed 979 of 979 after the last plan (plan 4 report). What is still unproven is the real archive: AC4 against a fresh copy of the live store has not been run, so the repair of the 175 doubled `-code-cadence` groups is shown only by the planning prototype in PLAN-4's Notes, not by this build.
