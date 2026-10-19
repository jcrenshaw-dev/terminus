---
phase: 4
plan: 1
requirements: [ING-13]
files:
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/backfill.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/lineage.rs
  - crates/terminus-core/src/discover.rs
  - crates/terminus-core/src/inject/prompt.rs
  - crates/terminus-core/src/verify.rs
  - crates/terminus/src/cmd/backfill.rs
  - crates/terminus-core/tests/ingest.rs
  - crates/terminus-core/tests/discover.rs
  - crates/terminus-core/tests/backfill.rs
  - crates/terminus-core/tests/compaction.rs
  - crates/terminus-core/tests/exclusion.rs
  - crates/terminus-core/tests/lineage.rs
  - crates/terminus-core/tests/pass.rs
  - crates/terminus-core/tests/project.rs
  - crates/terminus-core/tests/recover.rs
  - crates/terminus-core/tests/retention.rs
  - crates/terminus/tests/cli.rs
  - crates/terminus/tests/lifecycle.rs
  - crates/terminus/tests/retention.rs
  - crates/terminus/tests/status.rs
---

# Phase 4: Session keys - Plan 1 of 3: the key

## Goal

A session's key is its transcript's path below the `projects` folder, computed from the path alone at every place that turns a path into a key. Moving `.claude`, or reaching it through a symlink, no longer archives any session a second time, and a sidecar that is a symlink is archived once, under the file it points at.

## Must be true when done

- After a pass over a transcript tree, renaming the tree, leaving a symlink at the old path and running ingest again, `terminus status` shows the same session count as before the move (AC1).
- A sidecar that is a symlink to another session's sidecar under `projects` is archived once, under the key of the file it points at, by the tree pass and by `terminus backfill` alike (AC2).
- `session_key` in `--json` output (`sessions`, `search`, `verify`, `observations`, the `export` manifest) and in `recall_search` and `recall_get` results is the path below `projects`, such as `-tmp-p/11111111-1111-4111-8111-111111111111.jsonl`. `session_meta.transcript_path` still holds the absolute path (AC5, D-13).
- A sidecar's `parent_session_key` equals the key its parent session is archived under, whichever of the two was ingested first.
- The hook still recognizes the live session from the payload's `transcript_path`, so `visible_in_session` suppression and the compaction recall keep working.
- The move regression test fails on the commit before this plan and passes after it (AC6).

## Context

- Locked and implemented here: D-01 (key form), D-02 (from the path only, never record content), D-03 (a link takes its target's key), D-04 (discovery still canonicalizes the root once and yields absolute paths), D-05 (every key site in one change), D-13 (output carries the new key), and D-14's first regression test plus its change to `tests/ingest.rs`.
- Plan 2 repairs stores that already hold old keys, by calling the key function this plan writes on each stored key. Plan 3 writes the docs and the changelog. The three run in order: 1, then 2, then 3. Until Plan 2 lands, this build run over a 0.1.x store would archive every session under a third key. Nothing ships between the plans.
- Out of scope: the repair, prose docs, `CHANGELOG.md`, keying on `sessionId` (D-02), and any change to how discovery resolves the root (D-04).
- Measured while planning, 2026-10-18. With a prototype of this key, 49 existing tests fail, all in the 12 test files leased here besides `ingest.rs` and `discover.rs`. Every failure comes from the test spelling the expected key as the absolute path. No other test file fails.

## Tasks

### Task 1: Key every session on the path below `projects`

- **Files:** crates/terminus-core/src/ingest/mod.rs (`path_key`, `prepare`, `ingest_locked`, `Pass`), crates/terminus-core/src/ingest/backfill.rs (`read_existing`), crates/terminus-core/src/lineage.rs (`sidecar_parent`, module comment), crates/terminus-core/src/inject/prompt.rs (`current_session`), crates/terminus-core/src/discover.rs (`is_uuid_transcript`, `is_agent_transcript`, module comment, `Discovered`), crates/terminus-core/src/verify.rs (module comment, `Failure`), crates/terminus/src/cmd/backfill.rs (`estimate`, `watermarks`), crates/terminus-core/tests/ingest.rs (`two_paths_to_one_transcript_produce_one_session`), crates/terminus-core/tests/discover.rs (`every_path_begins_with_the_canonical_root_even_through_a_symlink`), crates/terminus-core/tests/backfill.rs, crates/terminus-core/tests/compaction.rs, crates/terminus-core/tests/exclusion.rs, crates/terminus-core/tests/lineage.rs, crates/terminus-core/tests/pass.rs, crates/terminus-core/tests/project.rs, crates/terminus-core/tests/recover.rs, crates/terminus-core/tests/retention.rs, crates/terminus/tests/cli.rs, crates/terminus/tests/lifecycle.rs, crates/terminus/tests/retention.rs, crates/terminus/tests/status.rs
- **Action:** Change what `path_key` computes. Keep it as the one function every key site already calls (`ingest_locked`, `flag_divergence`, backfill's `read_existing`, `current_session`), so they all switch in one change (D-05). The key it returns is D-01's, built in three steps:
  1. If the path's last component is a symbolic link, resolve it and key the target (D-03). Test this with link metadata only. Never open the file or read a byte of it (D-02).
  2. Recognize the shape from the end of the path, never by stripping a configured root, so a stored key from a root that no longer exists still reduces (D-01). A file name discovery accepts as `<uuid>.jsonl`, whose grandparent directory is named `projects`, gives `<project dir>/<uuid>.jsonl`. A file name discovery accepts as `agent-*.jsonl`, whose ancestors read `projects/<project dir>/<sessionId>/subagents/` or `projects/<project dir>/<sessionId>/subagents/workflows/wf_*/`, gives the path below `projects`. Join the components with `/`. Use discovery's own name tests, `is_uuid_transcript` and `is_agent_transcript` in discover.rs, widened to crate visibility, so the key recognizes exactly the names the walk yields.
  3. Any other path is returned as its own text. Every caller hands in a canonical path: `run_with` canonicalizes it, and the walk joins onto the canonical root. So the result is D-01's "canonical absolute path" for a file under no `projects` folder. A path that does not exist, such as an old root's stored key, gets the shape test alone and is never an error. Plan 2 calls this function on every stored key, so it must accept any absolute path and must not need the file to be there.

  Keep the UTF-8 refusal as it is. Make the function public from terminus-core, because `cmd/backfill.rs` in the `terminus` crate needs it.

  `parent_session_key`, set in `prepare`, becomes the key this same function gives the parent's path from `lineage::sidecar_parent`. Take that parent path from the same resolved path the session's own key came from. Then a sidecar's parent key always equals the key its parent is archived under, ingested first or not. Rewrite the `sidecar_parent` doc comment, which calls its return value a session key, and the lineage module comment that says `sessions` is keyed on "the canonical transcript path".

  `current_session` keeps its canonicalize and goes through the same function. Rewrite its doc comment to say so. D-05's warning is the constraint: if this ever keyed differently from ingest, the compaction lookup would find no boundary and the live session's own turns would be injected back into it.

  `cmd/backfill.rs` `estimate` looks each discovered path's watermark up by this key, not by `path.to_str()`. D-05's list did not name this site, but it is one: left alone, `terminus install` and `terminus backfill` would report every archived byte as unread. Rewrite the `watermarks` doc comment, which says the key is the canonical path discovery builds.

  Rewrite the comments in this task's code that state the key is the canonical path: `Pass::session_key`, `path_key`'s own doc comment, and the comment inside `ingest_locked`. In discover.rs, the D-17 paragraph of the module comment and the `Discovered::transcripts` doc claim the walked paths equal the session keys. Correct only that claim. Discovery itself does not change (D-04). Do the same for verify.rs's "Which identifier names a failure" section and the `Failure::session_key` doc, and the doc comment on `every_path_begins_with_the_canonical_root_even_through_a_symlink` in tests/discover.rs.

  Update the failing tests in the 12 test files above so they expect the new key. Get it from the public key function or spell it as `<project dir>/<uuid>.jsonl`. Never loosen an assertion to make it pass. In tests/ingest.rs, change `two_paths_to_one_transcript_produce_one_session` as D-14 says: put the transcript inside a `projects/<dir>/` tree, so its expected key is the path below `projects` and not the absolute path.

  Avoid all of the following. Reading transcript bytes to compute a key (D-02). Keying on `sessionId` or `cwd`. Changing `discover_root`'s canonicalization or what it yields (D-04). Touching `ARCHIVE_FORMAT` or `DERIVED_SCHEMA` (D-06). A switch (D-07).
- **Verify:**
  - `cargo fmt --check` is clean.
  - `cargo clippy --workspace --all-features --all-targets -- -D warnings` is clean.
  - `cargo test --workspace --all-features --locked --no-fail-fast`, run with the three `--skip` hook-budget tests `.github/workflows/test.yml` skips, passes.
  - `cargo test -p terminus --all-features --test suppression` passes. That test ingests under a `projects` tree, so it covers `current_session` and the compaction lookup on the new key.
  - The lineage tests `both_sidecar_depths_carry_their_parents_session_key` and `a_sidecar_ingested_before_its_parent_records_the_same_key` pass, with the parent key in the new form.
  - By hand: with `TERMINUS_CONFIG_DIR` set to an empty temp directory (so no real `terminus.toml` `roots` applies), `TERMINUS_DATA_DIR` set to a temp directory, and `CLAUDE_CONFIG_DIR` set to a temp `.claude` holding `tests/fixtures/session-basic.jsonl` copied to `projects/-tmp-p/11111111-1111-4111-8111-111111111111.jsonl`, run `terminus ingest`. Then `terminus sessions --json --project '*'` and `terminus search --json --project '*' brillig` both show `session_key` `-tmp-p/11111111-1111-4111-8111-111111111111.jsonl`. A `recall_search` for `brillig` through `terminus mcp`, sent as `initialize`, `notifications/initialized` and then `tools/call` on stdin, shows the same key.

### Task 2: Ingest each key once per walk

- **Files:** crates/terminus-core/src/ingest/pass.rs (`walk`), crates/terminus-core/src/ingest/backfill.rs (`dispatch_round`, `commit_one`, `read_existing`), crates/terminus-core/src/ingest/mod.rs (`ingest_locked`)
- **Action:** After Task 1, two discovered paths can share one key. That happens with a sidecar link and the file it points at, of which there are 10 on the live store (D-03). It also happens when two configured roots hold the same path below `projects`; CONTEXT's second flagged assumption left that case to the planner. Backfill reads every `Existing` in a round before applying any of them. The comment in `dispatch_round` justifies that with "Two files never share a session key", which is now false. Two paths with one key in one round would both prepare against the same state, and the second apply would fail or rewrite what the first wrote.

  In both walks, `pass::walk` and backfill's rounds, resolve a link to its target once. Use the target for everything after that: the key, the read, `flag_divergence`, the `agent-*.meta.json` read beside it, and the `transcript_path` written. Then retention's `transcript_is_gone` check on `session_meta.transcript_path`, and the meta read, look at the real file and not at the link.

  The first discovered path that yields a key is ingested. A later path with the same key that resolves to the same file is skipped with no note. A later path with the same key that resolves to a different file is never opened. Record it as a per-file failure naming the file that already holds the key, so it shows in `runs.error`.

  This is the planner's call on the flagged assumption, made because ingest appends from the watermark without comparing the file to the archived prefix in the steady state. A second file's tail would be appended onto the first file's session, mixing two files in one stream with nothing to detect it.

  Rewrite the `dispatch_round` comment to state the new invariant. A single-file `terminus ingest <path>` already canonicalizes, so it reads and keys the target and needs no change.
- **Verify:**
  - The full suite passes, as in Task 1.
  - By hand, with the Task 1 environment variables: make a temp `.claude/projects/-tmp-p/` holding session A (`11111111-...jsonl` from `session-basic.jsonl`) with sidecar `11111111-.../subagents/agent-a.jsonl` (any fixture under `tests/fixtures/subagents/`), and session B (`22222222-...jsonl`). B's `22222222-.../subagents/agent-a.jsonl` is a symlink to A's sidecar.
  - `terminus backfill` and then `terminus ingest` each leave one session for the pair. Its key is `-tmp-p/11111111-.../subagents/agent-a.jsonl`. `terminus sessions --json --project '*'` counts 3 sessions, not 4. `terminus status` reports no failed file.
  - Then configure a second root through a temp `terminus.toml` `roots` list. Give it a different file at `projects/-tmp-p/11111111-...jsonl`. The next `terminus ingest` adds no session and no turn, and its `runs.error` (`terminus status --json`) names the second file and the file that holds the key.

### Task 3: Regression test for a moved `.claude`

- **Files:** crates/terminus-core/tests/pass.rs
- **Action:** Write D-14's first regression test. It runs a pass over a temp root with `pass::run_with` and `Config::from_parts`, with the root as the configured root. The tree holds a top-level transcript, a sidecar, and a sidecar link spelled through the root's own path, the way Claude Code spells its resume links through `/home/john/.claude`. Then it renames the root directory, leaves a symlink at the old path pointing at the new one, and runs a second pass with the same config.

  Assert that the session count `config::visible::counts` reports (the number `terminus status` prints) is the same after the second pass as after the first. Assert that the second pass committed no file and added no turn, and that the set of `sessions.session_key` values is unchanged. The link in the tree makes the same test show AC2 for the tree pass: the count equals the number of distinct transcript files, not the number of paths discovery yields.

  Use only APIs that exist at commit 2b39948 (`pass::run_with`, `Config::from_parts`, `Store`, and SQL) so the test compiles there and AC6 can be checked. Add no other test. D-14 names two regression tests, and the second is Plan 2's.
- **Verify:**
  - `cargo test -p terminus-core --all-features --test pass` passes.
  - Copied unchanged into a worktree at 2b39948, the same test compiles and fails on the session count (AC6). Report the two counts it printed there.

## Notes

- Plans run in order 1, 2, 3. Their file sets do not overlap, but Plan 2 calls the key function this plan writes, so they cannot run in parallel.
- The key recognizes only D-01's two sidecar depths, which are the depths the real tree has. Discovery accepts an `agent-*.jsonl` at any depth below a project. One at a third depth would keep its canonical absolute path as its key under D-01's last sentence, and would be doubled again by a later move.
- Two roots holding different files at one path below `projects` are handled in the walks (Task 2). A single-file `terminus ingest <path>` naming the second root's file would still append to the session the first root's file keyed. That is the rare case CONTEXT's second flagged assumption describes. It is not planned here.
- Recalled from CONTEXT (phase 4): D-01, D-02, D-03 and D-13 bind Task 1, and D-14 binds Task 3.
