PLAN COMPLETE
Plan: .planning/phases/7/PLAN-3.md
Tasks: 6 of 6
| Task | Commit | Note |
|---|---|---|
| 1 - Move install.sh's manifest parse into a function | 001c310 | read_manifest at line 88, called once; install_script 59 passed; marker test passed |
| 2 - Give the signer its own valid_version, and run nothing from install.sh | 9713ab9 | eval, read loop and two refusals gone; valid_version diff empty (20 lines); sign_release 19 passed |
| 3 - Prove a signing run executes nothing from install.sh | fc813ac | side-effect test on both shells plus no-eval text test; with the pre-task-2 signer all three fail, restored they pass |
| 4 - Check every manifest with the installer's own read_manifest | 3383d90 | read_manifest copy (25 lines, diff empty), tag version held as tag_version; wiring test fails on the old signer, passes now; only added test lines |
| 5 - Fail the test run when the signer's copies drift | 88a9d06 | the_signers_copies_are_install_shs_text; one-character changes in valid_version, digest, read_manifest and a renamed digest line each fail it, naming the function |
| 6 - Say in the docs that the signer runs no installer code | d7199f2 | THREAT-MODEL.md, docs/release.md, 0008; no eval or old phrase left; planning_tags passes |
Deviations: none
Open items: none. Full suite (cargo test --workspace --all-features --no-fail-fast, clean git archive HEAD, umask 022): only the three accepted SessionStart timing tests fail (brief a_hundred_session_starts..., hook every_event_exits_zero... and every_event_stays_inside_the_budget_with_the_redaction_knob_on); clippy --locked --all-targets --all-features clean.
