---
phase: 4
status: complete
completed: 2026-10-23
---

# Phase 4: Search - Summary

Search drops query words the bundled `unicode61` tokenizer makes no token from, checks `since` and `until` as real dates and times in one core check before scope resolves, and binds every list in `recall/` as one `json_each` parameter with `MAX_PATHS` gone (#23, #24, #108).

## What shipped

- A table of the 1,061 code points in 167 ranges that `unicode61` starts no token on, and the drop in `Query::parse`, with a sweep test over every Rust-alphanumeric code point against the linked SQLite - `crates/terminus-core/src/recall/unindexable.rs`, `crates/terminus-core/src/recall/query.rs`
- Tests that an unindexable word changes no hit through core search, the CLI and MCP, and that a pointed Hebrew word still narrows a search - `crates/terminus-core/tests/recall.rs`, `crates/terminus/tests/recall_cli.rs`, `crates/terminus/tests/mcp.rs`
- One public date check, `recall::search::checked_bound`, that validates the calendar and the clock, run by `search::run` before `scope::resolve` - `crates/terminus-core/src/recall/search.rs`
- CLI `search`, `sessions` and `observations regenerate` read their dates through `cmd::date_value` and exit 2 naming the flag; MCP `recall_search` returns `isError` naming `since` or `until`; `cmd::time_bound` and the dead `InvalidTimeFilter` arms are gone - `crates/terminus/src/cmd/mod.rs`, `crates/terminus/src/cmd/search.rs`, `crates/terminus/src/cmd/sessions.rs`, `crates/terminus/src/cmd/observations.rs`, `crates/terminus/src/cmd/mcp/tools.rs`
- The path filter, every turn-id list and both exclusion lists bound as one JSON parameter through `json_each`, with the exclusion lists' null guard kept - `crates/terminus-core/src/recall/search.rs`, `crates/terminus-core/src/recall/get.rs`, `crates/terminus-core/src/recall/excerpt.rs`, `crates/terminus-core/src/recall/scope.rs`
- `MAX_PATHS`, its `maxItems` and its cap notice removed from `recall_search`; the 40,000-path MCP test now expects hits with no error and no cap notice - `crates/terminus/src/cmd/mcp/tools.rs`, `crates/terminus/tests/mcp.rs`
- Docs that state the new behaviour - `docs/design/0005-search.md`, `docs/design/0006-recall-through-mcp.md`, `docs/commands.md`, `docs/json-shapes.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | c3231b2 | Drop query words the bundled tokenizer makes no token from |
| 1 | 2 | 9ecda07 | Prove an unindexable word changes no hit through every front end |
| 1 | 3 | 2a32625 | Say what `unicode61` makes tokens from and what the query drops |
| 2 | 1 | 0844d70 | Check a time bound's calendar and clock in one public core check |
| 2 | 2 | 843fb5a | Check a search's dates before its scope answers |
| 2 | 3 | f299719 | Check `recall_search`'s dates with core's check and name them as MCP spells them |
| 2 | 4 | d20ed0c | Check `search` and `sessions` dates with core's check as the flags are read |
| 2 | 5 | 77ad50d | Check `observations regenerate`'s date with core's check and delete the CLI's copy |
| 2 | 6 | f104a07 | Say a time bound must be a real date or time and is checked before the scope |
| 3 | 1 | ee73aa4 | Bind a search's path filter as one JSON parameter |
| 3 | 2 | 9713b51 | Bind every turn-id list as one JSON parameter |
| 3 | 3 | 62f4541 | Bind each exclusion list as one JSON parameter, keeping the null guard |
| 3 | 4 | 641c183 | Drop `recall_search`'s paths cap now the list is one parameter |
| 3 | 5 | 938ae29 | Drop `recall_search`'s paths cap from the docs |

## Deviations

- [deviation] Plan 1 task 2 asserted every new test fails before task 1. The core Hebrew test `a_pointed_word_still_narrows_a_search` passes with the drop disabled, because `separator_components` already kept the pointed word whole and FTS5 split it inside the quoted phrase. The test was kept as written: it pins the keep-whole half of the rule and fails if a word holding any unindexable character were dropped. The other three task 2 tests fail with the drop disabled. (9ecda07)

## Open items

- `crates/terminus/tests/recall_cli.rs`, in `an_unindexable_word_changes_no_hit` (added by plan 1), and `crates/terminus/src/cmd/hook.rs:245` are not rustfmt-clean. They were left alone because this cycle does not run `cargo fmt` over the workspace.

## Goal check

The commits plausibly deliver the goal. Unindexable words: c3231b2 adds the table and the drop in `Query::parse`, and 9ecda07 tests it through core, the CLI and MCP, with the sweep test failing on a removed U+0345 entry (plan 1 report). Dates: `git grep -n 'fn time_bound' crates/` prints nothing, `cmd/search.rs` and `cmd/mcp/tools.rs` no longer mention `InvalidTimeFilter`, and 843fb5a's test `a_bad_date_is_refused_before_any_scope_answer` failed against the old order with `Ok(UnknownProject)` before passing (plan 2 report). Lists: `git grep -nE 'repeat_n\("\?"' crates/` and `git grep -n MAX_PATHS crates/ docs/` print nothing, `json_each` now appears in `recall/excerpt.rs`, `get.rs`, `scope.rs` and `search.rs`, and the path-filter test failed with "too many SQL variables" before ee73aa4 (plan 3 report). The last plan's run of `cargo test --workspace --all-features` passed 1,028 tests with 0 failed (plan 3 report). Nothing in the roadmap's four criteria looks missing. Whether a 32,767-path search returns on the macOS runners is proved only in CI, which this phase has not run.
