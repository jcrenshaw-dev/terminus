//! A store written while sessions were keyed on their absolute path, repaired
//! by one ingest pass (#57, phase 4 D-14).
//!
//! One store holds every case. Each session is ingested from a staging tree of
//! its own and then moved to the absolute key an older build gave it, so its
//! block rows, turns and watermark are what ingest really writes.

#![cfg(feature = "testkit")]

use std::path::PathBuf;

use rusqlite::Connection;
use terminus_core::config::Config;
use terminus_core::ingest::pass::{self, PassOutcome};
use terminus_core::store::{Store, DB_FILE_NAME};
use terminus_core::{ingest, testkit};

const U0: &str = "a0000000-0000-4000-8000-000000000000";
const U1: &str = "a1111111-1111-4111-8111-111111111111";
const U2: &str = "a2222222-2222-4222-8222-222222222222";
const U3: &str = "a3333333-3333-4333-8333-333333333333";

struct Bench {
    dir: tempfile::TempDir,
    data_dir: PathBuf,
    /// The Claude config directory the pass walks.
    claude_dir: PathBuf,
    /// Each planted session's `session_no` and the key it is planted under.
    planted: Vec<(i64, String)>,
}

impl Bench {
    fn new() -> Bench {
        let dir = tempfile::tempdir_in(testkit::temp_root()).unwrap();
        let data_dir = dir.path().join("data");
        let claude_dir = dir.path().join("claude");
        std::fs::create_dir_all(claude_dir.join("projects")).unwrap();
        Bench {
            dir,
            data_dir,
            claude_dir,
            planted: Vec::new(),
        }
    }

    fn conn(&self) -> Connection {
        Connection::open(self.data_dir.join(DB_FILE_NAME)).unwrap()
    }

    /// An absolute key under a `.claude` at `root` that no longer exists.
    fn gone(&self, root: &str, below_projects: &str) -> String {
        self.dir
            .path()
            .join(root)
            .join(".claude/projects")
            .join(below_projects)
            .display()
            .to_string()
    }

    /// Archive `bytes` as a session of its own, to be keyed on `key` once every
    /// session is planted.
    fn plant(&mut self, bytes: &[u8], key: String) -> i64 {
        let staged = self
            .dir
            .path()
            .join("staging/projects")
            .join(format!("-stage-{}", self.planted.len()))
            .join(format!("{U0}.jsonl"));
        std::fs::create_dir_all(staged.parent().unwrap()).unwrap();
        std::fs::write(&staged, bytes).unwrap();
        match ingest::run_with(&self.data_dir, &staged, &Config::from_parts(vec![], vec![])) {
            Ok(ingest::Outcome::Committed(_)) => {}
            other => panic!("{other:?}"),
        }
        let session_no: i64 = self
            .conn()
            .query_row("SELECT max(session_no) FROM sessions", [], |r| r.get(0))
            .unwrap();
        self.planted.push((session_no, key));
        session_no
    }

    /// Move every planted session to its key.
    fn key_planted(&self) {
        let mut conn = self.conn();
        for (session_no, key) in &self.planted {
            let from: String = conn
                .query_row(
                    "SELECT session_key FROM sessions WHERE session_no = ?1",
                    [session_no],
                    |r| r.get(0),
                )
                .unwrap();
            testkit::plant_old_key(&mut conn, &from, key);
        }
    }

    fn pass(&self) {
        let config = Config::from_parts(vec![self.claude_dir.clone()], vec![]);
        match pass::run_with(&self.data_dir, &config).unwrap() {
            PassOutcome::Ran(_) => {}
            PassOutcome::LockHeld => panic!("nothing else holds the lock"),
        }
    }
}

/// Lines `range` of a fixture, newlines included.
fn lines(fixture: &str, range: std::ops::Range<usize>) -> Vec<u8> {
    let bytes = testkit::fixture_bytes(fixture);
    bytes
        .split_inclusive(|b| *b == b'\n')
        .skip(range.start)
        .take(range.len())
        .flatten()
        .copied()
        .collect()
}

fn keys(conn: &Connection) -> Vec<String> {
    let mut keys: Vec<String> = conn
        .prepare("SELECT session_key FROM sessions")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    keys.sort();
    keys
}

/// Everything about one session that a re-key must not change.
#[derive(Debug, PartialEq)]
struct Archived {
    session_no: i64,
    blocks: Vec<(i64, Vec<u8>, Vec<u8>)>,
    checksum: Vec<u8>,
    uncompressed_len: i64,
    transcript_path: String,
    turn_ids: Vec<i64>,
    watermark: Option<i64>,
}

fn archived(conn: &Connection, key: &str) -> Archived {
    let (session_no, checksum, uncompressed_len, transcript_path) = conn
        .query_row(
            "SELECT s.session_no, m.checksum, m.uncompressed_len, m.transcript_path
               FROM sessions s JOIN session_meta m USING (session_key)
              WHERE s.session_key = ?1",
            [key],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or_else(|e| panic!("{key}: {e}"));
    let turn_ids = conn
        .prepare("SELECT id FROM turns WHERE session_key = ?1 ORDER BY turn_seq")
        .unwrap()
        .query_map([key], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let watermark = conn
        .query_row(
            "SELECT byte_offset FROM watermarks WHERE transcript_path = ?1",
            [key],
            |r| r.get(0),
        )
        .ok();
    Archived {
        session_no,
        blocks: testkit::block_rows(conn, key),
        checksum,
        uncompressed_len,
        transcript_path,
        turn_ids,
        watermark,
    }
}

/// Every row a pass could change, outside `runs` and `meta`.
fn rows(conn: &Connection) -> Vec<String> {
    let mut out = Vec::new();
    for table in [
        "sessions",
        "session_meta",
        "blocks",
        "turns",
        "compactions",
        "observations",
        "entities",
        "paths",
        "watermarks",
        "decisions",
        "labels",
    ] {
        let mut statement = conn
            .prepare(&format!("SELECT * FROM {table} ORDER BY 1, 2"))
            .unwrap();
        let columns = statement.column_count();
        let mut cursor = statement.query([]).unwrap();
        while let Some(row) = cursor.next().unwrap() {
            let values: Vec<rusqlite::types::Value> =
                (0..columns).map(|i| row.get(i).unwrap()).collect();
            out.push(format!("{table} {values:?}"));
        }
    }
    out
}

fn last_error(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT error FROM runs ORDER BY id DESC LIMIT 1", [], |r| {
        r.get(0)
    })
    .unwrap()
}

fn foreign_key_violations(conn: &Connection) -> Vec<String> {
    conn.prepare("PRAGMA foreign_key_check")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn one_pass_repairs_a_store_keyed_on_absolute_paths() {
    let mut bench = Bench::new();
    let head = lines("session-basic.jsonl", 0..10);
    let joined = |parts: &[&[u8]]| parts.concat();

    // An archive-only orphan: its file is gone with the old `.claude`.
    let orphan_old = bench.gone("old-home", &format!("-p/{U0}.jsonl"));
    bench.plant(
        &testkit::fixture_bytes("session-basic.jsonl"),
        orphan_old.clone(),
    );
    // A sidecar whose key and parent key are both absolute.
    let sidecar_old = bench.gone("old-home", &format!("-p/{U0}/subagents/agent-a.jsonl"));
    bench.plant(
        &testkit::fixture_bytes("subagents/agent-alpha.jsonl"),
        sidecar_old.clone(),
    );

    // Two roots holding different files at one path below `projects`, which
    // differ in a full block.
    let apart = [
        bench.gone("root-a", &format!("-n/{U1}.jsonl")),
        bench.gone("root-b", &format!("-n/{U1}.jsonl")),
    ];
    bench.plant(
        &testkit::fixture_bytes("session-basic.jsonl"),
        apart[0].clone(),
    );
    bench.plant(
        &testkit::fixture_bytes("session-secrets.jsonl"),
        apart[1].clone(),
    );

    // Two that agree on every full block of the shorter stream and differ
    // inside its partial last block.
    let tail_apart = [
        bench.gone("root-a", &format!("-n/{U2}.jsonl")),
        bench.gone("root-b", &format!("-n/{U2}.jsonl")),
    ];
    let shorter = joined(&[&head, &lines("session-continuation.jsonl", 0..1)]);
    let longer = joined(&[&head, &lines("session-recall.jsonl", 0..2)]);
    assert!(shorter.len() < 2 * head.len() && longer.len() > shorter.len());
    bench.plant(&shorter, tail_apart[0].clone());
    bench.plant(&longer, tail_apart[1].clone());

    // A survivor and two longer members that both extend it and differ from
    // each other after its end: the first one passes, the second fails
    // against it.
    let three = [
        bench.gone("root-a", &format!("-g/{U3}.jsonl")),
        bench.gone("root-b", &format!("-g/{U3}.jsonl")),
        bench.gone("root-c", &format!("-g/{U3}.jsonl")),
    ];
    bench.plant(&head, three[0].clone());
    bench.plant(
        &joined(&[&head, &lines("session-basic.jsonl", 10..15)]),
        three[1].clone(),
    );
    bench.plant(
        &joined(&[&head, &lines("session-secrets.jsonl", 0..3)]),
        three[2].clone(),
    );

    bench.key_planted();
    {
        let conn = bench.conn();
        conn.execute(
            "UPDATE session_meta SET parent_session_key = ?2 WHERE session_key = ?1",
            [&sidecar_old, &orphan_old],
        )
        .unwrap();
        // Small blocks, so these pairs have full blocks to compare.
        for key in &apart {
            testkit::recut(&conn, key, 1024);
        }
        for key in &tail_apart {
            testkit::recut(&conn, key, head.len() as u32);
        }
    }
    let kept: Vec<String> = apart
        .iter()
        .chain(&tail_apart)
        .chain(&three)
        .cloned()
        .collect();
    let before = {
        let conn = bench.conn();
        let mut before: Vec<Archived> = vec![archived(&conn, &orphan_old)];
        before.extend(kept.iter().map(|key| archived(&conn, key)));
        before
    };

    bench.pass();

    let conn = bench.conn();
    let orphan = format!("-p/{U0}.jsonl");
    let sidecar = format!("-p/{U0}/subagents/agent-a.jsonl");
    let mut expected = vec![orphan.clone(), sidecar.clone()];
    expected.extend(kept.iter().cloned());
    expected.sort();
    assert_eq!(keys(&conn), expected, "one session per transcript");

    // Re-keyed in place: the same bytes, number, turn ids and watermark.
    assert_eq!(archived(&conn, &orphan), before[0]);
    let old_watermarks: i64 = conn
        .query_row(
            "SELECT count(*) FROM watermarks WHERE transcript_path = ?1",
            [&orphan_old],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(old_watermarks, 0, "the old key kept a watermark");
    let parent: String = conn
        .query_row(
            "SELECT parent_session_key FROM session_meta WHERE session_key = ?1",
            [&sidecar],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(parent, orphan);

    // Not provably one stream: kept exactly as they were.
    for (key, was) in kept.iter().zip(&before[1..]) {
        assert_eq!(&archived(&conn, key), was, "{key} changed");
    }

    let error = last_error(&conn).unwrap_or_default();
    assert!(
        error.contains("re-keyed 2 session(s)"),
        "runs.error: {error}"
    );
    for (pair, why) in [
        (&apart, "block 0 differs"),
        (&tail_apart, "the shorter stream's last, differs"),
    ] {
        let line = format!("{}: not merged into {}", pair[1], pair[0]);
        assert!(
            error
                .lines()
                .any(|l| l.starts_with(&line) && l.contains(why)),
            "{line} ({why}) is not reported: {error}"
        );
    }
    assert!(
        error.contains(&format!("{}: not merged into {}", three[2], three[0])),
        "the second longer member is not named: {error}"
    );
    assert!(
        !error.contains(three[1].as_str()),
        "the first longer member is named: {error}"
    );
    let report = terminus_core::verify::verify(&Store::open(&bench.data_dir).unwrap()).unwrap();
    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert_eq!(foreign_key_violations(&conn), Vec::<String>::new());

    // The steady state: a second pass changes no row and reports only the
    // twins it still cannot merge.
    let unmerged: Vec<&str> = error
        .lines()
        .filter(|line| line.contains("not merged into"))
        .collect();
    let rows_before = rows(&conn);
    bench.pass();
    assert_eq!(rows(&conn), rows_before, "the second pass changed a row");
    let second = last_error(&conn).unwrap_or_default();
    assert_eq!(second.lines().collect::<Vec<_>>(), unmerged);
}
