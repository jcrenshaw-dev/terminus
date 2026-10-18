---
phase: 3
status: complete
completed: 2026-10-18
---

# Phase 3: Owner-only files - Summary

Install and uninstall open every `settings.json` temporary and backup at 0600 in the creating syscall, a rewrite keeps the existing file's own mode, and `terminus data move` normalizes its destination and refuses one that is a symlink, another uid's, group- or world-writable, unlistable or occupied, both before the question and again after the answer and the lock.

## What shipped

- Every install temporary opens through `owner_only::options()` - `create_temporary` in `crates/terminus/src/cmd/install/mod.rs:68` (#64)
- A rewrite copies the target's mode only when it differs from 0600, and the backup is 0600 whatever the source's mode - `fill` and `copy_through` in `crates/terminus/src/cmd/install/json_file.rs` (#64)
- The destination loses its trailing slashes and `.` parts before it is checked, copied into or recorded - `absolute` in `crates/terminus/src/cmd/data.rs` (#59)
- The destination rule: not a symlink, a directory, owned by this uid (learned from a `.uid-probe-<pid>-<n>` file in the config directory), `mode & 0o077 == 0` with a `chmod 700` hint, listable and empty - `refuse_unchecked_destination` and `own_uid` in `crates/terminus/src/cmd/data.rs` (#59)
- The second check after the answer and the lock, which refuses unless the destination is still absent or the same directory by device and inode - `run` in `crates/terminus/src/cmd/data.rs` (#59)
- Tests: unit tests in `install/json_file.rs`, `install/mod.rs` and `cmd/data.rs`; integration tests in `crates/terminus/tests/install.rs` and `crates/terminus/tests/datamove.rs`
- Docs: `THREAT-MODEL.md` (file modes, Claude Code's own `settings.json` mode, the `data move` checks and the gaps they leave), `docs/commands.md`, `docs/design/0004-archive-and-block-rows.md`, `CHANGELOG.md` `[Unreleased]` `### Fixed` (#64, #59)

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 3322699 | Every install temporary opens at 0600 |
| 1 | 2 | e8e30c7 | A rewritten `settings.json` keeps its own mode |
| 1 | 3 | db3693f | The settings backup is 0600 whatever the source's mode |
| 1 | 4 | 2ab031d | Docs record the 0600 temporaries and the mode Claude Code 2.1.302 keeps |
| 2 | 1 | e89458d | The destination's trailing slashes and `.` parts are dropped before it is used |
| 2 | 2 | f5b7c63 | A symlink, foreign, wide, unlistable or occupied destination is refused before the question |
| 2 | 3 | 5d55f2b | The destination is checked again after the answer and the lock |
| 2 | 4 | ef5133e | Docs record the destination checks and the gaps they leave |

## Deviations

- [deviation] Plan 1 task 4: `THREAT-MODEL.md` "What the user would not notice" still said the `settings.json` temporary and backup sit at the umask mode for a moment, which the fix makes false. The task said to change only the `settings.json` bullets; that line was removed too (2ab031d).

## Open items

- The race in `data move` is narrowed, not closed. Between the second check and the `mkdir` in `copy_tree`, a directory another account creates at the destination is still accepted, because `owner_only::create_dir_all` returns Ok for an existing leaf and its `path.is_dir()` follows a symlink. `THREAT-MODEL.md:103` states the gap. Two automated security reviews of plan 2's commits flagged it at MEDIUM. #59 does not fully close while it stands.
- The mode Claude Code's second settings write path (`publishDiscipline:"followAtomic"`) gives a file was not traced through its storage backend. `THREAT-MODEL.md` states the gap.
- The existing `a_destination_that_already_holds_files_is_refused` test makes its directory at the umask default, so the mode rule now refuses it before the occupancy rule runs. It still passes, and the occupied case is covered by the `an_owner_only_directory_that_holds_a_file_is_refused` unit test instead of at the binary boundary.
- Pre-existing, outside this phase: the same-path and inside-the-source checks in `refuse_impossible_destinations` compare paths as text, so a destination that does not exist yet and is reached through a symlinked parent of the source passes them, and `copy_tree` would copy the source into itself. `THREAT-MODEL.md`'s "What it deletes" row states it.
- Issues #64 and #59 are not closed yet. The change has to close them when it lands.

## Goal check

The eight commits deliver the install half of the goal and most of the `data move` half. Every install temporary is opened through `owner_only::options()` (`crates/terminus/src/cmd/install/mod.rs:68`, 3322699), the backup no longer copies the source's mode (db3693f), and reverting either change fails its test with "settings.json is 644" or "the backup is 644" (plan 1 report). `data move` now refuses a symlink, 0777 and unlistable 0300 destination before the question (f5b7c63), and refuses a destination created, replaced or widened while the question waited (5d55f2b); reverting each check fails its datamove test with the move's own "moved." output (plan 2 report). The final workspace run passed 67 test binaries with 0 failures (plan 2 report). One thing falls short: "owner-only from the start" does not hold for the moment between the second check and the `mkdir`, where an existing directory is still accepted (`crates/terminus-core/src/owner_only.rs:54`, `THREAT-MODEL.md:103`). `Store::open` still checks no owner, which D-01 left out of this phase on purpose.
