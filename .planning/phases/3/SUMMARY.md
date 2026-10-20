---
phase: 3
status: complete
completed: 2026-10-19
---

# Phase 3: One excluded-project rule - Summary

The rule that keeps an excluded project out of recall now lives on `Scoped` as one SQL filter, `push_filter`, and one Rust check, `admits`, and every read site calls one of them (#80).

## What shipped

- `Scoped::push_filter` and `Scoped::admits`, with `excluded_projects()` and `excluded_pre_worktree()` private - `crates/terminus-core/src/recall/scope.rs`
- Search, the brief and `inject::count` call `push_filter`; both copies of `push_exclusion` are gone - `recall/search.rs`, `inject/brief.rs`, `inject/mod.rs`
- Get and context call `admits` and still answer `NoSuchTurn`; their `hidden` closures are gone - `recall/get.rs`, `recall/context.rs`
- `ProjectKeys::excluded` is gone; `visible::is_excluded` takes both keys and `retention::is_excluded` calls it - `config.rs`, `retention/mod.rs`
- Three tests that pin the exclusion paths no test covered, passing on the starting code and after - `tests/inject_brief.rs`, `tests/recall.rs`, `inject/brief.rs`'s `predating_store` module

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | b74b8bb | Pin the three exclusion paths no test covers |
| 1 | 2 | c12b625 | Write the scope and exclusion SQL once, on `Scoped` |
| 1 | 3 | 9dd14cc | Check a turn's scope and exclusion once, on `Scoped` |
| 1 | 4 | 3f0154c | Let retention use the visible check and drop the unused third copy |

## Deviations

None - plans executed as written.

## Open items

- `cargo fmt --check --all` flags existing formatting in `crates/terminus/src/cmd/hook.rs`, `tests/hook.rs`, `tests/release_workflows.rs` and `tests/sign_release.rs`. None of them was touched by this phase.
- The executor's break-and-restore runs restored files with `mv` from a backup older than the broken build, so cargo once reused the broken binary and reported a false failure after Task 4's break. Touching the file and rebuilding cleared it. No counted result depended on it.
- The `risk_surface` review raised one blocker, refuted: the review seam's credential filter blanked `key: Option<` in `scope.rs:155` (3 redactions), so the OpenAI reviewer read broken syntax the source does not hold. That is Cadence friction, not a Terminus defect.

## Goal check

The four commits deliver the goal. `push_filter` is defined once at `recall/scope.rs:182` and called from `recall/search.rs:348`, `inject/brief.rs:476` and `inject/mod.rs:470`; `admits` is defined at `recall/scope.rs:152` and called from `recall/get.rs:153` and `recall/context.rs:142`. A grep for `push_exclusion`, `ProjectKeys::excluded` and `fn excluded(` over `crates/` returns nothing, and `excluded_projects` and `excluded_pre_worktree` at `scope.rs:132` and `:137` carry no `pub`. `retention::is_excluded` at `retention/mod.rs:236` is a call to `visible::is_excluded`. The executor reports `cargo test --workspace --all-features` passing 1004 tests with no test or golden file line removed or changed; I did not rerun the suite myself. Nothing in the goal looks missing.
