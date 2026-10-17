---
status: testing
phase: 13
fields_version: 1
started: 2026-10-15
updated: 2026-10-17
---

## Items

### 1. Fresh store from scratch
expected: Stop everything, clear ephemeral state, start from scratch: an ingest into an empty data directory completes, the store is created at DERIVED_SCHEMA 5 with the compactions table and the two new decisions columns, and one recall search returns real turns.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: Scratch-dir ingest exited 0: derived_schema 5, compactions present and compaction_boundaries absent, decisions ends `compacted INTEGER, dropped TEXT`, 12 turns, a boundary row at offset 1281. `terminus search --project '*' record type archive` returned turn 1 with its real excerpt.

### 2. Test workflow runs on all four runners
expected: On a push and on a pull request, .github/workflows/test.yml starts four jobs (ubuntu-24.04, ubuntu-24.04-arm, macos-15-intel, macos-15), each runs the tool check before cargo test --workspace --all-features --locked, a missing tool fails the job at that step, and a failing test fails the job. (human-verify: needs GitHub Actions)
criterion: AC1
status: pass
first_pass: fail
source: model
evidence: Observed on GitHub for 1f64632: the push run (39055099956) and the pull request run (39055104913) each started eight jobs, the four runners ubuntu-24.04, ubuntu-24.04-arm, macos-15-intel and macos-15 on the debug and the release build (the item says four; 408ba74 added the release build), and all sixteen passed. In every job the steps run in the order Check the shell-test tools, Build the tests, Test, read off the jobs' step lists. A failing test fails the job: the push run on 9f366a3 (39041510572) failed its macos-15-intel job on backfill.rs:294, and the pull request run on the same commit (39041514023) failed the same job on hook.rs:455 and hook.rs:679. Read, not observed: no run in the workflow's history failed at the tool-check step, so 'a missing tool fails the job at that step' rests on the step's own text in test.yml, which exits 1 naming the missing tools before cargo test starts.
reported: it failed and we have a new failure please be more carefull
severity: major
cause: No product fault found. Both are wall-clock failures on the hosted macos-15-intel runner. (1) backfill.rs:294: the contended ingest took 82.65 ms against the 50 ms budget. The same assertion failed there at 5b6d259 (62.24 ms), and lock_race.rs:79, on the same 50 ms budget, failed there at f66b608 (82.06 ms). (2) hook.rs:455 and hook.rs:679: every test in the hook binary, including two that time nothing, finished about 10 s later than in the three macos-15-intel runs that passed, so the runner stalled for about 10 s at the start of that binary. The hook's own waits are 500 ms and 50 ms. What stalled the runner is not known. The merge c716597 changed only URL strings in install_script.rs and sign_release.rs, and both passed. Six assertions in the suite decide on a single wall-clock sample: backfill.rs:238 and :295, hook.rs:455 and :680, inject.rs:323, lock_race.rs:80.
fix: be6e468, 408ba74, retest
reason: CI for 9f366a3: push run 39041510572 failed on macos-15-intel at backfill.rs:294, and pull_request run 39041514023 failed on macos-15-intel at hook.rs:455 and hook.rs:679. The other three runners passed on both events.

### 3. Named testkit test refused without the feature
expected: cargo test -p terminus --test mcp, and each of the other 26 whole-file testkit tests named without --features testkit, exits non-zero with a message naming testkit; cargo test --workspace --all-features still builds and runs every test file.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: All 27 named runs (24 terminus-core, plus datamove, mcp and recall_cli) exited 101 with 'requires the features: `testkit`'. The declared set equals the set of whole-file gated files. The full --all-features --locked run passed with 63 test binaries for 63 files: 896 passed, 0 failed.

### 4. Diverged session opens its transcript once
expected: A diverged session's ingest pass records exactly one open of its transcript path in the testkit open log, and the regression test for it passes.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: ingest/mod.rs:1002 read_flagged does the length check, prefix and tail on one handle. a_flagged_session_opens_its_transcript_once_per_pass and a_flagged_session_rewritten_under_its_watermark_opens_it_once passed.

### 5. Old store gains the decision columns and keeps compaction facts
expected: Opening a store made before this change writably adds compacted and dropped to decisions; a decision drained after a compaction is stored with compacted = 1 and its dropped turn ids, and one drained before the columns existed keeps compacted as null.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: a_drained_decision_keeps_its_compaction gave (1, [41,42]) and (0, []). a_store_without_the_compaction_columns_gains_them_and_its_rows_read_unknown passed. Spot-check: after both columns were dropped, an ingest re-added them in a fresh store's order and the old rows read NULL.

### 6. stats counts compacted decisions apart
expected: terminus stats --json carries the two new keys after decisions, counting compacted and ordinary decisions, and the human output shows indented compacted and ordinary lines under decisions; docs/json-shapes.md and docs/commands.md describe them.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: Spot-check over compacted values 1, 0, NULL and 1: --json gave compacted_decisions 2 and ordinary_decisions 1, and the human output printed the indented '  compacted 2' and '  ordinary 1' lines under decisions. json-shapes.md, commands.md and DATA_COMMANDS updated, and the cli test passed.

### 7. replay honours --compacted-ranked for a compacted decision
expected: For a decision stored with compacted = 1, changing --compacted-ranked changes terminus replay's result; null and 0 decisions replay through the ordinary window as before.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: a_compacted_decision_replays_through_the_compacted_pool: compacted_ranked 0 changes only the compacted decision, and the 0 and null decisions do not move. compacted_ranked_moves_a_compacted_decision passed.

### 8. stats and replay answer on a store without the new columns
expected: Against a store whose decisions table lacks compacted and dropped, terminus stats and terminus replay both answer, treat the values as unknown, and do not alter the store.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: Spot-check with both columns dropped: stats --json and replay --json exited 0 with ok true and both split counts 0. terminus.db sha256 was the same before and after, and the columns were still absent. Both D-15 tests passed.

### 9. Drain sweeps old .tmp files and leaves young ones
expected: A drain deletes decisions/.decision-*.tmp and injection/.inject-*.tmp files older than the ten-minute age, leaves younger ones and anything that is not a regular file, and writes nothing to runs.error for the sweep.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: Spot-check via ingest: the 11-minute-old writer-named .tmp files in decisions/ and injection/ were deleted. The young ones, writer-named symlinks and directories, other names and the symlink targets all survived. a_pass_sweeps_the_writers_old_temporaries_and_nothing_else passed, with runs.error null.

### 10. Young empty reservation survives the drain
expected: A zero-length decision .json younger than the age limit is neither inserted nor deleted by a drain, and the record later renamed over it is inserted on the next pass; an older empty file and a non-empty unparseable file are still deleted.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: Spot-check: the young empty .json survived, and the old empty one and a non-empty unparseable one were deleted and named. a_young_empty_reservation_is_left_for_the_next_pass passed, so the record renamed over the reservation is inserted on the next pass.

### 11. Hook and state comments corrected
expected: crates/terminus/src/cmd/hook.rs no longer says the abandoned thread writes nothing, crates/terminus-core/src/inject/state.rs no longer says state files hold prompt text, and decision.rs no longer says the drain skips .tmp files.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: The greps for 'nothing on that thread writes' (hook.rs), 'quoted prompt text' (state.rs) and 'two different answers' (decision.rs) print nothing, and the read_all comment now names the drain's sweep.

### 12. Field-less compact boundary gets a row at its offset
expected: A system/compact_boundary record with no uuid or no timestamp gets a compactions row at its byte offset, the turns after it keep their ids, and the next prompt treats the turns before it as dropped.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: Spot-check: a boundary with no uuid at offset 5656 got a compactions row at 5656, no turn was added, and the row survived reindex. The no-uuid and no-timestamp offset tests passed, and so did the suppression test a_boundary_with_no_uuid_still_offers_back_what_fell_out.

### 13. DERIVED_SCHEMA 4 store rebuilt to 5
expected: A store at DERIVED_SCHEMA 4 is rebuilt to 5 on first open, including one that holds evicted sessions, and ends with the compactions table in place of compaction_boundaries.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: Spot-check: v4-shaped copies, plain and with the boundary's session evicted, both came up at derived_schema 5 after an ingest. compaction_boundaries was gone and the compactions rows were byte-identical. Both version-4 reindex tests passed.

### 14. README and SECURITY no longer say no workflow runs the tests
expected: README.md and SECURITY.md say the test workflow runs the suite on every push and pull request, and no longer say no workflow runs the tests.
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: The grep for the old wording prints nothing. README names test.yml running on every push and pull request and keeps the local command. SECURITY.md has the new 'Every push and pull request' bullet.

### 15. PR closes the six issues and passes on four runners
expected: The pull request for this work closes #47, #44, #15, #40, #38 and #17, and its test run passes on all four runners. (human-verify: needs GitHub Actions)
criterion: AC7
status: pass
first_pass: pass
source: model
evidence: `gh pr view 55 --json closingIssuesReferences` lists #15 #17 #38 #40 #44 #47, and the body carries `Fixes #47, Fixes #44, Fixes #15, Fixes #40, Fixes #38, Fixes #17`. The pull request test run on 1f64632 (run 39055104913) passed all eight jobs: ubuntu-24.04, ubuntu-24.04-arm, macos-15-intel and macos-15, each on the debug and the release build. The push run on the same commit (39055099956) passed all eight as well, and sentinel-fast (39055104912) passed its mechanical and security-review jobs.

### 16. CHANGELOG has Unreleased entries for each fix
expected: CHANGELOG.md's [Unreleased] section has an entry for the test workflow, the testkit refusal, the stats keys, compacted replay, the drain sweep and race fix, the one-handle diverged read and the field-less boundary rebuild.
status: pass
first_pass: fail
source: model
evidence: After 5c6f808, `sed -n '/## \[Unreleased\]/,/## \[0\.1\.1\]/p' CHANGELOG.md | grep -n '^- '` lists entries for the stats keys (line 5), the test workflow (#47, line 12), compacted replay (55), the .tmp sweep (#38, 62), the drain race (66), the one-handle read (#15, 71), the field-less boundary rebuild (#17, 77) and the testkit refusal (#44, 83).
reported: missing - [Unreleased] has no entry for the test workflow (#47) or for cargo refusing a named testkit test run without the feature (#44)
severity: minor
cause: PLAN-1.md's file list never named CHANGELOG.md, so the plan that added test.yml (#47) and the required-features declarations (#44) wrote no [Unreleased] entry for either; plans 2-4 wrote theirs.
fix: 5c6f808, retest

### 17. Comments this phase wrote, or left describing behaviour it changed, are accurate (D-18)
expected: behavior wrong - the new hook.rs comment understates what a cut-off decision write leaves behind, and two comments made stale by this phase's changes are still in place
origin: verifier
status: pass
first_pass: fail
source: model
evidence: After bfbbf5e, `grep -rn 'leaves at most\|quoted prompt text\|has already made it a turn' crates` prints nothing (exit 1). hook.rs:238-241 now says a decision write cut off after it reserves its .json also leaves that empty .json and the drain deletes both once old; record.rs:106-110 says a boundary missing uuid or timestamp is not a turn and still gets a boundary row; inject_state.rs:267-270 says the scratch holds turn ids, suppressions and the compaction flag. `cargo build -p terminus-core -p terminus` is clean. `RUSTDOCFLAGS=-D warnings cargo doc --no-deps -p terminus-core` fails on 21 private-intra-doc-link errors, none of them in a file this commit changed.
reported: behavior wrong - the new hook.rs comment understates what a cut-off decision write leaves behind, and two comments made stale by this phase's changes are still in place
severity: minor
cause: Plan 3 rewrote the hook.rs comment against the state write's shape only; decision::write_file reserves the target .json with create_new before the rename (decision.rs:323-351), so a kill between the two leaves an empty .json as well as the .tmp. parse/record.rs:103-110 and tests/inject_state.rs:267 were outside plans 3 and 4's leases and kept wording this phase made false.
fix: bfbbf5e, retest

## Summary

total: 17
passed: 17
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 3
