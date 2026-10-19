---
phase: 5
status: complete
completed: 2026-10-19
---

# Phase 5: Terminal output escaping - Summary

A shared `escape::controls` helper in the `terminus` binary now prints every C0 control except tab and newline, DEL and every C1 control in archived text as its lowercase `\u00XX` escape in the human output of `search`, the `show` header and window, and `observations`. `--json` and the `show` record body are unchanged.

## What shipped

- The helper `escape::controls(&str) -> Cow<str>` - `crates/terminus/src/escape.rs`, wired in `crates/terminus/src/lib.rs`
- Escaped excerpts and headers in `terminus search` - `crates/terminus/src/cmd/search.rs:147-152`
- Escaped header and `--before`/`--after` window in `terminus show`, with the record body left byte for byte - `crates/terminus/src/cmd/show.rs:110-174`
- Escaped file name, branch and fact lists in `terminus observations` - `crates/terminus/src/cmd/observations.rs:266-292`
- Regression tests that fail with the fix taken out - `crates/terminus/tests/recall_cli.rs` (`archived_control_characters_print_escaped_in_search_and_show`) and `crates/terminus/tests/observations.rs` (`archived_control_characters_print_escaped_in_the_listing`)
- The escape described in `THREAT-MODEL.md`, `docs/commands.md` and `CHANGELOG.md` (fixes #63, #65, #66)

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 23b2db8 | Add the helper that spells archived control characters as `\u00XX` |
| 1 | 2 | 4d6164c | Escape archived control characters in search and the show header and window |
| 1 | 3 | 5d9f4f4 | Escape archived control characters in the observations listing |
| 1 | 4 | 47d8718 | Describe the control-character escape in human output |

## Deviations

- [deviation] The plan said to commit Task 1 with the unused-function warning showing. The project's clippy run uses `-D warnings`, which turns that warning into an error, so the executor put `#[cfg_attr(not(test), expect(dead_code))]` on `mod escape;`. Task 2 (4d6164c) removed it once the helper had callers, so the final tree matches the plan. (23b2db8)
- [deviation] Task 2 asked for a control planted in the record type if ingest keeps such a record as a turn. It does not: `parse::record` makes a turn only for `user`, `assistant`, `attachment` and `system`. The test leaves the type clean and says why. `render_window` still escapes the type through the label. (4d6164c)
- [deviation] Task 3 asked for a control planted in the session's file name if ingest accepts such a name. The executor wrongly concluded it does not and left the file name clean. `discover::is_agent_transcript` admits any `agent-*.jsonl` name, so UAT added the planted name to the test; it fails with the escape removed from `Row::print`. (5d9f4f4, fixed in UAT)

## Open items

- `cargo fmt --check` reports drift in `crates/terminus/src/cmd/hook.rs` and `crates/terminus/tests/hook.rs` that predates this phase. It was left alone because those files are outside this plan.

## Goal check

The four commits plausibly deliver the goal. All three call sites the roadmap names now route archived text through the one helper: `search.rs:147-152` for the excerpt and its header, `show.rs:110-112` for the header and `show.rs:161-174` for the window, and `observations.rs:266-292` for the listing. The helper escapes `char::is_control` minus tab and newline (`escape.rs` `escaped`), which is U+0000 to U+001F and U+007F to U+009F. The executor's report states that both new regression tests failed with the fix stashed and passed with it restored, and that `cargo test --workspace --all-features --locked` was green with 68 test binaries. The `show` record body and `--json` are untouched by design (D-01, D-06), so DEL and C1 still reach the terminal through those two paths. `THREAT-MODEL.md:127` states that, so it is a documented limit and not a gap. The risk check matched the secrets category on a test constant (`PLANTED_TOKEN = "quixotrel"`, a search word), and the blocking cross-model review (openai, gpt-5.6-sol) of the range raised no findings.
