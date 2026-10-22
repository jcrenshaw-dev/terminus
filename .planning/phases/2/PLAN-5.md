---
phase: 2
plan: 5
requirements:
  - TAGS-01
files:
  - crates/terminus/src/cmd/install/binary.rs
  - crates/terminus/src/cmd/install/claude.rs
  - crates/terminus/src/cmd/install/json_file.rs
  - crates/terminus/src/cmd/install/mod.rs
  - crates/terminus/src/cmd/install/targets.rs
  - crates/terminus/src/cmd/hook.rs
  - crates/terminus/src/cmd/spawn.rs
  - crates/terminus/src/cmd/uninstall.rs
  - crates/terminus/src/cmd/data.rs
  - crates/terminus/src/cmd/doctor.rs
  - crates/terminus/src/cmd/backfill.rs
  - crates/terminus/src/cmd/ingest.rs
  - crates/terminus/tests/install.rs
  - crates/terminus/tests/uninstall.rs
  - crates/terminus/tests/lifecycle.rs
  - crates/terminus/tests/datamove.rs
  - crates/terminus/tests/owner_only.rs
  - crates/terminus/tests/tighten.rs
  - crates/terminus/tests/doctor.rs
  - crates/terminus/tests/hook.rs
  - crates/terminus/tests/lock_race.rs
  - crates/terminus/tests/brief.rs
  - crates/terminus/tests/prompt.rs
  - crates/terminus/tests/inject.rs
  - crates/terminus/tests/suppression.rs
  - crates/terminus/tests/decisions.rs
  - crates/terminus/tests/replay.rs
---

# Phase 2: Planning tags - Plan 5 of 7: the terminus crate's install, hook and injection side

## Goal

No comment in this plan's files cites a tag from the removed planning files: the install, hook, uninstall, data, doctor, backfill and ingest commands of `crates/terminus/src`, and the install, on-disk, hook and injection tests of `crates/terminus/tests`. Each one that carried a tag now states its rule itself (#106). Plan 6 covers the rest of the crate, `docs/json-shapes.md` and the manifests.

## Must be true when done

- Tag check A over this plan's files prints nothing. Tag check B over them prints exactly two lines: the `session_title` values "phase 3: owner-only on disk" in `crates/terminus/tests/owner_only.rs` and "phase 5: context injection" in `crates/terminus/tests/prompt.rs` (D-07).
- Every line changed in this plan's `.rs` files since 1c55fec is a comment line. D-06 names no tagged `assert!` message in these files, so the diff check prints nothing.
- `cargo check --workspace --all-features --all-targets` exits 0.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)" or "(D-08, INST-02)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-02: install run through the link ...", "AC1's export clause") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer a present-tense statement of the behavior over which phase or criterion asked for it. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or `//! #` heading whose only content is a tag a topic name, and drop the tag from one that already has a topic.
- **D-07:** the two `session_title` strings stay. No `assert!` message in this plan's files carries a tag, so no string literal changes.
- **D-12:** rewrap by hand. Never run `cargo fmt` or rustfmt; `crates/terminus/src/cmd/hook.rs` and `crates/terminus/tests/hook.rs` carry formatting diffs from before this phase. Otherwise edit comment lines only: no code line, identifier, other string literal or blank line changes. Find each site by its text; CONTEXT line numbers are at 1c55fec.
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. **Diff check:** `git diff -U0 1c55fec -- <.rs files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'`.

## Tasks

### Task 1: Cut the tags from the install, hook, uninstall, data, doctor, backfill and ingest command comments

- **Files:** crates/terminus/src/cmd/install/binary.rs, crates/terminus/src/cmd/install/claude.rs, crates/terminus/src/cmd/install/json_file.rs, crates/terminus/src/cmd/install/mod.rs, crates/terminus/src/cmd/install/targets.rs, crates/terminus/src/cmd/hook.rs, crates/terminus/src/cmd/spawn.rs, crates/terminus/src/cmd/uninstall.rs, crates/terminus/src/cmd/data.rs, crates/terminus/src/cmd/doctor.rs, crates/terminus/src/cmd/backfill.rs, crates/terminus/src/cmd/ingest.rs
- **Action:** Apply the Context rules to the 140 tagged lines in these files, most of them bare parentheticals.
  - These `//! #` headings keep their topics and lose their tags (D-09, D-11):
    - `install/binary.rs`: "(D-08, INST-02)" and "(D-07)".
    - `install/json_file.rs`: "(phase 3 D-12)".
    - `install/targets.rs`: "(D-05)", "(D-01, ING-10)", "(INST-05)" and "(phase 3 D-03)".
    - `spawn.rs`: "(D-03)" and "(D-04)".
  - In `cmd/hook.rs`'s module doc, "Phase 5 needs `session_id`, `transcript_path`, `cwd`, `prompt` and ..." says which hook path needs those fields now.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 2: Cut the tags from the install, uninstall, on-disk and hook tests

- **Files:** crates/terminus/tests/install.rs, crates/terminus/tests/uninstall.rs, crates/terminus/tests/lifecycle.rs, crates/terminus/tests/datamove.rs, crates/terminus/tests/owner_only.rs, crates/terminus/tests/tighten.rs, crates/terminus/tests/doctor.rs, crates/terminus/tests/hook.rs, crates/terminus/tests/lock_race.rs
- **Action:** Apply the Context rules to the 108 tagged lines in these files.
  - The `session_title` value "phase 3: owner-only on disk" in `owner_only.rs` stays as it is (D-07). It is hook payload data, and nothing reads `session_title`.
  - "AC1's export clause" in `owner_only.rs` states the property.
- **Verify:** Tag check A over this task's Files prints nothing. Tag check B over them prints only the `"session_title": "phase 3: owner-only on disk"` line of `owner_only.rs`. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 3: Cut the tags from the brief, prompt, inject, suppression, decisions and replay tests

- **Files:** crates/terminus/tests/brief.rs, crates/terminus/tests/prompt.rs, crates/terminus/tests/inject.rs, crates/terminus/tests/suppression.rs, crates/terminus/tests/decisions.rs, crates/terminus/tests/replay.rs
- **Action:** Apply the Context rules to the 67 tagged lines in these files.
  - The `session_title` value "phase 5: context injection" in `prompt.rs` stays as it is (D-07).
  - "AC2. Two `SessionStart` runs against an unchanged store, byte for byte." in `brief.rs` keeps the description.
  - "The three prompts AC1 names" in `decisions.rs` describes the prompts.
- **Verify:** Tag check A over this task's Files prints nothing. Tag check B over them prints only the `"session_title": "phase 5: context injection"` line of `prompt.rs`. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another. Plan 7, the guard, builds on all six and runs last.
- Plans 5 and 6 split the terminus crate between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 631,610. It takes the install, hook and lifecycle commands with the install, on-disk, hook and injection tests. Plan 6 takes the read, report and MCP commands with the CLI, corpus, crash, recall and MCP tests, `docs/json-shapes.md` and the manifests.
