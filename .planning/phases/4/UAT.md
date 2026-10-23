---
status: testing
phase: 4
fields_version: 1
started: 2026-10-23
updated: 2026-10-23
---

## Items

### 1. An unindexable word changes no hit, and a pointed Hebrew word still narrows
expected: `terminus search` for `foo` plus a standalone U+0345 prints the same hits as a search for `foo`, and so does MCP `recall_search`. A search for `foo` plus a Hebrew word with vowel points returns only turns that contain both words.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: query.rs:91 drops all-unindexable pieces in Query::parse before the dedup and cap. CLI and MCP tests pass. Branch-binary spot-check on a scratch store: 'quokkafoo' and 'quokkafoo U+0345' give the same two hits through the CLI and MCP; 'quokkafoo <pointed shalom>' gives only the turn holding both; 'U+0345' alone gives 'the query has no words that can be searched'.

### 2. The tokenizer sweep test guards the table
expected: A test runs every code point Rust counts as alphanumeric through the bundled SQLite's tokenizer; it passes on the tree and fails when an entry is removed from the Rust table.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: unindexable.rs:268 sweeps every char::is_alphanumeric code point through an in-memory FTS5 table and fts5vocab, and passes in the full run. The table has 167 ranges and 1,061 code points. With the U+0345 row deleted in a scratch copy, the test fails naming U+0345 and prints the regenerated table.

### 3. An impossible date is misuse naming the flag, from the CLI and MCP
expected: `terminus search --since 2026-02-30 x`, `terminus sessions --until 2026-13-01`, `terminus observations regenerate --since 2026-02-30` and `terminus search --until 2026-08-12T25:00:00.000Z x` each exit 2 with a message naming the flag as typed, including on a data directory with no store. MCP `recall_search` with `since: "2026-02-30"` returns `isError` with a message naming `since`.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: Branch binary on a store-less scratch data dir: the four commands each exit 2 with empty stdout and stderr naming --since or --until with the value; the data dir is not created; real edge dates exit 0. MCP since '2026-02-30' gives isError true with a reason naming since and no '--since'. Named CLI, observations and MCP tests pass.

### 4. One core date check, run before the scope
expected: `git grep -n 'fn time_bound' crates/` prints nothing, and the `InvalidTimeFilter` arms in `cmd/search.rs` and `cmd/mcp/tools.rs` are gone. A core test that calls `search::run` with a bad date from a directory no project covers gets `InvalidTimeFilter`, not an unknown-project answer.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: fn time_bound and the terminus-side InvalidTimeFilter arms grep empty. search.rs:342 checks the bounds before scope::resolve at :343. a_bad_date_is_refused_before_any_scope_answer passes, and fails with Ok(UnknownProject) when the check is moved after the scope in a scratch copy.

### 5. Every list is one parameter and the paths cap is gone
expected: `git grep -nE 'repeat_n\("\?"' crates/` prints nothing and `git grep -n MAX_PATHS crates/ docs/` prints nothing. A core test that sends 32,767 paths through `search::run` gets hits with no error, and the MCP test that sends 40,000 paths gets hits with no error and no cap notice.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: The repeat_n and MAX_PATHS greps are empty, and json_each now binds the lists in search.rs, get.rs, excerpt.rs and scope.rs. The core 40,000-path test and the rewritten MCP 40,000-path test pass. Branch-binary spot-check: 40,000 --path values via the CLI, and 40,000 paths via MCP, each return the one real hit with no error and a null reason.

### 6. Sessions with no project survive an exclusion
expected: With one project excluded in config, a session with no project still appears in `terminus search` results, and with a worktree excluded, the repository session with no pre-worktree key is still quoted and counted in the session-start brief.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: scope.rs:186 keeps the IS NULL guard on both json_each NOT IN clauses. a_session_with_no_project_survives_an_exclusion and a_session_excluded_under_its_worktree_key_is_neither_quoted_nor_counted pass, and both fail when the guard is removed in a scratch copy.

### 7. The workspace suite passes
expected: `cargo test --workspace --all-features` passes.
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: cargo test --workspace --all-features at the branch head: 1,028 passed, 0 failed, exit 0, no warning lines.

## Summary

total: 7
passed: 7
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 0
