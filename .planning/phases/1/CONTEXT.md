# Phase 1: Signer version check - Context

Gathered: 2026-10-19
Feeds: /cad-plan 1

## Scope boundary

In: `scripts/sign-release.sh` checks a release version with `install.sh`'s own `valid_version`, read out of `install.sh` at run time, so it refuses every version part longer than 20 digits or above u64 that `install.sh` refuses (#86). That covers the tests in `crates/terminus/tests/sign_release.rs` and closing #86.
Out: Any change to `install.sh` (D-03). Any change to what the signer prints for a bad tag (D-04). Any other check the signer copies or could share with `install.sh`.
Deferred: None.
Plan shape: one plan.

## Durable decisions

None this phase.

## Decisions

- D-01 (Source of the check): The signer reads the `valid_version` function out of `install.sh` when it runs and uses it, the same way it already reads the `release_key` line. It refuses to run unless `install.sh` holds exactly one `valid_version` function. This is the option #86 asks for first; keeping a full copy with a test that the two match was rejected. Evidence: `scripts/sign-release.sh:50-58`; `install.sh:58-77`; issue #86.
- D-02 (No copy): The signer's own `valid_version` at `scripts/sign-release.sh:22-30` is deleted, and the script keeps no copy of the check. Evidence: `scripts/sign-release.sh:22-30`.
- D-03 (install.sh unchanged): `install.sh` is not changed. It stays standalone, and its check stays as it is. Evidence: `install.sh:58-77`; issue #86.
- D-04 (Refusal message): A version part over 20 digits or above u64 gets the same usage line every bad tag gets today, `terminus: usage: sh scripts/sign-release.sh v<X.Y.Z>`, and nothing reaches `gh` or `ssh-keygen`. Install.sh's own u64 wording was rejected. Evidence: `scripts/sign-release.sh:46-47`; `crates/terminus/tests/sign_release.rs:434-448`.
- D-05 (Tests): The new refusals are added to `a_bad_tag_is_refused`, which runs under both `/bin/sh` and dash. One more test shows the largest legal part, `18446744073709551615`, still gets past the tag check. Evidence: `crates/terminus/tests/sign_release.rs:434-448`, `:563-570`.

## Acceptance criteria

- [ ] AC1: `scripts/sign-release.sh` run with `v123456789012345678901.0.0` or `v18446744073709551616.0.0` exits non-zero and prints exactly `terminus: usage: sh scripts/sign-release.sh v<X.Y.Z>` on stderr, under both `/bin/sh` and dash, and calls neither `gh` nor `ssh-keygen`.
- [ ] AC2: Run with `v18446744073709551615.0.0`, the signer gets past the tag check: its first `gh` call is `release view`.
- [ ] AC3: `scripts/sign-release.sh` holds no definition of `valid_version`.
- [ ] AC4: With an `install.sh` beside the script that holds no `valid_version` function, or two, the signer exits non-zero with a message naming `install.sh`, and calls neither `gh` nor `ssh-keygen`.
- [ ] AC5: `install.sh` is byte for byte the same as on `main`, and `cargo test --workspace --all-features` passes.
- [ ] AC6: Issue #86 is closed by the merged change. (human-verify: needs the merged PR on GitHub)

## Flagged assumptions

- How the function is cut out of `install.sh` and run (for example, the lines from `valid_version() (` to the next line that is only `)`, then `eval`) is the planner's call - Likely; if wrong: a cut that takes too much or too little runs the wrong code or fails every tag.
- The version check moves after `install.sh` is read, so a bad tag with no `install.sh` beside the script prints the `install.sh: not found` message instead of the usage line - Likely; if wrong: the order stays as it is and the planner must read `install.sh` first only for the check.
- The test fixture already copies the shipped `install.sh` with only the key swapped, so it carries the real `valid_version` with no fixture change - Confident; if wrong: the fixture's `install.sh` needs the function added. Evidence: `crates/terminus/tests/sign_release.rs:189-202`.
