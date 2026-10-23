---
phase: 6
plan: 1
requirements: [ARCV-01]
files:
  - crates/terminus-core/src/ingest/mod.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/ingest/runs.rs
  - crates/terminus-core/src/ingest/driver.rs
  - crates/terminus-core/tests/lineage.rs
---

# Phase 6: Archive - Plan 1 of 3 (continuation links)

## Goal

Continuation links survive ingest order (#11). A session archived before its predecessor gets its `continues_from` link on the first tree pass after the predecessor is archived, even though its own transcript did not change.

## Must be true when done

- A tree pass over a continuing session alone leaves its `session_meta.continues_from` null. A later tree pass that archives the predecessor fills it with the predecessor's session id, even though the later session's file was unchanged and not committed on that pass.
- `terminus show <its first turn id> --after 1 --project '*' --json` reports that predecessor's session id in the record's `context.continues_from`.
- A session whose link was already set keeps it: a session with a foreign `session_id` keeps that id through a later pass that archives the session its `parentUuid` names.
- Only the tree pass fills links. `terminus backfill`, `terminus ingest <path>` and `reindex` do not run the sweep. A link they miss is filled on the next tree pass.
- The sweep never fails the pass. Whatever it could not do appears as one line in that pass's `runs.error`, and a healthy sweep writes nothing to `error` or `notes`.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-01, D-03, D-04, D-05, D-06 and D-17's `continues_from` half from `.planning/phases/6/CONTEXT.md`. The link is filled by a sweep in `Tree::after_walk`. It fills only NULLs and uses the rules of `ingest::continues_from`. It adds no new index, no `DERIVED_SCHEMA` bump, no one-off repair, and no call from `driver::drive`, the backfill, a named file or `reindex`.
- Out of scope (CONTEXT): `show`, the MCP tools, `recall::context::window` (they already read the column), the user docs, and linking from `terminus backfill` or `terminus ingest <path>` before the next tree pass.
- Patterns: `Tree::after_walk`'s steps each sit under a comment giving their position and why. `runs::insert` gathers each step's error lines in the order the steps run. `a_sidecar_ingested_before_its_parent_records_the_same_key` in `tests/lineage.rs` is the shape of a two-pass ingest-order test.
- Comments and tests cite `#11` only, never a requirement id or decision tag (`crates/terminus-core/tests/planning_tags.rs`). Rewrap comments by hand. Do not run `cargo fmt`.

## Tasks

### Task 1: Move the `parentUuid` lookup into one function that ingest and a sweep can both call

- **Files:** crates/terminus-core/src/ingest/mod.rs (`continues_from`)
- **Action:** Move the `parentUuid` half of `continues_from` into one crate-internal function in `ingest/mod.rs`. That half is the query resolving a message uuid through `turns` to the `session_meta.session_id` of a different file holding it, plus the `lineage::is_self_link` filter against the transcript's own session id. The function takes the connection, the session key, the parent uuid and the transcript's own session id. It answers the predecessor's session id, or none. `continues_from` calls it with `lineage::own_session_id(scan)`.
  - This task changes no behaviour. Keep these exactly as they are: the SQL text, the `t.session_key <> ?2` exclusion, the `LIMIT 1`, the `.flatten()` of a null `session_id`, and the order (the limit first, the self-link filter second).
  - The foreign `session_id` branch and the first-turn checks (`turn_seq != 0`, no `parent_uuid`) stay in `continues_from`. They read the scan, and the sweep in task 2 has no scan.
  - The "A different FILE may still report the same session id" comment moves with the filter.
  - The point is that task 2's sweep calls this same function, so the resolution rules are written once (D-01: "by the same rules as `ingest::continues_from`").
- **Verify:** `cargo test -p terminus-core --features testkit --test lineage` passes, `the_parent_uuid_fallback_still_links_a_fork` included. `cargo test -p terminus-core --all-features --lib ingest::` passes. `git diff --stat` names only `crates/terminus-core/src/ingest/mod.rs`.

### Task 2: Fill a missing continuation link on every tree pass

- **Files:** crates/terminus-core/src/ingest/mod.rs (`continues_from` and its doc, the function from task 1), crates/terminus-core/src/ingest/pass.rs (`Summary`, `Tree::after_walk`), crates/terminus-core/src/ingest/runs.rs (`insert`), crates/terminus-core/src/ingest/driver.rs (module doc), crates/terminus-core/tests/lineage.rs (model: `a_sidecar_ingested_before_its_parent_records_the_same_key`)
- **Action:**
  - **The sweep.** Add a crate-internal function in `ingest/mod.rs`, beside `continues_from`.
    - It selects every `session_meta` row whose `continues_from` is NULL and whose `turns` row at `turn_seq = 0` carries a `parent_uuid`, together with that row's own `session_id`.
    - For each row it calls task 1's function with that `session_id` as the session's own id.
    - Each id that resolves is written into `session_meta.continues_from` by an UPDATE whose WHERE also requires `continues_from IS NULL`.
    - It writes no other column and no other table. `show` and the MCP tools read the column through `recall::context::window`, so nothing else changes (D-03).
  - **D-05's rules hold by construction:**
    - A row that has a link is never selected. That includes every row whose foreign `session_id` was recorded at ingest, so a foreign id still wins over the fallback, and a link is never overwritten.
    - Only `turn_seq = 0` names a predecessor.
    - The self-link test is task 1's, against `session_meta.session_id`. `write_session_meta` fills that column from the first `sessionId`, which is exactly what `lineage::own_session_id` reads. So a sidecar whose first turn resolves into its parent's file still resolves to its own id, and stays NULL.
  - **Cost.** The sweep runs on every tree pass, which means every hook spawn. So it must reach `turns` only through existing indexes and never scan `turns`: `idx_turns_uuid` for the uuid, and the `UNIQUE (session_key, turn_seq)` index for the first turn. Add no index and no `DERIVED_SCHEMA` bump (D-01).
  - **Wiring.**
    - Call the sweep as the first step of `Tree::after_walk`, directly after the walk and before `feedback::outcomes`.
    - Run it on every tree pass, whatever the walk committed. A predecessor archived by `terminus backfill` or `terminus ingest <path>` shows up on a tree pass that has nothing new to commit (D-01).
    - The comment above the call gives its position and the reason, the way the comments on the steps below it do, and cites #11.
    - Do not call it from `driver::drive`, the backfill's walk, the named file's walk or `reindex` (D-01; `reindex.rs`'s module doc says `session_meta` is never touched).
  - **Failure.**
    - Nothing in the sweep fails the pass, and nothing in it panics.
    - What it could not do becomes one line naming the step, carried in a new `Summary` field. `runs::insert` adds that line to `error` after `summary.feedback.lines()` and before `summary.outcomes.errors()`, which is the order the steps run.
    - The field's doc says its line reaches `runs.error`.
    - A healthy sweep adds nothing to `error` or `notes`.
  - **Docs (D-17).**
    - `continues_from`'s doc says this: a `parentUuid` predecessor that is not archived yet leaves the column NULL at ingest, and the tree pass's sweep fills it once the predecessor is archived (#11).
    - The `ingest/driver.rs` module doc lists the steps only the tree pass runs. Add the sweep to that list. D-17 does not name this list, but it would be wrong without the sweep.
  - **The tests (D-06, AC1).** Add two tests in `tests/lineage.rs`, each running two real passes through `Tree::pass`, in the shape of `a_sidecar_ingested_before_its_parent_records_the_same_key`. They are separate so the first pass of the link test holds the continuing session alone, as D-06 and AC1 lock it.
    - **The link test.** Pass one's tree holds only a later session whose first record carries `parentUuid` naming the uuid of `record(1, ...)` (as in `the_parent_uuid_fallback_still_links_a_fork`). It has its own `sessionId` and no `session_id` field.
      - After pass one, assert that its `continues_from` is None.
      - Pass two adds the predecessor with `tree.transcript(1, &session(1), None)`. Assert that `files_committed` is 1, which proves the later session was unchanged and only the sweep could link it. Assert that the later session now names `session(1)`.
    - **The kept-link test.** Pass one's tree holds only a session whose first record carries both a foreign `session_id`, naming a session that is never archived, and a `parentUuid` naming the uuid of `record(1, ...)`.
      - After pass one, assert that it names its foreign id.
      - Pass two adds the predecessor with `tree.transcript(1, &session(1), None)`. Assert that the session still names its foreign id.
    - Comments cite #11 only.
- **Verify:**
  - **Lineage tests.** `cargo test -p terminus-core --features testkit --test lineage` passes, both new tests included.
  - **Mutation check.** With the sweep's call removed from `Tree::after_walk`, the link test fails on the later session's link. Put the call back.
  - **Callers.** `git grep -n` for the sweep function's name shows exactly its definition in `ingest/mod.rs` and one call in `ingest/pass.rs`.
  - **No scan of `turns`.** Run EXPLAIN QUERY PLAN with `sqlite3` on the scratch store the `show --json` check below builds. It shows no `SCAN` of `turns` for any statement the sweep runs.
  - **`show --json`.** Use a scratch directory with `TERMINUS_DATA_DIR`, an empty `TERMINUS_CONFIG_DIR`, and `CLAUDE_CONFIG_DIR` all pointing inside it.
    - Place the later transcript under `projects/<project>/` and run `target/debug/terminus ingest`.
    - Add the predecessor and run `terminus ingest` again.
    - `sqlite3 <data>/terminus.db "SELECT id FROM turns WHERE turn_seq = 0 AND session_key LIKE '%<later file name>'"` gives the turn id.
    - `terminus show <id> --after 1 --project '*' --json` prints a record whose `context.continues_from` is the predecessor's `sessionId`.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices**, all where CONTEXT left the call open:
  - The sweep resolves through the same function `continues_from` uses (task 1), so the self-link rule and the lookup are written once.
  - It is the first post-walk step.
  - Its failure reaches `runs.error` through a new `Summary` field. A link it fills is not reported, because CONTEXT asks for no such note.
- **One file beyond D-17's list:** the `ingest/driver.rs` module doc lists the steps only the tree pass runs, and that list would be wrong without the sweep.
- **Plan shape:** this is plan 1 of 3. The three plans touch disjoint files and none depends on another's behaviour, so they can run in parallel.
- **Prior art:** D-01 (phases/6/CONTEXT.md, phase 6) is task 2. Phase 5 found that `rustfmt --check` on `ingest/mod.rs` also walks into `observe/egress.rs`, which already has formatting diffs from before this phase (phases/5/SUMMARY.md, phase 5). Leave those diffs alone.
