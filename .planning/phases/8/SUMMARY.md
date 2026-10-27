---
phase: 8
status: complete
completed: 2026-10-27
---

# Phase 8: Output - Summary

Every command's human output escapes archived controls while documents stay raw, every `--json` failure is a document, sizes print through one formatter, the purge size counts everything `--purge` deletes, and the prompt line marks a turn it cut to a share of 3 or less.

## What shipped

- The output mode reaches the dispatcher, so `main` writes the failure document for every `--json` failure and `read::unusable` is gone - `crates/terminus/src/lib.rs`, `crates/terminus/src/cmd/`
- `outln!`, `out!` and `errln!` escape by default; documents, the hook document and `show`'s body write raw; `print_stderr` is denied - `crates/terminus/src/out.rs`, `crates/terminus/src/escape.rs`
- One byte formatter (`cmd::human`), one duration formatter, `Failure`'s one text, and the cleanup days defined once - `crates/terminus/src/cmd/mod.rs`, `crates/terminus/src/cmd/install/mod.rs`
- `uninstall --purge` measures the whole data directory through `cmd::directory_bytes` - `crates/terminus/src/cmd/mod.rs:172`, `crates/terminus/src/cmd/uninstall.rs:453`
- The prompt line shows a cut its share cannot hold as `...` alone and records only the turns it showed with text - `crates/terminus-core/src/inject/prompt.rs`, `docs/design/0007-injection.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 758ebb5 | Carry the output mode into the dispatcher |
| 1 | 2 | 2b5c716 | Build each parsing command's empty document from its parsed arguments |
| 1 | 3 | 3908149 | Main writes the failure document; `read::unusable` deleted |
| 1 | 4 | f7e64b3 | `json-shapes.md`: every `--json` failure is a document, misuse is not |
| 2 | 1 | 819a3a6 | Escape by default in `outln!`/`out!`; documents and the hook write raw |
| 2 | 2 | 8d1b8e3 | Route `show` and `doctor` through the escaping write; per-field escapes dropped |
| 2 | 3 | 217f2d0 | Escape stderr through `errln!`; deny `print_stderr` |
| 2 | 4 | 1b3e6f5 | Docs say every command's human output escapes |
| 3 | 1 | e2bf4b3 | Every byte count through `cmd::human` |
| 3 | 2 | 952b320 | Every duration through one function |
| 3 | 3 | a59c822 | `Failure` gets one text |
| 3 | 4 | 0a80736 | Cleanup days defined once |
| 4 | 1 | d57ec87 | Purge size counts every file under the data directory |
| 5 | 1 | 5242337 | A prompt-line cut the share cannot hold shows as the mark alone |
| 5 | 2 | b200aac | Record only the prompt-line turns shown with text |

## Deviations

- [deviation] Plan 5 task 1: the plan's Action let a mark overspend its share by up to 3 characters and left that to `frame`'s backstop. CONTEXT D-17, the later locked decision, says `render` counts the mark into the fixed cost. The executor followed D-17. At a budget that cannot hold the ids plus the marks (a spare of 0 to 2 characters for one turn), the backstop cuts the line, so it does not show `...`; the budget still holds. 5242337.

## Open items

- Plan 3 task 3: `grep -rn 'Failure::Silent =>' crates/terminus/src` shows two arms, not one. `lib.rs:104` maps `Silent` to exit code 1, which is not a failure text, so it was left alone.
- Plan 4: the first full-suite run failed once in `install_script::a_rebuild_at_the_same_version_replaces_its_file::sh`, outside the plan's files. It passed alone and in the confirming `--no-fail-fast` run, so it reads as a flake.
- Plan 5's deviation: at a spare of 0 to 2 characters for one turn the line is cut by `frame`'s backstop rather than showing `...`. Verify should check this against success criterion 5.

## Goal check

The commits plausibly deliver the phase goal. Criterion 1 (escaping) is 819a3a6, 8d1b8e3 and 217f2d0, which the plan 2 report says add `escaped_output.rs`, doctor, show and stderr tests. Criterion 2 is 758ebb5, 2b5c716 and 3908149, and `grep -rn 'fn unusable\|read::unusable' crates/terminus/src` returns nothing. Criterion 3 is e2bf4b3, and a grep for a quoted number followed by `GB`, `MB` or `KB` in `crates/terminus/src` returns nothing. Criterion 4 is d57ec87, with `uninstall.rs:453` measuring through `directory_bytes` and `tests/uninstall.rs:802` planting files in `snapshots/` and `decisions/`. Criterion 5 is 5242337 and b200aac, with one gap to check: plan 5's deviation leaves the line to the backstop when the budget cannot hold the ids plus the marks, so at that budget it is cut rather than marked. This paragraph does not run the tests; the reports say each plan's suites passed, and plan 4's report names one flake outside its files.
