//! `terminus stats`: whether injection helps.
//!
//! A CLI data command and never an MCP tool. Every tool description sits
//! in every session's context forever, and the model does not need to know how
//! its own recall is scoring - the three recall tools are the whole MCP
//! surface.
//!
//! It reads through the shared read path, so a machine that has never ingested
//! gets an empty answer with a reason and exit 0 rather than a store created as
//! the side effect of a question.
//!
//! The numbers are characters, never tokens, and the output says so.

use terminus_core::feedback::stats::{self, Stats};

use super::json::Document;
use super::read::{self, Opened};
use super::Failure;

/// The command name, which is also what the `--json` envelope reports.
const COMMAND: &str = "stats";

pub fn run(json: bool) -> Result<(), Failure> {
    let reader = match read::open()? {
        Opened::Ready(reader) => reader,
        Opened::Nothing(reason) => return read::empty(empty(), &reason, json),
    };

    // A decision log missing either table is damage, caught here so the
    // failure names it: the query below would fail with a raw
    // `no such table: decisions`. Zeroes reported as a clean answer would read
    // as "injection has recorded nothing", which a damaged store must not say.
    // Under `--json` `main` writes the empty document with `ok: false` and this
    // reason (#107).
    match read::decision_log(&reader) {
        read::DecisionLog::Present => {}
        read::DecisionLog::Damaged(tables) => {
            return Err(Failure::Operational(read::damaged(tables)))
        }
    }

    let stats = stats::stats(reader.store()).map_err(|e| Failure::Operational(e.to_string()))?;

    if json {
        // The same numbers the human output prints, and no others.
        document(&stats).emit();
        return Ok(());
    }

    outln!("decisions         {}", stats.decisions);
    // Indented as a split of the line above. A decision whose `compacted` is
    // null is in neither, so the two need not add up to it.
    outln!("  compacted       {}", stats.compacted_decisions);
    outln!("  ordinary        {}", stats.ordinary_decisions);
    outln!("injected turns    {}", stats.injected_turns);
    outln!("hits              {}", stats.hits);
    outln!("false positives   {}", stats.false_positives);
    // Two decimals, or the word: a precision printed as 0.00 on an archive
    // whose sessions are all still open would read as "injection never helps".
    match stats.precision {
        Some(precision) => outln!("precision         {precision:.2}"),
        None => outln!("precision         (nothing labelled yet)"),
    }
    outln!("misses            {}", stats.misses);
    outln!("wasted budget     {}", stats.wasted_budget);
    outln!("chars injected    {}", stats.chars_injected);
    outln!("chars referenced  {}", stats.chars_referenced);
    Ok(())
}

/// The document with every count at zero and no precision. The no-store answer,
/// and what `main` writes when the command fails.
pub(crate) fn empty() -> Document {
    document(&Stats::default())
}

/// The envelope for this command, with the numbers already in it.
fn document(stats: &Stats) -> Document {
    Document::new(COMMAND)
        .field("decisions", stats.decisions)
        .field("compacted_decisions", stats.compacted_decisions)
        .field("ordinary_decisions", stats.ordinary_decisions)
        .field("injected_turns", stats.injected_turns)
        .field("hits", stats.hits)
        .field("false_positives", stats.false_positives)
        .field(
            "precision",
            match stats.precision {
                Some(precision) => serde_json::Value::from(precision),
                // Present and null rather than absent, the way the envelope
                // carries `reason`: a consumer reads one shape either way.
                None => serde_json::Value::Null,
            },
        )
        .field("misses", stats.misses)
        .field("wasted_budget", stats.wasted_budget)
        .field("chars_injected", stats.chars_injected)
        .field("chars_referenced", stats.chars_referenced)
}
