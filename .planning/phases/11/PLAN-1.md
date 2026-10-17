---
phase: 11
plan: 1
requirements:
  - DOC-04
files:
  - docs/design/TEMPLATE.md
  - docs/design/0001-overview.md
  - docs/design/README.md
  - DESIGN-BRIEF.md
---

# Phase 11: How Terminus Works - Plan 1 of 3

## Goal

An engineer who wants to see inside Terminus can open `docs/design/`, read an overview that starts with one diagram of the whole system from the hook to ingest to the archive to search to recall, and follow a link to one doc per part, each describing the system as it stands now, with Mermaid diagrams GitHub renders with no other tool. This plan lays the spine: the template, the overview and the index, and points `DESIGN-BRIEF.md` at them.

## Must be true when done

- `docs/design/TEMPLATE.md` has exactly seven `## ` headings, in D-02's order, and none of Baley's requirements, roles and actors, instructions served, build status or open questions sections.
- `docs/design/0001-overview.md` opens with one Mermaid flowchart of the whole system, from Claude Code's transcripts through the hook and the detached ingest to `terminus.db` and its FTS5 index and on to recall over MCP, with injection branching off the hook, and links to each of the seven part docs by the file names fixed below.
- `docs/design/README.md` indexes 0001 to 0008 in number order and states the rules every doc follows: current state only, four-digit numbers never reused, the template's form, the diagram rules, the citation rule, and what is linked to the user docs rather than restated.
- The first line of `DESIGN-BRIEF.md` says it is the original design input and points to `docs/design/`, and that one added line is the file's only change.
- No file this plan writes cites code by line number, styles a diagram or leaves one without a caption, and `/cad-docs-verify` reports no stale claim in any of them.
- The owner has reviewed each of the four files in RustRover, and the overview's diagram renders on GitHub in light and dark theme (human-verify).

## Context

- Locked: D-01 (layout and numbering), D-02 (template), D-03 (diagrams), D-04 (brief line), D-05 (citations), D-07 (link the user docs, do not restate them), D-08 (overview facts), D-18 (repository links), D-21 (voice and RustRover review), all in `.planning/phases/11/CONTEXT.md`. The shape to follow is `/code/baley/docs/design/README.md` and `/code/baley/docs/design/TEMPLATE.md`.
- The eight file names, fixed here and used by plans 2 and 3: `0001-overview.md`, `0002-hooks.md`, `0003-ingest.md`, `0004-archive-and-block-rows.md`, `0005-search.md`, `0006-recall-through-mcp.md`, `0007-injection.md`, `0008-install-and-release.md`, titled `# NNNN: Title` as Baley's are (planner's choice under D-01).
- Before drafting any prose, read `/home/john/.claude/john-voice.md` in full (D-21; Part 3 "Design docs and diagrams" and "README, project docs" bind these files). Open each file in RustRover once it is written, by running `rustrover <absolute path>` (the launcher is on PATH) or through the RustRover MCP server when it is connected.
- Every fact a doc states comes from reading the code it names, never from CONTEXT alone. Where the code and a CONTEXT decision disagree, the doc follows the code and the executor records the difference as a deviation for the owner.
- Out of scope: the body of `DESIGN-BRIEF.md`, any user doc under `docs/`, `CHANGELOG.md`, `.planning/PROJECT.md`, a C4 model, ADRs, a PRD, and any cargo test that reads Markdown.

## Tasks

### Task 1: Write the part-doc template

- **Files:** docs/design/TEMPLATE.md
- **Action:** Create the template every part doc follows, cut down from Baley's `TEMPLATE.md` to D-02's seven sections. It opens with the title line `# NNNN: Title`, then one or two sentences saying the doc states the current design of its part and nothing else, is edited in place when the design changes, leaves the history to git, and never tracks work. Then exactly these seven `## ` headings, in this order and with this wording (the wording is the planner's choice under D-02, and plans 2 and 3 compare every part doc against it): `## 1. Purpose and scope`, `## 2. Terms`, `## 3. Crate, modules and boundaries`, `## 4. Records`, `## 5. Commands and operations`, `## 6. States and workflows`, `## 7. Settings`. Under each, short guidance on what goes there. Section 1: what the part does, what it does not, and which parts it hands work to or takes it from, by doc number. Section 2: a Term and Meaning table. Section 3: the crate, each module or file by its full repo-relative path linked relative to the doc (`../../crates/...`), and the boundary to each other part it talks to, with a link to that part's doc; never `path:line` (D-05). Section 4: the tables and files the part writes and what each holds; the columns of every table are given once, in 0004's schema, and other docs link there rather than repeat them. Section 5: each command and operation the part owns, with its inputs, outputs and refusals, linking `../commands.md` for options and `../json-shapes.md` for `--json` fields and exit codes rather than restating them (D-07). Section 6: the part's diagrams, with a `stateDiagram-v2` skeleton and a `sequenceDiagram` skeleton with an `alt` branch, in the form of Baley's sections 7 and 8, using placeholder participants, each followed by a caption line in the form `*Figure 1. What the diagram shows.*` (D-03; caption form from `/code/baley/docs/design/0002-system-design.md`), and a line naming the four allowed types and the question each answers, from Baley's README diagram table without `classDiagram`. Section 7: a Key and What it changes here table, each key linking its entry in `../configuration.md` for its full effect; a part with no key of its own says so in one sentence. Leave out Baley's header table (Status, Design issue, Requirement prefix, Applies, Related) because Terminus has no design issues, requirement prefixes, ADRs or C4 views, and a status that flips after review would make the doc track work. Leave out Baley's sections 3, 4, 10, 11 and 12, which D-02 drops. The template's placeholders hold no backticked path, command or symbol that does not exist in this repo, and no `path:line` placeholder, so `/cad-docs-verify` has nothing stale to find in it. No diagram carries a `%%{init` line, `style`, `classDef`, `class` or a fill (D-03).
- **Verify:** Run every check from `/code/terminus`. These are "the doc checks" the later tasks name, each run on a file F:
  1. Headings: `grep '^## ' docs/design/TEMPLATE.md` prints exactly the seven headings above, in that order, and nothing else.
  2. Diagrams: `awk 'FNR==1{inb=0;want=0} $0=="\140\140\140mermaid"{inb=1;first=1;n[FILENAME]++;next} inb&&$0=="\140\140\140"{inb=0;want=1;next} inb&&first{first=0;if($1!~/^(flowchart|sequenceDiagram|stateDiagram-v2|erDiagram)$/)print FILENAME":"FNR": type "$1} inb&&/%%[{]|^[ \t]*(style|classDef|linkStyle|class)[ \t]|:::|fill:/{print FILENAME":"FNR": styling"} want&&NF{want=0;if($0!~/^\*Figure [0-9]+\. /)print FILENAME":"FNR": no caption"} END{for(i=1;i<ARGC;i++)if(!(ARGV[i] in n))print ARGV[i]": no diagram"}' F` prints nothing. It flags a block whose first line is not one of the four types, any styling inside a block, a block with no `*Figure N. ` caption as the next non-blank line, and a file with no diagram.
  3. Line citations: `grep -nE '\.(rs|sh|ya?ml|toml|md|jsonl?):[0-9]|#L[0-9]' F` prints nothing.
  4. Current state only: `grep -nE '[Pp]reviously|[Nn]o longer|\bD-[0-9]|[Pp]hase [0-9]|\.planning/|block-framed|rebuilds derived tables only|\b(STOR|ING|RCL|INST|INJ|FEED|OBS|PRIV|RET|REL|DOC)-[0-9]|github\.com/crenshawdev/' F` prints nothing.
  5. Links: `grep -oE '\]\([^)]+\)' F | sed -E 's/^\]\(//; s/\)$//; s/#.*$//' | grep -vE '^(https?:|mailto:|$)' | while read -r p; do [ -e "docs/design/$p" ] || echo "MISSING $p"; done` prints nothing.
  6. Running `/cad-docs-verify docs/design/TEMPLATE.md` reports 0 stale claims.
  7. human-verify (RustRover): the owner opens `docs/design/TEMPLATE.md` in RustRover and confirms its Markdown preview renders both skeleton diagrams and the guidance reads as he wants it.

### Task 2: Write the overview, opening on the whole-system diagram

- **Files:** docs/design/0001-overview.md
- **Action:** Create `# 0001: Overview`. The overview is not a part doc and does not use the template. The first non-blank line after the title opens a `mermaid` fence holding a `flowchart` of the whole system as D-08 draws it: Claude Code writing transcripts under the `projects` folder of each root (from `roots`, else `$CLAUDE_CONFIG_DIR`, else `~/.claude`); Claude Code running `terminus hook <event>`; the hook starting the detached `terminus ingest`; the ingest reading the transcripts and writing `terminus.db`, with the archive's block rows, the derived tables and the FTS5 index; Claude Code starting `terminus mcp`, which serves `recall_search`, `recall_context` and `recall_get` from the store read-only; and injection branching off the hook back to Claude Code, the brief on `SessionStart` and earlier turns on `UserPromptSubmit`. It may also show the CLI reading the same store. Quote any label holding `/`, `*`, `(`, `)`, `{` or `}`, and write angle brackets as Mermaid entity codes (`#lt;` and `#gt;`) or leave them out, so GitHub's parser takes the label as text. A `*Figure 1. ...*` caption follows the diagram. After it, prose covering D-08 in full, read from the code: one Rust workspace with two crates, `crates/terminus` the binary (the `lexopt` CLI and `USAGE` in `crates/terminus/src/lib.rs`, `cmd/*`, `out.rs`, and `cmd/json.rs` as the one `--json` envelope, linking `../json-shapes.md`) and `crates/terminus-core` the library (store, ingest, index, recall, inject, feedback, observe, retention, config); the one binary's three jobs (the hook, the stdio MCP server, the CLI); the data directory's resolution order (`TERMINUS_DATA_DIR`, then the `data-location` pointer, then `$XDG_DATA_HOME/terminus` or `~/.local/share/terminus` on Linux and `~/Library/Application Support/terminus` on macOS) and what it holds (`terminus.db` with `-wal` and `-shm`, `LOCK`, `snapshots/`, `injection/`, `decisions/`), with `crates/terminus-core/src/owner_only.rs` creating what Terminus writes there readable only by its user (planner's placement; D-06 leaves it unplaced); where `terminus.toml` resolves from (`TERMINUS_CONFIG_DIR`, then `$XDG_CONFIG_HOME/terminus`, then `~/.config/terminus`, D-06), with `crates/terminus-core/src/config.rs`, linking `../configuration.md` for every key; `ARCHIVE_FORMAT` 2 and `DERIVED_SCHEMA` 5, linking 0004 for what each gates; and no async runtime and no `rayon`, with threads started only in `ingest::backfill` as `std::thread` workers sharing one SQLite writer. Then one section linking each of the seven part docs by the file names in Context, with one sentence each on what it covers. Start reading at `crates/terminus/src/lib.rs` (`USAGE`), `crates/terminus-core/src/lib.rs`, `crates/terminus-core/src/store/open.rs` (`ARCHIVE_FORMAT`, `DERIVED_SCHEMA` and the data directory resolution), `crates/terminus-core/src/ingest/lock.rs`, `crates/terminus-core/src/store/snapshot.rs`, `crates/terminus-core/src/config.rs` and `crates/terminus-core/src/ingest/backfill.rs`. Name only values the code or a test in the repo pins; cite no measured corpus figure, because `/cad-docs-verify` cannot check one against the code and it goes stale as the corpus grows (planner's call under the CONTEXT's flagged assumption).
- **Verify:** From `/code/terminus`, with F = `docs/design/0001-overview.md`:
  1. `awk 'NR>1 && NF {print; exit}' F` prints a line of three backticks followed by `mermaid`, and the line after it starts with `flowchart`.
  2. Within that first block, `for w in hook ingest terminus.db FTS5 mcp recall_search recall_context recall_get projects SessionStart UserPromptSubmit; do awk '$0=="\140\140\140mermaid"{f=1;next} f&&$0=="\140\140\140"{exit} f' F | grep -qF -- "$w" || echo "missing $w"; done` prints nothing.
  3. `for w in ARCHIVE_FORMAT DERIVED_SCHEMA TERMINUS_DATA_DIR data-location XDG_DATA_HOME 'Application Support' TERMINUS_CONFIG_DIR XDG_CONFIG_HOME LOCK snapshots/ injection/ decisions/ lexopt std::thread rayon; do grep -qF -- "$w" F || echo "missing $w"; done` prints nothing.
  4. Doc checks 2 to 4 from Task 1 print nothing for F.
  5. Doc check 5 prints exactly seven lines, `MISSING` for each of `0002-hooks.md`, `0003-ingest.md`, `0004-archive-and-block-rows.md`, `0005-search.md`, `0006-recall-through-mcp.md`, `0007-injection.md` and `0008-install-and-release.md`, which plans 2 and 3 write, and nothing else.
  6. `/cad-docs-verify docs/design/0001-overview.md` reports 0 stale claims.
  7. human-verify (RustRover): the owner opens F in RustRover, the whole-system diagram renders in its Markdown preview, and the doc stands as written.
  8. human-verify (github.com in a browser, once the branch is pushed): the diagram renders with no parse error and reads in both GitHub's light and dark themes.

### Task 3: Write the index

- **Files:** docs/design/README.md
- **Action:** Create the folder's index in the shape of Baley's `docs/design/README.md`, cut to what Terminus has (D-01). It says what the folder is: how Terminus works as it stands, for an engineer who wants to see inside it. It states the rules: each doc gives the current design only, is edited in place when the design changes, carries no amendment lists or superseded copies, leaves the history to git, and never tracks work. Phrase that rule without the words the current-state check bans, since the index is checked too. Numbers are four digits, assigned in order and never reused. The overview has its own form and opens on the whole-system diagram; every part doc follows [TEMPLATE.md](TEMPLATE.md)'s seven sections in order, and a section with nothing to say keeps its heading and says why in one sentence. Diagrams are Mermaid in the Markdown, linking https://mermaid.js.org/, of four types only, given as a table of the question each answers (Baley's table without `classDiagram` and without the C4 export); each has a caption line under it in the form `*Figure N. ...*`; none sets a `%%{init` line, `style`, `classDef` or a fill, so GitHub themes it for light and dark (D-03). Code is named by its full repo-relative path and linked relative to the doc, never cited by line (D-05). What the user docs already pin is linked and not restated: every option in `../commands.md`, every key and default in `../configuration.md`, the `--json` fields and exit codes in `../json-shapes.md`, install behaviour in `../install.md` and the release checks in `../release.md` (D-07). [DESIGN-BRIEF.md](../../DESIGN-BRIEF.md) is the original design input and is not kept current (matches D-04's line). Then the index: a Document and What it covers table with one row per doc, 0001 to 0008 in number order, each linked by the file names in Context and titled as in Context. There is no Status column, because a status that changes after review would make the index track work (planner's choice). Leave out Baley's PRD, ADR, glossary, C4 and Structurizr, requirements and traceability, review and approval, tooling and tracking material: Terminus has none of those artifacts (D-01), and the CONTEXT sets no process for them.
- **Verify:** From `/code/terminus`, with F = `docs/design/README.md`:
  1. `grep -E '^\|' F | grep -oE '\(0[0-9]{3}-[a-z-]+\.md\)' | awk '!seen[$0]++'` prints the eight file names from Context, 0001 to 0008, in that order, and no other.
  2. `grep -cF '(TEMPLATE.md)' F` prints at least 1.
  3. Doc check 2 from Task 1 prints exactly one line, `docs/design/README.md: no diagram`; doc checks 3 and 4 print nothing.
  4. Doc check 5 prints `MISSING` for exactly the seven part docs listed in Task 2's Verify, and nothing else.
  5. `/cad-docs-verify docs/design/README.md` reports 0 stale claims.
  6. human-verify (RustRover): the owner opens F in RustRover and the index stands as written.

### Task 4: Point DESIGN-BRIEF.md at the design docs

- **Files:** DESIGN-BRIEF.md
- **Action:** Insert exactly one line above the current first line, `# Terminus — Design Brief`, saying in one plain sentence that this file is the original design input and the current design is in `docs/design/`, linking `docs/design/README.md` (D-04). Add no blank line after it: a heading may follow a paragraph line directly in Markdown, so every existing line moves down by exactly one and none changes. Do not edit the body. Its stale claims stay as they are, because 30 files under `crates/` and `fuzz/` cite it by `DESIGN-BRIEF.md:NNN` (D-04).
- **Verify:** From `/code/terminus`:
  1. `git diff --numstat -- DESIGN-BRIEF.md` reports 1 line added and 0 removed.
  2. `tail -n +2 DESIGN-BRIEF.md | diff - <(git show HEAD:DESIGN-BRIEF.md)` prints nothing.
  3. `sed -n 1p DESIGN-BRIEF.md | grep -cF 'docs/design/'` prints 1, and that line says the file is the original design input.
  4. human-verify (RustRover): the owner opens `DESIGN-BRIEF.md` in RustRover and the line reads as he wants it.

## Notes

- This plan builds the spine plans 2 and 3 fill in. Plans 2 and 3 build on its output (the template's headings, the file names and the index's links), so the three plans run in order, 1, 2, 3, and are not parallel-safe even though no two declare the same file.
- Until plans 2 and 3 land, the overview's and the index's links to the seven part docs point at files that do not exist yet. Each part doc's task in plans 2 and 3 resolves its link, and plan 3's last task checks every link in the folder.
- The one line Task 4 adds moves every line of `DESIGN-BRIEF.md` down by one, so each of the 45 `DESIGN-BRIEF.md:NNN` citations in 30 files under `crates/` and `fuzz/` points one line later than it does today. D-04 and AC6 lock the line at the top, so the plan adds it as written. Several citations already land a line or two off (`:98`, `:135`, `:163` and `:504` fall on blank lines today).
