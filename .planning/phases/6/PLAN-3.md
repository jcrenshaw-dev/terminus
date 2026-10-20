---
phase: 6
plan: 3
requirements:
  - "#90"
files:
  - crates/terminus/src/cmd/status.rs
  - crates/terminus/src/cmd/verify.rs
  - crates/terminus-core/src/inject/brief.rs
  - crates/terminus-core/src/recall/scope.rs
  - crates/terminus-core/src/testkit.rs
  - crates/terminus/tests/cli.rs
  - crates/terminus/tests/status.rs
---

# Phase 6: Simplify the 1.0.0 changes - Plan 3 (read side and tests)

## Goal

A simplify pass over the code the 1.0.0 fixes change, before the README and the docs sweep describe it (#90). It removes code that reinvents something the codebase already has, code that can be simpler, and wasted work, and it changes no behavior. This plan applies SIMPLIFY.md findings 10, 11, 13 and 16: the `status` and `verify` no-store documents, the brief's guard, `Scoped`'s accessors, and the shared test helpers.

## Must be true when done

- `cargo test --workspace --all-features --no-fail-fast` reports 0 failed, and `git diff` from the commit this plan started from adds, removes or changes no line containing `assert`.
- With no store, `terminus status --json` and `terminus verify --json` print the same bytes as before. Each document is now built by the same function as the real one, fed zero and null values, the way `stats` and `usage` already build theirs.
- The brief's guard works out the room after the frame once, and `frame_cost(DESCRIPTION)` appears once in `brief.rs`.
- `Scoped` has no `excluded_projects()` or `excluded_pre_worktree()` method, and its fields stay private.
- `schema` and `age` are defined once, in `terminus_core::testkit`, and the inline aging in `cli.rs` calls `age`.
- `git diff --name-only` from the commit this plan started from lists only the seven files in this plan's frontmatter.

## Context

- `.planning/phases/6/SIMPLIFY.md` stands in for CONTEXT.md. Each task applies one finding under `## Apply` exactly as written. Nothing under `## Skip` is planned: not the `try_clip` beside `clip` (skip 12), the no-store assertion helper (skip 17), or the other test helpers (skips 19 to 22).
- Binding on every task: no behavior change, every existing test passes, and no assertion changes. Finding 16 moves test helpers only (SIMPLIFY.md `## Criterion 2`).
- Earlier decisions kept: phase 2 D-05 (each no-store field at zero or null), phase 3 D-05, D-06 and D-12 (the filter's SQL text, the per-list skip, and lists private to `scope.rs`), and phase 5 D-05 (`clip` does not change).
- Out of scope: `inject/mod.rs`'s `frame_fits` and `frame_cost`, core's `verify.rs` and `config.rs`, and every test in `cli.rs` and `status.rs` other than the two named in Task 4.

## Tasks

### Task 1: Build the no-store documents with the real builder (finding 10)

- **Files:** crates/terminus/src/cmd/status.rs (`run`, `nothing`), crates/terminus/src/cmd/verify.rs (`run`)
- **Action:** In `status.rs`, move the `Document::new("status")` chain from `run`'s `--json` arm into a function in the same file that takes each field's value and returns the `Document`. The store path and the exclusion list come in as options, so a missing one is written as JSON null. The size, a `visible::Counts`, and the last run also come in, the last run as an option. The `--json` arm calls it with the values it uses now (`store.path().display().to_string()`, `size_bytes(data_dir)`, `counts`, `config.exclusions()`, `last`) and emits the document. The no-store arm calls it with no path, size 0, `visible::Counts::default()` (`Counts` derives `Default` in `crates/terminus-core/src/config.rs`), no exclusion list and no run, in place of `nothing()`. Delete `nothing()`. Put its point, that the no-store call gives every field at zero or null (phase 2 D-05), in the new function's doc comment. Field names, their JSON types and their values stay the same.
  In `verify.rs`, move the `checked` and `failures` fields into a function over `&verify::Report` that returns the `Document`. The `--json` arm calls it with the report and then applies `.failed().because(...)` when `!report.is_ok()`, with the same reason text as now. The no-store arm calls it with `&verify::Report::default()` (`Report` derives `Default` in `crates/terminus-core/src/verify.rs`), which gives `checked` 0 and `failures` empty. This is how `stats` (`document(&Stats::default())`) and `usage` (`document(&Usage::default())`) build theirs. The text output of both commands does not change.
- **Verify:** Before editing, build the binary and capture `terminus status --json` and `terminus verify --json` with `TERMINUS_DATA_DIR` set to a path that does not exist and `TERMINUS_CONFIG_DIR` set to an empty scratch directory. After the change, capture the same two commands the same way. `cmp` shows each pair is identical. `grep -n "fn nothing" crates/terminus/src/cmd/status.rs` prints nothing. `grep -c 'Document::new("status")' crates/terminus/src/cmd/status.rs` and `grep -c 'Document::new("verify")' crates/terminus/src/cmd/verify.rs` each print 1. `cargo test -p terminus --all-features --test status --test cli --test tighten` reports 0 failed. The passes include `status_json_with_no_store_is_a_reason_zeroes_and_nulls`, `status_json_carries_the_same_numbers_as_the_human_output`, and the `cli.rs` tests that call `assert_verify_says_no_store`.

### Task 2: One threshold in the brief's guard (finding 11)

- **Files:** crates/terminus-core/src/inject/brief.rs (`render`)
- **Action:** In `render`, two things work out the room after the frame now: the guard `!super::frame_fits(DESCRIPTION, budget) || budget - super::frame_cost(DESCRIPTION) <= chars(excerpt::ELISION)`, and the `budget - super::frame_cost(DESCRIPTION)` after it. Replace both with one computation: `budget.checked_sub(super::frame_cost(DESCRIPTION))`, filtered to a room larger than `chars(excerpt::ELISION)`, returning `None` when nothing is left. Use that room where the second subtraction's result is used now, which is the rebinding of `budget` after `framed`. The two forms agree on every budget. `frame_fits` is `frame_cost(description) < budget` (`inject/mod.rs`), so the only budget it refuses that `checked_sub` lets through is `budget == frame_cost`. There the room is 0, and the filter refuses it. `framed` stays the full budget passed to `super::frame`. `frame_fits` and `frame_cost` in `inject/mod.rs` do not change, and `frame_fits` keeps its callers there and in `prompt.rs`. `clip` is not touched (phase 5 D-05, skip 12). The doc comment above `render` already describes this rule and stays as it is.
- **Verify:** `grep -c "frame_cost(DESCRIPTION)" crates/terminus-core/src/inject/brief.rs` prints 1. `grep -n "frame_fits" crates/terminus-core/src/inject/brief.rs` prints nothing. `cargo test -p terminus-core --all-features --test inject_brief --test redacted_recall --test inject_state --test recall_golden` and `cargo test -p terminus --all-features --test brief` report 0 failed. `git diff --stat -- 'crates/*/tests/goldens'` prints nothing.

### Task 3: Read `Scoped`'s fields directly (finding 13)

- **Files:** crates/terminus-core/src/recall/scope.rs (`Scoped::excluded_projects`, `Scoped::excluded_pre_worktree`, `Scoped::admits`, `Scoped::push_filter`)
- **Action:** Delete the private methods `excluded_projects()` and `excluded_pre_worktree()` and their doc comments. `admits` and `push_filter` read `self.excluded_projects` and `self.excluded_pre_worktree` directly, as slices where the code needs `&[String]`. The fields stay private, and that is what keeps phase 3 D-12's rule: no read site outside `scope.rs` can rebuild the filter from them. The clause text, the param order and the per-list skip in `push_filter` stay as they are (phase 3 D-05, D-06).
- **Verify:** `grep -rn "excluded_projects()\|excluded_pre_worktree()" crates` prints nothing. `grep -n "pub excluded\|pub(crate) excluded" crates/terminus-core/src/recall/scope.rs` prints nothing. `cargo test -p terminus-core --all-features --test exclusion --test recall --test inject_brief --test inject_prompt --test retention` reports 0 failed.

### Task 4: One schema snapshot and one aging step for the tests (finding 16)

- **Files:** crates/terminus-core/src/testkit.rs, crates/terminus/tests/cli.rs (`schema`, `verify_on_an_older_store_says_so_and_changes_nothing`), crates/terminus/tests/status.rs (`schema`, `age`)
- **Action:** Move `schema`, which is identical in `cli.rs` and `status.rs`, and `status.rs`'s `age` into `terminus_core::testkit` as public functions. Keep their names, signatures and bodies. Write the rusqlite names the way testkit writes them (`rusqlite::Connection`, `rusqlite::OpenFlags`), and use `crate::DERIVED_SCHEMA` and `crate::store::META_DERIVED_SCHEMA` where `age` names them through `terminus_core::`. In `age`'s doc comment, drop "`status` never reads them", since the function now serves both files. Keep what the store is and that a writable open would add the columns back. Delete both `schema` copies and `status.rs`'s `age`, and call `testkit::schema` and `testkit::age`. Both files already have `use terminus_core::testkit`. In `cli.rs`'s `verify_on_an_older_store_says_so_and_changes_nothing`, the inline block runs `bench.conn()`, the `UPDATE meta`, the two `ALTER TABLE decisions DROP COLUMN` statements and `drop(conn)`. Replace it with `testkit::age` on the database path, and set `path` before that call. The SQL is the same. Both versions open the file with `Connection::open` and close it before `schema` reads. No assertion changes. Remove any import the move leaves unused in either test file, and nothing else. The `UPDATE meta` in `json_mode_keeps_stdout_pure_and_warnings_on_stderr` only lowers `derived_schema`, so it is not the same step, and it stays.
- **Verify:** `grep -n "fn schema\|fn age" crates/terminus/tests/cli.rs crates/terminus/tests/status.rs` prints nothing. `grep -n "pub fn schema\|pub fn age" crates/terminus-core/src/testkit.rs` prints two lines. `git diff HEAD -- crates/terminus/tests | grep -E '^[-+][^-+]' | grep assert` prints nothing. `cargo test -p terminus --all-features --test cli --test status` reports 0 failed, its passes include `verify_on_an_older_store_says_so_and_changes_nothing` and `status_on_an_older_store_says_so_and_changes_nothing`, and the build prints no `warning:` that names either test file. Then, for the plan as a whole: `cargo test --workspace --all-features --no-fail-fast` reports 0 failed. `git diff --name-only <the commit this plan started from>` lists only this plan's seven frontmatter files. `git diff <that commit> -- crates | grep -E '^[-+][^-+]' | grep assert` prints nothing. `cargo fmt --all --check` names none of the seven files.

## Notes

- Criterion 3 and finding 16: `crates/terminus-core/src/testkit.rs` is not in `git diff main...traders`. Finding 16 says to move the helpers into `terminus_core::testkit`, and John adjudicated it as Apply. The two helpers it moves were added by the traders diff. It is planned as written.
- Task 1's byte check runs the real binary against the user's environment with both directories redirected, so it reads no real store and no real config.
- SIMPLIFY.md's line numbers were checked against `traders` at 4c078c8 and all match: `status.rs:167`, `verify.rs:16-18`, `brief.rs:221-227`, `scope.rs:129-139`, `cli.rs:1049`, `:1101-1111`, `status.rs:470`, `:518`.
- Criterion 4 (`Fixes #90` in the landing PR body) is carried as a truth in PLAN-1.
