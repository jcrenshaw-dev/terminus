---
phase: 1
status: complete
completed: 2026-10-22
---

# Phase 1: Old store code - Summary

Recovery is down to the store open, the derived-table catch-up and the watermark sweep. The store open brings no column forward, drops no summary column and retires no pre-5 boundary table. The read side no longer checks for columns or tables that only older layouts lacked (#105).

## What shipped

- The re-key and the twin merge are removed from recovery. `Recovered` keeps `lowered`, `removed` and `duration`. - `crates/terminus-core/src/recover.rs`
- The re-key's tests and `testkit::plant_old_key` are deleted. - `crates/terminus-core/tests/session_keys.rs` (deleted), `crates/terminus-core/src/testkit.rs`
- `BRING_FORWARD_COLUMNS` is an empty slice. The store open still creates a table the store lacks, and eight concurrent opens of such a store all succeed. - `crates/terminus-core/src/store/schema.rs:441`, `crates/terminus-core/src/store/open.rs`, `crates/terminus-core/tests/store_open.rs`
- `drop_summary_columns`, both `DROPPED_` constants and `retire_old_boundaries` are deleted, along with the tests that aged a store by dropping a listed column. - `crates/terminus-core/src/store/schema.rs`, `crates/terminus-core/src/reindex.rs`, `crates/terminus-core/tests/`
- Reads name the pre-worktree column directly, and the decision log's columns are read without checking for them. A store missing the whole decision log is reported as damaged. A missing observations table fails as SQLite reports it. `--kind observation` is accepted like any other kind. The tool-result filter is folded into the search's own rows. - `crates/terminus-core/src/recall/`, `crates/terminus-core/src/feedback/`, `crates/terminus/src/cmd/`
- The docs and threat model no longer describe a re-key, a column bring-forward or the removed refusals. - `docs/design/0003-ingest.md`, `docs/design/0004-archive-and-block-rows.md`, `docs/design/0005-search.md`, `docs/design/0007-injection.md`, `docs/commands.md`, `docs/json-shapes.md`, `THREAT-MODEL.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 7d8fed3 | Delete the re-key's tests and their helper |
| 1 | 2 | b6cbbc8 | Remove the re-key and the twin merge from recovery |
| 1 | 3 | 43593fb | Bring the docs and threat model up to date on recovery |
| 2 | 1 | 14b3ea7 | Pin the store open's write-lock race through a missing table |
| 2 | 2 | 4178389 | Cut the store and schema tests that drop a listed column |
| 2 | 3 | e1e6913 | Delete the read-path tests that drop a listed column |
| 2 | 4 | ff5b7f1 | Age a test store by its derived schema alone |
| 2 | 5 | 71101c0 | Empty the bring-forward column list |
| 2 | 6 | b4d12b9 | Delete the summary-column drop and its test |
| 2 | 7 | 3b78acc | Delete the old boundary table's retirement and its tests |
| 2 | 8 | 18e61a0 | Bring the archive design doc up to date on the store open |
| 3 | 1 | 08f29d8 | Name the pre-worktree column directly in every read |
| 3 | 2 | ee3cfd1 | Read the decision log's columns without checking for them |
| 3 | 3 | cf91c83 | Treat a store missing the whole decision log as damaged |
| 3 | 4 | 18689f5 | Let a missing observations table fail as SQLite reports it |
| 3 | 5 | 13ef897 | Accept --kind observation like any other kind |
| 3 | 6 | 7e496dd | Fold the tool-result filter into the search's own rows |
| 3 | 7 | 71b3fd4 | Bring the read-side docs up to date |

## Deviations

- [deviation] Plan 1 task 1's Verify asserted that `grep -rn 'session_keys\|plant_old_key' crates/` prints nothing. It prints two lines from `crates/terminus-core/tests/snapshot.rs:89,151`, a local `fn session_keys` that lists session keys for the snapshot test and has nothing to do with the re-key. The file is outside the plan's lease and the acceptance criterion does not search for `session_keys`, so it was left alone. (7d8fed3)

## Open items

- `testkit::recut` in `crates/terminus-core/src/testkit.rs` has no caller now that `session_keys.rs` is gone. It was kept as plan 1's Notes say, for John to decide.
- `docs/design/0006-recall-through-mcp.md:127` and `:144` still describe the `--kind observation` refusal that plan 3 task 5 removed. The file was outside plan 3's lease.
- The doc comment on `stats_and_replay_call_half_a_decision_log_a_failure` in `crates/terminus/tests/replay.rs` still says "missing both" is the only shape age can produce. It was left because plan 3 asks for that test unchanged.
- Success criterion 3 (an ingest pass on a copy of the live store completes with no error, and `terminus verify` reports every block intact) is human-verify on John's machine and was not run. The commands are in PLAN-3.md's Notes.
- `cargo fmt --check` reports diffs in files no plan touched (`crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `release_workflows.rs`, `sign_release.rs`, `observe/egress.rs`). They predate this phase. Every file the phase touched passes rustfmt.
- `cargo doc` reports 30 warnings that predate this phase (links to private items), including `MissingColumn`'s link to the private `bring_forward` in `crates/terminus-core/src/store/open.rs:70`. The phase added none.

## Goal check

The commits deliver the goal on every criterion that can be checked from the tree. `grep -ciE 'rekey|stale_keys|merge|remap_ids' crates/terminus-core/src/recover.rs` prints 0, and `crates/terminus-core/tests/session_keys.rs` no longer exists (criterion 1). `BRING_FORWARD_COLUMNS` is `&[]` at `crates/terminus-core/src/store/schema.rs:441`, and `grep -rnE 'drop_summary_columns|DROPPED_OBSERVATION_COLUMNS|retire_old_boundaries' crates/` matches nothing (criterion 2). `grep -niE '\bre-?key'` over the four docs that criterion 4 names matches nothing. The full suite passed after each plan, with 1015, 1004 and 1002 tests in turn as the deleted tests fell away (plan reports 1, 2 and 3). Each plan's risk review passed with no surviving finding. Criterion 3, the ingest pass and `terminus verify` on a copy of the live store, has not been run: it needs John's store and is the one gap left for `/cad-verify 1`. The cross-model risk reviews read diffs that Cadence's outbound filter had partly blanked (47, 12 and 7 spans, including ordinary lines such as `use rusqlite::Connection;`), so they are weaker evidence than their clean results suggest.
