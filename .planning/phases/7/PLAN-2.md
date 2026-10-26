---
phase: 7
plan: 2
requirements: [OWN-01]
files:
  - crates/terminus/src/cmd/data.rs
  - crates/terminus-core/src/store/open.rs
  - crates/terminus-core/src/store/mod.rs
  - crates/terminus/tests/owner_only.rs
  - crates/terminus-core/src/owner_only.rs
  - THREAT-MODEL.md
---

# Phase 7: Owner-only files and signing - Plan 2 of 3 (data move, the pointer, and the docs for #73)

## Goal

`terminus data move` creates its directories, its copies, its uid probe, its pointer and the config directory owner-only through the `owner_only` helpers. The pointer's writer lives in core beside its reader. No exclusive-create retry loop is left outside `owner_only.rs` (#73).

## Must be true when done

- Start under umask 022 with no config directory. `terminus data move` to a missing destination, and to an existing empty destination, each leave `data-location` at 0600 and the config directory at 0700. A config directory that already exists keeps its mode.
- Outside `crates/terminus-core/src/owner_only.rs` and test code:
  - `.create_new(true)` appears only in `store/open.rs`'s create of `terminus.db`;
  - no code calls `std::os::unix::fs::symlink`;
  - no loop retries on `AlreadyExists`.
- `data move` still writes an owner-only tree however wide the source was, and every `tests/datamove.rs` test passes unchanged.
- Two descriptions now match the code:
  - `THREAT-MODEL.md`'s config directory paragraph gives the pointer's mode, the config directory's mode, and the temporary nothing sweeps.
  - The `owner_only.rs` header names every site that creates through it.
- `cargo test --workspace --all-features` passes and `cargo clippy --locked --all-targets --all-features` reports no warning.

## Context

- **Locked decisions** (`.planning/phases/7/CONTEXT.md`):
  - D-02 for the `data.rs` sites.
  - D-03: `data move`'s truncating `LOCK` keeps `options()`.
  - D-04: the probe keeps `.uid-probe-{pid}-{n}` and 100 attempts.
  - D-08: the pointer's writer moves into `store/open.rs` as a public function.
  - D-09: both places that can create the config directory change.
  - D-10: the pointer's temporary gets a unique name from the retry helper.
  - D-11: the proof runs through the shipped binary with the config directory absent.
  - D-16: `THREAT-MODEL.md` and the `owner_only.rs` header.
  - D-17.
- **Builds on plan 1:**
  - task 1's single exclusive create, retry file form and exclusive directory;
  - task 4's `replace`.
- **Out of the phase:**
  - repairing a config directory that already exists at another mode;
  - sweeping a stray pointer temporary;
  - `docs/design/0004-archive-and-block-rows.md` and `docs/design/0001-overview.md`, which stay as they are (D-16).
- **Comments and tests** cite #73 only (`crates/terminus-core/tests/planning_tags.rs`). Rewrap comments by hand. Do not run `cargo fmt`.
- **Commands.** Run every `cargo test` with `--all-features`, because `datamove` needs `testkit`.

## Tasks

### Task 1: Move data move's probe, directories and file copies onto owner_only

- **Files:** crates/terminus/src/cmd/data.rs (`own_uid`, `create_destination`, `copy_tree`, `copy_file`, `Found`, and the unit tests on `create_destination`)
- **Action:**
  - **`own_uid`'s probe loop** becomes the retry file form, named `.uid-probe-{pid}-{n}`, with 100 attempts (D-04).
    - The probe is still removed once its uid is read.
    - Exhaustion keeps the message "every probe name under {dir} for pid {pid} is taken ... nothing was moved".
    - Any other error keeps "could not be created to learn this account's uid: {e}; nothing was moved".
    - Leave `own_uid`'s `std::fs::create_dir_all` for task 2, which owns the config directory.
  - **`create_destination`** becomes the exclusive directory helper: parents at the process default, the leaf one exclusive mkdir at 0700. Its `AlreadyExists` still reaches `run`'s "it was not there and now is" mapping.
  - **`copy_tree`'s `DirBuilder`** becomes the exclusive directory helper. Its parent `to` already exists, so only the leaf is made.
  - **`copy_file`** becomes the single exclusive create followed by `std::io::copy`.
  - **Remove** the `DirBuilderExt` imports.
  - **Unit tests.** `data.rs`'s three tests on `create_destination` keep passing: owner-only with its parents, a directory already there, a symlink already there. Either keep `create_destination` as the call they test, or remove it with them, since plan 1 task 1's `owner_only` tests cover the same three cases.
  - **Doc comments.** On `copy_tree`, `copy_file`, `create_destination` and `Found::Nothing`, keep the reasons specific to `data move`: `refuse_unchecked_destination` found the destination empty or absent, so a taken name is something else writing mid-move. Drop the generic `create_new` explanation. Every intra-doc link must name an item that still exists.
  - **`LOCK`** keeps `options()` (D-03).
- **Verify:**
  - **Unit tests.** `cargo test -p terminus --all-features --lib cmd::data::` passes.
  - **Move tests.** `cargo test -p terminus --all-features --test datamove` passes. So does `cargo test -p terminus --all-features --test owner_only a_data_move_writes_an_owner_only_tree_however_wide_the_source_was`.
  - **No hand-built creates.** `grep -n "create_new\|DirBuilder\|AlreadyExists => continue" crates/terminus/src/cmd/data.rs` prints nothing outside `mod tests`.
  - **Clippy.** `cargo clippy --locked --all-targets --all-features` reports no warning.

### Task 2: Write the pointer 0600 and the config directory 0700 from core

- **Files:** crates/terminus-core/src/store/open.rs (beside `location_pointer_path` and `pointed_at`), crates/terminus-core/src/store/mod.rs (the `pub use open::{...}` list), crates/terminus/src/cmd/data.rs (`write_pointer`, `own_uid`, `run`), crates/terminus/tests/owner_only.rs (model: `a_data_move_writes_an_owner_only_tree_however_wide_the_source_was`, `bench_located`)
- **Action:**
  - **The core function.** In `store/open.rs`, beside `location_pointer_path` and `pointed_at`, add a public function that records where the data directory moved (D-08).
    - It takes the destination and resolves the pointer's path through `location_pointer_path`.
    - It creates the config directory with `owner_only::create_dir_all`. That is 0700 when this call creates it; a config directory that already exists keeps its mode (D-09).
    - It writes the destination's display text plus one newline, byte for byte what `write_pointer` writes today, through `owner_only::replace`.
    - Its temporary is named `.data-location-{pid}-{n}.tmp`. That is the planner's name under D-10: the leading dot keeps it from reading as the pointer, and `.tmp` matches the other temporaries.
    - Its errors are `Error::io`, naming the path that failed.
  - **Its doc comment:**
    - It carries `write_pointer`'s reason for the rename: a truncated pointer would name half a path.
    - It says the pointer is 0600 from the create, and the config directory is 0700 only when this call creates it.
    - It says a move killed mid-write can leave the temporary behind, and nothing deletes it.
  - **Re-export.** If `data.rs` reaches the function as `store::...`, re-export it from `store/mod.rs` beside `location_pointer_path`.
  - **`data.rs` changes:**
    - Remove `write_pointer`.
    - `run` calls the new function where it called `write_pointer`. The call is a plain `?`: the error reaches the CLI through `From<terminus_core::Error> for Failure` (D-08).
    - `own_uid`'s `std::fs::create_dir_all(dir)` becomes `terminus_core::owner_only::create_dir_all(dir)` (D-09). It runs before the question when the destination exists, so whichever of the two runs first creates the config directory, and creates it 0700.
    - `own_uid`'s doc still says why the probe goes in the config directory; fix any sentence that names `write_pointer`.
  - **The test** (D-11) goes in `tests/owner_only.rs`. It runs the shipped binary under the bench's umask 022 with `Located::ByPlatform`, in one test or two, over two cases:
    - an absent destination;
    - an existing empty destination, which the test creates and sets to 0700. `refuse_unchecked_destination` refuses a wider one.
  - **Each case:**
    1. Place a fixture and run `ingest`.
    2. Remove the bench's config directory and assert it is absent. The bench creates `config_dir` up front, so this step is what makes the test prove the fix (D-11).
    3. Run `data move <destination> --yes`.
    4. Assert `config_dir` is 0700, and that `config_dir/data-location` is 0600 and holds the destination plus a newline.
  - Spell the file names out, as the module header says. Cite #73 only.
- **Verify:**
  - **The new test.** `cargo test -p terminus --all-features --test owner_only` passes, both cases included.
  - **Mutation 1.** Put `own_uid`'s create back to `std::fs::create_dir_all`. The existing-destination case fails on the config directory's mode. Restore it.
  - **Mutation 2.** Swap the new core function's `owner_only::create_dir_all` for `std::fs::create_dir_all`. The absent-destination case fails on the config directory's mode. Restore it.
  - **Other suites.** `cargo test -p terminus --all-features --test datamove` passes, and so does `cargo test -p terminus-core --all-features --test store_open --test config`.
  - **Old writer gone.** `git grep -n "data-location.tmp\|fn write_pointer" -- crates/` prints nothing.

### Task 3: Describe the pointer's modes, and name every owner_only site

- **Files:** THREAT-MODEL.md (the **Config directory.** paragraph; check the file-modes table, the **File modes.** paragraph and the `settings.json` and binary bullets), crates/terminus-core/src/owner_only.rs (the module header's site list)
- **Action:**
  - **`THREAT-MODEL.md`'s **Config directory.** paragraph.** Today it says the pointer is written with `std::fs::write` through `data-location.tmp`, at the umask mode, following a symlink at the temporary name. Replace that with what is now true:
    - `terminus data move` writes `data-location` 0600 through an exclusive temporary, `.data-location-<pid>-<n>.tmp`, and a rename (`core/store/open.rs`).
    - It creates the config directory 0700, when it learns the account's uid or when it writes the pointer, whichever comes first.
    - A config directory that already exists keeps its mode.
    - A move killed during the write can leave the temporary behind, and nothing deletes it.
  - **No table row.** The temporaries table covers files in the data directory, so the pointer's temporary is described in this paragraph and gets no row. That is the planner's answer to D-16's "if the pointer's temporary needs an entry".
  - **Other lines.** Read the **File modes.** paragraph, the table, the `settings.json` bullet (which names `create_temporary` in `bin/cmd/install/mod.rs`) and the binary bullet. Change only what is now wrong.
  - **The `owner_only.rs` header.** Its site list names the store, the ingest lock, the snapshot copy, the injection scratch and the export, and calls them "the two helpers below". Rewrite it to name every site that now creates through this module:
    - the store and the ingest lock;
    - the snapshot;
    - the injection state and decision writers;
    - the export;
    - install's temporaries and its stable-path link;
    - `data move`'s directories, copies, uid probe and `LOCK`;
    - the `data-location` pointer and the config directory.
    - Drop the count of helpers, and keep the std-alone paragraph.
- **Verify:**
  - **The paragraph.** `grep -n "std::fs::write\|data-location.tmp" THREAT-MODEL.md` prints nothing. The **Config directory.** paragraph names 0600, 0700 and `.data-location-<pid>-<n>.tmp`, and says nothing deletes a temporary left behind.
  - **AC2, `create_new`.** `git grep -n "create_new(true)" -- crates/terminus/src crates/terminus-core/src` hits only:
    - `owner_only.rs`;
    - `store/open.rs` inside `Store::open`;
    - lines inside `#[cfg(test)]` modules.
  - **AC2, symlinks.** `git grep -n "unix::fs::symlink" -- crates/terminus/src crates/terminus-core/src` hits only `owner_only.rs` and lines inside `#[cfg(test)]` modules.
  - **AC2, retry loops.** `git grep -n "AlreadyExists => continue" -- crates/terminus/src crates/terminus-core/src` hits nothing outside `owner_only.rs`.
  - **Tags.** `cargo test -p terminus-core --all-features --test planning_tags` passes.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices,** where CONTEXT left the call open:
  - The pointer's temporary is named `.data-location-{pid}-{n}.tmp`.
  - The core writer takes the destination and resolves the pointer path itself.
  - `THREAT-MODEL.md` describes that temporary in the config directory paragraph and adds no table row.
- **Plan shape:** this is plan 2 of 3. It runs after plan 1, which it builds on and shares `owner_only.rs` and `tests/owner_only.rs` with.
- **Deviation from CONTEXT's plan shape note.** CONTEXT says #73 and #99 touch separate files, but D-16 puts a `THREAT-MODEL.md` edit in each. This plan and plan 3 therefore share that file, and plan 3 runs after this one rather than beside it. The two edits are in different paragraphs.
- **Prior art:** hardin/phases/3/UAT.md (phase 3). `THREAT-MODEL.md` describes the temporaries as created 0600 and names the gaps it leaves, such as the unchecked parent directory. Task 3 keeps that style: it names the stray temporary nothing sweeps.
