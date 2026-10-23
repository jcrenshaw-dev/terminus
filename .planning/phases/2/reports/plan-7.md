PLAN COMPLETE
Plan: .planning/phases/2/PLAN-7.md
Tasks: 3 of 3; fix for #115: 1 of 1
| Task | Commit | Note |
|---|---|---|
| 1 - Write the tag matcher and prove it fires and holds its boundaries | 9a9663f | 4 matcher self-tests pass; the three git grep checks print nothing over the file; Cargo.toml unchanged since 1c55fec |
| 2 - Extract comment text per file kind, without reading string literals | 19975e2 | 6 extraction self-tests pass beside the 4 matcher tests; greps still print nothing; file rustfmt-clean |
| 3 - Scan the named roots, enforce the file floor and report every finding by file and line | 7674f87 | Scan passes on the tree (11 tests). Probes, one at a time and each undone: lib.rs:53 SRCH-02, cli.rs:1548 D-04, schema.rs:63 INJ-07 (inside CREATE_SQL), fuzz/Cargo.toml:117 AC3, docs/json-shapes.md:748 ING-08 each failed naming file and line; "phase 2" on json-shapes.md:1 and "// UTF-16" in lib.rs passed. Floors 40/25/25/20/10/1 .rs and 10 .md against 53/36/35/27/13/1 and 15 |
| Full suite | - | `cargo test --workspace --all-features` (workflow.test_command is null; the command CI and AC3 name) exit 0, no failed result; output shows `Running tests/planning_tags.rs` with 11 tests. AC1 prints only the four session_title lines; AC2 prints nothing; AC5 Rust pipeline prints 46 lines, the 23 assert!/panic! messages as -/+ pairs; the TOML pipeline and the non-.md file list print nothing |
| Fix #115 - fail when the schema's SQL comments go unread | 281ee53 | Main test now needs at least 100 SQL comment lines from schema.rs (about 166 there), counted as pieces equal to their whole source line; the schema self-test checks a `// --` comment is not counted. planning_tags passes 11 of 11. Probes, each undone: SQL_DECLARATION renamed in the test, and schema.rs:61 reformatted so the opening quote sits on the next line, each failed with `only 0 SQL comment lines read from crates/terminus-core/src/store/schema.rs`. The three tag greps print nothing over the file; clippy -D warnings and check --all-targets exit 0; file rustfmt-clean |
| Full suite (fix #115) | - | `cargo test --workspace --all-features` (workflow.test_command is null) exit 0, 68 `test result: ok` lines, none failed; planning_tags 11 passed |
Deviations: none
Open items: Rust block comments are not read; the scanned roots hold none, so the lexer skips them rather than carry a branch nothing exercises.
Open items (fix #115): `cargo fmt --all -- --check` already reports drift in 9 files outside this lease (crates/terminus/src/cmd/hook.rs, crates/terminus/tests/{cli,hook,release_workflows,sign_release}.rs, crates/terminus-core/src/observe/egress.rs, crates/terminus-core/tests/reindex.rs); left alone.
