PLAN COMPLETE
Plan: .planning/phases/7/PLAN-2.md
Tasks: 3 of 3
| Task | Commit | Note |
|---|---|---|
| 1 - Move data move's probe, directories and copies onto owner_only | 946ed98 | cmd::data:: (9), datamove (13), owner_only tree test pass; clippy clean in a clean copy |
| 2 - Write the pointer 0600 and the config directory 0700 from core | ab3d3a4 | owner_only (4), datamove, store_open, config pass; both mutations fail the new test on the config directory's mode (755 want 700), restored |
| 3 - Describe the pointer's modes, name every owner_only site | 919200b | AC2 greps clean; planning_tags passes in the clean copy; clippy clean |
Deviations: none
Open items:
- Task 1's grep for `create_new` in data.rs prints the one line `owner_only::create_new(destination)` in `copy_file`: it is the helper call the task asks for, not a hand-built create.
- Removed `create_destination` and its three unit tests; `run` calls `owner_only::create_dir_new` directly (plan 1's owner_only tests cover the same cases).
- `planning_tags` fails in /code/terminus as it stands, on the untracked credentials.rs from the other line of work (PRIV-02, D-14 ...); it passes in the clean copy of the tracked tree.
- Suite: `cargo test --workspace --all-features --no-fail-fast` (workflow.test_command is null; this is what test.yml runs), under umask 022, in a clean `git archive HEAD` of 919200b: 66 targets ok, 2 failed with only the three accepted SessionStart timing tests (brief a_hundred_session_starts_stay_inside_the_budget_and_the_wall_clock; hook every_event_exits_zero_with_an_empty_stdout_inside_the_budget and every_event_stays_inside_the_budget_with_the_redaction_knob_on). Nothing else failed.
- The shell umask is 002; all runs used umask 022.
