---
phase: 4
plan: 2
requirements: [SRCH-02]
files:
  - crates/terminus-core/src/recall/search.rs
  - crates/terminus-core/src/recall/mod.rs
  - crates/terminus-core/src/error.rs
  - crates/terminus-core/tests/recall.rs
  - crates/terminus/src/cmd/mcp/tools.rs
  - crates/terminus/tests/mcp.rs
  - crates/terminus/src/cmd/search.rs
  - crates/terminus/src/cmd/sessions.rs
  - crates/terminus/tests/recall_cli.rs
  - crates/terminus/src/cmd/observations.rs
  - crates/terminus/src/cmd/mod.rs
  - crates/terminus/tests/observations.rs
  - docs/commands.md
  - docs/json-shapes.md
  - docs/design/0005-search.md
---

# Phase 4: Search - Plan 2 of 3 (dates, #24)

## Goal

Search answers an unindexable word, checks dates as dates, and binds every list as one parameter (#23, #24, #108). This plan covers #24. terminus-core gets one public date check that validates the calendar and the clock. The CLI's `search`, `sessions` and `observations regenerate`, and MCP's `recall_search`, all use it. `recall::search::run` checks dates before it resolves scope. `cmd::time_bound` is deleted.

## Must be true when done

- `terminus search --since 2026-02-30 x`, `terminus sessions --until 2026-13-01`, `terminus observations regenerate --since 2026-02-30` and `terminus search --until 2026-08-12T25:00:00.000Z x` each exit 2, print nothing on stdout, and print a message naming the flag (`--since` or `--until`) and the value. This holds with a store and on a data directory with no store.
- MCP `recall_search` with `since: "2026-02-30"` returns `isError: true` with a reason naming `since`, with or without a store.
- `git grep -n time_bound crates/ docs/` and `git grep -n InvalidTimeFilter crates/terminus/src/` print nothing.
- A core call to `search::run` with a bad date, from a directory no project covers, returns `InvalidTimeFilter`, not `Ok` with an unknown-project reason.
- Real edge dates still pass: `2024-02-29`, `2000-02-29` and `2026-08-12T23:59:59.999Z`. `2025-02-29`, `1900-02-29`, month `00` or `13`, `2026-04-31`, hour `24`, and minute or second `60` are refused.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-04, D-05, D-06 and D-07, and D-12 and D-13 as they apply to #24. Source: `.planning/phases/4/CONTEXT.md`, phase 4.
- My call, so AC4 can be checked literally: no front end matches on the `InvalidTimeFilter` variant. Each maps whatever error the core check returns into its own refusal, so `git grep -n InvalidTimeFilter crates/terminus/src/` prints nothing.
- Out of scope: `observe::regenerate` checking its own `since` (D-04 names the four front-end callers, not the library function); any new crate, a date crate included; running `cargo fmt` over the workspace (D-13). New tests and comments may cite `#24`, never a requirement id or a decision tag.
- Plans run in order 1, 2, 3. This plan runs after plan 1, which edits `recall/mod.rs`, `tests/recall.rs`, `recall_cli.rs`, `mcp.rs` and `0005-search.md`, and before plan 3, which edits `recall/search.rs`, `tests/recall.rs`, `cmd/mcp/tools.rs`, `mcp.rs`, `recall_cli.rs`, `docs/commands.md` and `docs/json-shapes.md`.

## Tasks

### Task 1: Make core's date check validate the calendar and the clock, and make it public

- **Files:** crates/terminus-core/src/recall/search.rs (`bound`, `is_shaped`, `FIELD_SINCE`, `FIELD_UNTIL`), crates/terminus-core/src/recall/mod.rs (the `pub use search::` line), crates/terminus-core/src/error.rs (`Error::InvalidTimeFilter` and its message), crates/terminus-core/tests/recall.rs (`a_malformed_time_is_a_caller_error`)
- **Action:** `bound` in `recall/search.rs` becomes terminus-core's one public date check (D-04). Make it `pub` and re-export it from `recall` beside `Filters`. You may give it a public name that says what it does; if you do, record the name in the summary. It keeps both shapes it accepts today, and it keeps returning the bound as a string that compares against `turns.ts`. A bare date extends to `T00:00:00.000Z` for `since` and `T23:59:59.999Z` for `until`, so the front ends keep handing on the extended value. On top of the shape, it now checks the calendar and the clock (D-07):
  - month 01 to 12
  - day 01 up to the month's Gregorian length: February has 29 days in a year divisible by 4, except a year divisible by 100 that is not divisible by 400
  - in the full form, hour 00 to 23, and minute and second 00 to 59
  - the milliseconds stay any three digits

  `Filters::push_onto` keeps calling the same function, so library callers get the same check. Keep `Error::InvalidTimeFilter { field, value }` exactly: no new field and no new variant. Rewrite its message and doc so they no longer say "is not a time" or that the shape is wrong. The message names the field and the value, says the value is not a real date or time, and names the two accepted forms. That wording is true both for a value of the wrong shape and for a well-shaped impossible one. Rewrite `bound`'s doc to cover the calendar and clock checks and why they exist: `2026-02-30` is well shaped, sorts between 28 February and 1 March, and so would answer a plausible wrong window. No date crate. In `a_malformed_time_is_a_caller_error`, add these values, each still checked as both `since` and `until`: `2026-02-30`, `2025-02-29`, `1900-02-29`, `2026-13-01`, `2026-00-10`, `2026-04-31`, `2026-08-00`, `2026-08-32`, `2026-08-12T24:00:00.000Z`, `2026-08-12T25:00:00.000Z`, `2026-08-12T23:60:00.000Z`, `2026-08-12T23:59:60.000Z` and `2026-02-30T00:00:00.000Z`. Add a test, citing #24, that `2024-02-29`, `2000-02-29`, `2026-12-31`, `2026-08-12T00:00:00.000Z` and `2026-08-12T23:59:59.999Z` are each accepted as both bounds, and that the message for `2026-02-30` names the field and the value.
- **Verify:** `cargo test -p terminus-core --all-features --test recall` passes, including the extended and the new test. `git grep -n 'is not a time' crates/terminus-core/` prints nothing. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 2: Check the dates before the scope in `search::run`

- **Files:** crates/terminus-core/src/recall/search.rs (`run`, `Filters::push_onto`), crates/terminus-core/tests/recall.rs
- **Action:** `run` checks `since` and `until` with task 1's check before it calls `scope::resolve` (D-06). Every scope short circuit then comes after the date check: everything excluded, nothing archived, unknown project, project excluded. Check each bound once per search; how the checked value reaches `push_onto` is your call. Rewrite `run`'s doc so it gives the order: dates, then scope, then the nothing-searchable answer. Add a test citing #24. With `Scope::Directory` of a path under the bench root that no project covers, a `since` of `2026-02-30` returns `Err(InvalidTimeFilter)` with field `since` and that value, and an `until` of `last tuesday` returns the same error for `until`. With a config excluding `/` and `Scope::Everything`, a bad `since` still returns `InvalidTimeFilter`. The control: the same uncovered directory with a real date returns `Ok` with `Reason::UnknownProject`.
- **Verify:** `cargo test -p terminus-core --all-features --test recall` passes. The new test fails when the date check runs after `scope::resolve`: on the parent commit it gets `Ok` with an unknown-project reason.

### Task 3: Check MCP's dates with core's check

- **Files:** crates/terminus/src/cmd/mcp/tools.rs (`optional_time`, `run_search`, the `use terminus_core::Error` import), crates/terminus/tests/mcp.rs (`a_malformed_call_is_an_empty_result_with_a_reason_and_never_a_throw`)
- **Action:** `optional_time` calls core's date check instead of `crate::cmd::time_bound`. It turns any error from the check into `Refused::caller` (`isError: true`) with a message naming the argument as MCP spells it, `since` or `until` with no dashes, and the value (D-05). It does this without matching on the `InvalidTimeFilter` variant; the core error's own message already starts with the field. `run_search` still calls it before `reader()`, so a bad date is a caller error with no store too. Remove the `Err(Error::InvalidTimeFilter { .. })` arm from `run_search`, which can no longer be reached (D-05 and the 2026-10-22 simplify review's comment on #24), along with any import only that arm used. Rewrite `optional_time`'s doc: it goes through terminus-core's date check, the one the CLI uses. In the malformed-call test, rewrite the comment that calls `2026-13-99` deliberately not a caller error (D-12). Add `since: "2026-13-99"`, `since: "2026-02-30"` and `until: "2026-08-12T25:00:00.000Z"` to the cases. Add a test, citing #24: `since: "2026-02-30"` gives `isError` true with a reason that contains `since` and `2026-02-30` and does not contain `--since`, both on the ingested bench and on a data directory that does not exist.
- **Verify:** `cargo test -p terminus --all-features --test mcp` passes. `grep -n 'time_bound\|InvalidTimeFilter' crates/terminus/src/cmd/mcp/tools.rs` prints nothing. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 4: Check `search` and `sessions` dates with core's check

- **Files:** crates/terminus/src/cmd/search.rs (`parse`, `run`, the `use terminus_core::Error` import), crates/terminus/src/cmd/sessions.rs (`parse`), crates/terminus/tests/recall_cli.rs (`a_bad_command_line_is_misuse_and_not_an_empty_result`)
- **Action:** In both parsers, `--since` and `--until` go through core's date check while the arguments are read, before the store opens (D-05). That stays necessary because `run` returns `Opened::Nothing` with exit 0 before `search::run` is reached. An error becomes `Failure::Misuse`, exit 2, with a message naming the flag with its dashes and the value as typed, again without matching on the `InvalidTimeFilter` variant. Remove the `Err(Error::InvalidTimeFilter { .. })` arm from `cmd/search.rs`'s `run`, along with the import only it used. Any other error from `search::run` stays `Failure::Operational`. Rewrite the comment above the `--since` arm in `cmd/search.rs`, which says the check is here so a bad bound is misuse whatever project the caller stands in. It is here because a data directory with no store answers before the query layer is reached. Leave `cmd::time_bound` for now; `observations` still calls it until task 5. In `a_bad_command_line_is_misuse_and_not_an_empty_result`, drop `--project *` from the `--since last tuesday` case and drop the comment about stepping around the order (D-12), so the case runs from the work directory no project covers. Add `search --since 2026-02-30 x`, `search --until 2026-08-12T25:00:00.000Z x` and `sessions --until 2026-13-01`. Each exits 2, prints nothing on stdout, and has stderr naming the flag with its dashes and the value. Add a test, citing #24, that runs the same three date cases on a bench with nothing ingested, so no store: each exits 2 and names its flag.
- **Verify:** `cargo test -p terminus --all-features --test recall_cli` passes. `grep -n 'time_bound\|InvalidTimeFilter' crates/terminus/src/cmd/search.rs crates/terminus/src/cmd/sessions.rs` prints nothing. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 5: Check `observations regenerate` with core's check and delete `cmd::time_bound`

- **Files:** crates/terminus/src/cmd/observations.rs (`regenerate_args`, the `Regenerate` struct's `since` doc), crates/terminus/src/cmd/mod.rs (`time_bound`), crates/terminus/tests/observations.rs (`a_malformed_since_is_misuse_and_rebuilds_nothing`)
- **Action:** `regenerate_args` reads `--since` through core's date check, with an error mapped to `Failure::Misuse` naming `--since` and the value, the way task 4 does. Rewrite the two comments that point at `cmd::time_bound`: the `Regenerate::since` doc and the comment in `regenerate_args`. Then delete `time_bound` and its doc from `cmd/mod.rs` (D-04). Its last caller is gone, and the workspace now has one date rule, core's. Leave `observe::regenerate` unchanged. In `a_malformed_since_is_misuse_and_rebuilds_nothing`, add `2026-02-30`: it exits 2, stdout is empty, stderr names `--since`, and the spoiled row is still `SPOILED`. Add a test, citing #24, that `observations regenerate --since 2026-02-30` on a bench with nothing ingested, so no store, exits 2 and names `--since`.
- **Verify:** `git grep -n time_bound crates/ docs/`, `git grep -n InvalidTimeFilter crates/terminus/src/` and `git grep -n 'is not a time' crates/ docs/` each print nothing. `cargo test -p terminus --all-features --test observations --test recall_cli --test mcp` passes. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 6: Say in the docs that a bound must be a real date or time

- **Files:** docs/commands.md (`### Time bounds`), docs/json-shapes.md (the `isError` bullet before `### recall_search`), docs/design/0005-search.md (§6, Figure 1 and its caption)
- **Action:** Docs describe the current design only. In `### Time bounds`, keep the two forms, the bare-date rule and the inclusive bounds. Replace "Any other form exits 2": a value in any other form, or one that is not a real date or time such as `2026-02-30`, month 13 or hour 25, exits 2 with a message naming the flag. In `json-shapes.md`, "a `since` or `until` in neither time form" becomes a `since` or `until` that is not a real date or time in one of the two forms (D-12). In Figure 1 of `0005-search.md`, add a date check between `Query::parse` and `scope::resolve` (D-06 and D-12): `since` and `until`, when given, must each be a real date or time in one of the two forms. Its "no" edge ends at a caller error, `InvalidTimeFilter`, and its "yes" edge goes on to `scope::resolve`. If the caption describes the order, it says the same.
- **Verify:** `grep -n 'Any other form exits 2\|in neither time' docs/commands.md docs/json-shapes.md` prints nothing. `grep -n 'InvalidTimeFilter' docs/design/0005-search.md` shows the new Figure 1 node. In the flowchart source, the edge into `scope::resolve` comes from the date check, not from `Query::parse`.

## Notes

- `observe::regenerate` does not check its own `since`. D-04 routes all four front-end callers through core's check and does not ask for a check inside the library function, so a library caller passing a bad `since` there is unchecked, as it is today. This is for John to decide, not this plan.
- A grep-based Verify here is scoped to the files or the pattern the task changes, so unrelated matches cannot fail it (lesson from `.planning/phases/1/SUMMARY.md`, phase 1).
