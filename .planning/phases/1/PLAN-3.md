---
phase: 1
plan: 3
requirements: [SHED-01]
files:
  - crates/terminus-core/src/config.rs
  - crates/terminus-core/src/recall/context.rs
  - crates/terminus-core/src/recall/get.rs
  - crates/terminus-core/src/retention/mod.rs
  - crates/terminus-core/src/feedback/replay.rs
  - crates/terminus-core/src/feedback/stats.rs
  - crates/terminus/src/cmd/read.rs
  - crates/terminus/src/cmd/stats.rs
  - crates/terminus/src/cmd/replay.rs
  - crates/terminus/tests/replay.rs
  - crates/terminus/src/cmd/observations.rs
  - crates/terminus/tests/observations.rs
  - crates/terminus/src/cmd/search.rs
  - crates/terminus/src/cmd/mcp/tools.rs
  - crates/terminus/tests/recall_cli.rs
  - crates/terminus/tests/mcp.rs
  - crates/terminus-core/src/recall/search.rs
  - crates/terminus-core/src/inject/prompt.rs
  - docs/design/0007-injection.md
  - docs/json-shapes.md
  - docs/design/0003-ingest.md
  - docs/commands.md
  - docs/design/0005-search.md
---

# Phase 1: Old store code - Plan 3 of 3 (read-side branches)

## Goal

Recovery and the store open carry no code for store layouts 1.0.0 can't create (#105). This plan covers the read side: no read command branches on a store missing a column or table that 1.0.0 creates, and a 1.0.0 store still opens, ingests and passes `terminus verify`.

## Must be true when done

- `config::visible::pre_worktree_column` no longer exists, `DecisionLog` has no `Absent` variant, and `quotable` runs no query of its own.
- On a temp store with both `decisions` and `labels` dropped, `terminus stats --json` and `terminus replay --json` each exit 1 and print a document with `ok: false`. On a temp store with `observations` dropped, `terminus observations` exits 1.
- `terminus search --kind observation x` exits 0 with no hits, and MCP `recall_search` with `kind: "observation"` returns empty hits with no `isError`.
- `docs/commands.md` and `docs/design/0005-search.md` no longer say `--kind observation` is refused, and no doc describes a store missing the decision log or the `observations` table as an empty answer.
- `cargo test --workspace --all-features` passes.
- (human-verify: needs the live store on John's machine) On a `VACUUM INTO` copy of the live store with `TERMINUS_DATA_DIR` pointed at it, this branch's release build runs `terminus ingest` with no error line, and `terminus verify` reports every block intact. The commands are in Notes.

## Context

- Locked: D-08 to D-12, the read-side half of D-13, and D-14. Source: `.planning/phases/1/CONTEXT.md`, phase 1. Deferred to phase 8 (#107), and not touched here: removing `DecisionLog::Damaged`, `read::unusable` and `read::damaged`.
- Runs after plan 2. Plan 2 already deleted the tests that reached these branches by dropping a column. That is why the branches can go here without breaking a test.
- `recall/search.rs` is edited again in phase 4 (SRCH-03), which runs after this phase on `mule`.
- Out of scope: any new check for a store damaged from outside Terminus, and the `--json` failure path for `observations` (phase 8).

## Tasks

### Task 1: Name the pre-worktree column directly in every read

- **Files:** crates/terminus-core/src/config.rs (`visible::pre_worktree_column`; `visible::sessions`; `visible::projects`), crates/terminus-core/src/recall/context.rs (`window`), crates/terminus-core/src/recall/get.rs (`rows_for`), crates/terminus-core/src/retention/mod.rs (the candidate query in selection)
- **Action:** D-08: `pre_worktree_column` exists only for its NULL arm, a store without `session_meta.project_pre_worktree`, and 1.0.0 creates that column. Delete the function and its doc comment. Each of its five callers names the column in its SQL directly: `m.project_pre_worktree` where the query aliases `session_meta` as `m` (`sessions`, `rows_for`, the retention candidate query), and `project_pre_worktree` where it does not (`projects`, `context::window`). Delete each caller's comment about the degraded-read helper. Every query keeps its projection order and its result types, so no row mapping changes.
- **Verify:** `grep -rn 'pre_worktree_column' crates/` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --lib --test recall --test retention --test redacted_recall --test inject_brief` and `cargo test -p terminus --all-features --test recall_cli --test mcp` pass.

### Task 2: Read the decision log's columns without checking for them

- **Files:** crates/terminus-core/src/feedback/replay.rs (`replay`; `eligible`; `recomputed`; `would_inject`; module docs), crates/terminus-core/src/feedback/stats.rs (`stats`)
- **Action:** D-08: remove the `recorded` and `typed` branches in `feedback/replay.rs` and the `recorded` branch in `feedback/stats.rs`. #105 does not list the last one, but D-08 includes it. In `replay`, both flags and their `missing_columns` lookups go. `eligible` always selects the guarded dropped-set expression, and `would_inject` always runs `prompt::quotable`. The `typed` parameter leaves `recomputed` and `would_inject`. In `stats`, the compacted and ordinary split is always queried. A row whose `compacted` is null is still counted in neither, which `docs/commands.md` already says. Rewrite the module docs and the doc comments that describe a store without `compacted`, `dropped` or `is_typed`. Keep the module docs' account of how a compacted decision replays, and the existing call shape of `prompt::quotable`, which task 6 changes.
- **Verify:** `grep -rn 'missing_columns' crates/terminus-core/src/feedback/` prints nothing; `grep -nE '\btyped\b' crates/terminus-core/src/feedback/replay.rs` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test feedback --test label` and `cargo test -p terminus --all-features --test replay` pass.

### Task 3: Treat a store missing the whole decision log as damaged

- **Files:** crates/terminus/src/cmd/read.rs (`DecisionLog`; `decision_log`; `damaged`; `NO_DECISION_LOG`), crates/terminus/src/cmd/stats.rs (`run`), crates/terminus/src/cmd/replay.rs (`run`), crates/terminus/tests/replay.rs (`stats_and_replay_on_a_store_without_the_decision_log_still_answer`)
- **Action:** D-08 and D-09: delete `DecisionLog::Absent` and `NO_DECISION_LOG`. In `stats` and `replay`, the `Absent` arms and the comments above the match that describe an older store go too. A store missing `decisions`, `labels` or both is `Damaged`, and both commands answer it through `read::unusable` as they now answer one missing table: exit 1, and under `--json` a document with `ok: false` and the reason. `Damaged`'s payload and `damaged`'s message are your call, with two constraints. The reason names every table of the two that is missing. No message claims the other table is still there when both are gone. The existing single-table test, `stats_and_replay_call_half_a_decision_log_a_failure`, keeps passing unchanged. Rewrite the doc comments on `DecisionLog` and `decision_log` to the new rule: 1.0.0 creates both tables, so a store missing either was damaged from outside. Rewrite `stats_and_replay_on_a_store_without_the_decision_log_still_answer` for the new outcome and rename it to say so. With both tables dropped, `stats --json` and `replay --json` each exit 1 with `ok` false and a reason naming `decisions` and `labels`. The human mode exits 1 with no `no such table` on stderr. Keep `DecisionLog::Damaged`, `read::unusable`, `read::damaged` and `Store::missing_tables`: phase 8 removes them.
- **Verify:** `grep -rnE 'DecisionLog::Absent|^ +Absent,|NO_DECISION_LOG|predates the decision log' crates/terminus/src crates/terminus/tests` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus --all-features --test replay --lib` passes; the rewritten test fails if `decision_log` maps both tables missing to `Present`.

### Task 4: Let a missing observations table fail as SQLite reports it

- **Files:** crates/terminus/src/cmd/observations.rs (`NO_TABLE`; `list`; the module docs paragraph on a store older than the table), crates/terminus/tests/observations.rs (`a_store_older_than_the_table_names_it_rather_than_naming_sqlite`)
- **Action:** D-10: remove the missing-table check at the top of `list`, the `NO_TABLE` constant and the module docs paragraph that explains them, which names `read::decision_log` as its model. Delete the test that expects an empty exit-0 answer for a dropped `observations` table. A store whose `observations` table was dropped from outside now gets the `no such table` error and exits 1. Give this case no new message and no new check: making `--json` failures write a document is phase 8's work.
- **Verify:** `grep -n 'NO_TABLE\|missing_tables' crates/terminus/src/cmd/observations.rs` prints nothing; `cargo test -p terminus --all-features --test observations` passes; the AC4 observations check in Notes prints `exit 1` and a stderr line containing `no such table: observations`.

### Task 5: Accept `--kind observation` like any other kind

- **Files:** crates/terminus/src/cmd/search.rs (`RETIRED_KIND`; `RETIRED_KIND_MESSAGE`; the `--kind` arm of argument parsing), crates/terminus/src/cmd/mcp/tools.rs (`run_search`), crates/terminus/tests/recall_cli.rs (`the_terminal_search_refuses_the_retired_observation_kind`), crates/terminus/tests/mcp.rs (`recall_search_refuses_the_retired_observation_kind_by_name`)
- **Action:** D-11: delete `RETIRED_KIND`, `RETIRED_KIND_MESSAGE` and the two guards that use them. The `--kind` arm then sets the filter as it does for any value, and `run_search` passes `kind` straight into `Filters`. The guard's message is 0.1.x history in a public error, and `observation` now behaves like any unknown kind: no hits and exit 0 from the CLI, and empty hits without `isError` from MCP. Delete the two tests that expect the refusal. Add no replacement test. The docs that describe the refusal change in task 7.
- **Verify:** `grep -rn 'RETIRED_KIND\|session summaries were removed' crates/` prints nothing; `cargo test -p terminus --all-features --test recall_cli --test mcp` passes; the AC5 checks in Notes print `exit 0` for the CLI and `[false,[]]` for MCP.

### Task 6: Fold the tool-result filter into the search's own rows

- **Files:** crates/terminus-core/src/recall/search.rs (`HEAD`; `Hit`; the `Hit` construction in `run`), crates/terminus-core/src/inject/prompt.rs (`quotable`; its call in the prompt arm; the `hit` test helper), crates/terminus-core/src/feedback/replay.rs (the `prompt::quotable` call in `would_inject`)
- **Action:** D-12: `HEAD` also projects `t.is_typed`, and `Hit` carries it as a nullable value. Name the field yourself, document it beside the other fields, and read it at its new position where `run` builds a `Hit`. `quotable` becomes an in-memory filter over the hits it is given. It takes no connection, runs no SQL and cannot fail. It drops exactly the hits the old query dropped: `record_type` `user` with `is_typed` 0. A hit whose `is_typed` is null is never dropped, and neither is any non-`user` hit. The ranked order stays as it is. Both callers use it: the prompt arm in `prompt.rs`, whose comment about a failed lookup injecting nothing goes, and `would_inject` in `feedback/replay.rs`. Rewrite `quotable`'s doc comment. Its paragraph on reading by turn id because search's projection feeds stores that predate the column no longer holds. Update the `hit` test helper in `prompt.rs` for the new field. Add unit tests to `prompt.rs`'s test module that pin the null rule: a `user` hit with `is_typed` null or 1 and an `assistant` hit pass, and a `user` hit with `is_typed` 0 is dropped, in order. `Hit` derives no `Serialize`, and the CLI and MCP build their own documents, so no output shape changes.
- **Verify:** `sed -n '/pub fn quotable/,/^}/p' crates/terminus-core/src/inject/prompt.rs` shows a signature with no `Connection` and a body with no `prepare`, `query` or `execute`; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --lib quotable` runs the new unit tests and they pass (name them so the filter `quotable` selects them); `cargo test -p terminus-core --all-features --test feedback --test inject_prompt --test recall --test recall_golden` and `cargo test -p terminus --all-features --test prompt --test replay --test inject` pass.

### Task 7: Bring the read-side docs up to date

- **Files:** docs/design/0007-injection.md (the exclusion step; `terminus stats` refusals; `terminus replay` outputs), docs/json-shapes.md (the paragraph on failures that write a document; `--kind`; `replay`'s `labels`; `observations`), docs/design/0003-ingest.md (`terminus observations` refusals), docs/commands.md (`--kind`), docs/design/0005-search.md (`terminus search` refusals)
- **Action:** D-11 and the read-side half of D-13. Design docs describe the current design only. In `0007-injection.md`: the exclusion step says `is_typed` comes with the search's own hits and drops the turn-id lookup and the failed-lookup sentence; the `stats` refusals say no store is an empty report with a reason and exit 0, and a store missing either decision-log table is damaged and exits 1; the `replay` outputs lose the clause about skipping `quotable` on a store whose `turns` lacks `is_typed`. In `json-shapes.md`: the list of failures that write a document says a decision log missing either or both of its tables, in place of "missing one of its two tables"; the `--kind` bullet loses the sentence that `observation` is refused; the `replay` `labels` bullet drops "or a store with no decision log"; the `observations` bullet about a store older than the table is deleted. In `0003-ingest.md`, the `terminus observations` refusals lose the sentence about a store with no `observations` table. In `commands.md`, the `--kind` bullet loses the sentence that `observation` exits 2. In `0005-search.md`, the refusals say `--limit 0` exits 2 and no longer name `--kind observation`.
- **Verify:** `grep -rn 'observation. exits 2\|kind observation\|Only .observation. is refused\|session summaries' docs/` prints nothing; `grep -rn 'no decision log\|missing one of its two tables\|older than the table\|carries no .observations. table\|no .observations. table' docs/` prints nothing; `grep -n 'failed lookup\|lacks .is_typed' docs/design/0007-injection.md` prints nothing.

## Notes

**Temp-store checks for tasks 4 and 5 (AC4, AC5).** Run them from the repository root after `cargo build -p terminus`:

1. `d=$(mktemp -d); w=$(mktemp -d); cp tests/fixtures/session-basic.jsonl "$w/"; TERMINUS_DATA_DIR="$d" target/debug/terminus ingest "$w/session-basic.jsonl"` archives one session.
2. AC5, CLI: `TERMINUS_DATA_DIR="$d" target/debug/terminus search --kind observation --project '*' x; echo "exit $?"` prints nothing on stdout and ends with `exit 0`.
3. AC5, MCP: `printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"check","version":"0"}}}' '{"jsonrpc":"2.0","method":"notifications/initialized"}' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"recall_search","arguments":{"query":"x","project":"*","kind":"observation"}}}' | TERMINUS_DATA_DIR="$d" target/debug/terminus mcp | tail -n 1 | jq -c '[.result.isError, (.result.content[0].text | fromjson | .hits)]'` prints `[false,[]]`.
4. AC4, observations: `sqlite3 "$d/terminus.db" 'DROP TABLE observations'; TERMINUS_DATA_DIR="$d" target/debug/terminus observations --project '*'; echo "exit $?"` prints a stderr line containing `no such table: observations` and ends with `exit 1`.
5. `rm -rf "$d" "$w"`.

**The live-store check (AC6, D-14), on John's machine.** The `terminus` on PATH is the installed 1.0.0 release and is used only to find the store. Never `cp` the live store while hooks may be writing to it:

1. `cd /code/terminus && cargo build --release -p terminus`
2. `copy=$(mktemp -d "$HOME/terminus-copy.XXXXXX")`. This goes under `$HOME` because the store is about 3 GiB and `/tmp` may be RAM-backed.
3. `sqlite3 "$(terminus status --json | jq -r .data.store)" "VACUUM INTO '$copy/terminus.db'"`. Instead, the newest snapshot works: `cp "$(ls -t ~/.local/share/terminus/snapshots/terminus-*.db | head -n 1)" "$copy/terminus.db"`. A snapshot is a finished file that no hook writes to.
4. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus ingest; echo "exit $?"` ends with `exit 0` and prints no line naming a failed file.
5. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus status --json | jq '.data.last_run.error'` prints `null`.
6. `TERMINUS_DATA_DIR="$copy" ./target/release/terminus verify; echo "exit $?"` prints no failure line and ends with `exit 0`.
7. `rm -rf "$copy"`.

**Planner's choices and findings.**

- Task 3: CONTEXT does not say how `Damaged` reports a store missing both tables. The planner chose to name every missing table, and to never claim the other table survives. The old message said "but not the rest of the decision log", which would be false when both are gone.
- Task 6 adds unit tests for `quotable`. D-12 locks the rule that a null `is_typed` is never dropped. Once the filter moves out of SQL, nothing else checks that rule.
- Task 7 covers doc lines that describe the removed code but are not in D-13's list: `0007-injection.md`'s exclusion step and `stats` refusals, `json-shapes.md`'s `--kind`, `labels` and `observations` bullets, and `0003-ingest.md`'s `observations` refusals. D-13 asks for every doc that describes the removed code, so they are in scope.
