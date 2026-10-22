---
status: testing
phase: 1
fields_version: 1
started: 2026-10-22
updated: 2026-10-22
---

## Items

### 1. A fresh store starts clean
expected: With TERMINUS_DATA_DIR at an empty temp directory, this branch's build creates the store, ingests a transcript with no error line, `terminus verify` passes, and `terminus search` returns a hit from that transcript
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: Fresh temp store: ingest of session-basic.jsonl exit 0; verify `1 session(s) checked, 0 failed` exit 0; last_run.error null; search `holds` returned 2 hits from the transcript

### 2. The re-key and twin merge are gone
expected: `grep -rniE '\bre-?key|stale_keys|stale_parents|remap_ids|move_key|plant_old_key|UnmergedTwin|\btwins?\b' crates/ fuzz/` matches no line, `grep -ci merge crates/terminus-core/src/recover.rs` prints 0, and `crates/terminus-core/tests/session_keys.rs` does not exist
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: AC1 grep over crates/ fuzz/ matched nothing; `grep -ci merge crates/terminus-core/src/recover.rs` = 0; tests/session_keys.rs absent; recover() at recover.rs:112-122 is open, up_to_date, sweep_watermarks

### 3. The column bring-forward and old-layout code are gone
expected: `BRING_FORWARD_COLUMNS` is an empty slice, `grep -rnE 'drop_summary_columns|summary_columns|DROPPED_OBSERVATION_COLUMNS|DROPPED_META_KEY|retire_old_boundaries' crates/` matches no line, `DecisionLog` has no `Absent` variant, and `quotable` runs no query of its own
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: schema.rs:441 `= &[];`; AC2 grep rc=1; read.rs:122-128 DecisionLog is Present | Damaged; prompt.rs:582-586 quotable is a pure filter over Vec<Hit> with no Connection or SQL

### 4. The workspace test suite passes
expected: `cargo test --workspace --all-features` passes
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: cargo test --workspace --all-features at 4f390fe: exit 0, 1002 passed, 0 failed, 0 ignored across 67 result lines

### 5. A store missing tables fails loudly
expected: On a temp store with both `decisions` and `labels` dropped, `terminus stats --json` and `terminus replay --json` each exit 1 and print a document with `ok: false`; on a temp store with `observations` dropped, `terminus observations` exits 1
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: Both tables dropped: stats --json and replay --json each exit 1 with ok:false and a reason naming `decisions` and `labels`; human mode exit 1 without `no such table`. observations table dropped: `terminus observations` exit 1 with `no such table: observations`

### 6. --kind observation is accepted like any kind
expected: `terminus search --kind observation x` exits 0 with no hits, and MCP `recall_search` with `kind: "observation"` returns empty hits with no `isError`
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: CLI: empty stdout, exit 0, --json gives ok:true and hits []. MCP recall_search kind observation: [isError, hits] = [false, []]

### 7. The live store still ingests and verifies
expected: On a `VACUUM INTO` copy of the live store with `TERMINUS_DATA_DIR` pointed at it, this branch's build runs `terminus ingest` with no error line and `terminus verify` reports every block intact (human-verify: needs the live store on John's machine)
criterion: AC6
status: pass
first_pass: pass
source: model
evidence: Build: cargo build --release -p terminus at 4f390fe. Copy: sqlite3 ~/.local/share/terminus/terminus.db "VACUUM INTO '<copy>/terminus.db'" -> exit 0; sessions 3339 and turns 696712 match the live store. TERMINUS_DATA_DIR=<copy> ./target/release/terminus ingest -> exit 0, no output; status last_run: files_seen 3348, files_committed 2, files_failed 0, turns_added 42, error null. TERMINUS_DATA_DIR=<copy> ./target/release/terminus verify -> '3339 session(s) checked, 0 failed', exit 0. Copy removed.

### 8. The docs no longer describe the re-key or the refusal
expected: `grep -rniE '\bre-?key|\btwins?\b' docs/ THREAT-MODEL.md README.md` matches no line, and `docs/commands.md` and `docs/design/0005-search.md` no longer say `--kind observation` is refused
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: AC7 grep over docs/ THREAT-MODEL.md README.md matched nothing; commands.md and 0005-search.md carry no observation refusal. The same refusal is still described in two files AC7 does not name, reported as a separate gap

### 9. docs/design/0006-recall-through-mcp.md and THREAT-MODEL.md still describe the removed kind: observation refusal
expected: behavior wrong - two public docs describe an MCP refusal plan 3 task 5 removed, which goes against D-13 (bring every doc that describes the removed code up to date) and the rule that design docs describe only the current design
origin: verifier
status: pass
first_pass: fail
source: model
evidence: After 8903ee7: grep -rnE 'kind. of .observation|observation. is refused|"observation"` is refused' docs/ THREAT-MODEL.md README.md -> no line, rc=1. 0006-recall-through-mcp.md:127 and :144 and THREAT-MODEL.md:265 no longer name the refusal.
reported: behavior wrong - two public docs describe an MCP refusal plan 3 task 5 removed, which goes against D-13 (bring every doc that describes the removed code up to date) and the rule that design docs describe only the current design
severity: minor
cause: Plan 3 task 5 removed the refusal from the code and from docs/commands.md and 0005-search.md, but docs/design/0006-recall-through-mcp.md was outside plan 3's lease and THREAT-MODEL.md:265 was not on D-13's line list (117, 188, 300), so three lines still say kind observation is refused: 0006:127, 0006:144 and THREAT-MODEL.md:265.
fix: 8903ee7, retest

### 10. Three comments still describe old-layout handling the phase removed
expected: behavior wrong (comment only) - the comments describe a column bring-forward and table-age cases that 1.0.0 layouts no longer reach, and one now contradicts the test above it
origin: verifier
status: pass
first_pass: fail
source: model
evidence: After a0d59dc: grep -nE 'column bring-forward|naming columns' store/open.rs, grep -n 'predates `observations`' cmd/observations.rs and grep -nE 'only shape age|damage, not age' tests/replay.rs -> no line each, rc=1. cargo test -p terminus --test replay -> 7 passed, 0 failed; the three files pass rustfmt under the workspace's edition 2021.
reported: behavior wrong (comment only) - the comments describe a column bring-forward and table-age cases that 1.0.0 layouts no longer reach, and one now contradicts the test above it
severity: cosmetic
cause: The phase emptied BRING_FORWARD_COLUMNS and made a missing decision log damage, but three comments outside the lines the plans edited still describe the old cases: store/open.rs:176-179 says gating the column bring-forward would leave commands naming absent columns, cmd/observations.rs:152-155 says the write open is how a store that predates observations acquires it, and tests/replay.rs:354-357 says missing both tables is the shape age produces, which the test above it at :303 now calls damage.
fix: a0d59dc, retest

## Summary

total: 10
passed: 10
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 2
