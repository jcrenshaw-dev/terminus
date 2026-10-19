//! Recovery, run at the top of every ingest run (ING-03, D-24).
//!
//! No repair command, no external supervisor, no stale-lock handling. The last
//! of those is not an omission: the ingest lock is an OS lock that dies with the
//! process (`crate::ingest::lock`), so there is no stale lock to recover from.
//! What is left is three things, in this order.
//!
//! **The re-key** (#57, phase 4). A session used to be keyed on its absolute
//! path, so a moved or symlinked `.claude` gave every transcript a second key
//! and archived it twice. The key is now the path below `projects`
//! ([`crate::ingest::path_key`]), and a store written before that is re-keyed
//! here, in one transaction, before anything is rebuilt or read. A session with
//! no twin is re-keyed in place: its key moves in every table that holds it, and
//! its block rows, `session_no` and turn ids do not change. Sessions that share
//! a new key are left exactly as they are. The steady state reads the stored
//! keys that are still absolute paths and writes nothing.
//!
//! **The rebuild.** `reindex::up_to_date` brings a store older than this build
//! forward before a single transcript is read, so a walk never appends turn
//! rows in one shape beside rows written in another. It runs after the re-key,
//! so it never rebuilds a session the re-key is about to change (D-09).
//!
//! **The watermark sweep.** A killed pass can leave a watermark claiming bytes
//! the committed blob does not hold - that is the state phase 1's crash harness
//! enumerates and the one STOR-02 says a store must never be left in. The sweep
//! reconciles it against `session_meta.uncompressed_len` rather than by
//! decompressing anything, which is what makes it one indexed pass over ~2,000
//! rows instead of 988 MB of zstd. Cheap enough to run on every entry point,
//! which is what ING-03's "every run" requires.
//!
//! No archived byte is lost here. A watermark is lowered so the next pass
//! re-reads the gap, an orphaned watermark is removed because it claims bytes
//! nothing archived, and a session is re-keyed in place.
//!
//! **The sweep's first arm is outside a session that was not captured under
//! `full`** (ING-07, phase 8 D-05). It compares a FILE offset against a count of
//! STORED bytes, and that comparison only carries information while the two
//! coordinate systems coincide - which is exactly what a capture mode's elision
//! ends. For a `lean` or `minimal` session `byte_offset > uncompressed_len` is
//! the NORMAL state, so left alone the sweep would "repair" every elided session
//! on every run, lowering its watermark and making the next pass re-read and
//! re-append bytes the blob already holds, forever.
//!
//! Nothing is lost by standing back. The blob and the watermark commit in one
//! transaction (STOR-02), so there is no divergence between them for this to
//! find; the arm was defence in depth against a state the transaction already
//! prevents. It does mean an elided session has no independent second witness
//! for its watermark, and there is none available without a second
//! `session_meta` column that D-13 deliberately did not add.
//!
//! The ORPHAN arm is untouched and must stay untouched. It compares nothing
//! against the stored bytes - a non-zero watermark for a path nothing archived
//! is wrong under every capture mode - and it is what removes the watermark of a
//! session retention deleted.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use rusqlite::Connection;

use crate::error::Result;
use crate::ingest::path_key;
use crate::store::Store;

/// A watermark that claimed more bytes than its blob holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredWatermark {
    pub session_key: String,
    pub from: u64,
    /// The uncompressed length the committed blob actually holds.
    pub to: u64,
}

/// A watermark naming no archived session at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedWatermark {
    pub session_key: String,
    pub byte_offset: u64,
}

/// What recovery repaired.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recovered {
    pub lowered: Vec<LoweredWatermark>,
    pub removed: Vec<RemovedWatermark>,
    /// Sessions whose absolute-path key moved to the path below `projects`.
    pub rekeyed: usize,
    pub duration: Duration,
}

impl Recovered {
    pub fn repaired(&self) -> bool {
        !self.lowered.is_empty() || !self.removed.is_empty() || self.rekeyed > 0
    }

    /// One line per repair, for a `runs.error` or a `status` report. The re-key
    /// is one line however many sessions it moved: the first run on a large
    /// store moves thousands.
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.rekeyed > 0 {
            out.push(format!(
                "re-keyed {} session(s) to their path below `projects`",
                self.rekeyed
            ));
        }
        for w in &self.lowered {
            out.push(format!(
                "{}: watermark {} was past the {} bytes the archive holds; lowered",
                w.session_key, w.from, w.to
            ));
        }
        for w in &self.removed {
            out.push(format!(
                "{}: watermark {} named no archived session; removed",
                w.session_key, w.byte_offset
            ));
        }
        out
    }
}

/// Open the store for work, recovering it first.
///
/// One callable with two callers - the tree pass and single-file
/// `ingest::run` - because ING-03 says every run and `ingest::run` is a run.
/// Wiring only the pass would leave `terminus ingest <path.jsonl>` proceeding
/// against a watermark ahead of what that file's committed blob holds, so
/// `read_tail` would resume from an offset the archive never reached and the
/// gap would be lost from the blob in silence.
///
/// Both callers already hold the ingest lock when they get here, and both call
/// it before any transcript is read.
///
/// A re-key that fails stops the run. Ingesting under the new keys over a store
/// still holding the old ones would archive every session a third time.
pub fn recover(data_dir: &Path) -> Result<(Store, Recovered)> {
    let started = Instant::now();
    let mut store = Store::open(data_dir)?;
    let mut report = Recovered::default();
    rekey(&mut store, &mut report)?;
    let mut store = crate::reindex::up_to_date(store)?;
    let (lowered, removed) = sweep_watermarks(&mut store)?;
    report.lowered = lowered;
    report.removed = removed;
    report.duration = started.elapsed();
    Ok((store, report))
}

/// Move every absolute-path key to the path below `projects` (#57).
fn rekey(store: &mut Store, report: &mut Recovered) -> Result<()> {
    let groups = stale_keys(store.conn())?;
    let parents = stale_parents(store.conn())?;
    if groups.is_empty() && parents.is_empty() {
        return Ok(());
    }

    let tx = store.conn_mut().transaction()?;
    // The key is a parent of four tables, none declared `ON UPDATE CASCADE`.
    // Deferred, the parent and its children move in any order and are checked
    // at commit, which still refuses a child left behind.
    tx.execute_batch("PRAGMA defer_foreign_keys = ON")?;

    for (new, old) in &groups {
        let held = tx.query_row(
            "SELECT count(*) FROM sessions WHERE session_key = ?1",
            [new],
            |r| r.get::<_, i64>(0),
        )? > 0;
        // Two or more sessions for one transcript are twins, and are left
        // exactly as they are.
        if let [only] = old.as_slice() {
            if !held {
                move_key(&tx, only, new)?;
                report.rekeyed += 1;
            }
        }
    }
    for (old, new) in &parents {
        tx.execute(
            "UPDATE session_meta SET parent_session_key = ?2 WHERE parent_session_key = ?1",
            [old, new],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// The stored keys that are still absolute paths and reduce to another key,
/// grouped by that key.
///
/// `/` sorts just below `0`, so the range is a walk of the primary key's index
/// over exactly the absolute keys, and a store with none reads nothing else.
fn stale_keys(conn: &Connection) -> Result<BTreeMap<String, Vec<String>>> {
    let keys: Vec<String> = conn
        .prepare("SELECT session_key FROM sessions WHERE session_key >= '/' AND session_key < '0'")?
        .query_map([], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in keys {
        let new = path_key(Path::new(&key))?;
        if new != key {
            groups.entry(new).or_default().push(key);
        }
    }
    Ok(groups)
}

/// Every `parent_session_key` that is an absolute path reducing to another key,
/// with its reduction.
fn stale_parents(conn: &Connection) -> Result<Vec<(String, String)>> {
    let keys: Vec<String> = conn
        .prepare(
            "SELECT DISTINCT parent_session_key FROM session_meta
              WHERE parent_session_key >= '/' AND parent_session_key < '0'",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    let mut out = Vec::new();
    for key in keys {
        let new = path_key(Path::new(&key))?;
        if new != key {
            out.push((key, new));
        }
    }
    Ok(out)
}

/// Move one session's key in every table that holds it. Block rows, turn ids and
/// `transcript_path` stay as they are.
///
/// A watermark already under the new key is a pass's row for a file it found
/// under that key, and the session's own row replaces it.
fn move_key(tx: &Connection, from: &str, to: &str) -> Result<()> {
    for table in [
        "sessions",
        "session_meta",
        "turns",
        "compactions",
        "observations",
    ] {
        tx.execute(
            &format!("UPDATE {table} SET session_key = ?2 WHERE session_key = ?1"),
            [from, to],
        )?;
    }
    tx.execute("DELETE FROM watermarks WHERE transcript_path = ?1", [to])?;
    tx.execute(
        "UPDATE watermarks SET transcript_path = ?2 WHERE transcript_path = ?1",
        [from, to],
    )?;
    Ok(())
}

/// Reconcile `watermarks` against what the committed blobs actually hold.
///
/// One transaction, so a store is never left half-reconciled.
fn sweep_watermarks(store: &mut Store) -> Result<(Vec<LoweredWatermark>, Vec<RemovedWatermark>)> {
    let tx = store.conn_mut().transaction()?;

    // `session_meta.uncompressed_len`, not a decompression: the length is the
    // whole question, and reading it keeps the sweep an indexed join over two
    // primary keys.
    //
    // Restricted to sessions captured under `full` (ING-07, phase 8 D-05) - see
    // the module comment. Null is `full`: it is what every row written before
    // the column existed means.
    let lowered: Vec<LoweredWatermark> = tx
        .prepare(
            "SELECT w.transcript_path, w.byte_offset, m.uncompressed_len
             FROM watermarks w
             JOIN session_meta m ON m.session_key = w.transcript_path
             WHERE w.byte_offset > m.uncompressed_len
               AND coalesce(m.capture_mode, 'full') = 'full'
             ORDER BY w.transcript_path",
        )?
        .query_map([], |r| {
            Ok(LoweredWatermark {
                session_key: r.get(0)?,
                from: r.get::<_, i64>(1)? as u64,
                to: r.get::<_, i64>(2)? as u64,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;

    // A non-zero watermark for a path nothing archived. Zero is excluded
    // because it claims no bytes: it is what a pass writes for a transcript
    // with no complete record yet, and removing it would be churn, not repair.
    //
    // A session row present WITHOUT its `session_meta` row is deliberately not
    // in either query. That is the damaged state phase 1 refuses to guess at
    // and `verify` reports; touching its watermark here would be exactly the
    // guess D-12 forbids.
    let removed: Vec<RemovedWatermark> = tx
        .prepare(
            "SELECT w.transcript_path, w.byte_offset
             FROM watermarks w
             WHERE w.byte_offset <> 0
               AND NOT EXISTS (
                   SELECT 1 FROM sessions s WHERE s.session_key = w.transcript_path
               )
             ORDER BY w.transcript_path",
        )?
        .query_map([], |r| {
            Ok(RemovedWatermark {
                session_key: r.get(0)?,
                byte_offset: r.get::<_, i64>(1)? as u64,
            })
        })?
        .collect::<std::result::Result<_, _>>()?;

    if lowered.is_empty() && removed.is_empty() {
        // Nothing to write. A healthy store leaves this transaction having read
        // two indexed queries and changed no row.
        drop(tx);
        return Ok((lowered, removed));
    }

    for repair in &lowered {
        tx.execute(
            "UPDATE watermarks SET byte_offset = ?2,
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE transcript_path = ?1",
            rusqlite::params![repair.session_key, repair.to as i64],
        )?;
    }
    for repair in &removed {
        tx.execute(
            "DELETE FROM watermarks WHERE transcript_path = ?1",
            [&repair.session_key],
        )?;
    }

    tx.commit()?;
    Ok((lowered, removed))
}
