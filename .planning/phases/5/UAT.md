---
status: testing
phase: 5
fields_version: 1
started: 2026-10-20
updated: 2026-10-20
---

## Items

### 1. Every budget gives no brief, the uncut brief, or a marked brief
expected: For a fixture whose quoted turns are each under 240 characters, so the uncut brief holds no `...`, every budget from 0 to the uncut length yields a brief that is empty, byte-identical to the uncut brief, or contains `...`.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: inject_brief.rs:936 sweeps every budget 0..=uncut length with the no-`...` premise asserted at :908. `a_budget_that_leaves_a_quoted_turn_no_room_marks_it_and_does_not_record_it` passes at the default TMPDIR and at a 130-char TMPDIR. brief.rs:220-224 plus clip's marked branch mean no rendered cut can lack `...`.

### 2. A turn with no room shows its label and the mark and is not briefed
expected: At a budget that leaves a quoted turn no room, that turn's line reads `<label>: ...`, and the turn is not in the session state's briefed turns.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: brief.rs:405-412 renders the label and ELISION alone with `shown: false`; brief.rs:286 filters those out of `Brief::turns`. The test asserts both lines exactly (inject_brief.rs:921 and the answered line) and an empty `State::load(...).brief` (:931), and passes at both TMPDIRs.

### 3. The long-path regression test fails before the fix and passes after
expected: A new test with a long project path fails at `de82912` and passes after the change.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: Run against de82912's brief.rs in a scratch copy of HEAD: FAILED at inject_brief.rs:1035 "the brief quotes no prompt between the tags", with both turns dropped and no mark. Passes at HEAD at both TMPDIRs. Project name is 155 chars.

### 4. The workspace suite passes at the default and a 130-character TMPDIR
expected: `cargo test --workspace --all-features` passes with the default `TMPDIR` and with `TMPDIR` set to an existing 130-character directory.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: 130-char TMPDIR full workspace run: 68 binaries, 1015 passed, 0 failed. Default TMPDIR: every binary that reads the edited fixture or sets a brief budget (16 binaries) passes by name. The executor's default full run was 1015/0.

### 5. No assertion or golden changed
expected: `git diff de82912 -- 'crates/*/tests'` removes or changes no line inside an `assert!`, `assert_eq!` or `assert_ne!` call, and changes no file under `tests/goldens`.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: With explicit paths, the only removed lines under crates/*/tests since de82912 are 3 comments and the `let budget` setup line in inject_brief.rs. Goldens show no diff and no status, and recall_golden passes. The AC's literal `'crates/*/tests'` pathspec matches nothing in git, so the check needs explicit directories.

### 6. The docs no longer say a quotation with no share is dropped, and the changelog has #46
expected: Neither `docs/design/0007-injection.md` nor the `brief.rs` doc comments still say a quotation with no share is dropped, and `CHANGELOG.md` has a Fixed entry for #46 under `[Unreleased]`.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: Stale-phrase greps on brief.rs and 0007 return nothing, and the prompt-arm sentence is kept. The CHANGELOG #46 entry is at :218-228 under [Unreleased] ### Fixed. The 0007 paragraph and the entry match brief.rs:220-224, :286, :405 and mod.rs:454-460.

### 7. Issue #46 is closed by the merged change
expected: Issue #46 is closed by the merged change. (human-verify: needs the merged PR on GitHub)
criterion: AC7
status: skipped
reason: checked when traders lands

### 8. John reads the changed docs and comments
expected: The budget paragraph in `docs/design/0007-injection.md`, the #46 entry in `CHANGELOG.md` and the `brief.rs` doc comments read correctly to John.
status: pass
first_pass: pass
reported: approved

## Summary

total: 8
passed: 7
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 0
