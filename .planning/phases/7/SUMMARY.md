---
phase: 7
status: complete
completed: 2026-10-26
---

# Phase 7: Owner-only files and signing - Summary

Install, injection, snapshot, export and `terminus data move` now create every exclusive or owner-only file through the `owner_only` helpers (#73), and `scripts/sign-release.sh` carries its own copies of `valid_version`, `digest` and `read_manifest`, runs no code out of `install.sh`, and fails a test when those copies drift (#99).

## What shipped

- Exclusive 0600 create, a retrying exclusive create in file and symlink forms, an exclusive 0700 directory, and `replace` - `crates/terminus-core/src/owner_only.rs`
- Install's temporaries and symlink, the injection writers and decision reservation, the snapshot reservation and the export files moved onto those helpers - `crates/terminus/src/cmd/install/`, `crates/terminus-core/src/inject/`, `crates/terminus-core/src/store/snapshot.rs`, `crates/terminus/src/cmd/export.rs`
- The hook session-id check kept once - `crates/terminus-core/src/inject/mod.rs`
- `data move`'s probe, directories and copies on the helpers; the `data-location` pointer written 0600 and the config directory 0700 from core - `crates/terminus/src/cmd/data.rs`, `crates/terminus-core/src/store/`
- `install.sh`'s manifest parse as the function `read_manifest`, with no change in behaviour - `install.sh`
- A signer with no `eval`, its own `valid_version` and a verbatim `read_manifest` check of every manifest - `scripts/sign-release.sh`
- Tests that a signing run executes nothing from `install.sh` and that the signer's copies match it byte for byte - `crates/terminus/tests/sign_release.rs`
- The pointer's modes, every `owner_only` site, and the signer's own copies described - `THREAT-MODEL.md`, `docs/release.md`, `docs/design/0008-install-and-release.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 136460b | Add the exclusive creates to owner_only |
| 1 | 2 | 427e646 | Move install's temporaries and its symlink onto owner_only |
| 1 | 3 | e4a1bee | Create the snapshot reservation and export files through owner_only |
| 1 | 4 | 15c69ff | Add replace to owner_only |
| 1 | 5 | 2d53433 | Move the injection writers' temporaries and the decision reservation onto owner_only |
| 1 | 6 | ca718a2 | Keep the hook session-id check once |
| 2 | 1 | 946ed98 | Move data move's probe, directories and copies onto owner_only |
| 2 | 2 | ab3d3a4 | Write the data-location pointer 0600 and the config directory 0700 from core |
| 2 | 3 | 919200b | Describe the pointer's modes and name every owner_only site |
| 3 | 1 | 001c310 | Move install.sh's manifest parse into read_manifest |
| 3 | 2 | 9713ab9 | Give the signer its own valid_version and run nothing from install.sh |
| 3 | 3 | fc813ac | Prove a signing run executes nothing from install.sh |
| 3 | 4 | 3383d90 | Check every manifest with a copy of install.sh's read_manifest |
| 3 | 5 | 88a9d06 | Fail when the signer's copies differ from install.sh's |
| 3 | 6 | d7199f2 | Say the signer holds its own copies and runs no installer code |

## Deviations

None - plans executed as written.

## Open items

- Three SessionStart timing tests fail at the phase base 4a89b49 and after every plan: `brief` a_hundred_session_starts_stay_inside_the_budget_and_the_wall_clock (p99 about 22 ms) and `hook` every_event_exits_zero_with_an_empty_stdout_inside_the_budget and every_event_stays_inside_the_budget_with_the_redaction_knob_on (p99 about 21 to 36 ms), each against a 10 ms budget. The orchestrator reran them twice at 4a89b49 and twice at ca718a2 in clean copies with matching numbers. John accepted plan 1 with them red; they are not fixed by this phase.
- The working tree holds untracked files from another line of work (`credentials.rs`, `tests/judgment.rs` and others) whose tests do not compile, so `cargo clippy --all-targets`, `cargo test --workspace` and `planning_tags` fail in /code/terminus as it stands. Every plan ran clippy and the suite in a clean `git archive HEAD` instead.
- The shell umask here is 002; `owner_only::tests::only_the_leaf_directory_is_owner_only` and the 0755-parent test fail under 002 and pass under 022, as plan 1 states. Every run used umask 022.
- `install_script::a_missing_signature_is_refused::sh` failed once on a curl TLS verify of the local test server during plan 1 and passed on the immediate rerun and in the suite.

## Goal check

The commits plausibly deliver the goal. On #73, outside `owner_only.rs` and test code the only `.create_new(true)` left is `store/open.rs:128`'s create of `terminus.db`, which plan 2 names as the one exception (`crates/terminus/src/cmd/install/mod.rs:538` sits under `#[cfg(test)]` at line 510). No non-test code calls `std::os::unix::fs::symlink` (`crates/terminus/src/cmd/install/mod.rs:522` is the same test module). The `AlreadyExists` matches left in `data.rs:180`, `data.rs:389` and `install/mod.rs:92` map a helper's error to a message, not a retry loop. On #99, `grep eval scripts/sign-release.sh` prints nothing, and plan 3's report records the drift test failing on a one-character change in each copied function (88a9d06). The full suite was not green: the three SessionStart timing tests above fail at the base too, so this phase neither caused nor fixed them.
