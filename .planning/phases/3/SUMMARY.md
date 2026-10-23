---
phase: 3
status: complete
completed: 2026-10-23
---

# Phase 3: Hook tests on macOS Intel - Summary

The stdout hook test now holds its ingest at the testkit fault point `fault::AFTER_BLOB` and proves the read returned mid-pass from the ready file and an empty `runs` table, and a timing miss in either #104 test prints its poll gap, a `/usr/bin/true` spawn time and the hook's own evidence with one verdict.

## What shipped

- The stdout test holds its ingest at a fault point instead of polling `ps` for it - `crates/terminus/tests/hook.rs:738`
- The stdout test's read is bounded by its 250 ms limit, so a held stdout fails at the limit rather than after the 30 s hold - `crates/terminus/tests/hook.rs:767`
- A stdin test miss says whether the hook was blocked or the runner stalled - `crates/terminus/tests/hook.rs:432`
- A stdout test miss says whether the hook was blocked, the runner stalled, or a process the hook started held stdout open - `crates/terminus/tests/hook.rs:819`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 15536bd | Hold the stdout test's ingest at a fault point and prove the read from the ready file and the runs table |
| 1 | 2 | 91d5340 | Bound the stdout test's read by its 250 ms limit |
| 1 | 3 | 1d0a636 | Diagnose a stdin test miss as a blocked hook or a stalled runner |
| 1 | 4 | 93e40ca | Diagnose a stdout test miss three ways |

## Deviations

- [deviation] The plan's warning check (`cargo test ... --no-run 2>&1 | grep 'tests/hook.rs'`) was asserted to print nothing, but cargo always prints its own `Executable tests/hook.rs (...)` line for `--no-run`, so it can never be empty. The executor ran it as `grep -E 'warning|tests/hook.rs:'` after touching the file to force a rebuild, and that printed nothing. No code changed for it.

## Open items

- The `macos-15-intel` release job criterion stays human-verify until the milestone pull request runs in GitHub Actions.
- The drain-line-present branch of the stdin evidence ("past stdin and had not exited") matches `cmd::hook`'s text at `crates/terminus/src/cmd/hook.rs:393`, but no verify step forces that branch.

## Goal check

The four commits plausibly deliver the goal, with the macOS Intel run itself still to be seen. `git diff --name-only 6b05012..HEAD` lists only `crates/terminus/tests/hook.rs`, and nothing under `.github` changed. The stdout test sets `fault::AT` to `fault::AFTER_BLOB` and `fault::READY` (`hook.rs:754-755`) and no longer calls `await_process`, which is now called only at `hook.rs:278` and `hook.rs:1025`, the two sibling tests. Both limits stand: `from_millis(250)` at `hook.rs:767` and `from_secs(10)` at `hook.rs:451`. The executor's report records the stdout test passing a 20-run `--release` loop, the full hook file passing 8 tests in debug and release, the inherit mutation failing at about 253 ms with the held-stdout verdict, and the full workspace suite exiting 0. None of that proves the `macos-15-intel` runner passes, which is the open human-verify item above.
