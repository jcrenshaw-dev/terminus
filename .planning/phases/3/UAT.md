---
status: testing
phase: 3
fields_version: 1
started: 2026-10-18
updated: 2026-10-19
---

## Items

### 1. A fresh settings.json is 0600 under umask 022
expected: Under umask 022, `terminus install` into a config directory with no `settings.json` creates one at mode 0600.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: create_temporary opens through owner_only::options() (crates/terminus/src/cmd/install/mod.rs:68); fill copies no mode when there is no target (json_file.rs:699-702). a_settings_file_install_creates_is_owner_only (tests/install.rs:483) runs the binary under 'umask 022; exec' and passes at HEAD, and fails 'settings.json is 644' against 3322699^ code.

### 2. Temporaries open at 0600 and modes are kept
expected: Every file `create_temporary` opens is 0600 the moment it is opened, before any byte is written. After install, an existing 0644 `settings.json` is still 0644, its backup is 0600, and the stored version file is 0755.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: Every install/uninstall file create goes through create_temporary (json_file.rs:478, :666; binary.rs:273). The unit test a_temporary_is_owner_only_from_the_open (mod.rs:540) passed under umask 022 with no skip line. a_settings_file_keeps_its_own_mode_through_install (tests/install.rs:505) shows 0644 kept and backup 0600; the version file is asserted 0755 (install.rs:498-499; binary.rs:366). Restore keeps the live mode (json_file.rs:1222).

### 3. data move refuses an unsafe destination before asking
expected: `terminus data move <dest>` refuses before asking when the destination is a symlink (with or without a trailing slash), is owned by another uid, has any group or world bit, or cannot be listed, and the store and the pointer stay as they were. A destination that does not exist, or an empty 0700 directory this uid owns, is still accepted.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: refuse_unchecked_destination (crates/terminus/src/cmd/data.rs:295-350) runs at :118 before the prompt. datamove tests at tests/datamove.rs:449, :472, :490, :521 pass at HEAD (13/13), and the other-uid unit seam test data.rs:628 passes. An absent destination moved in a real CLI spot-check, created 0700.

### 4. data move refuses a destination changed while the question waits
expected: When the destination is created or replaced while `move it? [y/N]` waits, answering `y` is refused and nothing is copied.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: The second check at data.rs:154-170 compares against the first Found. The created, replaced and widened tests (tests/datamove.rs:603, :615, :633) pass at HEAD and fail against e89458d^ data.rs. An EEXIST injected into the leaf mkdir via strace is refused with nothing copied (data.rs:176-187).

### 5. Each regression test fails before its fix and passes after
expected: Each fix's regression test fails on the commit before the fix and passes after it.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: HEAD's install tests against 3322699^ code: 2 FAILED ('settings.json is 644', 'the backup is 644'). HEAD's datamove tests against e89458d^ data.rs: 7 FAILED. Mutations of create_temporary, copy_through, fill and the uid comparison each fail their named tests. All pass at HEAD. The a816e1d follow-up is reported separately as a gap.

### 6. The docs describe the fixes and the gaps
expected: `THREAT-MODEL.md` describes the temporary as created 0600, the `data move` destination checks, the unchecked parent directory, the unchecked owner of a store reached through `TERMINUS_DATA_DIR` or an older pointer, and the mode Claude Code writes `settings.json` with. `docs/commands.md` and `docs/design/0004-archive-and-block-rows.md` list the new refusals, and `CHANGELOG.md` `[Unreleased]` carries #64 and #59.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: THREAT-MODEL.md:67, :91-93, :97-104, :115, :270; docs/commands.md:152; docs/design/0004-archive-and-block-rows.md:288-290; CHANGELOG.md:152-173 under [Unreleased] ### Fixed. The bundle strings quoted at THREAT-MODEL.md:93 all occur in versions/2.1.302.

### 7. The landing PR closes #64 and #59
expected: The pull request that lands this phase carries `Fixes #64` and `Fixes #59`, and both close once it merges. (human-verify: needs GitHub, after landing)
criterion: AC7
status: skipped
reported: skip both, checked after landing
reason: Checked at landing; the PR must carry Fixes #64 and Fixes #59.

### 8. The exclusive-mkdir fix (a816e1d) has a regression test that fails when its wiring in run is removed
expected: unwired test - the fix is real and behaves (strace spot-checks), but no test fails if run stops calling create_destination
origin: verifier
status: pass
first_pass: fail
source: model
evidence: At 4fd349e, with the create_destination call in run disabled (if false && found == Found::Nothing, crates/terminus/src/cmd/data.rs:178), cargo test -p terminus --features testkit --test datamove: FAILED. 10 passed; 3 failed (a_hook_and_an_mcp_server_both_answer_off_the_new_location, a_trailing_slash_is_dropped_before_the_pointer_is_written, the_whole_directory_moves_and_the_settings_file_does_not_change, each with "could not be copied: No such file or directory"). Restored: ok. 13 passed; 0 failed. Full workspace: 67 test binaries ok.
reported: unwired test - the fix is real and behaves (strace spot-checks), but no test fails if run stops calling create_destination
severity: minor
cause: copy_tree still creates the top of the destination itself with owner_only::create_dir_all (crates/terminus/src/cmd/data.rs:511), which makes an absent destination on its own. So when run skips create_destination (data.rs:176-187), an absent destination is still made and the move still succeeds, and the only case that would differ is something planted between the second check and the mkdir, which no test can pause the process to plant. The unit tests at data.rs:671 and :678 call create_destination directly, so nothing ties run to it.
fix: 4fd349e, retest

### 9. THREAT-MODEL.md states correctly when the unchecked parent directory lets another account swap the destination
expected: behavior wrong (docs) - the stated window starts too late
origin: verifier
status: pass
first_pass: fail
source: model
evidence: At 703b17b, THREAT-MODEL.md:102 dates the swap "at any point from the second check on, before, during or after the copy"; :103 adds "That covers the `mkdir` alone; past it, the parent gap above still applies."; :104 says each directory below the destination is one exclusive `mkdir` at 0700 and each file `create_new` at 0600. This matches crates/terminus/src/cmd/data.rs at 4fd349e: run checks, then create_destination, then copy_tree opens every entry by path under the destination with DirBuilder::create and create_new. grep finds no other "after the copy" claim about the swap (the :115 row is about remove_dir_all).
reported: behavior wrong (docs) - the stated window starts too late
severity: minor
cause: THREAT-MODEL.md:102 dates the parent-directory swap from after the copy, but nothing holds the path between the second check and the copy: the copy opens every file by path under the destination (crates/terminus/src/cmd/data.rs copy_tree and copy_file), so an account that can rename entries in a parent without the sticky bit can swap the checked or just-created directory for another directory or a symlink at any point from the second check on. THREAT-MODEL.md:103 reads as if nothing planted after the second check is taken, which holds only for the exclusive mkdir itself. A docs gap, no code change: D-09 accepts the unchecked parent.
fix: 703b17b, retest

### 10. Open the pull request that lands phase 3 and confirm its body carries 'Fixes #64' and 'Fixes #59'; after it merges, confirm both issues on crenshawdev/terminus show as closed by it.
expected: The PR body has both closing keywords, and #64 and #59 close automatically on merge.
origin: verifier
why_human: Out-of-reach resource: it needs GitHub and a pull request that does not exist until the landing step, which comes after verification. The UAT item is tagged human-verify, after landing.
status: skipped
reported: skip both, checked after landing
reason: Checked at landing; the PR must carry Fixes #64 and Fixes #59.

## Summary

total: 10
passed: 8
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 2
