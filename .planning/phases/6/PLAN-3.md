---
phase: 6
plan: 3
requirements: [ARCV-03]
files:
  - crates/terminus-core/src/observe/mechanical.rs
---

# Phase 6: Archive - Plan 3 of 3 (the observations query)

## Goal

Observations read only a session's own entities (#70). The entity query starts from the session's turns, both in `observations regenerate` and in the ordinary pass, and a test fails if its plan goes back to scanning every entity of a kind.

## Must be true when done

- EXPLAIN QUERY PLAN of the SQL `mechanical::entity_values` runs, on a store built from `CREATE_SQL`, uses `idx_entities_turn` and never `idx_entities_lookup`. A unit test asserts this on that exact SQL, and fails when the query is put back to its kind-driven shape.
- `observations regenerate` and the ordinary pass both reach the query through `mechanical::observe` and produce the same documents as before. `crates/terminus-core/tests/observe.rs` passes unchanged.
- No index is added and `DERIVED_SCHEMA` stays at 5, so no existing store rebuilds. `crates/terminus-core/src/store/` is unchanged.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-02, D-13, D-14, D-15, D-16 and D-17's `entity_values` half from `.planning/phases/6/CONTEXT.md`. The query's shape alone is the fix: it starts from `turns` by `session_key`, with the join order forced. The guard test runs EXPLAIN QUERY PLAN on the shared SQL constant and asserts index names only.
- Out of scope (CONTEXT): `from_blob`, `files()`, `observe/mod.rs`, `cmd/observations.rs`, any new index, a `DERIVED_SCHEMA` bump, the user docs.
- Comments and tests cite `#70` only, never a requirement id or decision tag (`crates/terminus-core/tests/planning_tags.rs`). Rewrap comments by hand. Do not run `cargo fmt`.

## Tasks

### Task 1: Drive the entity query from the session's turns, guarded by its plan

- **Files:** crates/terminus-core/src/observe/mechanical.rs (`entity_values`, the `tests` module)
- **Action:**
  - **The query (D-02, D-14).**
    - Move the SQL `entity_values` prepares into a private module-level constant, and have `entity_values` prepare that constant.
    - Reshape it to start from `turns` and force that order: `turns t CROSS JOIN entities e`, joined on `e.turn_id = t.id`.
    - Keep `SELECT DISTINCT e.value_norm`, the filters `t.session_key = ?1` and `e.kind = ?2`, the same parameter order, and `ORDER BY e.value_norm`. The output must be the same values in the same order, because `cap` keeps the first `MAX_LIST` in that order and `regenerate` rewrites stored documents.
    - Choose `CROSS JOIN` over `+e.kind`. D-02 allows both, and both were measured to plan `sqlite_autoindex_turns_1` and then `idx_entities_turn`. `CROSS JOIN` pins the join order in the statement itself, whereas `+e.kind` only takes one index out of consideration and leaves the order to the planner.
    - Do not use the `e.turn_id IN (SELECT id FROM turns WHERE session_key = ?1)` shape that `reindex::clear_derived` uses. With `e.kind = ?` beside it, it still plans on `idx_entities_lookup` (D-02).
  - **What stays unchanged.** Change nothing in `store/schema.rs`, `store/open.rs`, `observe/mod.rs` or `cmd/observations.rs`, and nothing in `files()` or `from_blob` (D-02, D-13, D-15).
  - **The doc (D-17).** `entity_values`'s doc says why the query drives from `turns`, citing #70:
    - The kind-driven join planned `SEARCH e USING INDEX idx_entities_lookup (kind=?)`, which reads every entity of that kind in the store for each session.
    - Measured on a 3,448-session store holding 138,371 `tool` entities, 30 sessions took 1.82 s that way and 0.011 s this way, and returned the same 86 values.
    - The order is forced rather than indexed because an index reaches an existing store only through a `DERIVED_SCHEMA` bump, which rebuilds every store.
    - The test below holds the shape.
  - **The guard test (D-16, AC4).** Add it in `mod tests`, using the constant itself, never a copy of its text.
    - Build a store in a `tempfile` directory, either by executing `crate::store::schema::CREATE_SQL` on a connection to a database file there, or with `Store::open` on that directory.
    - Run EXPLAIN QUERY PLAN on the constant with both parameters bound, and collect the `detail` column.
    - Assert that some detail names `idx_entities_turn` and none names `idx_entities_lookup`.
    - Assert index names from the schema only. Do not assert SQLite's plan wording or a `sqlite_autoindex_*` name, and run no `ANALYZE`. That way a SQLite rewording on a rusqlite bump cannot fail the test.
- **Verify:**
  - **Guard test.** `cargo test -p terminus-core --all-features --lib observe::mechanical` passes.
  - **Mutation check.** With the constant put back to the old kind-driven text (`FROM entities e JOIN turns t ON t.id = e.turn_id WHERE t.session_key = ?1 AND e.kind = ?2`), the guard test fails. Restore the new shape.
  - **Same output.** `cargo test -p terminus-core --features testkit --test observe` passes, and `git diff --exit-code e8b4d8a -- crates/terminus-core/tests/observe.rs` exits 0 (AC5).
  - **`observations regenerate`.** `cargo test -p terminus --all-features --test observations` passes.
  - **Nothing else touched.** `git diff --exit-code e8b4d8a -- crates/terminus-core/src/store crates/terminus-core/src/observe/mod.rs crates/terminus/src/cmd/observations.rs` exits 0.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices**, all where CONTEXT left the call open:
  - `CROSS JOIN` over `+e.kind`.
  - The guard test lives in `mechanical.rs`'s `mod tests` and the SQL constant stays private. This resolves CONTEXT's flagged assumption: an integration test could not see a `pub(crate)` constant.
- **Prior art:**
  - Phase 2 showed that a fresh store built from `CREATE_SQL` opens clean and answers real queries (phases/2/UAT.md, phase 2), which is the store this test builds.
  - Phase 4's decision that a test runs against the bundled SQLite, so a rusqlite bump fails the suite (phases/4/CONTEXT.md, phase 4), is the same idea. D-16 narrows it to index names, so a rewording alone cannot fail the test.
- **Plan shape:** this is plan 3 of 3. It shares no file with plans 1 and 2, and none depends on another's behaviour.
