---
phase: 2
plan: 3
requirements:
  - TAGS-01
files:
  - crates/terminus-core/tests/schema.rs
  - crates/terminus-core/tests/store_open.rs
  - crates/terminus-core/tests/verify.rs
  - crates/terminus-core/tests/reindex.rs
  - crates/terminus-core/tests/snapshot.rs
  - crates/terminus-core/tests/sqlite_capabilities.rs
  - crates/terminus-core/tests/derive.rs
  - crates/terminus-core/tests/recover.rs
  - crates/terminus-core/tests/ingest.rs
  - crates/terminus-core/tests/parse.rs
  - crates/terminus-core/tests/pass.rs
  - crates/terminus-core/tests/backfill.rs
  - crates/terminus-core/tests/capture.rs
  - crates/terminus-core/tests/recall.rs
  - crates/terminus-core/tests/recall_golden.rs
  - crates/terminus-core/tests/index.rs
  - crates/terminus-core/tests/redacted_recall.rs
  - crates/terminus-core/tests/egress.rs
---

# Phase 2: Planning tags - Plan 3 of 7: terminus-core tests, store to recall

## Goal

No comment or assertion message in the store, schema, verify, reindex, ingest, parse, capture, recall, index and egress tests of `crates/terminus-core/tests` cites a tag from the removed planning files. Each one that carried a tag now states its rule itself (#106). Plan 4 covers the rest of that directory and `tests/fixtures/README.md`.

## Must be true when done

- Tag check A and tag check B (Context), run over every file in this plan's `files:` list, print nothing. The list is 18 of the 33 tagged files of `crates/terminus-core/tests`; plan 4 holds the other 15. Plan 7's guard file joins that directory later and holds `UTF-16` in its allow-list, which tag check A's generic pattern would match.
- Every line changed in this plan's `.rs` files since 1c55fec is a comment line or a line of one of the 9 `assert!`/`panic!` messages this plan names (D-06). The diff check prints nothing else.
- Every identifier is unchanged, and `cargo check --workspace --all-features --all-targets` exits 0.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)" or "(STOR-07, D-06)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-01: this is why ...", "AC1's block counts") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer what the test proves, in the present tense, over which phase or acceptance criterion asked for it. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or heading whose only content is a tag a topic name, keeping its dash fill to the same width, and drop the tag from one that already has a topic.
- **D-06:** a named `assert!`/`panic!` message is reworded to say what failed. Keep every `{...}` placeholder and every macro argument, and keep a message that is only a tag ("AC7", "D-14", "AC4") as a message that describes the assertion. A message prints only on failure, so no test's behavior changes.
- **D-12:** rewrap by hand. Never run `cargo fmt` or rustfmt. Otherwise edit comment lines only: no code line, identifier, other string literal or blank line changes. Find each site by its text; CONTEXT line numbers are at 1c55fec.
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. **Diff check:** `git diff -U0 1c55fec -- <.rs files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'`.

## Tasks

### Task 1: Cut the tags from the store, schema, verify and reindex tests

- **Files:** crates/terminus-core/tests/schema.rs, crates/terminus-core/tests/store_open.rs, crates/terminus-core/tests/verify.rs, crates/terminus-core/tests/reindex.rs, crates/terminus-core/tests/snapshot.rs, crates/terminus-core/tests/sqlite_capabilities.rs, crates/terminus-core/tests/derive.rs, crates/terminus-core/tests/recover.rs
- **Action:** Apply the Context rules to the 89 tagged lines in these files.
  - D-06 messages to reword: "rowid IS turns.id (D-10)" in `derive.rs`, "AC4" in `reindex.rs`, and "a parent created on the way keeps the umask's default (D-04)" in `store_open.rs`.
  - The banner "--- Phase 8: session_meta.capture_mode (ING-07, D-13, D-19) ---" in `schema.rs` keeps its topic, `session_meta.capture_mode`.
  - "Phase 8 D-03: the one case where the blob cannot answer." in `reindex.rs` keeps the topic.
  - "Phase 1 asserted the opposite here - that neither table ever gained a row -" in `derive.rs` becomes what the test asserts now (D-10).
  - "The rebuild itself is PLAN-2's." in `store_open.rs` either names where the rebuild is tested or is deleted.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints only lines of the three named messages. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 2: Cut the tags from the ingest, parse, pass, backfill and capture tests

- **Files:** crates/terminus-core/tests/ingest.rs, crates/terminus-core/tests/parse.rs, crates/terminus-core/tests/pass.rs, crates/terminus-core/tests/backfill.rs, crates/terminus-core/tests/capture.rs
- **Action:** Apply the Context rules to the 75 tagged lines in these files.
  - D-06 messages to reword: "AC7, for the capture fixture" in `capture.rs`; "AC7", "AC1: N blocks for a turn spanning N", "AC1: one block for a turn that fits in one" and "D-14" in `ingest.rs`; and "D-03 already classifies the boundary" in `parse.rs`.
  - The module doc of `ingest.rs` ("AC7's byte fidelity, AC1's block counts") names the properties themselves.
  - "Phase 1's `record_run` inserts a literal `1` for `files_seen`" in `pass.rs` is checked against the current code. Keep it as a present-tense fact if `record_run` still does that, and rewrite it as the current fact if not (D-10).
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints only lines of the six named messages. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 3: Cut the tags from the recall, index and egress tests

- **Files:** crates/terminus-core/tests/recall.rs, crates/terminus-core/tests/recall_golden.rs, crates/terminus-core/tests/index.rs, crates/terminus-core/tests/redacted_recall.rs, crates/terminus-core/tests/egress.rs
- **Action:** Apply the Context rules to the 81 tagged lines in these files.
  - D-10: above `an_evicted_session_returns_the_record_flagged_and_reads_none_of_its_rows` in `recall.rs`, "retention is phase 8 and nothing writes it before then" is false. `retention::apply` sets `session_meta.is_evicted` (its `UPDATE session_meta SET is_evicted = 1`). Say instead that the test sets the column directly so the blob stays intact and the flag is read off the column alone.
  - The four banners "--- RCL-01: expansion ---" to "--- RCL-04: the per-turn cap ---" in `index.rs` keep their topics.
  - "Phase 1 indexed" in `index.rs` becomes a statement of why the current code indexes what it does.
  - "one turn per shape hardin's phase 2 adds" in `egress.rs` names the redaction shapes without the milestone.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another. Plan 7, the guard, builds on all six and runs last.
- Plans 3 and 4 split `crates/terminus-core/tests` between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 469,061.
- D-06 counts 23 tagged messages in all. This plan holds 9 of the 15 in `crates/terminus-core/tests`, plan 4 holds the other 6, and plan 6 holds the 8 in `crates/terminus/tests`.
