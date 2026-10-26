---
status: testing
phase: 7
fields_version: 1
started: 2026-10-26
updated: 2026-10-26
---

## Items

### 1. data move leaves the pointer 0600 and the config dir 0700
expected: Under umask 022 with no config directory, terminus data move to a missing destination and to an existing empty destination each leave data-location at 0600 and the config directory at 0700
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: store/open.rs:824-832 write_location_pointer (owner_only::create_dir_all + replace), called at cmd/data.rs:223; own_uid's owner_only::create_dir_all at data.rs:379. tests/owner_only.rs:384-413 a_data_move_creates_the_config_directory_0700_and_the_pointer_0600 removes the config dir, runs the binary under umask 022 for an absent and an existing empty destination, and asserts 0700, 0600 and destination+newline. It passed at HEAD.

### 2. Only store/open.rs uses create_new outside owner_only, and no symlink call
expected: Outside owner_only.rs and test code, .create_new(true) appears only in store/open.rs's create of terminus.db, and no non-test code calls std::os::unix::fs::symlink
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: Non-test create_new(true) only at owner_only.rs:92 and store/open.rs:128 (install/mod.rs:538 is in #[cfg(test)] at 510). Non-test unix::fs::symlink only at owner_only.rs:123. 'AlreadyExists => continue' only at owner_only.rs:138. All 16 owner_only unit tests, the install stale-name test and the drain sweep test passed at HEAD.

### 3. Session-id limits defined once and still refused
expected: MIN_NAME and MAX_NAME are each defined once in crates/, and the tests refusing a bad hook session id for the state file and the decision file pass
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: MIN_NAME/MAX_NAME are defined only at inject/mod.rs:57-58. The check checked_session_id at mod.rs:73-79 is used by state.rs:220 and decision.rs:309. a_session_id_that_could_name_a_path_creates_nothing_anywhere and inject::decision::tests::a_hostile_session_id_reaches_no_filesystem_call passed. The core tests/ directory is unchanged since 4a89b49.

### 4. Signer has no eval and runs no installer code
expected: scripts/sign-release.sh contains no eval, and a signing run whose fixture install.sh has a valid_version that creates a file leaves no such file
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: grep -c eval scripts/sign-release.sh prints 0. Only the release_key= value is read from install.sh (sign-release.sh:92-98). sign_release_holds_no_eval and a_signing_run_executes_nothing_from_install_sh::{sh,dash} passed at HEAD and failed against 4a89b49's signer, where the marker file appeared.

### 5. Drift test catches a one-character change
expected: A cargo test passes on the tree and fails when the signer's valid_version, digest or read_manifest differs by one character from install.sh's
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: the_signers_copies_are_install_shs_text (sign_release.rs:723-738) passed at HEAD. Each one-character mutation of the signer's valid_version, digest and read_manifest failed the test and named the function, and a renamed digest() line failed too. It runs under test.yml:194 cargo test --workspace on four runners and is not on the skip list.

### 6. Manifest tests unchanged after read_manifest
expected: The manifest tests in crates/terminus/tests/install_script.rs pass with no expected value changed
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: git diff 4a89b49 HEAD -- crates/terminus/tests/install_script.rs is empty. The install.sh diff moves the parse verbatim into read_manifest (install.sh:88-112, called at 235), with $scratch/manifest becoming $1. install_script: 59 passed, 0 failed at HEAD.

### 7. Full workspace suite passes
expected: cargo test --workspace --all-features passes
criterion: AC7
status: skipped
first_pass: fail
source: verifier
evidence: crates/terminus/tests/brief.rs:328 a_hundred_session_starts_stay_inside_the_budget_and_the_wall_clock: p99 21.87 ms over the 10 ms budget. crates/terminus/tests/hook.rs:413 every_event_exits_zero_with_an_empty_stdout_inside_the_budget: p99 20.74 ms. crates/terminus/tests/hook.rs:1132 every_event_stays_inside_the_budget_with_the_redaction_knob_on: p99 36.17 ms. At 4a89b49, run twice by name in a clean export: p99 23.8 to 39.8 ms, all failing. Every other test target passed. .github/workflows/test.yml:182-186 skips these three on all hosted runners (cbd2bdc0), so the CI gate is green on them by design, but the local command AC7 names is red.
reported: 1
severity: major
cause: Not caused by phase 7. The three SessionStart p99 budget tests (brief.rs:328, hook.rs:413, hook.rs:1132) miss their 10 ms budget on this machine at the phase base 4a89b49 as well as at HEAD, with matching numbers. CI skips exactly these three on every runner (test.yml:182-186, cbd2bdc0), so AC7 names a stricter command than the CI gate runs.
reason: The three SessionStart budget tests fail at the phase base 4a89b49 too, so phase 7 did not cause them, and CI skips exactly these three on every runner since cbd2bdc0.

## Summary

total: 7
passed: 6
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 1
