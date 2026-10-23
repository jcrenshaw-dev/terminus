---
status: testing
phase: 3
fields_version: 1
started: 2026-10-23
updated: 2026-10-23
---

## Items

### 1. The stdout test holds the ingest at a fault point instead of polling ps
expected: reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs calls no await_process and runs no ps poll waiting for the ingest to appear. It holds the ingest at a testkit fault point and proves the read returned mid-pass from the fault point's ready file and from no runs row existing when the read returned.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: hook.rs:738-888: fault::AT=AFTER_BLOB and READY at :754-755; await_process count in the fn is 0 (only :278, :1025); ready-file-only arrival wait :852-863; runs_at_eof sampled :809 and asserted 0 at :867-871; a single ps lookup of the held ingest :875-886. The stall (ingest/mod.rs:506) precedes every runs insert (mod.rs:672-673, pass.rs:508-554, backfill.rs:445-486). The stdout run passed in debug and release.

### 2. The hook tests pass in debug and release, and the stdout test passes 20 release runs in a row
expected: cargo test -p terminus --all-features --test hook passes in debug and in --release, and the stdout test run 20 times in a row with --release passes every time.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: Hook file: 8 passed in debug (1.35s) and in --release (0.52s) at load 0.62. The plan's 20-run --release loop had 0 failures.

### 3. The stdout test fails fast and names the cause when stdout is held or the fault point is missing
expected: With the detached ingest made to inherit the hook's stdout, the test fails and says a process the hook started held stdout open. With the fault point's environment variable removed, it fails within its arrival deadline with a message naming testkit, not at CI's 15-minute kill.
criterion: AC3
status: pass
first_pass: pass
source: model
evidence: Held stdout: changed spawn_from in crates/terminus/src/cmd/spawn.rs to .stdout(Stdio::inherit()), ran `cargo test -p terminus --all-features --test hook -- --exact reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`: panicked at hook.rs:826 "still open after 253.5 ms against a limit of 250.0 ms; the hook had exited, first seen 5.1 ms after the read began; the longest gap between polls was 5.1 ms; a /usr/bin/true spawn took 0.4 ms. Verdict: a process the hook started held stdout open", finished in 0.26s. Fault variable removed: deleted the .env(fault::AT, fault::AFTER_BLOB) line, same command: panicked at hook.rs:853 "the ingest never reached `after-blob` within 20s, and 1 `runs` row(s) exist now: the fault points are inert unless terminus-core is built with its `testkit` feature", finished in 20.01s. Both files restored; git diff --exit-code exited 0 for each.

### 4. The macos-15-intel release job passes the stdout test on the milestone pull request
expected: The macos-15-intel release job passes the stdout test in every CI run of the milestone's pull request (human-verify: needs GitHub Actions on the milestone pull request)
criterion: AC4
status: pending

### 5. A forced timing miss in either #104 test names one verdict with its evidence
expected: With its limit set to 0, each of the two #104 tests prints the longest gap between its polls, how long a /usr/bin/true spawn took, and the hook's own evidence (stdout test: whether the hook had exited; stdin test: whether the hook logged that it stopped waiting on stdin), and names one verdict: the hook was blocked, or the runner stalled. The stdout test can also name a process the hook started holding stdout open.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: Seven observed misses on the real limits. Stdout: blocked (gap 5.1 ms, spawn 0.3 ms, hook had not exited), runner stalled (gap 203.3 ms), held stdout (hook had exited at 5.1 ms). Stdin: blocked without the drain line, runner stalled (gap 6004.5 ms), blocked with the drain line present (stderr quoted). Each output names exactly 1 distinct verdict.

### 6. The phase's diff stays inside the two #104 tests
expected: The phase's diff leaves .github/workflows/test.yml unchanged, keeps the 250 ms and 10 s limits, and changes no test function other than the two #104 tests.
criterion: AC6
status: pass
first_pass: pass
source: verifier
evidence: The scope check from ba581ad lists only hook.rs. test.yml diff is quiet. Limits are at :451 and :767. -U0 hunks fall only in the use line, the stdin test, the new helpers time_a_spawn/runner_stalled/ms, the stdout test, and the new runs helper.

## Summary

total: 6
passed: 5
failed: 0
pending: 1
skipped: 0
blocked: 0
reworked: 0
