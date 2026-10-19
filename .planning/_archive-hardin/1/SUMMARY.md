---
phase: 1
status: complete
completed: 2026-10-18
---

# Phase 1: Recall injection framing - Summary

Both injection hooks now leave tool-result turns out and quote everything else inside one fixed `<terminus-recall>` tag pair under a quoted-history header, with planted closing tags neutralized (#60).

## What shipped

- Tool-result turns (`user` turns with `turns.is_typed` = 0) are never injected by the prompt hook, through the shared `prompt::quotable` step - `crates/terminus-core/src/inject/prompt.rs:441`, `:578`
- `terminus replay` drops the same turns, so it credits what the live hook injects - `crates/terminus-core/src/feedback/replay.rs:308`
- The frame (`OPEN_TAG`, `CLOSE_TAG`, `QUOTED_HISTORY`, `frame`, `frame_fits`) and the closing-tag neutralizer, defined once - `crates/terminus-core/src/inject/mod.rs:281`, `:482`
- The prompt hook and the `SessionStart` brief both frame their output and neutralize every archived string they quote (excerpts, days, project labels, branches); a budget too small for the frame writes nothing - `inject/prompt.rs:288`, `inject/brief.rs:218`
- How Claude Code 2.1.302 hands hook context to the model, recorded from the bundle - `THREAT-MODEL.md`
- The frame and the exclusion described in `docs/design/0007-injection.md`, `THREAT-MODEL.md` and `CHANGELOG.md`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 37fa479 | Record how Claude Code 2.1.302 hands hook context to the model |
| 1 | 2 | 6f6ed4f | Keep tool results out of the prompt hook's injection |
| 1 | 3 | 75c3a1f | Hold `terminus replay` to the same tool-result exclusion |
| 2 | 1 | 05daa09 | Define the recall frame and the closing-tag neutralizer once |
| 2 | 2 | 773bf77 | Frame the prompt hook's injection as quoted history |
| 2 | 3 | 2f1fc4b | Frame the `SessionStart` brief, pointer and all |
| 2 | 4 | e918032 | Describe the recall frame and the tool-result exclusion |
| 2 | fix | 9f3ac99 | Let any non-ASCII char sit inside a closer the neutralizer catches |

## Deviations

- [review fix] The blocking `risk_surface` review on plan 2 found that the neutralizer's combining-mark table covered five Unicode blocks, so a closing tag with a Cyrillic, Hebrew, Arabic or Thai mark inside it passed through. Confirmed by probe. John chose to let any non-ASCII character sit inside a closer; fixed in 9f3ac99, which also makes a run of non-ASCII brackets linear. The narrowed re-review of the fix found nothing.
- [review] The plan 1 `risk_surface` review (one high, on an old store lacking `is_typed`) and the plan 2 review's medium (a budget exactly equal to the frame cost writes nothing) were both refuted against the code; rulings in `ADJUDICATION-risk_surface-plan-1.json` and `ADJUDICATION-risk_surface-plan-2.json`.

## Open items

- Claude Code 2.1.302 wraps hook context in its own `<system-reminder>` pair and escapes look-alikes of that tag. Whether archived text could close the host's wrapper, and how the header should sit inside the host's frame, are John's call. Whether the user sees hook context is not settled; both [VERIFY] marks in `THREAT-MODEL.md` stay.
- `THREAT-MODEL.md` carries a [VERIFY] that the header's effect on a model is unmeasured.
- `terminus replay` still counts turns as injected when `prompt_chars` is under 240, where the live hook now writes nothing. Replay never modelled the budget.
- `docs/design/0007-injection.md`: Figure 2 shows neither the exclusion nor the frame; the replay section's function list leaves out `quotable`; lines 90 and 118 still describe the old ignorable set (whitespace, default-ignorables, combining marks) rather than whitespace and any non-ASCII character.
- `cargo fmt --check` reports pre-existing diffs in files this phase did not touch (`crates/terminus/src/cmd/hook.rs`, `crates/terminus/tests/hook.rs`, `crates/terminus/tests/release_workflows.rs` and others).

## Goal check

The commits plausibly deliver the goal. The prompt hook drops tool-result turns before the cap at `inject/prompt.rs:441`, and the planted-Read regression test `a_read_result_is_never_injected_but_the_read_still_fires` fails with that call bypassed and passes with it (plan 1 report). Replay applies the same step at `feedback/replay.rs:308`, and its test shows the replayed total equal to what the hook selects. Both hooks frame their output through one `frame` (`inject/prompt.rs:288`, `inject/brief.rs:233`), and every archived string they quote goes through `neutralize`; the planted-closer tests cover nine variants in the prompt arm and eight spans in the brief, and the fix added four scripts' combining marks. The budget sweeps run from 0 to the unbudgeted length, and in each one, aiming the backstop clip at the whole text made the sweep fail. The full suite ran 932 passed, 0 failed after the fix (plan 2 report). What the code cannot show is whether a model actually treats the framed text as history, not instructions: that is the open [VERIFY] in `THREAT-MODEL.md`. Both hooks' output also still sits inside Claude Code's own `<system-reminder>` wrapper, which this phase recorded and left for John.
