---
phase: 2
status: complete
completed: 2026-10-18
---

# Phase 2: Redaction - Summary

With `[privacy] redact_recall` on, rule 5 no longer panics on a multi-byte character before an `=`, a bare `sk-` key with 20 or more key characters is replaced by its own marker, keyed string values in a `tool_use` input, `toolUseResult` and attachment are replaced on every excerpt surface the way `recall_get` replaces them, and the egress test checks every planted shape's marker in the turn that holds it.

## What shipped

- Rule 5 reads its name as ASCII bytes, walking back from the `=` over `is_flag_name_byte` - `redact_assignments` in `crates/terminus-core/src/observe/egress.rs` (#61)
- Rule 9, a bare `sk-` key with a 20-character floor and the marker `[redacted:an-sk-key]` - `redact_sk_keys`, `MIN_SK_KEY_BODY`, `REDACTED_SK_KEY` in `observe/egress.rs` (#5)
- The keyed-value step on the parsed record, run before `text::project` with the knob on - `Redaction::apply_record` and `redact_keyed_values` in `observe/egress.rs`, called from `of_record_scrubbed` in `crates/terminus-core/src/recall/excerpt.rs` (#62)
- A rooted fixture for the phase's shapes under `project-epsilon` - `tests/fixtures/session-redaction.jsonl`, registered in `crates/terminus-core/src/testkit.rs`, described in `tests/fixtures/README.md`
- The per-turn egress test across both fixtures - `crates/terminus-core/tests/egress.rs` (#4); keyed-value tests in `crates/terminus-core/tests/redacted_recall.rs`
- Process-boundary tests for the multi-byte turn with the knob on: the three MCP tools (`crates/terminus/tests/mcp.rs`), both injection hooks (`brief.rs`, `prompt.rs`), and `terminus search` and `terminus show` (`recall_cli.rs`)
- Docs: README's "Current state" section removed; `THREAT-MODEL.md`, `docs/configuration.md`, `CHANGELOG.md` and `docs/design/0006-recall-through-mcp.md` describe the `sk-` rule, the keyed values and the nine rules

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 2c3b5d2 | Rule 5 reads its name as ASCII bytes; five unit tests, fuzz crash input runs clean |
| 1 | 2 | bd3d9b8 | Rule 9 for a bare `sk-` key under its own marker; four unit tests |
| 2 | 1 | 312ca14 | Rooted fixture `session-redaction.jsonl` under `project-epsilon` |
| 2 | 2 | bbe41a5 | Keyed values replaced on the parsed record before the projection |
| 2 | 3 | 3484ddc | Per-turn check of each planted shape across both fixtures |
| 3 | 1 | d0cffd6 | The three MCP tools answer the multi-byte turn with the knob on |
| 3 | 2 | b3c1715 | Both injection hooks inject the multi-byte turn with the knob on |
| 3 | 3 | e4b578d | `terminus search` and `terminus show` print the multi-byte turn |
| 3 | 4 | 034ece3 | Docs say what redaction now matches and what it still misses |

## Deviations

- [deviation] Plan 3 task 4: README's status callout ended with a link to the "Current state" heading the task removed, so that sentence was removed too and nothing else (034ece3). The callout still reads "Status: built, with known gaps." with no list of gaps on the page.

## Open items

- README's "Status: built, with known gaps." no longer points to any list of gaps. Whether it keeps that wording is John's call.
- A string under an ordinary key inside an object or list held by a secret-named key (`{"api_token":["X"]}`) is not replaced on any surface, `recall_get` included. It is the scope D-02 locked and PRIV-06 states; the plan 2 risk review raised it at high and it was ruled downgraded on that basis (`ADJUDICATION-risk_surface-plan-2.json`).
- Issues #62, #5, #61 and #4 are not closed yet. The change has to close them when it lands (success criterion 5).
- With rule 4 removed, the egress test fails first on `session-secrets.jsonl`'s password row, so whether the three new keyed rows fail on their own in the `recall_get` check was not observed.
- `cargo fmt --check` reports existing diffs in `crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `crates/terminus/tests/release_workflows.rs` and `crates/terminus/tests/sign_release.rs`, none touched by this phase.
- `cargo doc` warns that the public `egress` module doc links the private `flag_value_span`. That predates this phase.

## Goal check

The nine commits plausibly deliver the goal. #61's root cause is fixed in the rule itself with no `catch_unwind` (2c3b5d2), and reverting it fails the unit tests, the MCP test ("byte index 25 is not a char boundary; it is inside 'é'"), both hook tests and both CLI tests, per the plan 1 and plan 3 reports. A bare `sk-` key is caught with its own marker and the 19/20-character floor pinned (bd3d9b8). Keyed values are replaced through the one call in `of_record_scrubbed` that D-05 named, so `recall_search`, `recall_context`, the brief and the prompt hook share it (bbe41a5), and removing that call fails the keyed search, injection and nested tests. The egress test removes each of the nine rules and the keyed step in turn and fails on that rule's row (3484ddc). The final workspace run passed 951 tests across 67 binaries with 0 failures (plan 3 report). Two things fall short of the letter of the success criteria: the issues are not closed until the change lands, and keyed values held inside a non-string under a secret-named key still go out, which is the scope D-02 accepted rather than a miss of the plans.
