# Roadmap: Terminus

## Overview

mule takes the bugs open at 1.0.0 and the cleanup the 2026-10-22 simplify review found in the same code, all as filed issues. Dead upgrade code goes first, so no later phase edits code that is about to be deleted. The planning tags go next, so every later phase reads comments that point at something real, and a test stops new ones being written during the cycle. The macOS Intel hook tests come third, so the merge gate means something for the fixes after them. Search, the pass driver, the archive fixes, owner-only files and command output follow, with the pass driver ahead of the two phases that run inside it.

## Phases

- [x] **Phase 1: Old store code** - remove upgrade code for store layouts 1.0.0 can't create
- [x] **Phase 2: Planning tags** - cut the planning tags from comments and keep them out
- [x] **Phase 3: Hook tests on macOS Intel** - stop the hook timing tests failing the merge gate
- [x] **Phase 4: Search** - unindexable words, real dates, and lists bound as one parameter
- [x] **Phase 5: One ingest pass** - one pass driver, and recovery notes no longer read as errors
- [x] **Phase 6: Archive** - continuation links, retention's delete, and the observations query
- [x] **Phase 7: Owner-only files and signing** - one owner-only helper set, and a signer that runs no installer code
- [x] **Phase 8: Output** - escaped text, `--json` failure documents, an honest purge size and cut mark

## Phase Details

### Phase 1: Old store code
**Goal:** Recovery and the store open carry no code for store layouts 1.0.0 can't create (#105)
**Depends on:** Nothing (first phase)
**Requirements:** SHED-01
**Success Criteria:**
1. `crates/terminus-core/src/recover.rs` has no `rekey`, `stale_keys`, `merge` or `remap_ids`, and `crates/terminus-core/tests/session_keys.rs` no longer exists
2. `BRING_FORWARD_COLUMNS` is empty, and `drop_summary_columns`, `DROPPED_OBSERVATION_COLUMNS` and `retire_old_boundaries` no longer exist
3. On a copy of the live store, an ingest pass completes with no error and `terminus verify` reports every block intact
4. `docs/design/0003-ingest.md`, `docs/design/0004-archive-and-block-rows.md`, `docs/commands.md` and `THREAT-MODEL.md` no longer describe a re-key

### Phase 2: Planning tags
**Goal:** No comment cites a tag from the removed planning files, and none can be added (#106)
**Depends on:** Phase 1
**Requirements:** TAGS-01
**Success Criteria:**
1. The pattern in #106 matches no tag in `crates/`, `tests/`, `fuzz/`, `Cargo.toml` or `docs/`; any line it still matches is ordinary prose about a phase
2. A test fails when a source comment cites a planning tag, and passes on the tree
3. Apart from the new test, the phase's diff changes only comment lines

### Phase 3: Hook tests on macOS Intel
**Goal:** The hook timing tests stop failing the merge gate on macOS Intel (#75, #104)
**Depends on:** Nothing
**Requirements:** GATE-01, GATE-02
**Success Criteria:**
1. `reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs` proves the read returned while the pass was running without polling `ps` for a process that may already have exited
2. That test passes on the `macos-15-intel` release job in every CI run of the milestone's pull request
3. When either test named in #104 fails on time, its message says whether the hook process was still blocked or the runner had stalled

### Phase 4: Search
**Goal:** Search answers an unindexable word, checks dates as dates, and binds every list as one parameter (#23, #24, #108)
**Depends on:** Nothing
**Requirements:** SRCH-01, SRCH-02, SRCH-03
**Success Criteria:**
1. A search for `foo` plus a standalone U+0345 returns the same hits as a search for `foo`
2. `terminus search --since 2026-02-30 x` exits non-zero with a message naming `--since`, and MCP `since: "2026-02-30"` returns an error naming `since`
3. `cmd::time_bound` no longer exists, and core rejects a bad date before it resolves scope
4. `recall/search.rs`, `recall/get.rs` and `recall/excerpt.rs` build no `IN (?, ...)` list, and a search with more `--path` values than SQLite's variable limit returns without an error

### Phase 5: One ingest pass
**Goal:** One pass driver runs every pass, and a pass that only recovered is not reported as an error (#31, #96)
**Depends on:** Phase 1
**Requirements:** PASS-01, PASS-02
**Success Criteria:**
1. `backfill::record_pass` and `RunRow` no longer exist, and the tree pass, the backfill and single-file ingest call one driver
2. After a backfill, its `runs` row fills the same columns a tree pass fills, `notes` included
3. For a pass whose only notes come from recovery and that failed no file, `status` prints no `error` line and `doctor` exits 0
4. A pass that fails a file still prints under `error` in `status`, and `doctor` exits 1

### Phase 6: Archive
**Goal:** Continuation links survive ingest order, retention re-checks before it deletes, and observations read only a session's own entities (#11, #43, #70)
**Depends on:** Phase 5
**Requirements:** ARCV-01, ARCV-02, ARCV-03
**Success Criteria:**
1. Ingesting a continuing session before its predecessor, then the predecessor, leaves `continues_from` set on the later session
2. A transcript that comes back between selection and delete keeps its session archived
3. `retention::apply::delete_session` clears derived rows by calling `reindex::clear_derived`, so the foreign-key delete list exists once (from the simplify review, done with #43)
4. The observe query reads a session's entities through its turns in both `observations regenerate` and the ordinary pass, and a test fails if its plan goes back to scanning every entity of a kind

### Phase 7: Owner-only files and signing
**Goal:** Every file Terminus writes is owner-only through one helper set, and signing runs no installer code (#73, #99)
**Depends on:** Nothing
**Requirements:** OWN-01, OWN-02
**Success Criteria:**
1. `terminus data move` creates its pointer 0600 and the config directory 0700
2. No exclusive-create retry loop exists outside `crates/terminus-core/src/owner_only.rs`, and the seven sites named in #73 and its comment call its helpers
3. The session-id check that keeps a hook's session id out of a path exists once
4. `scripts/sign-release.sh` has no `eval` and reads no code out of `install.sh`, and a CI test fails when its copies of the version check, digest and manifest check differ from `install.sh`

### Phase 8: Output
**Goal:** Commands escape archived text, answer a `--json` failure with a document, and say what they really do (#72, #107, #95, #94)
**Depends on:** Phase 5
**Requirements:** OUT-01, OUT-02, OUT-03, OUT-04
**Success Criteria:**
1. Each of the nine commands #72 lists prints ESC and BEL in archived text as escapes, while `--json`, the hook document and `show`'s record are byte for byte the archived text
2. `compact`, `reindex`, `observations regenerate`, `stats` and `replay`, made to fail under `--json`, each print a document with `ok: false`, and `read::unusable` no longer exists
3. The install estimate and `status` print the same size in the same unit
4. The size `uninstall --purge` shows before it asks includes `snapshots/`
5. A prompt-line share of 1 to 3 characters shows a cut mark rather than a bare fragment, or leaves the turn out when the budget cannot hold the mark
