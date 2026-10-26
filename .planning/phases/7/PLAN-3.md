---
phase: 7
plan: 3
requirements: [OWN-02]
files:
  - install.sh
  - scripts/sign-release.sh
  - crates/terminus/tests/sign_release.rs
  - THREAT-MODEL.md
  - docs/release.md
  - docs/design/0008-install-and-release.md
---

# Phase 7: Owner-only files and signing - Plan 3 of 3 (a signer that runs no installer code)

## Goal

Signing a release runs no code read out of `install.sh`, and a CI test fails when the signer's copies of the shared checks differ from `install.sh` (#99).

## Must be true when done

- `scripts/sign-release.sh` contains no `eval`. It reads only the `release_key=` value out of `install.sh`.
- A signing run whose `install.sh` has a `valid_version` that creates a file leaves no such file.
- `install.sh`'s manifest parse is the function `read_manifest`, with no change in behaviour. Every test in `crates/terminus/tests/install_script.rs` passes with no expected value changed.
- The signer checks each manifest with a verbatim copy of `read_manifest`, so a manifest the installer refuses is refused before anything is signed.
- A test under `cargo test` passes on the tree and fails when the signer's `valid_version`, `digest` or `read_manifest` differs from `install.sh`'s by one character.
- `THREAT-MODEL.md`, `docs/release.md` and `docs/design/0008-install-and-release.md` describe the signer's own copies and the test, and nothing says it runs `install.sh`'s code.
- `cargo test --workspace --all-features` passes and `cargo clippy --locked --all-targets --all-features` reports no warning.

## Context

- **Locked decisions** (`.planning/phases/7/CONTEXT.md`):
  - D-01: the manifest parse becomes a function in `install.sh`, and the signer carries a verbatim copy.
  - D-12: the signer's `valid_version` is today's, with the 20-digit u64 bound.
  - D-13: the `release_key=` read stays, and so does `sign_release_holds_no_copy_of_the_key`.
  - D-14: the drift test is in Rust, cuts each function as text, and runs under `test.yml`'s `cargo test --workspace`.
  - D-15: a side-effect regression test and a no-`eval` text test.
  - D-16: the signer's header and comments, and the docs.
  - D-17.
- **Patterns:**
  - `install_sh_holds_the_same_marker` in `crates/terminus/src/cmd/install/binary.rs` and `install_sh_pins_the_release_key_once` in `crates/terminus/tests/install_script.rs` read a script as text and compare one piece.
  - `sign_release.rs`'s `fixture`, `both_shells!` and `assert_refused_before_signing` are the shape of a signer run.
- **Out of the phase:** copying the release key into the signer, and #109 to #113.
- **Comments and tests** cite #99 only (`crates/terminus-core/tests/planning_tags.rs`). Do not run `cargo fmt`, and do not run `rustfmt` on `sign_release.rs`, which already carries rustfmt drift (D-17).
- **Commands.** `dash` and `ssh-keygen` are installed. Run every `cargo test` with `--all-features`.

## Tasks

### Task 1: Move install.sh's manifest parse into a function

- **Files:** install.sh (the manifest parse in the final brace group, from the `IFS= read -r version_line` group to the `manifest is not exactly two UTF-8 lines` check)
- **Action:**
  - **Move it unchanged** into a function named `read_manifest`. D-01 leaves the name to the planner. Define it with the other functions above the final brace group, directly after `digest`.
  - **The body**, in today's order and with today's messages:
    - the `{ ... }` read group;
    - the `version ` and `sha256 ` prefix cases;
    - `valid_version`;
    - the hex and length checks;
    - the comment on shells that drop NUL bytes;
    - the exact-text digest compare.
    - Give it a `{ }` body, not a subshell, because it sets `version` and `sha256` for the caller.
  - **The one change.** It takes the manifest's path as its one argument, and reads `"$1"` where the inline code reads `"$scratch/manifest"`. That is what lets the signer carry the same text (D-01). The brace group calls `read_manifest "$scratch/manifest"` where the parse was, before `verify_download`.
  - **Boundary lines** for task 5's drift test (D-14):
    - the opening line is exactly `read_manifest() {`;
    - the closing line is exactly `}` at column 0;
    - no line between them is exactly `}`. The inner read group's closing line stays indented.
  - **No change in behaviour:** the same refusals, in the same order, with the same messages.
  - **Prior art:** phases/10/UAT.md (phase 10). Pushing v0.1.0 leaves manifests that are exactly `version 0.1.0` and `sha256 <hash>`, each line ending in a newline. That is the shape this function accepts.
- **Verify:**
  - **AC6.** `cargo test -p terminus --all-features --test install_script` passes, and `git diff --stat -- crates/terminus/tests/install_script.rs` is empty.
  - **Syntax.** `sh -n install.sh` and `dash -n install.sh` both exit 0.
  - **Placement.** `grep -n '^read_manifest() {$' install.sh` prints one line, whose line number is lower than that of the final brace group's opening `{`, so the function is defined before the group that calls it. `grep -n 'scratch/manifest' install.sh` shows the `curl`, the `ssh-keygen` and the one call, and no `read`.
  - **Neighbour tests.** `cargo test -p terminus --all-features --lib cmd::install::binary::tests::install_sh_holds_the_same_marker` passes.

### Task 2: Give the signer its own valid_version, and run nothing from install.sh

- **Files:** scripts/sign-release.sh (the header, the `valid_version` read loop and `eval` after the `release_key` check, the comment above them)
- **Action:**
  - **Add `valid_version`** beside `fail` and `digest`, as a verbatim copy of `install.sh`'s function as it is today. That is the one with the 20-digit u64 bound, not the copy removed in 6a75972 (D-12). It has the same opening line, `valid_version() (`, and the same closing `)`.
  - **Remove:**
    - the `grep -c` count of `valid_version`, and its "needs exactly one valid_version function" refusal;
    - the read loop;
    - the "needs a valid_version() ( line" refusal;
    - the `eval`.
  - **Keep:**
    - `valid_version "$version" || fail "$usage"`, because `a_bad_tag_is_refused` pins that message;
    - the `release_key=` read exactly as it is (D-13).
  - **Comments** (D-16). Rewrite the header's last two lines and the comment above the version check to say what is now true:
    - the key is read from `install.sh`'s `release_key=` line as a value;
    - `valid_version` and `digest` are copies of `install.sh`'s, so the signer refuses the versions the installer refuses without running any code out of `install.sh`.
  - **The string `eval`** appears nowhere in the script, not even in a comment. Task 3's text test fails on it.
- **Verify:**
  - **Signer tests.** `cargo test -p terminus --all-features --test sign_release` passes on both shells, including `a_bad_tag_is_refused`, `the_largest_u64_part_passes_the_tag_check` and `sign_release_holds_no_copy_of_the_key`.
  - **Removed.** `grep -n "eval\|while IFS= read -r text\|valid_version function\|valid_version() ( line" scripts/sign-release.sh` prints nothing.
  - **Verbatim.** `diff <(sed -n '/^valid_version() ($/,/^)$/p' install.sh) <(sed -n '/^valid_version() ($/,/^)$/p' scripts/sign-release.sh)` prints nothing, and the first range is not empty.
  - **Syntax.** `sh -n scripts/sign-release.sh` and `dash -n scripts/sign-release.sh` both exit 0.

### Task 3: Prove a signing run executes nothing from install.sh

- **Files:** crates/terminus/tests/sign_release.rs (model: `fixture`, `both_shells!`, `sign_release_holds_no_copy_of_the_key`)
- **Action:** Add two tests (D-15), citing #99.
  - **The side-effect test** runs under both shells and is added to the `both_shells!` list.
    1. Call `fixture(shell)`.
    2. Rewrite the fixture's `install.sh` so that the line after `valid_version() (` creates a file under the fixture root. Use a shell builtin redirection such as `: > '<path>'`: the fixture's `PATH` holds only `TOOLS`, so there is no `touch`.
    3. Assert the rewrite changed the text, so the test cannot pass vacuously.
    4. Run the signer with `TAG`. Assert it succeeds and the file does not exist.
  - **The text test** is a plain `#[test]`: `scripts/sign-release.sh` does not contain the string `eval`.
- **Verify:**
  - **Pass.** `cargo test -p terminus --all-features --test sign_release` passes, the new test under both `::sh` and `::dash` included.
  - **Falsify.**
    1. Put `scripts/sign-release.sh` back to its version before task 2, for example with `git show <task 1's commit>:scripts/sign-release.sh > scripts/sign-release.sh`.
    2. Both new tests fail. The side-effect test fails because the file appears.
    3. Restore with `git checkout -- scripts/sign-release.sh`. The tests pass again, and `git status --short scripts/` is clean.

### Task 4: Check every manifest with the installer's own read_manifest

- **Files:** scripts/sign-release.sh (the per-binary check loop: the `grep -q -x -F -e "sha256 $actual"` check and the exact-text compare under it, and the header), crates/terminus/tests/sign_release.rs (model: `a_manifest_naming_another_version_is_refused`)
- **Action:**
  - **The copy.** Copy `install.sh`'s `read_manifest` verbatim into the signer, beside `valid_version` and `digest` (D-01).
  - **The loop.** Replace the `grep` and the exact-text compare with three steps, in this order:
    1. Run `read_manifest` on the binary's manifest.
    2. Refuse when the manifest's `sha256` is not the binary's actual digest, with today's "SHA-256 mismatch" message.
    3. Refuse when the manifest's version is not the tag's, with today's message, "not exactly the lines version <tag's version> and sha256 <actual>".
    - `a_binary_that_does_not_match_its_manifest_is_refused` and `a_manifest_naming_another_version_is_refused` pin how both messages start, and their expected values do not change.
  - **The tag's version.** `read_manifest` assigns `version`, `sha256`, `version_line`, `digest_line`, `extra`, `manifest_digest` and `text_digest`. Before the loop, hold the tag's version under a name it does not assign, and use that name in the compare and the messages.
  - **Refusals.** A manifest `install.sh` refuses is now refused with `install.sh`'s own message, through `fail`, before anything is signed.
  - **The header** now lists the manifest check among the copies.
  - **The wiring test**, under both shells, cites #99:
    - The draft has one manifest with a third line after its two correct ones.
    - It is refused before signing (`assert_refused_before_signing`), with stderr exactly `terminus: manifest has content after line 2\n`.
    - Today's signer refuses that draft with its own "not exactly the lines" message, so the test fails before this task and proves the copy is wired in.
- **Verify:**
  - **Signer tests.** `cargo test -p terminus --all-features --test sign_release` passes on both shells, the new test included.
  - **Pinned messages.** `git diff -- crates/terminus/tests/sign_release.rs` shows only added lines.
  - **Old check gone.** `grep -n 'grep -q -x -F' scripts/sign-release.sh` prints nothing.
  - **Verbatim.** `diff <(sed -n '/^read_manifest() {$/,/^}$/p' install.sh) <(sed -n '/^read_manifest() {$/,/^}$/p' scripts/sign-release.sh)` prints nothing, and the first range is not empty.

### Task 5: Fail the test run when the signer's copies drift from install.sh

- **Files:** crates/terminus/tests/sign_release.rs (model: `install_sh_pins_the_release_key_once` in crates/terminus/tests/install_script.rs)
- **Action:** Add a plain `#[test]` (D-14) that reads `install.sh` and `scripts/sign-release.sh` as text through `repo_file`. Cite #99.
  - **Three functions:**
    - `valid_version() (` closed by `)`;
    - `digest() {` closed by `}`;
    - `read_manifest() {` closed by `}`.
  - **For each function, in each script:**
    1. Assert exactly one line is equal to the opening line.
    2. Cut from that line through the first following line equal to the closer.
    3. Assert the cut is closed and holds more than the two boundary lines.
  - **Then** assert the two cuts are equal, with a message naming the function and both files.
  - **Where it runs.** It runs under `test.yml`'s `cargo test --workspace` on all four runners. Add no workflow step.
- **Verify:**
  - **Pass.** `cargo test -p terminus --all-features --test sign_release` passes.
  - **AC5, falsify.** In the signer's copies, change one character of each function in turn and run the test each time. It must fail and name that function. Suggested changes:
    - `-le 20` to `-le 21` in `valid_version`;
    - `%% *` to `% *` in `digest`;
    - one letter of a message in `read_manifest`.
  - **A renamed function.** Rename the signer's `digest() {` line, and the test fails too.
  - **Restore.** Restore after each change with `git checkout -- scripts/sign-release.sh`. `git status --short scripts/` is clean at the end.

### Task 6: Say in the docs that the signer runs no installer code

- **Files:** THREAT-MODEL.md (the **Secrets Terminus itself holds** paragraph), docs/release.md (How a release is signed, step 3), docs/design/0008-install-and-release.md (section 1's first paragraph, the Release key row in section 2, and the Inputs and Outputs of `scripts/sign-release.sh v<X.Y.Z>`)
- **Action:** These are D-16 and D-13. Docs describe the current design only (`.planning/PROJECT.md`), so write no history of the `eval`.
  - **`THREAT-MODEL.md`.** Replace the clause from "keeps no copy of the public key or the version check" through "each time a release is signed" with what is now true:
    - it keeps no copy of the public key, and reads it from `install.sh`'s `release_key=` line as a value;
    - it holds its own copies of `install.sh`'s `valid_version`, `digest` and `read_manifest`, and runs no code out of `install.sh`;
    - a test in `crates/terminus/tests/sign_release.rs` fails when a copy differs from `install.sh`'s.
    - The attestation sentence stays.
  - **`docs/release.md` step 3.** It checks the tag with its own copy of `install.sh`'s `valid_version`. A test CI runs holds its copies of `valid_version`, `digest` and the manifest check to `install.sh`'s text, so it refuses exactly what the installer refuses and runs none of `install.sh`. Steps 2 and 6 stay; they are still true.
  - **`0008`, section 1.** "the signing script reads its key and its version check" becomes "reads its key".
  - **`0008`, the Release key row.** It drops "as it reads `install.sh`'s `valid_version` function, and keeps no copy of either", and says the signer keeps no copy of the key.
  - **`0008`, Inputs.** Drop "and one `valid_version` function".
  - **`0008`, Outputs.** It checks the tag's version with its own copy of `install.sh`'s `valid_version`, and every manifest with its own copy of `install.sh`'s manifest check, and a test holds the copies to `install.sh`.
  - **`0008`, Refusals.** It stays unless a phrase in it is now wrong.
- **Verify:**
  - **`eval` gone.** `grep -n "eval" THREAT-MODEL.md docs/release.md docs/design/0008-install-and-release.md` prints nothing.
  - **Old phrases gone.** `grep -n "keeps no copy of it either\|one .valid_version. function\|as it reads .install.sh.s. .valid_version." docs/release.md docs/design/0008-install-and-release.md` prints nothing.
  - **Tags.** `cargo test -p terminus-core --all-features --test planning_tags` passes.
  - **Workspace.** `cargo test --workspace --all-features` passes. `cargo clippy --locked --all-targets --all-features` reports no warning.

## Notes

- **The planner's choices,** where CONTEXT left the call open:
  - The manifest function is named `read_manifest` and sits directly after `digest`.
  - It takes the manifest's path as its one argument, so the same text reads `$scratch/manifest` in the installer and each draft manifest in the signer.
- **One test beyond D-15** (task 4's third-line manifest). It proves the signer calls its copy of `read_manifest`. The drift test proves the copy is identical, and the existing tests would pass even if the copy were never called.
- **For the human.** A malformed manifest is now refused with `install.sh`'s wording, which does not name which of the four manifests failed. No decision covers naming it, so this plan does not add that.
- **Plan shape:** this is plan 3 of 3. It shares only `THREAT-MODEL.md` with plan 2, in a different paragraph, so it runs after plan 2 rather than beside it. CONTEXT's note that #73 and #99 touch separate files missed that shared file. No behaviour here depends on plans 1 or 2.
- **Prior art:**
  - D-01 (phases/7/CONTEXT.md, phase 7) is tasks 1 and 4.
  - phases/10/UAT.md (phase 10) gives the manifest shape `read_manifest` accepts.
