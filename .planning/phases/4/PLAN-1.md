---
phase: 4
plan: 1
requirements: [SRCH-01]
files:
  - crates/terminus-core/src/recall/unindexable.rs
  - crates/terminus-core/src/recall/mod.rs
  - crates/terminus-core/src/recall/query.rs
  - crates/terminus-core/tests/recall.rs
  - crates/terminus/tests/recall_cli.rs
  - crates/terminus/tests/mcp.rs
  - docs/design/0005-search.md
---

# Phase 4: Search - Plan 1 of 3 (unindexable words, #23)

## Goal

Search answers an unindexable word, checks dates as dates, and binds every list as one parameter (#23, #24, #108). This plan covers #23: a query word the bundled SQLite's `unicode61` tokenizer makes no token from is dropped when the query is parsed, so the other words still find their hits.

## Must be true when done

- `terminus search --project '*' cargo` and `terminus search --project '*' cargo ͅ` (U+0345 as its own word) print the same hits, and MCP `recall_search` answers the two queries with the same hits.
- A search for a word plus the pointed Hebrew word שָׁלוֹם returns only the turns that hold both words.
- A query made only of words the tokenizer makes no token from answers "the query has no words that can be searched", the same as a query of punctuation.
- A test runs all 149,106 code points Rust counts as alphanumeric through the bundled SQLite's tokenizer. It passes on the tree and fails when a range is removed from the Rust table.
- `docs/design/0005-search.md` no longer says `unicode61` splits on every character that is not a letter or a digit.
- `cargo test --workspace --all-features` passes.

## Context

- Locked: D-01, D-02, D-03, and D-12 and D-13 as they apply to #23. Source: `.planning/phases/4/CONTEXT.md`, phase 4.
- Measured while planning (2026-10-23, rusqlite 0.40.2 bundled, SQLite 3.53.2, debug build): one row per code point in an in-memory FTS5 table, read back through an `fts5vocab` table of type `instance`, gives 149,106 alphanumerics and 1,061 with no token, in 167 runs, none numeric, 131 cased. That matches D-01. The sweep took 0.5 s. The same probe confirmed D-02: against rows `'foo bar'` and `'foo שָׁלוֹם baz'`, `"foo" AND "ͅ"` gives no hit, `"fooͅbar"` gives row 1, and `"foo" AND "שָׁלוֹם"` gives row 2 only.
- Out of scope: index-side tokenization, `separator_components` and `index/expand.rs`; any new crate; running `cargo fmt` over the workspace (D-13). New tests and comments may cite `#23`, never a requirement id or a decision tag (`tests/planning_tags.rs` fails on those).
- Plans run in order 1, 2, 3. This plan shares `crates/terminus-core/src/recall/mod.rs`, `crates/terminus-core/tests/recall.rs`, `crates/terminus/tests/recall_cli.rs`, `crates/terminus/tests/mcp.rs` and `docs/design/0005-search.md` with plan 2, and the last four with plan 3.

## Tasks

### Task 1: Drop query words the bundled tokenizer makes no token from

- **Files:** crates/terminus-core/src/recall/unindexable.rs (new), crates/terminus-core/src/recall/mod.rs (the module list), crates/terminus-core/src/recall/query.rs (`Query::parse`, the module docs, `Query::is_empty`), crates/terminus-core/tests/recall.rs
- **Action:** Create `recall/unindexable.rs`, declared as a private module of `recall` and not re-exported, because nothing outside the query layer needs it. It holds a sorted table of inclusive code point ranges and a lookup that answers, for one `char`, whether it is in the table, by binary search over the ranges. The table is every code point `char::is_alphanumeric` accepts on which the bundled SQLite's `unicode61` tokenizer makes no token. That is the default tokenizer, and the one `turns_fts` uses, since `store/schema.rs` gives it no tokenize option. The table is plain Rust (D-01): no connection, no build script, no Unicode crate. Put the sweep test in the same file as a `#[cfg(test)]` unit test, so it can reach the private table. The test opens an in-memory connection, which is the bundled SQLite this crate links, and creates an FTS5 table with no tokenize option. In one transaction it inserts each Rust-alphanumeric code point as its own row, with the code point as the rowid. It reads which rowids produced at least one token from an `fts5vocab` table of type `instance` over that table. Then, for every Rust-alphanumeric code point, it asserts that the lookup says the code point is in the table exactly when SQLite made no token from it. When they disagree, the failure message names the disagreeing code points and prints the whole table as SQLite now computes it, in the table's own source syntax: maximal runs of consecutive code points. Regenerating the table after a rusqlite bump is then pasting that message, and this is also how you fill the table the first time: write it empty, run the test, paste. Do not hard-code the totals in the test, because a legitimate SQLite change would then fail it twice. The module docs say why the table exists (#23), which SQLite it was measured against (3.53.2, through rusqlite 0.40.2 `bundled`), that the sweep test pins it, and how to regenerate it. In `Query::parse`, drop a piece from `separator_components` when every one of its characters is in the table (D-02). Drop it before the case-folded dedup and before the `MAX_QUERY_TOKENS` cap (D-03), so a dropped word neither takes a slot nor sets `truncated`. A piece with at least one character outside the table is kept whole and as typed, `fooͅbar` and a pointed Hebrew, Arabic or Devanagari word included, because FTS5 splits it inside the quoted phrase itself. `Query::parse` stays infallible and takes no connection: `fuzz/fuzz_targets/search_query.rs` calls it with no store, `cmd/search.rs` calls it before `read::open`, and `inject/prompt.rs` decides whether to open the store from its result. Leave `separator_components` alone, since the index side is out of scope. Leave the stored-value split in `matches_entity` as it is too: D-03 puts the drop in `parse`, and whether stored values should drop the same words is left for John (see Notes). Rewrite the module docs' paragraph on the rule, and the `is_empty` doc, so they say a word the tokenizer makes no token from is dropped and a query made only of such words is empty. Add tests to `tests/recall.rs`, citing #23, for these cases. `Query::parse("foo \u{345}")` has the tokens `["foo"]`. `Query::parse("\u{345} \u{24B6} \u{24D0}")` is empty and not truncated. `Query::parse("foo\u{345}bar")` keeps `foo\u{345}bar` as one token. The pointed word U+05E9 U+05B8 U+05C1 U+05DC U+05D5 U+05B9 U+05DD comes back as one token, exactly as typed. 32 distinct ordinary words interleaved with U+0345 words give 32 tokens and `truncated()` false.
- **Verify:** `cargo test -p terminus-core --all-features --lib unindexable` passes. Delete one range from the table and run it again: it fails and names that range's code points. Restore the range and it passes. The table's ranges cover 1,061 code points, D-01's measurement. `cargo test -p terminus-core --all-features --test recall --test planning_tags` passes, including the new tests. `cargo check --workspace --all-features --all-targets` prints no `warning:` line.

### Task 2: Prove the drop through search, the CLI and MCP

- **Files:** crates/terminus-core/tests/recall.rs, crates/terminus/tests/recall_cli.rs, crates/terminus/tests/mcp.rs
- **Action:** Add tests that show the drop through every front end, each citing #23. Before task 1 every one of them fails, because a query carrying U+0345 returned no hits.
  - **Core, nothing searchable.** `search::run` with `Scope::Everything` and a query of only U+0345 and U+24B6 answers no hits with `Reason::NothingSearchable`, the path a query of punctuation already takes (D-03).
  - **Core, Hebrew.** Use the recall bench's `archive` helper, as `a_turn_excluded_only_by_its_worktree_key_is_refused_by_get_and_context` does. Write one session with two user turns: one holding a word no fixture holds plus the pointed word from task 1, the other holding that word alone. Under `Scope::Everything`, a search for the word plus the pointed word returns only the first turn, and a search for the word alone returns both.
  - **CLI.** In `recall_cli.rs`, with the fixtures ingested, `search --project '*' --json cargo` and `search --project '*' --json cargo ͅ` (U+0345 as its own argument) both exit 0 and return the same non-empty list of `turn_id`s in the same order. `search --project '*' --json ͅ` exits 0 with no hits and a reason saying the query has no words that can be searched.
  - **MCP.** In `mcp.rs`, `recall_search` with `"project": "*"` returns the same non-empty hit ids for `cargo` and for `cargo ͅ`, with `isError` false. `"*"`, because `cargo` is in no project-alpha turn (see the comment above `recall_cli.rs`'s project-beta test).
- **Verify:** `cargo test -p terminus-core --all-features --test recall` and `cargo test -p terminus --all-features --test recall_cli --test mcp` pass. Each new CLI and MCP test compares two non-empty hit lists, so the zero-hit answer from before task 1 fails it.

### Task 3: Say what `unicode61` does and what the query drops

- **Files:** docs/design/0005-search.md (§2 Terms, the "Query token" row; §3, the module table; §4, the paragraph that says `unicode61` already splits on every character that is not a letter or a digit; §6, Figure 1's `Query::parse` node)
- **Action:** Design docs describe the current design only: no history, and no "now" or "no longer". In §4, `unicode61` makes tokens from characters in the Unicode letter, number and private-use categories and splits on everything else. That includes some characters Rust counts as letters, such as combining marks like U+0345 and circled letters like U+24B6. Keep the paragraph's point that snake_case, kebab-case and path parts are already tokens of their own. In §2, a query token is a piece of the typed string split on every character that is not a letter or a digit, kept only when at least one of its characters is one the tokenizer makes a token from. In §3, add a row for `crates/terminus-core/src/recall/unindexable.rs`, linked the way the other rows are: the code points `unicode61` makes no token from, checked against the bundled SQLite by its own test. In Figure 1, the `Query::parse` node also drops words the tokenizer makes no token from, before the case-folded repeats and the cap of 32.
- **Verify:** `grep -n 'already splits on every character' docs/design/0005-search.md` prints nothing. `grep -n 'U+0345' docs/design/0005-search.md` shows the §4 paragraph. `grep -n 'unindexable.rs' docs/design/0005-search.md` shows the §3 row. The `Query::parse` node in Figure 1 names the drop.

## Notes

- `Query::matches_entity` still splits a stored entity value with `separator_components` and does not drop. A stored value with a word made only of such characters, such as a path segment `ͅ`, can no longer be an exact or covered match for a query that types it, because the query drops that word. D-03 puts the drop in `parse` only. Whether stored values should drop the same words is for John to decide, not this plan.
- The planning probe crate lives in the session scratchpad and is not part of the repository, like the 2026-10-23 probe D-01 cites.
