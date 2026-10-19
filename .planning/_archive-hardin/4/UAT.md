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
status: pass
first_pass: fail
source: model
evidence: Fresh sqlite3 .backup of the live store (no decision files waiting), then one `terminus ingest` with the release build of 9829de5, TERMINUS_DATA_DIR and TERMINUS_CONFIG_DIR set to scratch dirs. Exit 0 in 107 s. runs.error is the one summary line 're-keyed 2775 session(s) to their path below the project folder and merged 2203 twin(s) into the older session', with files_failed 0 and no 'not merged' line. Before: 4,978 sessions, all absolute keys, 510,635 turn uuids held by more than one session. After: `terminus status --json` shows 2,775 sessions and 607,083 turns; 0 keys begin with / or -. On disk, 2,784 transcript paths (10 of them links) reduce to 2,774 keys, each held by exactly one session, and no key is held by two regular files; the one archive-only session is the orphan. Orphan: 388a000f-7ac1-4333-9880-96d8ab237e51.jsonl is still session 2000 with 857,565 bytes, 14 blocks and the same checksum (67BCA0D4...CBC15498) as before. Dangling: 0 labels and 0 candidate, injected or suppressed decision turn ids point at a missing turn, before and after. `terminus verify`: 2775 session(s) checked, 0 failed. No two sessions share a checksum. The 9,022 uuids still held by two sessions all sit in sessions of different transcripts: 6,843 where one continues the other, 1,798 where both continue one session, 381 between sidecars of one session, none other. `terminus search --json --project '*'` over 8 queries (symlink, watermark, retention, brillig, sentinel, ingest, session key, compaction): 7 return 10 distinct records of 10 hits; 'retention' returns 9 of 10, the repeat being one record in efced699-... and in 6825b00d-..., whose continues_from is efced699, a record Claude Code copied into the resumed file. MCP recall_search for 'symlink' with project '*' returns 10 distinct records, where before 4 of its 10 showed twice under be0a247d-.../wf_c61a181c-c13/. That meets AC4 as John ruled it in PLAN-4 Notes: each transcript is archived once. A second pass committed 1 grown live file, runs.error null, sessions still 2,775.
severity: major
cause: The transcripts moved from one project folder to another (-code-cadence to -code-cadence-v3-9-1) and the old files are gone. D-01 keys on the path below `projects`, which includes the project folder, so the moved file is a new key and the archive-only copy is kept untouched under D-12. These are the '175 new-key sessions whose files are also gone' that D-12 counted as having no twin: they have no twin under the same key, but each is a copy of a live session in another project folder. The repair only pairs sessions whose old keys reduce to the same new key (D-10), so it never looks across project folders.
fix: plan 4 (b48c8c3 key drops the project folder), retest on a fresh copy of the live store

### 6. session_key output and the docs carry the new key
expected: session_key in --json output and MCP results is the path below projects. THREAT-MODEL.md, docs/json-shapes.md and design docs 0003 and 0004 describe the new key, and CHANGELOG.md [Unreleased] carries #57 under ### Fixed.
criterion: AC5
status: pass
first_pass: pass
source: model
evidence: The key is now the path below the project folder, per John's override of D-01 in PLAN-4 Notes, which AC5's wording 'the path below projects' predates. On temp dirs with the release build of 9829de5: `terminus search --json --project '*' brillig` and the MCP recall_search result both show session_key '11111111-1111-4111-8111-111111111111.jsonl', and `terminus sessions --json --project '*'` shows that key and '11111111-.../subagents/agent-a.jsonl'. THREAT-MODEL.md:240 (Session identity, keeping the forged-sessionId property and naming the same session file in two project folders), 0003-ingest.md:18, 0004-archive-and-block-rows.md:66 and json-shapes.md:73/:88/:315/:543 describe the key below the project folder. The grep from PLAN-4 Task 4 for the old forms ('below `projects`', 'project dir>/', '-code-app/', '-code-verbatim/', '-data-code-app/', '-abs-path-repo/') prints nothing in those five files, and the Task 1 grep prints nothing under crates. CHANGELOG.md's #57 entry sits under [Unreleased] ### Fixed (heading at :35, entry ending :191) and says moving a project folder keeps the key, the repair merges the copies a moved project folder left, and retention no longer deletes a moved session.
fix: plan 4 changed the key form (b48c8c3) and these docs (a24a04b) after this passed, retest

### 7. Each regression test fails before the fix and passes after
expected: Each phase 4 regression test fails on the commit before the fix and passes after it.
criterion: AC6
status: pass
first_pass: pass
source: model
evidence: Plans 1-2's tests, as recorded before: at an exported copy of c505fb0 with the tests added, the move test FAILED 'session count moved from 3 to 6' and the planted-store test FAILED 'the session count is 15 where one per transcript is 11'. Plan 4's three tests: in `git archive 742e352` with tests/pass.rs, tests/session_keys.rs and tests/retention.rs copied in from HEAD unchanged, `cargo test -p terminus-core --all-features --locked` compiled them and all three FAILED: a_moved_project_folder_archives_nothing_twice 'the session count moved from 2 to 4 when the project folder moved' (pass.rs:969); one_pass_merges_the_copies_a_moved_project_folder_left 'the session count is 6 where one per transcript is 3' (session_keys.rs:799); a_moved_claude_dir_keeps_every_session_whose_transcript_is_still_on_disk 'rows of [sessions, session_meta, turns, observations] went missing: the session was deleted while its transcript is still on disk', left [0, 0, 0, 0] right [1, 1, 8, 1] (retention.rs:887). All of them pass at HEAD 9829de5: full suite 979 passed, 0 failed, with the three hook-budget skips test.yml uses.
fix: plan 4 added three regression tests (ebf7b65, 95411fa) after this passed, retest them at 742e352

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
status: pass
first_pass: fail
source: model
evidence: Release build of 9829de5 on temp dirs. Run 1 with CLAUDE_CONFIG_DIR=a archived one session (8 turns, 1 observation row). Then `mv a b` with no symlink and terminus.toml [retention] action = "delete", age_days = 30 (the fixture's turns are from 2026-08-12). Run 2 with CLAUDE_CONFIG_DIR=b: files_seen 1, committed 0, turns_added 0, notes NULL (no deletion); session 0 keeps its checksum, 8 turns and its observation row, session_meta.transcript_path now names b/projects/-tmp-p/11111111-...jsonl, and `terminus sessions --json --project '*'` lists it with watermark 5656. Run 3: committed 0, turns_added 0, notes NULL, nothing changed. a_moved_claude_dir_keeps_every_session_whose_transcript_is_still_on_disk (crates/terminus-core/tests/retention.rs:817) passes in the full suite (979 passed, 0 failed).
reported: behavior wrong - an up-to-date pass never refreshes session_meta.transcript_path. After .claude moves with no symlink left behind, that column names a path that is gone, so retention's delete rule removes a session whose transcript is still on disk under the new root, along with its observation row, and the next pass archives the file a second time from offset 0.
severity: minor
cause: An up-to-date pass returns at ingest/mod.rs:515-517 before the session_meta upsert at :1397, the only writer of transcript_path, so after a move with no symlink the stored path of every unchanged file names a path that is gone. Retention delete (retention/mod.rs:206) asks transcript_is_gone of that stored path and deletes a session whose file is still on disk under the new root.
fix: plan 4 (95411fa records where an unchanged file was read from), retest

### 11. John reads each changed doc in RustRover
expected: THREAT-MODEL.md, docs/design/0003-ingest.md, docs/design/0004-archive-and-block-rows.md, docs/json-shapes.md and the CHANGELOG.md #57 entry read as his own, and say what the code does.
origin: verifier
why_human: A judgment only John can make. PLAN-3 Task 1 makes his review of the wording its acceptance, because this is prose he puts his name on. The factual content was checked here against the code.
status: pass
first_pass: pass
source: model
reported: fine go
fix: a24a04b changed these docs again after this passed, re-read

### 12. A moved project folder keeps its session count
expected: After a pass over a transcript tree, renaming a project folder inside projects with no symlink left behind and running ingest again, terminus status shows the same session count, and the second pass commits no file and adds no turn.
status: pass
first_pass: pass
source: model
evidence: Release build of 9829de5 on temp dirs, a tree holding projects/-tmp-old/11111111-...jsonl and its sidecar 11111111-.../subagents/agent-a.jsonl. Run 1: files_seen 2, committed 2, 11 turns; `terminus status --json` shows 2 sessions and 11 turns, keyed 11111111-...jsonl and 11111111-.../subagents/agent-a.jsonl. After `mv projects/-tmp-old projects/-tmp-new` with no symlink, run 2: files_seen 2, committed 0, turns_added 0, error NULL; status still shows 2 sessions and 11 turns under the same two keys. a_moved_project_folder_archives_nothing_twice (crates/terminus-core/tests/pass.rs:927) passes in the full suite.

## Summary

total: 12
passed: 11
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 2
