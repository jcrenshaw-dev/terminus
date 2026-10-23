//! No comment cites a planning tag.
//!
//! The planning files that gave requirement and decision ids their meaning are
//! gone, so a comment that cites one points a reader at nothing. This test
//! fails on any such id in a Rust comment, a TOML comment, an SQL comment in
//! the schema, or a line of the markdown docs. A comment states the rule it
//! explains instead.
//!
//! The source holds no literal tag, because this file sits inside a root it
//! scans: the families are kept as bare prefixes with the digits checked in
//! code, and every test input that carries a tag is assembled at run time.
//! Not gated on `testkit`, so `cargo test -p terminus-core` always runs it.

use std::path::{Path, PathBuf};

/// Where a piece of text came from. Code comments refuse a phase number;
/// markdown prose may name one.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Code,
    Markdown,
}

/// Real names of the generic shape, which are not tags.
const ALLOWED: [&str; 1] = ["UTF-16"];

fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The length of the run of bytes from `at` that `pred` accepts.
fn run(bytes: &[u8], at: usize, pred: fn(&u8) -> bool) -> usize {
    bytes[at.min(bytes.len())..]
        .iter()
        .take_while(|b| pred(b))
        .count()
}

/// Whether `at` ends a word: the end of the text or a byte that is not part of
/// one.
fn ends_word(bytes: &[u8], at: usize) -> bool {
    at >= bytes.len() || !is_word(bytes[at])
}

/// The end of a digit run starting at `at`, when it holds `min` to `max`
/// digits and ends a word.
fn digits_end(bytes: &[u8], at: usize, min: usize, max: usize) -> Option<usize> {
    let n = run(bytes, at, u8::is_ascii_digit);
    (n >= min && n <= max && ends_word(bytes, at + n)).then_some(at + n)
}

/// The tag starting at `at`, which already starts a word, and where it ends.
fn tag_at(bytes: &[u8], at: usize, kind: Kind) -> Option<usize> {
    let rest = &bytes[at..];
    // Two to six capitals, a hyphen and exactly two digits.
    let caps = run(bytes, at, u8::is_ascii_uppercase);
    if (2..=6).contains(&caps) && rest.get(caps) == Some(&b'-') {
        if let Some(end) = digits_end(bytes, at + caps + 1, 2, 2) {
            return Some(end);
        }
    }
    if rest.starts_with(b"D-") {
        if let Some(end) = digits_end(bytes, at + 2, 2, 2) {
            return Some(end);
        }
    }
    if rest.starts_with(b"AC") {
        if let Some(end) = digits_end(bytes, at + 2, 1, usize::MAX) {
            return Some(end);
        }
    }
    if rest.starts_with(b"PLAN-") {
        if let Some(end) = digits_end(bytes, at + 5, 1, usize::MAX) {
            return Some(end);
        }
    }
    if kind == Kind::Code && rest.len() > 6 && rest[..6].eq_ignore_ascii_case(b"phase ") {
        if let Some(end) = digits_end(bytes, at + 6, 1, usize::MAX) {
            return Some(end);
        }
    }
    None
}

/// Every tag in one piece of text, in order.
fn tags(text: &str, kind: Kind) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if at == 0 || !is_word(bytes[at - 1]) {
            if let Some(end) = tag_at(bytes, at, kind) {
                let tag = &text[at..end];
                if !ALLOWED.contains(&tag) {
                    found.push(tag.to_string());
                }
                at = end;
                continue;
            }
        }
        at += 1;
    }
    found
}

/// One piece of comment text, with its 1-based line.
#[derive(Debug, PartialEq)]
struct Piece {
    line: usize,
    text: String,
    kind: Kind,
}

/// Every line of a markdown file is prose a reader sees.
fn markdown_pieces(source: &str) -> Vec<Piece> {
    let lines = source.lines().enumerate();
    lines
        .map(|(n, line)| Piece {
            line: n + 1,
            text: line.to_string(),
            kind: Kind::Markdown,
        })
        .collect()
}

/// A TOML comment is a line that starts with `#`. A comment after a value is
/// not read.
fn toml_pieces(source: &str) -> Vec<Piece> {
    let lines = source.lines().enumerate();
    lines
        .filter(|(_, line)| line.trim_start().starts_with('#'))
        .map(|(n, line)| Piece {
            line: n + 1,
            text: line.to_string(),
            kind: Kind::Code,
        })
        .collect()
}

/// Where the Rust scan stands between characters. A string remembers whether
/// it is the schema's `CREATE_SQL`, whose `--` lines are SQL comments.
#[derive(Clone, Copy)]
enum Lex {
    Code,
    Str { sql: bool },
    Raw { hashes: usize, sql: bool },
}

/// The schema constant whose SQL comments are read as comments.
const SQL_DECLARATION: &str = "pub const CREATE_SQL";

/// The comment text of a Rust file: every `//` comment, alone on its line or
/// after code, and the `--` lines inside `CREATE_SQL`. A string, raw string,
/// char literal or identifier is never read, so a `//` in a URL or a quote in
/// a char literal does not start a comment, and a tag in a test's data does
/// not count. Block comments are not read; the tree has none.
fn rust_pieces(source: &str) -> Vec<Piece> {
    let mut pieces = Vec::new();
    let mut state = Lex::Code;
    for (n, line) in source.lines().enumerate() {
        if let Lex::Str { sql: true } | Lex::Raw { sql: true, .. } = state {
            if line.trim_start().starts_with("--") {
                pieces.push(Piece {
                    line: n + 1,
                    text: line.to_string(),
                    kind: Kind::Code,
                });
            }
        }
        let declares_sql = line.contains(SQL_DECLARATION);
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            match state {
                Lex::Str { .. } => match c {
                    '\\' => i += 2,
                    '"' => {
                        state = Lex::Code;
                        i += 1;
                    }
                    _ => i += 1,
                },
                Lex::Raw { hashes, .. } => {
                    let closes = c == '"' && (1..=hashes).all(|k| chars.get(i + k) == Some(&'#'));
                    if closes {
                        state = Lex::Code;
                        i += 1 + hashes;
                    } else {
                        i += 1;
                    }
                }
                Lex::Code => match c {
                    '/' if chars.get(i + 1) == Some(&'/') => {
                        let text: String = chars[i + 2..].iter().collect();
                        pieces.push(Piece {
                            line: n + 1,
                            text,
                            kind: Kind::Code,
                        });
                        break;
                    }
                    '"' => {
                        state = Lex::Str { sql: declares_sql };
                        i += 1;
                    }
                    '\'' => {
                        // A char literal, escaped or not, or else a lifetime
                        // or label, which has no closing quote.
                        if chars.get(i + 1) == Some(&'\\') {
                            let close = chars[i + 3..].iter().position(|&ch| ch == '\'');
                            i = close.map_or(chars.len(), |at| i + 3 + at + 1);
                        } else if chars.get(i + 2) == Some(&'\'') {
                            i += 3;
                        } else {
                            i += 1;
                        }
                    }
                    c if c.is_alphabetic() || c == '_' => {
                        let start = i;
                        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                            i += 1;
                        }
                        let word: String = chars[start..i].iter().collect();
                        if word == "r" || word == "br" {
                            let hashes = chars[i..].iter().take_while(|&&ch| ch == '#').count();
                            if chars.get(i + hashes) == Some(&'"') {
                                state = Lex::Raw {
                                    hashes,
                                    sql: declares_sql,
                                };
                                i += hashes + 1;
                            }
                        }
                    }
                    _ => i += 1,
                },
            }
        }
    }
    pieces
}

/// The directories walked, the extension read in each, and a floor under the
/// files found. Each holds well over its floor, so a walk that finds fewer went
/// somewhere else. `fuzz` is never walked whole: its ignored build output and
/// corpus hold tens of thousands of files.
const WALKED: [(&str, &str, usize); 7] = [
    ("crates/terminus-core/src", "rs", 40),
    ("crates/terminus-core/tests", "rs", 25),
    ("crates/terminus/src", "rs", 25),
    ("crates/terminus/tests", "rs", 20),
    ("fuzz/fuzz_targets", "rs", 10),
    ("fuzz/src", "rs", 1),
    ("docs", "md", 10),
];

/// The single files read, beside the walked directories.
const NAMED: [&str; 5] = [
    "Cargo.toml",
    "crates/terminus-core/Cargo.toml",
    "crates/terminus/Cargo.toml",
    "fuzz/Cargo.toml",
    "tests/fixtures/README.md",
];

/// The file that declares `CREATE_SQL`, and a floor under the SQL comment lines
/// read from it. The constant is found by its declaration's exact text, so a
/// rename or a reformat would leave its comments unread with every file floor
/// still met; the schema holds well over this many.
const SCHEMA: (&str, usize) = ("crates/terminus-core/src/store/schema.rs", 100);

/// How many SQL comment lines the Rust scan reads from `source`. Such a piece
/// is its whole source line, where a `//` piece is only the text after `//`.
fn sql_lines(source: &str) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let pieces = rust_pieces(source);
    let sql = pieces
        .iter()
        .filter(|piece| piece.text == lines[piece.line - 1]);
    sql.count()
}

fn files_with(dir: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_with(&path, extension, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            out.push(path);
        }
    }
}

/// The comment text of one file, read by its extension.
fn pieces(path: &Path) -> Vec<Piece> {
    let source =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => rust_pieces(&source),
        Some("toml") => toml_pieces(&source),
        Some("md") => markdown_pieces(&source),
        other => panic!("{} has no reader for {other:?}", path.display()),
    }
}

/// Every comment in the named roots, and every line of the docs, is free of
/// planning tags. All findings are collected first, so one run names every
/// site by file and line.
#[test]
fn no_comment_or_doc_line_cites_a_planning_tag() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files: Vec<PathBuf> = NAMED.iter().map(|name| root.join(name)).collect();
    for (dir, extension, floor) in WALKED {
        let mut found = Vec::new();
        files_with(&root.join(dir), extension, &mut found);
        assert!(
            found.len() >= floor,
            "only {} .{extension} files under {dir}",
            found.len()
        );
        files.extend(found);
    }
    let (schema, floor) = SCHEMA;
    let source =
        std::fs::read_to_string(root.join(schema)).unwrap_or_else(|e| panic!("read {schema}: {e}"));
    let read = sql_lines(&source);
    assert!(
        read >= floor,
        "only {read} SQL comment lines read from {schema}; is `{SQL_DECLARATION}` still declared there?"
    );
    let mut findings = Vec::new();
    for path in &files {
        let shown = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        for piece in pieces(path) {
            for found in tags(&piece.text, piece.kind) {
                findings.push(format!("{shown}:{}: {found}", piece.line));
            }
        }
    }
    assert!(
        findings.is_empty(),
        "planning tags in comments:\n{}",
        findings.join("\n")
    );
}

/// A tag of `prefix` and `digits`, built at run time so this file holds none.
fn tag(prefix: &str, digits: &str) -> String {
    format!("{prefix}{digits}")
}

#[test]
fn every_family_fires_in_a_code_comment() {
    let cases = [
        tag("SRCH-", "02"),
        tag("INJ-", "07"),
        tag("STOR-", "11"),
        tag("D-", "04"),
        tag("AC", "3"),
        tag("AC", "12"),
        tag("PLAN-", "1"),
        tag("PLAN-", "123"),
    ];
    for case in &cases {
        let text = format!("// see {case}, which sets this");
        assert_eq!(tags(&text, Kind::Code), vec![case.clone()], "{text}");
    }
    let pair = format!("({}, {})", tag("RET-", "05"), tag("D-", "06"));
    assert_eq!(
        tags(&pair, Kind::Code),
        vec![tag("RET-", "05"), tag("D-", "06")]
    );
}

#[test]
fn a_phase_number_fires_in_code_in_any_case_and_not_in_markdown() {
    for word in ["phase", "Phase", "PHASE"] {
        let text = format!("// lands in {word} {}", 4);
        assert_eq!(tags(&text, Kind::Code), vec![format!("{word} 4")], "{text}");
        assert!(tags(&text, Kind::Markdown).is_empty(), "{text}");
    }
    // A phase with no number, or a word that only ends in phase, is prose.
    assert!(tags("each phase of the pass", Kind::Code).is_empty());
    let compound = format!("subphase {}", 2);
    assert!(tags(&compound, Kind::Code).is_empty());
}

#[test]
fn the_other_families_fire_in_markdown() {
    for case in [
        tag("ING-", "08"),
        tag("D-", "01"),
        tag("AC", "4"),
        tag("PLAN-", "7"),
    ] {
        let text = format!("The fixture {case} covers it.");
        assert_eq!(tags(&text, Kind::Markdown), vec![case.clone()], "{text}");
    }
}

#[test]
fn names_that_only_look_like_tags_pass() {
    let cases = [
        "0xAC12".to_string(),
        "MAC1".to_string(),
        "ACK".to_string(),
        tag("ABCDEFG-", "12"),
        tag("SHA-", "256"),
        tag("UTF-", "8"),
        tag("ISO-", "8601"),
        tag("UTF-", "16"),
        tag("srch-", "02"),
        tag("SRCH-", "02a"),
        tag("XSRCH_SRCH-", "02"),
        tag("D-", "4"),
        tag("D-", "123"),
        tag("PLAN-", "x"),
    ];
    for case in &cases {
        let text = format!("// {case} here");
        assert!(tags(&text, Kind::Code).is_empty(), "{text}");
        assert!(tags(&text, Kind::Markdown).is_empty(), "{text}");
    }
}

/// The line and tags of every piece that carries one.
fn tagged(pieces: &[Piece]) -> Vec<(usize, Vec<String>)> {
    let tagged = pieces
        .iter()
        .map(|piece| (piece.line, tags(&piece.text, piece.kind)));
    tagged.filter(|(_, found)| !found.is_empty()).collect()
}

#[test]
fn a_rust_comment_is_read_alone_on_its_line_or_after_code() {
    let (a, b, c) = (tag("SRCH-", "02"), tag("D-", "04"), tag("AC", "3"));
    let source = format!("/// {a}\nfn f() {{}}\nlet x = 1; // {b}\n//! {c}\n");
    let found = tagged(&rust_pieces(&source));
    assert_eq!(found, vec![(1, vec![a]), (3, vec![b]), (4, vec![c])]);
}

#[test]
fn a_rust_string_is_not_read() {
    let (a, b) = (tag("SRCH-", "02"), tag("OBS-", "01"));
    let source = [
        format!("let title = \"lands in {a}\";"),
        format!("assert!(url.contains(\"://\"), \"{a}\"); // {b}"),
        format!("let quoted = \"a \\\" // {a}\"; // {b}"),
        format!("let s = \"one\n// {a}\ntwo\";"),
        format!("let raw = r#\"\"// {a}\"\"#; // {b}"),
        format!("let raw = br##\"\n// {a} \"# still in\n\"##;"),
        format!("fn f<'a>(s: &'a str) {{}} // {b}"),
    ]
    .join("\n");
    let found = tagged(&rust_pieces(&source));
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, vec![2, 3, 7, 11]);
    assert!(found
        .iter()
        .all(|(_, tags)| tags == std::slice::from_ref(&b)));
}

#[test]
fn a_comment_after_a_quote_char_literal_is_read() {
    let a = tag("RCL-", "09");
    let source = [
        format!("out.push('\"'); // {a}"),
        format!("if byte == b'\"' {{ return; }} // {a}"),
        format!("let q = '\\''; // {a}"),
        format!("let e = '\\u{{22}}'; // {a}"),
    ]
    .join("\n");
    let found = tagged(&rust_pieces(&source));
    let lines: Vec<usize> = found.iter().map(|(line, _)| *line).collect();
    assert_eq!(lines, vec![1, 2, 3, 4]);
}

#[test]
fn only_the_schema_constant_has_its_sql_comments_read() {
    let (a, b) = (tag("INJ-", "07"), tag("STOR-", "03"));
    let source = [
        format!("{SQL_DECLARATION}: &str = \"\\"),
        format!("-- {a}"),
        "CREATE TABLE t (x INTEGER);".to_string(),
        format!("  -- {a}"),
        "\";".to_string(),
        "const QUERY: &str = \"".to_string(),
        format!("-- {b}"),
        "SELECT 1\";".to_string(),
        format!("-- {b}"),
    ]
    .join("\n");
    let found = tagged(&rust_pieces(&source));
    assert_eq!(found, vec![(2, vec![a.clone()]), (4, vec![a])]);
    assert_eq!(sql_lines(&format!("{source}\nlet x = 1; // -- {b}")), 2);
}

#[test]
fn a_toml_comment_line_is_read_and_a_key_line_is_not() {
    let a = tag("AC", "3");
    let source = format!("# {a}\nname = \"{a}\"\n  # {a}\n");
    let found = tagged(&toml_pieces(&source));
    assert_eq!(found, vec![(1, vec![a.clone()]), (3, vec![a])]);
}

#[test]
fn every_markdown_line_is_read() {
    let a = tag("ING-", "08");
    let source = format!("# Shapes\n\nThe field {a} names.\n`{a}`\n");
    let found = tagged(&markdown_pieces(&source));
    assert_eq!(found, vec![(3, vec![a.clone()]), (4, vec![a])]);
}
