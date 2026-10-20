---
status: testing
phase: 3
fields_version: 1
started: 2026-10-20
updated: 2026-10-20
---

## Items

### 1. Every read site calls the one rule on Scoped
expected: Search, the brief and inject::count call Scoped::push_filter; get and context call Scoped::admits. excluded_projects() and excluded_pre_worktree() carry no pub. A grep over crates/ finds no push_exclusion and no ProjectKeys::excluded, and get.rs and context.rs hold no exclusion closure.
criterion: AC1
status: pass
first_pass: pass
source: verifier
evidence: push_filter (scope.rs:182) is called at search.rs:348, brief.rs:476 and inject/mod.rs:470, and the prompt line reaches it through search::run (prompt.rs:430). admits (scope.rs:152) is called at get.rs:153 and context.rs:142. excluded_projects (:132) and excluded_pre_worktree (:137) have no pub. Greps for push_exclusion, fn excluded(, .excluded() and ProjectKeys::excluded over crates/ find nothing, and get.rs and context.rs contain no `hidden`. Mutations show the wiring is real: breaking push_filter's pre-worktree clause failed the search and brief tests, and breaking admits' pre-worktree check failed the get/context test.

### 2. Retention uses the visible check
expected: retention::is_excluded is only a call to visible::is_excluded and holds no check of its own.
criterion: AC2
status: pass
first_pass: pass
source: verifier
evidence: The body of retention/mod.rs:236-242 is only `visible::is_excluded(config, row.project.as_deref(), row.project_pre_worktree.as_deref())`. With visible::is_excluded forced to false in a scratch copy, retention.rs a_session_in_an_excluded_project_is_passed_over_and_counted failed (retention.rs:284). It passes at HEAD.

### 3. Three new tests pin the uncovered exclusion paths
expected: The three new tests (brief pre-worktree exclusion, get/context pre-worktree NoSuchTurn, store with no pre-worktree column) pass on the pre-plan code at 3d7bbc6 and on HEAD.
criterion: AC3
status: pass
first_pass: pass
source: verifier
evidence: In a scratch worktree at b74b8bb, which is 3d7bbc6 plus added test lines only (tests identical to HEAD's), predating_store::search_and_the_brief_still_answer_and_hide_an_excluded_project, a_session_excluded_under_its_worktree_key_is_neither_quoted_nor_counted and a_turn_excluded_only_by_its_worktree_key_is_refused_by_get_and_context all passed. All three also pass at HEAD in the full suite. Each one failed under the mutation it is meant to catch: the empty-list skip, the pre-worktree SQL clause, and admits' pre-worktree check.

### 4. No existing test or golden edited, full suite passes
expected: git diff main under crates/*/tests shows only added lines, and cargo test --workspace --all-features passes.
criterion: AC4
status: pass
first_pass: pass
source: verifier
evidence: The phase's test diff from 3d7bbc6 is +196 lines and 0 removed (inject_brief.rs, recall.rs), and the goldens are unchanged. The 32 removed lines that `git diff main` shows all come from main..3d7bbc6, which is phases 1 and 2. PLAN.md Notes moves the check to the start commit for this reason. `cargo test --workspace --all-features` exited 0 with 1004 passed and 0 failed, recall_golden included.

### 5. Issue #80 closed by the merged change
expected: Issue #80 shows closed by the merged PR on GitHub. (human-verify: needs the merged PR on GitHub)
criterion: AC5
status: skipped
reported: skip both, checked when traders lands
reason: checked when traders lands

### 6. Confirm issue #80 is closed by the merged phase 3 PR on GitHub (crenshawdev/terminus)
expected: Issue #80 shows as closed, and the closing reference is the merged PR that carries commits b74b8bb, c12b625, 9dd14cc and 3f0154c
origin: verifier
why_human: The resource is out of reach. The change is still on branch traders and not merged, so the PR that would close #80 does not exist yet. GitHub's issue state cannot be read from the repository, and this verification is barred from the network.
status: skipped
reported: skip both, checked when traders lands
reason: duplicate of item 5; checked when traders lands

## Summary

total: 6
passed: 4
failed: 0
pending: 0
skipped: 2
blocked: 0
reworked: 0
