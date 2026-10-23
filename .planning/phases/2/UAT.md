---
status: testing
phase: 2
fields_version: 1
started: 2026-10-23
updated: 2026-10-23
---

## Items

### 1. Fresh store from the reworded schema
expected: Stop everything, clear ephemeral state, start from scratch: a new store built from the reworded CREATE_SQL opens clean, its sqlite_master SQL carries the new comment text with no planning tag, and one recall query returns real data.
origin: smoke
status: pass
first_pass: pass
source: verifier
evidence: Fresh scratch store (all four dir env vars in scratchpad): ingest of session-recall.jsonl exits 0, verify reports 1 checked and 0 failed, and search SearchManager returns 1 hit with the archived turn. The read-only sqlite_master.sql dump has 69 comment lines, no tag in any family, and no old-only 1c55fec comment text.

### 2. #106's pattern finds only the four session_title lines
expected: git grep -nE with #106's exact pattern over crates/ tests/ fuzz/ Cargo.toml docs/ prints only the four session_title lines, in tests/fixtures/hooks/session-start.json, tests/fixtures/hooks/user-prompt-submit.json, crates/terminus/tests/owner_only.rs and crates/terminus/tests/prompt.rs.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: git grep -nE with #106's exact pattern -- crates tests fuzz Cargo.toml docs prints exactly owner_only.rs:444, prompt.rs:151, hooks/session-start.json:1 and hooks/user-prompt-submit.json:1, all session_title values. A worktree grep -r, untracked files included, prints the same 4.

### 3. The tags #106 misses are gone
expected: Over the same roots, git grep -nE '\bPhase [0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' and git grep -ni hardin print no line.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: The capital Phase N / PLAN-N / ROADMAP criterion grep and git grep -ni hardin print nothing over the five roots. Wider sweeps (generic capitals-hyphen-digits, phases N, loose D/AC) print nothing either.

### 4. Full test suite passes and runs the guard
expected: cargo test --workspace --all-features passes, and its output lists the guard test by name.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: cargo test --workspace --all-features --no-fail-fast exited 0: 1013 passed, 0 failed, 0 ignored. The log shows 'Running tests/planning_tags.rs' and 11 tests ok, including no_comment_or_doc_line_cites_a_planning_tag.

### 5. Guard fails on each added tag and allows real names
expected: Each of these, added one at a time, makes the guard test fail with a message naming the file and line: // see SRCH-02 in a crates/terminus-core/src file, // (D-04) in a crates/terminus/tests file, -- INJ-07 inside CREATE_SQL, # AC3 in fuzz/Cargo.toml, and ING-08 in docs/json-shapes.md. Adding phase 2 to a line of docs/json-shapes.md, or UTF-16 to a Rust comment, does not make it fail.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: One probe at a time, each restored exactly. SRCH-02 in core lib.rs gives lib.rs:53, D-04 in terminus tests/cli.rs gives cli.rs:1548, '-- INJ-07' in CREATE_SQL gives schema.rs:62, '# AC3' in fuzz/Cargo.toml gives :117, and ING-08 in docs/json-shapes.md gives :3. Each exits 101 naming file:line:tag. 'phase 2' in json-shapes.md and '// UTF-16' in a Rust comment both exit 0.

### 6. Only comments, assertion messages and doc prose changed
expected: In the phase's diff since 1c55fec, every changed line outside crates/terminus-core/tests/planning_tags.rs is a comment line (//, ///, //!, TOML #, or SQL -- inside CREATE_SQL), the text of an assert! or panic! message, or a prose line in a markdown file.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: 46 non-comment .rs lines form 23 message-only pairs at the D-06 sites. A comment-stripped token comparison of all 143 changed .rs files finds exactly 23 differing tokens, all assert/panic message strings. All 121 changed -- lines sit inside CREATE_SQL. The TOML diff is # lines only. The only other changed files are docs/json-shapes.md and tests/fixtures/README.md.

### 7. Guard fails when it reads nothing
expected: Renaming the CREATE_SQL declaration in schema.rs makes the guard fail with "only 0 SQL comment lines read", and a scanned root that yields fewer files than its floor makes the guard fail rather than pass with nothing scanned.
status: pass
first_pass: pass
source: verifier
evidence: Renaming the declaration to SCHEMA_SQL, or moving its opening quote to the next line, fails with 'only 0 SQL comment lines read from crates/terminus-core/src/store/schema.rs'. Moving fuzz/src/lib.rs aside fails with 'only 0 .rs files under fuzz/src'.

### 8. Plan task numbers from the removed planning files remain in test comments
expected: behavior wrong - the cleanup missed a family of planning references: plan task numbers. They sit outside #106's pattern and the AC2 greps, but inside the goal ('no comment cites a tag from the removed planning files'), and the guard has no family for them, so more can still be added
origin: verifier
status: pass
first_pass: fail
source: model
evidence: Retest at 58bf1d2 after ced30c1 and 58bf1d2. git grep -niE '\btask [0-9]+' -- crates tests fuzz Cargo.toml docs ':!crates/terminus-core/tests/planning_tags.rs' prints nothing (exit 1), and the plan-pointer grep "the plan('s)? |plan names|acceptance criterion names" prints nothing (exit 1). cargo test -p terminus-core --test planning_tags: 11 passed, 0 failed, including a_phase_or_task_number_fires_in_code_in_any_case_and_not_in_markdown. Probe: appending "// Task 5's work still ran" to crates/terminus-core/src/config.rs fails the guard with 'crates/terminus-core/src/config.rs:1321: Task 5'; file restored, git diff empty. Every changed line in ced30c1 outside planning_tags.rs is a // comment line. cargo test --workspace --all-features --no-run builds every test target.
reported: behavior wrong - the cleanup missed a family of planning references: plan task numbers. They sit outside #106's pattern and the AC2 greps, but inside the goal ('no comment cites a tag from the removed planning files'), and the guard has no family for them, so more can still be added
severity: minor
cause: The cleanup's scope was #106's pattern plus the families a capitals-hyphen-digits sweep found (CONTEXT D-02, D-03). "Task N" carries no hyphen, so the sweep never saw it, and the guard's families (D-01) have no rule for it. Confirmed by git grep -nE '\b[Tt]ask [0-9]+' over the five roots: exactly the five lines in crates/terminus/tests/install.rs (410, 799, 953, 1291) and crates/terminus/tests/uninstall.rs:747, all unchanged since 1c55fec. Three more comments point at a removed plan without a number: crates/terminus-core/tests/backfill.rs:32, crates/terminus-core/tests/egress.rs:18 and crates/terminus/src/cmd/mcp/tools.rs:92.
fix: ced30c1, 58bf1d2, retest

## Summary

total: 8
passed: 8
failed: 0
pending: 0
skipped: 0
blocked: 0
reworked: 1
