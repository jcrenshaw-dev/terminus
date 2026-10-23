---
phase: 4
plan: 3
requirements: [SRCH-03]
files:
  - crates/terminus-core/src/recall/search.rs
  - crates/terminus-core/tests/recall.rs
  - crates/terminus-core/src/recall/get.rs
  - crates/terminus-core/src/recall/excerpt.rs
  - crates/terminus/src/cmd/mcp/tools.rs
  - crates/terminus/tests/mcp.rs
  - crates/terminus-core/src/recall/scope.rs
  - crates/terminus/tests/recall_cli.rs
  - docs/commands.md
  - docs/json-shapes.md
  - docs/design/0006-recall-through-mcp.md
---

# Phase 4: Search - Plan 3 of 3 (lists bound as one parameter, #108)

## Goal

Search answers an unindexable word, checks dates as dates, and binds every list as one parameter (#23, #24, #108). This plan covers #108. Every list in `recall/search.rs`, `recall/get.rs`, `recall/excerpt.rs` and `recall/scope.rs` is bound as one JSON array parameter read through `json_each`, so a list longer than SQLite's variable limit still runs. `MAX_PATHS` goes.

## Must be true when done

- `search::run` with 40,000 paths, more than the bundled SQLite's 32,766 variables, returns hits with no error. MCP `recall_search` with 40,000 `paths` returns hits with `isError` false and no cap notice.
- `git grep -nE 'repeat_n\("\?"' crates/` and `git grep -n MAX_PATHS crates/ docs/` print nothing.
- With one project excluded in config, a session whose `project` is null still appears in `terminus search --project '*'` results. With a worktree excluded, a session whose `project_pre_worktree` is null is still quoted and counted in the session-start brief.
- `MAX_IDS` is still 25, and no comment in `crates/terminus/` says a list is bound one SQL placeholder per element.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-08, D-09, D-10 and D-11, and D-12 and D-13 as they apply to #108. Source: `.planning/phases/4/CONTEXT.md`, phase 4.
- The model to follow is `crates/terminus-core/src/feedback/label.rs` (`sought_sql`: `EXISTS (SELECT 1 FROM json_each(...) s WHERE s.value = ...)`). Measured while planning, 2026-10-23, on the bundled SQLite 3.53.2: `path IN (SELECT value FROM json_each(?1))` over 40,000 paths, one of them real, finds the real row, and `json_each` over a JSON integer array matches integer ids.
- Serialize each list in a way that cannot fail. A `serde_json::Value` array's `Display` is infallible, for instance. Never fall back to an empty array: for the two exclusion lists, an empty array would show an excluded project.
- Out of scope: the value of `MAX_IDS`; a CLI test with 32,767 or more `--path` arguments (D-11: argv size on the macOS runners has not been checked); rusqlite's `limits` feature; any new crate; #109; running `cargo fmt` over the workspace (D-13). New tests and comments may cite `#108`, never a requirement id or a decision tag.
- Plans run in order 1, 2, 3. This plan runs last. It shares `recall/search.rs`, `tests/recall.rs`, `cmd/mcp/tools.rs`, `mcp.rs`, `recall_cli.rs`, `docs/commands.md` and `docs/json-shapes.md` with plan 2, and `tests/recall.rs`, `recall_cli.rs` and `mcp.rs` with plan 1.

## Tasks

### Task 1: Bind the path filter as one JSON parameter

- **Files:** crates/terminus-core/src/recall/search.rs (`Filters::push_onto`), crates/terminus-core/tests/recall.rs
- **Action:** The `paths` clause of `Filters::push_onto` compares `p.path` against the values of one `json_each` over a single bound parameter: the paths as a JSON array, serialized with `serde_json`, which core already depends on (D-08). Keep the `EXISTS` form, so a turn carrying several of the paths is still one hit. Keep writing nothing for an empty list. Remove the per-path `?` loop. Add a test, citing #108, that calls `search::run` through the `narrowed` helper (so `Scope::Everything`) with query `docs/RETRY.md` and 40,000 paths. The first 39,999 name nothing; the last is `docs/RETRY.md`, which only `session-recall.jsonl`'s `Read` turn carries. Putting it last means a cut list would lose it. The test expects `Ok` with exactly that one hit (D-11: at least 32,767 paths).
- **Verify:** `cargo test -p terminus-core --all-features --test recall` passes, including the new test. On the parent commit the new test fails with SQLite's "too many SQL variables". `grep -n "sql.push('?')" crates/terminus-core/src/recall/search.rs` prints nothing.

### Task 2: Bind every turn-id list as one JSON parameter

- **Files:** crates/terminus-core/src/recall/search.rs (`weight_by_entities`), crates/terminus-core/src/recall/get.rs (`rows_for`), crates/terminus-core/src/recall/excerpt.rs (`coordinates`), crates/terminus/src/cmd/mcp/tools.rs (the `MAX_IDS` doc, the `run_get` doc), crates/terminus/tests/mcp.rs (the doc of `a_list_longer_than_the_cap_is_truncated_and_said_so`)
- **Action:** Each of the three `IN (...)` lists built with `std::iter::repeat_n("?", ...)` becomes `IN` over the values of one `json_each` of a single bound parameter: the ids as a JSON array (D-08). The rows each query returns do not change. No `repeat_n("?")` builder is left in `crates/`. `MAX_IDS` stays 25, because each id can cost a whole decompressed session (D-10). Its doc, `run_get`'s doc ("The id list is bounded here and nowhere else...") and the doc of `a_list_longer_than_the_cap_is_truncated_and_said_so` stop citing one SQL placeholder per id. The reason left for the cap is the decompression cost. `terminus show` stays uncapped. The `MAX_PATHS` doc is task 4's.
- **Verify:** `git grep -nE 'repeat_n\("\?"' crates/` prints nothing. `grep -n 'placeholder per id\|placeholder at a time' crates/terminus/src/cmd/mcp/tools.rs crates/terminus/tests/mcp.rs` prints nothing. `cargo test -p terminus-core --all-features --test recall --test recall_golden --test redacted_recall --test inject_prompt` and `cargo test -p terminus --all-features --test mcp --test recall_cli` pass. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 3: Bind the two exclusion lists as one JSON parameter each, keeping the null guard

- **Files:** crates/terminus-core/src/recall/scope.rs (`Scoped::push_filter`), crates/terminus/tests/recall_cli.rs
- **Action:** Each `NOT IN` list in `push_filter` becomes `NOT IN` over the values of one `json_each` of a single bound parameter (D-08). Keep the `({column} IS NULL OR ...)` guard on both columns. `NULL NOT IN (...)` is never true, so without the guard a session with no key would drop out of search, the resume brief's last session and the brief's counts. Keep skipping an empty list. `push_filter` stays infallible, so its three callers stay as they are: `recall/search.rs` (`run`), `inject/brief.rs` (`last_session_row`) and `inject/mod.rs` (`count`). Rewrite the parts of its doc that describe one placeholder per excluded value. Add a test to `recall_cli.rs`, citing #108, for AC6:
  - Write a small transcript whose records carry no `cwd`, shaped like the file's other hand-written transcripts, and ingest it through the binary with the fixtures.
  - Assert the premise: its `session_meta.project` is null.
  - Exclude project-beta in `terminus.toml`.
  - `search --project '*' --json` for a word only that transcript holds exits 0, and returns that session's hit with `project` null.

  The brief's half of AC6 is already proved, and stays unchanged, by `crates/terminus-core/tests/inject_brief.rs`'s `a_session_excluded_under_its_worktree_key_is_neither_quoted_nor_counted`. With the worktree excluded, the repository session, whose `project_pre_worktree` is null, is still the one quoted and is still counted. A brief's scope always names a project, so the guard it depends on is the `project_pre_worktree` one.
- **Verify:** `cargo test -p terminus --all-features --test recall_cli` and `cargo test -p terminus-core --all-features --test inject_brief --test recall --test exclusion` pass. Remove `IS NULL OR` from `push_filter` and run the new CLI test: it fails. Restore the guard and it passes.

### Task 4: Drop `recall_search`'s paths cap

- **Files:** crates/terminus/src/cmd/mcp/tools.rs (`MAX_PATHS`, `descriptors`, `run_search`), crates/terminus/tests/mcp.rs (`a_path_filter_longer_than_the_cap_is_bounded_and_said_so`)
- **Action:** Remove `MAX_PATHS` and its doc, the `maxItems` on the `paths` descriptor, the "at most N per call" in its description, the cut to `MAX_PATHS` in `run_search`, and the "only the first N of M paths were filtered on" reason (D-09). Every path a client sends then reaches `search::run`, and the answer stays capped by `MAX_HITS`. Rewrite the 40,000-path test and its doc, which says the filter binds one SQL placeholder per path (D-11, D-12). Stand in project-alpha. Send `paths` of 39,999 paths nothing carries followed by `docs/RETRY.md`, with query `docs/RETRY.md`. Expect at least one hit, `isError` false and `reason` null, so no cap notice. Keep the assertion that no SQL text or `sqlite:` reaches the reason. Rename the test to say what it now proves.
- **Verify:** `git grep -n MAX_PATHS crates/` prints nothing. `git grep -n -i 'sql placeholder' crates/terminus/` prints nothing. `cargo test -p terminus --all-features --test mcp` passes, and the rewritten test still sends 40,000 paths.

### Task 5: Drop the paths cap from the docs

- **Files:** docs/commands.md (`terminus mcp`, the `recall_search` bullet), docs/json-shapes.md (`### recall_search`), docs/design/0006-recall-through-mcp.md (the `tools.rs` row of the file table, the `recall_search` argument table's `paths` row, the `reason` paragraph after that table, and the constants paragraph that lists the tool limits)
- **Action:** Docs describe the current design only. In `commands.md`, `paths` loses "(at most 25)". In `json-shapes.md`, "and only the first 25 `paths` are filtered on" goes, and so does the last step of the `reason` order, `paths` cut to the first 25. The scope boundary asks for docs that state the old behaviour to be rewritten, and these two lines do. In `0006-recall-through-mcp.md`, `MAX_PATHS` leaves the `tools.rs` row and the constants paragraph, the `paths` row loses its cap sentence, and the `reason` paragraph loses "or that only the first 25 paths were filtered on" (D-09). Every `MAX_IDS` and `turn_ids` mention stays (D-10).
- **Verify:** `git grep -n MAX_PATHS crates/ docs/` prints nothing. `grep -n 'paths' docs/json-shapes.md docs/design/0006-recall-through-mcp.md docs/commands.md | grep 25` prints nothing. `grep -n 'first 25' docs/json-shapes.md docs/design/0006-recall-through-mcp.md` prints only lines about turn ids: the `recall_get` section of `json-shapes.md`, and the `turn_ids` row and `recall_get`'s `reason` row in 0006.

## Notes

- D-11's flagged assumption stands: the CLI hands its `--path` list to `search::run` unchanged (`cmd/search.rs`, `parse` pushes every `--path` onto `Filters::paths`), so task 1's core test covers `terminus search` as well.
