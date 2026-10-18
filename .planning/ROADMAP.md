# Roadmap: Terminus

## Overview

The archive is the product, so it is built first and everything else is built on top of it: a store that can hold a session losslessly and prove it, then an ingest that survives a real transcript tree, then a query path that finds a turn from the terminal and from the model. Only once recall works is the thing wired into Claude Code, because hooks and install are worth nothing until there is something to recall. Injection follows, then the feedback loop that measures whether injection helped — deliberately after injection exists, since it labels injection's own decisions. Observations and their egress boundary come next as the one opt-in, one network-touching feature, and lifecycle management lands last because retention is meaningless until there is data worth aging out. Each phase leaves something exercisable: a store you can verify, an ingest you can run, a search you can type, hooks that fire, a brief that appears, stats that prove precision.

## Phases

- [x] **Phase 1: Recall injection framing** - injected recall marks archived tool output as quoted history, never as the user's own turns
- [x] **Phase 2: Redaction** - redaction catches keyed JSON secrets and bare `sk-` tokens, never panics, and its test proves each rule
- [ ] **Phase 3: Owner-only files** - install's `settings.json` temporaries and the destination of `data move` are owner-only from the start
- [ ] **Phase 4: Session keys** - a moved or symlinked `.claude` no longer archives every session twice, and the doubled archive is repaired
- [ ] **Phase 5: Terminal output escaping** - `search`, `show` and `observations` neutralize control characters in archived text

## Phase Details

### Phase 1: Recall injection framing
**Goal:** The `UserPromptSubmit` hook and the `SessionStart` brief stop handing archived tool output to the model as the user's own history (#60). Today `search::run` applies no record-type filter on the inject path, `assemble()` pastes excerpts of file contents, fetched pages and command output under "Terminus recall - past turns from this project" (`inject/prompt.rs:292`, `:301`), and `hook.rs:341` emits that as `additionalContext`. So text an attacker planted in a file Claude once read comes back in a later session as the user's own past turns, even after the file is fixed. Whether tool-result turns are left out of injection or labelled, and the shape of the delimiter, are settled at context.
**Depends on:** Nothing (first phase of this cycle)
**Requirements:** INJ-09
**Success Criteria:**
1. An injected excerpt from a tool_result or `toolUseResult` turn is either absent or carries its record type and tool name.
2. Injected recall sits inside a delimiter that no excerpt text can close, under a header saying it is quoted history, not instructions, and the `SessionStart` brief quotations follow the same rule.
3. A regression test plants instructions in an archived Read result, names the file in a later prompt, and fails before the fix and passes after.
4. Issue #60 is closed by the change.

### Phase 2: Redaction
**Goal:** With `[privacy] redact_recall` on, redaction keeps the promise `SECURITY.md` makes and stops crashing. A secret under a secret-named JSON key in a tool input, tool result or attachment reaches `recall_search` excerpts, `recall_context` text and injected text whole, because `text::project` drops the keys before the name-based rules run (`recall/excerpt.rs:194`, #62). Bare `sk-` tokens match no shape rule (#5). `redact_assignments` slices inside a multi-byte character and panics the MCP server, the hooks and the read commands on text as ordinary as `café=1` (`observe/egress.rs:615-619`, #61). The egress test cannot tell which rule produced a marker (#4).
**Depends on:** Nothing
**Requirements:** PRIV-06, PRIV-07, PRIV-08, PRIV-09
**Success Criteria:**
1. A keyed secret in a `tool_use` input, a `toolUseResult` and an attachment is redacted in a `recall_search` excerpt, `recall_context` text and injected text, matching `recall_get`.
2. A bare `sk-` token in prose is redacted.
3. `café=1`, `Größe=10` and a curly-quoted `token=abc` pass through recall, the hooks, `terminus search` and `terminus show` without a panic.
4. The egress test checks each planted shape's own marker in the turn that holds it.
5. Each fix has a regression test that fails before it and passes after, and issues #62, #5, #61 and #4 are closed by the change.

### Phase 3: Owner-only files
**Goal:** Two places write sensitive files without owner-only permissions from the start. Install and uninstall write the whole of `settings.json`, which can hold an `env` block of credentials, into a temporary opened at the umask default and narrow it to 0600 only after the bytes are on disk, in both the rewrite and the backup (`install/mod.rs:57-67`, `install/json_file.rs:687-697`, `:711-719`, #64). `terminus data move` accepts an existing destination directory, or a symlink to one, without checking who owns it, and counts a directory it cannot list as empty (`cmd/data.rs:210-212`, `:306`, #59). Whether `Store::open` should also refuse a store another uid owns, which reverses its deliberate skip of a failed chmod (`store/open.rs:427-430`), is settled at context.
**Depends on:** Nothing
**Requirements:** INST-15, STOR-12
**Success Criteria:**
1. Every temporary install and uninstall create is 0600 from the moment it is opened, and a `settings.json` Terminus creates is 0600.
2. `terminus data move` refuses a destination that is a symlink, is owned by another uid, has group or world bits, or cannot be listed, and checks the destination again after the confirmation.
3. Each fix has a regression test that fails before it and passes after, and issues #64 and #59 are closed by the change.

### Phase 4: Session keys
**Goal:** A session's key is its transcript's absolute path, so when `~/.claude` moved to `/data/.claude` behind a symlink, discovery canonicalized the root, every transcript came back with a new path, and backfill archived all of them a second time (#57). Search and context results show twice, and `status` counts about double. The key has to survive a move, and the archive that already holds both keys has to be repaired without losing the one session that exists only under its old key. What the key becomes, and how the existing duplicates are re-keyed or dropped, are settled at context.
**Depends on:** Nothing
**Requirements:** ING-13
**Success Criteria:**
1. Moving the transcript directory behind a symlink and rerunning ingest archives no session a second time.
2. On a store that already holds sessions under both an old and a new path, the repair leaves one session per transcript, keeps every session that exists only in the archive, and `terminus verify` passes afterwards.
3. `recall_search` returns each turn once, and `terminus status` counts each session once.
4. A regression test fails before the fix and passes after, and issue #57 is closed by the change.

### Phase 5: Terminal output escaping
**Goal:** Human-mode output writes archived text to the terminal as it is, control characters included: `terminus search` excerpts (`cmd/search.rs:147`), the neighboring turns `terminus show --before/--after` prints (`cmd/show.rs:148`), and the facts `terminus observations` prints (`cmd/observations.rs:276`). An escape sequence in a file or command output a past session read can overwrite the clipboard or hide and forge lines (#65, #66, #63). One shared helper covers all three. `--json` output is already escaped and stays as it is.
**Depends on:** Nothing
**Requirements:** RCL-12
**Success Criteria:**
1. `search`, `show` and `observations` in human mode print C0 controls other than tab and newline, DEL, and C1 controls in archived text as visible escapes, through one shared helper.
2. `--json` output from all three is byte-identical to before.
3. A regression test plants ESC and BEL in an archived tool result and fails before the fix and passes after, and issues #63, #65 and #66 are closed by the change.
