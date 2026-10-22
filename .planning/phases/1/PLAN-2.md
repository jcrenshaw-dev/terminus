---
phase: 1
plan: 2
requirements: [SHED-01]
files:
  - crates/terminus-core/tests/store_open.rs
  - crates/terminus-core/tests/schema.rs
  - crates/terminus-core/tests/reindex.rs
  - crates/terminus-core/tests/recall.rs
  - crates/terminus-core/src/inject/brief.rs
  - crates/terminus-core/tests/feedback.rs
  - crates/terminus/tests/replay.rs
  - crates/terminus-core/src/testkit.rs
  - crates/terminus/tests/cli.rs
  - crates/terminus-core/src/store/schema.rs
  - crates/terminus-core/src/store/open.rs
  - crates/terminus-core/src/reindex.rs
  - docs/design/0004-archive-and-block-rows.md
---

# Phase 1: Old store code - Plan 2 of 3 (store open and schema)

## Goal

Recovery and the store open carry no code for store layouts 1.0.0 can't create (#105). This plan covers the store open and the schema. No column is brought forward, no summary column is dropped, and no pre-5 boundary table is retired. A table this build declares is still created where a store lacks it.

## Must be true when done

- `BRING_FORWARD_COLUMNS` is an empty slice, and `grep -rnE 'drop_summary_columns|summary_columns|DROPPED_OBSERVATION_COLUMNS|DROPPED_META_KEY|retire_old_boundaries' crates/` matches no line.
- A store missing a table this build declares gains it on the next writable open. Eight processes opening such a store at once all succeed.
- A store whose `derived_schema` is one behind is still reported by a read-only open, rebuilt by `reindex::open_up_to_date`, and named by `terminus status` and `terminus verify` with "predate this build".
- No test in the workspace drops a column to age a store.
- `docs/design/0004-archive-and-block-rows.md` names no brought-forward column, no dropped observation column and no dropped `meta` key.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-05, D-06 and D-07, and the store-open half of D-13. Source: `.planning/phases/1/CONTEXT.md`, phase 1. `ARCHIVE_FORMAT` and `DERIVED_SCHEMA` do not change. `bring_forward` keeps its missing-table arm, and `missing`, `missing_columns`, `MissingColumn` and `Store::missing_tables` stay.
- Order inside this plan: first the tests that depend on a listed column coming back (tasks 1 to 4), then the code (tasks 5 to 7). Every task leaves `cargo test` green.
- The read-side branches that become unreachable once the list is empty (`pre_worktree_column`'s NULL arm, `replay`'s and `stats`' column checks) are removed in plan 3, not here.
- Runs after plan 1, which also edits `reindex.rs`, `testkit.rs` and `0004-archive-and-block-rows.md`. Plan 3 also edits `crates/terminus/tests/replay.rs`.

## Tasks

### Task 1: Test the store open's write-lock race through a missing table

- **Files:** crates/terminus-core/tests/store_open.rs (`concurrent_opens_of_an_older_store_all_bring_it_forward`)
- **Action:** The concurrent-open test ages its store by dropping `runs.notes`, a listed column, and once the list is empty nothing adds it back. Rewrite it to age the store by dropping a whole table this build declares, so `bring_forward`'s missing-table arm runs and the IMMEDIATE-transaction fix stays tested (D-06). Use a table no other table references, such as `labels`. Keep the rest: eight openers behind a barrier, twenty rounds, every open succeeds, and afterwards the dropped table is there. Rename the test and rewrite its doc comment to say what it checks: several opens of a store missing a declared table all succeed, and the table is created once.
- **Verify:** `grep -n 'DROP COLUMN' crates/terminus-core/tests/store_open.rs` no longer shows the `runs` line; `cargo test -p terminus-core --all-features --test store_open` passes; with `TransactionBehavior::Immediate` in `bring_forward` changed to `TransactionBehavior::Deferred` for the check alone, the rewritten test fails in at least one of three runs. Restore `Immediate` before committing. If the test never fails under `Deferred`, it does not pin the race: report that as a deviation, and do not weaken the check.

### Task 2: Cut the store and schema tests that age a store by dropping a listed column

- **Files:** crates/terminus-core/tests/store_open.rs (`a_read_only_open_of_an_older_store_reports_it_and_writes_nothing`), crates/terminus-core/tests/schema.rs (`aged_store`; `Aged`; `a_reindex_open_brings_every_phase_two_column_back`; `a_plain_open_brings_the_columns_back_and_fires_no_rebuild`; `a_brought_forward_session_meta_has_the_same_column_order_as_a_fresh_one`; `the_capture_mode_column_reaches_an_older_store_without_a_rebuild`), crates/terminus-core/tests/reindex.rs (`a_previous_builds_store_gains_is_typed_and_is_filled_from_the_blobs`)
- **Action:** D-06: a test that ages a store by dropping a listed column is deleted, or cut to its derived-schema half. In `store_open.rs`, cut the read-only older-store test to its derived-schema half. It still lowers `derived_schema`, and still checks that `rebuild_required` reports it, that `predates_this_build` is true, and that the read-only open leaves every byte as it was. Drop the column loop, the `missing_columns` assertions and the `project_pre_worktree` query that is expected to fail. In `schema.rs`, `aged_store` only lowers `derived_schema`, and `Aged`'s check keeps only its archive half: the same key, blob bytes and checksum. `a_reindex_open_brings_every_phase_two_column_back` and `a_plain_open_brings_the_columns_back_and_fires_no_rebuild` keep their derived-schema halves: the reindex open stamps the store forward, and the plain open does not rebuild and still reports a rebuild as required. The plain-open test loses the SELECTs that name phase 2 columns. Rename both to say what they now check. Delete `a_brought_forward_session_meta_has_the_same_column_order_as_a_fresh_one` and `the_capture_mode_column_reaches_an_older_store_without_a_rebuild`, which test only a dropped column coming back. In `reindex.rs`, delete `a_previous_builds_store_gains_is_typed_and_is_filled_from_the_blobs`: its premise is a dropped `turns.is_typed`, and the preserving rebuild has its own test (`a_reindex_carries_an_evicted_session_across_rather_than_losing_its_index`). Leave the summary-column test in `schema.rs` and the version-4 section of `reindex.rs` to tasks 6 and 7. Delete any helper or import these deletions leave unused, apart from the ones those two later tasks need. Leave `the_project_index_reaches_a_store_that_predates_it` and `the_observations_table_reaches_a_store_that_predates_it`, which test the rebuild and the missing-table arm.
- **Verify:** `grep -n 'DROP COLUMN\|BRING_FORWARD_COLUMNS {' crates/terminus-core/tests/store_open.rs crates/terminus-core/tests/schema.rs crates/terminus-core/tests/reindex.rs` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test store_open --test schema --test reindex` passes.

### Task 3: Delete the read-path tests that age a store by dropping a listed column

- **Files:** crates/terminus-core/tests/recall.rs (`a_store_predating_the_pre_worktree_column_still_answers_reads`), crates/terminus-core/src/inject/brief.rs (the `predating_store` test module), crates/terminus-core/tests/feedback.rs (`a_store_without_the_compaction_columns_gains_them_and_its_rows_read_unknown`; `replay_never_credits_a_tool_result_the_hook_refuses`), crates/terminus/tests/replay.rs (`replay_on_a_store_without_the_compaction_columns_still_answers`; `stats_on_a_store_without_the_compaction_columns_still_answers`)
- **Action:** D-06, read side. Delete the `recall.rs` test that drops every listed column and reads through the NULL arm. Delete the whole `predating_store` module at the end of `inject/brief.rs`, with the doc comment above it. In `feedback.rs`, delete `a_store_without_the_compaction_columns_gains_them_and_its_rows_read_unknown` and its `table_info` helper if nothing else uses it. Cut `replay_never_credits_a_tool_result_the_hook_refuses` before its last block, the one that drops `turns.is_typed` and replays the aged store, and drop the doc comment sentence about that store. The live-versus-replay half stays exactly as it is. In `crates/terminus/tests/replay.rs`, delete the two tests that drop `decisions.compacted` and `decisions.dropped`. Leave `stats_and_replay_on_a_store_without_the_decision_log_still_answer` for plan 3, which changes what it expects. Delete only imports these deletions leave unused. The library branches these tests reached stay until plan 3, so no library code changes here.
- **Verify:** `grep -n 'DROP COLUMN' crates/terminus-core/tests/recall.rs crates/terminus-core/tests/feedback.rs crates/terminus/tests/replay.rs crates/terminus-core/src/inject/brief.rs` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test recall --test feedback --lib` and `cargo test -p terminus --all-features --test replay` pass.

### Task 4: Age a test store by its derived schema alone

- **Files:** crates/terminus-core/src/testkit.rs (`age`), crates/terminus/tests/cli.rs (the doc comment on `verify_on_an_older_store_says_so_and_changes_nothing`)
- **Action:** D-07: `testkit::age` keeps only the `derived_schema` bump and stops dropping `decisions.compacted` and `decisions.dropped`. Its doc comment says the store is one `derived_schema` behind, and nothing about missing columns or a writable open adding them back. Rewrite the doc comment on the `cli.rs` test the same way. Its body does not change. The "derived tables predate this build" message its two callers check stays reachable in 1.x through `Store::predates_this_build`, so both tests keep their assertions.
- **Verify:** `grep -n 'DROP COLUMN' crates/terminus-core/src/testkit.rs` prints nothing; `cargo test -p terminus --all-features --test status status_on_an_older_store_says_so_and_changes_nothing` and `cargo test -p terminus --all-features --test cli verify_on_an_older_store_says_so_and_changes_nothing` each run one test and pass.

### Task 5: Empty the bring-forward column list

- **Files:** crates/terminus-core/src/store/schema.rs (`BRING_FORWARD_COLUMNS`; the four `CREATE_SQL` comments citing it), crates/terminus-core/src/store/open.rs (`bring_forward`; `Store::open_read_only` doc)
- **Action:** D-05: `BRING_FORWARD_COLUMNS` becomes an empty slice of the same type, and the comment on its `turns` entry goes with the entry. Its doc comment keeps the rule for any column added later (append it in both places, declared last, additive only) and says nothing about the columns that used to be listed. `v1.0.0`'s `CREATE_SQL` already declares all ten columns. In `CREATE_SQL`, the four comments that say a column is "appended last in `BRING_FORWARD_COLUMNS`" lose that claim, because those columns are no longer in it. In `open.rs`, `bring_forward`, `missing`, `missing_columns`, `MissingColumn`, `Store::missing_columns`, `Store::missing_tables` and `Store::predates_this_build` keep their code. Rewrite the `Store::open_read_only` doc paragraph that names `project_pre_worktree`, `agent_meta` and `transcript_diverged` as columns that may be absent, and the part of the `bring_forward` doc that explains the IMMEDIATE transaction with a missing column, so both are true of an empty list and a missing table. That race is the one task 1's test now pins. Do not touch `ARCHIVE_FORMAT`, `DERIVED_SCHEMA`, `TABLES` or the missing-table arm.
- **Verify:** `grep -n 'BRING_FORWARD_COLUMNS: ' crates/terminus-core/src/store/schema.rs` shows the declaration and `= &[];` on that same line; `grep -c 'appended last in .BRING_FORWARD_COLUMNS' crates/terminus-core/src/store/schema.rs` prints `0`; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test --workspace --all-features` passes.

### Task 6: Delete the summary-column drop and its test

- **Files:** crates/terminus-core/src/store/open.rs (`drop_summary_columns`; `summary_columns`; the call in `Store::open`; the `bring_forward` doc that links to `drop_summary_columns`), crates/terminus-core/src/store/schema.rs (`DROPPED_OBSERVATION_COLUMNS`; `DROPPED_META_KEY`), crates/terminus-core/tests/schema.rs (`a_store_with_the_summary_columns_opens_without_them`; `SUMMARY_ERA_OBSERVATIONS`; `mechanical_bytes`)
- **Action:** D-05: delete `drop_summary_columns`, `summary_columns`, their call in `Store::open`, `DROPPED_OBSERVATION_COLUMNS` and `DROPPED_META_KEY`, with their doc comments. The `bring_forward` doc links to `drop_summary_columns`; that link goes, and nothing in `open.rs` links to a deleted item. In `tests/schema.rs`, delete `a_store_with_the_summary_columns_opens_without_them` and the helpers only it used: `SUMMARY_ERA_OBSERVATIONS`, `mechanical_bytes`, and `column_layout` if task 2 left it with no other caller. A store with the summary columns predates 1.0.0. The archive format gate does not refuse it, but 1.0.0 never creates one, which is #105's line.
- **Verify:** `grep -rnE 'drop_summary_columns|summary_columns|DROPPED_OBSERVATION_COLUMNS|DROPPED_META_KEY|observe.daily_tokens' crates/` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test schema --test store_open` passes.

### Task 7: Delete the pre-5 boundary table's retirement and its tests

- **Files:** crates/terminus-core/src/reindex.rs (`retire_old_boundaries`; its call in `reindex`; the module docs section on retiring the old boundary table; the comment in the drop loop that names it), crates/terminus-core/tests/reindex.rs (the version-4 section: `into_version_4`; `has_table`; `boundary_rows`; `version_4_bench`; `a_version_4_store_moves_its_boundary_rows_to_compactions`; `a_version_4_store_carries_an_evicted_sessions_boundary_rows_across`)
- **Action:** D-05: delete `reindex::retire_old_boundaries` and its call. 1.0.0 has no `compaction_boundaries` table (`v1.0.0:crates/terminus-core/src/store/schema.rs`). Delete the module docs section about retiring the old boundary table and the two sentences in the drop loop's comment about the pre-5 table already being gone. The rest of the drop-order comment stays. In `tests/reindex.rs`, delete the version-4 section from its divider comment to the end of the file, with any helper only that section used. Do not touch `clear_derived`, the preserving path or `up_to_date`.
- **Verify:** `grep -rn 'retire_old_boundaries\|compaction_boundaries' crates/` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test reindex --test compaction` passes.

### Task 8: Bring the archive design doc up to date on the store open

- **Files:** docs/design/0004-archive-and-block-rows.md (the `store/open.rs` row of the file table; the bring-forward bullet)
- **Action:** D-13, store-open half. The `store/open.rs` row says the module creates a table the store lacks. It no longer says it brings an older store's tables and columns forward. The bullet that lists the brought-forward columns and the dropped observation columns says that every writable open of an existing store creates any table in `TABLES` the store lacks, and that a read-only open changes nothing and reports what is missing. It names no column, neither `DROPPED_` constant, no earlier build and no phase. The design doc describes the current design only.
- **Verify:** `grep -nE 'DROPPED_|agent_meta., .transcript_diverged|brought-forward|an older store' docs/design/0004-archive-and-block-rows.md` prints nothing; `grep -n 'TABLES' docs/design/0004-archive-and-block-rows.md` still shows the sentence about creating a missing table.

## Notes

- The read-only open of a store missing a column keeps `Store::missing_columns` and `MissingColumn` (D-05). With an empty list they always report nothing, and `predates_this_build` then rests on the version integers alone. That is the intended end state, not a gap.
- Task 1's verify asks the executor to break the fix on purpose and see the test fail. That is the only way to show the rewritten test still pins the race. The change is reverted inside the task. With `Deferred`, a second opener's read transaction can't upgrade to a write once another opener has created the table. SQLite does not call the busy handler for that upgrade, so the planner expects the old failure to come back for a missing table just as it did for a missing column.
- The store-open Outputs line in `0004-archive-and-block-rows.md` ("An existing one is brought forward as above") stays as it is. After task 8, "as above" points at the missing-table sentence.
