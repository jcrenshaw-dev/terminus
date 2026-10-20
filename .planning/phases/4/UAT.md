---
status: testing
phase: 4
fields_version: 1
started: 2026-10-20
updated: 2026-10-20
---

## Items

### 1. Before-and-after measurement is recorded
expected: MEASUREMENT.md in the phase directory records passes on a sqlite3 .backup copy of the live store for 57c036e and the final build: one discarded warm-up, then at least 5 timed runs per build, each with wall clock, runs.duration_ms, files_committed and a strace -c summary.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: MEASUREMENT.md has 6 timed plus 6 traced passes per build for 57c036e and fb88658 (crates identical to HEAD), after a discarded warm-up (run 1615), each with wall ms, duration_ms, files_committed and strace -c. Every number matches series.log, the copy's runs rows 1616-1639 and the 12 strace summaries. The copy came from `sqlite3 -readonly .backup`.

### 2. Unchanged file reads no block rows
expected: With the testkit read log on, a pass over a session whose transcript is unchanged logs no read of that session's blocks rows, hashes included; a grown file, a diverged session and a session whose file is gone still log their reads.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: tests/pass.rs:1459 passes at HEAD and fails with 2 reads on the pre-gate source (0fb7422 plus 8c50128's test). The grown, diverged and gone read tests (:1400, :1416, :1438) pass. A steady-state pass on the copy read 73 blocks pages, about one grown session's worth, while walking 2,915 unchanged files.

### 3. Unchanged file keeps its five outcomes
expected: New tests cover the held-key refusal, the no-session_meta refusal, the checksum refusal, the shorter-than-watermark flag and the transcript_path update for a moved file, and each passes both at 57c036e and at HEAD.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: The five tests (tests/pass.rs:1193-1336, walk and named file each) pass at 57c036e's source, run by name from `git archive 34071a6` (crates identical to 57c036e except pass.rs): 5 passed, 0 failed. They also pass at HEAD in the full suite, and their bodies are unchanged since 34071a6.

### 4. One writable open when no snapshot is due
expected: strace -e openat on the copy shows one successful O_RDWR open of terminus.db in a pass with no snapshot due; with a snapshot due the snapshot is taken and last_snapshot_at is written only after the copy lands.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: A fresh strace -e openat pass on the copy (run 1643, nothing due) shows 1 successful O_RDWR open of terminus.db, against 2 for the recorded baseline trace. Snapshot due, in a scratch tree: a new snapshot is taken, the stamp goes from 0 to 1792504862, and the snapshot's rename comes before the WAL stamp write. pass.rs:261 writes the stamp only after take returns Ok. tests/snapshot.rs: 11 passed.

### 5. Tests only gain lines and the suite passes
expected: git diff 57c036e under crates/*/tests shows only added lines, and cargo test --workspace --all-features passes.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: `git diff 57c036e HEAD -- 'crates/*/tests/*'`: only tests/pass.rs, +369 and 0 removed lines. tests/ fixtures and goldens are unchanged. `cargo test --workspace --all-features` at HEAD: exit 0, 1014 passed, 0 failed.

### 6. Issue #85 closed by the merged change
expected: Issue #85 on GitHub is closed by the merged PR for this phase. (human-verify: needs the merged PR on GitHub)
criterion: AC6
status: skipped
reported: skip
reason: checked when traders lands

### 7. Once the phase 4 PR (branch traders) is merged, open issue #85 on GitHub (crenshawdev/terminus) and check how it was closed.
expected: #85 is closed, and its timeline shows the merged phase 4 PR closed it (the PR description carries 'Closes #85').
origin: verifier
why_human: An out-of-reach resource, not an unexercised probe: the PR has not been merged, so the closing event does not exist yet. Observing it also needs GitHub, and this verification runs with no network.
status: skipped
reported: skip
reason: duplicate of item 6; checked when traders lands

## Summary

total: 7
passed: 5
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 0
