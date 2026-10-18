---
status: testing
phase: 14
fields_version: 1
started: 2026-10-18
updated: 2026-10-18
---

## Items

### 1. Cold start: fresh binary serves real data
expected: From a clean build of terminus, a freshly spawned `terminus mcp` over a store with archived sessions answers one recall_search tools/call with real hits and exits 0 at EOF.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: After a fresh cargo build -p terminus and a scratch store ingested from tests/fixtures/session-recall.jsonl, one recall_search (query src/worker/S.ts) piped to target/debug/terminus mcp from root/project-alpha gave exit 0, one result, isError false, 1 real hit (turn_id 1, project-alpha, reason null), and nothing on stderr.

### 2. server/discover answers the D-04 object
expected: `server/discover` with `_meta` naming 2026-07-28 returns resultType "complete", supportedVersions exactly ["2026-07-28"], capabilities.tools an object, _meta serverInfo with name terminus and the crate version, ttlMs 0, cacheScope "private", and no instructions key. The same request with no _meta gets the identical result.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: On the binary, server/discover with _meta 2026-07-28 and with no params returned identical results: resultType complete, supportedVersions [2026-07-28], capabilities {tools:{}}, _meta serverInfo {name terminus, version 0.1.1}, ttlMs 0, cacheScope private, no instructions key. discover() is at mod.rs:236-243. The whole-object unit test server_discover_answers_the_one_stateless_revision_with_or_without_meta passed.

### 3. Every result carries the envelope; tools/list carries cache fields
expected: initialize, ping, tools/list, tools/call and server/discover results all carry resultType "complete" and _meta serverInfo. tools/list carries ttlMs 0, cacheScope "private" and the three tools in their fixed order, with or without _meta.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: rpc::result (rpc.rs:165-174) is the one place the envelope is added, and every result arm of route (mod.rs:186-216) goes through it. On the binary, initialize, ping, tools/list, tools/call and server/discover all carried resultType complete and the serverInfo _meta. ping is exactly [_meta, resultType]. tools/list with and without _meta carries ttlMs 0, cacheScope private and recall_search, recall_context, recall_get in order. Unit test every_result_carries_the_result_type_and_the_servers_identity_on_both_eras passed.

### 4. Other stateless versions refused with -32022
expected: server/discover, tools/list and tools/call whose _meta names a version other than 2026-07-28 each get error -32022 with data.supported ["2026-07-28"] and data.requested echoing the asked version.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: On the binary: tools/list on 2025-11-25, server/discover on 2025-06-18 and tools/call on 2024-11-05 each got error -32022 with data {supported:[2026-07-28], requested:<the asked value>} and no result. The check runs before route (mod.rs:170-179, unsupported at mod.rs:224-227). Unit tests a_request_naming_another_revision_is_refused_with_what_is_supported and the_version_check_runs_first_and_never_on_initialize passed.

### 5. initialize unchanged; legacy tools/list needs no initialize
expected: initialize asking 2026-07-28 is answered 2025-06-18, each of the four legacy revisions is echoed, and 2026-07-28 is not in PROTOCOL_VERSIONS. tools/list with no _meta and no prior initialize still returns the tool list.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: Checked against D-08. initialize asking 2026-07-28 was answered 2025-06-18. 2025-11-25, 2025-06-18, 2024-11-05 and 2024-10-07 were each echoed. PROTOCOL_VERSIONS (mod.rs:74) holds only those four, and its doc comment (mod.rs:65-67) says why 2026-07-28 is absent. A bare tools/list as a process's first line returned the three tools. Unit tests initialize_negotiates_a_version_both_sides_know and a_tools_list_with_no_meta_and_no_initialize_is_served passed.

### 6. Stateless conversation against the spawned binary
expected: A spawned `terminus mcp` over a fixture store, fed server/discover, tools/list and one tools/call per tool, each with 2026-07-28 _meta and no initialize anywhere, answers every request with a result and exits 0 at EOF.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: Spawned target/debug/terminus mcp over the scratch fixture store with server/discover, tools/list, recall_search, recall_context and recall_get, all on 2026-07-28 _meta and with no initialize: exit 0, 5 results in order with no error, 1 search hit, context anchored on turn 1, get record for turn 1, and empty stderr. Integration test a_stateless_conversation_with_no_initialize_answers_every_request passed.

### 7. Claude Code 2.1.302 connects on 2026-07-28
expected: `claude -p --debug mcp --strict-mcp-config --mcp-config` pointing at the built binary leaves a debug log under ~/.claude/debug/ showing the server on 2026-07-28 with no fallback to initialize, the three tools listed, and a result from each tool when the prompt asks for all three. (human-verify: needs a live Claude Code session)
criterion: AC6
status: pass
first_pass: pass
reported: accept the log
reason: John accepted /home/john/.claude/debug/51ee0b2d-9fe8-4437-a4a6-f330a4201178.txt (written 2026-10-17 21:45 after a3f6ec2, before the tests-only commit d186b22): protocolEra modern, negotiatedProtocolVersion 2026-07-28, no initialize request, Claude Code 2.1.302, recall_search, recall_context and recall_get each completed successfully.

### 8. Design doc and changelog document the era
expected: docs/design/0006-recall-through-mcp.md has a server/discover section and names resultType, ttlMs, cacheScope and -32022; its initialize section and figure 1 no longer present initialize as the only handshake. CHANGELOG.md [Unreleased] has an Added entry naming 2026-07-28.
criterion: AC7
status: pass
first_pass: pass
source: verifier
evidence: docs/design/0006-recall-through-mcp.md has '### server/discover' (line 104) and '### Which era a request is on' (line 88). The initialize section calls it the legacy opening. resultType, ttlMs, cacheScope and -32022 appear 9, 5, 5 and 9 times. Figure 1 has an alt with both openings and figure 2 draws the -32022 branch. CHANGELOG.md [Unreleased] ### Added lines 14-20 name 2026-07-28 and #45.

### 9. PR body carries Fixes #45
expected: The phase's pull request body carries `Fixes #45`.
criterion: AC7
status: skipped
reported: skip it
reason: No pull request exists yet; the PR is opened at /cad-land, where Fixes #45 goes in the body and CI runs on the push.

### 10. Claude Code 2.1.302 connects on 2026-07-28: write an MCP config naming target/debug/terminus with args [mcp], run claude -p --debug mcp --strict-mcp-config --mcp-config <that file> with a prompt asking for recall_search and then recall_context and recall_get on one of its hits, and read the newest log under ~/.claude/debug/
expected: The log shows protocolEra modern and negotiatedProtocolVersion 2026-07-28, no initialize sent, the three tools listed, and each of recall_search, recall_context and recall_get completed successfully. The existing log /home/john/.claude/debug/51ee0b2d-9fe8-4437-a4a6-f330a4201178.txt already shows all four, and it ran against the shipped server code (only test code changed after it), so John can accept that log or rerun.
origin: verifier
why_human: Out-of-reach resource: it needs a live Claude Code 2.1.302 session that makes model calls on John's account over the network, which this verifier may not start. The orchestrator also reserved AC6 for the human walk.
status: pass
first_pass: pass
reported: accept the log
reason: Same check as item 7, answered there: John accepted the existing debug log.

### 11. PR body carries Fixes #45: when /cad-land opens the phase's pull request, confirm its body has the line Fixes #45 and that CI passes on the push
expected: The PR body contains Fixes #45, merging it closes #45, and the test workflow (cargo test --workspace --all-features --locked) is green on all runners.
origin: verifier
why_human: Not observable yet: the PR is opened at /cad-land, after verification, so no PR body or CI run exists to inspect. This is a later step in the flow, not something a person has to judge.
status: skipped
reported: skip it
reason: Same check as item 9, answered there: the PR is opened at /cad-land.

## Summary

total: 11
passed: 9
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 0
