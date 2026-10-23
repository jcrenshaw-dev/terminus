---
status: testing
phase: 6
fields_version: 1
started: 2026-10-23
updated: 2026-10-23
---

## Items

### 1. Continuation link filled on the second tree pass
expected: A test runs a tree pass over a continuing session alone, then a second tree pass with its predecessor added, and the later session's continues_from names the predecessor
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: fill_continuations (crates/terminus-core/src/ingest/mod.rs:1559-1583), first in Tree::after_walk (crates/terminus-core/src/ingest/pass.rs:345). a_continuing_session_is_linked_once_its_predecessor_is_archived (crates/terminus-core/tests/lineage.rs:283-296) passed on the merged tree with files_committed == 1 on pass two. With the call removed in a scratch copy, it failed at lineage.rs:296 'left: None, right: Some(session(1))'.

### 2. show --json reports the filled link
expected: terminus show --json on the later session reports the predecessor in context.continues_from
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: Scratch CLI run of target/debug/terminus (rebuilt after HEAD, all config/data/claude dirs in the scratchpad). Pass 1 left continues_from NULL; pass 2 (files_committed=1) set it to 00000001-2222-4222-8222-222222222222. 'terminus show 0 --after 1 --project * --json' printed context.continues_from = "00000001-2222-4222-8222-222222222222", ok:true, exit 0.

### 3. A link already set is kept
expected: A session whose continues_from was already set keeps it after the sweep
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: The NULL-only select and UPDATE are at crates/terminus-core/src/ingest/mod.rs:1564 and 1578. a_link_already_recorded_survives_the_sweep (crates/terminus-core/tests/lineage.rs:301-313) passed on the merged tree. With both NULL guards removed in a scratch copy, it failed at lineage.rs:312, the foreign id overwritten by session(1). A CLI sidecar probe also stayed NULL, so there is no self-link.

### 4. Retention keeps a session whose transcript came back
expected: A session retention::evaluate selected, whose transcript is written back before retention::apply, keeps every row; apply deletes nothing, and the kept session is reported among retention's notes, not its errors
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: The re-check in delete_one is at crates/terminus-core/src/retention/apply.rs:195-202, with Applied.kept and its notes() line at apply.rs:98-104, 128. a_transcript_that_comes_back_before_the_delete_keeps_its_session (crates/terminus-core/tests/retention.rs:594-632) passed on the merged tree. With the check removed in a scratch copy, it failed at retention.rs:618. The notes() to runs.notes seam (crates/terminus-core/src/ingest/runs.rs:92) is carried by a_pass_with_an_evict_policy_evicts_the_aged_sessions_and_says_so (crates/terminus-core/tests/pass.rs:740), which passed.

### 5. Retention delete goes through clear_derived
expected: apply.rs has no DELETE FROM for paths, entities, compactions, turns_fts or turns; delete_session calls reindex::clear_derived; the existing deletion tests in tests/retention.rs pass
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: The DELETE FROM grep over crates/terminus-core/src/retention/apply.rs printed nothing (exit 1). The clear_derived call is at apply.rs:212, and 'pub(crate) fn clear_derived' at crates/terminus-core/src/reindex.rs:245 has its five statements unchanged. The four existing deletion tests in crates/terminus-core/tests/retention.rs passed on the merged tree, and the phase diff removes no test line.

### 6. Observe query plan guard passes on the tree
expected: The EXPLAIN QUERY PLAN test on ENTITY_VALUES_SQL passes and asserts idx_entities_turn is used and idx_entities_lookup never is
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: a_plan_that_starts_from_the_sessions_turns (crates/terminus-core/src/observe/mechanical.rs:469-491) plans ENTITY_VALUES_SQL (mechanical.rs:252-256) on a CREATE_SQL store and asserts idx_entities_turn present (484) and idx_entities_lookup absent (488). Passed on the merged tree (full-suite.log line 681).

### 7. Observe query plan guard fails on the kind-driven shape
expected: With the query put back to its kind-driven shape, the guard test fails
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: In a scratch git-archive copy with 'FROM entities e JOIN turns t ON t.id = e.turn_id', the guard test exited 101: 'panicked at crates/terminus-core/src/observe/mechanical.rs:483:9: the plan never uses idx_entities_turn: ["SEARCH e USING INDEX idx_entities_lookup (kind=?)", ...]'.

### 8. Workspace suite green, observe.rs expectations unchanged
expected: cargo test --workspace --all-features passes on the merged tree, and the phase diff changes no expected value in crates/terminus-core/tests/observe.rs
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: 'cargo test --workspace --all-features --offline' on merged HEAD d52eb752: passed=1041, failed=0, ignored=0 across 68 test binaries, exit 0. 'git diff --exit-code ccb3bf2..HEAD -- crates/terminus-core/tests/observe.rs crates/terminus-core/src/store' exited 0.

## Summary

total: 8
passed: 8
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 0
