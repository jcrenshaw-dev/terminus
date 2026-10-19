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

/// A session that reduces to the same key as an older one, left as it is
/// because the two are not provably one stream (phase 4 D-10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmergedTwin {
    pub session_key: String,
    /// The group's oldest session, which the twin was tested against.
    pub survivor_key: String,
    /// The key both reduce to.
    pub new_key: String,
    pub reason: String,
}

/// What recovery repaired.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recovered {
    pub lowered: Vec<LoweredWatermark>,
    pub removed: Vec<RemovedWatermark>,
    /// Sessions whose absolute-path key moved to the path below `projects`.
    pub rekeyed: usize,
    /// Reported again on every run until they are merged.
    pub unmerged: Vec<UnmergedTwin>,
    pub duration: Duration,
}

impl Recovered {
    pub fn repaired(&self) -> bool {
        !self.lowered.is_empty()
            || !self.removed.is_empty()
            || self.rekeyed > 0
            || !self.unmerged.is_empty()
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
        for t in &self.unmerged {
            out.push(format!(
                "{}: not merged into {}, though both reduce to {}: {}; both are kept",
                t.session_key, t.survivor_key, t.new_key, t.reason
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
        if let ([only], false) = (old.as_slice(), held) {
            move_key(&tx, only, new)?;
            report.rekeyed += 1;
            continue;
        }

        // Twins: every session reducing to `new`, oldest first. The oldest
        // survives (D-11), and each other member is tested against the stream
        // the group holds so far, so two members that both extend the survivor
        // and differ after its end are never both taken.
        let mut group = Vec::with_capacity(old.len() + 1);
        for key in old.iter().chain(held.then_some(new)) {
            group.push(Member::read(&tx, key)?);
        }
        group.sort_by_key(|m| m.session_no);
        let (survivor, others) = group.split_first().expect("a group has members");
        let mut current = survivor;
        for member in others {
            match one_stream(&tx, current, member)? {
                Ok(()) => {
                    if member.stream.as_ref().map(|s| s.len)
                        > current.stream.as_ref().map(|s| s.len)
                    {
                        current = member;
                    }
                }
                Err(reason) => report.unmerged.push(UnmergedTwin {
                    session_key: member.key.clone(),
                    survivor_key: survivor.key.clone(),
                    new_key: new.clone(),
                    reason,
                }),
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

/// One session of a group that reduces to one key.
struct Member {
    key: String,
    session_no: i64,
    /// `None` when the session has no `session_meta` row, which `verify` reports
    /// and nothing here guesses at.
    stream: Option<Stream>,
}

/// What `session_meta` says about one session's stream.
struct Stream {
    evicted: bool,
    block_size: Option<i64>,
    len: i64,
    checksum: Vec<u8>,
}

impl Member {
    fn read(conn: &Connection, key: &str) -> Result<Member> {
        type Row = (i64, Option<bool>, Option<i64>, Option<i64>, Option<Vec<u8>>);
        let (session_no, evicted, block_size, len, checksum): Row = conn.query_row(
            "SELECT s.session_no, coalesce(m.is_evicted, 0) <> 0, m.block_size,
                    m.uncompressed_len, m.checksum
               FROM sessions s LEFT JOIN session_meta m USING (session_key)
              WHERE s.session_key = ?1",
            [key],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;
        let stream = match (evicted, len, checksum) {
            (Some(evicted), Some(len), Some(checksum)) => Some(Stream {
                evicted,
                block_size,
                len,
                checksum,
            }),
            _ => None,
        };
        Ok(Member {
            key: key.to_owned(),
            session_no,
            stream,
        })
    }
}

/// Whether one member's stream is a prefix of the other's (D-10), or why not.
///
/// Reads the hash column, and decompresses at most the two blocks holding the
/// shorter stream's partial end. Two streams that agree on every full block can
/// still differ inside that last block, and merging them would lose the shorter
/// one's bytes.
fn one_stream(
    conn: &Connection,
    a: &Member,
    b: &Member,
) -> Result<std::result::Result<(), String>> {
    let (Some(x), Some(y)) = (&a.stream, &b.stream) else {
        return Ok(Err("one of the two has no session_meta row".into()));
    };
    // An evicted session has no block rows, and an empty hash list is a prefix
    // of anything, so eviction is compared on what the metadata recorded.
    match (x.evicted, y.evicted) {
        (true, true) => {
            return Ok(
                if x.len == y.len && x.checksum == y.checksum && x.block_size == y.block_size {
                    Ok(())
                } else {
                    Err("both are evicted and recorded different streams".into())
                },
            )
        }
        (true, false) | (false, true) => return Ok(Err("one of the two is evicted".into())),
        (false, false) => {}
    }
    let size = match (x.block_size, y.block_size) {
        (Some(p), Some(q)) if p == q && p > 0 => p,
        _ => return Ok(Err("their block sizes differ or are not recorded".into())),
    };
    let ((short, s), (long, l)) = if x.len <= y.len {
        ((a, x), (b, y))
    } else {
        ((b, y), (a, x))
    };
    let short_hashes = match hashes(conn, short.session_no, s.len, size)? {
        Ok(h) => h,
        Err(e) => return Ok(Err(format!("{}: {e}", short.key))),
    };
    let long_hashes = match hashes(conn, long.session_no, l.len, size)? {
        Ok(h) => h,
        Err(e) => return Ok(Err(format!("{}: {e}", long.key))),
    };
    if s.len == l.len {
        return Ok(if s.checksum == l.checksum {
            Ok(())
        } else {
            Err("the two streams are the same length and hold different bytes".into())
        });
    }
    let full = (s.len / size) as usize;
    if let Some(i) = (0..full).find(|&i| short_hashes[i] != long_hashes[i]) {
        return Ok(Err(format!("block {i} differs")));
    }
    if s.len % size != 0 {
        let end = |m: &Member, len: i64| block(conn, m.session_no, full as i64, size, len);
        let (tail, over) = match (end(short, s.len)?, end(long, l.len)?) {
            (Ok(tail), Ok(over)) => (tail, over),
            (Err(e), _) | (_, Err(e)) => return Ok(Err(e)),
        };
        if !over.starts_with(&tail) {
            return Ok(Err(format!(
                "block {full}, the shorter stream's last, differs"
            )));
        }
    }
    Ok(Ok(()))
}

/// One session's block hashes, when they are numbered from 0 with no gap and
/// there are as many as its length needs.
fn hashes(
    conn: &Connection,
    session_no: i64,
    len: i64,
    size: i64,
) -> Result<std::result::Result<Vec<Vec<u8>>, String>> {
    let rows: Vec<(i64, Vec<u8>)> = conn
        .prepare_cached(
            "SELECT block_no, hash FROM blocks WHERE session_no = ?1 ORDER BY block_no",
        )?
        .query_map([session_no], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<std::result::Result<_, _>>()?;
    if let Some((i, _)) = rows
        .iter()
        .enumerate()
        .find(|(i, (no, _))| *no != *i as i64)
    {
        return Ok(Err(format!("block {i} is missing")));
    }
    let wanted = (len + size - 1) / size;
    if rows.len() as i64 != wanted {
        return Ok(Err(format!(
            "it has {} block rows where its length needs {wanted}",
            rows.len()
        )));
    }
    Ok(Ok(rows.into_iter().map(|(_, hash)| hash).collect()))
}

/// One block of one session, decompressed.
fn block(
    conn: &Connection,
    session_no: i64,
    block_no: i64,
    size: i64,
    len: i64,
) -> Result<std::result::Result<Vec<u8>, String>> {
    let bytes: Vec<u8> = conn.query_row(
        "SELECT bytes FROM blocks WHERE session_no = ?1 AND block_no = ?2",
        [session_no, block_no],
        |r| r.get(0),
    )?;
    let read = u32::try_from(size)
        .map_err(|_| format!("block size {size} is out of range"))
        .and_then(|size| {
            crate::blob::BlockReader::new(size, len as u64)
                .and_then(|reader| reader.decompress(block_no as u64, &bytes))
                .map_err(|e| e.to_string())
        });
    Ok(read)
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
