---
phase: 2
plan: 1
requirements: [PRIV-08, PRIV-07]
files:
  - crates/terminus-core/src/observe/egress.rs
---

# Phase 2: Redaction - Plan 1

## Goal

With `[privacy] redact_recall` on, the redaction rule set stops panicking on a multi-byte character in front of a `NAME=` run (#61) and catches a bare `sk-` token in prose (#5). This plan changes the rule set alone, in `observe/egress.rs`, with the unit tests that prove each fix. Plans 2 and 3 carry the fixture, the keyed values, the per-turn egress test and the process-boundary tests.

## Must be true when done

- `egress::for_model_context` hands back `café=1` and `Größe=10` unchanged and turns `“token=abc”` into `“token=[redacted]`. Today each of the three panics on a byte index inside a character.
- The `redaction` fuzz target's saved crash input (bytes `CA 32 3D 0A`) runs to completion.
- A bare `sk-` token with 20 or more key characters after the prefix, in prose with no name beside it, is replaced by the new rule's own marker. `sk-notification`, `task-notification` and an `sk-` run with 19 key characters come back unchanged.
- An `Authorization: Bearer sk-...` header and a secret-named `NAME=sk-...` assignment still come back as `Authorization: [redacted]` and `NAME=[redacted]`, with no `sk-` marker in them.
- Every existing egress, recall and injection test passes unchanged, so the output on ASCII text does not move.

## Context

- Locked: D-09 (rule 5 reads its name run as ASCII only and starts it after the whole character; it is the only slice in the set that can land inside one), D-10 (curly quotes stay content), D-12 (no `catch_unwind` around MCP requests), D-07 (a generic `sk-` prefix and a 20-character floor), D-08 (its own whitespace-free marker; after rules 3 to 6, beside rules 7 and 8; a whole run of credential bytes from a run boundary).
- Pattern: rules 7 and 8 are `redact_runs` over a byte class with a predicate and a marker constant. Rule 3's walk back over `is_name_byte` in `redact_header_spans`, and `is_flag_name_byte`, are the ASCII name reads D-09 points at.
- Out of this plan: the fixture, the keyed-value step, the per-turn egress test, the boundary tests and the user docs (plans 2 and 3). No new markers for rules 3, 4 or 5 (D-04).

## Tasks

### Task 1: Read rule 5's name as ASCII bytes (D-09, D-10)

- **Files:** crates/terminus-core/src/observe/egress.rs (redact_assignments; is_flag_name_byte; the tests module)
- **Action:** In `redact_assignments`, replace the `rfind` over `text[copied..at]` that adds 1 to the byte index of the character it found. That index is the character's first byte, so adding 1 lands inside `é` in `café=1`, inside `ß` in `Größe=10`, inside `“` in `“token=abc”` and inside U+FFFD in the fuzz crash, and the next slice panics. Walk back from the `=` over the bytes `is_flag_name_byte` accepts instead, stopping at `copied`, the way `redact_header_spans` walks back over `is_name_byte`; `is_flag_name_byte`'s doc already says it is the run rule 5 reads a name by. Every name byte is ASCII, so the run starts on a character boundary whatever stands in front of it. Leave the value half of the rule alone: D-09 checked it and every other rule, and none of them slices inside a character. Do not make curly quotes delimiters (D-10): the closing `”` stays in the value run, so `“token=abc”` becomes `“token=[redacted]`. Add no `catch_unwind` anywhere (D-12); this is the root-cause fix. Rewrite the function's doc comment to say why the name is read as bytes, naming the panic and #61, in the voice of the comments around it. Then add tests to the module's tests. `café=1` and `Größe=10` come back from `for_model_context` byte-identical, alone and inside prose. `“token=abc”` comes back exactly as `“token=[redacted]`. A secret name directly after a multi-byte character (`é` followed by an `API_KEY=` run, for one) still loses its value, which proves the run starts after the whole character rather than the assignment being skipped. The fuzz crash input, built from the bytes `CA 32 3D 0A` through `String::from_utf8_lossy` the way `fuzz/fuzz_targets/redaction.rs` builds it, returns normally through `for_model_context`, `Redaction::apply` and `Redaction::apply_json_line` on a redaction that is on; the tests module can build one, because the field is private to this file and the module is its child. A JSON line holding `café=1` in a string comes back from `apply_json_line` unchanged.
- **Verify:** `cargo test -p terminus-core --all-features --lib observe::egress` passes with the new tests. With the `rfind` form temporarily put back, the multi-byte tests fail with a panic saying the byte index is not a char boundary; restore the fix. From the repo root, `cargo +nightly fuzz run redaction fuzz/artifacts/redaction/crash-6d3bb0e0fc5b3fd9ca463f5bcd7278deb90aaa91` exits 0 (run it once before the change to watch it panic; the artifact is local and gitignored, and if the run rewrites `fuzz/Cargo.lock`, restore it, because nothing here changes a dependency). `cargo test -p terminus-core --all-features --test egress --test redacted_recall --test recall --test recall_golden --test inject_brief --test inject_prompt` and `cargo test -p terminus --all-features --test mcp --test recall_cli --test brief --test prompt` pass with no test edited. `git diff --stat` lists egress.rs alone.

### Task 2: Add a rule for a bare `sk-` token (D-07, D-08)

- **Files:** crates/terminus-core/src/observe/egress.rs (redact; beside REDACTED_GITHUB_TOKEN and MIN_GITHUB_TOKEN_BODY and redact_github_tokens; the module doc's rule list and "The direction to be wrong in"; the tests module)
- **Action:** Add a ninth rule, run in `redact` after `redact_github_tokens`, so rules 1 to 8 keep the numbers the docs, the tests and CONTEXT use (planner's choice; D-08 asks only that it run after rules 3 to 6, beside 7 and 8). Build it on `redact_runs` over `is_credential_byte` (ASCII letters, digits, `-` and `_`), so it only ever takes a whole run that starts at a run boundary and `task-notification` cannot match. Its predicate accepts a run that starts with the lowercase prefix `sk-` and has 20 or more bytes after it. The prefix is matched as written, the way `is_github_token` matches its prefixes; CONTEXT does not address case, so that is the planner's choice. Do not key on a list of provider prefixes (D-07). Keep the floor in a private constant documented the way `MIN_GITHUB_TOKEN_BODY` is, with D-07's reason, which runs the other way from rules 7 and 8's: this floor is sized against real keys and against the archive's placeholders as measured on 2026-10-18 (`sk-notification` 22 times, `sk-not-a-quoted-string` 64, `sk-abc` 156, and the rest D-07 lists), not against a test sentinel, and a floor of 6 would redact `sk-notification`. Give the rule its own public marker constant beside `REDACTED_GITHUB_TOKEN`: one whitespace-free `[redacted:...]` token like the others, documented for `REDACTED_JWT`'s reason. The marker must not hold a run that starts with `sk-` at a run boundary, or a second scrub would read the marker as a key. Add the rule to the module doc's numbered list with its marker. In "The direction to be wrong in", say how its floor was set and what it leaves out: a run with fewer than 20 key characters after `sk-`, and Stripe's `sk_live_`, whose underscore puts it outside the rule. Then add tests. Every sentinel has 20 or more key characters after `sk-` and is plainly fake: hyphenated words and a mark of its own, never `T3BlbkFJ`, `sk-ant-api03-` or `sk-proj-`, so GitHub push protection has nothing to match. A bare token in prose with no `:`, `=`, `--` flag or quoted pair near it is gone, the new marker stands where it was, and the prose around it is intact. A run with exactly 20 key characters after the prefix is caught, and one with 19 is not. `sk-notification` and `task-notification` come back unchanged inside prose. `Authorization: Bearer` with a 20-plus-character `sk-` value comes back holding `Authorization: [redacted]` and not the new marker, and so does a secret-named `NAME=` assignment, with `[redacted]`. A second `for_model_context` over each scrubbed text changes nothing.
- **Verify:** `cargo test -p terminus-core --all-features --lib observe::egress` passes with the new tests. With the new rule's call taken out of `redact`, the bare-token test fails and shows the token; putting it back makes it pass. `cargo test -p terminus-core --all-features --test egress --test redacted_recall --test recall --test recall_golden --test inject_brief --test inject_prompt` and `cargo test -p terminus --all-features --test mcp --test recall_cli --test brief --test prompt` pass with no test edited. `git diff --stat` lists egress.rs alone.

## Notes

- Sequential, not parallel. PLAN-2 also edits `observe/egress.rs`, and its egress-test rows use this plan's `sk-` marker. PLAN-3's boundary tests are proven by reverting Task 1. `plan-overlap` will report egress.rs, and the phase runs PLAN-1, then PLAN-2, then PLAN-3.
- Recalled from v0.0.1/phases/1/SUMMARY.md: #4 and #5 were filed from that phase's risk reviews, and #5 is the gap Task 2 closes.
- CONTEXT's flagged assumption stands unverified: every real `sk-` provider key is taken to run past 20 characters after the prefix.
