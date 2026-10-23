//! The `runs` table: the one row every pass leaves behind.
//!
//! The column list lives here and nowhere else. The driver writes the row the
//! same way for a tree pass, a backfill and one named file, so the last run
//! `terminus status` reports means the same thing whichever of the three ran.

use crate::error::Result;
use crate::ingest::pass::{note, Summary};

/// The one `runs` row a pass leaves behind.
///
/// `runs.error` and `runs.notes` are the only textual channel this product has -
/// there is no log file, by design, and `terminus status` surfaces this table -
/// so everything worth a human's attention goes in one of them. `error` holds
/// what went wrong: the path and reason of every file that was skipped, every
/// directory that could not be listed, every watermark recovery had to repair,
/// every step that could not run, and the failure that ended the pass if one
/// did. `notes` holds what a healthy pass did worth saying - labels written,
/// sessions retention evicted or deleted - so `doctor` can flag `error` alone.
/// Each stays null when there is genuinely nothing to say.
///
/// `fatal` is the line that says the pass as a whole failed, already worded by
/// the walk that failed, and it goes last.
pub(crate) fn insert(
    conn: &rusqlite::Connection,
    summary: &Summary,
    fatal: Option<String>,
) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();
    for (path, reason) in summary.failures.iter().chain(&summary.unreadable) {
        errors.push(note(path, reason));
    }
    errors.extend(summary.recovery.lines());
    errors.extend(summary.feedback.lines());
    errors.extend(summary.outcomes.errors());
    errors.extend(summary.observations.lines());
    errors.extend(summary.retention.errors());
    errors.extend(fatal);
    let mut notes = summary.outcomes.notes();
    notes.extend(summary.retention.notes());
    let joined = |lines: Vec<String>| (!lines.is_empty()).then(|| lines.join("\n"));
    let error = joined(errors);
    let notes = joined(notes);

    let elapsed = summary.duration;
    conn.execute(
        "INSERT INTO runs (
            started_at, finished_at, duration_ms, files_seen, bytes_read,
            turns_added, error, files_committed, files_failed, notes
         ) VALUES (
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1),
            strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
            ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
         )",
        rusqlite::params![
            format!("-{} seconds", elapsed.as_secs_f64()),
            elapsed.as_millis() as i64,
            summary.files_walked as i64,
            summary.bytes_read as i64,
            summary.turns_added as i64,
            error,
            summary.files_committed as i64,
            summary.failures.len() as i64,
            notes,
        ],
    )?;
    Ok(())
}
