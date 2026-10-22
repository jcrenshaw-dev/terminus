---
phase: 2
plan: 7
requirements:
  - TAGS-01
files:
  - crates/terminus-core/tests/planning_tags.rs
---

# Phase 2: Planning tags - Plan 7 of 7: the guard test

## Goal

A test fails when a source comment, manifest comment or doc line cites a planning tag, and passes on the cleaned tree. Plans 1 to 6 cleaned it (#106).

## Must be true when done

- `cargo test --workspace --all-features` passes, and its output names the `planning_tags` test binary and its tests (AC3).
- Each of these additions, made one at a time, makes the guard fail with a message naming the file and line: `// see SRCH-02` in a `crates/terminus-core/src` file, `// (D-04)` in a `crates/terminus/tests` file, `-- INJ-07` inside `CREATE_SQL`, `# AC3` in `fuzz/Cargo.toml`, and `ING-08` in `docs/json-shapes.md`. Adding `phase 2` to a line of `docs/json-shapes.md`, or `UTF-16` to a Rust comment, leaves it passing (AC4).
- AC1: `git grep -nE '\b(D|INJ|ING|RCL|STOR|PRIV|RET|FEED|OBS|INST|REL|DOC)-[0-9]{2}\b|\bphase [0-9]+\b|\bAC[0-9]+\b' -- crates tests fuzz Cargo.toml docs` prints only the four `session_title` lines. They are in `tests/fixtures/hooks/session-start.json`, `tests/fixtures/hooks/user-prompt-submit.json`, `crates/terminus/tests/owner_only.rs` and `crates/terminus/tests/prompt.rs`.
- AC2: over the same roots, `git grep -nE '\bPhase [0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion'` and `git grep -ni hardin` print nothing.
- AC5: `git diff -U0 1c55fec HEAD -- '*.rs' ':!crates/terminus-core/tests/planning_tags.rs' | grep -E '^[+-]' | grep -vE '^(\+\+\+ b/|--- a/)' | grep -vE '^[+-][[:space:]]*(//|--)'` prints only lines of the 23 `assert!`/`panic!` messages D-06 names. The same pipeline over `'*.toml'` with `#` in place of `(//|--)` prints nothing. Outside `.planning/`, `git diff --name-only 1c55fec HEAD -- . ':!.planning' ':!*.rs' ':!*.toml'` lists only `.md` files.

## Context

Locked by `.planning/phases/2/CONTEXT.md`: D-01 (what fails, and where), D-13 (file placement, no manifest edit), D-14 (named roots and a file-count floor), D-15 (comment text only in Rust and TOML, every line in markdown), D-16 (hand-written matching, no new dependency) and D-17 (no literal tag in the test's own source; a self-proof).
- Patterns to follow: `crates/terminus-core/tests/no_network.rs` (an ungated source scan with a per-root file floor), and `crates/terminus/tests/release_workflows.rs` (reaching the repository root through `CARGO_MANIFEST_DIR` and `../..`).
- Out of scope: reading `assert!`/`panic!` message strings (D-15 reads no string literal), TOML comments after a value (D-15 names `#` lines), SQL comments in strings other than `CREATE_SQL`, any file this plan does not create, and any `Cargo.toml` edit.

## Tasks

### Task 1: Write the tag matcher and prove it fires and holds its boundaries

- **Files:** crates/terminus-core/tests/planning_tags.rs
- **Action:** Create the guard file beside `no_network.rs`. Its module doc says what it keeps out and why, in the style of `no_network.rs`. It is not gated on `testkit` and gets no `[[test]]` entry; `crates/terminus-core/Cargo.toml` sets no `autotests = false`, so cargo discovers it (D-13). This task writes the matcher, which takes one piece of comment text and whether it came from code (Rust, TOML or `CREATE_SQL`) or markdown, and reports every tag in it (D-01). Match by hand with word-boundary checks, where a boundary is the start or end of the text or a character that is not ASCII alphanumeric or `_`. Add no `regex` or other crate (D-16). The families:
  - **Generic:** a run of 2 to 6 ASCII capitals, a hyphen and exactly two ASCII digits, bounded on both sides. So a seven-capital run, three digits (the `SHA-256` shape) and one digit (`UTF-8`) do not match. This rule catches this milestone's ids and any later milestone's with no edit.
  - The allow-list skips `UTF-16` and nothing else. The plan-time sweep found no real name of the generic shape in the scanned roots: `ISO-8601` has four digits, `SHA-256` three and `UTF-8` one. AC4 names `UTF-16`.
  - `D`, a hyphen and exactly two digits, bounded.
  - `AC` and one or more digits, bounded. So `0xAC12`, `MAC1` and the base64 run in `docs/release.md`'s SSH key do not match.
  - `PLAN`, a hyphen and one or more digits, bounded.
  - "phase", in any case, then one space and one or more digits, bounded. It fires in code comments only; markdown allows it as prose.

  Keep the families as bare prefixes and check the digits in code (D-17). The file sits inside a scanned root and inside AC1's and AC2's grep roots, so its source holds no literal tag anywhere, comments and string literals alike. Assemble every test input that carries a tag at run time from a prefix and digits. That covers "phase" followed by a number, and it means the old milestone name never appears.

  Self-tests (D-17):
  - Each family fires in code kind, including a generic id the test never names elsewhere, assembled from a prefix like `SRCH`.
  - "phase N" fires in code kind in lower, title and upper case, and does not fire in markdown kind.
  - The other families fire in markdown kind.
  - `0xAC12`, `MAC1`, a seven-capital run, a three-digit and a one-digit suffix, `ISO-8601` and `UTF-16` do not fire.
- **Verify:** `cargo test -p terminus-core --test planning_tags` passes without `--features testkit` and lists the matcher self-tests. These print nothing over `crates/terminus-core/tests/planning_tags.rs`: `git grep -nE '\b(D|INJ|ING|RCL|STOR|PRIV|RET|FEED|OBS|INST|REL|DOC)-[0-9]{2}\b|\bphase [0-9]+\b|\bAC[0-9]+\b|\bPhase [0-9]+\b|\bPLAN-[0-9]+\b|ROADMAP criterion'`, `git grep -ni hardin`, and `git grep -niE '\bphase [0-9]+\b'`. `git diff --stat 1c55fec -- crates/terminus-core/Cargo.toml` prints nothing.

### Task 2: Extract comment text per file kind, without reading string literals

- **Files:** crates/terminus-core/tests/planning_tags.rs
- **Action:** Add the extraction that turns one file's text into pieces of comment text, each with its 1-based line number and its kind for the matcher (D-15).
  - **Rust:** take the text of whole-line `//`, `///` and `//!` comments, and of a `//` comment that follows code on the same line. Never take text inside a string literal or an identifier. The scan must handle the shapes this tree holds:
    - `"://"` inside an ordinary string on a code line (23 such lines, such as `inject/prompt.rs`'s URL check);
    - char and byte literals of a double quote, `'"'` and `b'"'` (9 files, such as `index/entity.rs`'s `QUOTES`);
    - escaped quotes;
    - raw strings `r#"..."#` (28 files).
    
    A scan that flips state on every `"` misreads these: it reads a string as a comment, or it misses a trailing comment.
  - **CREATE_SQL:** also take the SQL `--` comment lines inside `CREATE_SQL`, starting at its `pub const CREATE_SQL` declaration and running to the line that closes that string. Key this on the declaration, not on the path of `store/schema.rs`, so the self-test can feed a synthetic constant. Take no `--` line from any other string, such as the query in `feedback/label.rs`.
  - **TOML:** take lines whose first non-blank character is `#`.
  - **Markdown:** take every line.

  The self-tests use synthetic sources whose tags are assembled at run time:
  - A whole-line comment and a trailing comment after code are both read.
  - A tag inside a string literal on a code line is not read.
  - A trailing comment after a `'"'` char literal is still read.
  - A `//` inside a raw string is not read.
  - A `--` line inside a synthetic `CREATE_SQL` is read, and one inside another string constant is not.
  - A TOML `#` line is read, and a TOML key line is not.
- **Verify:** `cargo test -p terminus-core --test planning_tags` passes and lists the extraction self-tests beside the matcher's. The three `git grep` checks from Task 1's Verify still print nothing over the file.

### Task 3: Scan the named roots, enforce the file floor and report every finding by file and line

- **Files:** crates/terminus-core/tests/planning_tags.rs
- **Action:** Wire the extraction and the matcher into one scan over the named roots, from the repository root reached as `CARGO_MANIFEST_DIR` joined with `../..` (D-14):
  - Rust `.rs` files under `crates/terminus-core/src`, `crates/terminus-core/tests`, `crates/terminus/src`, `crates/terminus/tests`, `fuzz/fuzz_targets` and `fuzz/src`, walked recursively.
  - Markdown `.md` files under `docs`, walked recursively.
  - The TOML files `Cargo.toml`, `crates/terminus-core/Cargo.toml`, `crates/terminus/Cargo.toml` and `fuzz/Cargo.toml`.
  - The markdown file `tests/fixtures/README.md`.

  Skip files of other extensions in the walked directories, such as the `.txt` golden under core's tests. Never walk `fuzz/` whole: locally it holds an ignored `fuzz/target` of about 705 MB and `fuzz/corpus` of about 89,200 files.

  Give each walked directory a file-count floor, as `no_network.rs` does, so a moved or emptied directory fails instead of passing on nothing. The plan-time counts are 53, 36, 35, 27, 13 and 1 `.rs` files, and 15 `.md` files under `docs`. Set floors the current tree clears and an empty directory does not.

  Collect every finding before failing, and report all of them in one failure message, each as its path relative to the repository root, its line number and the tag. That way one run names every site.
- **Verify:** `cargo test -p terminus-core --test planning_tags` passes on the tree. Then make each of the following edits one at a time, run that command, and confirm it fails with a message naming the file and line of the added text:
  - `// see SRCH-02` appended as a new line in `crates/terminus-core/src/lib.rs`;
  - `// (D-04)` in `crates/terminus/tests/cli.rs`;
  - `-- INJ-07` as a new line inside `CREATE_SQL` in `crates/terminus-core/src/store/schema.rs`;
  - `# AC3` in `fuzz/Cargo.toml`;
  - `ING-08` in `docs/json-shapes.md`.

  Then confirm it still passes with `phase 2` added to a line of `docs/json-shapes.md`, and with `// UTF-16` in a Rust source file. Undo each edit before making the next. These edits are temporary probes outside the lease: `git status --short -- crates docs fuzz/Cargo.toml` must show only `crates/terminus-core/tests/planning_tags.rs` before commit. Last, `cargo test --workspace --all-features 2>&1 | grep planning_tags` shows the binary running, and the whole run passes.

## Notes

- This plan builds on plans 1 to 6: the scan passes only once every tag they own is gone. Run it after them. The phase therefore takes the sequential path, though plans 1 to 6 are file-disjoint.
- A guard that passes on the tree before plans 1 to 6 land, or a seam that only fails in this plan, means a cleanup plan missed a site. Report it rather than editing outside this plan's one file.
- CONTEXT's flagged assumption left open whether the guard should also read `assert!`/`panic!` messages. This plan follows D-15 as written and does not. A tag written into a test message after this phase will pass the guard, although AC1's grep would still find it.
- Plan 7 Task 1 sets the allow-list to `UTF-16` alone, from the plan-time sweep. That sweep looked for every `[A-Z]{2,6}-[0-9]{2}` token in the scanned roots and found only #106's families.
