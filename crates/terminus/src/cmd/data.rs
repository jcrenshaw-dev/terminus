//! `terminus data move <path>`: put the whole data directory somewhere else and
//! record where it went.
//!
//! **The whole directory, never just the database.** The data directory holds
//! the `LOCK` file, the per-session injection-state directory
//! (`inject::decision::DIR_NAME`), the rolling snapshots and `terminus.db`'s two
//! WAL sidecars. A move that carried only `terminus.db` would strand the
//! undrained decision files, and the next pass would report "the decision log
//! could not be drained" into `runs.error` while the replay history sat at the
//! old path.
//!
//! **A copy and then a delete, never `fs::rename`.** A rename across
//! filesystems fails, and relocating onto another disk is the whole point of
//! the command.
//!
//! **Human-only, no `--json`.** It shows what it is about to do and asks
//! once, and a single JSON document on stdout cannot be both of those things -
//! the reason `main.rs` already gives for `install` and `uninstall`. `--yes` is
//! the way to skip the question, exactly as it is for `uninstall --purge`.
//!
//! **It touches no settings file.** The hook entries are written as
//! `args: ["hook", <event>]` and the MCP registration as `args: ["mcp"]`, so
//! neither names a store path and there is nothing there to rewrite.
//! Rewriting it would break the rule that hook entries are written once and
//! never rewritten.
//!
//! **It warns about network filesystems and asks anyway.** No `std` API
//! detects a network mount on Linux or macOS, and adding one would be a
//! platform-specific dependency on the hook-path binary. The
//! command asks regardless of destination, so an unconditional warning costs
//! nothing and is the whole mitigation.

use std::path::{Path, PathBuf};

use terminus_core::ingest::lock;
use terminus_core::store;

use super::Failure;

/// The one verb `data` takes.
const VERB: &str = "move";

/// The hazard nobody can detect for the user, said every time.
const NETWORK_WARNING: &str =
    "  sqlite is not safe on a network filesystem (nfs, smb, sshfs, or a synced folder\n  \
     that a background agent rewrites). terminus cannot detect one, so if that path is\n  \
     on a remote mount, answer no.";

pub struct Options {
    destination: PathBuf,
    yes: bool,
}

/// The second two-word subcommand in this binary, parsed the way
/// `cmd::observations::parse` parses the first.
///
/// The verb comes first or not at all, and any other second word is misuse
/// rather than a word that was silently ignored: `terminus data /somewhere`
/// reads like a move and must not be one.
pub fn parse(parser: &mut lexopt::Parser) -> Result<Options, Failure> {
    use lexopt::prelude::*;

    let mut verb: Option<String> = None;
    let mut destination: Option<PathBuf> = None;
    let mut yes = false;

    while let Some(arg) = parser.next().map_err(|e| Failure::Misuse(e.to_string()))? {
        match arg {
            Value(word) if verb.is_none() => {
                let word = word.to_string_lossy().into_owned();
                if word != VERB {
                    return Err(Failure::Misuse(format!(
                        "unknown `data` subcommand '{word}'; the only verb is `data {VERB} <path>`"
                    )));
                }
                verb = Some(word);
            }
            Value(path) if destination.is_none() => destination = Some(PathBuf::from(path)),
            Value(extra) => {
                return Err(Failure::Misuse(format!(
                    "`data {VERB}` takes one destination, and '{}' is a second",
                    extra.to_string_lossy()
                )))
            }
            Short('y') | Long("yes") => yes = true,
            // Including `--json`, deliberately.
            other => return Err(Failure::Misuse(crate::unexpected(other))),
        }
    }

    if verb.is_none() {
        return Err(Failure::Misuse(format!(
            "`data` needs a verb: `data {VERB} <path>`"
        )));
    }
    let Some(destination) = destination else {
        return Err(Failure::Misuse(format!(
            "`data {VERB}` needs the path to move the data directory to"
        )));
    };
    Ok(Options { destination, yes })
}

pub fn run(options: Options) -> Result<(), Failure> {
    let source = super::data_dir()?;
    let destination = absolute(&options.destination)?;
    let pointer = store::location_pointer_path()?;

    // Learned once, at the first check that meets an existing directory, and
    // reused by the second: a process's effective uid does not change between
    // the two.
    let mut uid = None;
    let mut learn_uid = || match uid {
        Some(known) => Ok(known),
        None => own_uid(&pointer).inspect(|&learned| uid = Some(learned)),
    };

    let found = refuse_impossible_destinations(&source, &destination, &mut learn_uid)?;

    outln!("terminus data move");
    outln!();
    outln!("  from  {}", source.display());
    outln!("  to    {}", destination.display());
    outln!("  {} byte(s), including the archive itself", bytes(&source));
    outln!();
    outln!("{NETWORK_WARNING}");
    outln!();

    if !answered(&source, options.yes)? {
        outln!("  nothing was moved.");
        return Ok(());
    }

    // Under the ingest lock, so a pass mid-transaction is never left writing
    // into a directory that has moved out from under it. `try_acquire` creates
    // the directory if it is not there yet, which is how a machine that has
    // never ingested can still choose where its store will land.
    let held = match lock::try_acquire(&source)? {
        lock::Attempt::Acquired(held) => held,
        lock::Attempt::Held => {
            return Err(Failure::Operational(
                "another terminus process holds the ingest lock; nothing was moved".to_owned(),
            ))
        }
    };

    // The second check, as late as it can run: after the answer and the
    // lock, right before the copy. The question can wait as long as the user
    // likes, and `copy_tree` writes into whatever directory is at the path, so
    // a directory swapped or opened up while it waited would otherwise receive
    // the store (#59). The whole rule runs again, and what it finds has to be
    // what the first check found: still nothing, or the same directory by
    // device and inode.
    let changed = |why: String| {
        Failure::Operational(format!(
            "{} changed while the question waited, so nothing was copied: {why}\n  \
             the store and the pointer are as they were",
            destination.display()
        ))
    };
    match refuse_unchecked_destination(&destination, &mut learn_uid) {
        Ok(now) if now == found => {}
        Ok(Found::Nothing) => return Err(changed("it was there and now is not".to_owned())),
        Ok(_) if found == Found::Nothing => {
            return Err(changed("it was not there and now is".to_owned()))
        }
        Ok(_) => return Err(changed("it is a different directory now".to_owned())),
        Err(Failure::Operational(why)) => return Err(changed(why)),
        Err(other) => return Err(other),
    }

    // The second check cannot hold the path still, so a destination both
    // checks found absent is made here with one exclusive `mkdir`, which a
    // directory, or a symlink to one, that another account put there after
    // the check fails rather than receives the store (#59). `copy_tree` makes
    // no directory at the top, so this is the only place an absent
    // destination is created.
    if found == Found::Nothing {
        create_destination(&destination).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                changed("it was not there and now is".to_owned())
            } else {
                Failure::Operational(format!(
                    "{} could not be created: {e}; nothing was moved",
                    destination.display()
                ))
            }
        })?;
    }

    copy_tree(&source, &destination).map_err(|e| {
        Failure::Operational(format!(
            "{e}\n  nothing was deleted, and the location still points at {}.\n  \
             what was copied so far is at {}",
            source.display(),
            destination.display()
        ))
    })?;

    // The lock file's own bytes carry no meaning - `ingest::lock` says so, and
    // says only the OS lock attached to it does - so the destination gets a
    // fresh empty one rather than a copy of the file this process is holding
    // locked.
    //
    // Owner-only from the creating open, like the one `ingest::lock` makes at
    // the source: the lock file is short, but a `data move`
    // that left one file in the new directory readable by the world would make
    // the destination's mode a lie on the very first listing.
    let lock_file = destination.join(lock::LOCK_FILE_NAME);
    terminus_core::owner_only::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_file)
        .map_err(|e| {
            Failure::Operational(format!("{} could not be created: {e}", lock_file.display()))
        })?;

    // Before the source is removed, and atomically: a crash between the copy
    // and the pointer leaves both directories intact and the store still
    // resolving to the old one, which is recoverable. The other order loses the
    // store's address with the data still on disk.
    write_pointer(&pointer, &destination)?;

    // Only now, with the pointer naming the destination: holding the lock this
    // long kept an ingest out of the source from the copy until the pointer
    // was written. Releasing it before the source is removed is an order kept
    // as it was, and harmless, since Linux and macOS remove a held file anyway.
    drop(held);
    std::fs::remove_dir_all(&source).map_err(|e| {
        Failure::Operational(format!(
            "the move is done and {} could not be removed: {e}\n  \
             it is now an unused copy and can be deleted by hand",
            source.display()
        ))
    })?;

    outln!("  moved. the store is at {}", destination.display());
    outln!("  recorded in {}", pointer.display());
    outln!("  nothing in settings.json changed: the hook and mcp entries never named a store path");
    Ok(())
}

/// The destinations that cannot be honored, refused before anything is printed
/// or asked: the data directory itself, a path inside it, and anything
/// [`refuse_unchecked_destination`] refuses.
///
/// The nested case is the one worth naming: copying a directory into a
/// subdirectory of itself walks the copy it is making, so it is refused rather
/// than allowed to fill the disk.
fn refuse_impossible_destinations(
    source: &Path,
    destination: &Path,
    uid: impl FnOnce() -> Result<u32, Failure>,
) -> Result<Found, Failure> {
    if destination == source {
        return Err(Failure::Operational(format!(
            "{} is where the data directory already is",
            destination.display()
        )));
    }
    if destination.starts_with(source) {
        return Err(Failure::Operational(format!(
            "{} is inside the data directory being moved",
            destination.display()
        )));
    }
    refuse_unchecked_destination(destination, uid)
}

/// What the destination rule found, for the second check to compare against.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Found {
    /// Nothing at the path; [`create_destination`] will create it 0700.
    Nothing,
    /// An empty owner-only directory, named by what a rename cannot carry.
    Directory { device: u64, inode: u64 },
}

/// The destination rule: a path that does not exist, or an empty
/// directory this account owns that nobody else can reach.
///
/// An existing destination passes only when it is not a symlink, is a
/// directory, is owned by `uid`, has no group or world bit, and can be listed
/// and is empty. Anything else refuses, including an error other than
/// `NotFound` from the first look and a directory that cannot be listed, which
/// is never read as empty. A missing path is accepted and
/// [`create_destination`] creates it 0700.
///
/// Only the last part is checked for a symlink, so a symlinked parent such as
/// macOS's `/var/folders` is fine, and the parent's owner and mode are not
/// checked. A wide directory is refused rather than narrowed, because
/// terminus narrows what it creates and nothing it merely found. `uid`
/// is a closure because learning it costs a probe file, and a destination that
/// does not exist has no owner to compare. What it returns is what the second
/// check after the question compares against.
fn refuse_unchecked_destination(
    destination: &Path,
    uid: impl FnOnce() -> Result<u32, Failure>,
) -> Result<Found, Failure> {
    use std::os::unix::fs::MetadataExt;

    let refused = |why: String| {
        Failure::Operational(format!(
            "{} {why}; nothing was moved",
            destination.display()
        ))
    };

    let found = match std::fs::symlink_metadata(destination) {
        Ok(found) => found,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Found::Nothing),
        Err(e) => return Err(refused(format!("could not be checked: {e}"))),
    };
    if found.file_type().is_symlink() {
        return Err(refused(
            "is a symlink, and the store would land wherever it points.\n  \
             name the directory itself"
                .to_owned(),
        ));
    }
    if !found.is_dir() {
        return Err(refused("is not a directory".to_owned()));
    }
    let expected = uid()?;
    if found.uid() != expected {
        return Err(refused(format!(
            "is owned by uid {}, not by this account (uid {expected})",
            found.uid()
        )));
    }
    if found.mode() & 0o077 != 0 {
        return Err(refused(format!(
            "is mode {:o}, open to other accounts.\n  \
             make it owner-only with: chmod 700 '{}'",
            found.mode() & 0o7777,
            destination.display()
        )));
    }
    let mut entries =
        std::fs::read_dir(destination).map_err(|e| refused(format!("could not be listed: {e}")))?;
    match entries.next() {
        None => Ok(Found::Directory {
            device: found.dev(),
            inode: found.ino(),
        }),
        Some(Ok(_)) => Err(refused(
            "already holds files.\n  name a path that does not exist or is empty".to_owned(),
        )),
        Some(Err(e)) => Err(refused(format!("could not be listed: {e}"))),
    }
}

/// This process's uid, read off a file it has just created.
///
/// A new file's owner is the creating process's effective uid on Linux and
/// macOS, so this needs `std` alone and no `libc`. The probe goes in the
/// pointer's directory: terminus's own config directory, which this command
/// has to write the pointer into anyway. A taken name is stepped over rather
/// than opened, and a probe that cannot be made refuses the move.
fn own_uid(pointer: &Path) -> Result<u32, Failure> {
    use std::os::unix::fs::MetadataExt;

    let Some(dir) = pointer.parent() else {
        return Err(Failure::Operational(format!(
            "{} has no parent directory to write into",
            pointer.display()
        )));
    };
    let failed = |path: &Path, e: std::io::Error| {
        Failure::Operational(format!(
            "{} could not be created to learn this account's uid: {e}; nothing was moved",
            path.display()
        ))
    };

    std::fs::create_dir_all(dir).map_err(|e| failed(dir, e))?;
    let pid = std::process::id();
    for n in 0..100 {
        let probe = dir.join(format!(".uid-probe-{pid}-{n}"));
        match terminus_core::owner_only::options()
            .write(true)
            .create_new(true)
            .open(&probe)
        {
            Ok(file) => {
                let owner = file.metadata().map(|m| m.uid());
                let _ = std::fs::remove_file(&probe);
                return owner.map_err(|e| failed(&probe, e));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(failed(&probe, e)),
        }
    }
    Err(Failure::Operational(format!(
        "every probe name under {} for pid {pid} is taken, so this account's uid could not be \
         learned; nothing was moved",
        dir.display()
    )))
}

/// The question, with `--yes` meaning "do it" rather than "take the default".
///
/// [`super::install::confirm`] answers with the DEFAULT under `--yes`, and the
/// default for a thing that rewrites where the archive lives is no - so a
/// `data move --yes` that went through it unchanged would print the question,
/// decline it and leave. Everything else about the question is install's,
/// including the refusal when there is no answer to read.
fn answered(source: &Path, yes: bool) -> Result<bool, Failure> {
    const QUESTION: &str = "  move it?";

    if yes {
        outln!("{QUESTION} [y/N] y (--yes)");
        return Ok(true);
    }
    super::install::confirm(QUESTION, false, false).map_err(|_| {
        Failure::Operational(format!(
            "data move needs an answer and there was none to read; {} is untouched.\n  \
             rerun with --yes to move it without being asked",
            source.display()
        ))
    })
}

/// The pointer `store::open::data_dir` reads, written through a temporary file
/// and a rename.
///
/// Atomic because the alternative is a truncated pointer: a process killed
/// mid-write would leave a file naming half a path, and the next command would
/// resolve a store that does not exist rather than either the old one or the
/// new one.
fn write_pointer(pointer: &Path, destination: &Path) -> Result<(), Failure> {
    let Some(dir) = pointer.parent() else {
        return Err(Failure::Operational(format!(
            "{} has no parent directory to write into",
            pointer.display()
        )));
    };
    let io = |path: &Path, e: std::io::Error| {
        Failure::Operational(format!("{} could not be written: {e}", path.display()))
    };

    std::fs::create_dir_all(dir).map_err(|e| io(dir, e))?;
    let temporary = pointer.with_extension("tmp");
    std::fs::write(&temporary, format!("{}\n", destination.display()))
        .map_err(|e| io(&temporary, e))?;
    std::fs::rename(&temporary, pointer).map_err(|e| io(pointer, e))
}

/// A relative destination resolved against the working directory, with
/// trailing slashes and `.` parts dropped.
///
/// The pointer file holds one ABSOLUTE path and the resolver refuses anything
/// else, because a relative pointer would resolve against whatever directory a
/// hook happened to be spawned in - a store per repository rather than one
/// store. So the resolution happens here, where a working directory still means
/// what the user meant by it.
///
/// The slash matters because `symlink_metadata("link/")` follows the link, so
/// the symlink check would pass a path the copy then writes through. Every
/// later use takes this one path, so what was checked and what the pointer
/// records cannot differ. A `..` stays as written: resolving it as text picks
/// a different directory when the part before it is a symlink.
fn absolute(path: &Path) -> Result<PathBuf, Failure> {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir().map_err(|e| {
            Failure::Operational(format!(
                "{} is a relative path and the working directory could not be read: {e}",
                path.display()
            ))
        })?;
        cwd.join(path)
    };
    // `components` drops the trailing slash, repeated slashes and every `.`
    // after the first part, and keeps `..`.
    Ok(joined.components().collect())
}

/// The destination, created 0700 by this process or not at all.
///
/// The parents are made at the process default, the way
/// `owner_only::create_dir_all` makes them, but the leaf is one exclusive
/// `mkdir`: anything already at the path, a directory or a symlink, fails it
/// with `AlreadyExists` instead of being taken as the destination.
fn create_destination(destination: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::DirBuilder::new()
        .mode(terminus_core::owner_only::DIR_MODE)
        .create(destination)
}

/// Everything under `from`, copied into `to`, directories included.
///
/// `to` is already there: [`run`] found it as an empty owner-only directory or
/// made it with [`create_destination`], and nothing here makes a directory at
/// the top that could stand in for either. Each directory below it is one
/// exclusive `mkdir`, for the reason [`copy_file`] uses `create_new`.
///
/// The `LOCK` file is skipped at the top level: this process is holding it, and
/// [`run`] creates an empty one at the destination instead.
fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    use std::os::unix::fs::DirBuilderExt;

    let failed =
        |path: &Path, e: std::io::Error| format!("{} could not be copied: {e}", path.display());

    // Every directory 0700 and every file 0600, written rather than reproduced.
    // A plain byte-for-byte copy carries the SOURCE's mode across with it,
    // which would make this the one command that can undo owner-only storage:
    // a store restored out of a tar, or opened up by hand, would arrive as
    // wide as it was at the old location.
    // What the destination gets is the mode this build writes, from the
    // creating syscall, whatever the source happens to be.
    let entries = std::fs::read_dir(from).map_err(|e| failed(from, e))?;
    for entry in entries {
        let entry = entry.map_err(|e| failed(from, e))?;
        let name = entry.file_name();
        if name == std::ffi::OsStr::new(lock::LOCK_FILE_NAME) {
            continue;
        }
        let source = entry.path();
        let destination = to.join(&name);
        let kind = entry.file_type().map_err(|e| failed(&source, e))?;
        if kind.is_dir() {
            std::fs::DirBuilder::new()
                .mode(terminus_core::owner_only::DIR_MODE)
                .create(&destination)
                .map_err(|e| failed(&source, e))?;
            copy_tree(&source, &destination)?;
        } else {
            copy_file(&source, &destination).map_err(|e| failed(&source, e))?;
        }
    }
    Ok(())
}

/// One file copied into a destination this process creates at 0600.
///
/// `create_new`, because [`refuse_unchecked_destination`] has established
/// that the destination was absent, or an empty owner-only directory this
/// account owns: a name already taken here means something else is writing
/// into the directory mid-move, and truncating it would destroy a file this
/// command never wrote.
fn copy_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    let mut reader = std::fs::File::open(source)?;
    let mut writer = terminus_core::owner_only::options()
        .write(true)
        .create_new(true)
        .open(destination)?;
    std::io::copy(&mut reader, &mut writer)?;
    Ok(())
}

/// What the move is about to carry, in bytes.
///
/// Every file under the directory rather than the three `cmd::status` sums:
/// what moves here is the injection-state directory (`decision::DIR_NAME`) and
/// the snapshots too, and a size that named only the database would understate
/// a directory holding three rolling copies of it by a factor of four.
fn bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0;
    for entry in entries.flatten() {
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => total += bytes(&entry.path()),
            Ok(_) => total += entry.metadata().map(|m| m.len()).unwrap_or(0),
            Err(_) => {}
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalized(path: &str) -> PathBuf {
        match absolute(Path::new(path)) {
            Ok(path) => path,
            Err(_) => panic!("{path} did not resolve"),
        }
    }

    #[test]
    fn a_trailing_slash_is_dropped() {
        assert_eq!(normalized("/x/store/"), PathBuf::from("/x/store"));
        assert_eq!(normalized("/x/store//"), PathBuf::from("/x/store"));
    }

    #[test]
    fn dot_parts_are_dropped() {
        assert_eq!(normalized("/x/./store/."), PathBuf::from("/x/store"));
    }

    #[test]
    fn a_dot_dot_part_is_kept() {
        assert_eq!(
            normalized("/x/link/../store/"),
            PathBuf::from("/x/link/../store")
        );
    }

    /// An empty 0700 directory in a fresh temporary directory, and its owner.
    fn owner_only_dir() -> (tempfile::TempDir, PathBuf, u32) {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("store");
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let owner = std::fs::metadata(&path).unwrap().uid();
        (dir, path, owner)
    }

    fn refusal(destination: &Path, uid: u32) -> String {
        match refuse_unchecked_destination(destination, || Ok(uid)) {
            Err(Failure::Operational(message)) => message,
            Err(_) => panic!(
                "{} was refused, but not operationally",
                destination.display()
            ),
            Ok(_) => panic!("{} was accepted", destination.display()),
        }
    }

    #[test]
    fn an_empty_owner_only_directory_this_uid_owns_is_accepted() {
        let (_dir, path, owner) = owner_only_dir();
        assert!(refuse_unchecked_destination(&path, || Ok(owner)).is_ok());
    }

    #[test]
    fn a_directory_another_uid_owns_is_refused_and_the_owner_named() {
        let (_dir, path, owner) = owner_only_dir();
        let message = refusal(&path, owner.wrapping_add(1));
        assert!(
            message.contains(&format!("owned by uid {owner}")),
            "{message}"
        );
    }

    #[test]
    fn an_owner_only_directory_that_holds_a_file_is_refused() {
        let (_dir, path, owner) = owner_only_dir();
        std::fs::write(path.join("someone-elses.txt"), "hands off\n").unwrap();
        assert!(refusal(&path, owner).contains("already holds files"));
    }

    #[test]
    fn a_regular_file_is_refused() {
        let (dir, _path, owner) = owner_only_dir();
        let file = dir.path().join("file");
        std::fs::write(&file, "").unwrap();
        assert!(refusal(&file, owner).contains("is not a directory"));
    }

    #[test]
    fn a_path_beneath_a_regular_file_is_refused() {
        let (dir, _path, owner) = owner_only_dir();
        let file = dir.path().join("file");
        std::fs::write(&file, "").unwrap();
        assert!(refusal(&file.join("store"), owner).contains("could not be checked"));
    }

    #[test]
    fn a_destination_is_created_owner_only_with_its_parents() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("parent").join("store");
        create_destination(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "the destination was created at {mode:o}");
    }

    #[test]
    fn a_directory_already_at_the_destination_is_not_taken() {
        let (_dir, path, _owner) = owner_only_dir();
        let e = create_destination(&path).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn a_symlink_already_at_the_destination_is_not_taken() {
        let (dir, target, _owner) = owner_only_dir();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let e = create_destination(&link).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn a_relative_path_is_resolved_and_normalized() {
        let cwd = std::env::current_dir().unwrap();
        assert_eq!(normalized("rel/./"), cwd.join("rel"));
    }
}
