---
phase: 2
plan: 2
requirements:
  - TAGS-01
files:
  - crates/terminus-core/src/recall/context.rs
  - crates/terminus-core/src/recall/excerpt.rs
  - crates/terminus-core/src/recall/get.rs
  - crates/terminus-core/src/recall/mod.rs
  - crates/terminus-core/src/recall/query.rs
  - crates/terminus-core/src/recall/scope.rs
  - crates/terminus-core/src/recall/search.rs
  - crates/terminus-core/src/index/entity.rs
  - crates/terminus-core/src/index/expand.rs
  - crates/terminus-core/src/index/mod.rs
  - crates/terminus-core/src/index/text.rs
  - crates/terminus-core/src/config.rs
  - crates/terminus-core/src/discover.rs
  - crates/terminus-core/src/project.rs
  - crates/terminus-core/src/capture.rs
  - crates/terminus-core/src/parse/mod.rs
  - crates/terminus-core/src/parse/record.rs
  - crates/terminus-core/src/error.rs
  - crates/terminus-core/src/lib.rs
  - crates/terminus-core/src/feedback/finalize.rs
  - crates/terminus-core/src/feedback/label.rs
  - crates/terminus-core/src/feedback/mod.rs
  - crates/terminus-core/src/feedback/replay.rs
  - crates/terminus-core/src/feedback/stats.rs
  - crates/terminus-core/src/observe/egress.rs
  - crates/terminus-core/src/observe/mechanical.rs
  - crates/terminus-core/src/observe/mod.rs
  - crates/terminus-core/src/retention/apply.rs
  - crates/terminus-core/src/retention/mod.rs
  - crates/terminus-core/src/testkit.rs
---

# Phase 2: Planning tags - Plan 2 of 7: terminus-core sources, recall to testkit

## Goal

No comment in the recall, index, config, discovery, project, capture, parse, error, crate-root, feedback, observe, retention and testkit sources of `crates/terminus-core/src` cites a tag from the removed planning files, and each comment that carried one now states its rule itself (#106). Plan 1 covers the rest of that directory.

## Must be true when done

- Tag check A and tag check B (Context), run over this plan's files, print nothing.
- Every line changed in this plan's files since 1c55fec is a `//`, `///` or `//!` comment line. The diff check (Context) over the plan's files prints nothing.
- A comment that cited old phase order now states a current fact. Retention, the injection threshold and the hook's argv are described as the code has them, not as "phase N will".
- `cargo check --workspace --all-features --all-targets` exits 0, and `cargo test -p terminus-core --test supported_systems` passes against the reworded `lib.rs` module doc.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)", "(STOR-07, D-06)" or "(phase 8 D-03)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-06 fixes what is compared", "AC1's block counts", a leading "INJ-06:") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer a present-tense statement of what the code does over saying which phase or milestone did what. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or `//! #` heading whose only content is a tag a topic name, and drop the tag from one that already has a topic.
- **D-12:** rewrap by hand to the width the surrounding comment uses. Never run `cargo fmt` or rustfmt; five files carry older formatting diffs (`.planning/phases/1/SUMMARY.md`).
- Edit comment lines only: no code line, identifier, string literal or blank line changes. Find each site by its text, not by a CONTEXT line number (those are at 1c55fec). The extra families are D-03's: capital "Phase N", `PLAN-N`, "ROADMAP criterion" and "hardin".
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. Together they read every line, string literals included, and cover the guard's rules for comments (plan 7), so a file that passes them passes the guard. A plan-time sweep found no real name of the guard's generic shape in this tree.
- **Diff check:** `git diff -U0 1c55fec -- <files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'` prints every changed line that is not a comment line.

## Tasks

### Task 1: Cut the tags from the recall and index comments

- **Files:** crates/terminus-core/src/recall/context.rs, crates/terminus-core/src/recall/excerpt.rs, crates/terminus-core/src/recall/get.rs, crates/terminus-core/src/recall/mod.rs, crates/terminus-core/src/recall/query.rs, crates/terminus-core/src/recall/scope.rs, crates/terminus-core/src/recall/search.rs, crates/terminus-core/src/index/entity.rs, crates/terminus-core/src/index/expand.rs, crates/terminus-core/src/index/mod.rs, crates/terminus-core/src/index/text.rs
- **Action:** Apply the Context rules to the 108 tagged lines in these files.
  - The `PLAN-N` lines in `recall/mod.rs` ("`terminus search` (PLAN-3) and `recall_search` (PLAN-4)") and `recall/search.rs` lose the parentheticals.
  - "Phase 5's structural rank-1..3 threshold" in `index/text.rs` and `recall/search.rs` becomes the injection's threshold, stated as current.
  - "Phase 1 indexed `String::from_utf8_lossy(record)`" in `index/text.rs`'s module doc becomes a statement of why the current code indexes what it does. Indexing the whole record would cause the problem the comment describes; no history of who did it is needed.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 2: Cut the tags from the config, discovery, project, capture, parse and error comments

- **Files:** crates/terminus-core/src/config.rs, crates/terminus-core/src/discover.rs, crates/terminus-core/src/project.rs, crates/terminus-core/src/capture.rs, crates/terminus-core/src/parse/mod.rs, crates/terminus-core/src/parse/record.rs, crates/terminus-core/src/error.rs, crates/terminus-core/src/lib.rs
- **Action:** Apply the Context rules to the 137 tagged lines in these files; 48 of the 71 in `config.rs` are bare parentheticals.
  - In `config.rs`, "Phase 3's search, phase 5's injection and ..." names the current readers without phase numbers.
  - `config.rs` has one `///` block fenced as `text`. Lines inside it are still comment lines, so the same rules apply.
  - `crates/terminus-core/tests/supported_systems.rs` reads `lib.rs` and matches its code lines by exact text and position relative to each other. Change only `//!` lines in `lib.rs`.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0, and `cargo test -p terminus-core --test supported_systems` passes.

### Task 3: Cut the tags from the feedback, observe, retention and testkit comments

- **Files:** crates/terminus-core/src/feedback/finalize.rs, crates/terminus-core/src/feedback/label.rs, crates/terminus-core/src/feedback/mod.rs, crates/terminus-core/src/feedback/replay.rs, crates/terminus-core/src/feedback/stats.rs, crates/terminus-core/src/observe/egress.rs, crates/terminus-core/src/observe/mechanical.rs, crates/terminus-core/src/observe/mod.rs, crates/terminus-core/src/retention/apply.rs, crates/terminus-core/src/retention/mod.rs, crates/terminus-core/src/testkit.rs
- **Action:** Apply the Context rules to the 80 tagged lines in these files.
  - D-10: in `feedback/finalize.rs`'s module doc, "the hook argv phases 4 and 5 settled means a spawned pass cannot know which session just ended" becomes the current fact. The hook starts the pass as a bare `ingest` (`spawn::detached(&["ingest"])` in `crates/terminus/src/cmd/hook.rs`), with no session id, so the pass cannot know which session ended.
  - In `testkit.rs`, "`session-redaction.jsonl` is the hardin phase 2 one" names the fixture by what it holds: the redaction shapes.
  - `observe/egress.rs` carries formatting diffs from before this phase (D-12). Edit it by hand only.
  - The SQL `--` lines in `feedback/label.rs`'s query carry no tag. Leave them as they are.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another, so they can run in any order. Plan 7, the guard, builds on all six and runs last.
- Plans 1 and 2 split `crates/terminus-core/src` between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 466,163.
- Tagged-line counts per task come from tag checks A and B at cc1d91d, which equals 1c55fec for every file outside `.planning/`.
- `cargo fmt --check` already reports diffs in files no plan touched (`.planning/phases/1/SUMMARY.md`, phase 1); D-12 keeps this phase from adding or removing any.
