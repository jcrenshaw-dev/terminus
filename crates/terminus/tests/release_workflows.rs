//! The release workflows build and draft, and the owner signs and publishes
//! from his own machine. These tests keep it that way.

use std::path::Path;

/// Reading a secret, by either syntax, running a signing step, or holding a
/// private key.
const FORBIDDEN: [&str; 4] = ["secrets.", "secrets[", "-Y sign", "PRIVATE KEY"];

/// What one line of a workflow holds that it must not.
fn forbidden_in(line: &str) -> Vec<&'static str> {
    FORBIDDEN.into_iter().filter(|needle| line.contains(needle)).collect()
}

/// CI never holds the release key. The draft is made with the job's
/// `github.token`, and no workflow reads a secret. A workflow that reads one
/// or signs anything means a key has moved off the owner's machine. An empty
/// folder fails too, so a moved one cannot pass by reading nothing.
#[test]
fn workflows_read_no_secret_and_never_sign() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows");
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    let mut workflows = 0;
    let mut found = Vec::new();
    for entry in entries {
        let path = entry.unwrap().path();
        if !path.is_file() {
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "yml") {
            workflows += 1;
        }
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (n, line) in text.lines().enumerate() {
            for needle in forbidden_in(line) {
                found.push(format!("{}:{}: {needle}", path.display(), n + 1));
            }
        }
    }
    assert!(workflows > 0, "{} holds no .yml file", dir.display());
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// A secret under any name, by either syntax, is refused.
#[test]
fn any_secret_line_is_refused() {
    for line in [
        "          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}",
        "          SIGNING_KEY: ${{ secrets.RELEASE_KEY }}",
        "          KEY: ${{ secrets['OTHER'] }}",
    ] {
        assert_eq!(forbidden_in(line).len(), 1, "{line}");
    }
}
