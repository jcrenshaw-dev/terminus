---
status: testing
phase: 1
fields_version: 1
started: 2026-10-19
updated: 2026-10-19
---

## Items

### 1. An oversized version part is refused at the tag check
expected: Run with v123456789012345678901.0.0 or v18446744073709551616.0.0, the signer exits non-zero, prints exactly `terminus: usage: sh scripts/sign-release.sh v<X.Y.Z>` on stderr under both /bin/sh and dash, and calls neither gh nor ssh-keygen.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: a_bad_tag_is_refused::sh and ::dash pass in the full-suite run with both new tags in the list (sign_release.rs:434-456). Scratch run of the new signer under sh and dash: rc=1, stderr exactly the usage line, no gh or ssh-keygen log. Main's signer reaches gh release view for the same tags, so the test pins the bound.

### 2. The largest u64 part gets past the tag check
expected: Run with v18446744073709551615.0.0, the signer's first gh call is `release view`.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: the_largest_u64_part_passes_the_tag_check::sh and ::dash pass (sign_release.rs:460-467, registered at :584). Scratch run under sh and dash: first gh call is 'release view v18446744073709551615.0.0 --repo jcrenshaw-dev/terminus --json isDraft --jq .isDraft'.

### 3. The signer keeps no copy of valid_version
expected: scripts/sign-release.sh holds no definition of valid_version.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: grep -c -E '^[[:space:]]*valid_version[[:space:]]*\(' scripts/sign-release.sh prints 0. The diff against main deletes the old function, and the function the signer runs is cut from install.sh at :53-66 and evaled at :68.

### 4. A missing or doubled valid_version in install.sh is refused
expected: With an install.sh beside the script holding no valid_version function, or two, the signer exits non-zero with a message naming install.sh and calls neither gh nor ssh-keygen.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: Scratch runs under sh and dash beside an install.sh with the function removed, doubled, cut off unclosed, or doubled in another form: every run exits 1 with one stderr line starting 'terminus: scripts/../install.sh:' ('found 0', 'found 2', or 'needs a valid_version() ( line and a ) line closing it'), and the gh and ssh-keygen stand-ins log nothing (scripts/sign-release.sh:51-52, :67).

### 5. install.sh is unchanged and the suite passes
expected: install.sh is byte for byte the same as on main, and `cargo test --workspace --all-features` passes.
criterion: AC5
status: pass
first_pass: pass
source: verifier
evidence: git diff --quiet main -- install.sh and git diff --quiet main HEAD -- install.sh both exit 0. cargo test --workspace --all-features exits 0 with 992 passed and 0 failed.

### 6. Issue #86 is closed by the merged change
expected: Issue #86 is closed by the merged change. (human-verify: needs the merged PR on GitHub)
criterion: AC6
status: skipped
reported: skip both
reason: no PR yet; #86 closes when traders lands

### 7. After the PR that carries phase 1 merges, check issue #86: gh issue view 86 --repo crenshawdev/terminus --json state,closedByPullRequestsReferences
expected: state is CLOSED, and the closing reference is the merged PR whose body says Closes #86.
origin: verifier
why_human: Out-of-reach resource: the fix exists only on the local traders branch, so there is no PR or merge yet, and this pass is not allowed to reach GitHub. Every code-level truth was exercised here; this one depends on a merge that has not happened.
status: skipped
reported: skip both
reason: no PR yet; #86 closes when traders lands; duplicate of item 6

## Summary

total: 7
passed: 5
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 0
