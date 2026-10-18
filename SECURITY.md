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

Terminus is audited with [sentinel](https://github.com/jcrenshaw-dev/sentinel), which this repo carries as a git submodule at `.sentinel`. [THREAT-MODEL.md](THREAT-MODEL.md) lists what Terminus reads, writes and runs, and where untrusted data enters, and the pull request review works from it.

- **Every pull request** runs sentinel's mechanical checks through `.github/workflows/fast.yml`: `cargo audit`, `cargo deny` against this repo's `deny.toml`, `cargo clippy` and `semgrep`. If any of them fails, the pull request fails. When they pass, a second job, on GitHub only, has Claude Code review the pull request's diff against THREAT-MODEL.md, reading only the repository's files, and posts the review as a comment on the pull request. The job fails if the review did not run. Findings don't fail it; they are posted for the author to read. It reads the `ANTHROPIC_API_KEY` secret, which GitHub does not pass to workflows run from a fork.
- **Every push and pull request** runs the whole test suite through `.github/workflows/test.yml`: `cargo test --workspace --all-features --locked`, on the debug and the release build, on Linux and macOS, on x86_64 and aarch64. If any test fails in any of the eight jobs, the run fails. It reads no secret.
- **The full audit** runs by hand, not in CI. The next section says what it runs.
- **Fuzzing.** `fuzz/` holds a cargo-fuzz target for every parser that reads input from outside Terminus: transcripts, `terminus.toml`, the command line, hook input, MCP messages and Claude Code's settings files. `cargo +nightly fuzz build` builds them all, and `cargo +nightly fuzz run <target>` runs one.
- **Miri.** Terminus has one `unsafe` block, the volatile read of the install marker in `crates/terminus/src/cmd/install/binary.rs`. The two tests that exercise it without touching the filesystem run under Miri:

  ```sh
  rustup +nightly component add miri
  cargo +nightly miri test -p terminus --lib -- a_marker_split_across_two_reads_is_still_found a_file_without_the_marker_is_foreign
  ```

## The full audit

The full audit runs sentinel's audit in five steps, and stops after each one until the maintainer says to go on. Nothing in it runs Terminus's code: the scans, the lenses and the check only read it.

1. **Mechanical checks.** The same checks every pull request runs, with their output saved to `audit/check.txt`.
2. **Claude Security scan.** The maintainer picks one of three: a release scan of the whole repository at medium effort, a branch scan of the branch's committed changes at high, or a quick scan of the whole repository at low. After a whole-repository scan, every production file the report says it did not cover is listed, and a scan of exactly those files is offered at the same effort, with its report saved to `audit/claude-security-skipped.md`.
3. **Lens pass.** Eight review prompts in `.sentinel/lenses/`, one concern each, read the whole repository against THREAT-MODEL.md, on Claude, on Codex, on both, or not at all, and nothing runs until the maintainer picks. Each lens runs once, on a copy of the repository that leaves out `audit/`, the Claude Security reports and symlinks. Claude gets only the Read, Grep and Glob tools. Codex runs under a permission profile that can read only the copy, Codex's own install folder and the system directories its commands need, with no writes, no network and web search off.
4. **Check.** A lens finding is set aside, with its reason, when the Claude Security report already has it, when THREAT-MODEL.md already documents it, or when it needs a same-user process that can already write the files. Read-only subagents trace the rest in the code at file:line, and only the confirmed and partly confirmed ones go into `audit/lenses.md`.
5. **Findings log.** One row per step: which mechanical checks failed, and how many findings each scan and the checked lenses reported at each severity, or `lenses (skipped)` when they did not run.

The audit finds and records. It fixes no code, writes no tests and commits nothing. What happens to each finding is the maintainer's decision.

## Fixed findings

Every finding that is fixed gets a regression test. The test goes into the test suite of the crate the finding is in, it fails before the fix and passes after, and it stays there so the same bug cannot come back unnoticed.
