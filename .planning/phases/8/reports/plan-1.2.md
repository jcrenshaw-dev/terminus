PLAN CHECKPOINT: suite-red
Plan: .planning/phases/8/PLAN-1.md
Tasks: 4 of 4
| Task | Commit | Note |
|---|---|---|
| 1 - Carry the output mode into the dispatcher | 758ebb5 | Committed after the identity/signing checkpoint resolved. cli/doctor/lifecycle/status pass, fuzzing cfg check passes, clippy clean. |
| 2 - Empty document from parsed arguments | 2b5c716 | Seven parsing commands expose `empty`; failure test widened to all 14 DATA_COMMANDS with replay thresholds, regenerate since and export destination asserted; four `--json` misuse cases added. Targeted tests, clippy and fuzzing cfg check pass. |
| 3 - Main writes the failure document, `read::unusable` deleted | 3908149 | reindex and observations regenerate held locks, and stats/replay damaged log, return Failure::Operational; compact unchanged. New tests: damaged-log documents pinned in tests/replay.rs, regenerate held lock in tests/cli.rs. replay/cli/observations/lifecycle pass, clippy clean, greps as specified. |
| 4 - json-shapes.md | f7e64b3 | Operational-failure list removed; misuse is the one run with no document; export refusal documented. Greps as specified. |
Suite: `workflow.test_command` is null; ran `cargo test -p terminus --all-features --no-fail-fast` (the plan's scope; workspace-wide fails on untracked files before this plan). Every test binary passes except three, none touched by this plan and none repairable inside the lease:
- `tests/config_mode.rs` (untracked, named in the plan's Notes as not part of the tree): 3 failed, `terminus.toml` still has a `[provider]` table.
- `tests/install_script.rs`: 29 failed, all `::dash` cases, "dash is not installed".
- `tests/sign_release.rs`: 11 failed, all `::dash` cases (sign_release.rs:66), same cause.
No repair round attempted: the causes are a missing `dash` on this machine and an untracked file outside the lease.
Deviations: none
Open items: install `dash` (or accept the red) and decide on `crates/terminus/tests/config_mode.rs`; neither is this plan's.
