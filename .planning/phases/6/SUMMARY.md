---
phase: 6
status: complete
completed: 2026-10-20
---

# Phase 6: Simplify the 1.0.0 changes - Summary

Eleven of the SIMPLIFY.md findings (1, 3, 5, 6, 7, 8, 9, 10, 11, 13 and 16) are applied across the ingest pass, the store and the read side, with no change in behavior: 16 files changed, 283 lines in and 380 out.

## What shipped

- One resume rule, `parse::complete_len`, which `scan_from` and `prepare` both call - `crates/terminus-core/src/parse/mod.rs`, `ingest/mod.rs`
- No length stat where its answer is not read, and `Existing::read` looks up its own row once - `crates/terminus-core/src/ingest/mod.rs`, `ingest/backfill.rs`
- `Gate` is gone; the snapshot check returns a `Result` and `run_with` adds the note's prefix once - `crates/terminus-core/src/ingest/pass.rs`
- `Store::open` sets the statement cache's capacity itself - `crates/terminus-core/src/store/open.rs`, `derive.rs`
- The `block_reads` log records each read from one place per query, with `reset` and `record_key` removed - `crates/terminus-core/src/store/archive.rs`
- "No terminus store" is answered by the read-only open's first statement; `holds_no_tables` and the second connection are gone - `crates/terminus-core/src/store/open.rs`, `store/mod.rs`, `crates/terminus/src/cmd/read.rs`
- The no-store `status` and `verify` documents are built by the real builders - `crates/terminus/src/cmd/status.rs`, `cmd/verify.rs`
- The brief works out its room after the frame once - `crates/terminus-core/src/inject/brief.rs`
- `Scoped`'s exclusion lists are read straight from its fields - `crates/terminus-core/src/recall/scope.rs`
- One schema snapshot and one aging step for the tests, in testkit - `crates/terminus-core/src/testkit.rs`, `crates/terminus/tests/cli.rs`, `tests/status.rs`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 30d8fb8 | write the resume rule once, in parse |
| 1 | 2 | 2b061ee | skip the length stat where its answer is not read |
| 1 | 3 | 61298b0 | let Existing::read look up its own row and read it once |
| 1 | 4 | 9fe99ac | return the snapshot gate as a Result and prefix its note once |
| 1 | report | b315f60 | plan 1 executor report |
| 2 | 1 | 7af18b3 | set the statement cache's capacity instead of counting statements against it |
| 2 | 2 | 49996bb | log each block read from one place per query and drop the uncounted key reader |
| 2 | 3 | 2ad68ab | answer no store yet from the read-only open's first statement |
| 2 | report | 337efbf | plan 2 executor report |
| 3 | 1 | c963a35 | build the no-store status and verify documents with the real builder |
| 3 | 2 | a225950 | work out the brief's room after the frame once |
| 3 | 3 | afd86d1 | read Scoped's exclusion lists straight from its fields |
| 3 | 4 | 99cc79d | keep one schema snapshot and one aging step in testkit |
| 3 | report | f9369ca | plan 3 executor report |

The three plans ran in parallel worktrees. Plan 1 merged as a fast-forward; plans 2 and 3 merged at 0ca290a and 22a1576.

## Deviations

None - plans executed as written.

## Open items

- `cargo fmt --all --check` reports drift in four files outside this phase's scope, all present before it started: `crates/terminus/src/cmd/hook.rs:244`, `crates/terminus/tests/hook.rs:26`, `crates/terminus/tests/release_workflows.rs:18` and `:43`, and `crates/terminus/tests/sign_release.rs:388`.
- The `Fixes #90` criterion is checked on the merged PR, at landing.

## Goal check

The phase's commits plausibly deliver the goal. Each plan's named removals are gone from the merged tree: `grep -rn "consumed(" crates` and a word search for `Gate` in `crates/terminus-core/src/ingest` both print nothing, and `holds_no_tables` survives only as a unit test's name, as plan 2's report says. "No change in behavior" rests on three pieces of evidence. First, `git diff 24c90b4 HEAD -- crates` adds, removes or changes no line containing `assert`. Second, the full suite on the merged tree (`cargo test --workspace --all-features --no-fail-fast`) reports 1015 passed and 0 failed. Third, plan 3 compared the no-store `status --json` and `verify --json` output byte for byte before and after with `cmp`. Plan 3's range touched the migrations surface, but only because test-helper `ALTER TABLE` lines moved into testkit. The blocking risk review (openai, gpt-5.6-sol) raised no findings on that range, after its outbound filter redacted 6 credential-shaped spans. Nothing in the goal looks missing. The four fmt drifts above predate the phase and are outside its files.
