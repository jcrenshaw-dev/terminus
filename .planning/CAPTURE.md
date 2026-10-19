# Capture

## Todos

- [ ] (phase 4) `terminus observations regenerate` runs `entity_values` in `crates/terminus-core/src/observe/mechanical.rs` once per session for `tool` and again for `error`, and SQLite plans that query on `idx_entities_lookup(kind)`, so every session walks all of the store's entities of that kind and looks up each one's turn. On the live store that is 218,835 `tool` entities and about 0.11 s and 574 MB of page reads per session across 4,893 sessions, so one run takes about 10 minutes, and the cost grows with sessions times entities. Measured 2026-10-18 on 0.1.1; this branch has the same query.



## Seeds

## Notes

## Debt markers

- None.
