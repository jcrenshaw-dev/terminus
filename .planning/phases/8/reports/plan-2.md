PLAN COMPLETE
Plan: .planning/phases/8/PLAN-2.md
Tasks: 4 of 4
| Task | Commit | Note |
|---|---|---|
| 1 - Escape by default in outln!/out!, raw write for documents and hook | 819a3a6 | out.rs gains write, try_write, write_raw, try_write_raw; Document::emit/try_emit and hook emit write raw; escaped_output.rs and a brief.rs hook test added |
| 2 - Route show and doctor through the escaping path, remove per-field escapes | 8d1b8e3 | show lines through try_write, body through try_write_raw; doctor write_out through try_write; escape:: gone from search, show, observations; doctor and show tests added |
| 3 - Escape stderr through errln!, deny print_stderr | 217f2d0 | 33 eprintln! sites in 12 files replaced; write_err drops on a closed stderr; lint bite checked with throwaway eprintln!/println! in status.rs (both fail, named print_stderr/print_stdout, removed); stderr tests added |
| 4 - Say every command's human output escapes | 1b3e6f5 | escape.rs module doc, THREAT-MODEL.md CLI bullet and the three docs/commands.md entries rewritten; planning_tags passes |
Deviations: none
Open items: none
