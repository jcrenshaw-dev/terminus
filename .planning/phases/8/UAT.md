---
status: testing
phase: 8
fields_version: 1
started: 2026-10-27
updated: 2026-10-27
---

## Items

### 1. Human output escapes archived controls; documents stay raw
expected: With ESC and BEL archived in a session key, project path, branch, and a run's notes and error, the human output of sessions, usage, retention, reindex, verify, status, doctor, ingest, backfill, search, show and observations holds no 0x1b or 0x07 byte on stdout or stderr and shows \u001b and \u0007 instead; the --json output of those commands, the hook document and show <id>'s record body hold the archived bytes unchanged.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: out.rs:33-60 escapes the whole formatted text, with raw writes only at json.rs:123/134, hook.rs:347 and show.rs:125. All 9 tests in tests/escaped_output.rs pass (all 12 AC1 commands, both streams, sessions --json raw 0xC2 0x9B and 0x7F, show body raw). brief.rs the_session_start_document_carries_a_c1_control_raw passes.

### 2. Stray println!/eprintln! fails clippy; no per-field escapes left
expected: A println! or eprintln! added in crates/terminus/src outside the output helpers fails cargo clippy, and escape::controls is no longer called from search, show or observations.
criterion: AC2
status: pass
first_pass: fail
source: model
evidence: After b4a39d4: with println!("x") and eprintln!("y") added to main.rs's fn main, `cargo clippy --locked -p terminus --bins` fails with 'error: use of `println!`' (print_stdout) and 'error: use of `eprintln!`' (print_stderr), both pointing at main.rs:4's deny; with them removed it finishes clean, and `cargo clippy --locked -p terminus --all-targets --all-features` finishes clean. `grep -rn 'escape::' crates/terminus/src` matches only out.rs, so search, show and observations no longer call escape::controls.
reported: unwired - the print lints are denied only in the lib crate root, so main.rs, the bin's own crate root inside crates/terminus/src, is not covered
severity: minor
cause: The print lints are denied only at lib.rs:13, the lib crate root. main.rs is the bin target's own crate root, carries no deny, and neither Cargo.toml has [lints.clippy], so a println! or eprintln! in main.rs passes clippy.
fix: b4a39d4, retest

### 3. Every --json operational failure prints its own empty document
expected: Each --json command, made to fail operationally (e.g. an unreadable data directory), prints exactly one JSON document with ok: false, the reason, and the empty-document fields docs/json-shapes.md lists for it, and nothing on stderr; misuse still exits 2 with an empty stdout; read::unusable no longer exists.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: lib.rs:90-111 writes the held empty document with try_emit and no stderr, and grep 'unusable' is empty. cli.rs a_json_command_that_fails_operationally_answers_with_a_document (14 DATA_COMMANDS) and misuse_exits_two_with_an_empty_stdout pass, as do the replay damaged-log tests. On the binary with an unreadable data directory, 14 commands give one ok:false document with empty stderr; compact is the exception, filed as its own gap.

### 4. One byte formatter: 1,400,000,000 bytes is 1.3 GiB
expected: The install estimate and status format sizes through one formatter, so 1,400,000,000 bytes prints as 1.3 GiB in both, and no command prints GB or MB.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: cmd/mod.rs:191 is the only fn human. backfill.rs print_estimate (also called by install/mod.rs:359) and status.rs:174 use it. cmd::tests::a_byte_count_reads_in_powers_of_two and backfill_prints_an_estimate_and_finishes_behind_the_shell pass. GB, MB and KB appear only in comments.

### 5. Purge size counts every file under the data directory
expected: On a data directory holding snapshots/ and decisions/, the size uninstall --purge shows before it asks equals the sum of every file under the directory, and its deleted N byte(s) line prints the same N.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: uninstall.rs:453 measures once with directory_bytes (cmd/mod.rs:172, recursive, shared with data.rs:126) and prints it before the question and in the deleted line. purge_with_yes_prints_the_size_and_the_path_then_deletes (plants snapshots/ and decisions/) and the_size_is_shown_before_the_question pass.

### 6. Prompt-line cut to a share of 0-3 shows as ... and is not recorded
expected: A prompt-line hit given a share of 0 to 3 characters, whose text is longer than its share, shows as ...; that turn is not recorded as injected in the session state or in decision.injected, so a later prompt can quote it; the brief's output is unchanged.
criterion: AC6
status: pass
first_pass: fail
source: model
evidence: After 11a3d05, render (prompt.rs) leaves out trailing turns until every line is whole. `cargo test --locked -p terminus-core --all-features --lib inject::prompt` -> 9 passed: a lone turn at share 3 shows '...' unrecorded, at spare 0-2 render returns None with nothing recorded, [abcdefgh, ""] at spare 4 shows '...' beside the empty turn with only turn 2 recorded, and at every budget every line is '- turn N (day): ' plus its whole text or a text ending in '...'. `--test inject_prompt` -> 13 passed, including the every-budget sweep on the ingested fixture, which now fails on a cut turn line (confirmed: with the fit check disabled it fails at budget 250 with 'turn 1's line was cut'), and a_turn_shown_as_the_mark_alone_is_not_remembered_and_can_be_quoted_later. Workspace suite: 1077 passed, 0 failed. Brief unchanged: `git diff 758ebb5^..HEAD -- inject/brief.rs tests/inject_brief.rs inject/mod.rs` is empty; inject_brief 19 passed. AC6 and ROADMAP criterion 5 were reworded with John's approval to add the left-out case, since a lone turn with under 3 characters to spare cannot show the mark within the budget.
reported: behavior wrong - at a lone turn's share of 0 to 2 (a budget that cannot hold the ids, dates and marks), frame's backstop cuts the turn's line instead of the turn showing '...'. This is plan 5's recorded deviation, which follows the revised D-17, but AC6 and PLAN-5's must-have were not revised with it. The recording half is right.
severity: minor
cause: render (prompt.rs:295-327) counts each mark into the fixed cost, but when the ids, dates and marks alone are over what the frame leaves, it hands the overspend to frame's backstop (prompt.rs:329), which clips the end of the last line: the ': ' separator, and at share 0 the date's closing parenthesis. So at a lone turn's share of 0 to 2 the line ends in a backstop cut, not ': ...'. The tests check a lone turn only at spare 3, and the inject_prompt sweep skips backstop-cut lines. Reachable only with prompt_chars set within 2 characters of the frame plus ids plus dates; recording is correct.
fix: 11a3d05, retest

### 7. Docs match the code
expected: THREAT-MODEL.md no longer names #72 as open, docs/json-shapes.md holds no list of operational failures that write no document, and docs/commands.md and docs/install.md say the purge size counts everything --purge deletes.
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: grep '#72' THREAT-MODEL.md is empty, and THREAT-MODEL.md:128 covers every command. json-shapes.md:52-65 has no failure list, and misuse is the one run with no document. commands.md:219 and install.md:91 say the size counts everything --purge deletes. 0007-injection.md:122 matches render, with no #94.

### 8. compact --json reads an unreadable data directory as no store and answers ok: true
expected: behavior wrong (pre-existing, 970f2c4) - Path::exists swallows the permission error, so compact says there is no store when there is one it cannot read. That breaks AC3's own example of how to make each --json command fail, and the goal's 'say what they really do'.
origin: verifier
status: pass
first_pass: fail
source: model
evidence: After bc4f055 (cmd::has_store uses try_exists, called from compact.rs and uninstall.rs purge). On target/debug/terminus with session-recall.jsonl ingested and the data directory chmod 000: `terminus compact` exits 1 with stderr 'terminus: cannot read the terminus store at .../data/terminus.db: Permission denied (os error 13)'; `terminus compact --json` exits 1 with 0 stderr bytes and one document {"command":"compact","data":{"after":{...0},"before":{...0},"reclaimed_bytes":0},"ok":false,"reason":"cannot read the terminus store at ...: Permission denied (os error 13)"}. With the directory back at 0700, `compact --json` exits 0 with ok:true and the measured sizes. Tests: cli.rs compact_on_an_unreadable_data_directory_fails and uninstall.rs purge_on_an_unreadable_data_directory_fails_and_keeps_the_archive pass, and both fail with has_store put back to Path::exists. Workspace suite: 1079 passed, 0 failed.
reported: behavior wrong (pre-existing, 970f2c4) - Path::exists swallows the permission error, so compact says there is no store when there is one it cannot read. That breaks AC3's own example of how to make each --json command fail, and the goal's 'say what they really do'.
severity: minor
cause: compact.rs:45 tests data_dir.join(DB_FILE_NAME).exists(); Path::exists turns a permission error into false, so compact answers an unreadable store as no store, ok: true, exit 0. Pre-existing since 970f2c4, not touched by this phase.
fix: bc4f055, retest

## Summary

total: 8
passed: 8
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 3
