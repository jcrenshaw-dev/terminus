# Requirements: Terminus

**Defined:** 2026-10-22
**Core Value:** The transcript is the record: every session archived losslessly and verifiably, and everything else derived from it and rebuildable.

## Active

Committed scope for mule. Each maps to exactly one roadmap phase and names the issue it comes from.

### Old store code


### Planning tags


### Test gate


### Search


### Ingest pass


### Archive


### Owner-only files and signing


### Output


## Shipped

Delivered and verified. Kept as rows for shipped-scope trace; git history
holds the full requirement text. Archived out of `## Traceability` so a new
milestone's audit starts clean (the audit seam parses only the Traceability
table).

| Requirement | Phase | Status | Milestone |
|-------------|-------|--------|-----------|
| SHED-01 (A maintainer finds no code that upgrades a store layout 1.0.0 can't create, and a 1.0.0 store still opens, ingests and passes `terminus verify` (#105)) | 1 | Complete | mule |
| TAGS-01 (A reader finds no comment citing a decision, requirement or phase tag from the removed planning files, and a test fails if one is added (#106)) | 2 | Complete | mule |
| GATE-01 (The hook test that proves stdout returns while the ingest runs passes on macOS Intel release without depending on the ingest outliving a `ps` poll (#75)) | 3 | Complete | mule |
| GATE-02 (A maintainer can tell from a failed run of either hook timing test whether the hook blocked or the runner stalled (#104)) | 3 | Complete | mule |
| SRCH-01 (User gets the hits for the other words when a search includes a word SQLite's tokenizer can't index (#23)) | 4 | Complete | mule |
| SRCH-02 (User gets an error naming the argument as they spelled it when a `since` or `until` date is not a real calendar date, from the CLI and from MCP (#24)) | 4 | Complete | mule |
| SRCH-03 (User can repeat `search --path` past SQLite's variable limit and still get results, because every list is bound as one parameter (#108)) | 4 | Complete | mule |
| PASS-01 (The tree pass, the backfill and single-file ingest run through one pass driver and write their `runs` row the same way (#31)) | 5 | Complete | mule |
| PASS-02 (User sees no error from `status` or `doctor` for a pass whose only notes come from recovery and that failed no file (#96)) | 5 | Complete | mule |
| ARCV-01 (User sees a session's "continues from" link in `show` and the MCP tools even when it was ingested before its predecessor (#11)) | 6 | Complete | mule |
| ARCV-02 (Retention never deletes a session whose transcript reappeared after it was selected (#43)) | 6 | Complete | mule |
| ARCV-03 (User can run `observations regenerate`, and the ordinary pass can observe a session, with each session reading only its own entities (#70)) | 6 | Complete | mule |
| OWN-01 (Every file and directory Terminus creates, `data move`'s pointer and config directory included, is owner-only from the moment it exists, through one set of `owner_only` helpers; the installed binary and the folders that hold it keep the modes `install.sh` gives them (#73)) | 7 | Complete | mule |
| OWN-02 (Signing a release runs no code read out of `install.sh`, and CI fails when the signer's copies of the shared checks differ from `install.sh` (#99)) | 7 | Complete | mule |
| OUT-02 (User gets a JSON document with `ok: false` and the reason from every command that fails under `--json` (#107)) | 8 | Complete | mule |
| OUT-01 (User sees control characters in archived text escaped in every command's human output, while `--json`, the hook document and `show`'s record stay byte for byte (#72)) | 8 | Complete | mule |
| OUT-03 (User is shown the size of everything `uninstall --purge` deletes, snapshots included, before it asks (#95)) | 8 | Complete | mule |
| OUT-04 (User can tell a quoted turn the prompt line cut was cut, even when its share is under four characters (#94)) | 8 | Complete | mule |

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

Bare headers - `/cad-plan` seeds a row per requirement when its phase is
planned (`references/req-traceability.md`).

---
*Last updated: 2026-10-22 after adopting the repository into Cadence*
