---
phase: 11
status: complete
completed: 2026-10-17
---

# Phase 11: How Terminus Works - Summary

Ten Markdown files under `docs/design/` describe Terminus as it stands, with 20 Mermaid diagrams: an overview on one diagram of the whole system, one doc per part from the hooks to install and release, a seven-section template every part doc follows, and an index. The design brief and the README point at them, and the README's #26 gap bullet is gone. Twelve docs-only commits on `encyclopedists`, 2036 lines added.

## What shipped

- The seven-section template every part doc keeps to - `docs/design/TEMPLATE.md`
- The overview, opening on one diagram from Claude Code's transcripts through the hook, the detached ingest, `terminus.db` and its FTS5 index, to recall over MCP, with injection branching off the hook, and linking each part doc once - `docs/design/0001-overview.md`
- The index of the folder and the rules the docs keep - `docs/design/README.md`
- The hooks doc: one hook run starting the ingest and handing back context - `docs/design/0002-hooks.md`
- The ingest doc: one pass from the lock to the snapshot - `docs/design/0003-ingest.md`
- The archive doc: every table and column once, on the block rows they rest on - `docs/design/0004-archive-and-block-rows.md`
- The search doc: a query from the words typed to the excerpt - `docs/design/0005-search.md`
- The recall doc: what each of the three MCP tools takes and hands back, from the field names they share with `terminus search --json` (D-14) - `docs/design/0006-recall-through-mcp.md`
- The injection doc: a prompt from its spellings to the label it earns, with the compaction flag - `docs/design/0007-injection.md`
- The install and release doc: a release from the tag to the install that checks it - `docs/design/0008-install-and-release.md`
- The design brief's first line says it is the original design input and points at `docs/design/` - `DESIGN-BRIEF.md`
- README points at the design docs in "How it works" and "How it is designed and built", and the #26 bullet is dropped from "Current state" - `README.md`

Every file was opened in RustRover and the owner approved each as written. The full suite (`cargo test --workspace --all-features --locked`) was green at the end of every plan: 902 passed, 0 failed, across 67 test binaries.

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 760cb14 | give the design docs a seven-section template |
| 1 | 2 | 21cc54a | open the design docs on one diagram of the whole system |
| 1 | 3 | c87aea3 | index the design docs and state the rules they keep |
| 1 | 4 | adce27d | point the design brief at the current design |
| 2 | 1 | c0a78a9 | show how one hook run starts the ingest and hands back context |
| 2 | 2 | 57b2cfa | walk one ingest pass from the lock to the snapshot |
| 2 | 3 | 60ea0a6 | give every table and column once, with the block rows they rest on |
| 2 | 4 | 3adbf77 | follow a query from the words typed to the excerpt |
| 3 | 1 | a15a313 | show what each recall tool takes and hands back |
| 3 | 2 | 61ce111 | follow a prompt from its spellings to the label it earns |
| 3 | 3 | 30c3d3f | trace a release from the tag to the install it checks |
| 3 | 4 | 466d542 | point the README at the design docs and drop the #26 gap |

## Deviations

Each is a place where the code disagreed with a context decision. The doc follows the code in every case, and the decision's line in CONTEXT.md carries the correction.

- [deviation] D-08 said threads start only in `ingest::backfill`. That is true of `terminus-core` alone: the binary also starts threads in `crates/terminus/src/cmd/hook.rs` (the stdin reader and the injection worker, both under a deadline and never joined) and in `crates/terminus/src/cmd/install/claude.rs` (draining `claude mcp`'s stderr). The overview names all three. Commit 21cc54a.
- [deviation] D-10 gave the per-file write order as project through git, lineage, block rows, then turns, `compactions`, `turns_fts`, `entities`, `paths`, then the watermark. The code cuts and compresses the block rows and finds a sidecar's parent first (in `prepare`), resolves the project and the continued session outside the transaction (in `apply`), then inside it writes the session row, block rows and `session_meta`, then per turn `turns`, `turns_fts`, `entities`, `paths` and the turn's `compactions` row, then the boundary-only `compactions` rows, then the watermark. Figure 2 in 0003 follows the code; the pass-level order D-10 gives is right. Commit 57b2cfa.
- [deviation] D-12 said indexing adds camel, snake, kebab and path parts at ingest. `index/expand.rs` appends camelCase and PascalCase pieces only; `unicode61` already splits snake, kebab and path parts, and the separator rule only seeds the set of tokens not to append again. 0005 says that and names all four rules. Commit 3adbf77.
- [deviation] D-13 said request ids must be integers. `rpc::parse` accepts a string or an integer that fits an `i64` or `u64`, echoes either back as sent, and refuses any other id, a float included, with `-32600` and a null id. 0006 follows the code. Commit a15a313.
- [deviation] D-13 said a failure returns an empty result with a `reason`, never a JSON-RPC error. That holds once a `tools/call` names one of the three tools. A `tools/call` with no `params.name`, or an unknown name, gets `-32602` from `tools::call`; of the empty results, unusable arguments carry `isError` true and an archive with no answer carries `isError` false. 0006 says all three. Commit a15a313.
- [deviation] D-15 said the compacted pool is filtered to turns before the `compactions` row's offset. `inject::compaction::dropped` keeps the session's turns with a lower `stream_offset` than its latest boundary, less every turn whose uuid `compactMetadata.preservedMessages.uuids` lists, leaving out a turn with no uuid; an unreadable or empty preserved list gives an empty set. The flag also stays set past a prompt with no spellings, no store or no keyed `transcript_path`. 0007 follows the code. Commit 61ce111.

## Open items

- The diagrams have not been checked on github.com in light and dark theme: the overview, the template skeletons, and all seven part docs (plan 1 task 2 check 8; plan 2 checks 9, 6, 7 and 5; plan 3 checks 10, 6 and 5). That needs the branch pushed, which happens at land time.
- Issue #26 is closed by the phase's PR, whose description carries a `Fixes #26` line (D-14). The PR is written at land time.
- The `mimalloc` comment in the root `Cargo.toml` still says backfill allocates "across rayon threads". Out of every plan's lease.
- `terminus backfill` runs lock, recovery, discovery, the walk and the `runs` row only; it does not drain decisions, label, observe, apply retention or snapshot (`crates/terminus-core/src/ingest/backfill.rs`). 0003 says so. Recorded so the install doc keeps to the same fact.

## Goal check

The twelve commits deliver the phase goal. Success criterion 1: `docs/design/0001-overview.md` (21cc54a) opens on one Mermaid diagram of the whole system and links each of the seven part docs once in its parts table, which plan 1's link check confirmed by printing exactly the seven then-missing part docs. Success criterion 2: each of the seven parts has its own doc with at least one diagram, 0002 through 0008 (c0a78a9, 57b2cfa, 60ea0a6, 3adbf77, a15a313, 61ce111, 30c3d3f), and plan 3 task 4's phase-wide run found all 20 Mermaid blocks in the folder parse under mermaid 12.1.0, with the only "no diagram" line coming from the index. Every doc follows `TEMPLATE.md`'s seven headings, cites no line numbers and carries no change-log wording, per the heading, citation and current-state checks each plan re-ran on its final HEAD. The owner approved every file in RustRover. What is not yet shown: that GitHub renders the diagrams in both themes, which needs the push, and the #26 closure, which needs the PR. Neither is a gap in the docs themselves.
