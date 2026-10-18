---
phase: 14
status: complete
completed: 2026-10-17
---

# Phase 14: MCP 2 - Summary

`terminus mcp` speaks protocol 2026-07-28: `server/discover` is answered, every result carries `resultType` and the server's identity under `_meta`, `tools/list` carries `ttlMs` and `cacheScope`, a request naming any other stateless revision is refused with `-32022`, and Claude Code 2.1.302 connected on 2026-07-28 with no fallback to `initialize`.

## What shipped

- The result envelope on every result, `resultType` `"complete"` and `_meta["io.modelcontextprotocol/serverInfo"]`, added in one place - `rpc::result` in `crates/terminus/src/cmd/mcp/rpc.rs`
- `tools/list` carrying `ttlMs` 0 and `cacheScope` `"private"` beside the three tools - `answer` in `crates/terminus/src/cmd/mcp/mod.rs`
- The era check on `params._meta`, the `-32022` refusal with `data.supported` and `data.requested`, and `server/discover` answering the D-04 object - `answer` in `mod.rs`, `rpc::code::UNSUPPORTED_PROTOCOL_VERSION` and `rpc::failure_with_data` in `rpc.rs`
- The `PROTOCOL_VERSIONS` doc comment saying why 2026-07-28 is not in the list - `mod.rs`
- Tests pinning AC1 through AC5, including a stateless conversation with no `initialize` against the spawned binary - the `tests` modules of `mod.rs` and `rpc.rs`, `crates/terminus/tests/mcp.rs`
- The design doc's two openings, era section, `server/discover` section and both figures, and the changelog entry - `docs/design/0006-recall-through-mcp.md`, `CHANGELOG.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 17d0eda | `rpc::result` adds `resultType` and the serverInfo `_meta` to every result; `tools/list` carries `ttlMs` and `cacheScope`; the three `ping` assertions pin the two-member result |
| 1 | 2 | a3f6ec2 | `answer` refuses a non-`initialize` request whose `_meta` names another revision with `-32022` and `data` before the method runs; `server/discover` answers; `PROTOCOL_VERSIONS` comment rewritten |
| 1 | 3 | d186b22 | Unit tests for AC1 through AC4, the `data` error envelope test in `rpc.rs`, and the AC5 integration conversation with no `initialize` |
| 1 | 4 | 6afec92 | Design doc 0006 documents the era, `server/discover`, `-32022` and the envelope, with both figures updated; changelog `[Unreleased]` entry naming 2026-07-28 and #45 |

## Deviations

None - plans executed as written.

## Open items

- Roadmap success criterion 3 says `initialize` offers 2026-07-28 again. D-08 keeps it out of `PROTOCOL_VERSIONS` because that revision has no `initialize`, so the criterion is corrected with `/cad-phase edit 14`, not by code.
- `Fixes #45` goes in the PR body at `/cad-land`; criterion 5 closes there, and CI runs on the push.
- `cargo fmt --check -p terminus` reports pre-existing diffs in `crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `crates/terminus/tests/release_workflows.rs` and `crates/terminus/tests/sign_release.rs`, all outside this plan's files and untouched by it.
- The two mermaid figures in `docs/design/0006-recall-through-mcp.md` were checked by reading only, because no mermaid renderer is installed here. Their GitHub rendering is for John's review.

## Goal check

The four commits deliver the goal. 17d0eda puts `resultType` and the serverInfo `_meta` on every result through `rpc::result` (`crates/terminus/src/cmd/mcp/rpc.rs:164`) and `ttlMs` 0 and `cacheScope` `"private"` on `tools/list`. a3f6ec2 adds `server/discover` answering `supportedVersions` `["2026-07-28"]` and refuses any other stateless revision with `-32022` carrying `data.supported` and `data.requested` before the method runs. d186b22 pins AC1 through AC5: the executor report counts 15 unit tests in `cmd::mcp` and 28 in `tests/mcp.rs`, and the workspace suite `cargo test --workspace --all-features --locked` ran once after the last commit, 909 passed, 0 failed. 6afec92 documents the era in the design doc and the changelog. AC6 was run live: `claude -p --debug mcp --strict-mcp-config` on Claude Code 2.1.302 against the built binary left `~/.claude/debug/51ee0b2d-9fe8-4437-a4a6-f330a4201178.txt` showing `"protocolEra":"modern","negotiatedProtocolVersion":"2026-07-28"`, no `initialize` sent, and `recall_search`, `recall_context` and `recall_get` each completed. The blocking risk review on the committed range matched `api_contract` and `untrusted_input`; the OpenAI reviewer raised one high finding, that the additive members on legacy `initialize` results could break a strict older client, which was refuted against D-02 and the spec's open `Result` shape and declined by John (`.planning/phases/14/ADJUDICATION-risk_surface-plan-1.json`). What the phase does not close is listed above: roadmap criterion 3 is worded against D-08, and criterion 5 closes at the PR.
