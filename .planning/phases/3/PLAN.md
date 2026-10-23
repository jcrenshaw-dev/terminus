---
phase: 3
plan: 1
requirements:
  - GATE-01
  - GATE-02
files:
  - crates/terminus/tests/hook.rs
---

# Phase 3: Hook tests on macOS Intel - Plan

## Goal

The hook timing tests stop failing the merge gate on macOS Intel (#75, #104). The stdout test proves the ingest was running without racing a `ps` poll against a pass that can finish first. A timing failure of either #104 test says whether the hook blocked or the runner stalled.

## Must be true when done

- `reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs` holds its ingest at a testkit fault point. It proves the read returned while the pass was running from two facts: the fault point's ready file exists, and no `runs` row existed when the read returned. It calls no `await_process`. Its one `ps` call is a single lookup of the held, live ingest so the test can kill it.
- `cargo test -p terminus --all-features --test hook` passes in debug and in `--release`, and the stdout test passes 20 `--release` runs in a row.
- With the detached ingest made to inherit the hook's stdout, the stdout test fails at its 250 ms limit, not after the 30 s hold, and says a process the hook started held stdout open. With the fault point's variable removed, it fails within its arrival deadline and the message names `testkit`.
- A timing failure of either #104 test prints three things: the longest gap between its polls, how long a `/usr/bin/true` spawn took, and the hook's own evidence. It names exactly one verdict: the hook was blocked, the runner stalled, or (stdout test only) a process the hook started held stdout open.
- The 250 ms and 10 s limits stand, `.github/workflows/test.yml` is unchanged, `crates/terminus/tests/hook.rs` is the only file the phase changes, and no test function other than the two #104 tests changes.
- The `macos-15-intel` release job passes the stdout test in every CI run of the milestone's pull request (human-verify: needs GitHub Actions on the milestone pull request).

## Context

- Bound by CONTEXT D-01 to D-12:
  - The hold uses the existing fault point in `terminus_core::ingest::fault` (`AT`, `READY`, `stall` and its 30 s `STALL`). The detached ingest reaches it because `cmd::spawn::detached` passes the hook's environment on unchanged.
  - The proof is the ready file plus no `runs` row when the read returned.
  - `await_process` stays for the two sibling tests.
  - The test does not fall back to the old poll, and the arrival timeout names `testkit`.
  - The #104 work is a diagnosis, not prevention. The limits stay, with no retry and no skip.
  - The gap and the `/usr/bin/true` spawn are measured on the failure path only.
  - No planning tags go in comments or messages, and `cargo fmt` is not run.
- Patterns to follow:
  - `kill_at` in `crates/terminus/tests/crash.rs`: the ready file in the data dir, the 20 s `ARRIVAL_TIMEOUT`, and a message naming testkit.
  - `committed` for a read-only store read.
  - The timeout arm of `an_unterminated_line_stops_being_read_at_the_cap`: `kill`, then `wait_for`, then `read_for`.
  - The `spawn`/`output` wrappers every spawn in the file goes through, because of `COPYING`.
- CONTEXT's line numbers are at edf5c26. Re-find each site by its text.
- Named commands used in the Verify fields below:
  - **The stdout run** is `cargo test -p terminus --all-features --test hook -- --exact reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`, with `--release` before `--test` for the release build.
  - **The stdin run** is the same command with `a_stdin_that_is_opened_and_never_closed_does_not_hold_the_hook`.
  - **The 20-run loop** is `for i in $(seq 20); do cargo test -p terminus --all-features --release --test hook -- --exact reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs 2>&1 | grep -q 'test result: ok. 1 passed' || echo "run $i failed"; done`. It must print nothing.
  - **The scope check** has two parts. `git diff --name-only ba581ad -- . ':(exclude).planning' ':(exclude)fuzz/Cargo.lock'` prints only `crates/terminus/tests/hook.rs`, and `git diff --quiet ba581ad -- .github/workflows/test.yml` exits 0.
  - **The tag check** also has two parts. `cargo test -p terminus-core --test planning_tags` passes. `git diff ba581ad -- crates/terminus/tests/hook.rs | grep '^+' | grep -E '\b[A-Z]{2,6}-[0-9]{2}\b|\bD-[0-9]{2}\b|\bAC[0-9]+\b|\bPLAN-[0-9]+\b|\b([Pp]hase|[Tt]ask) [0-9]'` prints nothing, so no tag appears in a message either.
  - **The warning check** is `cargo test -p terminus --all-features --test hook --no-run 2>&1 | grep 'tests/hook.rs'`, which must print nothing.
- Temporary mutations used in the Verify fields below. Each one is reverted before the task commits:
  - **The inherit mutation**: in `crates/terminus/src/cmd/spawn.rs` `spawn_from`, change `.stdout(Stdio::null())` to `.stdout(Stdio::inherit())`. After reverting it, `git diff --exit-code -- crates/terminus/src/cmd/spawn.rs` must exit 0.
  - **The no-fault mutation**: delete the line that sets `fault::AT` on the stdout test's hook command.
  - **The zero-limit mutation**: set the test's limit to zero.
  - **The zero-stall-line mutation**: set the test's stall line to zero.

## Tasks

### Task 1: Hold the stdout test's ingest at a testkit fault point and prove the read from the ready file and the runs table

- **Files:** crates/terminus/tests/hook.rs (`reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`; a new read-only `runs` counter beside `committed`; the `use` lines)
- **Action:**
  - Import `terminus_core::ingest::fault` the way `crash.rs` does.
  - **Ready file.** Before spawning the hook, create `hook.data_dir` and choose a ready-file path inside it, as `kill_at` in `crash.rs` does. `fault::stall` drops a failed write (`let _ = std::fs::write`), so a ready path whose directory does not exist never appears. The test would then blame inert fault points for what is really a path bug.
  - **The hold.** On the hook's command, set `fault::AT` to `fault::AFTER_BLOB` and `fault::READY` to that path, next to what `Hook::command` already sets.
    - The detached ingest inherits this environment through `cmd::spawn::detached`. Its tree pass stalls in `prepare` once the blob is built and before any transaction.
    - The walk's one `runs` row comes from `record_pass`, after the walk, so it cannot be written while the pass is held.
    - `AFTER_BLOB` is the earliest per-file point and sits outside the transaction, so a kill there leaves nothing half written.
    - Do not add a new testkit hold, a FIFO transcript, or a bigger transcript (D-02).
  - **Transcript size.** The hold keeps the pass running, not the transcript's size. Write this test's transcript with `big_transcript` at a repeat count of 1 instead of `BIG`, because the pass needs only one file with work to reach `fault::AFTER_BLOB`. `BIG` and `big_transcript`'s doc comment stay as they are for the two sibling tests.
  - **Unchanged here.** Keep the blocking read, the empty-stdout assertion, the `elapsed < 250 ms` assertion and its message, and the hook's exit-status assertion as they are. Tasks 2 and 4 change the read and the message.
  - **Runs-row sample.** As soon as the read returns, count the store's `runs` rows with a new read-only helper shaped like `committed`:
    - A missing database file, a failed read-only open and a failed query all count as zero.
    - Never open the store the ordinary way, because that would create the database file the ingest has not made yet.
  - **Arrival wait.** After the read assertions, wait for the ready file with a 20 s arrival deadline, the value `crash.rs` and `backfill.rs` use for `ARRIVAL_TIMEOUT`. Poll the file, nothing else.
    - If the deadline passes, fail with a message that names `testkit`. The fault points are inert without `terminus-core`'s `testkit` feature, which this crate's dev-dependency turns on for its test build.
    - The message also says whether a `runs` row exists at that moment, which tells whether the pass ran to the end without stopping at the fault point (D-01).
    - There is no fallback to `await_process` or to any other poll (D-06).
  - **Proof assertion.** Only after the arrival wait, assert the sample was zero. On failure, the message says the pass had already recorded its `runs` row when the read returned, so the read proves nothing about a running pass. The arrival wait comes first so that a build with inert fault points always fails with the testkit message, however fast the pass is.
  - **End the hold.** Take one `ps()` snapshot filtered to `hook.ingest_argv()`. It must hold exactly one process: the held ingest, alive for the 30 s stall. Otherwise fail with the count found. `sigkill` that pid, then `drain(&hook.ingest_argv())`.
    - This is the lookup D-03 allows: it looks up a process the hold keeps alive, and never polls for one that may have exited.
    - I chose it over two alternatives. Writing the pid into the ready file would mean a change to `fault::stall`, which `crash.rs` and `backfill.rs` share, and gains nothing while the hold keeps the process alive. Waiting out the 30 s stall would cost 30 s per run and ten minutes for the 20-run loop.
  - **`await_process`.** Remove this test's call to `await_process`. The function stays for the two sibling tests (D-05).
  - **Comments.** Rewrite this test's doc comment and inline comments to describe the hold and the proof. Remove the comment that sizes the pass at roughly 700 ms.
- **Verify:**
  - The stdout run passes in debug and in `--release`.
  - The 20-run loop prints nothing.
  - `perl -ne 'print if /^fn reading_the_hooks_stdout_to_eof/../^}/' crates/terminus/tests/hook.rs | grep -c await_process` prints 0. The same extraction contains `fault::AFTER_BLOB`.
  - With the no-fault mutation, the stdout run fails, its panic message contains `testkit`, and its `test result:` line reports `finished in` under 60 s. After restoring the line, the stdout run passes again.
  - The tag check, the warning check and the scope check pass.

### Task 2: Bound the stdout test's read by its 250 ms limit

- **Files:** crates/terminus/tests/hook.rs (`reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`)
- **Action:**
  - **Reading thread.** Read the hook's stdout to EOF on a thread of its own, as `read_for` does. The thread sends back the bytes and how long after `started` the read returned, measured on that thread. That way a poll that wakes late does not charge its own delay to the hook.
  - **Poll loop.** The test thread polls for that result with a fixed 5 ms sleep until the result arrives or 250 ms have passed since `started`.
  - **On-time failures.** The test fails on time in two cases: the limit passes with no result, or a result arrives whose own elapsed time is 250 ms or more. Both keep the existing message wording ("...: the ingest is holding a handle the hook inherited") with the time. Task 4 replaces that wording.
    - Leave the reading thread unjoined on that path. A pipe held by a stalled ingest stays open for the whole 30 s hold, and joining the thread would bring back the wait this deadline removes (D-04).
  - **Why a poll loop.** CONTEXT left open whether the stdout test gets a poll loop of its own. It does: the loop gives the read its deadline, and Task 4 measures its poll gap in it (D-08).
  - **Runs-row sample.** Task 1's sample is now taken when the poll sees the result. A row absent then was also absent when the read returned, because a pass only adds its row. Everything Task 1 added after the read stays as it is.
  - `read_for` itself is unchanged, because the cap test still uses it.
- **Verify:**
  - The stdout run passes in debug and in `--release`.
  - With the inherit mutation, the stdout run fails with the on-time message, and its `test result:` line reports `finished in` under 5 s. Before this task, the same mutation held the read for the 30 s hold.
  - After reverting the mutation, `git diff --exit-code -- crates/terminus/src/cmd/spawn.rs` exits 0.
  - The tag check, the warning check and the scope check pass.

### Task 3: Diagnose a timing failure of the stdin test as a blocked hook or a stalled runner

- **Files:** crates/terminus/tests/hook.rs (`a_stdin_that_is_opened_and_never_closed_does_not_hold_the_hook`; new code beside `wait_for` and `read_for` for the stall evidence and verdict)
- **Action:**
  - **Gap tracking.** In the test's existing `try_wait` loop, record the gap between consecutive polls, counting the first from `started`, and keep the longest. The 10 ms sleep and the 10 s limit stay.
  - **Spawn timing.** When the limit passes with the hook still running, first time one spawn of `/usr/bin/true`.
    - Spawn it through `output`, so it takes the `COPYING` read guard like every spawn in this file. The time therefore includes any wait on a sibling test's binary copy, and the comment says so.
    - If the spawn fails or exits non-zero, report that in the message with its error. Do not panic a second time.
    - This spawn runs only on the failure path (D-08).
  - **Hook evidence.** Next, collect the hook's own evidence the way the cap test's timeout arm does: kill the hook, `wait_for` it for up to 5 s, then `read_for` its stderr for up to 5 s.
    - The kill has to come first, because the hook holds its stderr open until it ends.
    - The evidence is whether stderr holds the line `cmd::hook::drain` writes when it gives up on stdin ("stopped waiting on a stdin that stayed open for N ms"), matched on `stopped waiting on a stdin`. The captured stderr goes into the message word for word.
  - **Verdict.** The message names exactly one verdict:
    - "the runner stalled", when the longest gap or the spawn time is at or above the stall line;
    - otherwise "the hook was blocked". The evidence then says whether the hook was still blocked on stdin (no drain line) or had got past stdin but not exited (drain line present).
  - **Stall line.** The stdin test's stall line is 5 s, half its 10 s limit. A single freeze of half the limit is enough on its own to make the timing say nothing about the hook.
    - Hold it as a value of its own beside the limit, not computed from it, so forcing the limit to zero in a check leaves the verdict line where it is.
  - **Shared code.** Put the spawn timing and the stall-line verdict outside the test function, taking the stall line as an input, because Task 4 reuses them for the stdout test. Name them as you choose.
  - **The message.** It replaces the one that always blames an unbounded read on stdin. It carries the elapsed time, the longest gap, the spawn time, the drain-line evidence with the captured stderr, and the verdict.
  - **Success path.** Unchanged: the exit status, the empty stdout, the one stderr line containing "stdin", `drop(stdin)`, and `drain`.
- **Verify:**
  - The stdin run passes in debug and in `--release`.
  - With the zero-limit mutation on this test, the stdin run fails. Its panic message shows:
    - the longest gap between polls in ms;
    - the time a `/usr/bin/true` spawn took;
    - whether the hook had logged that it stopped waiting on stdin;
    - the verdict "the hook was blocked".
  - `grep -oE 'the hook was blocked|the runner stalled' <captured output> | sort -u | wc -l` prints 1.
  - With the zero-limit and zero-stall-line mutations together, the message names "the runner stalled".
  - After restoring both, the stdin run passes again, and the file still holds the 10 s limit.
  - The tag check, the warning check and the scope check pass.

### Task 4: Diagnose a timing failure of the stdout test three ways

- **Files:** crates/terminus/tests/hook.rs (`reading_the_hooks_stdout_to_eof_returns_at_once_while_the_ingest_still_runs`)
- **Action:**
  - **Gap and exit tracking.** In Task 2's poll loop, record the gap between consecutive polls, counting the first from `started`, and keep the longest. On each poll, before the limit is checked, call `child.try_wait()` and record when the hook's exit was first seen. The later `child.wait()` returns the status that `try_wait` already collected.
  - **Diagnosis path.** Both of Task 2's on-time failures take the same path. On that path only, time the `/usr/bin/true` spawn with Task 3's shared code.
  - **Verdict.** Judge it with a stall line of 125 ms: half the 250 ms limit, held as a value of its own for the same reason as Task 3's. The message names exactly one verdict:
    - "the runner stalled", when the longest gap or the spawn time is at or above 125 ms;
    - otherwise "a process the hook started held stdout open", when the hook's exit was seen before the read returned, or before the limit if the read is still open;
    - otherwise "the hook was blocked".
  - **The message.** It replaces "the ingest is holding a handle the hook inherited". It carries the read's state and time, whether and when the hook had exited, the longest gap, the spawn time, and the verdict.
  - Everything Task 1 added after the read stays as it is.
- **Verify:**
  - The stdout run passes in debug and in `--release`, and the 20-run loop prints nothing.
  - With the inherit mutation, the stdout run fails, and its `test result:` line reports `finished in` under 5 s. Its panic message:
    - says the hook had exited;
    - shows the longest gap and the `/usr/bin/true` spawn time;
    - names "a process the hook started held stdout open".
  - After reverting the inherit mutation, `git diff --exit-code -- crates/terminus/src/cmd/spawn.rs` exits 0.
  - With the zero-limit mutation on this test, the panic message shows the longest gap, the spawn time, and whether the hook had exited. `grep -oE 'the hook was blocked|the runner stalled|a process the hook started held stdout open' <captured output> | sort -u | wc -l` prints 1.
  - With the zero-limit and zero-stall-line mutations together, the message names "the runner stalled". Restore both.
  - `cargo test -p terminus --all-features --test hook` passes in debug and in `--release`.
    - If it fails only in `every_event_exits_zero_with_an_empty_stdout_inside_the_budget` or `every_event_stays_inside_the_budget_with_the_redaction_knob_on` while the machine is loaded, record the load average and rerun once, as phase 2 did (`.planning/phases/2/SUMMARY.md`).
    - Any other failure stands.
  - The scope check, the tag check and the warning check pass.
  - `grep -nE 'from_secs\(10\)|from_millis\(250\)' crates/terminus/tests/hook.rs` still shows both limits.
  - Every hunk of `git diff -U0 ba581ad -- crates/terminus/tests/hook.rs` lies inside one of the following: the two #104 test functions, a helper they call, the new `runs` counter, or the `use` lines.

## Notes

- **Requirements.** GATE-01 is Tasks 1 and 2, and GATE-02 is Tasks 3 and 4. All four tasks change one file, so this is one plan run in order. That matches the CONTEXT plan shape.
- **CI proof.** Roadmap criterion 2 (AC4) is proved only on the `macos-15-intel` release job across the milestone pull request's own runs. It stays open in this phase's UAT until that pull request runs. The phase makes no push of its own (D-11).
- **Mutations.** The inherit mutation edits `crates/terminus/src/cmd/spawn.rs`, which this plan does not lease. It is reverted before each task commits, and so is every other mutation.
- **Pre-existing changes.** `fuzz/Cargo.lock` and `.planning/config.json` were already modified in the working tree before this phase. They are not this plan's, and no task commits them.
- **Stall lines on macOS.** Starting `/usr/bin/true` once has taken up to 150 ms on a healthy macOS runner (`.github/workflows/test.yml`), which is above the stdout test's 125 ms line. So a stdout miss on such a runner reads as a runner stall. That is the intended reading, because the hook must itself spawn before it exits on that runner. The message prints the raw numbers either way.
- **Load.** The two p99 budget tests fail under heavy local load, as `.planning/phases/2/SUMMARY.md` records (load 12.6, green on rerun). They are outside this phase (D-10), so the full-file run in Task 4 allows the one recorded rerun for them alone.
