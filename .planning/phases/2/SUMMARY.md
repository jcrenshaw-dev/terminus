---
phase: 2
status: complete
completed: 2026-10-23
---

# Phase 2: Planning tags - Summary

Every comment, assertion message, manifest comment and doc line that cited a tag from the removed planning files now states its rule itself, and `crates/terminus-core/tests/planning_tags.rs` fails when a new one is written (#106).

## What shipped

- Tag-free comments in the terminus-core sources, store through injection - `crates/terminus-core/src` (plan 1), including the SQL `--` comments inside `schema::CREATE_SQL`
- Tag-free comments in the rest of the terminus-core sources, recall through testkit - `crates/terminus-core/src` (plan 2)
- Tag-free comments and assertion messages in the terminus-core tests - `crates/terminus-core/tests` (plans 3 and 4)
- The fixture catalog described by what each fixture holds and proves - `tests/fixtures/README.md` (plan 4)
- Tag-free comments in the terminus crate's install, hook and injection side - `crates/terminus/src/cmd`, `crates/terminus/tests` (plan 5)
- Tag-free comments and assertion messages in the terminus crate's read, report and MCP side, `docs/json-shapes.md` stating the stdout/stderr split as the current contract, and tag-free manifest comments - `crates/terminus`, `docs/json-shapes.md`, `Cargo.toml`, `crates/terminus/Cargo.toml` (plan 6)
- The guard test: a hand-written matcher and comment reader over the named roots, with per-root file floors and a floor under the SQL comment lines read from `schema.rs` - `crates/terminus-core/tests/planning_tags.rs` (plan 7)

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 399fc49 | cut planning tags from the store, blob and owner-only comments |
| 1 | 2 | 4a9190b | cut planning tags from the derive, reindex, verify, recover and lineage comments |
| 1 | 3 | af3c913 | cut planning tags from the ingest comments |
| 1 | 4 | 26a49ab | cut planning tags from the injection comments |
| 1 | fix | 4591f07 | say when a snapshot is taken (risk review finding, #114) |
| 2 | 1 | ca4dab3 | cut planning tags from the recall and index comments |
| 2 | 2 | b08f435 | cut planning tags from the config, discovery, parse and error comments |
| 2 | 1, 2 | 1a94fd8 | rewrap three search and config comments the tag cuts left wide |
| 2 | 3 | c91a3b6 | cut planning tags from the feedback, observe, retention and testkit comments |
| 3 | 1 | 7e33910 | cut planning tags from the store, schema, verify and reindex tests |
| 3 | 2 | b8841c9 | cut planning tags from the ingest, parse, pass, backfill and capture tests |
| 3 | 3 | c11c3a7 | cut planning tags from the recall, index and egress tests |
| 4 | 1 | f809b59 | cut planning tags from the injection and compaction tests |
| 4 | 2 | 95abae3 | cut planning tags from the feedback, label and observe tests |
| 4 | 3 | bd87402 | cut planning tags from the config, discovery, project, lineage and retention tests |
| 4 | 4 | c83df71 | describe the fixture catalog by what each fixture holds |
| 5 | 1 | a2fdb03 | cut planning tags from the install, hook and lifecycle commands |
| 5 | 2 | 512d60e | cut planning tags from the install, uninstall, on-disk and hook tests |
| 5 | 3 | 1a47892 | cut planning tags from the brief, prompt, inject, suppression, decisions and replay tests |
| 6 | 1 | 0e37ecd | cut planning tags from the read, report and MCP commands |
| 6 | 2 | 725cc14 | cut planning tags from the CLI, corpus, crash, backfill, status and retention tests |
| 6 | 3 | 22131f9 | cut planning tags from the recall CLI, MCP and observations tests |
| 6 | 4 | d3364dc | cut planning tags from json-shapes.md and the manifest comments |
| 7 | 1 | 9a9663f | add a planning-tag matcher with self-tests |
| 7 | 2 | 19975e2 | read comment text from Rust, TOML, the schema SQL and markdown |
| 7 | 3 | 7674f87 | fail on a planning tag in any scanned comment or doc line |
| 7 | fix | 281ee53 | fail when the schema's SQL comments go unread (risk review finding, #115) |

## Deviations

- [deviation] Plan 5's first full-suite run failed in two wall-clock tests in `crates/terminus/tests/hook.rs`, `every_event_exits_zero_with_an_empty_stdout_inside_the_budget` (SessionStart p99 16.75 ms over 10 ms) and `every_event_stays_inside_the_budget_with_the_redaction_knob_on` (p99 16.53 ms over 10 ms), at load average 12.6 right after the cargo builds. The plan's diff is comment lines only, so nothing was changed; the one re-run, with `--no-fail-fast`, was green at load 8 to 11.

## Open items

- The `hook.rs` p99 budget tests (10 ms over 100 spawns) fail on Linux under heavy load, not only on macOS Intel. Worth adding to #75 and #104 when phase 3 takes them up.
- `crates/terminus-core/tests/inject_brief.rs`, above `nothing_in_the_injection_path_spawns_a_process_or_reads_observations`, says "this module's own doc explains at length why neither appears", but `crates/terminus-core/src/inject/brief.rs` explains only the git half. The line carried no tag, so plan 4 left it.
- `crates/terminus-core/src/observe/egress.rs` now reads "Measured over real transcripts" where it named the corpus a decision used; the archived context did not say which corpus, so the sentence no longer names it.
- Two reworded `assert_eq!` messages stay on one line so the change is message-only, and `rustfmt --check` would now split them: `crates/terminus-core/tests/reindex.rs:117` and `crates/terminus/tests/cli.rs:472`. Both files were rustfmt-clean at 1c55fec. `cargo fmt --all -- --check` now reports 7 files; the other 5 (`egress.rs`, `cmd/hook.rs`, `tests/hook.rs`, `release_workflows.rs`, `sign_release.rs`) carried the same drift before this phase.
- The guard reads `//`, `///` and `//!` comments, TOML `#` lines, the `--` lines inside `CREATE_SQL` and every markdown line, in the named roots only (D-14, D-15). A tag in a Rust block comment, a TOML comment after a value, or a top-level markdown file such as `README.md` or `THREAT-MODEL.md` would pass it. The tree has no block comments today.

## Goal check

The commits deliver the goal. Over `crates tests fuzz Cargo.toml docs`, #106's pattern prints only the four `session_title` lines (`crates/terminus/tests/owner_only.rs:444`, `crates/terminus/tests/prompt.rs:151` and the two `tests/fixtures/hooks/*.json` files), and the capital "Phase N", `PLAN-N`, "ROADMAP criterion" and "hardin" greps print nothing (AC1, AC2, run at 281ee53). Since 1c55fec the Rust diff outside `planning_tags.rs` changes 46 non-comment lines, and every one is half of a pair rewording one of the 23 `assert!`/`panic!` messages D-06 names; the TOML diff changes only `#` lines, and the only other files changed are `docs/json-shapes.md` and `tests/fixtures/README.md` (AC5). `cargo test -p terminus-core --test planning_tags` passes 11 of 11, and plan 7's full `cargo test --workspace --all-features` run exited 0 (reports/plan-7.md), which covers AC3. Plan 7's probes added `SRCH-02` to `lib.rs`, `D-04` to `cli.rs`, `INJ-07` inside `CREATE_SQL`, `AC3` to `fuzz/Cargo.toml` and `ING-08` to `docs/json-shapes.md`, one at a time, and each failed the guard naming file and line, while "phase 2" in `json-shapes.md` and `UTF-16` in a Rust comment passed (AC4, reports/plan-7.md task 3). After the #115 fix, renaming the declaration or moving the constant's opening quote fails the guard with "only 0 SQL comment lines read". "None can be added" holds within the comment forms and roots D-14 and D-15 chose. The limits are listed under Open items.
