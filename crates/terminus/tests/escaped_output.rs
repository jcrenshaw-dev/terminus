//! Every command's human output escapes archived controls, and the three
//! outputs that are the bytes themselves do not (#72).
//!
//! Spawns the binary against a store planted with ESC and BEL in a session
//! key, a project path, a branch and a run's notes and error. What a person
//! reads must hold neither byte; `--json`, the hook's document and `show`'s
//! record body must.
//!
//! **Why the raw checks use DEL and U+009B.** serde_json spells ESC and BEL
//! `\u001b` and `\u0007` whatever happens after it, so a raw check on them
//! would pass with the document wrongly escaped. DEL and C1 are what it writes
//! raw, so only they tell a raw document from an escaped one.
//!
//! The harness is a copy of `tests/cli.rs`'s: this repository copies
//! harnesses rather than sharing a module.

use std::path::PathBuf;
use std::process::{Command, Output};

use rusqlite::Connection;
use serde_json::Value;
use terminus_core::store::DB_FILE_NAME;

/// The sidecar's file name, which is its session key's last component.
const SIDECAR: &str = "agent-\u{1b}\u{07}.jsonl";

/// The project directory's name: ESC and BEL, then DEL and C1.
const PROJECT_DIR: &str = "planted\u{1b}\u{07}\u{7f}\u{9b}project";

/// The branch: the same four.
const BRANCH: &str = "phase\u{1b}\u{07}\u{7f}\u{9b}seven";

/// What the planted run row's `notes` and `error` say.
const RUN_NOTE: &str = "note \u{1b}[31m\u{07} end";
const RUN_ERROR: &str = "error \u{1b}]52;c;\u{07} end";

/// Forty days: past a 30-day retention window and past the idle rule, so the
/// session is both selectable and finalized.
const SECONDS_AGO: i64 = 40 * 86_400;

/// The text of the one turn `show` is pointed at.
const TAIL: &str = "tail\u{7f}\u{9b}end";

/// Every directory a spawned `terminus` may touch, all of them temporary.
struct Bench {
    _dir: tempfile::TempDir,
    data_dir: PathBuf,
    config_dir: PathBuf,
    claude_dir: PathBuf,
    root: PathBuf,
}

fn bench() -> Bench {
    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let config_dir = dir.path().join("config");
    let claude_dir = dir.path().join("claude");
    let root = dir.path().join("root");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(claude_dir.join("projects")).unwrap();
    std::fs::create_dir_all(root.join(PROJECT_DIR)).unwrap();
    Bench {
        _dir: dir,
        data_dir,
        config_dir,
        claude_dir,
        root,
    }
}

impl Bench {
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_terminus"))
            .args(args)
            .env("TERMINUS_DATA_DIR", &self.data_dir)
            .env("TERMINUS_CONFIG_DIR", &self.config_dir)
            .env("CLAUDE_CONFIG_DIR", &self.claude_dir)
            .output()
            .expect("spawn terminus")
    }

    fn conn(&self) -> Connection {
        Connection::open(self.data_dir.join(DB_FILE_NAME)).unwrap()
    }

    /// Where the planted sidecar sits in the transcript tree.
    fn sidecar(&self) -> PathBuf {
        self.claude_dir
            .join("projects")
            .join("project-alpha")
            .join(SIDECAR)
    }

    /// Plant the sidecar and archive it with a bare `ingest`, then plant the
    /// run row's notes and error.
    fn planted() -> Bench {
        let bench = bench();
        let cwd = bench.root.join(PROJECT_DIR).to_string_lossy().into_owned();
        let record = |n: usize, seconds_ago: i64, kind: &str, content: Value| {
            serde_json::json!({
                "parentUuid": null,
                "isSidechain": true,
                "cwd": cwd,
                "sessionId": "planted-session",
                "gitBranch": BRANCH,
                "type": kind,
                "uuid": format!("planted-{n}"),
                "timestamp": ts(seconds_ago),
                "message": {"role": if kind == "user" { "user" } else { "assistant" },
                            "model": "claude-opus-5",
                            "content": content},
            })
        };
        let lines: Vec<String> = [
            record(
                0,
                SECONDS_AGO,
                "user",
                serde_json::json!([{"type": "text", "text": "copy the greeting"}]),
            ),
            record(
                1,
                SECONDS_AGO - 1,
                "assistant",
                serde_json::json!([{"type": "text", "text": TAIL}]),
            ),
            record(
                2,
                SECONDS_AGO - 2,
                "assistant",
                serde_json::json!([{"type": "text", "text": "the greeting is copied"}]),
            ),
        ]
        .iter()
        .map(Value::to_string)
        .collect();
        let path = bench.sidecar();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();

        let out = bench.run(&["ingest"]);
        assert!(out.status.success(), "ingest: {}", stderr(&out));
        let changed = bench
            .conn()
            .execute(
                "UPDATE runs SET notes = ?1, error = ?2 WHERE id = (SELECT MAX(id) FROM runs)",
                [RUN_NOTE, RUN_ERROR],
            )
            .unwrap();
        assert_eq!(changed, 1, "the premise: a run row to plant");
        bench
    }

    /// The one session's key, as the store spells it.
    fn session_key(&self) -> String {
        self.conn()
            .query_row("SELECT session_key FROM session_meta", [], |r| r.get(0))
            .unwrap()
    }
}

/// A UTC timestamp `seconds_ago` from now, in the shape a transcript writes.
fn ts(seconds_ago: i64) -> String {
    Connection::open_in_memory()
        .unwrap()
        .query_row(
            "SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1)",
            [format!("-{seconds_ago} seconds")],
            |r| r.get(0),
        )
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Whether `bytes` holds an ESC or a BEL.
fn drives_a_terminal(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| *b == 0x1b || *b == 0x07)
}

/// Whether `haystack` holds `needle` as a byte string.
fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// A human run's stdout holds no ESC or BEL and spells both where they were.
fn assert_escaped_stdout(label: &str, output: &Output) {
    assert!(
        !drives_a_terminal(&output.stdout),
        "{label} wrote ESC or BEL to stdout: {:?}",
        stdout(output)
    );
    let text = stdout(output);
    assert!(
        text.contains("\\u001b") && text.contains("\\u0007"),
        "{label} does not show the escapes: {text:?}"
    );
}

#[test]
fn the_human_output_of_the_listing_commands_escapes_archived_controls() {
    let bench = Bench::planted();
    for args in [
        &["sessions", "--project", "*"][..],
        &["usage"],
        &["status"],
    ] {
        let out = bench.run(args);
        assert_eq!(out.status.code(), Some(0), "{args:?}: {}", stderr(&out));
        assert_escaped_stdout(&args.join(" "), &out);
    }
}

#[test]
fn retention_dry_run_escapes_the_key_it_names() {
    let bench = Bench::planted();
    std::fs::write(
        bench.config_dir.join("terminus.toml"),
        "[retention]\naction = \"evict\"\nage_days = 30\n",
    )
    .unwrap();
    let out = bench.run(&["retention", "--dry-run"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_escaped_stdout("retention --dry-run", &out);
}

#[test]
fn verify_escapes_the_key_of_the_session_it_fails() {
    let bench = Bench::planted();
    terminus_core::testkit::damage(&bench.conn(), &bench.session_key());
    let out = bench.run(&["verify"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_escaped_stdout("verify", &out);
}

#[test]
fn sessions_json_carries_the_raw_bytes_the_human_output_escapes() {
    let bench = Bench::planted();
    let out = bench.run(&["sessions", "--project", "*", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        holds(&out.stdout, &[0xC2, 0x9B]) && out.stdout.contains(&0x7F),
        "the document was escaped: {:?}",
        stdout(&out)
    );
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    let text = value.to_string();
    // serde_json spells ESC and BEL in its own text form, so decode before
    // looking: the values themselves must hold the characters.
    let sessions = value["data"]["sessions"].as_array().unwrap();
    assert!(
        sessions.iter().any(|s| {
            s.to_string().contains("\\u001b") && s.to_string().contains("\\u0007")
        }),
        "{text}"
    );
    let planted = sessions
        .iter()
        .flat_map(|s| s.as_object().unwrap().values())
        .filter_map(Value::as_str)
        .any(|v| v.contains('\u{1b}') && v.contains('\u{07}'));
    assert!(planted, "no value decodes to a string holding ESC and BEL: {text}");
}

#[test]
fn doctor_escapes_the_last_run_in_its_report_and_keeps_it_in_json() {
    let bench = Bench::planted();
    let out = bench.run(&["doctor"]);
    assert_escaped_stdout("doctor", &out);

    let out = bench.run(&["doctor", "--json"]);
    let value: Value = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("doctor --json is not a document ({e}): {}", stdout(&out)));
    let last_run = &value["data"]["last_run"];
    let finding = last_run["finding"]
        .as_str()
        .unwrap_or_else(|| panic!("no last_run finding: {value}"));
    assert!(
        finding.contains('\u{1b}') && finding.contains('\u{07}'),
        "the finding was escaped on its way into the document: {finding:?}"
    );
}

#[test]
fn show_writes_the_record_body_raw_under_an_escaped_header() {
    let bench = Bench::planted();
    let id: i64 = bench
        .conn()
        .query_row("SELECT id FROM turns WHERE turn_seq = 1", [], |r| r.get(0))
        .unwrap();
    let out = bench.run(&["show", &id.to_string(), "--project", "*", "--before", "1"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        holds(&out.stdout, &[b't', b'a', b'i', b'l', 0x7F, 0xC2, 0x9B, b'e']),
        "the body was not written raw: {:?}",
        stdout(&out)
    );
    let header = out.stdout.split(|b| *b == b'\n').next().unwrap();
    assert!(
        !drives_a_terminal(header),
        "the header wrote ESC or BEL: {:?}",
        String::from_utf8_lossy(header)
    );
    let header = String::from_utf8_lossy(header);
    assert!(
        header.contains("\\u001b") && header.contains("\\u0007"),
        "the header does not show the escapes: {header}"
    );
}
