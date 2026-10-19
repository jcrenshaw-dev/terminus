---
status: testing
phase: 1
fields_version: 1
started: 2026-10-18
updated: 2026-10-18
---

## Items

### 1. Planted Read result never reaches the prompt hook
expected: A test archives a Read of file F whose result holds planted instructions P, then submits a prompt naming F. The hook injects the Read tool-call turn and P appears nowhere in its output. The test fails with the exclusion bypassed and passes with it.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: prompt.rs:441 quotable (prompt.rs:578-596) sits between eligible and the cap on the hook's path (hook.rs:254 -> user_prompt_submit -> selected). a_read_result_is_never_injected_but_the_read_still_fires passes, and its premise shows the result turn was eligible. Binary probe: terminus search shows the planted text in the result's excerpt, and the hook output omits it and still names the Read tool-call turn.

### 2. terminus replay drops tool-result turns
expected: For a decision whose ranked hits include a tool-result turn, terminus replay does not report that turn as injected, matching what the live hook selects.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: replay.rs:302-313 runs prompt::quotable in the live arm's order. replay_never_credits_a_tool_result_the_hook_refuses passes: total 1, equal to prompt::select, and 2 on a store without is_typed.

### 3. Both hooks frame every quoted string and neutralize planted closers
expected: Prompt-hook output and the SessionStart brief both sit between one fixed <terminus-recall> opening tag and one closing tag under a quoted-history header. An excerpt, day, project label or branch holding the closing tag in another case, with whitespace inside, or in fullwidth lookalikes still yields output where the closing tag appears exactly once, at the end.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: mod.rs:221-249, :281-454, :482-492; prompt.rs:288, :322, :479-485; brief.rs:233, :253, :424-430, :575, :631. a_closing_tag_planted_in_a_firing_turn_is_neutralized, a_closing_tag_planted_in_any_archived_string_is_neutralized and 31 lib inject:: tests pass, and recall_golden passes. Binary probe: a U+200B-split closer was neutralized in additionalContext.

### 4. A budget too small for the frame writes nothing
expected: With a budget smaller than the header and both tags, the prompt hook emits nothing and exits 0. At every budget where it emits, additionalContext ends with the closing tag.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: mod.rs:470-488, prompt.rs:458, brief.rs:218-222. the_injection_is_cut_to_the_configured_budget and every_brief_budget_gives_nothing_or_a_framed_brief_that_fits sweep every budget, and the binary test a_budget_too_small_for_the_frame_writes_nothing (prompt_chars 200, exit 0, no stdout) passes.

### 5. Renders are byte-identical for unchanged state
expected: Two SessionStart renders against an unchanged store are byte-identical, and two prompt-hook renders of the same prompt are byte-identical.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: two_filtered_briefs_against_an_unchanged_store_are_byte_identical, two_session_starts_against_an_unchanged_store_are_byte_identical and the_injected_text_names_the_turn_and_the_day_and_repeats_itself pass, with fixed tags and no nonce.

### 6. Design doc and threat model describe the frame and the exclusion
expected: docs/design/0007-injection.md and THREAT-MODEL.md no longer say injected text has no delimiter, and both describe the frame and the exclusion of tool results.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: No 'no delimiter' or 'any record type qualifies' in either file. Both spell both tags. 0007-injection.md:104 is the is_typed exclusion step, and THREAT-MODEL.md:149 and :202 describe the frame and the exclusion. The bundle strings quoted at THREAT-MODEL.md:116 are present in 2.1.302.

### 7. The landing PR closes #60
expected: The pull request that lands this phase carries Fixes #60, and #60 is closed once it merges. (human-verify: needs GitHub, after landing)
criterion: AC7
status: skipped
reported: skip them, checked after landing
reason: checked after landing

### 8. CHANGELOG records the frame and the exclusion
expected: CHANGELOG.md has an entry saying the prompt hook no longer injects tool results and both hooks quote recall inside the <terminus-recall> frame.
status: pass
first_pass: pass
source: verifier
evidence: CHANGELOG.md:123-131 under [Unreleased] / ### Fixed names the exclusion, the <terminus-recall> frame, the neutralization and #60.

### 9. A closing tag split by an invisible ASCII control character passes the neutralizer
expected: behavior wrong - the phase treats invisible chars inside a closer as still closing it (U+200B is neutralized), but invisible ASCII controls are not ignorable, so the same attack with U+001F, U+001B, U+0000 or U+007F reaches the model intact. ROADMAP criterion 2 asks for a delimiter no excerpt text can close.
origin: verifier
status: pass
first_pass: fail
source: model
evidence: Retest after e4bec6e. With is_ignorable reverted to c.is_whitespace() || !c.is_ascii(): cargo test -p terminus-core --lib ascii_controls_anywhere -> 1 failed; cargo test -p terminus-core --features testkit --test inject_prompt a_closing_tag_planted_in_a_firing_turn -> panicked at inject_prompt.rs:910, 1 failed. With the fix: cargo test --workspace --all-features --locked --no-fail-fast -> 933 passed, 0 failed.
reported: yes, record it and go to failure 2
severity: major
cause: is_ignorable (inject/mod.rs:452) admits whitespace and non-ASCII only. ASCII control characters U+0000-0008, U+000E-001F and U+007F are ASCII and not whitespace, so one inside a closer breaks the match, and flatten keeps them, so the closer reaches the model.
fix: e4bec6e, retest

### 10. The design doc's neutralizer description and Figure 2 do not match what shipped
expected: behavior wrong (doc) - the design record describes the pre-9f3ac99 ignorable set and a prompt flow without the exclusion or the frame
origin: verifier
status: pass
first_pass: fail
source: model
evidence: Retest after 0f08cfc: grep -c "Default_Ignorable\|zero-width, default-ignorable" docs/design/0007-injection.md THREAT-MODEL.md -> 0 and 0; the replay section reads `quotable`, `within`; Figure 2 has the exclusion node at :184 and the frame-budget decision at :192. Mermaid render not checked.
reported: yes, record it and go to failure 3
severity: minor
cause: 0007-injection.md was written against the plan-2 neutralizer and not revised for 9f3ac99 (any non-ASCII char) or e4bec6e (ASCII controls): :90, :118 and THREAT-MODEL.md:149 still list whitespace, default-ignorables and combining marks. Figure 2 predates the exclusion and the frame, and the replay section names the arm functions without quotable.
fix: 0f08cfc, retest

### 11. terminus replay credits turns the live hook no longer injects when prompt_chars is below the frame's cost
expected: behavior wrong - D-06 made the live arm silent under a 239-char budget, and replay does not model the budget, so it diverges from the hook there
origin: verifier
status: pass
first_pass: fail
source: model
evidence: Retest after 653dff6. With replay.rs stashed: cargo test -p terminus-core --all-features --test feedback replay_credits_nothing_when_the_budget -> panicked at feedback.rs:1523, left 1 right 0 (false positive 1, wasted budget 1). With the fix: cargo test --workspace --all-features --locked --no-fail-fast -> 934 passed, 0 failed.
reported: Yes
severity: minor
cause: The live arm puts the budget to super::frame_fits(DESCRIPTION, budget(config)) at prompt.rs:458 and fires nothing when the frame does not fit. replay::would_inject (replay.rs:302-317) stops at prompt::capped and never asks that question, so under a prompt_chars below the frame cost it credits turns the hook never hands over. The check lives only in the live path, the shape D-04 ruled out for the exclusion.
fix: 653dff6, retest

### 12. After landing, confirm the pull request that merges hardin carries 'Fixes #60' and that issue #60 is closed once it merges
expected: The PR body contains 'Fixes #60' and crenshawdev/terminus issue #60 shows closed by that PR
origin: verifier
why_human: Out-of-reach resource and a future event: the PR does not exist until the phase lands, and checking it needs GitHub, which this verification may not reach (no network). It is not a truth the model cannot exercise. It just cannot be exercised before the landing.
status: skipped
reported: skip them, checked after landing
reason: checked after landing

## Summary

total: 12
passed: 10
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 3
