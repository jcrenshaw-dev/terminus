---
phase: 7
plan: 1
requirements: [OWN-01]
files:
  - crates/terminus-core/src/owner_only.rs
  - crates/terminus/src/cmd/install/mod.rs
  - crates/terminus/src/cmd/install/binary.rs
  - crates/terminus-core/src/store/snapshot.rs
  - crates/terminus/src/cmd/export.rs
  - crates/terminus-core/src/inject/state.rs
  - crates/terminus-core/src/inject/decision.rs
  - crates/terminus-core/src/inject/mod.rs
  - crates/terminus/tests/owner_only.rs
---

# Phase 7: Owner-only files and signing - Plan 1 of 3 (the owner_only helpers, and install, injection, snapshot and export on them)

## Goal

Every exclusive or owner-only create that install, the injection writers, the snapshot and the export build by hand goes through one set of `owner_only` helpers, and the check that keeps a hook's session id out of a path exists once (#73).

## Must be true when done

- `crates/terminus-core/src/owner_only.rs` still has `DIR_MODE`, `FILE_MODE`, `create_dir_all` and `options()`. Beside them it now has:
  - a single exclusive 0600 file create;
  - an exclusive create that retries under a new name, in a file form and a symlink form, using the caller's names and attempt limit;
  - an exclusive 0700 directory;
  - `replace`, which writes bytes through an exclusive 0600 temporary, renames it into place, removes it on failure and does no fsync.
  - Each helper has its own unit tests.
- In `install/mod.rs`, `install/binary.rs`, `inject/state.rs`, `inject/decision.rs`, `store/snapshot.rs` and `cmd/export.rs`:
  - `.create_new(true)` and `std::os::unix::fs::symlink` appear only inside `#[cfg(test)]` modules;
  - no loop retries on `AlreadyExists`.
- Every temporary keeps its name:
  - a stale `.terminus-install-<pid>` or `.settings.json.terminus-<pid>` symlink is stepped over, and its target is left untouched;
  - the drain's sweep still deletes old `.inject-<pid>-<n>.tmp` and `.decision-<pid>-<n>.tmp` files.
- Every file keeps its mode:
  - install's temporaries are 0600 from the open;
  - the stored version file is 0755;
  - an existing 0644 `settings.json` stays 0644, and its backup is 0600;
  - the injection state file, decision records, snapshots and export files are 0600.
- `MIN_NAME` and `MAX_NAME` are each defined once in `crates/`. The tests that refuse a hostile hook session id still pass, for the state file and for the decision file.
- `cargo test --workspace --all-features` passes and `cargo clippy --locked --all-targets --all-features` reports no warning.

## Context

- **Locked decisions** (`.planning/phases/7/CONTEXT.md`):
  - D-02 for the sites in this plan.
  - D-03: the ingest lock, `Store::open`'s create of `terminus.db` and `data move`'s `LOCK` keep `options()`.
  - D-04: each site keeps its names and its attempt limit.
  - D-05: `replace` does no fsync. `json_file.rs` and `binary.rs` `store` keep their own fill.
  - D-06: the binary's 0755 stays a chmod in `binary.rs`. The versions folder and `~/.local/bin` keep plain `create_dir_all`.
  - D-07: one session-id check.
  - D-16: the comment lines in these files.
  - D-17.
- **In the other plans:** `data move` and the pointer are plan 2, and so are the `owner_only.rs` header and `THREAT-MODEL.md`. The signer is plan 3.
- **Out of the phase:** the ingest lock, `Store::open`'s create, `LOCK`, a sweep for a stray pointer temporary, `cargo fmt` over the workspace, and #109 to #113.
- **No new cost.** `owner_only` stays std-only (its header: no `libc`, no `nix`). The hook path gets no new work: a state save and a decision write make the same syscalls they make today, with no fsync and no extra stat.
- **Comments and tests** cite #73 only, never a requirement id or a decision tag (`crates/terminus-core/tests/planning_tags.rs`). Rewrap comments by hand. Do not run `cargo fmt`.
- **Commands.** Run every `cargo test` with `--all-features`: some test targets need `testkit` and are skipped without a word when it is off (`crates/terminus/Cargo.toml`).

## Tasks

### Task 1: Add the exclusive creates to owner_only

- **Files:** crates/terminus-core/src/owner_only.rs (beside `create_dir_all` and `options`)
- **Action:**
  - **Three helpers.** Add three public helpers beside the existing four items. Build them on `options()` and `DirBuilderExt::mode`, the way the existing ones are built.
    - **A single exclusive create.** It opens a new file for writing with `create_new` at `FILE_MODE` and returns the `File`. Anything already at the path is `AlreadyExists` and is never followed, a symlink included. Plan 1 task 3 and plan 2 task 1 use it for the snapshot reservation, the export files and `data move`'s `copy_file`.
    - **The retry.** It takes a directory, an attempt limit and the file name for each attempt number, counting from 0, and tries each name in turn.
      - `AlreadyExists` moves on to the next name. Any other error returns at once.
      - On success it returns the path it landed at and what the create made.
      - When every name was taken, it returns an error of kind `AlreadyExists`, so a caller can tell exhaustion from every other failure and word its own message.
      - It comes in two forms. The file form makes each attempt with the single exclusive create. The symlink form makes each attempt a symlink to a given target with `std::os::unix::fs::symlink`, which also fails on a taken name.
      - The caller's name function decides every name, so each site keeps the names D-04 lists. Install, for one, keeps `{prefix}{pid}` and then `{prefix}{pid}-{n}`.
      - Whether the two forms share one private loop is the executor's choice. By the end of plan 2, no non-test code outside this file calls `std::os::unix::fs::symlink`.
    - **An exclusive directory.** It creates missing parents at the process default, exactly as `create_dir_all` does. Then it makes the leaf with one exclusive mkdir at `DIR_MODE`. Anything already at the leaf, a directory or a symlink to one, is `AlreadyExists`. This is `data.rs`'s `create_destination` shape, moved here; its callers move in plan 2.
  - **Names.** #73 proposes `create_temporary` for the retry and `create_dir_new` for the directory. Use those names unless they clash.
  - **No mode option** on any helper (D-06). The binary's 0755 stays a chmod in `binary.rs`.
  - **Doc comments.** Each one gives the reasons once, here, so later tasks can drop the copies their callers carry (#73: "each copy repeats the same explanation of `create_new`"):
    - The mode rides the creating syscall, for the reason the module header gives.
    - `create_new` is `O_EXCL`, so a stale symlink at a name is never followed and truncated.
    - A taken name is answered with another name, never reused.
  - **Leave alone:** `create_dir_all`, `options()` and the module header. Plan 2 rewrites the header's site list once every site has moved.
- **Verify:**
  - **New tests.** `cargo test -p terminus-core --all-features --lib owner_only::` passes. The five existing tests are unchanged, and new tests show:
    - The single create lands 0600. It refuses an existing file, and a symlink to a file, with `AlreadyExists`. The symlink's target keeps its bytes.
    - With a file at the attempt-0 name and a symlink to a victim at the attempt-1 name, the file form lands at the attempt-2 name, 0600. The victim's bytes are unchanged.
    - With all three names taken and a limit of 3, it returns kind `AlreadyExists`.
    - Into a directory that does not exist, it returns `NotFound` on the first attempt.
    - The symlink form leaves a link at the first free name, and its `read_link` is the target.
    - The exclusive directory leaves a 0700 leaf with a parent it created at 0755, under the umask 022 `only_the_leaf_directory_is_owner_only` already assumes. It refuses an existing directory, and a symlink to one, with `AlreadyExists`.
  - **Clippy.** `cargo clippy --locked --all-targets --all-features` reports no warning.

### Task 2: Move install's temporaries and its symlink onto owner_only

- **Files:** crates/terminus/src/cmd/install/mod.rs (`create_temporary`, `temporary`), crates/terminus/src/cmd/install/binary.rs (`link`, `TEMPORARY`)
- **Action:**
  - **`create_temporary`** keeps its signature. It stays the one place install builds its temporary names and turns a failure into a `Failure`.
    - It calls the retry's file form with 64 attempts, naming attempt 0 `{prefix}{pid}` and attempt n `{prefix}{pid}-{n}`, exactly as `temporary` does today. The names are D-04's: `tests/install.rs` plants `.terminus-install-{pid}` and `.settings.json.terminus-{pid}`.
    - Exhaustion keeps today's message, naming the directory, the 64 and `'{prefix}{pid}...'`.
    - Any other error keeps "could not be created". It names the directory if the helper's error no longer carries the path.
  - **The symlink.** The generic `temporary` closure form goes away. `binary.rs` `link` gets the symlink form with the same names, the same attempts and the same messages. The naming and the `Failure` wording live once in `install/mod.rs` for both forms, and `binary.rs` no longer calls `std::os::unix::fs::symlink`.
  - **Keep everything else** (D-05, D-06):
    - `binary.rs` `store` and `json_file.rs` `write_atomically` and `backup` still call `create_temporary`, and each keeps its own fill. `copy_into` chmods 0755 and calls `sync_all`. `fill` calls `sync_all` and copies the live file's mode. `copy_through` calls `sync_all`.
    - `json_file.rs` is not edited. `create_temporary`'s signature does not change, and its "see `super::create_temporary`" comment still points at a real item.
    - `binary.rs` `create_dir` stays plain `std::fs::create_dir_all`.
    - `copy_into` and `copy_through` are not merged. D-06 leaves that to the planner, and no criterion needs it.
  - **Doc comments** (D-16). Rewrite the ones on `install/mod.rs`'s temporaries. The `O_EXCL` and stale-symlink explanation now lives on the `owner_only` helper, so these say what install adds:
    - its names, and why they are kept (the stale-name test);
    - the 64 attempts;
    - that a caller needing another final mode sets it before the rename, as `binary::copy_into` sets 0755.
    - Trim the comment above `store`'s call the same way.
  - **Prior art:** hardin/phases/3/UAT.md (phase 3). Temporaries open at 0600. After install, an existing 0644 `settings.json` is still 0644, its backup 0600 and the stored version file 0755.
- **Verify:**
  - **Install tests.** `cargo test -p terminus --all-features --test install` passes, including `a_stale_temporary_name_is_stepped_over_rather_than_followed` and the mode assertions.
  - **Unit tests.** `cargo test -p terminus --all-features --lib cmd::install::` passes, including `a_temporary_is_owner_only_from_the_open`.
  - **Other install paths.** `cargo test -p terminus --all-features --test uninstall --test install_script` passes.
  - **No hand-built creates.** `grep -n "create_new\|fs::symlink\|AlreadyExists => continue" crates/terminus/src/cmd/install/mod.rs crates/terminus/src/cmd/install/binary.rs` shows hits only inside `mod tests`. A `kind() == AlreadyExists` test that picks the exhaustion message is expected and allowed.
  - **`json_file.rs` untouched.** `git diff --stat` does not list `crates/terminus/src/cmd/install/json_file.rs`.

### Task 3: Create the snapshot reservation and the export files through owner_only

- **Files:** crates/terminus-core/src/store/snapshot.rs (`take`), crates/terminus/src/cmd/export.rs (`write_owner_only`)
- **Action:**
  - **`snapshot.rs` `take`.** The reservation `crate::owner_only::options().write(true).create_new(true).open(&temp)` becomes task 1's single exclusive create.
    - Everything else stays: removing a stale temporary first, dropping the handle at once, `VACUUM INTO`, the rename, and `Error::io(&temp, e)`.
    - No retry here. The fixed name, removed and then created exclusively, is the shape this site keeps.
    - The "created here, EMPTY and 0600" comment stays, because it is the reason `VACUUM INTO` writes into a file this code made.
  - **`export.rs` `write_owner_only`.** It becomes the single exclusive create followed by `write_all`.
    - Whether it stays as a two-line wrapper or its callers call the helper directly is the executor's choice.
    - The export-specific reason stays where the create is: `refuse_if_occupied` already found the destination empty, so a taken name means something else is writing into it, and truncating that file is what export says it never does.
- **Verify:**
  - **Snapshot.** `cargo test -p terminus-core --all-features --test snapshot` passes.
  - **Export and modes.** `cargo test -p terminus --all-features --test owner_only --test lifecycle --test tighten` passes, including `an_export_directory_and_every_file_in_it_are_owner_only` and `exporting_into_an_occupied_directory_refuses_and_overwrites_nothing`.
  - **No hand-built creates.** `grep -n "create_new" crates/terminus-core/src/store/snapshot.rs crates/terminus/src/cmd/export.rs` prints nothing.

### Task 4: Add replace to owner_only

- **Files:** crates/terminus-core/src/owner_only.rs
- **Action:**
  - **What it does.** Add a public `replace`, the name #73 and D-05 give it. It writes the given bytes to the target path through a temporary in the target's directory, made with task 1's retry file form, then renames the temporary over the target. On any failure it removes the temporary and returns the error.
  - **No fsync** (D-05). Every hook state save would otherwise pay a disk flush on the 50 ms hook path. The doc comment says so, and says the rename alone gives a reader the old file or the new one, never a prefix.
  - **The temporary's name.** Besides the path and the bytes, it takes the prefix and suffix the temporary's name is built from: `{prefix}{pid}-{n}{suffix}`, 64 attempts. CONTEXT writes it as `replace(path, bytes)`. The name parts are needed because the state file's temporaries must stay `.inject-{pid}-{n}.tmp`, the names the drain's sweep in `feedback/mod.rs` matches (D-04). Plan 2's pointer gets its own name.
  - **No directory creation.** It does not create the directory; the caller does, with `create_dir_all`. The state file needs two levels created and the pointer one.
- **Verify:**
  - **New tests.** `cargo test -p terminus-core --all-features --lib owner_only::` passes, with new tests showing:
    - A new target lands 0600 and holds exactly the bytes.
    - An existing target is replaced by the rename: its inode changes, the check `a_write_replaces_the_target_rather_than_rewriting_it_in_place` in `tests/inject_state.rs` makes for the state file. Afterwards the directory holds only the target.
    - A symlink to a victim at the attempt-0 temporary name is stepped over, and the victim's bytes are unchanged.
    - A rename that fails, because the target is a non-empty directory, returns an error and leaves no file with the prefix in the directory.
  - **Clippy.** `cargo clippy --locked --all-targets --all-features` reports no warning.

### Task 5: Move the injection writers' temporaries and the decision reservation onto owner_only

- **Files:** crates/terminus-core/src/inject/state.rs (`write_atomically`, `create_temporary`), crates/terminus-core/src/inject/decision.rs (`write_file`, `create_temporary`, `ATTEMPTS`)
- **Action:**
  - **`state.rs`.** `write_atomically` keeps its two `owner_only::create_dir_all` calls, the comment on why there are two, and the pretty-printed serialization. It then hands the bytes to `replace` with `TEMPORARY_PREFIX` and `TEMPORARY_SUFFIX`, and still returns `None` on any failure.
    - `state.rs`'s `create_temporary` is removed.
    - The doc on `write_atomically` keeps what is specific to this file: the abandoned hook thread, and why no fsync is right for per-session scratch. It drops what now lives on `replace`.
    - Beside the call, keep the point that the temporary can never collide with a session's own file: it starts with a dot and ends in `.tmp`, and the session-id allow-list admits neither.
  - **`decision.rs`.**
    - `create_temporary` becomes the retry file form, with names `{TEMPORARY_PREFIX}{pid}-{n}{TEMPORARY_SUFFIX}` and 64 attempts, and the function is removed.
    - The reservation in `write_file` becomes the retry file form, with names `{stem}-{pid}-{stamp}-{n}.json` and 64 attempts.
      - The reserved handle is dropped and the temporary is renamed over the reservation.
      - A failed rename removes the reservation and the temporary and returns `None` without trying another name.
      - Exhaustion, or any other create error, removes the temporary and returns `None`. These are the outcomes the loop has today.
    - A decision cannot use `replace`. Its target is a new reserved name per event, never an existing file to replace.
    - `ATTEMPTS` stays only if something still reads it.
  - **Doc comments** (D-16). The two `create_temporary` doc comments go with the functions. What is specific to each stays at its call: what the temporary holds, and why it is 0600 from the create. The generic `O_EXCL` explanation is not repeated.
  - **Keep** `TEMPORARY_PREFIX`, `TEMPORARY_SUFFIX` and their "the drain's sweep matches on these" comments. `feedback/mod.rs` `sweep` reads them.
  - **Leave the session-id check alone.** It is task 6.
- **Verify:**
  - **Core tests.** `cargo test -p terminus-core --all-features --test inject_state --test feedback` passes, including `a_pass_sweeps_the_writers_old_temporaries_and_nothing_else` and `a_write_replaces_the_target_rather_than_rewriting_it_in_place`.
  - **Inject unit tests.** `cargo test -p terminus-core --all-features --lib inject::` passes, including `a_record_and_its_directory_are_owner_only` and `two_prompts_of_one_session_land_as_two_files`.
  - **Binary tests.** `cargo test -p terminus --all-features --test owner_only --test decisions --test suppression` passes.
  - **No hand-built creates.** `grep -n "create_new\|AlreadyExists => continue" crates/terminus-core/src/inject/state.rs crates/terminus-core/src/inject/decision.rs` prints nothing.

### Task 6: Keep the hook session-id check once

- **Files:** crates/terminus-core/src/inject/mod.rs, crates/terminus-core/src/inject/state.rs (`MIN_NAME`, `MAX_NAME`, `path`, `file_name`), crates/terminus-core/src/inject/decision.rs (`MIN_NAME`, `MAX_NAME`, the comment above them, `file_stem`, `write_file`, the `MAX_NAME + 1` in `a_hostile_session_id_reaches_no_filesystem_call`), crates/terminus/tests/owner_only.rs (the doc comment on `SESSION_ID`)
- **Action:**
  - **Where the check lives.** Put it in `inject/mod.rs`. D-07 leaves the place to the planner, and the parent of both writers means neither owns the other's defence.
  - **What moves there:**
    - `MIN_NAME` 8 and `MAX_NAME` 64, with `state.rs`'s doc comment on why the window is wider than a uuid.
    - One crate-internal function that takes a session id and returns it checked, or nothing. It admits a byte length within `MIN_NAME..=MAX_NAME` with every byte an ASCII hex digit or `-`, which is what both copies admit today.
    - Its doc keeps the allow-list reasoning both copies carry: the list of characters a session id is made of is closed, while the list of dangerous spellings is open-ended and per-platform; and byte length is character length because every admitted byte is ASCII. It also keeps the rule that the check runs before any path is built.
  - **Callers.**
    - `state.rs` adds `.json` to the checked id. `decision.rs` uses it as the file stem.
    - Remove both copies' constants and filters, and the comment in `decision.rs` that defends keeping two copies (D-07).
    - A child module can read a private item of `inject/mod.rs`, so the decision test's `MAX_NAME + 1` names the moved constant.
  - **The test comment.** The doc comment on `SESSION_ID` in `tests/owner_only.rs` says the allow-list is `inject::state`'s, "mirrored in `inject::decision`". It now names the one check in the inject module (D-16).
  - **No behaviour change.** Every id admitted today is admitted, and every id refused today is refused.
- **Verify:**
  - **One definition.** `git grep -n "const MIN_NAME\|const MAX_NAME" -- crates/` prints exactly two lines, both in `crates/terminus-core/src/inject/mod.rs`.
  - **Refusal tests.** `cargo test -p terminus-core --all-features --test inject_state a_session_id_that_could_name_a_path_creates_nothing_anywhere` passes, and so does `cargo test -p terminus-core --all-features --lib inject::decision::tests::a_hostile_session_id_reaches_no_filesystem_call`.
  - **No mirror left.** `grep -in "mirrored" crates/terminus-core/src/inject/decision.rs crates/terminus/tests/owner_only.rs` prints nothing.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices,** where CONTEXT left the call open:
  - **The retry helper** has a file form and a symlink form rather than taking the create as a closure. It builds names from a caller-supplied function of the attempt number, so every site keeps the names D-04 lists.
  - **`replace`** takes the temporary's prefix and suffix besides `(path, bytes)`. D-04 keeps the state temporaries' names, and they cannot be derived from the target's name.
  - **`copy_into` and `copy_through`** are not merged.
  - **The session-id check** lives in `inject/mod.rs`.
- **Plan shape:** this is plan 1 of 3.
  - Plan 2 builds on this plan's helpers and `replace`, and shares `owner_only.rs` and `tests/owner_only.rs` with it, so it runs after this one.
  - Plan 3 (#99) touches none of this plan's files.
- **Prior art:** hardin/phases/3/UAT.md (phase 3), the modes install leaves (task 2).
