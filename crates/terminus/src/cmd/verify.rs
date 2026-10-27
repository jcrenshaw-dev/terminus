//! `terminus verify`: check every session's block rows and its recorded checksum.

use serde_json::json;
use terminus_core::{verify, Config};

use super::json::Document;
use super::read::{self, Opened};
use super::Failure;

pub fn run(json: bool) -> Result<(), Failure> {
    // Read-only, and with the default config: the check reads no visible-session
    // filter, so a `terminus.toml` that does not parse must not stop it.
    let reader = match read::open_in(&super::data_dir()?, Config::default())? {
        Opened::Ready(reader) => reader,
        Opened::Nothing(reason) => return read::empty(empty(), &reason, json),
    };
    let report = verify::verify(reader.store())?;
    // Sessions, not failures: one session can fail several checks.
    let failed = report
        .failures
        .iter()
        .map(|f| f.session_key.as_str())
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    if json {
        // The ids and the count in one document. Split across two streams they
        // are the same two facts, and a `--json` caller reading only stdout
        // would have to infer "how many were checked" from "how many failed".
        let mut document = document(&report);
        if !report.is_ok() {
            document = document.failed().because(format!(
                "{failed} of {} session(s) failed verification",
                report.checked
            ));
        }
        document.emit();
    } else {
        // Failing session keys are the data, so they go to stdout. The count is
        // commentary and goes to stderr, which also keeps stdout to the failing
        // keys and nothing else.
        out!("{}", report.render());
        // Not printed in JSON mode, and that is the rule rather than an
        // omission: `--json` means the document IS the answer, so routine
        // commentary moves into it and stderr is left for warnings. Printing
        // both would give a caller two accounts of one walk that can disagree.
        errln!("{} session(s) checked, {failed} failed", report.checked);
    }

    if report.is_ok() {
        Ok(())
    } else {
        // Exit 1 with the document already written. A caller that parses `ok`
        // and a caller that checks the exit code must not disagree about
        // whether the store is whole.
        Err(Failure::Silent)
    }
}

/// The document for a store with nothing checked in it: `checked` 0 and no
/// failures. The no-store answer, and what `main` writes when the command fails
/// before it has a report.
pub(crate) fn empty() -> Document {
    document(&verify::Report::default())
}

/// The `--json` document.
fn document(report: &verify::Report) -> Document {
    Document::new("verify")
        .field("checked", report.checked as i64)
        .field(
            "failures",
            report
                .failures
                .iter()
                .map(|failure| {
                    json!({
                        "session_key": failure.session_key,
                        "block": failure.block,
                        "detail": failure.detail,
                    })
                })
                .collect::<Vec<_>>(),
        )
}
