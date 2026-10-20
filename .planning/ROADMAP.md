# Roadmap: Terminus

## Overview

The archive is the product, so it is built first and everything else is built on top of it: a store that can hold a session losslessly and prove it, then an ingest that survives a real transcript tree, then a query path that finds a turn from the terminal and from the model. Only once recall works is the thing wired into Claude Code, because hooks and install are worth nothing until there is something to recall. Injection follows, then the feedback loop that measures whether injection helped — deliberately after injection exists, since it labels injection's own decisions. Observations and their egress boundary come next as the one opt-in, one network-touching feature, and lifecycle management lands last because retention is meaningless until there is data worth aging out. Each phase leaves something exercisable: a store you can verify, an ingest you can run, a search you can type, hooks that fire, a brief that appears, stats that prove precision.

## Phases

- [x] **Phase 1: Signer version check** - `sign-release.sh` refuses every version `install.sh` refuses
- [x] **Phase 2: Read-only status and verify** - `status` and `verify` open the store read-only, like every other read command
- [x] **Phase 3: One excluded-project rule** - the rule that keeps an excluded project out of recall lives in one place
- [x] **Phase 4: Steady-state ingest pass** - a pass over unchanged transcripts stops reading their archive state
- [ ] **Phase 5: Brief says when it cuts** - a brief that drops its quoted turns says so, and the tests stop depending on the temp path's length
- [ ] **Phase 6: Simplify the 1.0.0 changes** - a simplify pass over the code the 1.0.0 fixes change, with no change in behavior
- [ ] **Phase 7: README for 1.0.0** - the README describes 1.0.0 and what versions mean from here
- [ ] **Phase 8: Doc and comment sweep** - docs and code comments describe the code as it is, with the versioning rule written down

## Phase Details

### Phase 1: Signer version check
**Goal:** `scripts/sign-release.sh` carries a copy of `install.sh`'s `valid_version` without its 20-digit and u64 bound, so it would sign a manifest the installer refuses (#86). The signer must use the installer's exact check before 1.0.0 is signed.
**Depends on:** Nothing (first phase of this cycle)
**Requirements:** None; delivered against #86
**Success Criteria:**
1. `sign-release.sh` refuses a version part longer than 20 digits or above u64, exactly as `install.sh` does.
2. The check is `install.sh`'s own, taken from it the way the signer already takes `release_key`, or an identical copy with a test that fails if the two differ.
3. A test signs nothing for a version `install.sh` would refuse.
4. Issue #86 is closed by the change.

### Phase 2: Read-only status and verify
**Goal:** `terminus status` and `terminus verify` only read, but open the store with the writable `Store::open`, so they create a store on a fresh machine and change an older store's schema without the ingest lock (#77). They move to `read::open`, the path every other read command uses.
**Depends on:** Nothing
**Requirements:** None; delivered against #77
**Success Criteria:**
1. On a machine with no store, `status` and `verify` report that there is none and create nothing.
2. On a store older than the binary, both say so and leave the schema unchanged.
3. Their output on an ordinary store is unchanged.
4. A regression test for each fails before the change and passes after, and issue #77 is closed by it.

### Phase 3: One excluded-project rule
**Goal:** The rule that keeps an excluded project out of search, context, get, the brief and the prompt line is written at every read site: two copies of `push_exclusion`, three sites that push the filter by hand, two `hidden` closures and a second `is_excluded` in retention (#80). It moves onto `Scoped`, and every site calls it.
**Depends on:** Nothing
**Requirements:** None; delivered against #80
**Success Criteria:**
1. `Scoped` carries one SQL filter and one Rust check, and no read site builds the exclusion itself.
2. Both `push_exclusion` functions and both `hidden` closures are gone, and `retention::is_excluded` calls the `visible` one.
3. Every existing exclusion test passes unchanged.
4. Issue #80 is closed by the change.

### Phase 4: Steady-state ingest pass
**Goal:** The ingest pass runs on every hook event and reads every non-evicted session's block hashes and tail before it knows the file grew, with five lookups per file, a triple parse of new lines, statements compiled per turn, and a second store open to check for a snapshot (#85). Measure a steady-state pass on the live store, then remove the work an unchanged file does not need.
**Depends on:** Nothing
**Requirements:** None; delivered against #85
**Success Criteria:**
1. A measurement of a steady-state pass on the live store, before and after, is recorded in the phase.
2. An unchanged transcript costs the pass no read of its block rows.
3. What is archived and indexed is byte for byte the same: the existing ingest, recover and divergence tests pass unchanged.
4. Issue #85 is closed by the change.

### Phase 5: Brief says when it cuts
**Goal:** With a long project path and a small brief budget, the brief's fixed lines use up the budget and its quoted turns are dropped whole with no mark, and three core tests fail when `TMPDIR` is a long path (#46). The brief must say when it cut, and the tests must not depend on the temp path's length.
**Depends on:** Nothing
**Requirements:** None; delivered against #46
**Success Criteria:**
1. A brief that drops quoted turns for lack of budget says so in the brief.
2. `inject_brief`, `redacted_recall` and `inject_prompt` pass with a `TMPDIR` of 130 characters.
3. A regression test with a long project path fails before the change and passes after.
4. Issue #46 is closed by the change.

### Phase 6: Simplify the 1.0.0 changes
**Goal:** A simplify pass over the code the 1.0.0 fixes change, before the README and the docs sweep describe it (#90). It looks for code that reinvents something the codebase already has, code that can be simpler, and wasted work, and it changes no behavior.
**Depends on:** Phases 1 to 5
**Requirements:** None; delivered against #90
**Success Criteria:**
1. `/simplify` has run over the changes delivered against #86, #77, #80, #85 and #46. Every finding it applied is committed, and every one it skipped is listed with the reason.
2. No behavior changes: the full suite passes unchanged.
3. Code outside those changes is untouched.
4. Issue #90 is closed by the change.

### Phase 7: README for 1.0.0
**Goal:** The README still speaks for 0.1.1 (#87). It describes 1.0.0, what changed that a user sees, and what version numbers mean from 1.0.0 on.
**Depends on:** Phases 1 to 6, so it describes the code that ships
**Requirements:** None; delivered against #87
**Success Criteria:**
1. The status line, install section and command list match 1.0.0.
2. Every claim in the README holds against the code and `docs/`.
3. The README says what a version number means from 1.0.0 on.
4. Issue #87 is closed by the change.

### Phase 8: Doc and comment sweep
**Goal:** Docs and code comments describe the code as it is (#88): the versioning rule in `CHANGELOG.md` and `docs/release.md`, the 1.0 rule gone from `DESIGN-BRIEF.md`, comments that still point at `.planning/phases/`, overlong and wrong comments trimmed to their reason, and `docs/` checked against the code.
**Depends on:** Phases 1 to 6
**Requirements:** None; delivered against #88
**Success Criteria:**
1. `CHANGELOG.md` and `docs/release.md` state the versioning rule, and `DESIGN-BRIEF.md` no longer gates 1.0 on a full feature set.
2. No comment points at `.planning/phases/`.
3. Every comment #88 names is cut to its reason or corrected.
4. No code behavior changes: the full suite passes unchanged.
5. Issue #88 is closed by the change.
