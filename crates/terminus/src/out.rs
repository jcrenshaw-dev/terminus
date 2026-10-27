//! Everything the binary writes to stdout goes through here.
//!
//! `println!` panics with exit 101 when the reader has closed the pipe, so
//! `terminus sessions | head` printed a Rust panic, and a hook whose reader went
//! away broke its exit-0 rule. These write the same bytes and drop them once the
//! reader is gone: the command still finishes its work and exits with its own
//! code, which is what `doctor` already did for its report.
//!
//! **Every write escapes by default (#72).** `outln!` and `out!` run the whole
//! formatted text through [`crate::escape::controls`] before it is written, so
//! no command's human output can carry a terminal control that came out of
//! the archive, and a new command cannot forget to. The text is escaped after
//! it is formatted, never per argument, because a per-argument wrapper is the
//! per-site rule the default replaces.
//!
//! **Raw is an explicit call at the site that must stay byte for byte.**
//! [`write_raw`] and [`try_write_raw`] write exactly the bytes they are given.
//! The sites are `Document::emit` and `try_emit`, the hook's document, and the
//! body `show` prints. No mode chooses it: the hook's document and `show`'s body
//! are not `--json` output, and a process-wide flag would be one more thing to
//! forget.

use std::fmt;
use std::io::{self, Write};

use crate::escape;

/// `println!`, without the panic on a closed pipe, escaped.
macro_rules! outln {
    () => {
        $crate::out::write(format_args!("\n"))
    };
    ($($arg:tt)*) => {
        $crate::out::write(format_args!("{}\n", format_args!($($arg)*)))
    };
}

/// `print!`, without the panic on a closed pipe, escaped.
macro_rules! out {
    ($($arg:tt)*) => {
        $crate::out::write(format_args!($($arg)*))
    };
}

/// The formatted text with every control other than tab and newline spelled out.
fn escaped(args: fmt::Arguments<'_>) -> String {
    escape::controls(&args.to_string()).into_owned()
}

/// What a write to a stream does with its failure: drop it when the reader has
/// gone, panic on any other.
///
/// A full disk under a redirect is a lost result, not a reader that stopped
/// reading, and it panics exactly as `print!` would.
fn settle(result: io::Result<()>, stream: &str) {
    match result {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
        Err(e) => panic!("failed printing to {stream}: {e}"),
    }
}

fn put(bytes: &[u8]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    out.write_all(bytes)?;
    out.flush()
}

/// Write to stdout with its controls escaped, and drop the bytes if the reader
/// has gone.
pub fn write(args: fmt::Arguments<'_>) {
    settle(try_write(args), "stdout");
}

/// The same escaped write, with its failure returned rather than handled.
pub fn try_write(args: fmt::Arguments<'_>) -> io::Result<()> {
    put(escaped(args).as_bytes())
}

/// Write exactly these bytes to stdout, and drop them if the reader has gone.
pub fn write_raw(bytes: &[u8]) {
    settle(try_write_raw(bytes), "stdout");
}

/// The same raw write, with its failure returned rather than handled.
pub fn try_write_raw(bytes: &[u8]) -> io::Result<()> {
    put(bytes)
}
