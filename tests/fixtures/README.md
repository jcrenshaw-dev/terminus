# Synthetic transcript fixtures

Real Claude Code transcripts are private and never enter this repository.
These files are synthetic, deterministic, and shaped
to facts measured against a real corpus.
They are the CI corpus; the private real-corpus fixture is reached by env var.

The fixtures are byte-load-bearing. Tests assert on their exact bytes, and
`session-truncated.jsonl` is a byte prefix of `session-basic.jsonl`. Editing one
by hand without re-deriving the others will fail
`crates/terminus-core/tests/fixtures.rs`.

| File | Exists to exercise |
|---|---|
| `session-basic.jsonl` | All fifteen record types the parser knows; an out-of-order timestamp, so `turn_seq` comes from byte order; state records carrying one of `uuid` and `timestamp`, against the rule that a turn has both; a `file-history-snapshot` with no `sessionId`; a `parentUuid` chain |
| `session-large-record.jsonl` | A turn larger than one 64 KB block |
| `session-continuation.jsonl` | File-level lineage through a foreign `session_id` |
| `session-truncated.jsonl` | A watermark that must stop at the last `\n`, not at end-of-file |
| `session-compacted.jsonl` | A session ending in a `compact_boundary` record whose `compactMetadata` bytes are stored verbatim |
| `session-recall.jsonl` | Search expansion and entities: a `SearchManager` turn, an `src/worker/S.ts` turn, and the structured-versus-prose pair - a `Read` `tool_use` naming a file against an assistant turn naming that same file in prose |
| `session-edits.jsonl` | Prompt injection: an `Edit` `tool_use` storing an **absolute** path beneath the fixture root, the spelling the real corpus almost always uses, with symbols beside it so one turn carries two independent entities - and a prose turn naming the same file structurally invisibly |
| `session-envelope.jsonl` | A session whose **last** `user` record is a `<task-notification>` envelope and whose second-to-last is a `tool_result`, so the resume brief has to pass over both to quote the last prompt a person typed |
| `session-secrets.jsonl` | One turn for each of the seven credential shapes the egress filter must catch on their way into the model's context - an `Authorization: Bearer` header, a JSON `"password"` pair, a space-separated `--token` flag, a mid-line `Cookie` with prose after it, a bare JWT, a GitHub token and a connection URL carrying userinfo - and two more holding the header and the `Cookie` inside a quoted curl `-H` argument |
| `session-redaction.jsonl` | One turn per redaction shape `session-secrets.jsonl` does not hold - a PEM private key, a bare `sk-` key in prose, a secret-named key in a `tool_use` input, in a `toolUseResult` and in an attachment - and one `tool_use` turn holding `café=1`, `Größe=10` and `“token=abc”` |
| `session-errors-a.jsonl` | Error entities: `tool_result` blocks with `is_error` and a `toolUseResult` carrying `stdout`/`stderr`/`interrupted`, holding the a-half of both stderr pairs |
| `session-errors-b.jsonl` | The b-half of the error pairs: one stderr differing from a's only in the parts normalization strips, one differing only in a bare integer it must not |
| `session-capture.jsonl` | Capture modes: a `toolUseResult` and an `attachment` each above and below the `lean` threshold, beside a plain prompt and a plain assistant turn |
| `subagents/agent-alpha.jsonl` | A sidecar reporting its *parent's* `sessionId`, which is why a session is keyed on its file |
| `subagents/agent-echo.jsonl` | Search ranking: a sidechain turn whose message text is byte-identical to a top-level turn, so "sorts below at equal score" has two genuinely equal scores to compare |
| `subagents/workflows/wf_demo/agent-deep.jsonl` | A sidecar two directories deeper than the usual one, which only an unbounded walk reaches |
| `subagents/workflows/wf_demo/journal.jsonl` | A `.jsonl` in the tree that is not a transcript at all |
| `subagents/agent-alpha.meta.json` | The sidecar metadata file, stored as opaque bytes and never a session |

## `hooks/`: the payloads, not the transcripts

`hooks/session-start.json`, `hooks/user-prompt-submit.json`,
`hooks/session-end.json` and `hooks/post-compact.json` are a different kind of
fixture. They are not transcripts and they are not ingested: each is the single
line of JSON Claude Code writes to a hook's stdin before closing it, and
`crates/terminus/tests/hook.rs` feeds them to `terminus hook <event>` to hold
each hook to "exit 0, nothing on stdout, p99 under 10 ms".

Their fields are read off the 2.1.231 payload schemas rather than guessed. Every
payload carries the base object the bundle's builder returns - `session_id`,
`transcript_path`, `cwd`, `prompt_id`, `permission_mode`, `agent_type` - then
`hook_event_name`, then that event's own fields: `source`, `model` and
`session_title` for `SessionStart`, `prompt` and `session_title` for
`UserPromptSubmit`, `reason` for `SessionEnd`, `trigger` and `compact_summary`
for `PostCompact`. The enumerated values are the schema's own: `source`
`"resume"`, `reason` `"prompt_input_exit"`, `trigger` `"auto"`.

The hook reads five of those fields - `session_id`, `transcript_path`, `cwd`,
`prompt` and `source` - and nothing else. Every field is shaped correctly
anyway, so code that starts reading another one meets a fixture read off the
schema rather than one invented against the code that consumes it.

`hooks/session-start-compact.json` is the fifth, and the only one of the five
that is a *recording* rather than a reading of the schema. It was captured
live - a matcher-less `SessionStart` entry appending its stdin to a file, a
fresh session, then `/compact` - and it shows that **a compaction fires a
`SessionStart`, with `hook_event_name` `SessionStart` and `source`
`"compact"`**, after the compaction completes and under the same `session_id`
as the `"startup"` line that opened the session. That event is what triggers
the injection after a compaction, so the trigger needs no fallback that reads
the boundary row at `UserPromptSubmit`.

The capture is those bytes with the identifying fields replaced, because this
repository is public and a real transcript never enters it: `session_id`,
`transcript_path` and `cwd` carry the same synthetic values as the other four,
`prompt_id` is the same synthetic UUID they use, and `model` is theirs rather
than the build-specific name the live session reported. `hook_event_name` and
`source` are exactly as captured, and they are the two fields anything reads.

The recording differs from the schema reading above in fields nothing reads.
The live `"compact"` line carries `prompt_id` and the live `"startup"` line
does not, so `prompt_id` is not an unconditional base field; and neither live
line carries `permission_mode`, `agent_type` or `session_title` at all. Code
that starts reading one of those three should observe a live payload rather
than trust the other four fixtures.

Each is one line with a trailing newline, matching `stdin.write(payload + "\n")`.

## What each one pins

**`session-basic.jsonl`** carries all fifteen `type` values: the four turn types
(`user`, `assistant`, `attachment`, `system`), each with both `uuid` and
`timestamp`, and the eleven state types (`last-prompt`, `ai-title`, `mode`,
`permission-mode`, `file-history-snapshot`, `queue-operation`,
`bridge-session`, `file-history-delta`, `agent-name`, `agent-setting`,
`frame-link`), which produce no turn, FTS or entity row. A turn needs *both*
identity fields, so the state records split the cases deliberately: `ai-title`
carries a `uuid` and no `timestamp`, while `mode`, `permission-mode`,
`queue-operation` and `last-prompt` carry a `timestamp` and no `uuid`. A
classifier that checks either field alone passes a fixture with only one of
those shapes and fails this one.

Two adjacent `assistant` turns have decreasing `timestamp` values in file order
(records 9 and 10). That was measured in 31 of 62 real transcripts: `turn_seq`
comes from byte order, never from sorting on `timestamp`. The
`file-history-snapshot` record carries no `sessionId` at all, matching all 129
real records of that type.

The token `brillig` appears in exactly one turn across the whole fixture set,
for tests that need a match with a known, unique answer. It is deliberately
*outside* the byte prefix `session-truncated.jsonl` keeps, so ingesting both
fixtures into one store still yields exactly one hit.

**`session-large-record.jsonl`** holds one turn whose text field is 204,800
bytes of deterministic filler. That record is the **first line of the file**, so
its offset in the uncompressed session stream is 0 and it occupies exactly four
64 KB blocks. Do not reorder this file: at a non-zero offset the same record
straddles a fifth block and the block-count assertions change.

**`session-continuation.jsonl`** distinguishes the two lineage signals ingest
keeps separate. `sessionId` (camelCase) is this file's own session; `session_id`
(snake_case) names `session-basic.jsonl`'s session, and that is the file-level
signal `session_meta.continues_from` reads first. `parentUuid` is
message-level threading, and ingest falls back to it only when no record names
a foreign `session_id`: the transcript's first turn's `parentUuid`, resolved
through `turns` to the session holding that record.

**`session-truncated.jsonl`** is `session-basic.jsonl` cut inside record ten,
with no trailing newline: nine complete records plus a partial one. It is the
"transcript still being written" case. The resume offset is the byte just past
its last `\n` (2,977), strictly less than the file length. Appending the rest of
the cut line and the remaining records must produce a store identical to
ingesting the complete file in one pass.

**`session-compacted.jsonl`** is three ordinary turns followed by the record a
compaction leaves behind: `type: "system"`, `subtype: "compact_boundary"`, with
a `uuid`, a `timestamp`, a `logicalParentUuid` and a `compactMetadata` object.
It pins that `system` is already a turn type and the record carries both
identity fields, so the boundary is an ordinary turn plus a derived row, never a
new record class. Its keys sit in the same order as the one real boundary record
in the corpus, and `compactMetadata` deliberately does **not** sit last, so
nothing may extract it by position.

The metadata reproduces the measured numbers: `preTokens` 45,500, `postTokens`
7,436, `cumulativeDroppedTokens` 38,064, and a `preservedMessages` whose `uuids`
list is a *proper* subset of its `allUuids` list. That subset relation is the
fact the dropped-turn set rests on: the uuid lists describe the preserved
segment and cannot enumerate the ~38k dropped tokens, so the boundary record
does not say which turns fell out. Ingest therefore stores the bytes verbatim
and interprets none of them, and injection derives the dropped turns at query
time as the archived turns `preservedMessages.uuids` does not name.

The boundary is the **last** line of the file, so a test can append that one
record to an already-ingested transcript and watch a boundary row appear under a
session that had none. Adding a line after it breaks that.

**`subagents/agent-alpha.jsonl`** reports `session-basic.jsonl`'s `sessionId` on
every record with `isSidechain: true`. This is the session-id collision: 812
real sidecar files do exactly this, so a `sessions` table keyed on the record's
`sessionId` would overwrite the parent's blob with a small agent transcript.

**`subagents/workflows/wf_demo/agent-deep.jsonl`** is a sidecar at the deep
position. Real sidecars sit at two depths: 781 at
`<project>/<sessionId>/subagents/agent-*.jsonl` and 41 two levels further down
at `<project>/<sessionId>/subagents/workflows/wf_*/agent-*.jsonl`. A
depth-limited walk finds `agent-alpha.jsonl` and silently drops this one, and
without a fixture in the deep position no test on the synthetic corpus catches
that. Like `agent-alpha.jsonl` it reports the parent's `sessionId` with
`isSidechain: true`.

**`subagents/workflows/wf_demo/journal.jsonl`** is not a transcript. Its records
are `{agentId, key, result, type}` with none of `sessionId`, `uuid`, `timestamp`
or `cwd`. Four such files exist in the real tree, and recursive discovery with
`dot: true` reaches them, so the parser must tolerate them without erroring.

**`subagents/agent-alpha.meta.json`** is the meta file Claude Code writes
beside a sidecar transcript: same directory, same stem, a different extension.
816 real ones exist against 818 sidecars, so a missing one is normal and the
column simply stays null. It carries the five fields every real one carries -
`agentType`, `description`, `toolUseId`, `spawnDepth`, `model` - and its
`toolUseId` names the `tool_use` block in `session-basic.jsonl` that spawned the
agent, so the fixture set stays internally coherent.

Ingest stores its **bytes** and only checks whether they are already stored; no
code parses them. The format is undocumented and may drift, and keeping the
bytes lets `description` be extracted later without a reingest. It is not a
transcript, so it is not in `TRANSCRIPT_FIXTURES` and the JSONL assertions below
do not apply to it - discovery's filename filter excludes it on the extension,
which is what keeps it from ever becoming a session.

**`session-recall.jsonl`** is the search index fixture. It carries a turn whose
message text contains `SearchManager` (found by a search for `SearchManager` and
by a search for `manager`, which is the only expansion rule that changes recall),
a turn whose text contains `src/worker/S.ts` (found by a search for `worker`),
and the structured-versus-prose pair: a `Read` `tool_use` whose `file_path` is
`docs/RETRY.md`, and a *separate* assistant turn naming `docs/RETRY.md` in prose
and nothing else. The first emits a `path` entity and a `paths` row; the second
emits neither, which is what "entities come from structured tool records, never
from prose" has to mean. A `Grep` `tool_use` carries the `retryBudget` pattern
its `symbol` extraction reads. Its last turn's message text is byte-identical to
`subagents/agent-echo.jsonl`'s.

**`session-errors-a.jsonl`** and **`session-errors-b.jsonl`** are two distinct
sessions carrying the two stderr pairs error normalization is about. Their
`tool_result` blocks carry `is_error` and their records carry a top-level
`toolUseResult` with `stdout`, `stderr` and `interrupted` - the key set measured
on 2,920 real Bash results, and the only place an error signal exists, since the
transcript has no exit-code field anywhere.

The first pair (`is_error: true`) differs between the two files *only* in a
`:line:col` suffix, a `0x` address, an ISO-8601 timestamp and a UUID, so
normalization collapses it to one value. The second pair (`is_error: false`,
non-empty `stderr`, which is the error rule's other arm) differs *only* in a
bare integer that is not a line number, and must stay two values: stripping bare
integers was measured to merge genuinely different errors (46 recurring values
before, 39 after). `session-errors-a.jsonl` ends with a third result that is
`interrupted` with an empty `stderr` and no error flag - the control for "an
interruption is not an error".

**`subagents/agent-echo.jsonl`** is a second sidecar reporting
`session-recall.jsonl`'s `sessionId` with `isSidechain: true`, and it has no
`agent-*.meta.json` beside it, which is the normal case for 2 of every 818 real
sidecars. Its second turn's message text is byte-identical to the last turn of
`session-recall.jsonl`, so the two project to the same FTS body and score
identically under BM25. That is what makes "a sidechain turn ranks below a
top-level turn of equal score" a statement a test can hold to, rather than one
that passes because the scores happened to differ.

**`session-edits.jsonl`** is the prompt injection fixture, and what it holds
that nothing else does is an **absolute** path. Over 120 sampled real
transcripts, `file_path` held 1,029 absolute values against 2 relative ones, so
an archived `path` entity is almost always absolute while the user types a
relative one - and `Query::matches_entity` needs every token of the stored value
present in the query, so the two cannot meet without resolving the relative
spelling against the payload's `cwd`. A fixture storing a relative path would
let a test of that resolution pass without the resolution.

Its `Edit` call names `{{ROOT}}/project-alpha/crates/gizmo/lantern.rs` and
carries `lanternFlicker` and `lanternSteady` across its two edit sides, so one
turn emits four distinct entities - a tool, a path and two symbols - which is
what the injection threshold's "two or more independent entities
co-occurring" needs to have something to fire on. Its third turn names that same
absolute path in **prose** and emits no entity at all, so "matched structurally"
and "mentions the words" stay two different answers about one file, the way
`session-recall.jsonl` already does for the relative case.
The `Edit` turn opens with a short prose block naming `lanternFlicker`, ahead of
the `tool_use`, so the excerpt window reaches the asserted text whatever the
length of the temp directory's path. Moving it behind the path brings back #46.

**`session-secrets.jsonl`** is the egress fixture: twelve turns
that exist to be *sent*, not to be found. Each of the seven shapes the egress
rule set is asked to catch sits in a turn of its own, so a wire test can
assert on one sentinel at a time and attribute the catch to one rule. Two more
turns hold the header and the `Cookie` the way curl spells them, inside a
double-quoted `-H` argument: `-H "Authorization: Bearer sk-VBEGRESS-curlz-5e6"`
and `-H "Cookie: sid-VBEGRESS-qcrumb-1f9"`. The first two turns and the last
carry no credential.

Four of the seven - the bare JWT, the GitHub token, the connection URL's
userinfo and the space-separated `--token` value - sit with **no name-keyed text
beside them**. That is the load-bearing part: written `GITHUB_TOKEN=ghp_...`
instead, the assignment rule would catch it and the assertion would be about
nothing. The `Cookie` turn puts its value **mid-line** with ordinary prose after
it on the same line, which is what makes "the rest of the turn survived the
match" a thing a test can read.

Its sentinels are short, unrealistic and distinct from one another
(`ghp_VBEGRESSgh0zq`, `sk-VBEGRESS-authz-9f2`), following this repository's
existing convention: a realistic-length credential in a public repository risks
GitHub push protection, and a shared sentinel would let one rule's catch pass as
another's.

Like `session-envelope.jsonl` it is deliberately inert. Its project key is a
**fourth** one, `project-delta`, which no assertion anywhere scopes against; it
carries no `tool_use` and therefore emits no entity; and its vocabulary shares
none of the corpus's counted tokens - no `brillig`, no `SearchManager`, no
`cargo`, none of the paths the entity tests match on.

**`session-redaction.jsonl`** is the redaction fixture for the shapes
`session-secrets.jsonl` does not hold: six records in one session, each built so
that exactly one rule can catch what it holds. In order:

1. A typed `user` turn (`heron`) whose prose carries one PEM private key block,
   the shape `tests/egress.rs` already uses, with a plainly fake body line. Only
   the PEM rule can take it.
2. A typed `user` turn (`marigold`) carrying `sk-VBREDACT-plainly-fake-bare-0e5`
   in prose, with no `:`, `=`, `--` flag or quoted pair near it. Only the bare
   `sk-` rule can take it. It is the last typed prompt, so it is the one the
   resume brief quotes.
3. An `assistant` turn calling `Write` with an absolute `file_path`,
   `{{ROOT}}/project-epsilon/juniper/wren.toml`, beside `vault_secret`. The path
   makes the turn a prompt-hook target, which is how a test reaches the
   per-prompt injection with a keyed secret in it.
4. The `user` tool result answering it (`thistle`): a plain string
   `tool_result` and a top-level `toolUseResult` holding `relay_token` beside
   `type` and `status`. No `is_error` and no `stderr`, so it emits no error
   entity.
5. An `attachment` (`bramble`) holding `signing_key` beside `type` and
   `content`.
6. A second `Write` call (`saffron`) whose `content` holds `café=1`,
   `Größe=10` and `“token=abc”` as raw UTF-8, never as `\u` escapes, because
   that is how a real transcript stores them and what a byte search of a
   `recall_get` body has to find.

The three keyed values are the point of the file. The projection a search
excerpt is cut from drops every key, so `vault_secret`, `relay_token` and
`signing_key` never reach the rules that need a name beside a value. Each of
those values sits alone in its turn: no other key in the subtree contains a
fragment the name test matches, the key's own name appears nowhere in the turn's
text, and the value has no `sk-`, `eyJ` or GitHub prefix, no `://...@` and no
`name:`, `NAME=` or `--name` shape. A turn that held a second thing a rule could
catch could not say which rule fired.

The `sk-` sentinel runs longer than this repository's other sentinels because
the bare `sk-` rule needs 20 key characters after the prefix before it fires,
which is what keeps placeholders like `sk-notification` out of it. It stays
plainly fake the same way the others do: it spells out what it is and carries
the fixture's mark, `VBREDACT`, which every sentinel here shares and none of
`session-secrets.jsonl`'s `VBEGRESS` assertions touch.

The three multi-byte strings share one turn because that turn serves every
surface at once. It is the last assistant record, so the brief quotes it, and
its path makes it a prompt-hook target. Its `content` key sorts ahead of
`file_path`, and the projection emits input leaves in key order, so the three
strings sit at the head of the turn where the brief's window and a search
window around `saffron` both carry them whole.

Its project key is a **fifth** one, `project-epsilon`, which no assertion over
the fixture corpus scopes against. Its tool is `Write`, which no other fixture
calls, so each `tool_use` turn emits a tool and a path entity and nothing more.
Its vocabulary shares none of the corpus's counted tokens - no `brillig`, no
`SearchManager`, no `cargo`, no `retry`, `budget` or `gateway`, none of the
paths the entity tests match on. It is last in `TRANSCRIPT_FIXTURES`, because
session numbers follow ingest order and turn ids are built from them; anywhere
earlier would move the turn ids the recall golden pins.

**`session-envelope.jsonl`** is the brief fixture: five records
whose last `user` record is not one a person typed. In order - a `user` text
turn (`wire up the quince exporter and tell me what it prints`, the prompt the
brief must end up quoting), an `assistant` turn carrying a `Glob` `tool_use`, a
`user` turn whose `message.content` is a `tool_result` block and whose top-level
`toolUseResult` carries a `stdout` string and nothing else, an `assistant` text
turn, and last a `user` text turn opening with `<task-notification>`.

That covers two of the not-typed shapes at once, and it is the *last* record
being one of them that matters: a brief reading "the last `user` turn" quotes
the envelope, and a brief reading "the last typed `user` turn" quotes record
one. `session-recall.jsonl` cannot observe that difference - its only `user`
record is a plain text block - which is why `crates/terminus/tests/brief.rs`'s
byte-identity test seeds from this file instead.

Three things about it are deliberate. Its project key is a **third** one,
`project-gamma`: every project-scoped assertion elsewhere in the repository is
written against `project-alpha` or `project-beta`, so a new member of neither
moves none of them. Its tool is `Glob`, which appears in no other fixture and
whose `pattern` input emits no entity beyond the tool itself, so no entity count
anywhere moves either. And its vocabulary shares nothing with the corpus's
counted tokens - no `brillig`, no `SearchManager`, no `cargo`, none of the
paths the entity tests match on.

**`session-capture.jsonl`** is the capture-mode fixture: six records in one
session. In order - a typed `user` prompt, a `tool_result` whose top-level
`toolUseResult` serializes to 14,081 bytes, one whose `toolUseResult` is 77
bytes, an `attachment` of 12,112 bytes, one of 68 bytes, and a plain
`assistant` turn. The large two sit over the 8,192-byte `lean` threshold and
the small two under it, and the prompt and the assistant turn carry neither
key. So `lean` elides two subtrees and `minimal` elides all four, which is what
makes "the stored bytes strictly decrease from `full` to `lean` to `minimal`" a
measurement rather than a tautology.

It is not in `TRANSCRIPT_FIXTURES`, because the tests that ingest that set
count its sessions, turns and bytes. `tests/capture.rs` and `tests/pass.rs`
read it by name.

## The rooted `cwd`

The unrooted fixtures hardcode `"cwd": "/data/code/verbatim"`. The eight
rooted fixtures carry `{{ROOT}}` instead, and
`testkit::copy_rooted_fixture_into` rewrites it to a root the test owns,
creating the project directory it names. Five projects exist across them -
`project-alpha` for `session-recall.jsonl`, its sidecar and `session-edits.jsonl`,
`project-beta` for the two error sessions, `project-gamma` for
`session-envelope.jsonl` alone, `project-delta` for `session-secrets.jsonl`
alone and `project-epsilon` for `session-redaction.jsonl` alone - so a
project-scoped search has
something to be both true and false about, on any checkout and without a project
key that depends on where this repository happens to sit. `session-edits.jsonl`
needs the root for a second reason: the absolute path it stores has to be
beneath the same root its own `cwd` names, or no prompt resolved against that
`cwd` could reach it.

Every transcript fixture except `session-truncated.jsonl` ends with a trailing
`\n`, and no fixture contains a `\r` byte.
