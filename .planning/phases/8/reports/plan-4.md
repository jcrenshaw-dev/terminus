PLAN COMPLETE
Plan: .planning/phases/8/PLAN-4.md
Tasks: 1 of 1
| Task | Commit | Note |
|---|---|---|
| 1 - Count every file under the data directory in the purge size | d57ec87 | `bytes` moved from data.rs to cmd/mod.rs as `directory_bytes`; uninstall's `size_bytes` removed; tests and both docs updated |
Deviations: none
Open items: the first full-suite run (`cargo test --workspace --all-features`) failed once in `install_script::a_rebuild_at_the_same_version_replaces_its_file::sh`, which is outside this plan's files; it passed alone and in the confirming `--no-fail-fast` run, so it reads as a flake.
