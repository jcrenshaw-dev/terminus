---
status: testing
phase: 5
fields_version: 1
started: 2026-10-19
updated: 2026-10-19
---

## Items

### 1. ESC and BEL print escaped in search, the show window and observations
expected: With ESC and BEL archived in a tool result, human output of `terminus search`, the `terminus show --before/--after` window and `terminus observations` holds no 0x1b or 0x07 byte and shows `\u001b` and `\u0007` where they were. A control planted in a turn's time or tool name prints the same way in the show header.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: Both new tests pass on HEAD. Probe on a scratch store: search, show header+window and observations stdout held no raw control byte; show header `2026-10\u001b19T12:00:00.000\u0007Z  planted\u001bfile.jsonl  .../planted\u001bproject`, window `assistant/Ba\u001bsh` and `\u001b]52;c;aGVsbG8=\u0007 quixotrel`. Call sites search.rs:147-152, show.rs:110-112, :161-162, :174, observations.rs:266-292.

### 2. The full escape set and nothing else
expected: Every code point U+0000 to U+001F except tab and newline, plus U+007F to U+009F, prints in human output as its lowercase `\u00XX` escape. Tab, newline, U+200B and U+202E print unchanged.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: escape.rs:49-51; the escape:: unit tests pass (all of U+0000..U+10FFFF walked, 63 changed). End to end, an observations command holding all 63 controls plus tab, newline, U+200B and U+202E printed exactly as the 63 lowercase escapes with those four raw. In search and show, VT, FF, CR and NEL arrive as spaces through core's flatten (D-08).

### 3. --json output unchanged
expected: `--json` output of search, show and observations on the planted transcript is byte-identical to the output of the commit before the change.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: HEAD and pre-fix (crates/terminus/src at 23b2db8^) binaries on one store produced byte-identical --json for search (x3), show (x3, --raw included) and observations. No phase hunk in the JSON branches; terminus-core, Cargo.toml and Cargo.lock unchanged.

### 4. show body unchanged and --raw still escapes
expected: The body `terminus show <id>` prints is byte-identical to the pre-change commit. With `--raw`, the window and header are still escaped.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: show.rs:121-127 unchanged. Body line identical between HEAD and pre-fix binaries and equal to the written transcript line for two anchors, with and without --raw; the header and window held no raw control under --raw.

### 5. Regression tests fail before the fix and pass after
expected: Both new regression tests fail on a commit before the fix (helper reverted) and pass on HEAD.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: On the pre-fix tree the search/show test failed at recall_cli.rs:1572 and the observations test at observations.rs:608, each on its no-ESC-or-BEL assertion; on HEAD both pass. The full suite on HEAD exits 0: 68 binaries, 987 passed, 0 failed.

### 6. Docs describe the escape
expected: THREAT-MODEL.md no longer says nothing filters control characters from show or search output; it describes the escape, the DEL and C1 the show body still passes, and --json passing DEL and C1. docs/commands.md describes the escape for all three commands. CHANGELOG.md [Unreleased] carries #63, #65 and #66 under ### Fixed.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: The old THREAT-MODEL sentence is gone (grep exits 1). THREAT-MODEL.md:127 covers the escape, the DEL and C1 the body passes, and --json passing DEL and C1. docs/commands.md:61, :74 and :92 cover all three commands. CHANGELOG.md has #63, #65 and #66 under [Unreleased] ### Fixed (:35-:199). SECURITY.md and docs/json-shapes.md are untouched.

### 7. PR closes #63, #65 and #66
expected: The pull request that lands this phase carries `Fixes #63`, `Fixes #65` and `Fixes #66`, and all three close once it merges. (human-verify: needs GitHub, after landing)
criterion: AC7
status: pending

### 8. The observations regression test leaves the session file name clean on a false premise
expected: behavior wrong - test coverage: Task 3 required planting a control in the file name if a bare ingest accepts such a name, and it does accept one (agent-*.jsonl). The test skipped it, and its comment and the SUMMARY deviation both claim it cannot be done.
origin: verifier
status: pass
first_pass: fail
source: model
evidence: `cargo test -p terminus --all-features --locked --test observations` -> `7 passed; 0 failed`. With escape::controls removed from crates/terminus/src/cmd/observations.rs:266, archived_control_characters_print_escaped_in_the_listing -> `panicked at crates/terminus/tests/observations.rs:617:5: the listing wrote ESC or BEL to the terminal`, `1 failed`; restored with git checkout. clippy -D warnings clean, rustfmt --check clean. Fix 42fd350.
reported: behavior wrong - test coverage: Task 3 required planting a control in the file name if a bare ingest accepts such a name, and it does accept one (agent-*.jsonl). The test skipped it, and its comment and the SUMMARY deviation both claim it cannot be done.
severity: minor
cause: The comment at crates/terminus/tests/observations.rs:582-584 assumes a bare ingest never archives a file name holding a control. crates/terminus-core/src/discover.rs:264-268 admits any name that starts with `agent-` and ends with `.jsonl`, so `agent-<ESC>x<BEL>.jsonl` is archived and listed. The test therefore never plants a control in the file name, and removing escape::controls at crates/terminus/src/cmd/observations.rs:266 fails no test. The code is right; the test coverage and SUMMARY.md:33 are wrong.
fix: 42fd350, retest

### 9. After the phase 5 PR merges, check that its description carries `Fixes #63`, `Fixes #65` and `Fixes #66`, and that all three issues on crenshawdev/terminus show as closed by it
expected: The PR body has all three Fixes lines, and #63, #65 and #66 are closed and linked to the merged PR
origin: verifier
why_human: Out-of-reach resource: the PR does not exist until landing, and checking it needs GitHub, which this verifier may not reach (no network)
status: pending

## Summary

total: 9
passed: 7
failed: 0
pending: 2
skipped: 0
blocked: 0
reworked: 1
