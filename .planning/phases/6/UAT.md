---
status: testing
phase: 6
fields_version: 1
started: 2026-10-20
updated: 2026-10-20
---

## Items

### 1. Cold start on a fresh data directory
expected: With TERMINUS_DATA_DIR pointing at a path that does not exist, `terminus status` and `terminus verify` report no store and create nothing; after an ingest into that directory, `status` reports real session and turn counts.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: HEAD build, missing TERMINUS_DATA_DIR: status --json and verify --json exit 0 with the no-store reason and zero/null fields, data dir still absent afterwards; after `terminus ingest` of three fixtures, status --json reports sessions 3, turns 16, watermarks 3.

### 2. Every applied finding is committed and every skip has its reason
expected: Each finding under SIMPLIFY.md's Apply (1, 3, 5, 6, 7, 8, 9, 10, 11, 13, 16) maps to a commit in the phase, and every entry under Skip carries a reason.
status: pass
first_pass: pass
source: verifier
evidence: Apply 1,3,5,6,7,8,9,10,11,13,16 map one-to-one to 30d8fb8, 2b061ee, 61298b0, 7af18b3, 49996bb, 9fe99ac, 2ad68ab, c963a35, a225950, afd86d1, 99cc79d, each touching the files its finding names; every Skip entry at SIMPLIFY.md:23-33 states its reason.

### 3. The full suite passes unchanged
expected: `cargo test --workspace --all-features --no-fail-fast` reports 0 failed, and `git diff` from the phase's starting commit adds, removes or changes no line containing `assert`.
status: pass
first_pass: pass
source: verifier
evidence: cargo test --workspace --all-features --no-fail-fast: exit 0, 1015 passed, 0 failed. git diff 24c90b4 HEAD -- crates changes 0 lines containing assert, and removes no #[test].

### 4. Code outside the 1.0.0 changes is untouched
expected: Every file the phase changed is in the main...traders diff from before the phase, except parse/mod.rs and testkit.rs, which findings 1 and 16 name and John adjudicated as Apply.
status: pass
first_pass: pass
source: verifier
evidence: Of the 16 code files the phase changed, only parse/mod.rs and testkit.rs fall outside the 40-file pre-phase main...traders diff; every other changed path is under .planning/.

### 5. No-store status and verify JSON are byte for byte unchanged
expected: With no store, `terminus status --json` and `terminus verify --json` print the same bytes from a build of the phase's starting commit and from HEAD.
status: pass
first_pass: pass
source: verifier
evidence: Builds of 24c90b4 and HEAD, same env with a missing data dir: cmp finds stdout and stderr identical for status --json and verify --json.

### 6. The removed pieces are gone
expected: `grep -rn "consumed(" crates` and `grep -n Gate crates/terminus-core/src/ingest/pass.rs` print nothing, `holds_no_tables` survives only as a unit test name, and `record_key`/`reset` are gone from archive.rs.
status: pass
first_pass: pass
source: verifier
evidence: grep for consumed( in crates and Gate in pass.rs prints nothing; holds_no_tables only at cmd/read.rs:299 (unit test name); record_key and reset absent from archive.rs.

### 7. The phase's files are fmt-clean
expected: `cargo fmt --all --check` names none of the 16 files the phase changed.
status: pass
first_pass: pass
source: verifier
evidence: cargo fmt --all --check flags only cmd/hook.rs, tests/hook.rs, tests/release_workflows.rs and tests/sign_release.rs, none of the 16 phase files.

### 8. Issue #90 is closed by the merged change
expected: The landing PR body carries `Fixes #90` and #90 closes when it merges (human-verify: needs the merged PR on GitHub)
status: skipped
reported: skip it
reason: checked at landing: no PR from traders exists yet and #90 is open; /cad-land writes the PR body with Fixes #90

## Summary

total: 8
passed: 7
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 0
