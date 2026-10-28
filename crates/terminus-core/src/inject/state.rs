//! What one Claude Code session has already been given.
//!
//! One JSON file per session, under an `injection` subdirectory of the data
//! directory, keyed on the payload's `session_id`. It carries the turn ids
//! injected so far this session, the turn ids the resume brief carried, the
//! turns that were suppressed and why, and the flag a `SessionStart` whose
//! `source` is `compact` leaves for the next prompt to consume.
//!
//! **Not a table.** A write on the prompt path is the one thing its deadline
//! cannot tolerate: a backfill was measured holding the store for 49 s over the
//! real corpus, and a blocked WRITER is strictly worse than a blocked reader,
//! which is all the read side ever risks. And a new table could not reach an
//! existing user's store at all - `Store::open` runs `CREATE_SQL` only against
//! a fresh store and `bring_forward` covers columns, not tables - so it would
//! arrive either as `no such table` on every upgraded machine or as a
//! `DERIVED_SCHEMA` bump forcing that same ~49 s rebuild on the first
//! hook-spawned pass after an upgrade.
//!
//! **Not the decision log.** That log ([`super::decision`]) is durable, joined
//! against the transcripts that follow, and carries the entities, the
//! candidates, the scores, the thresholds and the tokens spent. This is
//! disposable per-session scratch and holds only what it takes to tell a turn
//! already injected, a turn of the session in view and a turn the brief quoted
//! apart, which is why the reasons stop at three.
//!
//! **Every read fails open.** A missing file, an unreadable one, bytes that are
//! not JSON and a document this build does not recognize all read as empty
//! state rather than as an error. Scratch that cannot be read must never become
//! a prompt that cannot be answered - the worst an empty read can do is inject
//! something twice, and the worst an error could do is inject nothing ever
//! again.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The subdirectory of the data directory these files live in.
///
/// A directory of their own rather than files beside `terminus.db`, so that a
/// listing of the data directory still reads as one store plus its injection
/// scratch however many sessions a machine has had.
pub const DIR_NAME: &str = "injection";

/// The document shape this build writes, and the only one it reads.
///
/// A file carrying anything else - an older build's, a newer build's, or an
/// object that never was one of ours - reads as empty state. There is no
/// migration and there should not be: the whole file is one session's
/// disposable scratch, and the cost of discarding it is at most one turn
/// injected twice.
const FORMAT: u32 = 1;

/// How many suppressions one session's file keeps.
///
/// The list is a record to read and not a fact anything decides on, so it is
/// the one part of this document that forgets. A ranked window is ten
/// candidates and every one of them can be refused, so an unbounded list would
/// grow by ten a prompt for as long as a session lasts.
const MAX_SUPPRESSED: usize = 100;

/// How a write's temporary is named. The drain's sweep matches on these same
/// two, so a change here changes what it deletes.
pub(crate) const TEMPORARY_PREFIX: &str = ".inject-";
pub(crate) const TEMPORARY_SUFFIX: &str = ".tmp";

/// Why a candidate turn was not injected.
///
/// Exactly the three distinctions the suppressions draw and no more. Entities,
/// candidates, scores, thresholds and token counts belong to the decision log,
/// which owns them; recording them here would be a second, lossier decision log
/// to keep in step with the real one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// An earlier prompt in this session was already given this turn.
    AlreadyInjected,
    /// The turn is in the session the user is looking at.
    VisibleInSession,
    /// The resume brief quoted this turn at `SessionStart`.
    CarriedByBrief,
}

/// One candidate that was not injected, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suppressed {
    pub turn_id: i64,
    pub reason: Reason,
}

/// One session's injection scratch.
///
/// `format` is deliberately private and deliberately has no `serde` default: a
/// document that does not spell it is a document this build did not write, and
/// the missing field is what makes [`State::load`] discard it rather than read
/// half of it. Every other field defaults, so a file written before a field
/// existed still loads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    format: u32,
    /// Turn ids injected so far this session, in the order they were given.
    #[serde(default)]
    pub injected: Vec<i64>,
    /// Turn ids the resume brief quoted.
    #[serde(default)]
    pub brief: Vec<i64>,
    /// Candidates refused, with the reason, for a person to read.
    #[serde(default)]
    pub suppressed: Vec<Suppressed>,
    /// Set by a `SessionStart` whose `source` is `compact`, consumed by the
    /// next `UserPromptSubmit`.
    ///
    /// It persists until it fires rather than being cleared by the prompt that
    /// found no boundary row yet: the ingest that commits that row is racing
    /// this prompt, and a flag cleared on the losing side of that race is a
    /// compaction whose dropped turns are never offered at all.
    #[serde(default)]
    pub compaction_owed: bool,
}

/// The empty state, which is also what every failed read returns.
impl Default for State {
    fn default() -> State {
        State {
            format: FORMAT,
            injected: Vec::new(),
            brief: Vec::new(),
            suppressed: Vec::new(),
            compaction_owed: false,
        }
    }
}

impl State {
    /// This session's state, or the empty state.
    ///
    /// Infallible by construction - see the module header. The `session_id` is
    /// an `Option` because the payload's is: an event that carried none has no
    /// session to have state for, and that is one less suppression rather than
    /// an error.
    pub fn load(data_dir: &Path, session_id: Option<&str>) -> State {
        let Some(path) = path(data_dir, session_id) else {
            return State::default();
        };
        let Ok(bytes) = std::fs::read(path) else {
            return State::default();
        };
        match serde_json::from_slice::<State>(&bytes) {
            Ok(state) if state.format == FORMAT => state,
            _ => State::default(),
        }
    }

    /// Write this session's state, reporting whether it landed.
    ///
    /// `false` covers a refused `session_id`, a data directory that cannot be
    /// created and a write that failed - none of which is worth failing a
    /// prompt over, and all of which cost at most a repeated injection.
    pub fn save(&self, data_dir: &Path, session_id: Option<&str>) -> bool {
        let Some(path) = path(data_dir, session_id) else {
            return false;
        };
        write_atomically(&path, self).is_some()
    }

    /// Has an earlier prompt in this session already been given this turn?
    pub fn was_injected(&self, turn_id: i64) -> bool {
        self.injected.contains(&turn_id)
    }

    /// Did the resume brief quote this turn?
    pub fn was_briefed(&self, turn_id: i64) -> bool {
        self.brief.contains(&turn_id)
    }

    /// Remember that this prompt injected `turn_id`.
    ///
    /// Idempotent, because the list is the answer to `was_injected` and a turn
    /// recorded twice would say nothing the first entry does not. It is not
    /// capped: it is what the already-injected suppression is decided on, and a
    /// forgotten id is a turn injected twice - three per prompt, so a session
    /// long enough for the size to matter has other problems.
    pub fn record_injected(&mut self, turn_id: i64) {
        if !self.injected.contains(&turn_id) {
            self.injected.push(turn_id);
        }
    }

    /// Remember that the resume brief quoted `turn_id`.
    pub fn record_brief(&mut self, turn_id: i64) {
        if !self.brief.contains(&turn_id) {
            self.brief.push(turn_id);
        }
    }

    /// Remember that `turn_id` was refused, and why.
    ///
    /// Capped, unlike the two lists above, and that asymmetry is the design:
    /// this one is a record for a person to read rather than a fact
    /// anything decides on, and it grows by up to one entry per ranked
    /// candidate per prompt instead of by at most three. The oldest go first,
    /// so what is in the file is what the session just did.
    pub fn record_suppressed(&mut self, turn_id: i64, reason: Reason) {
        let entry = Suppressed { turn_id, reason };
        if self.suppressed.contains(&entry) {
            return;
        }
        self.suppressed.push(entry);
        if self.suppressed.len() > MAX_SUPPRESSED {
            let excess = self.suppressed.len() - MAX_SUPPRESSED;
            self.suppressed.drain(..excess);
        }
    }
}

/// The file one `session_id` names, or nothing.
///
/// The check runs before any path is built, so a `session_id` that fails it
/// reaches no filesystem call at all: [`super::checked_session_id`].
fn path(data_dir: &Path, session_id: Option<&str>) -> Option<PathBuf> {
    let id = super::checked_session_id(session_id?)?;
    Some(data_dir.join(DIR_NAME).join(format!("{id}.json")))
}

/// Write `state` to `path`, replacing it whole.
///
/// This runs on the hook's abandoned thread: the injection runs on a thread the
/// binary never joins and process exit takes it wherever it got to, so a write
/// cut off at the deadline leaves the target holding its previous content and at
/// worst a `.inject-*.tmp` beside it - never a document the next prompt reads
/// half of.
///
/// No `fsync`, which is right for per-session scratch whose loss costs one
/// repeated injection, paid for otherwise with a disk flush on the path where
/// the user is waiting.
///
/// Pretty-printed, because a person reads suppression reasons out of this file
/// and the whole document is a few hundred bytes.
fn write_atomically(path: &Path, state: &State) -> Option<()> {
    let dir = path.parent()?;
    // TWO creations, not one. `injection/`'s parent is terminus's own data
    // directory, and a hook that fires before any ingest ever ran is the first
    // thing to make it - so it has to be asked for by name, because the
    // primitive narrows only the leaf it is given and would otherwise leave the
    // data directory at the umask. Everything above it - the user's
    // `~/.local/share` - still keeps their own default, which is the whole
    // reason the mode is not applied recursively.
    if let Some(data_dir) = dir.parent() {
        crate::owner_only::create_dir_all(data_dir).ok()?;
    }
    crate::owner_only::create_dir_all(dir).ok()?;
    let bytes = serde_json::to_vec_pretty(state).ok()?;

    // The temporary can never collide with a session's own file: it starts with a
    // dot and ends in `.tmp`, and [`super::checked_session_id`] admits neither.
    crate::owner_only::replace(path, &bytes, TEMPORARY_PREFIX, TEMPORARY_SUFFIX).ok()
}
