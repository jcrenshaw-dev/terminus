# Task: close-data-move-mkdir-race

## What shipped

- data move now creates a destination both checks found absent with one exclusive 0700 mkdir (create_destination in crates/terminus/src/cmd/data.rs) and refuses the move when anything is already at the path, closing the window between the second check and copy_tree's create_dir_all (#59). THREAT-MODEL.md and docs/design/0004 no longer say the race is open.

## Commits

| Task | Commit | Description |
| --- | --- | --- |
| 1 | a816e1dffdc4519793690b4f7bf0592664573d72 | fix: create a data move destination with an exclusive mkdir so nothing planted after the second check is taken |
| 1 | c741b46f302cd60fefd85f1e658e122cdc2ea8ff | docs: record that data move creates an absent destination itself and refuses one planted after the check |

## Files

### Task 1: close-data-move-mkdir-race

- **Files:** THREAT-MODEL.md, crates/terminus/src/cmd/data.rs, docs/design/0004-archive-and-block-rows.md
