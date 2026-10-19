# Phase 2: Read-only status and verify - Context

Gathered: 2026-10-19
Feeds: /cad-plan 2

## Scope boundary

In: `terminus status` and `terminus verify` open the store through `cmd/read.rs`, so a missing store is reported and creates nothing, and an older store is reported and keeps its schema (#77). `read::open_in` reports a database with no `meta` table as no store. The tests that pin both, the existing tests built on the writable open, and the docs, comments and CHANGELOG entry the change makes false.
Out: Making the writable `Store::open` take the ingest lock as an argument (D-02). Any change to `Store::open` or `Store::open_read_only` (D-01).
Deferred: None.
Plan shape: one plan.

## Durable decisions

None this phase.

## Decisions

- D-01 (Which open): `status` and `verify` open through `cmd/read.rs` (`read::open` or `read::open_in`), the path search, show, sessions, doctor, stats, replay and backfill use. `Store::open` and `Store::open_read_only` do not change. Evidence: `crates/terminus/src/cmd/status.rs:40`, `crates/terminus/src/cmd/verify.rs:12`, `crates/terminus/src/cmd/read.rs:94-115`, `crates/terminus-core/src/store/open.rs:137-256`; v0.1.0 phase 3 D-10, v0.1.0 phase 4 D-12, hardin phase 3 D-01.
- D-02 (Lock argument): #77's second ask, making the writable open take the ingest lock as an argument, is filed as its own issue and is not done here. #77 closes on the routing change alone. The issue's list of lock-holding openers is partly wrong (`data move` and `uninstall` never open the store), ingest's `roll` calls `Store::open` after the guard is dropped and phase 4 (#85) changes that function, and about 64 test call sites use `Store::open(`. Evidence: `crates/terminus-core/src/ingest/pass.rs:135-147`, `:165-172`; `crates/terminus/tests/cli.rs:290-293`; issue #77.
- D-03 (Store being created): `read::open_in` reports a database with no `meta` table as no store, the same answer as a missing one, for every read command. A `status` run in the moment the detached backfill is creating the store then reports no store instead of exiting 1. Leaving the read path as it is was rejected, because it brings back in a new form the "database is locked" fix CHANGELOG already records. Evidence: `crates/terminus-core/src/store/open.rs:137-165`, `:234-256`; `CHANGELOG.md` Unreleased Fixed; `docs/install.md:45`; v0.1.0 phase 4 SUMMARY.
- D-04 (verify's config): `verify` opens with a default config, not `Config::load()`, because it reads no visible-session filter. A malformed `terminus.toml` cannot stop the integrity check. Evidence: `crates/terminus/src/cmd/verify.rs:10-13`, `crates/terminus-core/src/verify.rs:103-130`, `crates/terminus/src/cmd/read.rs:83-87`, `crates/terminus-core/src/config.rs:486-506`.
- D-05 (No store): With no store, both follow `read::empty` and exit 0, `verify` included. In text mode stdout is empty and the reason goes to stderr. With `--json` there is one document with `ok: true`, the reason, and every documented `data` field at zero or null: for `status`, `size_bytes` and the counts 0, `excluded` and `last_run` null; for `verify`, `checked` 0 and `failures` empty. Exit 1 for `verify` alone was rejected: the reason line still reports the missing store, and no script that relies on today's exit 0 breaks. Evidence: `crates/terminus/src/cmd/read.rs:14-18`, `:98-105`, `:196-203`; `crates/terminus/src/cmd/stats.rs:26-28`; `crates/terminus/tests/cli.rs:952-985`; `THREAT-MODEL.md:204`.
- D-06 (Older store): On a store older than the binary, both print the existing stale-store line from `read::open_in` on stderr, in both modes and with the same wording, then run their queries. Evidence: `crates/terminus/src/cmd/read.rs:73-75`, `:110-112`; `crates/terminus/tests/cli.rs:1252-1273`; `crates/terminus-core/src/store/open.rs:388-396`.
- D-07 (Columns): `status` and `verify` keep naming their columns directly, with no fallback for a missing one. Every column they read is declared at `ARCHIVE_FORMAT` 2 except `runs.notes`, which arrived 2 h 14 min later on 2026-10-14, so only a store created in that window and never opened for writing since can lack one. Evidence: commits `2a6fd01`, `65ef2d1`; `crates/terminus-core/src/store/open.rs` `read_versions`.
- D-08 (Older-store test): The older-store regression test builds a realistic format-2 store: `derived_schema` below the current one, plus a bring-forward column or table neither command reads (for example `decisions.compacted` or `dropped`, or no `compactions`). It does not use `aged_store`, which drops columns `status` reads, and a lowered version alone passes the schema half before the change. Evidence: `crates/terminus-core/tests/schema.rs:196-236`; `git show 2a6fd01:crates/terminus-core/src/store/schema.rs`; `crates/terminus-core/src/store/open.rs:517-538`.
- D-09 (Existing tests): The two no-store tests in `crates/terminus/tests/status.rs` change to the new answer. `crates/terminus/tests/tighten.rs` uses `status --json` as the writable open that narrows a wide data directory, so it switches to a command that still opens for writing. Evidence: `crates/terminus/tests/status.rs:107-121`, `:396-406`; `crates/terminus/tests/tighten.rs:1-8`, `:198-229`; `crates/terminus-core/src/store/open.rs:411-420`.
- D-10 (Ordinary store): On an ordinary store, `status`'s output is the same through the read-only open, `size_bytes` included. Measured on Linux on 2026-10-19: the writable and read-only opens both reach 204800 bytes (`terminus.db` 172032, `-shm` 32768, `-wal` 0); the read-only open leaves both sidecars on disk after it exits. Evidence: `crates/terminus/tests/uninstall.rs:776-792`, `crates/terminus/tests/lifecycle.rs:216-222`, `:290-324`.
- D-11 (Docs): This phase corrects every statement the change makes false: `THREAT-MODEL.md:75`; the `status` and `verify` entries in `docs/commands.md:108-122` and `docs/json-shapes.md:191-218`; the module doc of `crates/terminus/src/cmd/read.rs`; `crates/terminus-core/src/store/open.rs:186-196`; the `BRING_FORWARD_COLUMNS` doc in `crates/terminus-core/src/store/schema.rs`; `crates/terminus/src/cmd/stats.rs:9-11`; `crates/terminus-core/tests/schema.rs:297-301`. It adds an Unreleased Fixed entry for #77 to `CHANGELOG.md`. Leaving `docs/` and the comments to phase 7 was rejected, because they would be wrong from this commit until then and phase 7 does not list `THREAT-MODEL.md`. Evidence: issue #88; `.planning/ROADMAP.md` phase 7.

## Acceptance criteria

- [ ] AC1: With `TERMINUS_DATA_DIR` naming a directory that does not exist, `terminus status` and `terminus verify` each exit 0 and create nothing. In text mode stdout is empty and stderr carries the no-store reason. With `--json`, stdout is one document with `ok: true`, the reason, and every `data` field at zero or null. The same holds for a database file that has no `meta` table.
- [ ] AC2: On a format-2 store older than the binary (its `derived_schema` below the current one, and lacking a column neither command reads), both exit 0 and print the stale-store line on stderr, and the store's `sqlite_master` and every table's `PRAGMA table_info` are the same after the run as before.
- [ ] AC3: On an ordinary store, `status --json` and `verify --json` print the same output as before the change, `size_bytes` included.
- [ ] AC4: With a malformed `terminus.toml`, `verify` on a clean store exits 0 and reports no failures.
- [ ] AC5: `THREAT-MODEL.md`, `docs/commands.md` and `docs/json-shapes.md` no longer say `status` or `verify` creates a store, and `CHANGELOG.md`'s Unreleased Fixed section has an entry naming #77.
- [ ] AC6: The regression tests for AC1 and AC2 fail against main's code and pass after the change, and `cargo test --workspace --all-features` passes.
- [ ] AC7: Issue #77 is closed by the merged change. (human-verify: needs the merged PR on GitHub)

## Flagged assumptions

- A read-only open on macOS (APFS) also creates the 32 KB `-shm` and an empty `-wal`, so `size_bytes` matches the writable open there - Unclear, measured on Linux only; if wrong: AC3 fails on the `macos-15` and `macos-15-intel` CI runners, and `status` reports a different size on macOS than before. CI settles it.
- For `verify`, the shared stale-store line says derived results may be incomplete, though `verify` checks archive blocks, not derived tables - Likely acceptable; if wrong: a user re-runs ingest expecting it to change a verify result.
