PLAN COMPLETE
Plan: .planning/phases/1/PLAN-1.md
Tasks: 3 of 3
| Task | Commit | Note |
|---|---|---|
| 1 - Delete the re-key's tests and their helper | 7d8fed3 | session_keys.rs, its [[test]] block and testkit::plant_old_key deleted; check, clippy, recover+capture tests green |
| 2 - Remove the re-key and the twin merge from recovery | b6cbbc8 | recover.rs down to open, up_to_date, sweep_watermarks (875 to 205 lines); Recovered keeps lowered, removed, duration; three outside comments rewritten; all greps empty, check and clippy clean, six named test targets green |
| 3 - Bring the docs and threat model up to date on recovery | 43593fb | 0003-ingest step 2, file table, flowchart node, read_rows paragraph and Figure 3 rewritten; 0004 Session row, commands.md status line, json-shapes key paragraph, three THREAT-MODEL entries; re-key/twin grep empty, merge counts 0/0/1 |
Deviations: [deviation] Task 1 Verify asserted `grep -rn 'session_keys\|plant_old_key' crates/` prints nothing; it prints two lines from `crates/terminus-core/tests/snapshot.rs:89,151`, a local `fn session_keys` that lists session keys for the snapshot test and has nothing to do with the re-key. The file is outside this plan's lease and AC1 does not search for `session_keys`, so it was left alone; with that file excluded the grep prints nothing.
Open items:
- `testkit::recut` (crates/terminus-core/src/testkit.rs) has no caller now that session_keys.rs is gone; kept as the plan's Notes say, for John to decide.
- `cargo fmt --check` reports a diff in `crates/terminus/src/cmd/hook.rs:247` (the `Arm` type alias wants wrapping) that predates this plan and is outside its lease; the files this plan touched pass rustfmt.
Suite: `cargo test --workspace --all-features` (workflow.test_command is null; this is the command CI and the plan name), 67 test binaries, 1015 passed, 0 failed.
