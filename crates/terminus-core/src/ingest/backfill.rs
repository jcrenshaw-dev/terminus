//! The bounded-parallelism pass a backfill runs.
//!
//! One pass over the tree, with the reading, JSON scanning and zstd compression
//! spread across a fixed number of [`std::thread`] workers while **exactly one**
//! thread owns the writable SQLite connection and performs every write - every
//! per-file transaction, every watermark, the `runs` row, all of it.
//!
//! # Why the writer is single and the workers are few
//!
//! SQLite has one writer. The ceiling on a backfill is therefore not the core
//! count, and adding workers past the point where the writer saturates buys
//! nothing but memory. Compression is the stage that actually scales, which is
//! why it is the stage that moved. `crates/terminus/src/cmd/backfill.rs` fixes
//! the count in one place, small and deliberately below any machine's cores.
//!
//! # No thread pool, and no thread on the hook path
//!
//! `std::thread` and `std::sync::mpsc`, no `rayon` and no global pool. The
//! threads are started here and nowhere else, inside `pipeline`, which is
//! reached only through [`run_with`] and only by `terminus backfill`; the
//! driver it shares with the tree pass starts none. A hook-triggered
//! `terminus ingest` runs `pass::run` exactly as it did before and starts no
//! thread at all - the measured 0.408 ms startup floor is the product, and it
//! must not pay for a command it never runs.
//!
//! # The work in flight is bounded, and that is what makes a backfill chunked
//!
//! Files are handed out in rounds of [`QUEUE_PER_WORKER`] per worker, and a
//! round is fully drained before the next is dispatched. Peak memory is
//! therefore the round size and never the corpus: real transcripts are p50
//! 290 KB, p90 1.03 MB and p99 3.1 MB, and an unbounded queue over 2,248 of
//! them would hold the whole 1.1 GB tree in RAM plus its compressed copy.
//!
//! Rounds are also what makes this deadlock-free with `std` channels alone. Both
//! channels are bounded at exactly the round size and at most one round is ever
//! outstanding, so no send on either side can block: the writer never waits to
//! dispatch while a worker waits to deliver.
//!
//! # Everything that makes a pass resumable is the tree pass's
//!
//! A backfill runs through the same driver as the tree pass (`ingest::driver`),
//! which takes the ingest lock once for the whole run before the store is
//! opened, recovers at the top, and writes one `runs` row for the whole pass
//! through the same INSERT. One transaction per file, so a kill loses at most
//! one file, and a per-file failure is recorded against its path and skipped
//! rather than propagated, by the same handler the sequential walk uses. Files
//! are applied in discovery order, exactly as the sequential walk applies them,
//! so the two reach byte-identical stores rather than merely equivalent ones.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::ingest::driver::{self, Walk};
use crate::ingest::pass::{PassOutcome, Summary};
use crate::ingest::{claim, Claim, Existing, Prepared, Rows};
use crate::project::Resolver;
use crate::store::Store;

/// Files in flight per worker.
///
/// Two: one being worked on and one already read, so a worker that finishes a
/// small file does not wait for the writer to come back round. Higher multiplies
/// peak memory by the p99 transcript size for no throughput a single writer can
/// absorb.
pub const QUEUE_PER_WORKER: usize = 2;

/// What one backfill pass did.
///
/// [`PassOutcome`] and [`Summary`] are `pass`'s, unchanged and unwrapped: a
/// backfill is a pass, and two accounts of what a pass did would be two things
/// to keep in step. The two fields beside it are the only facts that are about
/// the *pipeline* rather than about the pass.
#[derive(Debug, Clone)]
pub struct Report {
    pub outcome: PassOutcome,
    /// Workers asked for.
    pub workers: usize,
    /// Distinct worker threads that actually prepared at least one file. The
    /// evidence that the work was spread, rather than the claim that it was.
    pub threads_used: usize,
}

/// Walk every configured transcript root in parallel and ingest what is beneath
/// it.
pub fn run(data_dir: &Path, workers: usize) -> Result<Report> {
    let config = Config::load()?;
    run_with(data_dir, &config, workers)
}

/// The pass, against a config the caller already has.
///
/// The seam tests use, mirroring [`crate::ingest::pass::run_with`], so the two
/// can be run over one tree and compared.
pub fn run_with(data_dir: &Path, config: &Config, workers: usize) -> Result<Report> {
    // Zero workers is a caller's arithmetic, not a request to do nothing.
    let workers = workers.max(1);

    // The driver takes the lock once for the whole run, before the store is
    // opened, recovers, and writes the one row. A hook that fires during a
    // backfill loses that race and exits 0, and the thread set stays empty.
    let mut backfill = Backfill {
        config,
        workers,
        transcripts: Vec::new(),
        threads: HashSet::new(),
    };
    let outcome = driver::drive(data_dir, &mut backfill)?;
    Ok(Report {
        outcome,
        workers,
        threads_used: backfill.threads.len(),
    })
}

/// The backfill, as the driver walks it: discovery, then the pipeline over
/// every row, and none of the tree pass's other steps.
struct Backfill<'a> {
    config: &'a Config,
    workers: usize,
    /// What discovery found, filled in the step before the rows are read.
    transcripts: Vec<PathBuf>,
    /// Distinct worker threads that prepared a file.
    threads: HashSet<ThreadId>,
}

impl Walk for Backfill<'_> {
    fn before(&mut self, _store: &mut Store, summary: &mut Summary) {
        let found = crate::discover::discover(self.config);
        summary.unreadable = found.unreadable;
        summary.excluded = found.excluded;
        self.transcripts = found.transcripts;
    }

    fn walk(&mut self, store: &mut Store, rows: &Rows, summary: &mut Summary) -> Result<()> {
        // One resolver for the whole run, on the writer thread. Project
        // identity may spawn git and is memoized across 63 distinct `cwd`
        // values in a real corpus, so keeping it here costs a handful of
        // resolutions and saves every worker queueing on a lock around it.
        let mut projects = Resolver::new();
        pipeline(
            store,
            rows,
            &self.transcripts,
            summary,
            &mut projects,
            self.workers,
            &mut self.threads,
            self.config.capture_mode(),
        )
    }
}

/// A worker panic, turned into that file's own failure.
///
/// The drain loop in [`dispatch_round`] waits for exactly as many `Done`
/// messages as it sent, and its disconnect arm only fires once EVERY worker is
/// gone. So a single worker panicking inside `prepare` used to wedge the pass in
/// the worst way available: its `Done` never arrived, the surviving workers kept
/// `done_rx` connected so the disconnect arm never fired, and the scope could
/// not join to re-raise the panic because its own closure was the thing
/// blocked. The backfill simply stopped, with no error and no exit.
///
/// Catching it here restores the invariant the drain loop is written against -
/// every job sent comes back exactly once - and costs nothing else: one
/// damaged transcript is already not a reason to archive none of the tree, and
/// a panic is only the loudest way for one to be damaged. The file is
/// recorded and skipped like any other per-file failure.
fn panic_error(path: &Path, payload: Box<dyn std::any::Any + Send>) -> Error {
    let detail = payload
        .downcast_ref::<&'static str>()
        .map(|s| (*s).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "panicked".to_string());
    Error::Io {
        path: path.to_path_buf(),
        source: std::io::Error::other(format!("preparing this file panicked: {detail}")),
    }
}

/// One file handed to a worker: everything preparing it needs and nothing that
/// needs a connection.
struct Job {
    path: PathBuf,
    session_key: String,
    existing: Existing,
}

/// One file coming back: which thread did it, and what it produced.
struct Done {
    /// Position within the round, so the writer applies files in discovery
    /// order however the workers happened to finish.
    slot: usize,
    thread: ThreadId,
    prepared: Result<Prepared>,
}

/// The round-by-round pipeline.
///
/// Only a failure of the pass as a whole comes back as `Err`; every per-file
/// failure is recorded against its path and skipped, because the tree is the
/// unit of work and one damaged transcript is not a reason to archive none of
/// it.
#[allow(clippy::too_many_arguments)]
fn pipeline(
    store: &mut Store,
    rows: &Rows,
    transcripts: &[PathBuf],
    summary: &mut Summary,
    projects: &mut Resolver,
    workers: usize,
    threads: &mut HashSet<ThreadId>,
    // Resolved once for the whole run and copied into every worker.
    // `CaptureMode` is `Copy`, so the workers share a value rather than a lock.
    mode: crate::config::CaptureMode,
) -> Result<()> {
    if transcripts.is_empty() {
        return Ok(());
    }
    let round = workers * QUEUE_PER_WORKER;

    // Scoped threads: the workers borrow nothing that outlives this call, and
    // the scope joins every one of them before it returns - including on the
    // panic and early-return paths, which is what makes "the writer is the only
    // thread left touching the store" true rather than hoped for.
    std::thread::scope(|scope| {
        // Both bounded at the round size, with at most one round outstanding,
        // so neither side can ever block on a send. See the module comment.
        let (job_tx, job_rx) = mpsc::sync_channel::<(usize, Job)>(round);
        let (done_tx, done_rx) = mpsc::sync_channel::<Done>(round);
        let job_rx = Arc::new(Mutex::new(job_rx));

        for _ in 0..workers {
            let jobs = Arc::clone(&job_rx);
            let done = done_tx.clone();
            scope.spawn(move || {
                loop {
                    // The lock is held for the receive and released before the
                    // work, so the workers share one queue and take the next
                    // file whenever they are free - which is what keeps a
                    // 3 MB transcript from stalling three idle threads.
                    let next = {
                        let queue = jobs.lock().unwrap_or_else(|e| e.into_inner());
                        queue.recv()
                    };
                    let Ok((slot, job)) = next else {
                        // The writer dropped its end: the walk is over.
                        return;
                    };
                    // Every job that was sent comes back exactly once,
                    // whatever happened to it - see `panic_error`.
                    let path = job.path.clone();
                    let prepared =
                        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            crate::ingest::fault::panic_preparing(&job.path);
                            crate::ingest::prepare(&job.path, job.session_key, &job.existing, mode)
                        })) {
                            Ok(prepared) => prepared,
                            Err(payload) => Err(panic_error(&path, payload)),
                        };
                    if done
                        .send(Done {
                            slot,
                            thread: std::thread::current().id(),
                            prepared,
                        })
                        .is_err()
                    {
                        return;
                    }
                }
            });
        }
        // The writer holds no sender of its own, so `done_rx` closes when the
        // last worker exits rather than never.
        drop(done_tx);

        // Across rounds, not per round: a link and its target can fall in two.
        let mut taken = HashMap::new();
        let mut outcome = Ok(());
        for batch in transcripts.chunks(round) {
            if let Err(e) = dispatch_round(
                store, rows, batch, summary, projects, threads, &mut taken, &job_tx, &done_rx,
            ) {
                outcome = Err(e);
                break;
            }
        }

        // Whatever happened, release the workers and let the scope join them.
        drop(job_tx);
        outcome
    })
}

/// Build one round's `Existing`, hand them out, drain the results and apply
/// them in discovery order.
#[allow(clippy::too_many_arguments)]
fn dispatch_round(
    store: &mut Store,
    rows: &Rows,
    batch: &[PathBuf],
    summary: &mut Summary,
    projects: &mut Resolver,
    threads: &mut HashSet<ThreadId>,
    taken: &mut HashMap<String, PathBuf>,
    job_tx: &mpsc::SyncSender<(usize, Job)>,
    done_rx: &mpsc::Receiver<Done>,
) -> Result<()> {
    // Two paths can share a key, and [`claim`] lets only the first through,
    // across rounds, so no two jobs in this run share one. That is what makes
    // reading every row once before the run, and building a whole round's
    // `Existing` before applying any of it, safe: no apply in the run can
    // change a row another job reads.
    let mut sent = 0usize;
    let mut results: Vec<Option<Result<Prepared>>> = (0..batch.len()).map(|_| None).collect();
    // The file each slot reads, which is a link's target, not the link.
    let mut files: Vec<PathBuf> = batch.to_vec();

    for (slot, path) in batch.iter().enumerate() {
        summary.files_walked += 1;
        let claimed = claim(rows, taken, path).and_then(|claim| match claim {
            Claim::Ingest { file, session_key } => {
                read_existing(store, rows, file, session_key).map(Some)
            }
            Claim::Same => Ok(None),
            Claim::Held(e) => Err(e),
        });
        let job = match claimed {
            Ok(Some(job)) => job,
            // Another path to a file this walk already took: nothing to do.
            Ok(None) => continue,
            // A path that is not UTF-8, a key another file holds, or a store
            // row this build refuses. Recorded and skipped like any other
            // per-file failure.
            Err(e) => {
                results[slot] = Some(Err(e));
                continue;
            }
        };
        files[slot] = job.path.clone();
        // Cannot block: the channel is bounded at the round size and this loop
        // sends at most one round.
        if job_tx.send((slot, job)).is_err() {
            // Every worker is gone. Nothing more can be prepared, so the rest
            // of this round is left unsent and the loop below drains what did.
            break;
        }
        sent += 1;
    }

    for _ in 0..sent {
        let Ok(done) = done_rx.recv() else {
            // Every worker exited without answering. That can only be a panic
            // inside `prepare`, which the scope will re-raise when it joins.
            break;
        };
        threads.insert(done.thread);
        results[done.slot] = Some(done.prepared);
    }

    for (slot, result) in results.into_iter().enumerate() {
        let Some(result) = result else { continue };
        commit_one(store, &files[slot], result, summary, projects);
    }
    Ok(())
}

/// The writer's half for one file: apply it, or record why it was skipped.
fn commit_one(
    store: &mut Store,
    path: &Path,
    prepared: Result<Prepared>,
    summary: &mut Summary,
    projects: &mut Resolver,
) {
    let applied = prepared.and_then(|prepared| crate::ingest::apply(store, prepared, projects));
    // The same handler the sequential walk ends each file with, so a file
    // counts, fails and flags a divergence the same way on both.
    driver::tally(store, summary, path, applied);
}

/// What the store already holds for one transcript, built on the writer thread
/// from the row read at the start of the run.
///
/// `path` is the file [`claim`] resolved, a link's target and never the link,
/// so the worker reads, and the store records, that file.
fn read_existing(store: &Store, rows: &Rows, path: PathBuf, session_key: String) -> Result<Job> {
    let existing = Existing::read(store.conn(), rows, &session_key, &path)?;
    Ok(Job {
        path,
        session_key,
        existing,
    })
}
