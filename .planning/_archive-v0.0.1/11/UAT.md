---
status: testing
phase: 11
fields_version: 1
started: 2026-10-17
updated: 2026-10-17
---

## Items

### 1. docs/design holds the index, the template and eight numbered docs, indexed and on the template's headings
expected: `ls docs/design` lists README.md, TEMPLATE.md and 0001-overview.md through 0008-install-and-release.md; README.md links all eight; each of 0002 to 0008 has exactly TEMPLATE.md's `##` headings in the same order.
criterion: AC1
status: pass
first_pass: pass
source: model
evidence: `ls docs/design` -> 0001-overview.md 0002-hooks.md 0003-ingest.md 0004-archive-and-block-rows.md 0005-search.md 0006-recall-through-mcp.md 0007-injection.md 0008-install-and-release.md README.md TEMPLATE.md. `grep -c <name>.md docs/design/README.md` for each of the eight -> 0001: 2, 0002-0008: 1 each. `diff <(grep '^## ' <doc>) <(grep '^## ' TEMPLATE.md)` for 0002-0008 -> no difference in any of the seven; the template's seven headings are 1. Purpose and scope, 2. Terms, 3. Crate, modules and boundaries, 4. Records, 5. Commands and operations, 6. States and workflows, 7. Settings.

### 2. The overview opens on one whole-system diagram and links the seven part docs
expected: The first Mermaid block in 0001-overview.md is one diagram running from the hook to ingest to the archive to search to recall, and 0001 links each of 0002 to 0008.
criterion: AC2
status: pass
first_pass: pass
source: model
evidence: `grep -c '^```mermaid' docs/design/0001-overview.md` -> 1. That block is a `flowchart TD` whose nodes are Claude Code, the transcripts, `terminus hook <event>`, `terminus ingest` detached, `terminus.db` (archive block rows, derived tables, FTS5 index), injection, the decisions/ and injection/ files, `terminus mcp` over stdio, and the CLI search/show/sessions; edges run CC -> transcripts, CC -> hook -> ingest -> DB, hook -> injection -> DB and files, CC -> MCP -> DB with recall_search/recall_context/recall_get back to CC. `grep -c <part>.md docs/design/0001-overview.md` -> 1 for each of 0002-0008.

### 3. Every part doc has a captioned, unstyled Mermaid diagram of an allowed type
expected: Each of 0002 to 0008 holds at least one mermaid block whose type is flowchart, sequenceDiagram, stateDiagram-v2 or erDiagram, with a caption line under it; no block under docs/design sets a theme, `style`, `classDef` or fill.
criterion: AC3
status: pass
first_pass: pass
source: model
evidence: awk over each ```mermaid block in 0002-0008, printing its first token and the first non-blank line after the closing fence: 0002 sequenceDiagram + flowchart; 0003 flowchart + sequenceDiagram + stateDiagram-v2; 0004 erDiagram + stateDiagram-v2 + flowchart; 0005 flowchart + flowchart; 0006 sequenceDiagram + flowchart; 0007 stateDiagram-v2 + flowchart + sequenceDiagram; 0008 sequenceDiagram + flowchart. Every block is followed by an italic `*Figure N. ...*` caption. Type census over docs/design/*.md -> 9 flowchart, 6 sequenceDiagram, 4 stateDiagram-v2, 1 erDiagram, nothing else. `grep -i -E 'theme|^\s*style |classDef|fill'` inside every mermaid block -> two hits, both the word `backfill` in node labels in 0008 lines 53-54; no theme, style, classDef or fill directive anywhere.

### 4. Every diagram renders on GitHub and reads in the dark theme
expected: On github.com, each Mermaid block under docs/design renders as a diagram with no parse error and is readable with the dark theme on. (human-verify: needs github.com in a browser)
criterion: AC4
status: pass
first_pass: pass
reported: All good go

### 5. Docs verify reports no stale claim and no design doc cites code by line number
expected: `/cad-docs-verify` over README.md and docs/ reports no stale claim, and no file under docs/design cites code as `path:line`.
criterion: AC5
status: pass
first_pass: fail
source: model
evidence: Retest after 4ace2f5. `git show -U0 4ace2f5` -> 7 lines changed across docs/design/0003-ingest.md, 0004-archive-and-block-rows.md (2), 0007-injection.md (2), 0008-install-and-release.md and docs/commands.md; each new sentence states what the cited code does (backfill.rs:418-443 and pass.rs record_pass for the runs row; install/mod.rs:344-358 for the too-old-only refusal; `grep -rn integrity_check crates/` for nothing in the binary; brief.rs:149-158 for the brief arm's save; archive.rs:39-64 and excerpt.rs:116-124 for the empty-excerpt hit; json_file.rs:526-538 and install/mod.rs:166-179 for the full rewrite). `grep -n -E '\.(rs|sh|yml|toml)[:#]L?[0-9]+' docs/design/*.md` -> no match. Headings of 0002-0008 still diff clean against TEMPLATE.md. The 917-claim mechanical pass and the three verifiers' other 727 accurate claims are untouched by a prose-only edit; the edits add no link, constant, flag or path. No stale claim remains.
severity: minor
cause: Seven sentences state more than the code does. Six are in the design docs this phase wrote (0003:99, 0004:52, 0004:259, 0007:58, 0007:108, 0008:37) and one is in docs/commands.md:203 from phase 9. Each is a wording overreach, not a wrong mechanism: the fix is to narrow each sentence to what the code does, as docs/install.md:58 already does for the install refusal.
fix: 4ace2f5, retest

### 6. The recall doc states each tool's return, and README and the design brief point at the design docs with the #26 gap gone
expected: 0006 says what each of recall_search, recall_context and recall_get returns; README.md has no #26 gap line and links docs/design/README.md from both "How it works" and "How it is designed and built"; the first line of DESIGN-BRIEF.md says it is the original design input and points to docs/design/.
criterion: AC6
status: pass
first_pass: pass
source: model
evidence: `grep -n '^### ' docs/design/0006-recall-through-mcp.md` -> `### recall_search` (108), `### recall_context` (144), `### recall_get` (177), each a section 5 subsection stating the tool's result shape. `grep -n -E 'issues/26|#26' README.md` -> no match. `grep -n docs/design/README.md README.md` -> line 64, under `## How it works` (52), and line 146, under `## How it is designed and built` (144). `head -1 DESIGN-BRIEF.md` -> `This file is the original design input, and the current design of Terminus is in [docs/design/](docs/design/README.md).`

### 7. The phase's PR closes #26
expected: After the phase's PR merges, issue #26 on GitHub is closed by it. (human-verify: needs GitHub after the PR merges)
criterion: AC6
status: skipped
reported: Skip
reason: No PR exists yet; it is written at land time with a Fixes #26 line and the issue closes when it merges.

### 8. The owner reviewed each file under docs/design in RustRover
expected: Each of the ten files under docs/design was opened in RustRover and reviewed by the owner. (human-verify: needs the owner)
criterion: AC7
status: pass
first_pass: pass
reported: Approved go

## Summary

total: 8
passed: 7
failed: 0
pending: 0
skipped: 1
blocked: 0
reworked: 1
