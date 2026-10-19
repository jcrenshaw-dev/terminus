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
const U4: &str = "a4444444-4444-4444-8444-444444444444";
const U5: &str = "a5555555-5555-4555-8555-555555555555";
const U6: &str = "a6666666-6666-4666-8666-666666666666";
const U7: &str = "a7777777-7777-4777-8777-777777777777";

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

    /// Write a transcript into the tree the pass walks, answering its path.
    fn place(&self, below_projects: &str, bytes: &[u8]) -> String {
        let path = self.claude_dir.join("projects").join(below_projects);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        path.display().to_string()
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

fn session_no(conn: &Connection, key: &str) -> i64 {
    conn.query_row(
        "SELECT session_no FROM sessions WHERE session_key = ?1",
        [key],
        |r| r.get(0),
    )
    .unwrap_or_else(|e| panic!("{key}: {e}"))
}

/// The members `runs.error` says were not merged.
fn unmerged(error: &str) -> Vec<String> {
    let mut named: Vec<String> = error
        .lines()
        .filter_map(|line| line.split_once(": not merged into "))
        .map(|(member, _)| member.to_owned())
        .collect();
    named.sort();
    named
}

#[test]
fn one_pass_repairs_a_store_keyed_on_absolute_paths() {
    let mut bench = Bench::new();
    let basic = testkit::fixture_bytes("session-basic.jsonl");
    let alpha = testkit::fixture_bytes("subagents/agent-alpha.jsonl");
    let head = lines("session-basic.jsonl", 0..10);
    let joined = |parts: &[&[u8]]| parts.concat();

    // An archive-only orphan: its file is gone with the old `.claude`.
    let orphan_old = bench.gone("old-home", &format!("-p/{U0}.jsonl"));
    bench.plant(&basic, orphan_old.clone());
    // A sidecar whose key and parent key are both absolute.
    let sidecar_old = bench.gone("old-home", &format!("-p/{U0}/subagents/agent-a.jsonl"));
    bench.plant(&alpha, sidecar_old.clone());

    // Two roots holding different files at one path below `projects`, which
    // differ in a full block.
    let apart = [
        bench.gone("root-a", &format!("-n/{U1}.jsonl")),
        bench.gone("root-b", &format!("-n/{U1}.jsonl")),
    ];
    bench.plant(&basic, apart[0].clone());
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
    let three_longer = joined(&[&head, &lines("session-basic.jsonl", 10..15)]);
    bench.plant(&head, three[0].clone());
    bench.plant(&three_longer, three[1].clone());
    bench.plant(
        &joined(&[&head, &lines("session-secrets.jsonl", 0..3)]),
        three[2].clone(),
    );

    // The twins a moved `.claude` leaves: the older one under the old root,
    // the newer one under the path the pass reads today, which is on disk.
    let identical = [
        bench.gone("old-home", &format!("-q/{U4}.jsonl")),
        bench.place(&format!("-q/{U4}.jsonl"), &basic),
    ];
    bench.plant(&basic, identical[0].clone());
    bench.plant(&basic, identical[1].clone());
    let prefix = [
        bench.gone("old-home", &format!("-q/{U5}.jsonl")),
        bench.place(&format!("-q/{U5}.jsonl"), &basic),
    ];
    bench.plant(&head, prefix[0].clone());
    bench.plant(&basic, prefix[1].clone());

    // Claude Code's resume link: a sidecar in one session's `subagents/`
    // pointing at another session's, each archived under its own spelling.
    let target = bench.place(&format!("-r/{U6}/subagents/agent-x.jsonl"), &alpha);
    let link = bench
        .claude_dir
        .join(format!("projects/-r/{U7}/subagents/agent-x.jsonl"));
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let linked = [link.display().to_string(), target];
    bench.plant(&alpha, linked[0].clone());
    bench.plant(&alpha, linked[1].clone());

    bench.key_planted();
    {
        let conn = bench.conn();
        conn.execute(
            "UPDATE session_meta SET parent_session_key = ?2 WHERE session_key = ?1",
            [&sidecar_old, &orphan_old],
        )
        .unwrap();
        // Small blocks, so these pairs have full blocks to compare.
        for key in apart.iter().chain(&prefix) {
            testkit::recut(&conn, key, 1024);
        }
        for key in &tail_apart {
            testkit::recut(&conn, key, head.len() as u32);
        }
        conn.execute(
            "UPDATE session_meta SET continues_from = NULL WHERE session_key = ?1",
            [&identical[0]],
        )
        .unwrap();
        conn.execute(
            "UPDATE session_meta SET continues_from = 'b0000000-0000-4000-8000-000000000000'
              WHERE session_key = ?1",
            [&identical[1]],
        )
        .unwrap();
    }
    let kept: Vec<String> = apart
        .iter()
        .chain(&tail_apart)
        .chain([&three[2]])
        .cloned()
        .collect();
    let conn = bench.conn();
    let orphan_before = archived(&conn, &orphan_old);
    let kept_before: Vec<Archived> = kept.iter().map(|key| archived(&conn, key)).collect();
    let prefix_longer = archived(&conn, &prefix[1]);
    let prefix_stream = testkit::stream(&conn, &prefix[1]).unwrap();
    let three_stream = testkit::stream(&conn, &three[1]).unwrap();
    let oldest = |group: &[String]| group.iter().map(|k| session_no(&conn, k)).min().unwrap();
    let survivors = [
        (format!("-g/{U3}.jsonl"), oldest(&three)),
        (format!("-q/{U4}.jsonl"), oldest(&identical)),
        (format!("-q/{U5}.jsonl"), oldest(&prefix)),
        (format!("-r/{U6}/subagents/agent-x.jsonl"), oldest(&linked)),
    ];
    drop(conn);

    bench.pass();

    let conn = bench.conn();
    let orphan = format!("-p/{U0}.jsonl");
    let sidecar = format!("-p/{U0}/subagents/agent-a.jsonl");
    let mut expected = vec![orphan.clone(), sidecar.clone()];
    expected.extend(kept.iter().cloned());
    expected.extend(survivors.iter().map(|(key, _)| key.clone()));
    expected.sort();
    let found = keys(&conn);
    assert_eq!(
        found.len(),
        expected.len(),
        "the session count is {} where one per transcript is {}: {found:#?}",
        found.len(),
        expected.len()
    );
    assert_eq!(found, expected, "one session per transcript");

    // Re-keyed in place: the same bytes, number, turn ids and watermark.
    assert_eq!(archived(&conn, &orphan), orphan_before);
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

    // Each merged transcript is one session holding its group's oldest number.
    for (key, oldest) in &survivors {
        assert_eq!(session_no(&conn, key), *oldest, "{key}");
    }
    // The prefix pair's survivor holds the longer twin's stream byte for byte,
    // in the same block rows, with its watermark and file.
    let survivor = archived(&conn, &survivors[2].0);
    assert_eq!(
        testkit::stream(&conn, &survivors[2].0).unwrap(),
        prefix_stream
    );
    assert_eq!(survivor.blocks, prefix_longer.blocks);
    assert_eq!(survivor.checksum, prefix_longer.checksum);
    assert_eq!(survivor.watermark, prefix_longer.watermark);
    assert_eq!(survivor.transcript_path, prefix[1]);
    // The three-member group's survivor holds the first longer member's.
    assert_eq!(
        testkit::stream(&conn, &survivors[0].0).unwrap(),
        three_stream
    );
    let continues_from: Option<String> = conn
        .query_row(
            "SELECT continues_from FROM session_meta WHERE session_key = ?1",
            [&survivors[1].0],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        continues_from.as_deref(),
        Some("b0000000-0000-4000-8000-000000000000")
    );

    // Not provably one stream: kept exactly as they were.
    for (key, was) in kept.iter().zip(&kept_before) {
        assert_eq!(&archived(&conn, key), was, "{key} changed");
    }

    let error = last_error(&conn).unwrap_or_default();
    assert!(
        error.contains("re-keyed 6 session(s) to their path below `projects` and merged 4 twin(s)"),
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

    // The derived rows the merge wrote are the ones a rebuild writes.
    let derived = testkit::query_set_json(&conn);
    terminus_core::reindex::reindex(&mut Store::open(&bench.data_dir).unwrap()).unwrap();
    assert_eq!(testkit::query_set_json(&conn), derived);

    // The steady state: a second pass over the unchanged files changes no row
    // and reports only the twins it still cannot merge.
    let rows_before = rows(&conn);
    bench.pass();
    assert_eq!(rows(&conn), rows_before, "the second pass changed a row");
    let second = last_error(&conn).unwrap_or_default();
    assert!(
        second.lines().all(|l| l.contains(": not merged into ")),
        "the second pass reported more than the unmerged twins: {second}"
    );
    assert_eq!(unmerged(&second), unmerged(&error));
}
