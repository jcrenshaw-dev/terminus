# Terminus

## What This Is

Terminus keeps every Claude Code session and gives it back to Claude. It archives each session and every subagent's transcript losslessly in one local SQLite file, starts each new session with a brief of where the last one left off, hands Claude earlier turns when a prompt names the same paths or identifiers, and gives Claude and the user search over the whole history. It is one local binary for Claude Code users on Linux and macOS, it calls no model, runs no daemon and opens no network connection.

## Core Value

The transcript is the record: every session archived losslessly and verifiably, and everything else derived from it and rebuildable.

## Requirements

### Validated

- ✓ Archives every Claude Code session and every subagent's transcript, byte for byte under the default capture mode, in one SQLite file readable only by the user - 1.0.0
- ✓ Stores each session as numbered zstd blocks with a BLAKE3 hash per block and a hash over the block hashes per session; `terminus verify` checks every block - 1.0.0
- ✓ Starts each session with a brief of where the last session in the project left off: its date, its branch and the last thing said each way, capped at 6,000 characters - 1.0.0
- ✓ Hands Claude up to three earlier turns when a prompt names the same paths or identifiers and they clear the match threshold, capped at 4,000 characters - 1.0.0
- ✓ After a compaction, offers the next prompt only the turns the compaction dropped - 1.0.0
- ✓ Gives Claude three MCP tools, `recall_search`, `recall_context` and `recall_get`, from cheap to complete - 1.0.0
- ✓ Searches from the command line with `terminus search`, `show`, `sessions` and `observations`, scoped to the current project, another with `--project`, or all - 1.0.0
- ✓ Records what happened in each closed session (files, tools, commands, errors, commits) from the transcript's own records, with no model - 1.0.0
- ✓ Logs every injection decision, non-fires included, and reports on them with `stats` and `replay` - 1.0.0
- ✓ Filters text shaped like a secret out of what is handed to Claude with `[privacy] redact_recall` - 1.0.0
- ✓ Checks and rebuilds itself with `doctor`, `verify`, `reindex` and rolling snapshots (by default three, at most one a day) - 1.0.0
- ✓ Installs with one line that checks the release's SSHSIG signature before it installs anything, updates by running the same line again, and uninstalls with `terminus uninstall` - 1.0.0
- ✓ Runs on Linux and macOS, x86_64 and aarch64; a build for any other system fails at compile time - 1.0.0

### Active

No cycle open.

**Shipped: mule, closed 2026-10-27.**

The bugs open at 1.0.0, plus the cleanup the 2026-10-22 simplify review found in the same code. Upgrade code for store layouts 1.0.0 can't create is gone (#105), and so are the planning tags in comments, with a test that keeps them out (#106). The macOS Intel hook tests no longer fail the merge gate, and a failed timing test says whether the hook blocked or the runner stalled (#75, #104). Search returns the other words' hits past an unindexable word, checks dates as dates, and binds every list as one parameter (#23, #24, #108). One pass driver replaces three, and recovery notes no longer read as errors (#31, #96). Continuation links, retention's delete and the observations query are fixed (#11, #43, #70). Every file Terminus writes is owner-only through one helper set, and signing runs no installer code (#73, #99). Every command escapes archived text and answers a `--json` failure with a document, and `uninstall --purge` and the prompt line say what they really do (#72, #107, #95, #94).

| | |
|---|---|
| Phases | 8 (Old store code; Planning tags; Hook tests on macOS Intel; Search; One ingest pass; Archive; Owner-only files and signing; Output) |
| Requirements | 18, all Complete: SHED-01, TAGS-01, GATE-01..02, SRCH-01..03, PASS-01..02, ARCV-01..03, OWN-01..02, OUT-01..04. Eighteen issues, closed by PR #124 when it merges |
| Commits | 173 on `mule`, 2026-10-22 to 2026-10-27, excluding merges and this record's own commit |
| Audit | PASS, 18/18 traced requirement -> phase -> plan -> verified, 0 broken, 0 deferred; 51/51 acceptance criteria covered by a UAT item, 0 breaks |
| Manifest | 1.0.0 (`Cargo.toml` workspace), not bumped at close. The version is named at the release; mule has no `CHANGELOG.md` entry yet |

Where it lives now: the requirement rows under `## Shipped` in REQUIREMENTS.md, labelled mule; the per-phase deviations, UAT results and context decisions in `.planning/ARCHIVE.md` (113 rows added); the full phase directories under `.planning/_archive-mule/`, since no tag names this close. `ARCHIVE.md` and `_archive-mule/` stay on the maintainer's machine and are not in git.

Carried out of the close: `.planning/risk-carry/` holds 36 `risk_surface` reviews and adjudications from phases 1 to 6 and 8. It is transient, never committed, and the `/cad-land` that confirms the merge deletes it.

Still outstanding: PR #124 is a draft. It is marked ready for its review, and the merge of `mule` to `main` is `/cad-land`'s. No release tag is cut for this close.

### Out of Scope

- Windows - it fails TRUST.md's privacy and security criteria; a build for it fails at compile time
- Coding agents other than Claude Code - Terminus reads only Claude Code's transcripts and wires only into Claude Code
- Encryption at rest - Claude Code's own transcripts sit unencrypted beside it; Terminus's files are owner-only instead
- Semantic search, embeddings and summaries - each needs a model, and Terminus calls none
- A daemon, an HTTP server or a web viewer - no long-lived process and no open port is a design invariant
- Telemetry of any kind - Terminus collects nothing about the user
- Multi-machine sync, teams or cloud storage - the store is local and the binary opens no network connection
- Ingest-time redaction - it would make the archive lossy; redaction happens only on the way out
- A conversion tool for stores older than the archive format - the format gate refuses them, and a converter comes only if one is needed
- A built-in updater - rerunning `install.sh` updates it, so the binary stays offline

## Context

**Code.** A Rust 2021 workspace with two crates: `terminus-core` (archive, ingest, index, recall, injection, observations, retention, feedback) and `terminus` (the CLI, the hook, the MCP server, install and uninstall). 88 source files, about 35,500 lines including inline tests, with integration tests under `crates/*/tests`, fuzz targets under `fuzz/`, and the sentinel audit kit as a submodule.

**Docs.** `README.md` for users, `docs/` for commands, settings, JSON shapes, install and release, and `docs/design/` 0001 to 0008 for how it works. Design docs describe the current design only.

**Release state.** v1.0.0 was published 2026-10-21. CI runs `cargo test --workspace --all-features` on every pull request push and on main, on Linux and macOS, x86_64 and aarch64, debug and release.

**Work list.** Open GitHub issues, each with a type and a `sev:` label. The 2026-10-22 simplify review filed #105 to #113 and added findings to thirteen existing issues.

**Known rough edges.** The macOS Intel hook tests fail the merge gate now and then (#75, #104). 1,958 comment lines cite planning tags from removed files (#106). Recovery and the store open carry upgrade code no 1.0.0 store needs (#105).

**Live store used for measurements.** 3,305 sessions and 691,376 turns, 3.0 GiB, archive format 2, derived schema 5.

## Constraints

- **Tech stack**: Rust, one static binary, no async runtime; `rusqlite` with `bundled` (FTS5 compiled in), zstd and BLAKE3. Every linked crate is cold-start cost on the hook path
- **Performance**: hooks start a detached pass and exit; per-prompt injection runs under a hard deadline and fails open; a hook exits 0 whatever goes wrong
- **Compatibility**: Linux and macOS, x86_64 and aarch64; path comparison is case-folded on macOS
- **Privacy**: no telemetry and no model calls; the binary opens no network connection, and a test fails if any source file names a socket type
- **Distribution**: `install.sh` fetches signed releases from GitHub; a local build is not checked against any signature
- **Process**: `main` takes changes only through pull requests with signed commits, once the tests and sentinel's checks pass
- **Testing**: real SQLite databases in temporary directories, never mocks

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust, one static binary, no async runtime, `lexopt` for arguments | Cold start is the product; the hook path pays for every linked crate (`Cargo.toml`) | Shipped in 1.0.0 |
| SQLite through `rusqlite` `bundled` as the one store, with FTS5 search | No system dependency, and the blob and its watermark commit in one transaction | Shipped in 1.0.0 |
| Each session stored as 64 KB zstd blocks with a BLAKE3 hash per block; everything else derived and rebuilt by `reindex` | Lossless and verifiable, and reading one turn decompresses one block | Shipped in 1.0.0 |
| An archive format gate that refuses older and newer stores, with no conversion | One store shape to reason about; derived tables are rebuilt instead of migrated | Shipped in 1.0.0 |
| No model calls and no embeddings; identifiers indexed by their parts | Nothing runs up the user's bill or usage limit in the background | Shipped in 1.0.0 |
| No daemon: each hook spawns a detached pass under an OS lock | A lock that dies with its process leaves no stale lock and no orphaned process | Shipped in 1.0.0 |
| Redaction at egress only | The archive stays lossless | Shipped in 1.0.0 |
| Linux and macOS only | The platform criteria in TRUST.md; Windows fails them on privacy and security | Shipped in 1.0.0 |
| Releases signed with SSHSIG (Ed25519) on the maintainer's machine, never in CI; the public key pinned in `install.sh` | The signing key never leaves the maintainer's machine | Shipped in 1.0.0 |
| Updates by rerunning `install.sh`, no built-in updater | The binary stays offline | Shipped in 1.0.0 |
| Each release bumps the middle number, a fix-only release the last, and the first only for a large change; no compatibility promise | The changelog says what each release changes (`CHANGELOG.md`) | Shipped in 1.0.0 |
| Work is planned from filed GitHub issues | One work list, kept in the open (5933b3b) | - pending |
| MIT, public | `LICENSE` | Shipped in 1.0.0 |

---
*Last updated: 2026-10-22 after adopting the repository into Cadence*
