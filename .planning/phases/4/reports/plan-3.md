PLAN COMPLETE
Plan: .planning/phases/4/PLAN-3.md
Tasks: 5 of 5
| Task | Commit | Note |
|---|---|---|
| 1 - Bind the path filter as one JSON parameter | ee73aa4 | New test failed first with "too many SQL variables", passes after; recall 65 passed |
| 2 - Bind every turn-id list as one JSON parameter | 9713b51 | weight_by_entities, rows_for, coordinates on json_each; MAX_IDS, run_get and test docs cite decompression cost; named tests green, no warnings |
| 3 - Bind the two exclusion lists as one JSON parameter each | 62f4541 | push_filter on json_each with IS NULL guard kept; new CLI test a_session_with_no_project_survives_an_exclusion fails with the guard removed, passes restored; recall_cli 27, inject_brief/recall/exclusion green |
| 4 - Drop recall_search's paths cap | 641c183 | MAX_PATHS, maxItems, cut and cap reason gone; test renamed a_path_filter_past_the_variable_limit_is_filtered_on_in_full, sends 40,000 paths, expects hits, isError false, reason null; mcp 30 passed |
| 5 - Drop the paths cap from the docs | 938ae29 | commands.md, json-shapes.md, 0006 rewritten; MAX_PATHS grep empty; "first 25" left only on recall_get / turn_ids lines |
| Suite | - | cargo test --workspace --all-features (workflow.test_command null; the command CI and the plan name): exit 0, 1028 passed, 0 failed |
Deviations: none
Open items: none
