---
status: testing
phase: 2
fields_version: 1
started: 2026-10-20
updated: 2026-10-20
---

## Items

### 1. Cold start from scratch
expected: With TERMINUS_DATA_DIR pointed at an empty scratch directory, a fresh ingest creates the store with no migration error, and `terminus status` afterwards prints real session and turn counts, not zeroes.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: Fresh scratch data dir: `ingest tests/fixtures/session-basic.jsonl` exit 0, empty stderr; `status` printed `sessions       1` and `turns          8`.

### 2. No store: status and verify say so and create nothing
expected: With TERMINUS_DATA_DIR naming a directory that does not exist, `terminus status` and `terminus verify` each exit 0, print nothing on stdout and the no-store reason on stderr, and the directory still does not exist afterwards. With `--json`, each prints one document with `ok: true`, the reason, and every `data` field at zero or null.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: Nonexistent TERMINUS_DATA_DIR: status, status --json, verify, verify --json all exit 0; text runs have empty stdout and `no terminus store at <dir>/terminus.db; nothing has been archived yet` on stderr; the status document has ok true, the reason, store/excluded/last_run null and size_bytes/sessions/turns/watermarks/watermark_bytes 0; the verify document has checked 0 and failures []; the data dir never existed afterwards. Path: lib.rs:146,148 -> status.rs:41 / verify.rs:14 -> read.rs:103-107 -> read::empty. Named tests pass.

### 3. A database with no tables reads as no store
expected: A `terminus.db` that holds no tables at all gets the same no-store answer from `status` and `verify`, and still holds no tables afterwards; a `terminus.db` holding a table but no `meta` still fails as not a terminus store.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: read.rs:112-116 plus store::holds_no_tables (open.rs:423). A no-table WAL terminus.db got the no-store answer from status, verify (both modes), search and sessions, all exit 0, with 0 tables afterwards; a terminus.db with one foreign table exits 1 from all four status/verify runs with `exists but is not a terminus store`. Unit and integration tests for both shapes pass.

### 4. Older store is reported and its schema is left alone
expected: On a format-2 store older than the binary, `status` and `verify` exit 0 and print the stale-store line on stderr, and `sqlite_master` and every table's `PRAGMA table_info` are the same after the run as before.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: Aged store (derived_schema 4 of 5, decisions.compacted and decisions.dropped dropped): HEAD status/verify in both modes exit 0 with `predate this build` on stderr, and the 131-row sqlite_master plus pragma_table_info record is byte-identical afterwards; the 3900fed binary gives no stale line and adds the two columns back. status_on_an_older_store_says_so_and_changes_nothing and verify_on_an_older_store_says_so_and_changes_nothing pass.

### 5. Ordinary store output unchanged
expected: On an ordinary store, `status --json` and `verify --json` print the same output as the pre-phase binary, `size_bytes` included.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: 3900fed binary vs HEAD binary on one ingested store, run OLD, NEW, OLD, NEW: stdout md5 identical for status --json, status, verify --json and verify; stderr identical; size_bytes 204800 every time. Linux only; macOS is a separate human check.

### 6. Malformed terminus.toml cannot stop verify
expected: With a malformed `terminus.toml`, `terminus verify` on a clean store exits 0 and reports no failures.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: verify.rs:14 uses Config::default(). With `this is = = not toml` as terminus.toml: `verify` exit 0, empty stdout, `1 session(s) checked, 0 failed`; `verify --json` exit 0, ok true, checked 1, failures []. verify_checks_a_clean_store_past_a_config_that_does_not_parse passes.

### 7. Docs no longer say status or verify creates a store
expected: `THREAT-MODEL.md`, `docs/commands.md` and `docs/json-shapes.md` no longer say `status` or `verify` creates a store, and `CHANGELOG.md`'s Unreleased Fixed section has an entry naming #77.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: All eight Task 4 grep probes go from 1 match at 3900fed to 0 at HEAD; THREAT-MODEL.md:75 matches the Store::open callers in code; docs/commands.md:110,122 and docs/json-shapes.md state the no-store answer; CHANGELOG.md:207-217 names #77 under Unreleased `### Fixed` ahead of `## [0.1.1]` at line 219.

### 8. Regression tests fail before and pass after
expected: The AC1 and AC2 regression tests fail against the code the phase started from (3900fed) and pass after it, and `cargo test --workspace --all-features` passes.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: With HEAD's tests overlaid on a 3900fed tree, the four new status tests, the three new verify tests, json_mode_keeps_stdout_pure_and_warnings_on_stderr and the new no-table read unit test all FAIL; at HEAD `cargo test --workspace --all-features` exit 0, 1001 passed, 0 failed.

### 9. Issue #77 closed by the merged change
expected: Issue #77 is closed by the merged PR. (human-verify: needs the merged PR on GitHub)
criterion: AC7
status: skipped
reported: skip it
reason: No PR yet; settled when the PR merges with Closes #77.

### 10. A no-table database read leaves SQLite sidecars while the phase says it creates nothing
expected: behavior wrong vs claim - a read of a no-table WAL database leaves terminus.db-shm and an empty terminus.db-wal, but the new doc comment, two new test names and the docs say the read creates nothing, and AC1 extends 'create nothing' to this case
origin: verifier
status: pass
first_pass: fail
source: model
evidence: git grep for the four false phrasings (creates, writes and repairs nothing; no_tables...creates_nothing; exits 0, and creates nothing; read-only and create nothing) over crates, docs and THREAT-MODEL.md -> grep-exit 1. cargo test -p terminus --lib read::tests -> test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 78 filtered out; finished in 0.00s. cargo test -p terminus --test status -> test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s. Fixed in 6c0fa27: open.rs doc now says it creates no table and can leave -shm and an empty -wal; the two test names and docs/commands.md say no table / no store.
reported: behavior wrong vs claim - a read of a no-table WAL database leaves terminus.db-shm and an empty terminus.db-wal, but the new doc comment, two new test names and the docs say the read creates nothing, and AC1 extends 'create nothing' to this case
severity: minor
fix: 6c0fa27, retest

### 11. Confirm size_bytes parity on macOS through the PR's macos-15 and macos-15-intel CI jobs
expected: The status, uninstall and lifecycle test jobs pass on both macOS runners, so a read-only open leaves the same -shm and -wal footprint there as on Linux and status --json reports the same size_bytes as before.
origin: verifier
why_human: Out-of-reach resource: no macOS host is available here, and the only macOS run is GitHub Actions on the pushed PR, which needs the network. This is not a truth the model chose not to probe; truth 5 is verified on Linux only.
status: skipped
reported: skip it
reason: No PR yet; settled by the PR macos-15 and macos-15-intel CI jobs.

## Summary

total: 11
passed: 9
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 1
