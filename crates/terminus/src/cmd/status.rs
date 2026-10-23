//! `terminus status`: what the store holds and what the last run did.
//!
//! Every number comes out of the store. There is no log file, by design, so
//! `runs` is where a pass's failures live and this is what surfaces them.
//!
//! It takes no ingest lock. Reading while a pass runs must work, which is what
//! WAL is on for, and a status command that blocked behind an ingest would be
//! useless exactly when it is wanted.
//!
//! `rusqlite` is deliberately not named here: the binary crate does not depend
//! on it, and a read command is not a reason to put a second SQL dependency in
//! the hook path's build. The connection comes from `Store::conn` and every
//! query's error is mapped by [`read`].
//!
//! The session, turn and watermark numbers come from `config::visible`, not
//! from `count(*)`: an excluded project must be invisible on read as well as
//! never read on ingest, including for sessions archived before the exclusion
//! was configured. `search`, `show` and `sessions` read through the same
//! boundary.

use std::fmt::Display;
use std::path::Path;

use serde_json::json;
use serde_json::Value;
use terminus_core::config::visible;
use terminus_core::ingest::runs::{self, Run};
use terminus_core::store::DB_FILE_NAME;

use super::json::Document;
use super::read::Opened;
use super::Failure;

/// A store or filesystem error is operational (exit 1), never misuse.
fn read<T, E: Display>(result: std::result::Result<T, E>) -> Result<T, Failure> {
    result.map_err(|e| Failure::Operational(e.to_string()))
}

pub fn run(json: bool) -> Result<(), Failure> {
    // Read-only, through the path every read command uses: with no store this
    // says so and creates nothing, and an older store is reported, not changed.
    let reader = match super::read::open()? {
        Opened::Ready(reader) => reader,
        Opened::Nothing(reason) => {
            let document = document(None, 0, visible::Counts::default(), None, None);
            return super::read::empty(document, &reason, json);
        }
    };
    let (store, config) = (reader.store(), reader.config());
    let data_dir = store.data_dir();
    let conn = store.conn();

    let counts = read(visible::counts(conn, config))?;
    // An optional row from the one core reader: an empty table is `None` and a
    // failed read is an error, so "no run yet" and "the query failed" stay two
    // different answers.
    let last: Option<Run> = read(runs::last(conn))?;

    if json {
        // The same numbers the human output prints, and no others: two accounts
        // of one store that could disagree is the failure a shared shape exists
        // to prevent.
        document(
            Some(store.path().display().to_string()),
            size_bytes(data_dir),
            counts,
            Some(config.exclusions()),
            last.as_ref(),
        )
        .emit();
        return Ok(());
    }

    outln!("store          {}", store.path().display());
    outln!("size           {}", size(data_dir));
    outln!("sessions       {}", counts.sessions);
    outln!("turns          {}", counts.turns);
    outln!(
        "watermarks     {} covering {} byte(s)",
        counts.watermarks,
        counts.watermark_bytes
    );
    if !config.exclusions().is_empty() {
        // Named, because a count that dropped without explanation is a bug
        // report. The excluded sessions are still archived; they are not listed.
        outln!(
            "excluded       {} project(s): {}",
            config.exclusions().len(),
            config.exclusions().join(", ")
        );
    }

    let Some(run) = last else {
        // An empty store is not a failure: it is what a machine looks like
        // before the first hook has ever fired.
        outln!("last run       none");
        return Ok(());
    };

    outln!("last run       {}", run.started_at);
    outln!("  duration     {} ms", run.duration_ms);
    outln!(
        "  files        {} walked, {} committed, {} failed",
        run.files_seen,
        run.files_committed,
        run.files_failed
    );
    outln!("  bytes read   {}", run.bytes_read);
    outln!("  turns added  {}", run.turns_added);
    // What the pass did, then what went wrong, each in full: these are the only
    // account of that pass there is. Apart, so a routine line never reads as a
    // failure.
    if let Some(notes) = &run.notes {
        outln!("  notes");
        for line in notes.lines() {
            outln!("    {line}");
        }
    }
    if let Some(error) = &run.error {
        outln!("  error");
        for line in error.lines() {
            outln!("    {line}");
        }
    }

    Ok(())
}

/// The `--json` document. With no store it is called with no path, no
/// exclusion list and no run, so every field the shape carries is there at zero
/// or null.
fn document(
    store: Option<String>,
    bytes: u64,
    counts: visible::Counts,
    excluded: Option<&[String]>,
    last: Option<&Run>,
) -> Document {
    Document::new("status")
        .field("store", store)
        .field("size_bytes", bytes)
        .field("sessions", counts.sessions)
        .field("turns", counts.turns)
        .field("watermarks", counts.watermarks)
        .field("watermark_bytes", counts.watermark_bytes)
        .field("excluded", excluded)
        .field(
            "last_run",
            match last {
                None => Value::Null,
                Some(run) => json!({
                    "started_at": run.started_at,
                    "duration_ms": run.duration_ms,
                    "files_seen": run.files_seen,
                    "files_committed": run.files_committed,
                    "files_failed": run.files_failed,
                    "bytes_read": run.bytes_read,
                    "turns_added": run.turns_added,
                    "error": run.error,
                    "notes": run.notes,
                }),
            },
        )
}

/// The store's footprint: the database and its WAL sidecars together, because
/// a hot WAL can hold a large share of what has been written.
fn size(data_dir: &Path) -> String {
    let total = size_bytes(data_dir);
    format!("{total} byte(s) ({})", human(total))
}

/// The same footprint as a number, which is what `--json` carries: a consumer
/// that wants "3.2 MiB" can render it, and one that wants to compare two runs
/// cannot parse it back out of prose.
fn size_bytes(data_dir: &Path) -> u64 {
    let mut total = 0u64;
    for name in [
        DB_FILE_NAME.to_owned(),
        format!("{DB_FILE_NAME}-wal"),
        format!("{DB_FILE_NAME}-shm"),
    ] {
        if let Ok(meta) = std::fs::metadata(data_dir.join(name)) {
            total += meta.len();
        }
    }
    total
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
