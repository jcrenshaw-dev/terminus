# Phase 3: One excluded-project rule - Context

Gathered: 2026-10-19
Feeds: /cad-plan 3

## Scope boundary

In: `Scoped` in `crates/terminus-core/src/recall/scope.rs` gains `push_filter` for the SQL side and `admits` for the Rust side (#80). Search, the brief, `inject::count`, get and context call them, and both `push_exclusion` functions, both `hidden` closures and the unused `ProjectKeys::excluded()` go. `retention::is_excluded` calls `visible::is_excluded`. Three new tests pin the exclusion paths no test covers today. No behavior changes.
Out: The scope-only checks in `crates/terminus/src/cmd/sessions.rs` and `observations.rs` (D-11). The per-key `excludes` closure in `visible::projects`, which builds the lists `Scoped` holds. `covers` and `normalize` in `config.rs`, which hold the segment-boundary rule. Any change to what a read command shows.
Deferred: None.
Plan shape: one plan.

## Durable decisions

None this phase.

## Decisions

- D-01 (SQL filter covers scope): `Scoped::push_filter` writes the `AND m.project = ?` scope clause when `project()` is set, then the `m.project` exclusion, then the `m.project_pre_worktree` exclusion, the order all three sites use today. Each site calls it where its hand-written block sits now: in search after the `MATCH ?` param and before `filters.push_onto`, `before_turn_id` and the pool `LIMIT`; in the brief after `parent_session_key IS NULL` and before `ORDER BY`. Writing only the exclusions was rejected, because the scope clause would stay hand-written at three sites. Evidence: `crates/terminus-core/src/recall/search.rs:345-376`, `crates/terminus-core/src/inject/brief.rs:471-493`, `crates/terminus-core/src/inject/mod.rs:495-517`.
- D-02 (Rust check covers scope): `Scoped::admits(project, pre_worktree)` covers the scope check (`project != Some(wanted)`) and both exclusion checks, and returns yes or no. Get and context keep answering `Reason::NoSuchTurn` on a no, which they already do for both a scope miss and an exclusion, so an excluded turn's existence and path stay unconfirmed. Covering exclusion only was rejected, because the scope check would stay hand-written in get and context. Evidence: `crates/terminus-core/src/recall/get.rs:147-176`, `crates/terminus-core/src/recall/context.rs:124-154`, `crates/terminus/tests/mcp.rs:1674`; v0.1.0 phase 3 SUMMARY.
- D-03 (Dead copy): `ProjectKeys::excluded()` is deleted. Nothing in the workspace calls it, and it is a third copy of the two-key check. Evidence: `crates/terminus-core/src/config.rs:1335-1338`.
- D-04 (New tests first): Before the refactor, one test is added for each exclusion path no test covers: the brief's last-session row and its session count with a session excluded under its pre-worktree key; get and context with a turn excluded only by its pre-worktree key; and a store with no pre-worktree column and an exclusion configured. Each passes on main's code and after the change. Evidence: `crates/terminus-core/tests/inject_brief.rs:180`, `:215`; `crates/terminus-core/tests/recall.rs:1411-1419`, `:1904-1911`, `:1690-1693`.
- D-05 (Clause text kept): The clause text ` AND ({column} IS NULL OR {column} NOT IN (?,…))`, its null guard and outer parentheses, and the param order (one owned `String` per value, in list order) stay exactly as they are. The two `push_exclusion` copies differ only by a trailing newline, which has no effect because every clause pushed after it starts with whitespace. Evidence: `crates/terminus-core/src/inject/mod.rs:157-184`, `crates/terminus-core/src/recall/search.rs:701-722`, `:309`; `crates/terminus-core/src/inject/brief.rs:493`.
- D-06 (Per-list skip): The skip for an empty list stays per list. It is what keeps `m.project_pre_worktree` out of the SQL on a store older than that column, where `excluded_pre_worktree` is always empty. Evidence: `crates/terminus-core/src/config.rs:1198-1210`, `:1318-1330`; `crates/terminus-core/src/recall/scope.rs:176-182`; `crates/terminus-core/tests/recall.rs:1644`.
- D-07 (Alias and comments): `push_filter` uses the alias `m`, which every site already gives `session_meta`. The three comments that explain the rule (the null-guard doc, the search call-site comment and the "`NOT IN ()` is not SQL" note) move onto it. Evidence: `crates/terminus-core/src/recall/search.rs:284-296`, `:351-354`, `:701-703`; `crates/terminus-core/src/inject/mod.rs:157-164`; `crates/terminus-core/src/inject/brief.rs:472`, `:642-647`.
- D-08 (Exact match): `admits` keeps today's exact-string match against `Scoped`'s stored lists and does not call `excludes_path`. `Scoped` does not carry the `Config`, so its derives, `Scoped::nothing` and every `scope::resolve` caller stay as they are. Evidence: `crates/terminus-core/src/recall/get.rs:163-176`, `crates/terminus-core/src/recall/context.rs:147-154`, `crates/terminus-core/src/recall/scope.rs:113-120`, `:166-182`.
- D-09 (Config side): `visible::is_excluded` takes the two keys instead of a `Session`. `retention::is_excluded` stays as a thin function that passes `Candidate`'s two fields to it. Evidence: `crates/terminus-core/src/config.rs:1270`, `:1366-1373`; `crates/terminus-core/src/retention/mod.rs:116-123`, `:171`, `:229-239`.
- D-10 (Both forms stay): The list form (`Scoped::admits`) and the path form (`visible::is_excluded`) both stay. They agree on every key `session_meta` holds when `resolve` runs. Merging them would change how excluding `/` treats sessions with no project, and `a_session_with_a_null_project_stays_visible` would fail. Evidence: `crates/terminus-core/src/recall/scope.rs:162-164`, `crates/terminus-core/tests/exclusion.rs:198-226`, `crates/terminus-core/src/config.rs:819-823`.
- D-11 (Other sites): No other read site builds the exclusion. The scope-only checks in `sessions.rs` and `observations.rs` stay as they are: those commands get exclusion from `visible::sessions`, and #80 does not list them. Evidence: `crates/terminus/src/cmd/sessions.rs:69-76`, `crates/terminus/src/cmd/observations.rs:105-110`, `crates/terminus/src/cmd/show.rs:61-71`, `crates/terminus/src/cmd/mcp/tools.rs:363`, `:451`, `:545`, `crates/terminus-core/src/inject/prompt.rs:430`.
- D-12 (Private lists): `Scoped::excluded_projects()` and `excluded_pre_worktree()` become private to `scope.rs`, so a new read site cannot rebuild the filter from them. Evidence: `crates/terminus-core/src/recall/scope.rs:129-137`.

## Acceptance criteria

- [ ] AC1: Search, the brief and `inject::count` call `Scoped::push_filter`, and get and context call `Scoped::admits`. `excluded_projects()` and `excluded_pre_worktree()` are private to `recall/scope.rs`. `push_exclusion` and `ProjectKeys::excluded` are defined nowhere in the workspace, and `get.rs` and `context.rs` hold no exclusion closure.
- [ ] AC2: `retention::is_excluded` calls `visible::is_excluded` and holds no check of its own.
- [ ] AC3: One new test for each of these three paths passes against main's code and after the change: the brief neither quotes nor counts a session excluded under its pre-worktree key; get and context refuse a turn excluded only by its pre-worktree key with `NoSuchTurn`; on a store with no pre-worktree column and an exclusion configured, search and the brief still answer and hide the excluded project.
- [ ] AC4: No existing test and no golden file is edited: `git diff main` under `crates/*/tests` shows only added lines. `cargo test --workspace --all-features` passes.
- [ ] AC5: Issue #80 is closed by the merged change. (human-verify: needs the merged PR on GitHub)

## Flagged assumptions

- `inject::count` is the one site where the scope clause opens the `WHERE` (`" WHERE m.project = ?"` or `" WHERE 1 = 1"`), so it starts with a `WHERE` of its own and `push_filter` only ever writes `AND` clauses. How (a `WHERE 1 = 1`, or heads that carry the `WHERE`) is the planner's call - Likely; if wrong: an `AND` straight after a head with no `WHERE` is a syntax error that `index_pointer` swallows with `.ok()?`, so the brief silently loses its session-count line, and only the golden `recall-projections.txt` catches it.
- Every site's project clause differs from the others only in whitespace (` AND m.project = ?\n`, ` AND m.project = ?`), so writing it once changes no query result - Likely; if wrong: the golden `recall-projections.txt` text changes and AC4 fails.
