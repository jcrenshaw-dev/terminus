---
phase: 4
plan: 3
requirements: [ING-13]
files:
  - THREAT-MODEL.md
  - docs/design/0003-ingest.md
  - docs/design/0004-archive-and-block-rows.md
  - docs/json-shapes.md
  - CHANGELOG.md
---

# Phase 4: Session keys - Plan 3 of 3: the docs

## Goal

The documents that define the session key describe the key Plans 1 and 2 deliver, and the changelog records the fix to #57 and the `session_key` output change, so the docs and the code describing the key land together (D-15).

## Must be true when done

- `THREAT-MODEL.md`, `docs/json-shapes.md` and design docs 0003 and 0004 describe the key as the transcript's path below `projects`. They also say that a transcript which is a symlink takes the key of the file it points at, and that a file under no `projects` folder keeps its canonical path. None of them still says the key is the canonical path (AC5).
- `CHANGELOG.md` `[Unreleased]` carries #57 under `### Fixed`, including that `session_key` in `--json` and MCP output is now the path below `projects`, and that a script which opens it as a file breaks (AC5, D-13).
- (human-verify: GitHub, after landing) The pull request that lands this phase carries `Fixes #57`, and #57 closes once it merges (AC7).

## Context

- Depends on Plans 1 and 2. The docs describe what those plans built, so read the merged code before writing.
- Locked: D-15 (which docs and which lines), D-13 (output change and changelog wording), D-08 (no `repair` command, so the docs must not invent one).
- This is prose John puts his name on. Read `~/.claude/john-voice.md` in full first, from the file, and follow its rules for project docs and for the changelog.
- Out of scope: module comments, which are Plans 1 and 2's. Also out: docs D-15 does not name. The return marker lists one candidate.

## Tasks

### Task 1: Make the docs that define the key describe the new key

- **Files:** THREAT-MODEL.md (lines 12, 21 and 240, under "Transcripts", "`terminus ingest <path>`" and "Session identity"), docs/design/0003-ingest.md (the "Session key" and "Watermark" rows of the terms table), docs/design/0004-archive-and-block-rows.md (the "Session" term, the `sessions.session_key` and `session_meta.parent_session_key` column rows, the `watermarks.transcript_path` row, and the `sessions |o--o| watermarks` edge of the ER diagram), docs/json-shapes.md (the `search` example at line 77, the `status` example at line 304, the `retention` `evict` and `delete` description at line 381, the `export` manifest at line 530, and every other example or sentence that shows `session_key` as an absolute path)
- **Action:**
  - **What the docs must say.** The session key is the transcript's path below the `projects` folder: `<project dir>/<uuid>.jsonl`, or `<project dir>/<sessionId>/subagents/[workflows/wf_*/]agent-*.jsonl` for a sidecar. It is found from the shape of the path and not by stripping a root, so it survives a move or a symlinked `.claude`. A transcript that is a symlink takes the key of the file it points at, and a file under no `projects` folder keeps its canonical path. The absolute path the transcript was read from stays in `session_meta.transcript_path`.
  - **Line 240.** It is a security claim. Keep its point: a forged `sessionId` cannot overwrite another session, because the key is computed from the path before any byte is read and never from record content (D-02). Add what is now also true: two different files that reduce to one key are never appended to one session. The walk ingests the first and reports the other (Plan 1, Task 2).
  - **Lines 12 and 21.** State that the key no longer depends on the root.
  - **docs/json-shapes.md.** Show keys in the new form, such as `-code-verbatim/1111....jsonl`, and stop calling `evict` and `delete` "the canonical transcript paths".
  - **Design doc 0004.** Describe `parent_session_key` as the parent's key in the same form.
  - **The repair.** Describe it only where one of these lines already describes the key's lifecycle. It is part of recovery and has no command (D-08).
  - **Avoid.** Leaving a claim the code no longer makes. Inventing a command, option or field. Changing lines D-15 does not name, except the other `session_key` examples in json-shapes.md, which D-15 names as a whole file.
- **Verify:**
  - Searching the four files for `canonical path`, `canonical file path`, `canonical transcript path`, `/abs/path/session.jsonl` and `/home/you/.claude/projects` finds no sentence or example that describes `session_key`.
  - Each of D-15's lines in these four files describes the path below `projects`.
  - `THREAT-MODEL.md`'s "Session identity" bullet still states the forged-`sessionId` property.
  - (human-verify: John reads each changed doc in RustRover) The wording passes his review.

### Task 2: Record #57 in the changelog

- **Files:** CHANGELOG.md (`## [Unreleased]`, `### Fixed`)
- **Action:** Add one entry under `[Unreleased]` `### Fixed`, in the prose style of the entries around it, ending with `(#57)`. Say what was wrong: when `~/.claude` moved behind a symlink, or was reached through one, every session was archived a second time. Search and context results showed twice, and `terminus status` counted about double. Say what is fixed: a session is keyed on its transcript's path below `projects`, and a symlinked sidecar is archived once.

  Say that the first ingest after upgrading repairs a store that is already doubled. It merges each pair that is provably one transcript into the older session. It keeps every session that exists only in the archive. It names, in that run's errors, any pair it cannot prove is one transcript, and keeps both.

  Say plainly, per D-13, that `session_key` in `--json` output and in `recall_search` and `recall_get` results is now the path below `projects` and not an absolute path, so a script that opens `session_key` as a file breaks. Name no command for the repair, because there is none (D-08).
- **Verify:**
  - `CHANGELOG.md` `[Unreleased]` `### Fixed` holds one entry that contains `#57`, the phrase `session_key`, and the statement that a script opening it as a file breaks.
  - No other section changed: `git diff CHANGELOG.md` touches only lines inside that one `### Fixed`.

## Notes

- AC7 is the orchestrator's: the pull request body carries `Fixes #57`, and #57 is checked closed after the merge.
- John reviews docs himself. Open each file changed here in RustRover once it is written.
