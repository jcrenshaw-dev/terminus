---
status: testing
phase: 4
fields_version: 1
started: 2026-10-19
updated: 2026-10-19
---

## Items

### 1. Cold start on an empty store
expected: With an empty data dir, one ingest pass over a small transcript tree opens and creates the store cleanly, recovery runs with nothing to repair, and a search returns real turns from that tree.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: On temp dirs, `terminus ingest` on an empty data dir exited 0 with runs row 1 committing 3 files and error null; `terminus verify` printed '3 session(s) checked, 0 failed'; `terminus search --json --project '*' brillig` returned the archived turn. recovery_with_nothing_to_repair_leaves_decision_files_waiting (crates/terminus-core/tests/session_keys.rs:675) passes.

### 2. A moved tree behind a symlink keeps its session count
expected: After a pass over a transcript tree, renaming the tree, leaving a symlink at the old path and running ingest again, terminus status shows the same session count as before the move.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: a_moved_claude_dir_behind_a_symlink_archives_nothing_twice (crates/terminus-core/tests/pass.rs:848) passes. By hand, after renaming the tree and leaving a symlink, runs went from 4 seen/3 committed to 4 seen/0 committed, and `terminus status --json` showed sessions 3 before and after.

### 3. A symlinked sidecar is archived once
expected: A sidecar that is a symlink to another session's sidecar under projects is archived once, under the key of the file it points at.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: claim (ingest/mod.rs:194) resolves the link before path_key, in pass::walk (pass.rs:385) and backfill (backfill.rs:324). By hand, `terminus ingest` and `terminus backfill` on a 4-path tree with one sidecar link each left 3 sessions, with the link keyed as -tmp-p/11111111-.../subagents/agent-a.jsonl, its target's key.

### 4. One pass repairs a planted doubled store
expected: On a planted store holding an identical twin pair, a prefix twin pair, a link pair and an archive-only orphan, one ingest pass leaves one session per transcript, the prefix pair keeps the longer stream, the orphan's bytes are unchanged and terminus verify passes. A planted pair whose block hashes are not a prefix of each other is reported and both its sessions remain.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: one_pass_repairs_a_store_keyed_on_absolute_paths (crates/terminus-core/tests/session_keys.rs:340) passes, covering all of D-14's cases plus a tail-block pair and a three-member group, with verify, foreign_key_check, reindex equality and a no-op second pass. A schema-4 probe of the binary also repaired cleanly, and `terminus verify` was clean afterwards.

### 5. The repair on a copy of the live store
expected: On a copy of the live store, after one ingest pass, recall_search returns each turn once, terminus status counts each session once, the -code-cadence/388a000f orphan is still archived, and no label or decision turn id points at a turn that does not exist.
criterion: AC4
status: fail
first_pass: fail
source: model
evidence: Copied the live store with sqlite3 .backup into the scratchpad and ran one `terminus ingest` with the HEAD release binary (TERMINUS_DATA_DIR and TERMINUS_CONFIG_DIR set to scratch dirs). Exit 0 in 106 s; the run's note reads 're-keyed 2922 session(s) to their path below `projects` and merged 2028 twin(s) into the older session'. Before: 4,950 sessions, all absolute keys, 509,952 turn uuids held more than once. After: `terminus status --json` shows 2,923 sessions and 614,555 turns, no absolute key is left, and 21,165 uuids are held more than once. Orphan: session 2000 is now -code-cadence/388a000f-7ac1-4333-9880-96d8ab237e51.jsonl, with 857,565 bytes, 14 blocks and the same checksum as before. Dangling: 0 labels and 0 injected, candidate or dropped decision turn ids point at a missing turn (0 before as well). `terminus verify`: 2923 session(s) checked, 0 failed. Every regular transcript on disk has one session; the 10 links are keyed on their targets; 176 sessions are archive-only. recall_search with project '*', over 8 queries: 7 returned no turn twice. 'symlink' returned 4 of its 10 hits twice, under -code-cadence/be0a247d-.../subagents/workflows/wf_c61a181c-c13/ and under -code-cadence-v3-9-1/ with the same path. 175 transcripts are archived under both -code-cadence/ and -code-cadence-v3-9-1/ with the same path below the project folder: 173 have identical checksums, 1 has the old copy longer and 1 has it shorter. All 175 -code-cadence/ files are gone from disk, so 12,897 turns show twice. The live 0.1.1 store already held both copies (e.g. sessions 4448 and 4727, both under /data keys), so this did not come from #57's root move.
severity: major
cause: The transcripts moved from one project folder to another (-code-cadence to -code-cadence-v3-9-1) and the old files are gone. D-01 keys on the path below `projects`, which includes the project folder, so the moved file is a new key and the archive-only copy is kept untouched under D-12. These are the '175 new-key sessions whose files are also gone' that D-12 counted as having no twin: they have no twin under the same key, but each is a copy of a live session in another project folder. The repair only pairs sessions whose old keys reduce to the same new key (D-10), so it never looks across project folders.
fix: routed to /cad-plan

### 6. session_key output and the docs carry the new key
expected: session_key in --json output and MCP results is the path below projects. THREAT-MODEL.md, docs/json-shapes.md and design docs 0003 and 0004 describe the new key, and CHANGELOG.md [Unreleased] carries #57 under ### Fixed.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: `terminus sessions --json`, `terminus search --json` and the MCP recall_search result all show '-tmp-p/11111111-1111-4111-8111-111111111111.jsonl'. THREAT-MODEL.md:12/:17/:21/:240, 0003-ingest.md:18/:35/:133-140, 0004:15/:66 and json-shapes.md:76-80/:86/:313/:390 describe the path below projects. CHANGELOG.md:174-188 carries #57 under [Unreleased] ### Fixed.

### 7. Each regression test fails before the fix and passes after
expected: Each phase 4 regression test fails on the commit before the fix and passes after it.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: At an exported copy of c505fb0 with the tests added: the move test FAILED 'session count moved from 3 to 6', and the planted-store test FAILED 'the session count is 15 where one per transcript is 11'. Both pass at HEAD.

### 8. The landing PR closes #57
expected: The pull request that lands this phase carries Fixes #57, and #57 closes once it merges. (human-verify: needs GitHub, after landing)
criterion: AC7
status: skipped
reported: yes skip
reason: checked at landing: the PR has not merged yet

### 9. A second root's different file under a held key is refused
expected: When two roots hold different files that reduce to the same key, the second is refused unread in the pass, backfill and single-file ingest, and never appends onto the first root's session.
status: pass
first_pass: pass
source: verifier
evidence: a_second_roots_file_never_appends_onto_the_first_roots_session (crates/terminus-core/tests/ingest.rs:583) passes. By hand with two roots in terminus.toml, ingest and backfill each kept 3 sessions and 14 turns with the held stream at 5,656 bytes, and runs.error named both files. Single-file ingest of the second file exited 1 naming both.

### 10. A move with no symlink lets delete retention remove sessions whose files are still on disk
expected: behavior wrong - an up-to-date pass never refreshes session_meta.transcript_path. After .claude moves with no symlink left behind, that column names a path that is gone, so retention's delete rule removes a session whose transcript is still on disk under the new root, along with its observation row, and the next pass archives the file a second time from offset 0.
origin: verifier
status: fail
first_pass: fail
source: verifier
evidence: crates/terminus-core/src/ingest/mod.rs:515-517: apply returns Outcome::UpToDate before the session_meta upsert at :1390-1397, which is the only place ingest writes transcript_path. crates/terminus-core/src/retention/mod.rs:206 and :248-253: the delete rule asks transcript_is_gone of that stored path. CONTEXT.md D-13 says ingest 'already overwrites [transcript_path] on every pass', which is false for an unchanged file. PLAN-2.md Task 3 (line 89) names this exact hazard, 'an old root's path would read as gone after a move made without a symlink', and fixes it only for merged twins. Probe on temp dirs: run 1 archived one session (8 turns). I then moved the root (mv a b, no symlink), set CLAUDE_CONFIG_DIR=b and [retention] action = "delete", age_days = 30. Run 2 had files_seen 1 and committed 0, and its notes read 'retention deleted 1 session(s), removing 1 observation row(s) with them - the session's archive went too, so no rebuild reproduces them', while b/projects/-tmp-p/11111111-...jsonl (5,656 bytes) was still on disk. Run 3 archived it again from offset 0 with 8 turns added. On a real store the new session_no would differ, leaving label and decision turn ids naming nothing. Not a regression: before this phase the same move archived new copies under new keys and retention deleted the old ones, losing the same rows.
reported: behavior wrong - an up-to-date pass never refreshes session_meta.transcript_path. After .claude moves with no symlink left behind, that column names a path that is gone, so retention's delete rule removes a session whose transcript is still on disk under the new root, along with its observation row, and the next pass archives the file a second time from offset 0.
severity: minor
cause: An up-to-date pass returns at ingest/mod.rs:515-517 before the session_meta upsert at :1397, the only writer of transcript_path, so after a move with no symlink the stored path of every unchanged file names a path that is gone. Retention delete (retention/mod.rs:206) asks transcript_is_gone of that stored path and deletes a session whose file is still on disk under the new root.
fix: routed to /cad-plan

### 11. John reads each changed doc in RustRover
expected: THREAT-MODEL.md, docs/design/0003-ingest.md, docs/design/0004-archive-and-block-rows.md, docs/json-shapes.md and the CHANGELOG.md #57 entry read as his own, and say what the code does.
origin: verifier
why_human: A judgment only John can make. PLAN-3 Task 1 makes his review of the wording its acceptance, because this is prose he puts his name on. The factual content was checked here against the code.
status: pass
first_pass: pass
reported: fine go

## Summary

total: 11
passed: 8
failed: 2
pending: 0
skipped: 1
blocked: 0
reworked: 2
