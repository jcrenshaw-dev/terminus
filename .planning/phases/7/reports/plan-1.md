PLAN COMPLETE
Plan: .planning/phases/7/PLAN-1.md
Tasks: 6 of 6
| Task | Commit | Note |
|---|---|---|
| 1 - Add the exclusive creates to owner_only | 136460b | create_new, create_temporary, create_temporary_symlink, create_dir_new; 12 lib tests at the time |
| 2 - Install temporaries and symlink onto owner_only | 427e646 | install, uninstall, install_script and cmd::install:: tests pass |
| 3 - Snapshot reservation and export files | e4a1bee | snapshot, owner_only, lifecycle, tighten tests pass |
| 4 - replace | 15c69ff | 16 owner_only lib tests pass |
| 5 - Injection writers and decision reservation | 2d53433 | inject_state, feedback, inject::, owner_only, decisions, suppression pass |
| 6 - Session-id check once | ca718a2 | MIN_NAME/MAX_NAME defined once in inject/mod.rs; both refusal tests pass |

Suite: `cargo test --workspace --all-features --no-fail-fast` (workflow.test_command is null; this is what test.yml runs), under umask 022, in a clean copy of the tracked tree at ca718a2. 66 test targets ok, 2 failed, and both fail identically at the base commit 4a89b49 (checked in a clean archive of it):
- `-p terminus --test brief`: a_hundred_session_starts_stay_inside_the_budget_and_the_wall_clock, "p99 21.92 ms is over 10 ms" (base: 22.19 ms)
- `-p terminus --test hook`: every_event_exits_zero_with_an_empty_stdout_inside_the_budget ("SessionStart: p99 35.73 ms is over 10 ms"; base 35.52 ms) and every_event_stays_inside_the_budget_with_the_redaction_knob_on (36.28 ms; base 35.54 ms)
These are wall-clock p99 budgets on SessionStart (p50 about 2 ms). They fail the same way before and after this plan, on this machine, so they are not a regression from it, and the repair they need is outside this plan's lease. Re-run alone after the full run: same failures.

Deviations: none
Open items:
- The suite is red on the three timing tests above, with or without this plan (the #75/#104 area). Need the orchestrator to decide whether that stops the plan.
- The shell umask here is 002. `owner_only::tests::only_the_leaf_directory_is_owner_only` (and the new 0755-parent test) fail under 002 and pass under 022, as the plan states; all runs above used umask 022.
- The working tree holds untracked files from another line of work (observe/cost.rs, credentials.rs, tests/judgment.rs, cost.rs, provider.rs, session_keys.rs, credentials.rs, tests/config_mode.rs, and others). Their tests do not compile (`testkit::chat_completion` missing), so `cargo clippy --all-targets` and `cargo test --workspace` fail to build in /code/terminus as it stands. Clippy (`--locked --all-targets --all-features -- -D warnings`) and the suite were run in a clean copy of the tracked files and were clean. Nothing here touches those files.
- `install_script::a_missing_signature_is_refused::sh` failed once (curl TLS verify of the local test server) and passed on the immediate rerun and in the suite; unrelated to this plan.
- Declined: a symlink form returning `(PathBuf, ())`; it returns the `PathBuf` alone because nothing reads the unit.
- Declined: making the session-id check `pub(crate)`; it is private to `inject/mod.rs`, which both child writers can already read.

Accepted by John despite the red suite: the three SessionStart timing tests fail identically at 4a89b49 (rechecked by the orchestrator in clean copies).
