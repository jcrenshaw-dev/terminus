PLAN CHECKPOINT: human-verify
Plan: .planning/phases/14/PLAN.md
Tasks: 2 of 4
| Task | Commit | Note |
|---|---|---|
| 1 - Put the result envelope and the list cache fields on every result | 17d0eda | `rpc::result` adds `resultType` and `_meta` serverInfo to every result; `tools/list` carries `ttlMs` 0 and `cacheScope` private; three ping assertions now pin the two-member result. Verify printf conversation matched; lib cmd::mcp 9 passed, test mcp 27 passed; clippy and check clean |
| 2 - Serve the 2026-07-28 era: `server/discover` and the `-32022` refusal | a3f6ec2 | `answer` refuses a non-`initialize` request whose `_meta` names a version other than 2026-07-28 with `-32022` and `data` before `route` runs the method; `server/discover` answers the D-04 object; `rpc::code::UNSUPPORTED_PROTOCOL_VERSION` and `rpc::failure_with_data` added. `PROTOCOL_VERSIONS` doc comment rewritten; its `2025-03-26` paragraph opened "for the same reason", which pointed at the removed paragraph, so that reason is now stated in it. Verify six-line conversation matched (lines 1 and 2 equal the D-04 object); `left out until` grep empty; lib cmd::mcp 9 passed, test mcp 27 passed; clippy and check clean. AC6 human-verify not run: needs a live Claude Code 2.1.302 session |
Deviations: none
Open items: none
