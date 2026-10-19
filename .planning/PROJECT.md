# Terminus

## What This Is

Persistent, cross-session memory for Claude Code, built on the premise that the session transcript is the record. Claude Code already writes every prompt, tool call, tool result and assistant turn to `~/.claude/projects/**/*.jsonl`; Terminus tails those files, stores each session unmodified and permanently, indexes it at turn granularity, and gives the model precise recall over its own history through three MCP tools and hook-driven context injection. A single static Rust binary, local-only, for Claude Code users — John first. Named Verbatim until 2026-10-05.

## Core Value

The verbatim transcript is the record: every session archived losslessly and permanently, everything else derived, rebuildable, and measurable against it.

## Requirements

### Validated

(None yet — ship to validate)

### Active

**hardin, opened 2026-10-18.**

The fixes the 2026-10-18 sentinel audit says must land before the first release, all filed as issues. Injected recall marks archived tool output as quoted history instead of handing it to the model as the user's own turns (#60). Redaction catches a secret stored under a secret-named JSON key and a bare `sk-` token, never panics on a multi-byte character, and its test proves each rule's own marker (#62, #5, #61, #4). Install's `settings.json` temporaries and the destination of `terminus data move` are owner-only from the start (#64, #59). A session keeps its key when its transcript directory moves behind a symlink, and the archive that already holds both keys is repaired (#57). Human-mode output neutralizes control characters in archived text (#63, #65, #66).

The version is named at the release. The fixes and features phases 12 to 14 left under `## [Unreleased]` in `CHANGELOG.md` ship with it.

**Shipped: v0.0.1, closed 2026-10-17.**

The install and release cycle, with four phases of docs, fixes and CI on top of it. The first ten phases took the model calls, the API keys, the network use, the npm package and every Windows path out, gave `terminus install` its defaults and its receipt, wrote the one-line signed install and the release machinery, stored each session as block rows, wrote the README and user docs, and published 0.1.0. The last four wrote the design docs, closed twelve small issues, put the test suite in CI with the archive and decision fixes under it, and taught the MCP server protocol 2026-07-28.

| | |
|---|---|
| Phases | 14 (No Model Calls; Linux and macOS Only, No npm; Install Asks Nothing; One-Command Installer; Install from GitHub Releases; Signed Install; Blocks as rows; Release Builds; README and User Docs; Publish 0.1.0; How Terminus Works; Quick Issue Fixes; CI and Data Integrity; MCP 2) |
| Requirements | 16, all Complete: PRIV-05, INST-09..14, STOR-08..10, REL-01..02, DOC-01..04. Phases 12, 13 and 14 delivered against GitHub issues rather than requirement rows |
| Commits | 310 since the cycle opened on 2026-10-12, excluding merges, on `cadence/v0.0.1` (phases 1 to 9), the phase 10 docs branches, `hardin` (12), `mallow` (13) and `encyclopedists` (11, 13 and 14) |
| Audit | PASS, 16/16 traced requirement -> phase -> plan -> verified, 0 broken, 0 deferred; 94/94 acceptance criteria covered by a UAT item, 0 breaks |
| Manifest | 0.1.1 (`Cargo.toml` workspace), not bumped at close. v0.1.0 carried phases 1 to 9 and v0.1.1 the aarch64 install fix, both published mid-cycle; the work of phases 11 to 14 stands under `## [Unreleased]` |

Where it lives now: the requirement rows under `## Shipped` in REQUIREMENTS.md, labelled v0.0.1; the per-phase deviations, UAT results and context decisions in `.planning/ARCHIVE.md` (282 rows added); the full phase directories, plans, reviews and adjudications under `.planning/_archive-v0.0.1/`, since no tag names this close.

Carried out of the close: `.planning/risk-carry/` holds 50 `risk_surface` reviews and adjudications from ten of the pruned phases. It is transient, never committed, and the `/cad-land` that confirms the merge deletes it.

Still outstanding: the merge of `encyclopedists` to `main` is `/cad-land`'s, and its pull request body carries `Fixes #45`. No release tag is cut for this close.

**Shipped: v0.1.1, closed 2026-09-04.**

A privacy and correctness cycle. One new requirement, three phases of defect work against scope v0.1.0 had already shipped, and the finding that the egress filter was running two of its five rules on the wire.

| | |
|---|---|
| Phases | 4 (Prompt-True Resume Brief, Egress Filter Sees What Is Sent, Owner-Only On Disk, Redacted Recall) |
| Requirements | 1 new: INJ-07, Complete. Phases 2, 3 and 4 delivered against already-shipped v0.1.0 rows (PRIV-01..04, RCL-09, OBS-03) rather than opening new ones |
| Commits | 77 on `cadence/v0.1.1` |
| Audit | PASS, 1/1 traced requirement -> phase -> plan -> verified, 0 broken, 0 deferred; 27/27 acceptance criteria covered by a UAT item, 0 breaks |
| Manifest | 0.1.1 (`Cargo.toml` workspace, `npm/verbatim/package.json`, five npm platform packages), bumped at close |

Where it lives now: INJ-07's row under `## Shipped` in REQUIREMENTS.md; the per-phase narrative, deviations, UAT results and context decisions in `.planning/ARCHIVE.md` (86 rows added, 378 total); the release notes in `CHANGELOG.md`, which this cycle started. The phase directories were deleted rather than archived, since the release tag is their archive and git history holds them.

Carried out of the close: `.planning/risk-carry/` holds 22 `risk_surface` reviews and adjudications from the four pruned phases. It is transient, never committed, and the `/cad-land` that confirms the merge deletes it.

One known issue ships with it: `recall_get` runs its body through a lossy UTF-8 round trip when `redact_recall` is on, filed as [#1](https://github.com/crenshawdev/verbatim/issues/1) and recorded in `.planning/FILED.md`.

Still outstanding: the merge to base and the release tag. Both are `/cad-land`'s, and the tag is cut there on the pulled base after the merge confirms, never at close.

**Shipped: v0.1.0 — closed 2026-08-23.**

The full feature set landed in one cycle. Every capability bullet that stood here is delivered and verified:

| | |
|---|---|
| Phases | 8 (Archive Core, Ingest At Scale, Recall, Hooks And Install, Context Injection, Feedback Loop, Observations, Retention And Lifecycle) |
| Requirements | 64, all Complete — STOR-01..07, ING-01..11, RCL-01..11, INST-01..08, INJ-01..06, FEED-01..04, OBS-01..08, PRIV-01..04, RET-01..05 |
| Commits | 237 on `phase-2-ingest-at-scale` |
| Audit | PASS — 64/64 traced requirement -> phase -> plan -> verified, 0 broken, 0 deferred; 57/57 acceptance criteria covered by a UAT item, 0 breaks |
| Manifest | 0.1.0 (`Cargo.toml` workspace, `npm/verbatim/package.json`) — unbumped, since nothing had been published yet |

Where it lives now: the requirement rows under `## Shipped` in REQUIREMENTS.md; the per-phase narrative, deviations, UAT results and context decisions in `.planning/ARCHIVE.md`; the full phase directories, plans, reviews and adjudications under `.planning/_archive-v0.1.0/`.

Merged to `main` and tagged `v0.1.0`.

`/cad-phase add` opens the next cycle.

### Out of Scope

- Multi-machine sync, teams, auth, cloud — local-first product; stated so it doesn't creep in
- Harnesses other than Claude Code in 0.x (Codex, Cursor, OpenCode) — each is a parser to add later, not an integration to design for now
- Web viewer or HTTP server — no daemon, no port, no long-lived process is a design invariant
- Telemetry of any kind — privacy contract
- Daemon / long-lived process — the hook spawn is the scheduler; eliminates the orphaned-process failure class (claude-mem: 157 GB orphans, OOM)
- Encryption at rest — the source transcripts already sit in plaintext in `~/.claude/projects`; at-rest scrubbing buys little
- Ingest-time redaction — makes the store lossy, defeats "verbatim", destroys what it strips; redaction is egress-only
- Custom FTS5 tokenizer — normalize in Rust at ingest instead; every expansion rule stays a unit-testable function
- `repair` command or external supervisor — recovery lives in the binary at the top of every ingest run
- Second index store (Tantivy etc.) — second commit, divergence handling; FTS5 is adequate at this corpus size

## Context

**Greenfield.** The repo contains only `DESIGN-BRIEF.md` (the settled design this document derives from) on branch `restart`. No source code yet.

**The incumbent.** claude-mem (90k stars, 224 open issues) runs a second Claude instance that compresses every tool call into an LLM-authored "observation" and throws the raw turn away. Documented consequences: users burn token budget on the memory system itself ($90 in three hours reported); the LLM's guess is the only copy, so retrieval can never be audited; context injection is `ORDER BY created_at DESC LIMIT n`; 45 open Windows issues, 36 Chroma issues, orphaned processes reaching 157 GB. Terminus inverts all of it: keep the truth, derive everything else, make retrieval measurable. Claude-mem's issue tracker doubles as a free test plan — one regression test per failure class designed out.

**Transcript facts, verified against 1,896 real files:**

- Compaction is append-only: same file, same session id, two records appended (`system`/`compact_boundary`, then `user` with `isCompactSummary`). Byte-offset tailing is safe.
- `compact_boundary` carries `preTokens`, `postTokens`, `cumulativeDroppedTokens`, and `preservedMessages.uuids` / `allUuids` — exactly which turns fell out of the model's context.
- Subagents live in a nested sidecar directory (`<project>/<sessionId>/subagents/agent-*.jsonl` plus `.meta.json`, `isSidechain: true`): 214 dirs, 1,135 files, 84,825 records — invisible to a top-level glob; discovery must recurse with `dot: true`.
- 1 file = 1 session = filename; 0 unparseable lines in 5,915 sampled.
- 1.2% of sessions continue across files (resume/fork), linked by `parentUuid`.
- Every record carries `cwd` and `gitBranch` directly.

**Measured byte distribution:** 41.0% tool results, 35.8% assistant turns, 16.4% attachments, 2.6% user prompts. Measured corpus: 895 MB / 1,896 sessions / 25 days ≈ 13 GB/year uncompressed for very heavy use, ~2 GB/year compressed.

**Open items (dispositions settled in the brief):**

- Auto-tuner for injection thresholds — hypothesis, not a plan. Ship logging + replay; gate the tuner behind evidence it converges.
- Import of `/data/verbatim-legacy` (801 MB) — deferred.
- crates.io name `terminus` is taken — skip crates.io; Terminus is delivered by `install.sh` from the GitHub releases.

## Constraints

- **Tech stack**: Rust, single static binary — no Node, Bun, Python, or native shared libraries at runtime; `rusqlite` with `bundled` feature; zstd; `rayon` for backfill fan-out; no async runtime (cold start is the metric; the hook path must not pay for a runtime it doesn't use)
- **Performance**: SessionStart synchronous work in single-digit milliseconds; UserPromptSubmit under a hard deadline, fail open; hook cold start under an asserted budget in CI — startup time is the product
- **Compatibility**: Linux and macOS only. Windows was dropped, as it is across all of the owner's products, and a build for any other system fails at compile time. Path comparison is case-folded on macOS
- **Privacy**: no telemetry; the Terminus binary opens no network connection, and `install.sh`'s `curl` download is the one network use
- **Licensing**: MIT, public OSS — an explicit override of the private-by-default rule, scoped to this project; all dependencies permissive
- **Distribution**: there is no npm channel, because the npm package was removed; `install.sh` is the one-command installer. `origin` is github.com/crenshawdev/terminus, where releases are published
- **Testing**: real SQLite temp databases, never mocks; private real-corpus fixture reached via env var, small synthetic set in-repo for CI

## Key Decisions

Settled in DESIGN-BRIEF.md; outcomes pending until shipped and validated.

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust, single static binary, no async runtime | Cold start is the product; the hook path must not pay for a runtime it doesn't use | - Pending |
| SQLite via rusqlite `bundled` as the one store | Single-transaction consistency, no system dependency; redb (exclusive lock), Tantivy (second store), Turso (beta), Postgres (server), DuckDB (single-process) all rejected | - Pending |
| Session blobs are truth; everything else derived and rebuildable; archive table never migrates | Store format bump rebuilds derived tables only — prevents claude-mem's 49-migration situation | - Pending |
| Block-framed zstd blobs (64 KB), compression on by default | One turn read decompresses one block (~60 µs); ~13 GB/yr raw becomes ~2 GB/yr | - Pending |
| Hooks call the absolute binary path, no shell; spawn fully detached | Retires the failures claude-mem hit from PATH probing, version resolution and inherited handles | - Pending |
| No daemon; the hook spawn is the scheduler; OS try-locks for exclusivity | Locks die with the process — no PID files, no stale-lock wedging, no orphans | - Pending |
| Project identity = canonical git toplevel from the record's `cwd`, never encoded dir name or `basename()` | claude-mem's four identity bugs all trace to `basename()`; worktrees map to the parent repo | - Pending |
| Index at turn granularity, store at session granularity | Precise recall over cheap, consistent session-level storage | - Pending |
| Exactly three MCP tools, `readOnlyHint`, one-line descriptions | Every tool description sits in every session's context forever | - Pending |
| Tokenizer: normalize in Rust at ingest, plain `unicode61` in FTS5 | Both query shapes hit; every expansion rule is unit-testable; index grows only ~1.3–1.6× | - Pending |
| Entities never rejected at index time; IDF-weighted at query time; capped per turn | "Too common" changes as the corpus grows and index-time rejection is irreversible | - Pending |
| Injection is precision-first: structural thresholds, 0–3 turns, silent most of the time | Three near-misses are worse than silence; BM25 scores aren't comparable across queries | - Pending |
| Log every injection decision including non-fires; replay retrieval changes offline | Non-fires are where miss data lives; relevance becomes regression-testable, not tuned by feel | - Pending |
| Observations opt-in; parser does mechanical, LLM only judgment; strict JSON; claims anchored to `turn_id` | claude-mem asks the LLM for what a parser can do (0 of 5,059 rows populated) and its XML drifts into silent loss | - Pending |
| Redact at egress, never at ingest; keyed on destination, not operation | Ingest-time redaction makes the store lossy; local providers send nothing over the wire at all | - Pending |
| Retention and exclusion off by default; excluded means never read, on ingest and read paths both | Keep everything; claude-mem honors exclusion on write and ignores it on read | - Pending |
| Canonical stable binary path; hooks never rewritten | One binary, no version skew, no version-check hook. The npm package that used to deliver the binary was removed | - Pending |
| No Claude Code plugin in 0.x | Versioned plugin directories are the root cause of claude-mem's path-resolution script; revisit at 1.0 | - Pending |
| MIT, public OSS (explicit override of private-by-default) | Nobody installs a closed binary that reads every keystroke; AGPL is a documented negative signal here | - Pending |
| Auto-tuner is a hypothesis, not a plan | Per-user volume may be too low; "referenced downstream" is a weak proxy; gate behind evidence it converges | - Pending |
| Recommend disabling auto-compact; never change it ourselves | Compaction burns tokens summarizing context already stored losslessly; fresh session + targeted recall beats self-summarization | - Pending |
| Shared credentials at `~/.config/jcrenshaw/credentials.toml`, namespaced by provider | Cross-product infrastructure; every jcrenshawdev product reads the same keys | - Pending |
| Renamed Verbatim to Terminus (2026-10-05), as a hard cut with no aliases or fallbacks | Named alongside Baley as part of a suite of loosely coupled tools. Nothing was ever published under the old name, so there were no users to migrate. "Verbatim" stays the archive's property, not the product's name | Shipped |
| Linux and macOS only, with Windows dropped across all of the owner's products | The owner is removing Windows from every product, so it is removed outright rather than deferred. A build for any other system fails at compile time | - Pending |

---
*Last updated: 2026-10-17 at the v0.0.1 close, opening v0.2.0*
