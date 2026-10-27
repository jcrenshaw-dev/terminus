PLAN COMPLETE
Plan: .planning/phases/8/PLAN-5.md
Tasks: 2 of 2
| Task | Commit | Note |
|---|---|---|
| 1 - Show a cut the share cannot hold as the mark alone | 5242337 | `render` marks a turn longer than its share when the share is 3 or less; the mark is counted into the fixed cost (D-17) |
| 2 - Record only the turns the prompt line showed | b200aac | `selected` renders and records the turns shown with text; new test and sweep in inject_prompt.rs; design doc rewritten |
Deviations: [deviation] Task 1's Action says a mark may overspend its share by up to 3 characters and leaves that to `frame`'s backstop; CONTEXT D-17 (the later, locked decision) says `render` counts the mark into the fixed cost so the backstop fires only when ids, dates and marks alone are over the frame's room, because the backstop cuts the last turn line. Followed D-17. Consequence: at a budget that cannot hold the ids plus the marks (a spare of 0 to 2 characters for one turn) the line is cut by the backstop and is not `...`; the budget still holds.
Open items: none
