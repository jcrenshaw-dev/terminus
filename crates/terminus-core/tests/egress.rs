//! The shape rules recall applies when `redact_recall` is on (PRIV-03).
//!
//! Three claims. The rules take each secret shape out of plain text and leave
//! the text around it standing. The ingest path cannot reach them, because
//! ingest-time redaction is barred. And every sentinel planted in
//! `session-secrets.jsonl` and `session-redaction.jsonl` is gone from a real
//! recall projection, checked in the turn that holds it, with the marker of the
//! rule that caught it in that same turn.

use std::path::Path;

use terminus_core::config::REDACTED;
use terminus_core::observe::egress;

/// A key value nothing here can produce by accident.
const KEY: &str = "sk-TERMINUSEGRESS-71c4a8-do-not-log";

/// A text carrying a secret in each of the two shapes the plan names.
fn body() -> String {
    format!(
        "POST /v1/chat/completions HTTP/1.1\r\n\
         Authorization: Bearer {KEY}\r\n\
         Content-Type: application/json\r\n\
         \r\n\
         export OPENAI_API_KEY={KEY}\n"
    )
}

fn assert_withholds(rendered: &str, what: &str) {
    for fragment in [KEY, "TERMINUSEGRESS", "71c4a8"] {
        assert!(
            !rendered.contains(fragment),
            "{what} carries {fragment:?}: {rendered}"
        );
    }
}

/// Both shapes go on shape alone: nothing tells the filter the value.
#[test]
fn the_shape_rules_hold_without_a_resolved_credential() {
    let body = body();
    let sent = egress::for_model_context(&body);

    assert_withholds(&sent, "the filtered text");
    assert!(
        sent.contains("Authorization: [redacted]"),
        "the authorization header survived: {sent}"
    );
    assert!(
        sent.contains("OPENAI_API_KEY=[redacted]"),
        "the assignment survived: {sent}"
    );
}

// ---------------------------------------------------------------------------
// The rule set

/// A JSON pair keeps its structure: the name and the punctuation stay so the
/// reader can still see the shape of what was sent.
#[test]
fn a_json_pair_loses_its_value_and_keeps_its_name() {
    let scrubbed = egress::for_model_context("{\"model\":\"qwen3:8b\",\"api_key\":\"sk-abc123\"}");

    assert_eq!(
        scrubbed,
        format!("{{\"model\":\"qwen3:8b\",\"api_key\":\"{REDACTED}\"}}")
    );
}

/// A pretty-printed pair is the case the header rule must not eat: it would
/// take the trailing comma and leave a document that is not JSON at all.
#[test]
fn a_pretty_printed_json_pair_keeps_its_punctuation() {
    let scrubbed = egress::for_model_context("{\n  \"api_key\": \"sk-abc123\",\n  \"n\": 1\n}");

    assert_eq!(
        scrubbed,
        format!("{{\n  \"api_key\": \"{REDACTED}\",\n  \"n\": 1\n}}")
    );
}

/// A PEM private key goes whole; a certificate beside it does not, because a
/// certificate is public and a reader may need it.
#[test]
fn a_pem_private_key_goes_and_a_certificate_stays() {
    let text = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n\
                -----BEGIN RSA PRIVATE KEY-----\nSECRETLINES\n-----END RSA PRIVATE KEY-----\n\
                after\n";

    let scrubbed = egress::for_model_context(text);

    assert!(
        !scrubbed.contains("SECRETLINES"),
        "the private key survived: {scrubbed}"
    );
    assert!(
        scrubbed.contains(egress::REDACTED_PRIVATE_KEY),
        "the private key went silently: {scrubbed}"
    );
    assert!(
        scrubbed.contains("-----BEGIN CERTIFICATE-----"),
        "the certificate went too: {scrubbed}"
    );
    assert!(
        scrubbed.contains("after"),
        "everything after the block went with it: {scrubbed}"
    );
}

/// Nothing here changes text that carries no secret. A filter that mangled an
/// ordinary turn would make recall lie about what was said.
#[test]
fn text_with_nothing_secret_in_it_comes_back_unchanged() {
    let plain = "{\"model\":\"qwen3:8b\",\"messages\":[{\"role\":\"user\",\
                 \"content\":\"summarise session 3 = the one about sprockets\"}]}\n\
                 a line with a colon: and an = sign\n";

    assert_eq!(egress::for_model_context(plain), plain);
}

/// A multi-byte payload is cut at character boundaries or not at all.
#[test]
fn a_payload_holding_multibyte_text_survives_the_scan() {
    let text = "{\"note\":\"café — naïve ☕\",\"api_key\":\"sk-abc\"}";

    let scrubbed = egress::for_model_context(text);

    assert!(scrubbed.contains("café — naïve ☕"));
    assert!(!scrubbed.contains("sk-abc"));
}

// ---------------------------------------------------------------------------
// Egress only, never ingest

/// Ingest-time redaction is barred outright: it makes the store lossy and
/// destroys what it strips. The ingest write path must
/// therefore not be able to reach this module at all.
#[test]
fn nothing_in_the_ingest_path_names_the_egress_filter() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ingest");
    let mut checked = 0;
    for entry in std::fs::read_dir(&dir).expect("read src/ingest") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let code: String = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for banned in ["egress", "scrub", "redact"] {
            assert!(!code.contains(banned), "{} names {banned}", path.display());
        }
        checked += 1;
    }
    assert!(checked >= 3, "only {checked} files in {}", dir.display());
}

// ---------------------------------------------------------------------------
// Every planted shape, on a recall projection (PRIV-03)

// Gated per item and NOT with a file-level `#![cfg(feature = "testkit")]`: a
// file-level gate makes `cargo test --test egress` compile an empty binary and
// report green, which would retire the ingest-path scan above silently.

/// The transcript with one turn per credential shape the v0.1.1 rule set
/// catches.
#[cfg(feature = "testkit")]
const FIXTURE: &str = "session-secrets.jsonl";

/// The transcript with one turn per shape hardin's phase 2 adds: a PEM key, a
/// bare `sk-` key, a keyed value in each of the three subtrees the projection
/// drops the keys of, and the curly-quoted `NAME=` turn.
#[cfg(feature = "testkit")]
const REDACTION_FIXTURE: &str = "session-redaction.jsonl";

/// Every planted sentinel, with the fixture that holds it, a name for the
/// failure message and the marker of the only rule able to catch it.
///
/// One table across both fixtures rather than a test per shape, so a shape that
/// quietly stopped being planted can't be missed. The check is per turn: the
/// turn holding a sentinel holds nothing else any rule can catch, so the marker
/// found in that turn says which rule fired, even where rules 3, 4 and 5 share
/// [`REDACTED`]. The two curl spellings are the double-quoted forms of the first
/// and fourth rows, kept as their own rows because nothing caught them until
/// 2026-08-30.
///
/// A keyed row's value is taken in the window by the keyed-value step on the
/// parsed record and in a `recall_get` body by rule 4, so taking out either one
/// fails that row on one surface.
#[cfg(feature = "testkit")]
const PLANTED: &[(&str, &str, &str, &str)] = &[
    (
        FIXTURE,
        "an Authorization: Bearer header",
        "sk-VBEGRESS-authz-9f2",
        REDACTED,
    ),
    (
        FIXTURE,
        "a JSON \"password\" pair",
        "pw-VBEGRESS-mash-4d1",
        REDACTED,
    ),
    (
        FIXTURE,
        "a space-separated --token flag",
        "tk-VBEGRESS-flagv-7c3",
        egress::REDACTED_FLAG_VALUE,
    ),
    (
        FIXTURE,
        "a Cookie header",
        "sid-VBEGRESS-crumb-2b8",
        REDACTED,
    ),
    (
        FIXTURE,
        "a bare JWT",
        "eyJhbGciOiJIUzI1NiJ9.VBEGRESSjwt3e7",
        egress::REDACTED_JWT,
    ),
    (
        FIXTURE,
        "a GitHub token",
        "ghp_VBEGRESSgh0zq",
        egress::REDACTED_GITHUB_TOKEN,
    ),
    (
        FIXTURE,
        "a connection URL's userinfo",
        "dsn-VBEGRESS-pg-8a4",
        egress::REDACTED_URL_USERINFO,
    ),
    (
        FIXTURE,
        "a double-quoted Authorization header, the curl spelling",
        "sk-VBEGRESS-curlz-5e6",
        REDACTED,
    ),
    (
        FIXTURE,
        "a double-quoted Cookie header, the curl spelling",
        "sid-VBEGRESS-qcrumb-1f9",
        REDACTED,
    ),
    (
        REDACTION_FIXTURE,
        "a PEM private key block",
        "VBREDACT-pem-body-plainly-fake-6e1",
        egress::REDACTED_PRIVATE_KEY,
    ),
    (
        REDACTION_FIXTURE,
        "a bare sk- key in prose",
        "sk-VBREDACT-plainly-fake-bare-0e5",
        egress::REDACTED_SK_KEY,
    ),
    (
        REDACTION_FIXTURE,
        "a secret-named key in a tool_use input",
        "VBREDACT-input-plainly-fake-3c1",
        REDACTED,
    ),
    (
        REDACTION_FIXTURE,
        "a secret-named key in a toolUseResult",
        "VBREDACT-result-plainly-fake-7a2",
        REDACTED,
    ),
    (
        REDACTION_FIXTURE,
        "a secret-named key in an attachment",
        "VBREDACT-attach-plainly-fake-5d9",
        REDACTED,
    ),
    (
        REDACTION_FIXTURE,
        "a NAME=value pair inside curly quotes",
        "token=abc",
        REDACTED,
    ),
];

/// The `Cookie` turn's prose, which follows the value on the same line.
#[cfg(feature = "testkit")]
const AFTER_THE_COOKIE: &str = "the retry after that answered 204 with an empty body";

/// `fixture` ingested for real into a store of its own, then read by `read`
/// with the knob off and with it on: `(raw, redacted)`.
///
/// The knob goes through a real `terminus.toml`, the only way the binary can
/// turn it on.
#[cfg(feature = "testkit")]
fn read_both_ways<T>(
    fixture: &str,
    read: impl Fn(&rusqlite::Connection, &terminus_core::config::Config) -> T,
) -> (T, T) {
    use terminus_core::config::{Config, CONFIG_FILE_NAME};
    use terminus_core::store::DB_FILE_NAME;
    use terminus_core::{ingest, testkit};

    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().join("data");
    let work = dir.path().join("work");
    std::fs::create_dir_all(&work).unwrap();

    let path = testkit::copy_rooted_fixture_into(fixture, &work, &dir.path().join("root"));
    match ingest::run(&data_dir, &path).unwrap() {
        ingest::Outcome::Committed(_) => {}
        other => panic!("{fixture}: {other:?}"),
    }

    let conn = rusqlite::Connection::open(data_dir.join(DB_FILE_NAME)).unwrap();
    let with = |on: bool| -> T {
        let config_dir = work.join(format!("config-{on}"));
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join(CONFIG_FILE_NAME),
            format!("[privacy]\nredact_recall = {on}\n"),
        )
        .unwrap();
        read(&conn, &Config::load_from(&config_dir).unwrap())
    };
    (with(false), with(true))
}

/// `fixture` read back as one context window, turn by turn.
#[cfg(feature = "testkit")]
fn projected(fixture: &str) -> (Vec<String>, Vec<String>) {
    use terminus_core::recall::{context, Scope, MAX_CONTEXT_SIDE};

    read_both_ways(fixture, |conn, config| {
        let first: i64 = conn
            .query_row("SELECT id FROM turns ORDER BY turn_seq LIMIT 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        let window =
            context::window(conn, config, &Scope::Everything, first, 0, MAX_CONTEXT_SIDE).unwrap();
        assert!(
            window.at_session_start && window.at_session_end,
            "the window does not cover the whole of {fixture}: {window:?}"
        );
        window.turns.into_iter().map(|turn| turn.text).collect()
    })
}

/// Every turn of `fixture` read back through `recall_get`, one body each.
///
/// A body is the archived JSON line, so the `"password"` pair sits behind
/// escaped quotes here, unlike in the window.
#[cfg(feature = "testkit")]
fn got(fixture: &str) -> (Vec<String>, Vec<String>) {
    use terminus_core::recall::{get, Scope};

    read_both_ways(fixture, |conn, config| {
        let ids: Vec<i64> = conn
            .prepare("SELECT id FROM turns ORDER BY turn_seq")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let fetched = get::records(conn, config, &Scope::Everything, &ids).unwrap();
        assert!(
            fetched.absent.is_empty() && fetched.records.len() == ids.len(),
            "not every turn of {fixture} came back: {:?}",
            fetched.absent
        );
        fetched
            .records
            .into_iter()
            .map(|record| String::from_utf8(record.body.expect("a body")).unwrap())
            .collect()
    })
}

/// Every [`PLANTED`] sentinel, checked in the turn that holds it: present in
/// the raw read of that turn, gone from the redacted read of the same turn, and
/// that turn carrying the row's marker.
///
/// `read` gives one fixture's turns as `(raw, redacted)`, in the same order.
#[cfg(feature = "testkit")]
fn assert_every_planted_shape_caught(
    read: impl Fn(&str) -> (Vec<String>, Vec<String>),
    surface: &str,
) {
    for fixture in [FIXTURE, REDACTION_FIXTURE] {
        let (raw, redacted) = read(fixture);
        assert_eq!(
            raw.len(),
            redacted.len(),
            "{fixture} read back a different number of turns in {surface} with the knob on"
        );
        for (_, what, sentinel, marker) in PLANTED.iter().filter(|row| row.0 == fixture) {
            let turns: Vec<usize> = raw
                .iter()
                .enumerate()
                .filter(|(_, text)| text.contains(sentinel))
                .map(|(at, _)| at)
                .collect();
            // A shape the fixture stopped planting would pass the checks
            // below for the wrong reason.
            assert!(
                !turns.is_empty(),
                "{what} is not planted in {fixture} ({sentinel}), so its absence proves nothing"
            );
            for at in turns {
                let turn = &redacted[at];
                assert!(
                    !turn.contains(sentinel),
                    "{what} survived into {surface}, turn {at} of {fixture} ({sentinel}): {turn}"
                );
                assert!(
                    turn.contains(marker),
                    "{what} went from {surface}, turn {at} of {fixture}, without {marker}, \
                     so a reader can't tell which rule took it: {turn}"
                );
            }
        }
    }
}

/// Each planted shape is gone from what recall hands the model, and the rule
/// that took it left its marker in the same turn.
#[cfg(feature = "testkit")]
#[test]
fn every_planted_shape_is_gone_and_its_rule_named_itself() {
    assert_every_planted_shape_caught(projected, "the window");
}

/// The same for a `recall_get` body, where the JSON pair is escaped.
#[cfg(feature = "testkit")]
#[test]
fn every_planted_shape_is_gone_from_a_recall_get_body() {
    assert_every_planted_shape_caught(got, "recall_get");
}

/// The `Cookie` match is bounded: the value goes and the rest of its turn
/// stays.
#[cfg(feature = "testkit")]
#[test]
fn a_mid_line_cookie_takes_its_value_and_leaves_the_turn_standing() {
    let (raw, redacted) = projected(FIXTURE);
    let at = raw
        .iter()
        .position(|text| text.contains("sid-VBEGRESS-crumb-2b8"))
        .unwrap_or_else(|| panic!("no turn of {FIXTURE} carries the cookie"));
    let turn = &redacted[at];

    assert!(
        !turn.contains("sid-VBEGRESS-crumb-2b8"),
        "the cookie survived: {turn}"
    );
    assert!(
        turn.contains(AFTER_THE_COOKIE),
        "the match ran past the value and took the rest of the turn: {turn}"
    );
}
