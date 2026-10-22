---
phase: 2
plan: 4
requirements:
  - TAGS-01
files:
  - crates/terminus-core/tests/inject_brief.rs
  - crates/terminus-core/tests/inject_prompt.rs
  - crates/terminus-core/tests/inject_state.rs
  - crates/terminus-core/tests/inject_compaction.rs
  - crates/terminus-core/tests/compaction.rs
  - crates/terminus-core/tests/feedback.rs
  - crates/terminus-core/tests/label.rs
  - crates/terminus-core/tests/observe.rs
  - crates/terminus-core/tests/config.rs
  - crates/terminus-core/tests/discover.rs
  - crates/terminus-core/tests/exclusion.rs
  - crates/terminus-core/tests/project.rs
  - crates/terminus-core/tests/lineage.rs
  - crates/terminus-core/tests/retention.rs
  - crates/terminus-core/tests/fixtures.rs
  - tests/fixtures/README.md
---

# Phase 2: Planning tags - Plan 4 of 7: terminus-core tests from injection on, and the fixture catalog

## Goal

No comment, assertion message or fixture-catalog line in the injection, compaction, feedback, observe, config, discovery, project, lineage, retention and fixture tests of `crates/terminus-core/tests`, or in `tests/fixtures/README.md`, cites a tag from the removed planning files. Each one that carried a tag now states its rule or its content itself (#106). Plan 3 covers the rest of that directory.

## Must be true when done

- Tag check A and tag check B (Context), run over every file in this plan's `files:` list, print nothing. The list is 15 of the 33 tagged files of `crates/terminus-core/tests`, plan 3 holding the other 18, plus `tests/fixtures/README.md`. Plan 7's guard file joins that directory later and holds `UTF-16` in its allow-list, which tag check A's generic pattern would match.
- Every line changed in this plan's `.rs` files since 1c55fec is a comment line or a line of one of the 6 `assert!`/`panic!` messages this plan names (D-06). The diff check prints nothing else.
- `tests/fixtures/README.md` describes each fixture by what it holds and what it proves, in prose, with no tag at the start of a table row.
- The test names `the_stored_document_carries_every_obs_01_fact_by_name` and every other identifier are unchanged, and `cargo check --workspace --all-features --all-targets` exits 0.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)" or "(STOR-07, D-06)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-01: this is why ...", "AC1's block counts") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer what the test proves, in the present tense, over which phase or acceptance criterion asked for it. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or heading whose only content is a tag a topic name, keeping its dash fill to the same width, and drop the tag from one that already has a topic.
- **D-06:** a named `assert!`/`panic!` message is reworded to say what failed. Keep every `{...}` placeholder and every macro argument, and keep a message that is only a tag ("AC7", "D-14", "AC4") as a message that describes the assertion. A message prints only on failure, so no test's behavior changes.
- **D-12:** rewrap by hand. Never run `cargo fmt` or rustfmt. Otherwise edit comment lines only: no code line, identifier, other string literal or blank line changes. Find each site by its text; CONTEXT line numbers are at 1c55fec.
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. **Diff check:** `git diff -U0 1c55fec -- <.rs files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'`.

## Tasks

### Task 1: Cut the tags from the injection and compaction tests

- **Files:** crates/terminus-core/tests/inject_brief.rs, crates/terminus-core/tests/inject_prompt.rs, crates/terminus-core/tests/inject_state.rs, crates/terminus-core/tests/inject_compaction.rs, crates/terminus-core/tests/compaction.rs
- **Action:** Apply the Context rules to the 63 tagged lines in these files.
  - Above `nothing_in_the_injection_path_spawns_a_process_or_reads_observations` in `inject_brief.rs`, "D-10 and D-18, asserted against the source" names the two rules (a D-09 example).
  - In the same comment, "only fail once phase 7 exists to make the table appear on some machines and not others" becomes the current fact about the `observations` table (D-10).
  - "AC4: a compaction boundary appearing under an already-archived session." in `compaction.rs`'s module doc keeps the description.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 2: Cut the tags from the feedback, label and observe tests

- **Files:** crates/terminus-core/tests/feedback.rs, crates/terminus-core/tests/label.rs, crates/terminus-core/tests/observe.rs
- **Action:** Apply the Context rules to the 47 tagged lines in these files.
  - "PLAN-2's labelling and PLAN-3's replay extend exactly this file" in `feedback.rs`'s module doc becomes what the file holds now.
  - The test name `the_stored_document_carries_every_obs_01_fact_by_name` in `observe.rs` stays (D-08). A comment calling these "OBS-01 facts" names the facts by what they are.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0, and `git grep -n the_stored_document_carries_every_obs_01_fact_by_name -- crates/terminus-core/tests/observe.rs` still prints the test's `fn` line.

### Task 3: Cut the tags from the config, discover, exclusion, project, lineage and retention tests

- **Files:** crates/terminus-core/tests/config.rs, crates/terminus-core/tests/discover.rs, crates/terminus-core/tests/exclusion.rs, crates/terminus-core/tests/project.rs, crates/terminus-core/tests/lineage.rs, crates/terminus-core/tests/retention.rs
- **Action:** Apply the Context rules to the 71 tagged lines in these files.
  - D-06 messages to reword in `config.rs`: "D-06 folds a worktree into its parent repo, so excluding the repo must ...", which continues over a `\` line break; "D-07: these are the same encoded name"; and "... name, and D-06 folds them into the excluded repo".
  - D-06 message to reword in `discover.rs`: "D-06 folds a worktree into its repo, so excluding the repo excludes it: {names:?}". It keeps `{names:?}`.
  - "D-07 from the other side" in `project.rs` names the rule.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints only lines of the four named messages. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 4: Reword the fixture catalog, fixtures.rs and tests/fixtures/README.md, without tags

- **Files:** crates/terminus-core/tests/fixtures.rs, tests/fixtures/README.md
- **Action:** Apply the Context rules to the 31 tagged lines of `fixtures.rs` and the 44 of `tests/fixtures/README.md`. The two files describe one fixture set, so keep their wording consistent.
  - In `fixtures.rs`, the banners "--- Phase 3 ---" and "--- Phase 5 ---" get topic names saying what the fixtures in that section serve, such as search or injection. Read the tests under each banner to name it, and keep the dash fill to the same width (D-11).
  - D-06 messages to reword in `fixtures.rs`: "the pair differs in something D-04 does not normalize: {x} vs {y}", and the message continuing "INJ-03 nothing to fire on: {entities:?}".
  - In the README's fixture table, most description cells begin with a tag, such as "Phase 3 D-01/D-13:" or "hardin phase 2 D-13:". Each cell is reworded as prose saying what the fixture holds and proves (D-04). The table's columns and the fixture file names stay.
  - "Phase 5 D-08 builds" in the README's prose names what builds it.
  - "the hardin phase 2 redaction fixture" in the README names it by content.
  - D-10: in the README, "keeping the bytes lets phase 3 extract `description`" becomes the current fact, as `crates/terminus-core/src/store/schema.rs` will say. Ingest stores the bytes and checks only whether they are already stored. No code parses them, and keeping them lets `description` be extracted later without a reingest.
  - No test reads `tests/fixtures/README.md` (D-04).
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over `crates/terminus-core/tests/fixtures.rs` prints only lines of the two named messages. `grep -c '^|' tests/fixtures/README.md` prints the same count as `git show 1c55fec:tests/fixtures/README.md | grep -c '^|'`, so no table row was lost. `cargo check --workspace --all-features --all-targets` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another. Plan 7, the guard, builds on all six and runs last.
- Plans 3 and 4 split `crates/terminus-core/tests` between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 435,294.
- D-06 counts 23 tagged messages in all. This plan holds 6 of the 15 in `crates/terminus-core/tests`, plan 3 holds the other 9, and plan 6 holds the 8 in `crates/terminus/tests`.
- Task 4's README wording for `agent_meta` follows the fact plan 1 task 1 writes into `crates/terminus-core/src/store/schema.rs`. The Action states that fact in full, so this plan does not wait on plan 1.
- `tests/fixtures/hooks/session-start.json` and `tests/fixtures/hooks/user-prompt-submit.json` keep their "phase 4: hooks and install" titles (D-07), and no plan touches them.
