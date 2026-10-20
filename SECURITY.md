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

- **Every pull request and every push to `main`** runs sentinel's mechanical checks through `.github/workflows/fast.yml`. `cargo audit`, `cargo deny` against this repo's `deny.toml`, `cargo clippy` and `semgrep` are gates: if any of them fails, the run fails. `gitleaks` reads every commit in the history and reports what it finds without failing the run. It prints only each finding's commit, file, rule and line, never the secret. `shellcheck -S warning` reads the tracked shell scripts and `zizmor --offline` reads the tracked workflows, `action.yml` files and `dependabot.yml`, and both report without failing the run. CI installs shellcheck 0.11.0, checked against its SHA-256, and zizmor 1.30.1. The `semgrep` gate also runs any rules in a `.semgrep/` folder, after `semgrep --test` checks each against its test file; Terminus has none yet. On a pull request, a second job, on GitHub only, has Claude Code review the pull request's diff against THREAT-MODEL.md, reading only the repository's files, and posts the review as a comment on the pull request. It runs alongside the checks, so a failing check does not skip it. The job fails if the review did not run. Findings don't fail it; they are posted for the author to read. It reads the `ANTHROPIC_API_KEY` secret, which GitHub does not pass to workflows run from a fork.
- **Every push and pull request** runs the whole test suite through `.github/workflows/test.yml`: `cargo test --workspace --all-features --locked`, on the debug and the release build, on Linux and macOS, on x86_64 and aarch64. If any test fails in any of the eight jobs, the run fails. It reads no secret.
- **The full audit** runs by hand, not in CI. The next section says what it runs.
- **Fuzzing.** `fuzz/` holds a cargo-fuzz target for every parser that reads input from outside Terminus: transcripts, `terminus.toml`, the command line, hook input, MCP messages and Claude Code's settings files. `cargo +nightly fuzz build` builds them all, and `cargo +nightly fuzz run <target>` runs one. Before its first step, the full audit asks how to fuzz: a smoke test of 60 seconds per target, one at a time; a long run of 30 minutes per target, as many at once as the machine's cores and memory allow; or no fuzzing. CI does not run them.
- **Miri.** Terminus has one `unsafe` block, the volatile read of the install marker in `crates/terminus/src/cmd/install/binary.rs`. The two tests that exercise it without touching the filesystem run under Miri:

  ```sh
  rustup +nightly component add miri
  cargo +nightly miri test -p terminus --lib -- a_marker_split_across_two_reads_is_still_found a_file_without_the_marker_is_foreign
  ```

## The full audit

The full audit runs sentinel's audit in five steps, and stops after each one until the maintainer says to go on. Step 1 runs Terminus's code: `cargo clippy`, `cargo geiger` and the fuzz build compile it, which runs its build scripts and proc macros and those of its dependencies, the fuzz run runs its fuzz targets, and Miri runs its unit tests. The Claude Security scan, the lenses and the check only read it.

Before step 1, `.sentinel/threat-model-age.sh` checks that THREAT-MODEL.md still describes the code. When production files it covers changed after the commit it names, the audit stops and asks whether to update the model first or to go on with the gap recorded. If `audit/findings.md` does not exist yet, the audit then starts the findings register from the reports already in `audit/`, such as `claude-security.md`, `claude-security-skipped.md` and the lens reports, before step 2 overwrites any of them.

1. **Mechanical checks.** The checks every pull request runs, plus the fuzz run picked before this step, with their output saved to `audit/check.txt`. When a fuzz target crashes, the output shows the path of the input it saved, never the input itself.
2. **Claude Security scan.** The maintainer picks one of three: a release scan of the whole repository at medium effort, a branch scan of the branch's committed changes at high, or a quick scan of the whole repository at low. A finding the maintainer has marked accepted in the register is shown only as a count. After a whole-repository scan, every production file the report says it did not cover is listed, and a scan of exactly those files is offered at the same effort, with its report saved to `audit/claude-security-skipped.md`.
3. **Lens pass.** Eight review prompts in `.sentinel/lenses/`, one concern each, read the whole repository against THREAT-MODEL.md, on Claude, on Codex, on both, or not at all, and nothing runs until the maintainer picks. Each lens runs once, on a copy of the repository that leaves out `audit/`, the Claude Security reports and symlinks. Claude gets only the Read, Grep and Glob tools. Codex runs under a permission profile that can read only the copy, Codex's own install folder and the system directories its commands need, with no writes, no network and web search off. Each run's line in `run.txt` says `ok`, `FAILED`, `BLOCKED` (the reply ended in a refusal) or `UNPARSEABLE` (no finding and no `No findings` block). A Claude run's line also names the model that answered, says `MODEL DIFFERS` when it is not the one asked for, and says `SAFETY STOP` when a safeguard stopped one reply but the run finished; that report is kept. `coverage.txt` records which files each run read.
4. **Check.** A lens finding is set aside, with its reason, when the Claude Security report already has it, when THREAT-MODEL.md already documents it, when it needs a same-user process that can already write the files, or when it matches a finding the maintainer accepted in the register. Each of the rest goes to read-only subagents as a neutral claim, with no provider, lens or severity attached, and they try to refute it. Only the findings that stand or partly stand go into `audit/lenses.md`, with the severity the check gave them, and the refuted ones are listed with what stops them. A coverage alert is raised only for production files that neither Claude Security nor any lens read.
5. **Findings register and log.** Every finding gets an ID, `terminus-NNN`, in `audit/findings.md`, and keeps it from audit to audit. The audit never marks a finding fixed or accepted; only the maintainer does. Then one row per step goes into the findings log: which mechanical checks failed, and how many findings each scan and the checked lenses reported at each severity, with how many were new, how many were already in the register and how many open ones this audit did not report, or `lenses (skipped)` when they did not run.

The audit finds and records. It fixes no code, writes no tests and commits nothing. What happens to each finding is the maintainer's decision.

## Fixed findings

Every finding that is fixed gets a regression test. The test goes into the test suite of the crate the finding is in, it fails before the fix and passes after, and it stays there so the same bug cannot come back unnoticed.
