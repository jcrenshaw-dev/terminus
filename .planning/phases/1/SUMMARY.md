---
phase: 1
status: complete
completed: 2026-10-19
---

# Phase 1: Signer version check - Summary

`scripts/sign-release.sh` no longer keeps its own copy of `valid_version`; it reads install.sh's function at run time and uses it, so the signer refuses every version the installer refuses (#86).

## What shipped

- The signer takes `valid_version` from `install.sh` at run time and refuses when install.sh holds none, two, or an unclosed definition - `scripts/sign-release.sh`
- Two refused tags in `a_bad_tag_is_refused`: a part past 20 digits and a part above u64 - `crates/terminus/tests/sign_release.rs`
- `the_largest_u64_part_passes_the_tag_check`, under both sh and dash - `crates/terminus/tests/sign_release.rs`

## Commits

| Plan | Task | Commit | Description |
|---|---|---|---|
| 1 | 1 | 6a75972 | Check the version with install.sh's own valid_version, read at run time |
| 1 | 2 | cea9e82 | Refuse a tag whose part runs past 20 digits or above u64 |
| 1 | 3 | e477061 | Prove a part of u64's largest value still gets past the tag check |

## Deviations

None - plans executed as written.

## Open items

None.

## Goal check

The three commits deliver the goal. The signer's own copy is gone and the check is install.sh's function: `scripts/sign-release.sh:51` counts install.sh's definitions, `:52` and `:67` refuse a missing, doubled or unclosed one, and `:69` runs `valid_version "$version"`. The bound the old copy lacked is now tested from both sides: cea9e82 adds the 20-digit and above-u64 tags to the refused list, and e477061 shows a part of exactly u64's largest value still passes. The executor's report records that both new refusals fail against main's signer and both pass on this branch, under sh and dash, with `install.sh` unchanged from main and the full workspace suite green after the last commit.
