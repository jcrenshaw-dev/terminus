# Requirements: Terminus

**Defined:** 2026-10-22
**Core Value:** The transcript is the record: every session archived losslessly and verifiably, and everything else derived from it and rebuildable.

## Active

Committed scope for mule. Each maps to exactly one roadmap phase and names the issue it comes from.

### Old store code

- **SHED-01**: A maintainer finds no code that upgrades a store layout 1.0.0 can't create, and a 1.0.0 store still opens, ingests and passes `terminus verify` (#105)

### Planning tags

- **TAGS-01**: A reader finds no comment citing a decision, requirement or phase tag from the removed planning files, and a test fails if one is added (#106)

### Test gate

- **GATE-01**: The hook test that proves stdout returns while the ingest runs passes on macOS Intel release without depending on the ingest outliving a `ps` poll (#75)
- **GATE-02**: A maintainer can tell from a failed run of either hook timing test whether the hook blocked or the runner stalled (#104)

### Search

- **SRCH-01**: User gets the hits for the other words when a search includes a word SQLite's tokenizer can't index (#23)
- **SRCH-02**: User gets an error naming the argument as they spelled it when a `since` or `until` date is not a real calendar date, from the CLI and from MCP (#24)
- **SRCH-03**: User can repeat `search --path` past SQLite's variable limit and still get results, because every list is bound as one parameter (#108)

### Ingest pass

- **PASS-01**: The tree pass, the backfill and single-file ingest run through one pass driver and write their `runs` row the same way (#31)
- **PASS-02**: User sees no error from `status` or `doctor` for a pass whose only notes come from recovery and that failed no file (#96)

### Archive

- **ARCV-01**: User sees a session's "continues from" link in `show` and the MCP tools even when it was ingested before its predecessor (#11)
- **ARCV-02**: Retention never deletes a session whose transcript reappeared after it was selected (#43)
- **ARCV-03**: User can run `observations regenerate`, and the ordinary pass can observe a session, with each session reading only its own entities (#70)

### Owner-only files and signing

- **OWN-01**: Every file and directory Terminus creates, `data move`'s pointer and config directory included, is owner-only from the moment it exists, through one set of `owner_only` helpers (#73)
- **OWN-02**: Signing a release runs no code read out of `install.sh`, and CI fails when the signer's copies of the shared checks differ from `install.sh` (#99)

### Output

- **OUT-01**: User sees control characters in archived text escaped in every command's human output, while `--json`, the hook document and `show`'s record stay byte for byte (#72)
- **OUT-02**: User gets a JSON document with `ok: false` and the reason from every command that fails under `--json` (#107)
- **OUT-03**: User is shown the size of everything `uninstall --purge` deletes, snapshots included, before it asks (#95)
- **OUT-04**: User can tell a quoted turn the prompt line cut was cut, even when its share is under four characters (#94)

## v2 Requirements

Deferred. Tracked as open issues, not in this roadmap: #109 (scope enforced three ways), #110 (measure per-pass and per-call cost), #111 (observations as a derived table), #112 (hook busy timeout and prompt helpers), #113 (double binary placement), and every other open issue not named above.

## Out of Scope

| Feature | Reason |
|---------|--------|
| A conversion tool for older stores | The format gate refuses them, and a converter comes only if one is needed |
| Re-keying stores with absolute session keys | Only one store ever had them, and it was re-keyed (#96); #105 removes the code |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SHED-01 | Phase 1 | Complete |
| TAGS-01 | Phase 2 | Complete |
| GATE-01 | Phase 3 | Pending |
| GATE-02 | Phase 3 | Pending |
| SRCH-01 | Phase 4 | Complete |
| SRCH-02 | Phase 4 | Complete |
| SRCH-03 | Phase 4 | Complete |
| PASS-01 | Phase 5 | Complete |
| PASS-02 | Phase 5 | Complete |
| ARCV-01 | Phase 6 | Complete |
| ARCV-02 | Phase 6 | Complete |
| ARCV-03 | Phase 6 | Complete |

Bare headers - `/cad-plan` seeds a row per requirement when its phase is
planned (`references/req-traceability.md`).

---
*Last updated: 2026-10-22 after adopting the repository into Cadence*
