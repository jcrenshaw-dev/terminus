# Security policy

## Reporting a vulnerability

Report security problems privately through GitHub: on this repository, open the **Security** tab and choose **Report a vulnerability**. Do not open a public issue.

Include what you found, how to reproduce it, and what an attacker could do with it. You will get an answer, and credit in the fix if you want it.

## Scope

Reports about Terminus on the `main` branch and about any published release are in scope. Of most interest:

- `install.sh` installing a binary when the manifest does not verify against the pinned release key or the binary's SHA-256 does not match the manifest, or downloading the binary before the manifest's signature verifies;
- the Terminus binary opening a network connection;
- Terminus reading a project listed under `exclude` in `terminus.toml`, or returning its sessions;
- with `[privacy] redact_recall` on, a secret of a shape the redaction filter is built to replace reaching Claude Code through a hook or an MCP tool;
- an archived session whose stored blocks no longer match their recorded hashes, which `terminus verify` still passes.

## How Terminus is checked

Terminus is audited with [sentinel](https://github.com/jcrenshaw-dev/sentinel), which this repo carries as a git submodule at `.sentinel`, pinned to the version Terminus runs. [THREAT-MODEL.md](THREAT-MODEL.md) lists what Terminus reads, writes and runs, and where untrusted data enters, and every review works from it.

Every pull request and every push to `main` runs sentinel's mechanical checks through `.github/workflows/fast.yml`. The gates among them fail the run; the rest report. On a pull request from this repository, a second job has Claude Code review the diff against THREAT-MODEL.md, reading only the repository's files, and posts the result on the pull request. Findings do not fail that job; a review that did not run does. The kit's `README.md` in `.sentinel` lists every check and says which ones gate.

Every pull request and every push to `main` also runs the whole test suite through `.github/workflows/test.yml`: `cargo test --workspace --all-features --locked`, on the debug and the release build, on Linux and macOS, on x86_64 and aarch64. If any test fails in any of the eight jobs, the run fails. It reads no secret.

`fuzz/` holds a cargo-fuzz target for every parser that reads input from outside Terminus: transcripts, `terminus.toml`, the command line, hook input, MCP messages and Claude Code's settings files. The audit runs them; CI does not.

The full audit runs by hand on the maintainer's machine, never in CI. [SENTINEL.md](SENTINEL.md) at the repo root is the record of the last one: what ran, how each step went, and the open findings as counts by severity. It never names a finding or where it is; an open finding stays private until it is fixed. The release workflow fails any release from a commit the last audit did not cover. The audit finds and records: it fixes nothing, writes no tests and commits nothing.

## Fixed findings

Every finding that is fixed gets a regression test. The test goes into the test suite of the crate the finding is in, it fails before the fix and passes after, and it stays there so the same bug cannot come back unnoticed.
