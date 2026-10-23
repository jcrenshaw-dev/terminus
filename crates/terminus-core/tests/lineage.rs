//! Session lineage through a real pass.
//!
//! One claim shapes this file: after a pass, the count of rows whose
//! `continues_from` equals their own session id must be zero while the count of
//! non-null `continues_from` values must not be.

#![cfg(feature = "testkit")]

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use terminus_core::config::Config;
use terminus_core::ingest::pass::{self, PassOutcome};
use terminus_core::store::DB_FILE_NAME;

const PROJECT: &str = "-data-code-verbatim";

/// A Claude config directory whose `projects` tree the pass walks, plus a data
/// directory. No test process may resolve a real transcript root.
struct Tree {
    _dir: tempfile::TempDir,
    data_dir: PathBuf,
    claude_dir: PathBuf,
}

fn tree() -> Tree {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let claude_dir = dir.path().join("claude");
    std::fs::create_dir_all(claude_dir.join("projects")).unwrap();
    Tree {
        _dir: dir,
        data_dir,
        claude_dir,
    }
}

impl Tree {
    fn projects(&self) -> PathBuf {
        self.claude_dir.join("projects")
    }

    /// Write a transcript at `<projects>/<PROJECT>/<name>`, creating whatever
    /// directories the name implies.
    fn write(&self, name: &str, body: &str) -> PathBuf {
        let path = self.projects().join(PROJECT).join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        path.canonicalize().unwrap()
    }

    /// A top-level transcript reporting `own` as its session id and, when
    /// `foreign` is set, that session as the one it continues from.
    fn transcript(&self, n: u8, own: &str, foreign: Option<&str>) -> PathBuf {
        self.write(&format!("{}.jsonl", uuid(n)), &record(n, own, foreign))
    }

    fn pass(&self) -> pass::Summary {
        let config = Config::from_parts(vec![self.claude_dir.clone()], Vec::new());
        match pass::run_with(&self.data_dir, &config).unwrap() {
            PassOutcome::Ran(summary) => summary,
            PassOutcome::LockHeld => panic!("nothing else holds the lock"),
        }
    }

    fn conn(&self) -> Connection {
        Connection::open(self.data_dir.join(DB_FILE_NAME)).unwrap()
    }

    fn continues_from(&self, path: &Path) -> Option<String> {
        self.conn()
            .query_row(
                "SELECT continues_from FROM session_meta WHERE session_key = ?1",
                [key(path)],
                |r| r.get(0),
            )
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }
}

fn uuid(n: u8) -> String {
    format!("{n:08x}-1111-4111-8111-111111111111")
}

/// The key a transcript is archived under: its path below the project folder.
fn key(path: &Path) -> String {
    terminus_core::ingest::path_key(path).unwrap()
}

fn session(n: u8) -> String {
    format!("{n:08x}-2222-4222-8222-222222222222")
}

/// One turn record. `own` is `sessionId`, `foreign` is the snake_case
/// `session_id` - the file-level lineage signal, and a different field.
fn record(n: u8, own: &str, foreign: Option<&str>) -> String {
    let foreign = match foreign {
        Some(id) => format!("\"session_id\":\"{id}\","),
        None => String::new(),
    };
    format!(
        "{{\"type\":\"user\",\"uuid\":\"cccccccc-0000-4000-8000-0000000000{n:02x}\",\
          \"timestamp\":\"2026-08-12T10:00:{n:02}.000Z\",\"sessionId\":\"{own}\",{foreign}\
          \"cwd\":\"/data/code/verbatim\",\
          \"message\":{{\"role\":\"user\",\"content\":[{{\"type\":\"text\",\"text\":\"hi\"}}]}}}}\n"
    )
}

/// A session never continues from itself. 253 of the 433 real transcripts
/// carrying a `session_id` carry only their own, so an uncompared read makes
/// one in five sessions continue from itself.
#[test]
fn a_transcript_naming_only_its_own_session_id_continues_from_nothing() {
    let tree = tree();
    let own = session(1);
    let path = tree.transcript(1, &own, Some(&own));

    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&path), None);
}

/// 11 real files carry two distinct ids. The foreign one is the link, and byte
/// order decides among survivors.
#[test]
fn a_transcript_carrying_both_ids_records_the_foreign_one() {
    let tree = tree();
    let own = session(1);
    let other = session(9);

    // Its own id first in byte order, the foreign one second: the rule is a
    // comparison, not a position.
    let path = tree.write(
        &format!("{}.jsonl", uuid(1)),
        &format!(
            "{}{}",
            record(1, &own, Some(&own)),
            record(2, &own, Some(&other))
        ),
    );

    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&path), Some(other));
}

/// Continuation is a fan-out. Session `ebd78c2b` is named as predecessor
/// by four separate later transcripts in one real project, so nothing may
/// assume a single successor.
#[test]
fn one_predecessor_named_by_three_successors_produces_three_links() {
    let tree = tree();
    let predecessor = session(1);
    tree.transcript(1, &predecessor, None);
    let successors: Vec<PathBuf> = (2..=4)
        .map(|n| tree.transcript(n, &session(n), Some(&predecessor)))
        .collect();

    assert_eq!(tree.pass().files_committed, 4);
    for path in &successors {
        assert_eq!(tree.continues_from(path).as_deref(), Some(&*predecessor));
    }

    let links: i64 = tree
        .conn()
        .query_row(
            "SELECT count(*) FROM session_meta WHERE continues_from = ?1",
            [&predecessor],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        links, 3,
        "a predecessor may be named by any number of files"
    );
}

/// A predecessor need not be archived: 2 of the 173 real files naming a foreign
/// session name one that was aged out by `cleanupPeriodDays` and is not on disk
/// at all. No foreign key, no ingest-order requirement.
#[test]
fn a_predecessor_that_was_never_ingested_is_recorded_anyway() {
    let tree = tree();
    let missing = session(200);
    let path = tree.transcript(1, &session(1), Some(&missing));

    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&path), Some(missing.clone()));

    let known: i64 = tree
        .conn()
        .query_row(
            "SELECT count(*) FROM session_meta WHERE session_id = ?1",
            [&missing],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(known, 0, "the link names a session the store does not hold");
}

/// Over a tree: zero self-links, and more than zero links.
#[test]
fn over_a_tree_no_session_continues_from_itself_and_some_continue() {
    let tree = tree();
    let first = session(1);
    tree.transcript(1, &first, Some(&first)); // names only itself
    tree.transcript(2, &session(2), Some(&first)); // a real continuation
    tree.transcript(3, &session(3), None); // no signal at all
    tree.transcript(4, &session(4), Some(&first)); // the fan-out

    assert_eq!(tree.pass().files_committed, 4);

    let conn = tree.conn();
    let self_links: i64 = conn
        .query_row(
            "SELECT count(*) FROM session_meta WHERE continues_from = session_id",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let links: i64 = conn
        .query_row(
            "SELECT count(*) FROM session_meta WHERE continues_from IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap();

    assert_eq!(self_links, 0, "no row may continue from its own session id");
    assert_eq!(links, 2);
}

/// The `parentUuid` fallback stays exactly as it was: message-level
/// threading, resolved through `turns` to the session holding that record, and
/// used only when no foreign `session_id` is present.
#[test]
fn the_parent_uuid_fallback_still_links_a_fork() {
    let tree = tree();
    let origin = session(1);
    tree.transcript(1, &origin, None);
    assert_eq!(tree.pass().files_committed, 1);

    let forked = tree.write(
        &format!("{}.jsonl", uuid(2)),
        &format!(
            "{{\"type\":\"user\",\"uuid\":\"dddddddd-0000-4000-8000-000000000001\",\
              \"parentUuid\":\"cccccccc-0000-4000-8000-000000000001\",\
              \"timestamp\":\"2026-08-12T11:00:00.000Z\",\"sessionId\":\"{}\"}}\n",
            session(2)
        ),
    );

    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(
        tree.continues_from(&forked),
        Some(origin),
        "the fallback must name a session, not a message uuid"
    );
}

impl Tree {
    /// A later session whose first record names the message of `record(1, ..)`
    /// as its `parentUuid`, with its own `sessionId` and, when `foreign` is
    /// set, a `session_id` naming another session.
    fn continuing(&self, foreign: Option<&str>) -> PathBuf {
        let foreign = match foreign {
            Some(id) => format!("\"session_id\":\"{id}\","),
            None => String::new(),
        };
        self.write(
            &format!("{}.jsonl", uuid(2)),
            &format!(
                "{{\"type\":\"user\",\"uuid\":\"dddddddd-0000-4000-8000-000000000001\",\
                  \"parentUuid\":\"cccccccc-0000-4000-8000-000000000001\",\
                  \"timestamp\":\"2026-08-12T11:00:00.000Z\",\"sessionId\":\"{}\",{foreign}\
                  \"cwd\":\"/data/code/verbatim\"}}\n",
                session(2)
            ),
        )
    }
}

/// A session archived before its predecessor gets its link on the next pass,
/// though its own file did not change and was not committed again (#11).
#[test]
fn a_continuing_session_is_linked_once_its_predecessor_is_archived() {
    let tree = tree();
    let later = tree.continuing(None);

    // Pass one: the later session alone, so the parent message resolves to nothing.
    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&later), None);

    // Pass two commits only the predecessor, so the later session was not
    // ingested again and only the sweep can have linked it.
    tree.transcript(1, &session(1), None);
    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&later), Some(session(1)));
}

/// The sweep fills NULLs only: a foreign `session_id` recorded at ingest wins
/// over the `parentUuid` the sweep would resolve (#11).
#[test]
fn a_link_already_recorded_survives_the_sweep() {
    let tree = tree();
    let foreign = session(9);
    let later = tree.continuing(Some(&foreign));

    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&later), Some(foreign.clone()));

    tree.transcript(1, &session(1), None);
    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.continues_from(&later), Some(foreign));
}

// ---------------------------------------------------------------------------
// A sidecar's parent, which comes from the path and never the records.
// ---------------------------------------------------------------------------

impl Tree {
    /// A sidecar reporting `own` - its PARENT's session id, which is what all
    /// 818 real `agent-*.jsonl` files do.
    fn sidecar(&self, name: &str, n: u8, own: &str) -> PathBuf {
        self.write(
            name,
            &record(n, own, None).replace(
                "\"type\":\"user\"",
                "\"type\":\"user\",\"isSidechain\":true",
            ),
        )
    }

    fn parent_of(&self, path: &Path) -> Option<String> {
        self.conn()
            .query_row(
                "SELECT parent_session_key FROM session_meta WHERE session_key = ?1",
                [key(path)],
                |r| r.get(0),
            )
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn session_id_of(&self, path: &Path) -> Option<String> {
        self.conn()
            .query_row(
                "SELECT session_id FROM session_meta WHERE session_key = ?1",
                [key(path)],
                |r| r.get(0),
            )
            .unwrap()
    }
}

/// Both sidecar depths link to the top-level transcript whose directory they
/// sit under, and the top-level transcript's own parent stays null.
///
/// 781 real sidecars sit at `<project>/<sessionId>/subagents/` and 41 two
/// levels deeper under `subagents/workflows/wf_*/`.
#[test]
fn both_sidecar_depths_carry_their_parents_session_key() {
    let tree = tree();
    let own = session(1);
    let parent = tree.transcript(1, &own, None);
    let shallow = tree.sidecar(&format!("{}/subagents/agent-a.jsonl", uuid(1)), 2, &own);
    let deep = tree.sidecar(
        &format!("{}/subagents/workflows/wf_x/agent-deep.jsonl", uuid(1)),
        3,
        &own,
    );

    assert_eq!(tree.pass().files_committed, 3);

    let expected = key(&parent);
    assert_eq!(tree.parent_of(&shallow).as_deref(), Some(&*expected));
    assert_eq!(tree.parent_of(&deep).as_deref(), Some(&*expected));
    assert_eq!(
        tree.parent_of(&parent),
        None,
        "a top-level transcript has no parent session"
    );

    // The link cannot have come from the records: the sidecar reports exactly
    // the same session id as its parent, which is the self-link case.
    assert_eq!(tree.session_id_of(&shallow), tree.session_id_of(&parent));
    assert_eq!(tree.session_id_of(&deep), tree.session_id_of(&parent));
}

/// Ingest order does not matter: the key is stored whether or not the
/// parent transcript has been ingested, or exists at all.
#[test]
fn a_sidecar_ingested_before_its_parent_records_the_same_key() {
    let tree = tree();
    let own = session(1);
    let orphan = tree.sidecar(&format!("{}/subagents/agent-a.jsonl", uuid(1)), 2, &own);

    // Pass one: only the sidecar exists.
    assert_eq!(tree.pass().files_committed, 1);
    let expected = key(&tree
        .projects()
        .join(PROJECT)
        .canonicalize()
        .unwrap()
        .join(format!("{}.jsonl", uuid(1))));
    assert_eq!(tree.parent_of(&orphan).as_deref(), Some(&*expected));

    let sessions: i64 = tree
        .conn()
        .query_row("SELECT count(*) FROM sessions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(sessions, 1, "the parent is named, not required");

    // Pass two: the parent appears, and it is the key that was already stored.
    let parent = tree.transcript(1, &own, None);
    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(key(&parent), expected);
    assert_eq!(tree.parent_of(&orphan).as_deref(), Some(&*expected));
}

// ---------------------------------------------------------------------------
// The sidecar's `agent-*.meta.json`, stored unparsed.
// ---------------------------------------------------------------------------

impl Tree {
    /// Copy a fixture into the tree under a name of the caller's choosing.
    fn place(&self, name: &str, fixture: &str) -> PathBuf {
        let path = self.projects().join(PROJECT).join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::copy(terminus_core::testkit::fixture_path(fixture), &path).unwrap();
        path.canonicalize().unwrap()
    }

    fn agent_meta_of(&self, path: &Path) -> Option<Vec<u8>> {
        self.conn()
            .query_row(
                "SELECT agent_meta FROM session_meta WHERE session_key = ?1",
                [key(path)],
                |r| r.get(0),
            )
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    fn blob_of(&self, path: &Path) -> Vec<u8> {
        terminus_core::testkit::stream(&self.conn(), &key(path)).unwrap()
    }
}

/// The meta file's bytes land in `session_meta.agent_meta` unparsed, the meta
/// file itself never becomes a session, and the blob still holds only
/// transcript bytes.
#[test]
fn a_sidecars_meta_file_is_stored_verbatim_and_never_becomes_a_session() {
    let tree = tree();
    let dir = format!("{}/subagents", uuid(1));
    let sidecar = tree.place(
        &format!("{dir}/agent-alpha.jsonl"),
        "subagents/agent-alpha.jsonl",
    );
    let meta = tree.place(
        &format!("{dir}/agent-alpha.meta.json"),
        terminus_core::testkit::AGENT_META_FIXTURE,
    );

    let summary = tree.pass();
    assert_eq!(
        (summary.files_walked, summary.files_committed),
        (1, 1),
        "the meta file is not a transcript and is not walked"
    );

    let expected =
        terminus_core::testkit::fixture_bytes(terminus_core::testkit::AGENT_META_FIXTURE);
    assert_eq!(tree.agent_meta_of(&sidecar), Some(expected));

    let rows: i64 = tree
        .conn()
        .query_row(
            "SELECT count(*) FROM sessions WHERE session_key = ?1",
            [meta.to_str().unwrap()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 0, "the meta file must never become a session");

    assert_eq!(
        tree.blob_of(&sidecar),
        terminus_core::testkit::fixture_bytes("subagents/agent-alpha.jsonl"),
        "the blob holds the transcript bytes and nothing else"
    );

    // A second pass rereads nothing: the bytes never change, and 818 real
    // sidecars would otherwise be reopened on every hook-spawned run.
    terminus_core::discover::opened::reset();
    assert_eq!(tree.pass().files_committed, 0);
    assert!(
        terminus_core::discover::opened::under(&meta).is_empty(),
        "a stored meta file must not be reopened: {:?}",
        terminus_core::discover::opened::paths()
    );
}

/// Coverage is 816 meta files against 818 sidecars, so a missing one is normal:
/// null column, no error, nothing recorded.
#[test]
fn a_sidecar_with_no_meta_file_leaves_the_column_null() {
    let tree = tree();
    let sidecar = tree.place(
        &format!("{}/subagents/agent-alpha.jsonl", uuid(1)),
        "subagents/agent-alpha.jsonl",
    );

    let summary = tree.pass();
    assert_eq!(summary.files_committed, 1);
    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    assert_eq!(tree.agent_meta_of(&sidecar), None);
}

/// A top-level transcript is not a sidecar: nothing looks for a meta file
/// beside it, and the column stays null.
#[test]
fn a_top_level_transcript_has_no_agent_meta() {
    let tree = tree();
    let path = tree.transcript(1, &session(1), None);
    assert_eq!(tree.pass().files_committed, 1);
    assert_eq!(tree.agent_meta_of(&path), None);
}
