---
phase: 1
plan: 1
requirements: [SHED-01]
files:
  - crates/terminus-core/tests/session_keys.rs
  - crates/terminus-core/Cargo.toml
  - crates/terminus-core/src/testkit.rs
  - crates/terminus-core/src/recover.rs
  - crates/terminus-core/src/reindex.rs
  - crates/terminus-core/src/ingest/pass.rs
  - crates/terminus-core/src/retention/apply.rs
  - docs/design/0003-ingest.md
  - docs/design/0004-archive-and-block-rows.md
  - docs/commands.md
  - docs/json-shapes.md
  - THREAT-MODEL.md
---

# Phase 1: Old store code - Plan 1 of 3 (recovery)

## Goal

Recovery and the store open carry no code for store layouts 1.0.0 can't create (#105). This plan covers recovery: it opens the store, brings the derived tables up to date and sweeps the watermarks, and nothing else.

## Must be true when done

- `crates/terminus-core/src/recover.rs` has no `rekey`, `stale_keys`, `merge` or `remap_ids`: `grep -ci merge crates/terminus-core/src/recover.rs` prints 0, and `crates/terminus-core/tests/session_keys.rs` no longer exists.
- `grep -rniE '\bre-?key|stale_keys|stale_parents|remap_ids|move_key|plant_old_key|UnmergedTwin|\btwins?\b' crates/ fuzz/` matches no line.
- A pass over a store whose watermark is past its blob still lowers it, removes a watermark naming no session, and reports one line per repair, as `tests/recover.rs` and `tests/capture.rs` already check.
- `grep -rniE '\bre-?key|\btwins?\b' docs/ THREAT-MODEL.md README.md` matches no line, and the ingest design doc describes recovery as opening the store, the derived-table rebuild and the watermark sweep.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-01 (remove the re-key and twin merge outright, accepting that a transcript outside the `projects/<folder>/` shapes that later becomes a symlink into a project folder stays two sessions), D-02, D-03, D-04, and the re-key half of D-13. Source: `.planning/phases/1/CONTEXT.md`, phase 1.
- Tests go before the code they test, so every task leaves `cargo test` green.
- Out of scope: how a pass reports recovery's watermark lines (phase 5, #96), planning tags in comments (phase 2, #106). Comment edits here change only wording about the removed code; tags such as `ING-03` or `D-24` stay where they are.
- Plans run in order 1, 2, 3. This plan shares `reindex.rs`, `testkit.rs` and `docs/design/0004-archive-and-block-rows.md` with plan 2, and `docs/design/0003-ingest.md`, `docs/commands.md` and `docs/json-shapes.md` with plan 3.

## Tasks

### Task 1: Delete the re-key's tests and their helper

- **Files:** crates/terminus-core/tests/session_keys.rs, crates/terminus-core/Cargo.toml (the `[[test]]` block named `session_keys`), crates/terminus-core/src/testkit.rs (`plant_old_key`)
- **Action:** Delete `crates/terminus-core/tests/session_keys.rs`, the one `[[test]]` block in `crates/terminus-core/Cargo.toml` whose name is `session_keys`, and `testkit::plant_old_key` with its doc comment (D-03: `plant_old_key` has no caller besides `session_keys.rs`). Touch no other `[[test]]` block. Leave `testkit::recut` where it is even though `session_keys.rs` is its only caller today: D-03 names `plant_old_key` alone, and a `pub` function in the library raises no warning. The re-key code itself stays until task 2, so every remaining test still passes after this commit.
- **Verify:** `test ! -e crates/terminus-core/tests/session_keys.rs` succeeds; `grep -rn 'session_keys\|plant_old_key' crates/` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test recover --test capture` passes.

### Task 2: Remove the re-key and the twin merge from recovery

- **Files:** crates/terminus-core/src/recover.rs (`rekey` through `move_key`; `UnmergedTwin`; `Recovered`; `recover`; the module docs), crates/terminus-core/src/reindex.rs (the `up_to_date` doc comment), crates/terminus-core/src/ingest/pass.rs (the comment above the `read_rows` call in the pass), crates/terminus-core/src/retention/apply.rs (the `delete_session` doc comment)
- **Action:** In `recover.rs`, delete `rekey`, `stale_keys`, `stale_parents`, `Member`, `Stream`, `one_stream`, `hashes`, `block`, `merge`, `repoint`, `remap_ids` and `move_key`, the type `UnmergedTwin`, and the `Recovered` fields `rekeyed`, `merged`, `unmerged` and `notes` (D-02). `recover()` becomes `Store::open`, then `reindex::up_to_date`, then `sweep_watermarks`, timed as now. `Recovered::repaired()` answers whether anything was lowered or removed, and `Recovered::lines()` gives one line per lowered and per removed watermark in the exact wording it uses now, so `tests/recover.rs` and `tests/capture.rs` pass unchanged. Recovery no longer calls `feedback::drain`: only the re-key did, and the pass's own drain after recovery stays as it is. Remove the imports only the deleted code used. Rewrite the module docs, the `Recovered::lines` doc and the `recover` doc so they describe the two steps that remain (the rebuild and the watermark sweep) and no longer mention a re-key, twins, a merge or absolute keys (D-04). The word `merge` must not appear anywhere in the file in any case, `merged` and `emerge` included, because success criterion 1 is checked as a text search; `sweep_watermarks` and the capture-mode paragraphs keep their content. Then rewrite three comments outside `recover.rs` that describe the re-key (D-03): the `reindex::up_to_date` doc says recovery opens the store and then brings the derived tables forward, with no re-key or twin; the comment above `read_rows` in `ingest/pass.rs` says recovery's watermark sweep is what rewrites rows before the walk reads them; the `retention::apply::delete_session` doc drops the sentence about sharing with recovery, since `delete_one` is now its only caller. Do not change what `delete_session` does: phase 6 rewires it. D-01 accepts the one remaining effect: a store with absolute keys from before 1.0.0 is no longer re-keyed. The live store was measured on 2026-10-22 with none (CONTEXT D-01, phase 1).
- **Verify:** `grep -nE 'rekey|stale_keys|remap_ids' crates/terminus-core/src/recover.rs` prints nothing and `grep -ci merge crates/terminus-core/src/recover.rs` prints `0`; `grep -rniE '\bre-?key|stale_keys|stale_parents|remap_ids|move_key|plant_old_key|UnmergedTwin|\btwins?\b' crates/ fuzz/` prints nothing; `cargo check --workspace --all-features --all-targets` finishes with no `warning:` line; `cargo test -p terminus-core --all-features --test recover --test capture --test pass --test backfill --test retention --test reindex` passes.

### Task 3: Bring the docs and threat model up to date on recovery

- **Files:** docs/design/0003-ingest.md, docs/design/0004-archive-and-block-rows.md (the Session row), docs/commands.md (`terminus status`), docs/json-shapes.md (the session key paragraph before `### search`), THREAT-MODEL.md
- **Action:** Design docs describe the current design only, and nothing public carries 0.1.x history. In `0003-ingest.md`: the `recover.rs` row of the file table lists only the rebuild and the watermark sweep; the single-file ingest and backfill entries lose the sentence about recovery draining decision files when the re-key has work; step 2 of the pass says recovery opens the store, rebuilds the derived tables when `derived_schema` differs from `DERIVED_SCHEMA`, and sweeps the watermarks, with the re-key bullets, the twin test, the parent-key reduction and the "re-key that fails stops the run" sentence gone, while "no archived byte is lost" now speaks only of lowered and removed watermarks; the flowchart node for `recover::recover` names only the rebuild and the sweep; the `read_rows` paragraph says it runs after recovery and the drain because recovery's sweep rewrites watermark rows; Figure 3 loses the three transitions about re-keying and merging, and its caption loses the sentences about moving a watermark to a new key and giving a merged group the twin's watermark. In `0004-archive-and-block-rows.md`, delete the Session row's last sentence about recovery re-keying and merging. In `commands.md`, the `terminus status` paragraph's example of a recovery line becomes a lowered watermark rather than a re-key count, and it still says such lines print under `error` (phase 5 changes that, not this phase). In `json-shapes.md`, delete the two sentences saying a stored absolute key is moved by the next ingest and that a session keeps its absolute key when a twin cannot be proved; the sentence that a path outside the two shapes shows its canonical path stays. In `THREAT-MODEL.md`, delete the "Recovery's twin merge" row of the deletes table, the invariant's "one exception" sentence about the twin merge, and the "Recovery's re-key" bullet under what Terminus trusts.
- **Verify:** `grep -rniE '\bre-?key|\btwins?\b' docs/ THREAT-MODEL.md README.md` prints nothing; `grep -ci merge docs/design/0003-ingest.md docs/design/0004-archive-and-block-rows.md` prints `0` for both; `grep -ci merge THREAT-MODEL.md` prints `1` (the install line only); `grep -n 'recover::recover' docs/design/0003-ingest.md` shows the flowchart node naming only the rebuild and the watermark sweep.

## Notes

- The planner found no other caller of the helpers the re-key used (`feedback::drain`, `retention::apply::delete_session`, `store::archive::ranges`, `parse::scan_from`, `derive::derive_boundary`, `blob::BlockReader`, `schema::split_turn_id`). Each keeps a caller after task 2, as D-03 says.
- `testkit::recut` has no caller once `session_keys.rs` is gone. D-03 does not list it, so it stays. That is for John to decide, not this plan.
