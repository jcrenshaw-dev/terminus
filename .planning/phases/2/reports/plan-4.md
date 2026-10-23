PLAN COMPLETE
Plan: .planning/phases/2/PLAN-4.md
Tasks: 4 of 4
| Task | Commit | Note |
|---|---|---|
| 1 - Cut the tags from the injection and compaction tests | f809b59 | 63 tagged lines in 5 files; tag checks A and B and the diff check print nothing; cargo check and clippy clean |
| 2 - Cut the tags from the feedback, label and observe tests | 95abae3 | 47 tagged lines in 3 files; tag checks A and B and the diff check print nothing; the obs_01 test name is unchanged; cargo check and clippy clean |
| 3 - Cut the tags from the config, discover, exclusion, project, lineage and retention tests | bd87402 | 71 tagged lines in 6 files; tag checks A and B print nothing; the diff check prints only the four named messages; cargo check and clippy clean |
| 4 - Reword the fixture catalog, fixtures.rs and tests/fixtures/README.md, without tags | c83df71 | 31 + 44 tagged lines; tag checks A and B print nothing; the diff check prints only the two named messages; table rows 19 = 19 at 1c55fec; cargo check and clippy clean |
Deviations: none
Open items: inject_brief.rs above nothing_in_the_injection_path_spawns_a_process_or_reads_observations still says "this module's own doc explains at length why neither appears", but src/inject/brief.rs explains only the git half (no tag on that line, so it was left as it was).
