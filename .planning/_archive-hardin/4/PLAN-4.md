---
phase: 4
plan: 4
requirements: [ING-13]
files:
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/recover.rs
  - crates/terminus-core/src/lineage.rs
  - crates/terminus-core/src/discover.rs
  - crates/terminus-core/src/verify.rs
  - crates/terminus-core/src/config.rs
  - crates/terminus/src/cmd/backfill.rs
  - crates/terminus-core/tests/ingest.rs
  - crates/terminus-core/tests/session_keys.rs
  - crates/terminus-core/tests/pass.rs
  - crates/terminus-core/tests/lineage.rs
  - crates/terminus-core/tests/discover.rs
  - crates/terminus-core/tests/retention.rs
  - crates/terminus/tests/lifecycle.rs
  - crates/terminus/tests/retention.rs
  - crates/terminus/tests/cli.rs
  - THREAT-MODEL.md
  - docs/json-shapes.md
  - docs/design/0003-ingest.md
  - docs/design/0004-archive-and-block-rows.md
  - CHANGELOG.md
---

# Phase 4: Session keys - Plan 4: the gaps UAT found

## Goal

Moving a project folder keeps every session's key, the same way moving `.claude` already does. As a result, the live store's 175 transcripts archived under both `-code-cadence/` and `-code-cadence-v3-9-1/` merge into one session each (UAT item 5). A pass over an unchanged file also records where that file is now, so retention's delete rule never removes a session whose transcript is still on disk (UAT item 10).

## Must be true when done

- After a pass over a transcript tree, renaming a project folder with no symlink left behind, and running ingest again, the session count is unchanged and the second pass commits no file and adds no turn.
- On a planted store keyed on absolute paths that holds one transcript under two project folders, with the old folder's file gone, one ingest pass leaves one session per transcript. Identical copies merge. Where the copy archived first is the shorter one and is gone, the kept session holds the live file's stream byte for byte, its watermark is the live file's length, and a second pass reports nothing.
- `session_key` in `--json` and MCP output is `<uuid>.jsonl` for a session, or `<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar. `THREAT-MODEL.md`, `docs/json-shapes.md`, design docs 0003 and 0004 and the `CHANGELOG.md` #57 entry describe that key. No code comment, test comment or doc still calls the key the path below `projects`.
- After `.claude` moves with no symlink left and retention's delete rule is on, the next pass keeps every session whose transcript is still on disk. Its `session_meta.transcript_path` names the file at its new place, and the pass after that archives nothing again. A pass over an unmoved, unchanged file still writes nothing.
- Each new regression test fails at 742e352, the commit before this plan, and passes after it.
- (human-verify: a fresh copy of the live store and the release binary built from this branch) AC4 holds. After one ingest pass on the copy, `terminus status` counts each transcript once, no transcript is archived under two keys, the `388a000f-7ac1-4333-9880-96d8ab237e51.jsonl` orphan is still archived with its bytes, `terminus verify` passes, and no label or decision turn id dangles. The steps are in Notes.

## Context

- John's ruling on Gap A overrides D-01's key form and is locked. It is recorded in Notes. D-02, D-03, D-10, D-11, D-12 and D-13 still bind. Only the form of the key changes, not how it is found or how twins merge.
- The repair in `recover.rs` needs no change in what it does. It reduces each stored absolute key with `ingest::path_key`, so the new key alone makes it pair copies across project folders. This was measured with a prototype on a copy of the live store (see Notes).
- Out of scope: re-keying stores that already hold D-01-form relative keys (see Notes), a new command, a switch (D-07), a format bump (D-06), and any change to discovery (D-04).
- The docs are prose John puts his name on and reviews in RustRover, as PLAN-3 Task 1's acceptance did.

## Tasks

### Task 1: Key a session on its path below the project folder

- **Files:** crates/terminus-core/src/ingest/mod.rs (`path_key`, `below_projects`, `Pass::session_key`, the held-key comment in `run_with`, `claim`, `ingest_locked`), crates/terminus-core/src/recover.rs (module comment, `Recovered::rekeyed`, `Recovered::lines`, `rekey`), crates/terminus-core/src/lineage.rs (module comment), crates/terminus-core/src/discover.rs (module comment, D-17 paragraph), crates/terminus-core/src/verify.rs (module comment "Which identifier names a failure", `Failure::session_key`), crates/terminus-core/src/config.rs (`Session::session_key` doc comment), crates/terminus/src/cmd/backfill.rs (`watermarks` doc comment), crates/terminus-core/tests/ingest.rs (`two_paths_to_one_transcript_produce_one_session`, `a_second_roots_file_never_appends_onto_the_first_roots_session`), crates/terminus-core/tests/session_keys.rs (`Bench::plant`, `one_pass_repairs_a_store_keyed_on_absolute_paths`), crates/terminus-core/tests/pass.rs (`key_of` doc comment), crates/terminus-core/tests/lineage.rs (`key` doc comment), crates/terminus-core/tests/discover.rs (`every_path_begins_with_the_canonical_root_even_through_a_symlink` doc comment), crates/terminus/tests/lifecycle.rs, crates/terminus/tests/retention.rs, crates/terminus/tests/cli.rs
- **Action:** Change what `path_key` returns. The key becomes the path below the project folder: `<uuid>.jsonl` for a session, and `<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar, joined with `/`. Recognition does not change. `below_projects` still reads the shape from the end of the path, never by stripping a root. It still uses `discover::is_uuid_transcript` and `discover::is_agent_transcript` and the two sidecar depths. It still requires a project folder between those components and a component named `projects`. Only the project folder's name is left out of the result. A last-component link still keys on its target from link metadata alone (D-03, D-02). A path outside the shapes still keys on its own canonical text. A stored absolute key from a root or project folder that no longer exists still reduces without the file being there, because `recover::stale_keys` and `recover::stale_parents` depend on that. `lineage::sidecar_parent` and the `parent_session_key` written in `prepare` need no change: the parent's path goes through the same function and becomes `<sessionId>.jsonl`.

  Do not teach `path_key` or the repair to reduce a key that is already relative, such as a D-01 key `-code-cadence/<uuid>.jsonl`. John ruled to leave them, and the reason is in Notes.

  Rewrite every comment that states the D-01 form so it states the new one. Each must say "below the project folder" or show `<uuid>.jsonl`, and none may say "below `projects`" or `<project dir>/...`. The sites are:
  - `path_key`'s and `below_projects`' doc comments. Renaming `below_projects` is the executor's choice.
  - `Pass::session_key`.
  - The held-key comment in `run_with` and the `claim` doc. Both name "two roots holding the same path below `projects`". Now two project folders, in one root or two, holding the same session file also share a key, and that is the risk John accepted.
  - The `ingest_locked` doc.
  - recover.rs: the module comment, the `Recovered::rekeyed` doc and the `rekey` doc. The module comment may add that a moved project folder now keeps the key too.
  - `lineage.rs`'s module comment, discover.rs's D-17 paragraph, verify.rs's module comment and `Failure::session_key`, config.rs's `Session::session_key`, and cmd/backfill.rs's `watermarks`.
  - The test comments: tests/pass.rs `key_of`, tests/lineage.rs `key`, tests/discover.rs on `every_path_begins_with_the_canonical_root_even_through_a_symlink`, the key helpers in crates/terminus/tests/lifecycle.rs and crates/terminus/tests/retention.rs, the comment above the `transcript_path` query in crates/terminus/tests/cli.rs, and the comments in tests/ingest.rs and tests/session_keys.rs.

  The summary line `Recovered::lines` writes says sessions were re-keyed "to their path below `projects`". Reword it to name the key the way the docs will. It stays one line however many sessions moved.

  Update the tests that pin the key. A prototype of this key measured that exactly three fail, and no others:
  - In tests/ingest.rs, `two_paths_to_one_transcript_produce_one_session` and `a_second_roots_file_never_appends_onto_the_first_roots_session` assert `-tmp-p/{name}`. They now expect `{name}`.
  - In tests/session_keys.rs, `Bench::plant` stages every session at `staging/projects/-stage-N/<U0>.jsonl`. Under the new key all of those share one key, so the second plant is refused by the held-key check in `ingest::run_with` (measured: "its session key is already held by .../-stage-0/..."). Stage each plant so it never shares a key with a session still under its staging key. The bench must still work at 742e352, because Task 2 copies this file there.
  - `one_pass_repairs_a_store_keyed_on_absolute_paths` expects the new keys: `U0.jsonl`, `U0/subagents/agent-a.jsonl`, `U3.jsonl`, `U4.jsonl`, `U5.jsonl` and `U6/subagents/agent-x.jsonl`. It also expects the reworded summary line.

  Never loosen an assertion to make it pass. Avoid reading file bytes for the key (D-02), keying on `sessionId` or `cwd`, touching discovery (D-04), and touching `ARCHIVE_FORMAT` or `DERIVED_SCHEMA` (D-06).

  Most of these files change only a comment, a key spelling in a test, or the test bench. Only ingest/mod.rs changes what the build does.
- **Verify:**
  - `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean. `cargo fmt --check` reports none of this task's files; it already reports four files outside this plan at its base (Phase 4 SUMMARY deviation).
  - `cargo test --workspace --all-features --locked --no-fail-fast` passes, run with the three `--skip` hook-budget tests `.github/workflows/test.yml` skips.
  - `grep -rn -e 'below \`projects\`' -e 'below the \`projects\`' -e 'below projects' -e 'project dir>/' crates` prints no line. The held-key comment in `run_with` breaks the phrase across two lines, so check each listed site by eye as well.
  - By hand, use an empty temp `TERMINUS_CONFIG_DIR`, a temp `TERMINUS_DATA_DIR`, and a temp `CLAUDE_CONFIG_DIR` holding `tests/fixtures/session-basic.jsonl` at `projects/-tmp-p/11111111-1111-4111-8111-111111111111.jsonl`. After `terminus ingest`, both `terminus sessions --json --project '*'` and `terminus search --json --project '*' brillig` show `session_key` `11111111-1111-4111-8111-111111111111.jsonl`.

### Task 2: Prove a moved project folder, and the copies one leaves in the archive, are one session

- **Files:** crates/terminus-core/tests/pass.rs, crates/terminus-core/tests/session_keys.rs
- **Action:** Add two regression tests on real temp SQLite, in the style of D-14.

  **The move.** Add a test to tests/pass.rs, beside `a_moved_claude_dir_behind_a_symlink_archives_nothing_twice` and built the same way:
  - Run a pass with `pass::run_with` and `Config::from_parts` over a tree holding a top-level transcript and its sidecar in one project folder.
  - Rename that project folder inside `projects`, leaving no symlink.
  - Run a second pass with the same config.
  - Assert the session count from `config::visible::counts` is unchanged, the set of `sessions.session_key` values is unchanged, and the second pass committed no file, added no turn and recorded no failure.

  **The archive.** Add a new test function to tests/session_keys.rs, separate from `one_pass_repairs_a_store_keyed_on_absolute_paths`, using the `Bench`. It plants what UAT item 5 found on the live store. Each transcript is archived under an absolute key in one project folder of an old `.claude` that no longer exists, and again under an absolute key in a second project folder of the tree the pass walks, where the file is on disk. In every pair the gone copy is planted first, so it has the lower `session_no`. That is the shape measured in both live pairs whose copies differ. The pairs are:
  - an identical pair of top-level transcripts;
  - a prefix pair, the gone copy shorter and the live file longer;
  - an identical pair of sidecars at the `subagents/workflows/wf_*/` depth, as in the UAT's doubled `be0a247d-.../subagents/workflows/wf_c61a181c-c13/` hits, whose stored `parent_session_key` is also absolute.

  Run one pass. Then assert all of the following:
  - Each transcript has exactly one session, under `ingest::path_key` of its live file.
  - Each holds its pair's lower `session_no`.
  - The prefix pair's kept session holds the live file's bytes byte for byte. Its watermark equals the live file's length, and its `transcript_path` names the live file.
  - The sidecar's `parent_session_key` is its parent's new key.
  - The run's `runs.error` holds only the re-key summary line: no "not merged" line and no failed file.
  - `terminus_core::verify::verify` reports nothing, and `PRAGMA foreign_key_check` returns no row.
  - A second pass commits no file, adds no turn and records no error.

  This pins "the one-old-copy-longer case against the live file" as the live store actually holds it. Notes say why the reverse shape is not tested.

  Both tests use only APIs that exist at 742e352 (`pass::run_with`, `ingest::run_with`, `ingest::path_key`, `Config::from_parts`, `config::visible::counts`, `verify::verify`, the `testkit` helpers there, and SQL), so they compile there. Add no other test.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test pass` and `cargo test -p terminus-core --all-features --test session_keys` pass.
  - In an export of 742e352 (`git archive 742e352`), with the two files copied in unchanged, both new tests compile and fail on the session count. The move test fails because the count doubles, and the archive test because each pair is two sessions. Report the counts each one printed there.

### Task 3: Record where an unchanged file was read from

- **Files:** crates/terminus-core/src/ingest/mod.rs (`Outcome::UpToDate`, `Existing`, `Existing::read`, `Prepared`, `prepare`, `apply`), crates/terminus-core/tests/retention.rs
- **Action:** Today `apply` returns `Outcome::UpToDate` before `write_session_meta`, the only writer of `session_meta.transcript_path`. So after a move with no symlink, every unchanged file's stored path names a path that is gone. Retention's delete rule asks `transcript_is_gone` of that path and deletes a session whose file is still on disk. The next pass then archives the file again from offset 0 under a new `session_no`.

  Make every pass that reaches an archived session and writes no turn leave `transcript_path` equal to the path it read, whenever the two differ. That covers three cases: no complete record past the watermark, an evicted session (`prepare`'s early return), and a cleared divergence flag with nothing new. The path is the one `Prepared` carries, which is the canonical file after `claim` resolved any link, written the way `write_session_meta` writes it.

  The write happens in `apply`, the writer half, because `prepare` runs on backfill's worker threads and touches no connection. It is an update of the session's existing `session_meta` row only. A file that was never archived writes nothing.

  The steady state must keep costing what it costs today: one `session_meta` read and no write. Read the stored path in the query `Existing::read` already makes, compare in memory, and write only when it differs. Do not add a second query per file, and do not run an unconditional `UPDATE` per file: a hook-triggered pass sees about 2,770 files.

  The outcome stays `Outcome::UpToDate`. A moved, unchanged file is not a committed file, so `files_committed` and `turns_added` stay 0, and `a_moved_claude_dir_behind_a_symlink_archives_nothing_twice` asserts exactly that. Rewrite the `Outcome::UpToDate` doc, which says no row is touched, to say what is now true.

  Add a regression test to the core tests/retention.rs, on real temp SQLite. It follows UAT item 10's probe:
  - Archive a transcript at `<temp>/a/projects/-tmp-p/<uuid>.jsonl` with a pass whose config has root `a` and no retention. The fixture's timestamps are old enough for the idle rule to finalize it.
  - Rename `a` to `b`, leaving no symlink.
  - Run a pass whose config comes from a `terminus.toml` with `roots` naming `b` and `[retention]` `action = "delete"`, `age_days = 30`.
  - Assert the session's rows in `sessions`, `session_meta`, `turns` and `observations` are all still there, with the same `session_no` and checksum. Assert its `transcript_path` names the file under `b`, and that the pass committed no file.
  - Run a third pass. Assert it adds no turn and commits no file, and changes no `session_meta` row.

  Use only APIs that exist at 742e352, so the test compiles there.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test retention` passes.
  - The full suite passes with the three hook-budget skips, and clippy is clean.
  - In an export of 742e352 with tests/retention.rs copied in unchanged, the new test compiles and fails because the session was deleted. Report what it printed.
  - By hand, repeat UAT item 10's probe with the release build of this branch. Run 1 archives one session. Then `mv a b` with no symlink, set `CLAUDE_CONFIG_DIR=b`, and set `[retention]` `action = "delete"`, `age_days = 30`. Run 2's `runs.notes` names no deletion, and `terminus sessions --json --project '*'` still lists the session. Run 3 adds 0 turns.

### Task 4: Describe the key without the project folder in the docs

- **Files:** THREAT-MODEL.md ("Transcripts" and "`terminus ingest <path>`" paragraphs, the "Session identity" bullet), docs/json-shapes.md (the `session_key` paragraph under "`data` by command", the `search` example, the `observations` example, the `retention` example and its `evict` and `delete` bullet, the `export` manifest bullet), docs/design/0003-ingest.md (the "Session key" row of the terms table), docs/design/0004-archive-and-block-rows.md (the "Session" term, the `sessions.session_key` row, the `session_meta.parent_session_key` row), CHANGELOG.md (the `[Unreleased]` `### Fixed` entry ending `(#57)`)
- **Action:** Read `~/.claude/john-voice.md` in full first, from the file, and follow its rules for project docs and for the changelog. Read Tasks 1 to 3's merged code before writing, and describe what it does.
  - **What the docs must say.** The session key is the transcript's path below its project folder: `<uuid>.jsonl` for a session, or `<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar. It is read off the shape of the path, which must sit in a project folder inside a `projects` folder, and never by stripping a root. So moving `.claude`, reaching it through a symlink, or moving a project folder keeps the key. A transcript that is a symlink takes the key of the file it points at. A path outside those shapes keeps its canonical path. The absolute path the file was last read from stays in `session_meta.transcript_path`. Keep every other claim these passages make that is still true.
  - **THREAT-MODEL.md "Session identity".** It is a security claim. Keep the forged-`sessionId` point (D-02). Change its example of two different files that reduce to one key to include the same session file in two project folders. A walk ingests the first and reports the other without opening it.
  - **docs/json-shapes.md.** Show keys in the new form, such as `5e0a....jsonl` and `1111....jsonl`, in place of `-abs-path-repo/...`, `-code-verbatim/...` and `-data-code-app/...`.
  - **Design doc 0004.** Its `parent_session_key` example becomes `<sessionId>.jsonl`.
  - **CHANGELOG.md, the #57 entry only.** Replace the D-01 key and its `-code-app/<uuid>.jsonl` example with the new key. Say that moving a project folder keeps the key too. Say that the repair also merges the copies a moved project folder left behind. Add one sentence for Task 3, John's ruling: after `.claude` moves with no symlink left behind, retention's delete rule no longer removes a session whose transcript is still on disk. Keep the rest of the entry: the `session_key` output change, the script that opens it breaking, and no command named (D-08, D-13).
  - **Avoid.** Inventing a command, option or field. Touching any passage these bullets do not name. Adding anything for Task 3 outside the one changelog sentence: no doc says an unchanged file's path goes unrecorded, and 0004 already says `transcript_path` is the path the file was last read from.
  - Open each changed file in RustRover once it is written.
- **Verify:**
  - `grep -n -e 'below \`projects\`' -e 'below the \`projects\`' -e 'project dir>/' -e '-code-app/' -e '-code-verbatim/' -e '-data-code-app/' -e '-abs-path-repo/' THREAT-MODEL.md docs/json-shapes.md docs/design/0003-ingest.md docs/design/0004-archive-and-block-rows.md CHANGELOG.md` prints no line that describes `session_key`.
  - THREAT-MODEL.md's "Session identity" bullet still states the forged-`sessionId` property and names two project folders as a case of two files reducing to one key.
  - `git diff CHANGELOG.md` touches only the #57 entry.
  - (human-verify: John reads each changed doc in RustRover) The wording passes his review.

## Notes

- **Override of D-01: John's ruling, made after the UAT and locked.** The key drops the project folder. It becomes `<uuid>.jsonl` for a session, and `<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar. A file under no `projects` folder still keeps its canonical absolute path as its key. With that key, moving a project folder keeps the key the way a root move already does. The existing repair then pairs the 175 copies, because their old keys now reduce to one key. D-10's prefix test and D-11's survivor rule still govern the merge, and a pair that fails the prefix test is still reported with both sessions kept. Facts behind it: project identity comes from the record's `cwd` (`project.rs`), not from the folder name. The encoded folder name is read only by the exclusion test, which reads the path on disk in `discover.rs`, not the key. `recover::stale_keys` scans only keys that are still absolute paths. The risk John accepted: two different files with the same session id in two project folders now share a key, and the second is refused unread under the rule UAT item 9 verified. D-01 measured zero such duplicates across 2,733 files on disk.
- **Measured while planning, 2026-10-18, with a prototype.** The prototype was HEAD with only the key change (`below_projects` returning the components below the project folder).
  - The full suite failed exactly three tests, in tests/ingest.rs (2) and tests/session_keys.rs (1).
  - On a fresh `.backup` copy of the live store (4,960 sessions, all absolute keys), one `terminus ingest` from its release build exited 0 in 107 s. It re-keyed 2,757 sessions and merged 2,203 twins. It reported no unmerged twin and no failed file, and left 2,762 sessions with no absolute key.
  - All 175 groups spanning `-code-cadence` and `-code-cadence-v3-9-1` merged. Two of them have copies that differ, and in both the copy archived first is gone and is a strict prefix of the live file. In `be0a247d-...` the `-code-cadence` copy (session 4321, 2,117,725 bytes) is gone and the `-code-cadence-v3-9-1` copy (4751, 3,324,260 bytes) is on disk. In `1699878d-...` it is the other way round: the `-code-cadence-v3-9-1` copy (4822, 1,132,204 bytes) is gone and the `-code-cadence` copy (4827, 1,500,390 bytes) is on disk. This is UAT's "old copy longer" pair, and its longer copy is the live file. Each kept session holds the live file's length, with the watermark at that length, `transcript_path` naming the live file, and no divergence flag.
  - `terminus verify` checked 2,762 sessions with 0 failed. `388a000f-7ac1-4333-9880-96d8ab237e51.jsonl` is still session 2000 with 857,565 bytes. 0 labels and decision turn ids dangle. A second pass committed only 4 live files that had grown, and its `runs.error` was null.
- **Turns that still repeat across sessions.** After that pass, 9,022 turn uuids are held by more than one session. None is a transcript archived twice. No two sessions share a checksum, and every uuid is shared between different files:
  - 15 pairs where one session continues from the other, 6,843 uuids;
  - 6 pairs that continue from the same session, 1,798 uuids;
  - 5 pairs of sidecars of one session, 381 uuids.

  `terminus search --json --project '*'` returned 10 distinct turns of 10 hits for `symlink` and for `watermark`. For `retention` it returned 9 of 10, and the repeat is one record in `efced699-...` and in `6825b00d-...`, which continues from it. Claude Code writes those records into the resumed file itself. John ruled that AC4's "each turn once" reads as each transcript once: no transcript is archived under two keys. A record Claude Code copied into a resumed file may still show twice.
- **D-01-form relative keys are not re-keyed (John's ruling).** This branch is unreleased. The live store, the only real one, holds 0 relative keys (measured: 4,960 of 4,960 absolute), and its hook binary is 0.1.1. A D-01 key exists only in test stores and in scratch copies a build of this branch repaired. Ingesting such a store with this plan's build leaves each D-01 key in place and archives its file again under the new key. That is why the AC4 re-run must use a fresh copy.
- **Not planned: an archive-only copy longer than its live file.** D-11 would merge such a pair into the longer stream, so no byte is lost. The next pass would then find the live file shorter than the watermark and flag it diverged, and every later pass would name it in `runs.error`. The live store has no such pair (measured above), so no test pins it.
- **Task 3 and the merge's `transcript_path`.** The merge gives the survivor the newest absorbed member's `transcript_path`. After Task 3, the same pass's walk rewrites it to the file the pass read, before retention runs. CONTEXT D-13 says ingest "already overwrites [transcript_path] on every pass"; UAT item 10 showed that was false for an unchanged file, and Task 3 makes it true.
- **AC4 steps (human-verify).**
  1. Make a fresh copy with `sqlite3 <live data dir>/terminus.db ".backup '<copy dir>/terminus.db'"`. Do not reuse a copy a build of this branch already repaired. Copy any decision files waiting in the live data directory beside it.
  2. Before the pass, record on the copy the dangling-reference counts PLAN-2's Notes list. Also record `SELECT count(*) FROM (SELECT uuid FROM turns WHERE uuid IS NOT NULL GROUP BY uuid HAVING count(DISTINCT session_key) > 1)`.
  3. Run `TERMINUS_DATA_DIR=<copy dir> TERMINUS_CONFIG_DIR=<empty dir> terminus ingest` from this branch's release build.
  4. Check each of these:
     - the run's `runs.error` is the one re-key summary line, with no "not merged" line and no failed file;
     - no session key begins with `/` or `-`;
     - `SELECT session_no, uncompressed_len FROM sessions JOIN session_meta USING (session_key) WHERE session_key = '388a000f-7ac1-4333-9880-96d8ab237e51.jsonl'` returns session 2000 with 857,565 bytes;
     - `terminus verify` is clean;
     - no dangling count is higher than before;
     - each turn uuid still held by two sessions is held by two different transcripts (a resume or a sidecar pair, as above), never by one transcript under two keys.

     The figures above will have drifted with the live tree.
- **Plan shape.** This is one gap plan of four tasks, run in order. Tasks 1 and 3 share ingest/mod.rs, Task 2 tests Task 1's key, and Task 4 describes all three. CONTEXT's order is kept: the key first, then the repair's proof, then the docs. Task 3 is the separate Gap B.
- **Recalled.** UAT items 5 and 10 (phases/4/UAT.md, phase 4) are the gaps closed here. D-01 (phases/4/CONTEXT.md, phase 4) is overridden by the ruling above. D-11 (phases/4/CONTEXT.md, phase 4) still decides the survivor and its `transcript_path`.
