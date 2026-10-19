PLAN COMPLETE
Plan: .planning/phases/1/PLAN.md
Tasks: 3 of 3
| Task | Commit | Note |
|---|---|---|
| 1 - Take valid_version from install.sh at run time | 6a75972 | Own copy deleted; definitions counted with grep -c -E (|| :), function cut with a while IFS= read -r loop from `valid_version() (` to the first `)` line and eval'd, then `valid_version "$version" \|\| fail "$usage"` after the release_key block. grep count 0, sh -n and dash -n 0, sign_release 15/15, install.sh unchanged from main. Six hand runs (none, two, unclosed; sh and dash) each exit 1 with one stderr line naming install.sh, and no gh or ssh-keygen call. clippy and cargo check clean. |
| 2 - Refuse a version part over 20 digits or above u64 | cea9e82 | Two tags added to `a_bad_tag_is_refused`; the list is now one tag per line, as rustfmt wraps it, and the rest of the file is untouched. ::sh and ::dash pass. With main's signer swapped in, both fail on `v123456789012345678901.0.0`, with stderr the manifest-version refusal in place of the usage line. The restored signer greps 0 definitions. |
| 3 - Show the largest legal part still gets past the tag check | e477061 | `the_largest_u64_part_passes_the_tag_check`, registered in `both_shells!`; stand-in gh left at its default draft answer. sign_release 17/17. With the tag set to `v18446744073709551616.0.0` both ::sh and ::dash fail with "no gh call"; tag restored. install.sh unchanged from main; clippy and cargo check clean. Full suite `cargo test --workspace --all-features` (workflow.test_command is null; the plan names this command) green, run once after the last commit. |
Deviations: none
Open items: none
