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
    bytes[at.min(bytes.len())..].iter().take_while(|b| pred(b)).count()
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
    assert_eq!(tags(&pair, Kind::Code), vec![tag("RET-", "05"), tag("D-", "06")]);
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
    for case in [tag("ING-", "08"), tag("D-", "01"), tag("AC", "4"), tag("PLAN-", "7")] {
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
