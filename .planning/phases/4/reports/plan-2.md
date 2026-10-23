PLAN COMPLETE
Plan: .planning/phases/4/PLAN-2.md
Tasks: 6 of 6
| Task | Commit | Note |
|---|---|---|
| 1 - Make core's date check validate the calendar and the clock, and make it public | 0844d70 | `bound` is now `pub fn checked_bound(field, raw)` in `recall::search`, re-exported from `recall`; `FIELD_SINCE`/`FIELD_UNTIL` made `pub` in `recall::search` (not re-exported). Message is now `{field} {value:?} is not a real date or time; expected ...`, starting with the field so front ends prefix their own spelling. recall test 63 passed. |
| 2 - Check the dates before the scope in `search::run` | 843fb5a | `run` checks both bounds once through private `Filters::bounds` before `scope::resolve`; `push_onto` takes the checked `Bounds` and is now infallible. New test `a_bad_date_is_refused_before_any_scope_answer` failed first against the old order with `Ok(UnknownProject)`, as the Verify predicts; 64 passed after. |
| 3 - Check MCP's dates with core's check | f299719 | `optional_time` maps any `checked_bound` error to `Refused::caller(error.to_string())`; dead arm and `terminus_core::Error` import removed. New test `an_impossible_since_is_a_caller_error_with_or_without_a_store`; mcp test 30 passed. |
| 4 - Check `search` and `sessions` dates with core's check | d20ed0c | New CLI helper `cmd::date_value(parser, flag)` in `cmd/mod.rs` (in the plan lease) reads the flag and maps any core error to `Failure::Misuse(format!("--{error}"))`; both parsers use it. Dead arm and `terminus_core::Error` import removed from `cmd/search.rs`. New test `an_impossible_date_is_misuse_with_no_store`; recall_cli test 26 passed. |
| 5 - Check `observations regenerate` with core's check and delete `cmd::time_bound` | 77ad50d | `regenerate_args` uses `date_value`; `time_bound` and its doc deleted. New test `an_impossible_since_is_misuse_with_no_store` in observations; the three greps print nothing; observations 7, recall_cli 26, mcp 30 passed. `observe::regenerate` unchanged. |
| 6 - Say in the docs that a bound must be a real date or time | f104a07 | `### Time bounds`, the `isError` bullet and Figure 1 rewritten; Figure 1 has a date-check node between `Query::parse` and `scope::resolve` ending at `InvalidTimeFilter`, and its caption states the order. |
Suite: `cargo test --workspace --all-features` (workflow.test_command is null; the plan and CI name this one) green, no FAILED, no error line.
Deviations: none
Open items: `crates/terminus/tests/recall_cli.rs` (around line 584, in `an_unindexable_word_changes_no_hit`, from plan 4-1) and `crates/terminus/src/cmd/hook.rs:245` are not rustfmt-clean; left alone under D-13's no-workspace-fmt rule.
