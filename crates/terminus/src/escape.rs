//! Control characters in archived text, made visible before a terminal sees them.
//!
//! A tool result can carry any byte a program printed, and a transcript keeps
//! it: an ESC that starts a colour, a cursor move, an OSC 52 clipboard write.
//! Printed as it is, archived text can drive the terminal of whoever reads it
//! back. The human output of `search`, `show` and `observations` runs every
//! archived string through [`controls`] on the way out. The store, `--json` and
//! the `show` record body are untouched; this is a rendering, the way redaction
//! is.
//!
//! **The set is `char::is_control` minus tab and newline.** That is U+0000
//! to U+001F and U+007F to U+009F, 63 code points. Tab and newline lay text out
//! and drive nothing. CR stays in the set: an observation command is only
//! clipped, so a heredoc keeps its raw line breaks, and a CR in one would let
//! the command overprint its own line. Format characters such as U+200B and
//! U+202E are not controls and pass.
//!
//! **An escaped character prints as its JSON spelling, `\u001b`.** It is
//! ASCII, it is what serde_json writes for C0, and it is what the archived line
//! already holds, so `show --after` spells a character the same way in the body
//! and in the window. `escape_debug` also escapes quotes, backslash, tab and
//! U+202E and spells ESC `\u{1b}`. `\x1b` misdescribes C1, caret notation has
//! no form for C1, and Control Pictures garble under a non-UTF-8 locale.

use std::borrow::Cow;
use std::fmt::Write;

/// `text` with every control other than tab and newline spelled `\u00XX`.
///
/// Text with nothing to escape comes back borrowed and unchanged. A backslash
/// passes as it is, so a literal `\u001b` already in the text reads the same as
/// an escaped ESC, the same ambiguity the archived JSON line has.
pub(crate) fn controls(text: &str) -> Cow<'_, str> {
    if !text.chars().any(escaped) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 16);
    for c in text.chars() {
        if escaped(c) {
            // Writing to a String cannot fail.
            let _ = write!(out, "\\u{:04x}", u32::from(c));
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

fn escaped(c: char) -> bool {
    c.is_control() && c != '\t' && c != '\n'
}

#[cfg(test)]
mod tests {
    use super::controls;

    fn one(c: char) -> String {
        controls(c.encode_utf8(&mut [0; 4])).into_owned()
    }

    #[test]
    fn exactly_the_controls_other_than_tab_and_newline_change() {
        let mut changed = 0;
        for c in (0..=0x10FFFF).filter_map(char::from_u32) {
            let out = one(c);
            let control = c.is_control() && c != '\t' && c != '\n';
            assert_eq!(out != c.to_string(), control, "U+{:04X}", u32::from(c));
            if control {
                changed += 1;
                assert_eq!(out, format!("\\u{:04x}", u32::from(c)));
                assert!(out[2..]
                    .chars()
                    .all(|h| h.is_ascii_digit() || ('a'..='f').contains(&h)));
            }
        }
        assert_eq!(changed, 63);
    }

    #[test]
    fn the_terminal_controls_read_by_name() {
        assert_eq!(one('\u{1b}'), "\\u001b");
        assert_eq!(one('\u{07}'), "\\u0007");
        assert_eq!(one('\r'), "\\u000d");
        assert_eq!(one('\u{7f}'), "\\u007f");
        assert_eq!(one('\u{85}'), "\\u0085");
        assert_eq!(one('\u{9b}'), "\\u009b");
    }

    #[test]
    fn layout_format_characters_and_backslash_pass() {
        for c in ['\t', '\n', '\u{200b}', '\u{202e}', '\\'] {
            assert_eq!(one(c), c.to_string(), "U+{:04X}", u32::from(c));
        }
        let plain = "a\tb\nc \\u001b d";
        assert!(matches!(controls(plain), std::borrow::Cow::Borrowed(s) if s == plain));
        assert_eq!(
            controls("\u{1b}]52;c;aGk=\u{07}x"),
            "\\u001b]52;c;aGk=\\u0007x"
        );
    }
}
