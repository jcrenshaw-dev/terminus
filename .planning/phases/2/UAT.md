---
status: testing
phase: 2
fields_version: 1
started: 2026-10-18
updated: 2026-10-18
---

## Items

### 1. Keyed secrets are replaced on every excerpt surface
expected: With redact_recall on, a secret string under a secret-named key in a tool_use input, a toolUseResult and an attachment is absent from that turn's recall_search excerpt and recall_context text, and recall_get shows REDACTED in its place. The tool_use secret is also absent from prompt-hook output. A turn with no secret-keyed string value reads byte-identically with the knob on and off.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: apply_record (observe/egress.rs:307) wired at recall/excerpt.rs:227, shared by search, context, prompt hook and brief; passing: redacted_recall the_knob_takes_a_keyed_secret_out_of_its_turns_search_excerpt, the_knob_takes_a_keyed_secret_out_of_the_per_prompt_injection, a_secret_named_key_holding_no_string_projects_exactly_as_it_was, a_nested_secret_named_string_is_taken_at_any_depth; egress keyed rows in window and recall_get; recall the_knob_leaves_an_ordinary_context_window_exactly_as_it_was. index/ and recall/get.rs untouched.

### 2. A bare sk- key in prose gets its own marker
expected: With redact_recall on, an sk- run of 20 or more key characters in prose is replaced by [redacted:an-sk-key], and sk-notification and task-notification come back unchanged.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: redact_sk_keys (observe/egress.rs:917) called last in redact() (:375), floor 20 (:209), marker [redacted:an-sk-key] (:167); passing: a_bare_sk_key_in_prose_goes_under_its_own_marker, an_sk_run_is_a_key_from_twenty_characters_after_the_prefix, sk_placeholders_and_a_hyphenated_word_come_back_unchanged, a_named_sk_value_keeps_the_header_and_assignment_marker, and the egress 'a bare sk- key in prose' row in both surfaces.

### 3. Multi-byte NAME= turns pass every process boundary
expected: With redact_recall on, turns holding café=1, Größe=10 and “token=abc” pass through the three MCP tools, the prompt hook, the SessionStart hook, terminus search and terminus show. Each returns normal output with no panic on stderr, both hooks emit their injection, and café=1 and Größe=10 come back unchanged.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: ASCII byte name walk at observe/egress.rs:735-738, no catch_unwind in cmd/mcp; passing by name: mcp the_three_tools_answer_a_multi_byte_turn_with_the_knob_on, brief the_knob_on_brief_quotes_a_multi_byte_turn, prompt the_knob_on_prompt_hook_injects_a_multi_byte_turn, recall_cli the_knob_on_search_and_show_print_a_multi_byte_turn, plus the five rule-5 unit tests including the fuzz crash input.

### 4. The egress test checks each shape's marker in its own turn
expected: For each planted shape, the egress test checks the secret is gone and its own rule's marker is present in the same turn, in both the recall_context window and the recall_get body. Turning off any one rule makes the test fail on that shape's turn.
criterion: AC4
status: pass
first_pass: pass
source: model
evidence: Ran in a throwaway worktree at 4e00b0f, patched with a VB_MUTANT env switch that skips one step of redact() or the apply_record call, and with assert_every_planted_shape_caught collecting every failing row instead of stopping at the first. `VB_MUTANT=<step> cargo test -p terminus-core --all-features --test egress`. none: 10 passed. Each switched-off step failed in both the window and recall_get on its own rows only: pem -> PEM row; userinfo -> connection URL row; header -> Authorization, Cookie and both curl rows; json -> the JSON password row in both surfaces plus the three keyed rows in recall_get; assign -> the curly NAME= row; flag -> --token row; jwt -> JWT row; github -> GitHub row; sk -> bare sk- row; keyed (apply_record skipped) -> the three keyed rows in the window only. This settles SUMMARY's open item: with rule 4 off the keyed rows fail on their own in recall_get.

### 5. Each fix's regression test fails before the fix
expected: Each fix's regression test fails on the commit before the fix and passes after it.
criterion: AC5
status: pass
first_pass: pass
source: model
evidence: Same worktree and switch, each fix turned off at HEAD (at the literal parent commits the sk and keyed tests do not compile, since REDACTED_SK_KEY and apply_record do not exist yet). #61: VB_MUTANT=rfind puts back the old rfind name slice. `cargo test -p terminus-core --all-features --lib observe::egress`: 5 failed (the five rule-5 tests, 'start byte index N is not a char boundary'); none: 24 passed. `cargo test -p terminus --all-features --test <t> multi_byte` for mcp, brief, prompt, recall_cli: each 1 failed with 'byte index 25 is not a char boundary; it is inside é', the hooks reporting 'the hook panicked'; none: all pass. #5: VB_MUTANT=sk: a_bare_sk_key_in_prose_goes_under_its_own_marker and an_sk_run_is_a_key_from_twenty_characters_after_the_prefix fail; none: pass. #62: VB_MUTANT=keyed, `--test redacted_recall`: the keyed search excerpt, per-prompt injection and nested tests fail (3 failed); none: 10 passed. #4: the egress test is the fix; item 4 shows it fails per rule.

### 6. Docs no longer say a bare sk- token is missed
expected: README, THREAT-MODEL.md, docs/configuration.md, the egress.rs module doc and the CHANGELOG no longer say a bare sk- token is missed, and they describe keyed values being replaced.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: No missed-sk wording left in README.md, THREAT-MODEL.md, docs/configuration.md, docs/design/0006-recall-through-mcp.md or egress.rs; THREAT-MODEL.md:320-325, docs/configuration.md:120, egress.rs:19-27 and :61-71, CHANGELOG.md [Unreleased] Fixed entries for #62, #5, #61 describe the sk- rule and keyed values; SECURITY.md unchanged.

### 7. The landing PR closes #62, #5, #61 and #4
expected: The pull request that lands this phase carries Fixes #62, Fixes #5, Fixes #61 and Fixes #4, and all four close once it merges. (human-verify: needs GitHub, after landing)
criterion: AC7
status: skipped
reported: Skip with readon
reason: Checked at landing; the PR must carry Fixes #62, Fixes #5, Fixes #61 and Fixes #4.

## Summary

total: 7
passed: 6
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 0
