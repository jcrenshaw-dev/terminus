---
phase: 2
plan: 1
requirements:
  - TAGS-01
files:
  - crates/terminus-core/src/store/schema.rs
  - crates/terminus-core/src/store/open.rs
  - crates/terminus-core/src/store/archive.rs
  - crates/terminus-core/src/store/snapshot.rs
  - crates/terminus-core/src/blob/block.rs
  - crates/terminus-core/src/blob/mod.rs
  - crates/terminus-core/src/owner_only.rs
  - crates/terminus-core/src/derive.rs
  - crates/terminus-core/src/reindex.rs
  - crates/terminus-core/src/verify.rs
  - crates/terminus-core/src/recover.rs
  - crates/terminus-core/src/lineage.rs
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/ingest/backfill.rs
  - crates/terminus-core/src/ingest/lock.rs
  - crates/terminus-core/src/inject/brief.rs
  - crates/terminus-core/src/inject/compaction.rs
  - crates/terminus-core/src/inject/decision.rs
  - crates/terminus-core/src/inject/mod.rs
  - crates/terminus-core/src/inject/prompt.rs
  - crates/terminus-core/src/inject/state.rs
---

# Phase 2: Planning tags - Plan 1 of 7: terminus-core sources, store to injection

## Goal

No comment in the store, blob, owner-only, derive, reindex, verify, recover, lineage, ingest and injection sources of `crates/terminus-core/src` cites a tag from the removed planning files, and each comment that carried one now states its rule itself (#106). Plan 2 covers the rest of that directory.

## Must be true when done

- Tag check A and tag check B (Context), run over this plan's files, print nothing.
- Every line changed in this plan's files since 1c55fec is a `//`, `///` or `//!` comment line, or a SQL `--` comment line inside `schema::CREATE_SQL`. The diff check (Context) over the plan's files prints nothing.
- A comment that cited old phase order now states a current fact. The `agent_meta` bytes and injection's read of the session index are described as the code has them, not as "phase N will".
- `cargo check --workspace --all-features --all-targets` exits 0, and the tests that open a store and read `sqlite_master` pass with the new `CREATE_SQL` comment text.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)", "(STOR-07, D-06)" or "(phase 8 D-03)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-06 fixes what is compared", "AC1's block counts", a leading "INJ-06:") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer a present-tense statement of what the code does over saying which phase or milestone did what. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or `//! #` heading whose only content is a tag a topic name, and drop the tag from one that already has a topic.
- **D-12:** rewrap by hand to the width the surrounding comment uses. Never run `cargo fmt` or rustfmt; five files carry older formatting diffs (`.planning/phases/1/SUMMARY.md`).
- Edit comment lines only: no code line, identifier, string literal or blank line changes. Find each site by its text, not by a CONTEXT line number (those are at 1c55fec). The extra families are D-03's: capital "Phase N", `PLAN-N`, "ROADMAP criterion" and "hardin".
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. Together they read every line, string literals included, and cover the guard's rules for comments (plan 7), so a file that passes them passes the guard. A plan-time sweep found no real name of the guard's generic shape in this tree.
- **Diff check:** `git diff -U0 1c55fec -- <files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'` prints every changed line that is not a comment line.

## Tasks

### Task 1: Cut the tags from the store, blob and owner-only comments, CREATE_SQL included

- **Files:** crates/terminus-core/src/store/schema.rs, crates/terminus-core/src/store/open.rs, crates/terminus-core/src/store/archive.rs, crates/terminus-core/src/store/snapshot.rs, crates/terminus-core/src/blob/block.rs, crates/terminus-core/src/blob/mod.rs, crates/terminus-core/src/owner_only.rs
- **Action:** Apply the Context rules to the 104 tagged lines in these files. In `schema.rs`, begin at the `CREATE_SQL` constant. Its 41 SQL `--` lines (52 tags) count as comment lines and are cleaned (D-05). Each rewrapped SQL comment line must still begin with `--` at the indentation it had. It must contain no `"` and no `\`, and must not end in `\`: the comment sits inside an ordinary Rust string, and a stray backslash would escape a character or join the next SQL line into the comment. SQLite keeps these comments in `sqlite_master.sql`, so new stores carry the new text and existing archive tables keep the old text through `IF NOT EXISTS`. That is expected and needs no handling. D-10 sites:
  - On the `agent_meta` column, "keeping the bytes lets phase 3 extract `description`" becomes the current fact. Ingest stores the bytes and checks only whether they are already stored (`agent_meta_stored` in `ingest/mod.rs`). No code parses them, and keeping them means `description` can be extracted later without a reingest.
  - The comment above `idx_session_meta_project` says "phase 5's injection will read it on every prompt" and "D-19: ... phase 3's `DERIVED_SCHEMA` bump forces that reindex". Rewrite it to say what reads the column now. `inject::brief` reads the scoped project's sessions through this index; check the prompt path before claiming "every prompt". Also say that the index reaches an existing store only because `reindex` runs the whole batch and a `DERIVED_SCHEMA` bump forces that reindex.
  - "Phase 7 D-01, on the same terms" near the end of `schema.rs`.
  
  `store/open.rs` includes the four D-NN lines phase 1 added (CONTEXT D-01).
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0. `cargo test -p terminus-core --all-features --test schema --test store_open --test reindex` passes, and so does `cargo test -p terminus --all-features --test status --test cli`; between them they hold the three `sqlite_master.sql` readers D-05 names.

### Task 2: Cut the tags from the derive, reindex, verify, recover and lineage comments

- **Files:** crates/terminus-core/src/derive.rs, crates/terminus-core/src/reindex.rs, crates/terminus-core/src/verify.rs, crates/terminus-core/src/recover.rs, crates/terminus-core/src/lineage.rs
- **Action:** Apply the Context rules to the 70 tagged lines in these files.
  - In `verify.rs`'s module doc, rewrite "D-06 fixes what is compared" (a D-09 example), "Phase 2 adds one further failure, and it passes the same test: D-13's ..." and "Phase 8 adds one *silence*" as present-tense statements of what `verify` reports and what it stays silent on.
  - `reindex.rs`'s heading "# The one case where the blob cannot answer (phase 8 D-03)" loses its parenthetical (D-09, D-11).
  - In `recover.rs`, "the damaged state phase 1 refuses to guess at" names what refuses now: recovery, the pass or whatever the code shows.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 3: Cut the tags from the ingest comments

- **Files:** crates/terminus-core/src/ingest/mod.rs, crates/terminus-core/src/ingest/pass.rs, crates/terminus-core/src/ingest/backfill.rs, crates/terminus-core/src/ingest/lock.rs
- **Action:** Apply the Context rules to the 137 tagged lines in these files.
  - In `ingest/mod.rs`, "Phase 1 refuses rather than guessing" names the refusing code as it is now.
  - `ingest/backfill.rs` has three `//! #` headings with tags: "(D-11)", "the \"chunked\" half of ING-11" and "(D-22)". Each keeps its topic and loses the tag; the middle one is reworded so it says which half it means (D-11 of CONTEXT).
  - In `ingest/lock.rs`, "which is D-15 named" is reworded to state the choice itself.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 4: Cut the tags from the injection comments

- **Files:** crates/terminus-core/src/inject/brief.rs, crates/terminus-core/src/inject/compaction.rs, crates/terminus-core/src/inject/decision.rs, crates/terminus-core/src/inject/mod.rs, crates/terminus-core/src/inject/prompt.rs, crates/terminus-core/src/inject/state.rs
- **Action:** Apply the Context rules to the 159 tagged lines in these files.
  - In `inject/mod.rs`, "(`crates/terminus/src/cmd/hook.rs`, PLAN-1 task 5)" keeps the path and loses the plan reference.
  - In `inject/state.rs`, "what INJ-04 has to distinguish" is reworded to name what the state has to distinguish.
  - `tests/inject_brief.rs` reads this directory's source with comment lines filtered out, so only comment lines may change here; a changed code line would also change what that test sees.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another, so they can run in any order. Plan 7, the guard, builds on all six and runs last.
- Plans 1 and 2 split `crates/terminus-core/src` between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 444,197.
- Tagged-line counts per task come from tag checks A and B at cc1d91d, which equals 1c55fec for every file outside `.planning/`.
- `cargo fmt --check` already reports diffs in files no plan touched (`.planning/phases/1/SUMMARY.md`, phase 1); D-12 keeps this phase from adding or removing any.
