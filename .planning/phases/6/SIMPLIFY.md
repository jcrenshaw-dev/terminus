# Phase 6: Simplify the 1.0.0 changes - adjudicated findings

Reviewed: 2026-10-20, `/simplify` over `git diff main...traders` (planning files excluded, 40 files), run as a report only. No file was changed by the review. Four read-only reviewers (reuse, simplification, efficiency, altitude), deduplicated to 22 findings, each checked against the code on `traders`, then adjudicated by John.

/cad-plan 6 plans from this file. Apply exactly the findings under Apply, change no behavior, and touch no code outside the diff. Every skip below is the reason criterion 1 asks for.

## Apply

1. **One watermark rule** (#85). `crates/terminus-core/src/ingest/mod.rs:407` finds the resume point with its own `rposition` over the tail, a second copy of the rule `parse::scan_from` applies (`crates/terminus-core/src/parse/mod.rs:61-88`). `Scan::consumed` (`parse/mod.rs:44`) has no caller left. Put the search in one `parse` function, have `scan_from` derive `resume_offset` from it and `prepare` call it, and delete `Scan::consumed`.
3. **No stat where it is not read** (#85). `ingest/mod.rs:1173` stats the file before the `!evicted && (grown || whole)` test, so evicted and `whole` sessions pay a stat whose answer changes nothing. Move the stat inside the condition.
5. **`Existing::read` and its row** (#85). `ingest/mod.rs:1116-1163` copies each `Row` field out with its own default and checks `session` in two matches in a row. `Holder::of`'s `archived` argument (`:1007`, called at `:209` and `:1143`) is always true when it is read. All four `Existing::read` callers do the same `rows.get(&session_key)` (`ingest/mod.rs:256`, `backfill.rs:432`, `ingest/mod.rs:1689`, `:1716`). `read_rows` (`:1076-1086`) binds a `key` that is `only`. Give `Row` a `Default` and read through one fallback, build the session in one step, make it `Holder::of(row, path)`, and let `read` do the lookup.
6. **Statement cache size set, not counted** (#85). The `derive.rs:77-79` comment says the cache gain holds while a loop fits the default 16, with hand counts of 13 and 11, and nothing enforces it. Set the capacity explicitly on the connection in `Store::open` and drop the counts from the comment.
7. **Trim the `block_reads` log** (#85). In `crates/terminus-core/src/store/archive.rs`, `block_reads::reset` (`:436`) has no caller, and `record_key` (`:425`, stub `:455`) logs only `archive::rows`, whose one caller is `verify` (`verify.rs:213`), which no test counts. Both branches at `:278` and `:300` start with the same `record`, and `:308-310` tests `tail_no.is_some()` before a `match` on it. Delete `reset` and `record_key`, call `record` once before the branch, and fold the other into the `match`.
8. **`Gate` becomes a `Result`** (#85). `crates/terminus-core/src/ingest/pass.rs:150-218` defines a three-variant `Gate` that is `Result<Option<i64>, String>`, with `"no snapshot was taken: {e}"` written three times and a placeholder `NotDue` on the lock-held path. Return the `Result` and add the prefix once in `run_with`.
9. **No second connection for "no store yet"** (#77). `holds_no_tables` (`crates/terminus-core/src/store/open.rs:424-436`) opens a second read-only connection to count tables, while `open_read_only` already runs `SELECT count(*) FROM sqlite_master` at `open.rs:269` and discards the count. Count tables there and return `StoreNotFound` on 0, then delete `holds_no_tables`, its re-export in `store/mod.rs`, and the `NotAStore` arm in `crates/terminus/src/cmd/read.rs:112-116`. Every caller gets the same result: `read.rs` already maps both to `no_store`, `inject/mod.rs:130` discards the error, and `cmd/install/mod.rs:351` matches only `ArchiveFormatTooOld`.
10. **No-store documents built by the real builder** (#77). `status`'s `nothing()` (`crates/terminus/src/cmd/status.rs:167`) and `verify`'s no-store document (`crates/terminus/src/cmd/verify.rs:16-18`) write their field lists a second time by hand. Build each from the same function as the real document, fed zero and null values, as `stats` and `usage` already do.
11. **One threshold in the brief's guard** (#46). `crates/terminus-core/src/inject/brief.rs:221-227` tests `!frame_fits(..) || budget - frame_cost(..) <= chars(ELISION)` and then computes `frame_cost` again. Compute the room once with `checked_sub(...).filter(...)` and use it from there.
13. **Use `Scoped`'s fields directly** (#80). `excluded_projects()` and `excluded_pre_worktree()` (`crates/terminus-core/src/recall/scope.rs:129-139`) are private, called only inside the same `impl`, and return fields that are already private. Delete them and their doc comment, and read the fields.
16. **One schema snapshot and one aging step for the tests** (#77). `schema()` is identical in `crates/terminus/tests/cli.rs:1049` and `crates/terminus/tests/status.rs:470`, and the store aging is repeated at `status.rs:518` and inline at `cli.rs:1101-1111`. Move both into `terminus_core::testkit`. No assertion changes.

## Skip

2. **Open once instead of stat then open** (#85, `ingest/mod.rs:1173` with `:395`). Phase 4's D-08 chose a stat over an open on purpose, so the pass's counted opens stay one per file. The cost is about five syscalls per unchanged file, a few milliseconds a pass.
4. **One helper for the next turn number** (#85, `ingest/mod.rs:1178` against `recover.rs:536`). `max(turn_seq) + 1` and `count(*)` agree while turns are numbered from 0 with no gap, which phase 4's D-02 guarantees, and recovery is off the hot path, so a shared helper buys nothing.
12. **`try_clip` beside `clip`** (#46, `brief.rs:221-227` and `:400-416`). Phase 5's D-05 kept `clip` unchanged with the fix inside `brief.rs`. The prompt-arm defect this finding turned up is filed as #94, because fixing it changes behavior.
14. **Drop the `retention::is_excluded` wrapper** (#80, `retention/mod.rs:236-242`). Phase 3's D-09 kept it as a thin function on purpose.
15. **`sed` range in place of the signer's loop** (#86, `scripts/sign-release.sh:53-67`). The loop refuses a function with no closing `)` line, and a `sed` range prints to end of file instead. No gain in a signing script.
17. **One no-store assertion helper** (`crates/terminus/tests/cli.rs:989`, `status.rs:409`). Test churn before release, and it would tie the test's field list to the product's, which `testkit.rs:543` warns against.
18. **Testkit helpers in the `ingest/mod.rs` unit test** (`:1642-1730`). Test churn for a few lines.
19. **Public `testkit::session_no`** (`crates/terminus-core/tests/pass.rs:1150`). Test churn for a few lines.
20. **`unquoted_len` in the brief tests** (`crates/terminus-core/tests/inject_brief.rs:717`, `:741-744`, `:911-913`). Test churn for a few lines.
21. **One `refusal()` helper** (`crates/terminus-core/tests/pass.rs:1193`, `:1238`, `:1274`). Test churn for a few lines.
22. **Parameterize `Bench::flood`** (`crates/terminus-core/tests/recall.rs:2294`). Test churn for a few lines.

## Criterion 2

"The full suite passes unchanged" means every test passes. The one test change, finding 16, moves helpers and changes no assertion.

## Criterion 3

"Code outside those changes is untouched" means no code unrelated to the fixes is changed. Findings 1 and 16 edit `parse/mod.rs` and `testkit.rs`, which the diff never changed, because each moves a copy the diff created into the module that owns it.
