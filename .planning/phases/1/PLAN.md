---
phase: 1
plan: 1
requirements:
  - "#86"
files:
  - scripts/sign-release.sh
  - crates/terminus/tests/sign_release.rs
---

# Phase 1: Signer version check - Plan

## Goal

`scripts/sign-release.sh` refuses every version `install.sh` refuses, because it runs `install.sh`'s own `valid_version`, read out of `install.sh` when it runs, and keeps no copy of the check (#86).

## Must be true when done

- Run with `v123456789012345678901.0.0` or `v18446744073709551616.0.0`, the signer exits non-zero and prints exactly `terminus: usage: sh scripts/sign-release.sh v<X.Y.Z>` on stderr, under both `/bin/sh` and dash, and calls neither `gh` nor `ssh-keygen`.
- Run with `v18446744073709551615.0.0`, the signer gets past the tag check: its first `gh` call is `release view` for that tag.
- `scripts/sign-release.sh` holds no definition of `valid_version`. The check it runs is the function it reads out of `install.sh`.
- Beside an `install.sh` that holds no `valid_version` function, or two, the signer exits non-zero with a message naming `install.sh`, and calls neither `gh` nor `ssh-keygen`.
- `install.sh` is byte for byte the same as on `main`, and `cargo test --workspace --all-features` passes.
- Issue #86 is closed by the merged change (human-verify: needs the merged PR on GitHub).

## Context

- Locked: D-01 the signer reads `valid_version` out of `install.sh` at run time, the way it reads `release_key`, and refuses to run unless there is exactly one; D-02 the signer's own copy is deleted; D-03 `install.sh` is not changed; D-04 an over-bound tag gets the same usage line every bad tag gets; D-05 the two refusals go into `a_bad_tag_is_refused`, plus one test that `18446744073709551615` still gets past the tag check.
- Pattern to follow: the `release_key=` block in `scripts/sign-release.sh`, which counts with `grep -c` and fails naming `$install_sh` with what it needs and what it found.
- Out of scope: `install.sh`, the wording of any existing message, any other check the signer copies or could share with `install.sh`, `docs/`, `CHANGELOG.md`.

## Tasks

### Task 1: Take valid_version from install.sh at run time

- **Files:** scripts/sign-release.sh (start at the `valid_version` function and the `release_key=` block)
- **Action:** Delete the signer's own `valid_version` function (D-02). The script keeps no copy of the check, just as it keeps no copy of the key.
  Read the function out of `$install_sh` and define it in the signer's shell. Do this after the existing `release_key=` block, so the presence check on `install.sh` and its message are reused unchanged, and before the first `gh release view`. Then call `valid_version "$version" || fail "$usage"` at that point. The `[ "$#" -eq 1 ]` check, the `v*` case and `version=${tag#v}` stay where they are. CONTEXT's flagged assumptions accept the result: a bad tag with no `install.sh` beside the script now gets the not-found message instead of the usage line.
  Refuse unless `install.sh` holds exactly one definition (D-01). Count every line that begins, after optional blanks, with `valid_version`, then optional blanks, then `(`. That way a definition counts whatever its spacing or indentation, and the indented calls, which pass an argument and have no paren, do not. Count with `grep -c`, as the `release_key=` block counts `^release_key=`, and end the assignment with `|| :` as that block does: `grep -c` exits 1 when it counts zero, and under `set -e` a bare assignment would end the script before the zero is reported. For zero or for more than one, call `fail` naming `$install_sh`, worded like the `release_key=` message: what it needs and what it found. Count every form, not only the exact one, for this reason: the shell runs the last definition it reads, so if a second definition were written in another form, `install.sh` would run that one while the cut took the first.
  The cut starts at the line that is exactly `valid_version() (` and ends at the first line after it that is exactly `)`, both lines included. If no line is exactly `valid_version() (`, or no closing line follows it, call `fail` naming `$install_sh`. Do the cut with the shell's own `while IFS= read -r` loop over `$install_sh`, joining the lines with newlines. Do not use sed or awk. The test fixture's `PATH` holds only `TOOLS` (mktemp, rm, grep, sha256sum) in `crates/terminus/tests/sign_release.rs`, and `.github/workflows/test.yml` checks a fixed tool list, so a new tool would mean changing both. Then `eval` the cut text, so the function the signer runs is `install.sh`'s own, every line of it.
  Leave these unchanged: the usage message and every other message (D-04), including the not-found message's "the release key is read from it"; the `release_key=` block; the `gh` and `ssh-keygen` flow; and `install.sh` (D-03). The header comment line "The public key is read from install.sh's release_key line; this script keeps no copy." becomes one line saying the version check is read from `install.sh`'s `valid_version` as well, and the script keeps no copy of either. The script must still run under `/bin/sh` and dash with `set -eu`.
- **Verify:**
  - `grep -c -E '^[[:space:]]*valid_version[[:space:]]*\(' scripts/sign-release.sh` prints `0`.
  - `sh -n scripts/sign-release.sh` and `dash -n scripts/sign-release.sh` both exit 0.
  - `cargo test -p terminus --test sign_release` passes, with every test run as `::sh` and `::dash`. This shows the good draft is still signed and published using the cut function, and every existing bad tag still gets exactly the usage line.
  - `git diff --exit-code main -- install.sh` exits 0.
  - By hand, under both `sh` and `dash`, make a scratch folder outside the repo holding `scripts/sign-release.sh` copied from the repo. Beside it, put an `install.sh` built from the repo's in each of three ways: (a) the function removed, `sed '/^valid_version() ($/,/^)$/d' install.sh`; (b) the function twice, the repo file followed by the output of `sed -n '/^valid_version() ($/,/^)$/p' install.sh`; (c) cut off right after the definition line, `sed '/^valid_version() ($/q' install.sh`. First on `PATH`, put a folder holding stand-in `gh` and `ssh-keygen` scripts that append their arguments to a log file and exit 1. That way the real `gh`, which is logged in to the real release, is never reached. From the scratch folder, run `env -i PATH=<stand-ins>:/usr/bin:/bin HOME=<scratch> <shell> scripts/sign-release.sh v0.1.0`. Each of the six runs must exit non-zero with exactly one stderr line, which starts `terminus: ` and contains `install.sh`, and the log file must not exist.

### Task 2: Refuse a version part over 20 digits or above u64

- **Files:** crates/terminus/tests/sign_release.rs (start at `a_bad_tag_is_refused`)
- **Action:** Add `v123456789012345678901.0.0` (a 21-digit major) and `v18446744073709551616.0.0` (one above u64's largest value, 18446744073709551615) to the list of tags `a_bad_tag_is_refused` loops over (D-05). Its existing assertions cover them unchanged: a non-zero exit, stderr exactly `terminus: usage: sh scripts/sign-release.sh v<X.Y.Z>` and a newline (D-04), and no `gh` or `ssh-keygen` call. `both_shells!` already runs the test as `::sh` and `::dash`. `fixture` already writes the shipped `install.sh` with only the key swapped, so the cut takes the real function and the fixture needs no change (CONTEXT's flagged assumption, confirmed by reading `fixture`). Do not reformat the rest of the file. It already has one rustfmt diff from before this phase, and a whole-file reformat would bury the change.
- **Verify:**
  - `cargo test -p terminus --test sign_release a_bad_tag_is_refused` runs `a_bad_tag_is_refused::sh` and `a_bad_tag_is_refused::dash`, and both pass.
  - The test pins the bound. Copy the current `scripts/sign-release.sh` to a scratch file, replace it with main's copy (`git show main:scripts/sign-release.sh > scripts/sign-release.sh`) and rerun the same command. Both runs fail, and the failure names one of the two new tags, because main's copy passes them on to `gh`. Then copy the saved file back, and Task 1's first check, `grep -c -E '^[[:space:]]*valid_version[[:space:]]*\(' scripts/sign-release.sh`, prints `0` again, which shows the restored file is the new signer and not main's.

### Task 3: Show the largest legal part still gets past the tag check

- **Files:** crates/terminus/tests/sign_release.rs
- **Action:** Add one test that runs the fixture with `v18446744073709551615.0.0` and asserts that the first `gh` call is `release view` for that tag: its first three arguments are `release`, `view` and the tag (D-05). Register it in `both_shells!` so it runs as `::sh` and `::dash`, like every other test that runs the script. That is the planner's choice: D-05 does not say which shells, and the file's module comment says each test runs once per shell. Assert nothing about what follows. The fixture's manifests name 0.1.0, so the run is refused later, and that refusal is not what this test checks. Whether the stand-in `gh` answers draft or published is up to the executor. Put a doc comment above the test in the file's style, saying what it shows. Do not reformat the rest of the file, as in Task 2.
- **Verify:**
  - `cargo test -p terminus --test sign_release` passes and lists the new test as `::sh` and `::dash`.
  - The test pins the boundary. Change its tag by hand to `v18446744073709551616.0.0` and rerun: both runs fail, because no `gh` call is made. Put the tag back.
  - `cargo test --workspace --all-features` passes.
  - `git diff --exit-code main -- install.sh` exits 0.

## Notes

- Closing #86 needs the PR that carries this phase to say `Closes #86`, which the orchestrator writes. It is human-verify once merged.
- The refusal when there is no definition or two (AC4) is checked by hand in Task 1's Verify and is not a permanent test, because D-05 names exactly the tests this phase adds. A permanent test for it would be a follow-up for John to decide.
- `sh -n` and `dash -n` on the release script follow the shell check in `v0.0.1/phases/6/UAT.md`, phase 6.
- Left for John, not in this plan: `docs/release.md` "How a release is signed" step 1 says the signer reads only the key from `install.sh`. After this phase that step leaves out the version check, though nothing in it is wrong. That page mirrors the signer step by step (`v0.0.1/phases/9/UAT.md`, phase 9), and Phase 7's sweep checks `docs/` against the code. `CHANGELOG.md` gets no entry, because CONTEXT does not name it.
