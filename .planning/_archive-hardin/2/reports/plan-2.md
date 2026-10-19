PLAN COMPLETE
Plan: .planning/phases/2/PLAN-2.md
Tasks: 3 of 3
| Task | Commit | Note |
|---|---|---|
| 1 - Add the rooted fixture for this phase's shapes | 312ca14 | session-redaction.jsonl under project-epsilon, appended last to TRANSCRIPT_FIXTURES and ROOTED_FIXTURES; README row, pins paragraph, rooted-cwd counts. fixtures test 18/18; workspace run green with no test edited; greps print 1/1/1/1, one sk- run, 0 VBEGRESS. Task 1's Verify names the workspace suite, so it ran here as well as at the end |
| 2 - Replace keyed values on the parsed record before the projection | bbe41a5 | Redaction::apply_record walks tool_use input, toolUseResult and attachment to index::text::MAX_DEPTH, rule 4's name test, borrowed when off; called in of_record_scrubbed before text::project. redacted_recall 10/10; with the call removed the keyed search, injection and nested tests fail showing a sentinel, restored green. recall_golden, recall, egress, inject_brief, inject_prompt, lib, mcp, recall_cli, brief, prompt all green with no test edited; index/ and recall/get.rs unchanged |
| 3 - Check each planted shape in the turn that holds it | 3484ddc | PLANTED carries its fixture (9 secrets rows unchanged + 6 redaction rows); read_both_ways/projected/got take the fixture, one store each; per-turn check with a turn-count equality first. egress 10/10. Each of rules 1-9 and the keyed step removed in turn: every run failed on the row of that rule's marker (keyed step: the tool_use input row's window turn); sources restored, diff stat tests/egress.rs only |
Full suite: cargo test --workspace --all-features --locked (workflow.test_command is null), exit 0, 947 passed across 67 test binaries, 0 failed.
Deviations: none
Open items: cargo fmt --check reports pre-existing diffs in crates/terminus/src/cmd/hook.rs, crates/terminus/tests/hook.rs and crates/terminus/tests/release_workflows.rs, files outside this plan; left alone. The rule-4 removal run fails on session-secrets.jsonl's password row first, so it never reaches the keyed rows' recall_get check; the plan's bar (fail on that rule's row) is met, but whether the keyed get rows fail on their own with rule 4 out was not observed.
