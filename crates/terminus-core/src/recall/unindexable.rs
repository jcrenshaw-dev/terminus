//! The letters SQLite's tokenizer makes no token from (#23).
//!
//! A query word is cut on every character Rust does not count as alphanumeric,
//! and `turns_fts` is tokenized by `unicode61`, the default, because
//! `crate::store::schema` gives it no tokenize option. The two rules disagree.
//! `unicode61` starts a token only on a character in the Unicode letter, number
//! and private-use categories, so some characters Rust counts as letters - the
//! combining mark U+0345 and circled letters such as U+24B6 - are separators to
//! it. A query word made only of them is a quoted phrase with no token in it,
//! FTS5 matches no row on it, and the `AND` the query puts between its words
//! then makes the whole search return nothing.
//!
//! [`makes_no_token`] answers from [`UNINDEXABLE`], a plain table, so that
//! [`crate::recall::Query::parse`] needs no connection and stays infallible.
//! The table is every code point `char::is_alphanumeric` accepts from which the
//! bundled SQLite makes no token, measured against SQLite 3.53.2 through
//! rusqlite 0.40.2 `bundled`.
//!
//! The test below pins it. It runs every Rust-alphanumeric code point through
//! the SQLite this crate links and fails on any code point where the table and
//! the tokenizer disagree. To regenerate the table after a rusqlite bump, run
//! `cargo test -p terminus-core --lib unindexable` and paste the table its
//! failure message prints over the one below.

/// Inclusive ranges of code points, sorted and not overlapping, each a maximal
/// run of consecutive code points.
const UNINDEXABLE: &[(char, char)] = &[
    ('\u{345}', '\u{345}'),
    ('\u{363}', '\u{36F}'),
    ('\u{5B0}', '\u{5BD}'),
    ('\u{5BF}', '\u{5BF}'),
    ('\u{5C1}', '\u{5C2}'),
    ('\u{5C4}', '\u{5C5}'),
    ('\u{5C7}', '\u{5C7}'),
    ('\u{610}', '\u{61A}'),
    ('\u{64B}', '\u{657}'),
    ('\u{659}', '\u{65F}'),
    ('\u{670}', '\u{670}'),
    ('\u{6D6}', '\u{6DC}'),
    ('\u{6E1}', '\u{6E4}'),
    ('\u{6E7}', '\u{6E8}'),
    ('\u{6ED}', '\u{6ED}'),
    ('\u{711}', '\u{711}'),
    ('\u{730}', '\u{73F}'),
    ('\u{7A6}', '\u{7B0}'),
    ('\u{816}', '\u{817}'),
    ('\u{81B}', '\u{823}'),
    ('\u{825}', '\u{827}'),
    ('\u{829}', '\u{82C}'),
    ('\u{8E4}', '\u{8E9}'),
    ('\u{8F0}', '\u{8FE}'),
    ('\u{900}', '\u{903}'),
    ('\u{93A}', '\u{93B}'),
    ('\u{93E}', '\u{94C}'),
    ('\u{94E}', '\u{94F}'),
    ('\u{955}', '\u{957}'),
    ('\u{962}', '\u{963}'),
    ('\u{981}', '\u{983}'),
    ('\u{9BE}', '\u{9C4}'),
    ('\u{9C7}', '\u{9C8}'),
    ('\u{9CB}', '\u{9CC}'),
    ('\u{9D7}', '\u{9D7}'),
    ('\u{9E2}', '\u{9E3}'),
    ('\u{A01}', '\u{A03}'),
    ('\u{A3E}', '\u{A42}'),
    ('\u{A47}', '\u{A48}'),
    ('\u{A4B}', '\u{A4C}'),
    ('\u{A51}', '\u{A51}'),
    ('\u{A70}', '\u{A71}'),
    ('\u{A75}', '\u{A75}'),
    ('\u{A81}', '\u{A83}'),
    ('\u{ABE}', '\u{AC5}'),
    ('\u{AC7}', '\u{AC9}'),
    ('\u{ACB}', '\u{ACC}'),
    ('\u{AE2}', '\u{AE3}'),
    ('\u{B01}', '\u{B03}'),
    ('\u{B3E}', '\u{B44}'),
    ('\u{B47}', '\u{B48}'),
    ('\u{B4B}', '\u{B4C}'),
    ('\u{B56}', '\u{B57}'),
    ('\u{B62}', '\u{B63}'),
    ('\u{B82}', '\u{B82}'),
    ('\u{BBE}', '\u{BC2}'),
    ('\u{BC6}', '\u{BC8}'),
    ('\u{BCA}', '\u{BCC}'),
    ('\u{BD7}', '\u{BD7}'),
    ('\u{C01}', '\u{C03}'),
    ('\u{C3E}', '\u{C44}'),
    ('\u{C46}', '\u{C48}'),
    ('\u{C4A}', '\u{C4C}'),
    ('\u{C55}', '\u{C56}'),
    ('\u{C62}', '\u{C63}'),
    ('\u{C82}', '\u{C83}'),
    ('\u{CBE}', '\u{CC4}'),
    ('\u{CC6}', '\u{CC8}'),
    ('\u{CCA}', '\u{CCC}'),
    ('\u{CD5}', '\u{CD6}'),
    ('\u{CE2}', '\u{CE3}'),
    ('\u{D02}', '\u{D03}'),
    ('\u{D3E}', '\u{D44}'),
    ('\u{D46}', '\u{D48}'),
    ('\u{D4A}', '\u{D4C}'),
    ('\u{D57}', '\u{D57}'),
    ('\u{D62}', '\u{D63}'),
    ('\u{D82}', '\u{D83}'),
    ('\u{DCF}', '\u{DD4}'),
    ('\u{DD6}', '\u{DD6}'),
    ('\u{DD8}', '\u{DDF}'),
    ('\u{DF2}', '\u{DF3}'),
    ('\u{E31}', '\u{E31}'),
    ('\u{E34}', '\u{E3A}'),
    ('\u{E4D}', '\u{E4D}'),
    ('\u{EB1}', '\u{EB1}'),
    ('\u{EB4}', '\u{EB9}'),
    ('\u{EBB}', '\u{EBC}'),
    ('\u{ECD}', '\u{ECD}'),
    ('\u{F71}', '\u{F83}'),
    ('\u{F8D}', '\u{F97}'),
    ('\u{F99}', '\u{FBC}'),
    ('\u{102B}', '\u{1036}'),
    ('\u{1038}', '\u{1038}'),
    ('\u{103B}', '\u{103E}'),
    ('\u{1056}', '\u{1059}'),
    ('\u{105E}', '\u{1060}'),
    ('\u{1062}', '\u{1064}'),
    ('\u{1067}', '\u{106D}'),
    ('\u{1071}', '\u{1074}'),
    ('\u{1082}', '\u{108D}'),
    ('\u{108F}', '\u{108F}'),
    ('\u{109A}', '\u{109D}'),
    ('\u{1712}', '\u{1713}'),
    ('\u{1732}', '\u{1733}'),
    ('\u{1752}', '\u{1753}'),
    ('\u{1772}', '\u{1773}'),
    ('\u{17B6}', '\u{17C8}'),
    ('\u{18A9}', '\u{18A9}'),
    ('\u{1920}', '\u{192B}'),
    ('\u{1930}', '\u{1938}'),
    ('\u{19B0}', '\u{19C0}'),
    ('\u{19C8}', '\u{19C9}'),
    ('\u{1A17}', '\u{1A1B}'),
    ('\u{1A55}', '\u{1A5E}'),
    ('\u{1A61}', '\u{1A74}'),
    ('\u{1B00}', '\u{1B04}'),
    ('\u{1B35}', '\u{1B43}'),
    ('\u{1B80}', '\u{1B82}'),
    ('\u{1BA1}', '\u{1BA9}'),
    ('\u{1BAC}', '\u{1BAD}'),
    ('\u{1BE7}', '\u{1BF1}'),
    ('\u{1C24}', '\u{1C36}'),
    ('\u{1CF2}', '\u{1CF3}'),
    ('\u{1DD3}', '\u{1DE6}'),
    ('\u{24B6}', '\u{24E9}'),
    ('\u{2DE0}', '\u{2DFF}'),
    ('\u{A674}', '\u{A67B}'),
    ('\u{A69F}', '\u{A69F}'),
    ('\u{A802}', '\u{A802}'),
    ('\u{A80B}', '\u{A80B}'),
    ('\u{A823}', '\u{A827}'),
    ('\u{A880}', '\u{A881}'),
    ('\u{A8B4}', '\u{A8C3}'),
    ('\u{A926}', '\u{A92A}'),
    ('\u{A947}', '\u{A952}'),
    ('\u{A980}', '\u{A983}'),
    ('\u{A9B4}', '\u{A9BF}'),
    ('\u{AA29}', '\u{AA36}'),
    ('\u{AA43}', '\u{AA43}'),
    ('\u{AA4C}', '\u{AA4D}'),
    ('\u{AA7B}', '\u{AA7B}'),
    ('\u{AAB0}', '\u{AAB0}'),
    ('\u{AAB2}', '\u{AAB4}'),
    ('\u{AAB7}', '\u{AAB8}'),
    ('\u{AABE}', '\u{AABE}'),
    ('\u{AAEB}', '\u{AAEF}'),
    ('\u{AAF5}', '\u{AAF5}'),
    ('\u{ABE3}', '\u{ABEA}'),
    ('\u{FB1E}', '\u{FB1E}'),
    ('\u{10A01}', '\u{10A03}'),
    ('\u{10A05}', '\u{10A06}'),
    ('\u{10A0C}', '\u{10A0F}'),
    ('\u{11000}', '\u{11002}'),
    ('\u{11038}', '\u{11045}'),
    ('\u{11080}', '\u{11082}'),
    ('\u{110B0}', '\u{110B8}'),
    ('\u{11100}', '\u{11102}'),
    ('\u{11127}', '\u{11132}'),
    ('\u{11180}', '\u{11182}'),
    ('\u{111B3}', '\u{111BF}'),
    ('\u{116AB}', '\u{116B5}'),
    ('\u{16F51}', '\u{16F7E}'),
    ('\u{16F8F}', '\u{16F92}'),
    ('\u{1F130}', '\u{1F149}'),
    ('\u{1F150}', '\u{1F169}'),
    ('\u{1F170}', '\u{1F189}'),
];

/// Does the bundled SQLite's `unicode61` tokenizer make no token from `c`?
///
/// Only meaningful for a character `char::is_alphanumeric` accepts: the table
/// holds nothing else.
pub(super) fn makes_no_token(c: char) -> bool {
    UNINDEXABLE
        .binary_search_by(|&(first, last)| {
            if last < c {
                std::cmp::Ordering::Less
            } else if first > c {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;

    use rusqlite::Connection;

    use super::*;

    /// Maximal runs of consecutive code points in an ascending list.
    fn runs(points: &[char]) -> Vec<(char, char)> {
        let mut out: Vec<(char, char)> = Vec::new();
        for &c in points {
            match out.last_mut() {
                Some((_, last)) if *last as u32 + 1 == c as u32 => *last = c,
                _ => out.push((c, c)),
            }
        }
        out
    }

    /// The whole table in its own source syntax, ready to paste.
    fn as_source(table: &[(char, char)]) -> String {
        let mut out = String::from("const UNINDEXABLE: &[(char, char)] = &[\n");
        for (first, last) in table {
            writeln!(
                out,
                "    ('\\u{{{:X}}}', '\\u{{{:X}}}'),",
                *first as u32, *last as u32
            )
            .unwrap();
        }
        out.push_str("];\n");
        out
    }

    fn as_list(table: &[(char, char)]) -> String {
        table
            .iter()
            .map(|(first, last)| {
                if first == last {
                    format!("U+{:04X}", *first as u32)
                } else {
                    format!("U+{:04X}..U+{:04X}", *first as u32, *last as u32)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Every Rust-alphanumeric code point through the linked SQLite's
    /// tokenizer, one row each, against the table.
    #[test]
    fn the_table_is_what_the_bundled_tokenizer_does() {
        let alphanumerics: Vec<char> = (0..=char::MAX as u32)
            .filter_map(char::from_u32)
            .filter(|c| c.is_alphanumeric())
            .collect();

        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE VIRTUAL TABLE sweep USING fts5(body);
             CREATE VIRTUAL TABLE sweep_vocab USING fts5vocab(sweep, instance);",
        )
        .unwrap();
        let tx = conn.transaction().unwrap();
        {
            let mut insert = tx
                .prepare("INSERT INTO sweep(rowid, body) VALUES (?1, ?2)")
                .unwrap();
            for &c in &alphanumerics {
                insert
                    .execute(rusqlite::params![c as u32, c.to_string()])
                    .unwrap();
            }
        }
        tx.commit().unwrap();

        let tokenized: BTreeSet<u32> = conn
            .prepare("SELECT DISTINCT doc FROM sweep_vocab")
            .unwrap()
            .query_map([], |row| row.get::<_, u32>(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();

        let no_token: Vec<char> = alphanumerics
            .iter()
            .copied()
            .filter(|c| !tokenized.contains(&(*c as u32)))
            .collect();
        let disagreeing: Vec<char> = alphanumerics
            .iter()
            .copied()
            .filter(|c| makes_no_token(*c) != !tokenized.contains(&(*c as u32)))
            .collect();

        assert!(
            disagreeing.is_empty(),
            "the table and the bundled SQLite disagree on {}\n\
             The table as this SQLite computes it:\n{}",
            as_list(&runs(&disagreeing)),
            as_source(&runs(&no_token)),
        );
    }
}
