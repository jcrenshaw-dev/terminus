---
phase: 4
plan: 2
requirements: [ING-13]
files:
  - crates/terminus-core/src/recover.rs
  - crates/terminus-core/src/reindex.rs
  - crates/terminus-core/src/retention/apply.rs
  - crates/terminus-core/src/testkit.rs
  - crates/terminus-core/Cargo.toml
  - crates/terminus-core/tests/session_keys.rs
---

# Phase 4: Session keys - Plan 2 of 3: the repair

## Goal

An archive that already holds sessions under both an old and a new absolute path is repaired in place, inside recovery, at the top of every ingest run. Each pair of twins that is provably one stream is merged into the older session, every other session is re-keyed to the path below `projects`, and no session that exists only in the archive is lost.

## Must be true when done

- On a planted store holding an identical twin pair, a prefix twin pair, a link pair and an archive-only orphan, one ingest pass leaves one session per transcript. The kept session in the prefix pair holds the longer stream byte for byte, the orphan's bytes are unchanged, and `terminus verify` passes (AC3).
- A planted pair whose block hashes are not a prefix of each other is named in that run's `runs.error`, and both of its sessions are still there with their bytes (AC3). So is a member that extends the kept session differently from a member merged into it before.
- After the repair, no label or decision turn id points at a turn that does not exist. An id that pointed into a merged twin now names the same `turn_seq` in the kept session.
- A second run after the repair changes no row and reports nothing. `ARCHIVE_FORMAT` is still 2, and there is no new command and no switch (D-06, D-07, D-08).
- The planted-store regression test fails on the commit before the fix and passes after it (AC6).
- (human-verify: a copy of the live store and the release binary built from this branch) On a copy of the live store, after one ingest pass, `recall_search` returns each turn once, `terminus status` counts each session once, `-code-cadence/388a000f-...jsonl` is still archived, and no label or decision turn id points at a missing turn (AC4). The steps are in Notes.

## Context

- Depends on Plan 1: the repair reduces each stored key with Plan 1's public key function, applied to the stored key as a path. That function resolves a stored path that is a symlink today, which is how D-03's link pairs are found.
- Locked and implemented here: D-06 (format stays 2), D-07 (no switch), D-08 (in recovery, under the ingest lock, one transaction, module comment rewritten), D-09 (after `Store::open`, before the derived rebuild), D-10 (pairing rule), D-11 (survivor), D-12 (unpaired sessions re-keyed only), and D-14's second regression test.
- Facts read in the code. `blocks` is keyed on `session_no`, and a turn id is `session_no << 24 | turn_seq` (`schema::turn_id`), so re-keying moves no byte and changes no turn id. `session_meta`, `turns`, `compactions` and `observations` reference `sessions(session_key)` with no `ON UPDATE CASCADE`, and `watermarks.transcript_path` holds the key unenforced.
- Out of scope: a `repair` command (D-08), a format bump (D-06), a switch (D-07), docs and the changelog (Plan 3).

## Tasks

### Task 1: Re-key every unpaired session inside recovery

- **Files:** crates/terminus-core/src/recover.rs (`recover`, module comment, `Recovered`), crates/terminus-core/src/reindex.rs (`open_up_to_date`), crates/terminus-core/src/testkit.rs, crates/terminus-core/Cargo.toml, crates/terminus-core/tests/session_keys.rs (new)
- **Action:** `recover::recover` opens the store with `Store::open`, which brings the columns forward. It then runs the repair, then brings the derived tables forward exactly as `reindex::open_up_to_date` does today, then runs `sweep_watermarks` (D-09). Running the repair before the rebuild means the twins about to be deleted are never rebuilt; the live store reads `derived_schema` 4 against 5. You may split `open_up_to_date` into an open half and a bring-forward half, but `terminus reindex` keeps its behaviour and does not run the repair. The repair belongs to recovery alone (D-08).

  The repair is one transaction, under the ingest lock every caller of `recover` already holds. Defer foreign key checks for that transaction with `PRAGMA defer_foreign_keys`, so a parent key and its children can be updated in any order and are checked at commit. That resolves CONTEXT's first flagged assumption. If the repair fails, its error stops the run, as recovery's other errors do. Ingesting under new keys over an unrepaired store would archive every session a third time.

  In this task the repair re-keys unpaired sessions only. A stored key needs work when it is an absolute path and Plan 1's key function, applied to it, gives a different key. A session is unpaired (D-12) when no other stored key reduces to the same new key and no session already holds it. For each one, move the key in `sessions`, `session_meta.session_key`, `turns`, `compactions`, `observations` and `watermarks.transcript_path`. A watermark row already stored under the new key, which is what a zero-offset row for a not-yet-complete file looks like, is replaced by the session's own. Then replace every `session_meta.parent_session_key` that is an absolute path with its reduction.

  Never touch block rows, `session_no`, turn ids, `checksum`, `uncompressed_len` or `transcript_path` here, whether or not the file still exists (D-12). Leave every group of two or more sessions that reduce to one key exactly as it is. Tasks 2 and 3 handle them. `ARCHIVE_FORMAT` stays 2 (D-06), with no switch (D-07) and no command (D-08).

  The steady state must cost what a healthy recovery costs today. Read only the stored keys that are still absolute paths, write nothing and add no line when none of them reduces to a different key. `Recovered` carries what the repair did, `repaired()` counts it, and `lines()` adds one summary line giving how many sessions were re-keyed, not one line per session. The line format is the planner's choice, made because the first run on the live store re-keys about 2,900 sessions and `runs.error` should stay readable.

  Rewrite the `recover.rs` module comment (D-08). It must describe the third job, and replace "`sessions` and `session_meta` are not touched" with what is now true: no archived byte is lost; a session is re-keyed in place; a twin is deleted only after the survivor holds every byte it held (Task 3).

  Start D-14's planted-store regression test in the new `tests/session_keys.rs`. Declare it as a `[[test]]` in `crates/terminus-core/Cargo.toml` with `required-features = ["testkit"]`, as every testkit test there is (#44). In this task it plants an archive-only orphan under an absolute key `<root that no longer exists>/projects/-p/<uuid>.jsonl`, and a sidecar whose stored key and `parent_session_key` are absolute. One way to plant a session under an old key is to ingest it and then rewrite its keys to an absolute spelling with foreign keys deferred. Put that helper in testkit.rs if Tasks 2 to 4 need it too.

  This is one test for the whole plan. Each later task adds its planted case to the same store and its assertions to the same test.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test session_keys` passes. After one `pass::run_with`, the orphan's key is `-p/<uuid>.jsonl`, and its block rows, `checksum`, `uncompressed_len`, `session_no` and turn ids are byte-identical to before. Its watermark sits under the new key. The sidecar's `parent_session_key` is reduced. `terminus_core::verify::verify` reports nothing, and `PRAGMA foreign_key_check` returns no row.
  - A second `pass::run_with` changes no row in any table and its recovery report is empty.
  - The full suite passes with the three hook-budget skips, along with `cargo clippy --workspace --all-features --all-targets -- -D warnings` and `cargo fmt --check`.

### Task 2: Pair the sessions that reduce to one key, and report the pairs that are not one stream

- **Files:** crates/terminus-core/src/recover.rs, crates/terminus-core/src/testkit.rs, crates/terminus-core/tests/session_keys.rs
- **Action:** A group is every session whose stored key reduces to one new key, including a session already stored under that key. A group of two or more is twins (D-10). The survivor is the member with the lowest `session_no` (D-11). Test the other members one at a time, in ascending `session_no`, each against the group's current stream. That is the survivor's stream to start with, and after that the stream of the last member that passed and was longer. A member that passes and is longer becomes the current stream. A member that fails leaves it unchanged. Testing every member against the survivor's original stream alone is not enough: two members can both extend the survivor and differ from each other after its end, and merging both would lose one of them. The test passes only when one stream's block hashes are a prefix of the other's (D-10), checked as follows:
  1. Neither member is evicted, both have the same non-null `block_size`, and both have block rows numbered from 0 with no gap.
  2. Equal lengths pass on equal checksums.
  3. Unequal lengths pass when every full block of the shorter stream has the same stored hash as the longer stream's block at that number, and the shorter stream's partial last block, if it has one, decompresses to bytes that are a prefix of the longer stream's block at that number.
  4. Two evicted members pass only on equal checksum, `uncompressed_len` and `block_size`.
  5. Anything else fails. That includes exactly one evicted member, a block that does not decompress, and different block sizes.

  The partial-block comparison is required, not optional. D-10's two strict prefixes differ only in that tail block, and a pair that diverges inside it would otherwise merge and lose the shorter stream's diverging bytes, which is the loss ING-13 forbids. Read only the hash column plus at most the two blocks that partial comparison needs. Never decompress whole streams.

  An evicted member has no block rows, and an empty hash list is trivially a prefix of anything. Without rule 4, a merge would un-evict the survivor, or delete the only copy of a member's blocks.

  A member that fails is reported and left exactly as it is: its bytes and its stored key are untouched, and it is not deleted (D-10). The members of its group that passed still merge in Task 3. The report is one `Recovered` line per failing member, naming its stored key, the survivor's stored key, and the key both reduce to. In this task nothing in a group of two or more is written. Merging the members that pass is Task 3.

  Add D-14's non-prefix pair to the planted store: two sessions under absolute keys from two different roots that reduce to one key, whose full block hashes differ. Also add a pair that differs only inside the shorter stream's partial last block. Also add a three-member group: a survivor and two longer members that both extend it and differ from each other after its end. The first longer member passes, and the second fails against it.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test session_keys` passes. After one pass, both members of each planted failing pair are still there, with their stored keys, `session_no`, block rows and checksums unchanged. The pass's `runs.error` names both stored keys of each failing pair, and `verify` reports nothing.
  - In the three-member group, `runs.error` names the second longer member and not the first.
  - The full suite and clippy pass.

### Task 3: Merge each proven twin into the older session

- **Files:** crates/terminus-core/src/recover.rs, crates/terminus-core/src/retention/apply.rs (`delete_one`), crates/terminus-core/src/testkit.rs, crates/terminus-core/tests/session_keys.rs
- **Action:** Within the repair's one transaction, merge each member that passed the Task 2 test into the survivor, in the same ascending `session_no` order Task 2 tested them in, so each merge applies to the stream that member was tested against:
  - **When the member's stream is longer, the survivor takes it (D-11).** The survivor's block rows become the member's, moved and not recompressed. The stream-describing columns come from the member: `checksum`, `uncompressed_len`, `block_size`, `capture_mode` and `last_turn_at`. The survivor's turn rows beyond its old turn count are then derived from the added bytes through the seam `ingest::apply` uses: `parse::scan_from` from the survivor's old length and turn count, then `derive::derive_turn` and `derive::derive_boundary`. The survivor's turn ids for every `turn_seq` it already had stay the same. Its watermark becomes the member's, because the watermark has to match the stream it now holds. With its own lower watermark, the next pass would append bytes the stream already has.
  - **Every other `session_meta` column keeps the survivor's value, and takes the member's where the survivor's is null (D-11).** On the live store, two newer twins hold the only `continues_from`.
  - **`transcript_path` takes the newest absorbed member's value.** That is the spelling the most recent pass read. Retention's `transcript_is_gone` decides "gone" from this column, and an old root's path would read as gone after a move made without a symlink. Retention's `delete_one` does not check again before deleting, and an `observations` row it deletes cannot be recovered (recalled: v0.1.0/phases/8/SUMMARY.md, phase 8).
  - **Then the member is deleted whole, in `retention::apply::delete_one`'s order:** paths, entities, compactions, turns_fts, turns, observations, session_meta, blocks, sessions, watermarks. Factor that statement sequence out of `delete_one` into a helper both callers use, so the order has one definition. `delete_one` keeps its own transaction and its return value.

  **The new key** (the planner's composition of D-10, D-11 and D-12): after merging, if a member that failed the test already holds the new key, it keeps it. Otherwise the survivor is re-keyed as in Task 1 when it absorbed at least one member. When it absorbed none, no member is re-keyed and the group is reported again on every run. Handing the key to a stream that may not be the file's prefix could append one file's bytes onto another's stream, because ingest does not compare the file to the archived prefix in the steady state. Leaving the key free makes the next pass archive the file fresh under it, and that copy merges into the survivor on the following run when the survivor's stream is its prefix.

  The summary line from Task 1 also counts the members merged.

  Complete D-14's planted store with:
  - an identical twin pair;
  - a prefix pair, with the newer twin longer, as on the live store;
  - a link pair: a sidecar link in one session's `subagents/` folder pointing at another session's sidecar, each archived under an absolute key, with the link left on disk so the repair resolves it (D-03);
  - one twin with `continues_from` where the other is null.

  The test then holds D-14's five cases.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test session_keys` passes. After one pass, each planted transcript has exactly one session, and that session holds its group's lowest `session_no`. The prefix pair's survivor stream equals the longer twin's original stream byte for byte, and it holds the longer twin's watermark. The link pair is one session under the target's key. In the three-member group, the survivor holds the first longer member's stream byte for byte, and the second longer member is still there with its bytes and is named in `runs.error`. The `continues_from` value was carried over. The orphan is unchanged, the non-prefix pair is still reported and kept, `verify` reports nothing, and `PRAGMA foreign_key_check` returns no row.
  - `testkit::query_set_json` is the same before and after a `reindex::reindex` of that store, so the derived rows match what a rebuild writes. A further pass over the unchanged files adds no turn and no byte, and its recovery report is empty.
  - Copied with its testkit helpers into a worktree at 2b39948, the test compiles and fails on the session count (AC6).
  - `cargo test -p terminus-core --all-features --test retention` still passes after the `delete_one` refactor.

### Task 4: Point labels and decisions at the surviving session

- **Files:** crates/terminus-core/src/recover.rs, crates/terminus-core/src/testkit.rs, crates/terminus-core/tests/session_keys.rs
- **Action:** Before a member is deleted in Task 3's merge, rewrite every reference into it, in the same transaction:
  - `labels.turn_id`;
  - the `turn_id` of each entry in `decisions.candidates`, `decisions.injected` and `decisions.suppressed`;
  - each id in `decisions.dropped`.

  A reference is rewritten when its `session_no` half, from `schema::split_turn_id`, is the member's. Rewrite it to `schema::turn_id` of the survivor's `session_no` and the same `turn_seq`. That is exact because the survivor holds every `turn_seq` the member held (D-11). Ids into the survivor, and ids into a member that failed the test, stay as they are.

  Decision files still waiting on disk are references too. In a pass, `feedback::drain` runs after `recover` (`ingest/pass.rs:262-271`), so without a change the files waiting at the first upgraded run would be moved into the store after their twins were merged away. So when the repair has work to do, and only then, `recover` calls `feedback::drain` after `Store::open` and before the repair, and this task rewrites the rows it moved in along with every other decision. The pass's own drain then finds nothing to move. When the repair has no work, recovery does not drain, so the steady state is unchanged, and `terminus ingest <path>` and `terminus backfill` still leave decision files to the next pass. A drain that fails here is reported in a `Recovered` line and the repair continues, as the pass treats a failed drain. John chose this on 2026-10-18.

  D-11 names labels, injected and candidates. `suppressed` and `dropped` hold turn ids too, and AC4 requires that no decision turn id point at a missing turn, so they are rewritten as well. Keep every other field of each JSON document semantically unchanged. A document that does not parse is left as it is and named in a `Recovered` line. `decisions.watermark_session_no` is not touched: the survivor has the lower number, which is the reason D-11 keeps it.

  In the planted store, add a decision whose candidates, injected, suppressed and dropped lists name turns in both twins of the prefix pair, plus a label on a turn in each twin. Also leave a decision file waiting in the data directory, written the way the prompt hook writes one, whose candidates name a turn in the newer twin.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test session_keys` passes. After one pass, every `labels.turn_id` and every turn id in the four decision lists names an existing `turns.id`. Ids that named the newer twin now carry the survivor's `session_no` with the same `turn_seq`, ids that named the survivor are unchanged, and the other JSON fields are equal to before. The waiting decision file is now a row, its candidate names the survivor's `session_no`, and the file is gone from the data directory.
  - On a store with no old keys, recovery moves no decision file: a file left waiting before `recover` is still on disk after it.
  - The full suite and clippy pass.

## Notes

- **AC4 steps (human-verify).**
  1. Copy the live `terminus.db` with `sqlite3 <live data dir>/terminus.db ".backup '<copy dir>/terminus.db'"`; `.backup` is consistent while a hook-triggered pass writes. Then copy the decision files waiting in the live data directory beside it, so the check covers them.
  2. Before the pass, run on the copy:
     - `terminus search --json --project '*'` for three words that show doubled hits today;
     - the count of `labels.turn_id` not in `turns.id`;
     - for `candidates`, `injected` and `suppressed`, the count of `json_extract(value, '$.turn_id')` from `json_each(decisions.<column>)` not in `turns.id`;
     - for `dropped`, the count of `value` not in `turns.id`.
  3. Run `TERMINUS_DATA_DIR=<copy dir> terminus ingest` from this branch's release build. It reads the real transcripts and writes only the copy. The first pass also rebuilds every derived table, because the live store is at `derived_schema` 4.
  4. Check:
     - each of the three searches returns each turn once;
     - `terminus status --json` `sessions` equals the files discovery yields, with each sidecar link counted once, plus the archive-only sessions (176 at the 2026-10-19 measurement);
     - `SELECT session_key FROM sessions WHERE session_key LIKE '-code-cadence/388a000f%'` returns one row;
     - `terminus verify` is clean;
     - no reference count from step 2 is higher than before.
- **Pending decision files.** Task 4 drains them inside recovery before the repair, only when the repair has work, so their turn ids are rewritten with the rest. A `session_no` freed by a deleted twin can still be reused when that twin held the highest number (`ingest/mod.rs:1118-1120` allocates the highest plus one). After the repair no turn id names it, and `decisions.watermark_session_no` is left as D-11 says.
- **Edge not planned: a link sidecar under an old root that is gone.** Such a link session cannot be resolved, so it reduces by shape to its own folder's key and stays a separate session. D-03's 10 links are all archived under new-key numbers (2187 and 2562), so the live store does not have this case.
- **Measured while planning, 2026-10-18.** With SQLite 3.54.0, updating `sessions.session_key` with child rows fails at once with foreign keys on, commits once `PRAGMA defer_foreign_keys = ON` is set inside the transaction, and still fails at commit when a child is left dangling. The bundled build is proved by Task 1's `foreign_key_check`.
