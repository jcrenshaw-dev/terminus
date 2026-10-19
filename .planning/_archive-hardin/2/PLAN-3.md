---
phase: 2
plan: 3
requirements: [PRIV-08, PRIV-06, PRIV-07]
files:
  - crates/terminus/tests/mcp.rs
  - crates/terminus/tests/brief.rs
  - crates/terminus/tests/prompt.rs
  - crates/terminus/tests/recall_cli.rs
  - README.md
  - THREAT-MODEL.md
  - docs/configuration.md
  - docs/design/0006-recall-through-mcp.md
  - CHANGELOG.md
---

# Phase 2: Redaction - Plan 3

## Goal

With `[privacy] redact_recall` on, the MCP server, both injection hooks, `terminus search` and `terminus show` answer normally on a turn holding `café=1`, `Größe=10` and `“token=abc”`, and a test at each process boundary proves it (#61). The docs say what redaction now matches and what it still misses (#5, #62).

## Must be true when done

- With the knob on, `recall_search`, `recall_context` and `recall_get` over the fixture's multi-byte turn each answer, the server exits 0 with no panic on stderr, and each answer holds `café=1` and `Größe=10` unchanged.
- With the knob on, the `SessionStart` hook emits its brief and the `UserPromptSubmit` hook emits its injection for that turn. Each holds `café=1` and `Größe=10`, and neither run prints a panic on stderr.
- With the knob on, `terminus search` and `terminus show` print that turn and exit 0, with `café=1` and `Größe=10` unchanged and no panic on stderr.
- Each of those tests fails when PLAN-1's rule 5 fix is reverted.
- README.md has no "Current state" section. THREAT-MODEL.md, docs/configuration.md and CHANGELOG.md no longer say a bare `sk-` token is missed, and they describe keyed values being replaced. `docs/design/0006-recall-through-mcp.md` describes nine rules and the keyed-value step.

## Context

- Locked: D-11 (a test at every boundary the roadmap names; the hook tests check that the injection is present and that stderr has no panic, because the hook catches the unwind and exits 0), D-12 (no `catch_unwind` around MCP requests), D-14 (which docs change; SECURITY.md stays as it is).
- Pattern: in mcp.rs, `Bench::call` runs `Conversation::expect_ok`, which already fails on a non-zero exit or `panicked` on stderr, and `authz_turn` finds a turn by its record uuid. In recall_cli.rs, `run_in`, `stdout` and `stderr`. In brief.rs, `Bench::ingest`, `Bench::project`, `payload`, `Bench::hook` and `additional_context`. In prompt.rs, `Bench::hook`, `payload` and `injected`. Each file has its own `KNOB_ON` or writes the same table.
- Out of this plan: product code. No boundary tests for the keyed and `sk-` rules beyond what PLAN-2 has, and no doc outside D-14's list.

## Tasks

### Task 1: Prove the three MCP tools answer the multi-byte turn (D-11, D-12)

- **Files:** crates/terminus/tests/mcp.rs (beside the_three_tools_answer_the_planted_secret_until_the_knob_is_turned_on)
- **Action:** Add a test that archives the corpus through `Bench::ingest_fixtures`, writes `KNOB_ON`, stands in `project-epsilon`'s directory, and calls the three tools on `session-redaction.jsonl`'s multi-byte turn. Find that turn by its record uuid, the way `authz_turn` does and for its reason: a search would pick the test's subject out of the filtered text under test. Call `recall_search` for that turn's own word, `recall_context` anchored on it, and `recall_get` for its id. `Bench::call` already fails on a non-zero exit or a panic on stderr, which is AC3's panic half. Add that each answer (the hit's excerpt, the anchor's text, the body) holds `café=1` and `Größe=10` byte for byte. Add no `catch_unwind` to the server (D-12); the fix is PLAN-1's.
- **Verify:** `cargo test -p terminus --all-features --test mcp` passes. With PLAN-1 Task 1's `rfind` name walk temporarily put back in `redact_assignments`, the new test fails on the server's panic, and restoring the fix makes it pass (AC5). `git diff --stat` lists mcp.rs alone.

### Task 2: Prove both injection hooks inject on the multi-byte turn (D-11)

- **Files:** crates/terminus/tests/brief.rs (beside the_knob_filters_the_briefs_quoted_prompt_at_the_hook_boundary), crates/terminus/tests/prompt.rs (Bench::ingest; Bench::project)
- **Action:** In brief.rs, add a test that archives `session-redaction.jsonl` through `Bench::ingest`, writes the knob to the bench's config directory, runs `SessionStart` with `payload` for `project-epsilon`'s directory, and asserts through `additional_context` that the brief was emitted and holds `café=1` and `Größe=10`. The brief quotes the multi-byte turn because it is the session's last assistant record. Assert too that the run's stderr does not contain `panicked`. In prompt.rs, `Bench::ingest` and `Bench::project` are bound to `FIXTURE`; give the new test a way to archive and stand in `session-redaction.jsonl` by generalizing them or adding siblings, with every existing test calling what it calls today. The test writes the knob, sends a `UserPromptSubmit` whose `cwd` is `project-epsilon`'s directory and whose prompt names the multi-byte turn's file relative to it, and asserts through `injected` that the injection is present and holds `café=1` and `Größe=10`, and that stderr does not contain `panicked`. The stderr checks are required (D-11): `cmd::hook`'s `inject` catches the unwind and exits 0 with nothing injected, so a check on the exit status alone passes before the fix.
- **Verify:** `cargo test -p terminus --all-features --test brief --test prompt` passes, every existing test in both files included. With PLAN-1's rule 5 fix reverted, both new tests fail (nothing injected, and the panic message on stderr), and restoring the fix makes them pass (AC5). `git diff --stat` lists the two files alone.

### Task 3: Prove `terminus search` and `terminus show` print the multi-byte turn (D-11)

- **Files:** crates/terminus/tests/recall_cli.rs (beside the_knob_filters_the_search_excerpt_and_raw_reaches_past_it)
- **Action:** Add a test that archives the corpus through `Bench::ingest_fixtures`, writes `KNOB_ON`, stands in `project-epsilon`'s directory, and runs `terminus search` for the multi-byte turn's own word and `terminus show` with that turn's id, found by its record uuid the way `authz_turn` finds its turn. Each run exits 0, its stderr does not contain `panicked`, and its stdout holds `café=1` and `Größe=10`. Assert on the human output, which is what AC3 names.
- **Verify:** `cargo test -p terminus --all-features --test recall_cli` passes. With PLAN-1's rule 5 fix reverted, the new test fails on a non-zero exit with the panic on stderr, and restoring the fix makes it pass (AC5). `git diff --stat` lists recall_cli.rs alone.

### Task 4: Say in the docs what redaction now matches and misses (D-14, AC6)

- **Files:** README.md ("Current state"), THREAT-MODEL.md (the Redaction bullets "What it matches" and "What it misses"), docs/configuration.md (the `redact_recall` bullet under `[privacy]`), docs/design/0006-recall-through-mcp.md (the `egress.rs` row of the file table; "The egress filter" Outputs bullet), CHANGELOG.md (`[Unreleased]`)
- **Action:** Read `~/.claude/john-voice.md` in full first, which CLAUDE.md requires before README and doc prose, and write every change in that voice. In README.md, take out the whole "Current state" section, its heading and the #5 line under it, because nothing else is under it and an empty heading reads as unfinished (John's call at planning), and change nothing else (D-14). In docs/design/0006-recall-through-mcp.md, the `egress.rs` row of the file table and "The egress filter" Outputs bullet say nine rules, list the bare `sk-` rule ninth, and describe the keyed-value step: what it walks, that it runs on the parsed record before the projection, and only with the key on (John's call at planning). A design doc describes the current state, so write it as how the filter works, not as a change. In THREAT-MODEL.md, rewrite "What it matches" to add the bare `sk-` rule (a run of ASCII letters, digits, `-` and `_` that opens with `sk-` and has 20 or more characters after it) and to say that a string under a secret-named key in a tool's input, a tool's result or an attachment is replaced in every excerpt, context window, brief quotation and per-prompt injection, as `recall_get` already replaced it. Rewrite "What it misses" to drop the `sk-` item and say what is still missed, from the code as PLAN-1 and PLAN-2 leave it and from CONTEXT: an `sk-` run with fewer than 20 characters after the prefix and Stripe's `sk_live_` keys (D-07), and a value under a secret-named key that is not a string, such as a list of strings (D-02), alongside the misses already listed (AWS `AKIA` keys, Slack `xox` tokens, vendor keys with no name or known prefix, no entropy check, and the brief and MCP fields it never touches). In docs/configuration.md, the `redact_recall` bullet's list of what the filter replaces names bare `sk-` keys and says that values named like secrets inside a tool's input, a tool's result and an attachment are replaced too. In CHANGELOG.md, add entries under `[Unreleased]`'s `### Fixed`, one each for #62, #5 and #61, each saying what a user with the knob on saw before and what happens now, in the form the existing entries take. #4 changes only a test and gets no entry, because the changelog records what a user of Terminus sees (planner's choice). Leave SECURITY.md as it is (D-14). Check every claim against the code as it now stands.
- **Verify:** `grep -n 'issues/5)' README.md` and `grep -n '## Current state' README.md` print nothing, and README.md's diff removes only that section. `grep -n 'eight rules' docs/design/0006-recall-through-mcp.md` prints nothing, and its Outputs bullet names the `sk-` rule and the keyed-value step. `grep -n 'issue #5' THREAT-MODEL.md` prints nothing, and its "What it matches" list names the `sk-` rule and the keyed values. `grep -n 'sk-' docs/configuration.md` finds the new entry in the `redact_recall` bullet. The `[Unreleased]` section of CHANGELOG.md names #62, #5 and #61. `git diff --stat` lists only the five files, and `git diff --quiet HEAD -- SECURITY.md` exits 0.

## Notes

- Sequential, last. These tests need PLAN-1's fix to prove against and PLAN-2's fixture to run on, and the docs describe what both plans built.
- John reviews docs himself: once Task 4 is written, open README.md, THREAT-MODEL.md, docs/configuration.md and CHANGELOG.md in RustRover.
- `docs/design/0006-recall-through-mcp.md` and the removal of the README's "Current state" heading were added to Task 4 by John at planning. Neither is in D-14's list.
- AC7 (human-verify, needs GitHub, after landing): the pull request that lands this phase carries `Fixes #62`, `Fixes #5`, `Fixes #61` and `Fixes #4`, and all four issues close when it merges.
