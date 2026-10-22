---
phase: 2
plan: 6
requirements:
  - TAGS-01
files:
  - crates/terminus/src/cmd/mcp/mod.rs
  - crates/terminus/src/cmd/mcp/rpc.rs
  - crates/terminus/src/cmd/mcp/tools.rs
  - crates/terminus/src/cmd/read.rs
  - crates/terminus/src/cmd/search.rs
  - crates/terminus/src/cmd/sessions.rs
  - crates/terminus/src/cmd/show.rs
  - crates/terminus/src/cmd/export.rs
  - crates/terminus/src/cmd/json.rs
  - crates/terminus/src/cmd/observations.rs
  - crates/terminus/src/cmd/replay.rs
  - crates/terminus/src/cmd/stats.rs
  - crates/terminus/src/cmd/status.rs
  - crates/terminus/src/cmd/retention.rs
  - crates/terminus/src/cmd/reindex.rs
  - crates/terminus/src/cmd/compact.rs
  - crates/terminus/src/cmd/verify.rs
  - crates/terminus/src/cmd/mod.rs
  - crates/terminus/src/cmd/usage.rs
  - crates/terminus/src/escape.rs
  - crates/terminus/src/lib.rs
  - crates/terminus/tests/cli.rs
  - crates/terminus/tests/corpus.rs
  - crates/terminus/tests/crash.rs
  - crates/terminus/tests/backfill.rs
  - crates/terminus/tests/status.rs
  - crates/terminus/tests/retention.rs
  - crates/terminus/tests/recall_cli.rs
  - crates/terminus/tests/mcp.rs
  - crates/terminus/tests/observations.rs
  - docs/json-shapes.md
  - Cargo.toml
  - crates/terminus/Cargo.toml
---

# Phase 2: Planning tags - Plan 6 of 7: the terminus crate's read, report and MCP side, docs/json-shapes.md and the manifests

## Goal

No comment, assertion message, manifest comment or line of `docs/json-shapes.md` in this plan's files cites a tag from the removed planning files. Each one that carried a tag now states its rule itself (#106). Plan 5 covers the rest of the terminus crate.

## Must be true when done

- Tag checks A and B over this plan's files, `docs/json-shapes.md` and `Cargo.toml` among them, print nothing.
- Every line changed in this plan's `.rs` files since 1c55fec is a comment line or a line of one of the 8 `assert!` messages this plan names (D-06). Every changed manifest line is a TOML `#` comment.
- `docs/json-shapes.md` states the human-output stdout/stderr split as the current contract, with no release history.
- `cargo check --workspace --all-features --all-targets` exits 0, and the test name `a_finalized_session_is_listed_with_every_obs_01_fact` is unchanged.

## Context

Locked by `.planning/phases/2/CONTEXT.md`. These rules bind every task:
- **D-09:** delete a bare tag in parentheses, such as "(D-04)" or "(D-08, INST-02)", together with its parentheses, including one that wraps across lines. Cut a tag used inside a sentence ("D-02: install run through the link ...", "AC1's export clause") and reword the sentence so it states the rule itself, the way c5167d7 rewrote `crates/terminus/src/cmd/install/targets.rs`. Delete a sentence that was only a pointer to a tag. Prefer a present-tense statement of the behavior over which phase or criterion asked for it. If neither the comment nor the code makes a tag's rule plain, the removed files are readable with `git show 5933b3b^:.planning/<path>` (hardin under `.planning/_archive-hardin/`), but what you write must be true of the code now.
- **D-10:** rewrite a claim that only holds under the old phase order to the current fact. **D-11:** give a banner or `//! #` heading whose only content is a tag a topic name, and drop the tag from one that already has a topic.
- **D-06:** a named `assert!` message is reworded to say what failed, keeping every `{...}` placeholder and every macro argument. **D-08:** the `obs_01` test name stays.
- **D-12:** rewrap by hand. Never run `cargo fmt` or rustfmt. Otherwise edit comment lines only: no code line, identifier, other string literal or blank line changes. Find each site by its text; CONTEXT line numbers are at 1c55fec.
- **Tag check A:** `git grep -nE '\bD-[0-9]{2}\b|\b[A-Z]{2,6}-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion' -- <files>`. **Tag check B:** `git grep -niE '\bphase [0-9]+\b|hardin' -- <files>`. **Diff check:** `git diff -U0 1c55fec -- <.rs files> | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'`.

## Tasks

### Task 1: Cut the tags from the read, report and MCP command comments and the crate root

- **Files:** crates/terminus/src/cmd/mcp/mod.rs, crates/terminus/src/cmd/mcp/rpc.rs, crates/terminus/src/cmd/mcp/tools.rs, crates/terminus/src/cmd/read.rs, crates/terminus/src/cmd/search.rs, crates/terminus/src/cmd/sessions.rs, crates/terminus/src/cmd/show.rs, crates/terminus/src/cmd/export.rs, crates/terminus/src/cmd/json.rs, crates/terminus/src/cmd/observations.rs, crates/terminus/src/cmd/replay.rs, crates/terminus/src/cmd/stats.rs, crates/terminus/src/cmd/status.rs, crates/terminus/src/cmd/retention.rs, crates/terminus/src/cmd/reindex.rs, crates/terminus/src/cmd/compact.rs, crates/terminus/src/cmd/verify.rs, crates/terminus/src/cmd/mod.rs, crates/terminus/src/cmd/usage.rs, crates/terminus/src/escape.rs, crates/terminus/src/lib.rs
- **Action:** Apply the Context rules to the 139 tagged lines in these files.
  - The `PLAN-N` lines drop the plan reference:
    - In `read.rs`, "PLAN-4's server advertises `readOnlyHint`" names the MCP server.
    - In `search.rs`, "PLAN-4's `recall_search`" becomes `recall_search`.
    - In `sessions.rs`, "not through PLAN-2's projection" names the projection it means. Read the module to see which.
  - The heading "# The contract every data command keeps (RCL-06)" in `cmd/mod.rs` keeps its topic.
  - "PRIV-04's \"states what it contains\" half" in `export.rs` states the property.
  - "D-23: all 22,412 turns of a 180-file sample carry exactly" in `cmd/mod.rs` keeps the measurement and loses the tag.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints nothing. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 2: Cut the tags from the CLI, corpus, crash, backfill, status and retention tests

- **Files:** crates/terminus/tests/cli.rs, crates/terminus/tests/corpus.rs, crates/terminus/tests/crash.rs, crates/terminus/tests/backfill.rs, crates/terminus/tests/status.rs, crates/terminus/tests/retention.rs
- **Action:** Apply the Context rules to the 95 tagged lines in these files.
  - D-06 messages to reword:
    - `cli.rs`: "D-16 keeps integrity_check in doctor: {text}" and "AC4".
    - `corpus.rs`: "a session key that is not a transcript by AC1's own rule: {key}", and the message beginning "a session claims to continue itself (D-01), which is the link that would", which continues over a `\` line break.
    - `crash.rs`: "AC2 asks for 20 or more kill points, ran {kills}", "AC6 asks for ten or more kills against a first pass, ran {first_kills}" and "AC6 asks for ten or more kills against an append pass, ran {append_kills}". Each keeps its threshold and its placeholder and says what was required instead of which criterion asked.
  - The six banners in `corpus.rs`'s test body, from "--- AC1, the positive half: ..." to "--- D-24: the rebuild a bumped DERIVED_SCHEMA triggers, timed. ---", keep their topics and lose their tags (D-11).
  - "AC1 says every" in `corpus.rs`'s module doc states the rule.
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints only lines of the seven named messages. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 3: Cut the tags from the recall CLI, MCP and observations tests

- **Files:** crates/terminus/tests/recall_cli.rs, crates/terminus/tests/mcp.rs, crates/terminus/tests/observations.rs
- **Action:** Apply the Context rules to the 72 tagged lines in these files.
  - D-10: the doc comments in `mcp.rs` and `recall_cli.rs` that open "D-08 through the server" and "D-08 through the command" each say "Retention is phase 8 and nothing writes that column before then". That is false: `retention::apply` sets `session_meta.is_evicted`. Say instead that the test sets the column directly, so the flag comes off the column alone.
  - D-06 message to reword in `mcp.rs`: "the excerpt must show what matched (D-05): {hit}".
  - In `recall_cli.rs`:
    - "half of ROADMAP criterion 1 that no path expansion would have delivered" names the behavior.
    - "the one line of the query layer PLAN-2 could not reach" says which line, without the plan.
    - The code-line comment "// See the open item on PLAN-3." is deleted. It is a pointer only, and deleting it removes a comment line.
  - The test name `a_finalized_session_is_listed_with_every_obs_01_fact` in `observations.rs` stays (D-08).
- **Verify:** Tag checks A and B over this task's Files print nothing. The diff check over them prints only lines of the one named message. `git grep -n a_finalized_session_is_listed_with_every_obs_01_fact -- crates/terminus/tests/observations.rs` still prints the test's `fn` line. `cargo check --workspace --all-features --all-targets` exits 0.

### Task 4: Cut the tags from docs/json-shapes.md and the manifest comments

- **Files:** docs/json-shapes.md, Cargo.toml, crates/terminus/Cargo.toml
- **Action:** Apply the Context rules to the 8 tagged lines of `docs/json-shapes.md` and the TOML `#` comments of the two manifests: 3 lines in `Cargo.toml` and 1 in `crates/terminus/Cargo.toml`.
  - In `docs/json-shapes.md`, "Without `--json` the split is unchanged from what phase 1 and 2 shipped: the data on stdout, the counts and the warnings on stderr." becomes a statement of the current contract with no release history. It is a public doc (D-04).
  - In `Cargo.toml`, "D-19 also asserted that FTS5 needs an `fts5` cargo feature" states the rule without the tag (a D-09 example).
  - No test reads `docs/json-shapes.md`. Change no TOML key or value line.
- **Verify:** Tag checks A and B over this task's Files print nothing. `git diff -U0 1c55fec -- Cargo.toml crates/terminus/Cargo.toml | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*#'` prints nothing. `cargo metadata --format-version 1 --no-deps > /dev/null` exits 0.

## Notes

- Plans 1 to 6 declare disjoint files and none builds on another. Plan 7, the guard, builds on all six and runs last.
- Plans 5 and 6 split the terminus crate between them so that each plan's declared files stay under the per-plan read ceiling of 675,000 bytes. This plan declares 633,993. It takes the read, report and MCP commands with the CLI, corpus, crash, recall and MCP tests, `docs/json-shapes.md` and the manifests. Plan 5 takes the install, hook and lifecycle commands with the install, on-disk, hook and injection tests.
- D-06 counts 23 tagged messages in all. This plan holds the 8 in `crates/terminus/tests`; plans 3 and 4 hold the 15 in `crates/terminus-core/tests`.
- `fuzz/` and `crates/terminus-core/Cargo.toml` carry no tag, so no plan edits them. The guard in plan 7 still reads them.
