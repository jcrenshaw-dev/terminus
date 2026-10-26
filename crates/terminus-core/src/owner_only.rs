//! Creating a file or a directory owner-only, at the moment it is created.
//!
//! Nothing terminus writes is readable by group or world, and the mode is
//! carried IN the creating syscall rather than applied to a path that already
//! exists. That is not tidiness: a `create` followed by a `chmod` leaves a
//! window in which the file is on disk at the umask's mode with the first
//! bytes already in it, and the file this project writes holds whole session
//! transcripts. Every creation site in the store, the ingest lock, the
//! snapshot copy, the injection scratch and the export goes through the two
//! helpers below, so there is one place to read the rule off.
//!
//! `std` alone does it: [`std::os::unix::fs::DirBuilderExt::mode`] and
//! [`std::os::unix::fs::OpenOptionsExt::mode`]. No `libc`, no `nix` - the hook
//! path is measured in fractions of a millisecond and does not pay for a crate
//! to set two integers.

use std::path::{Path, PathBuf};

/// Owner-only for a directory: `rwx` for the owner, nothing for anyone else.
pub const DIR_MODE: u32 = 0o700;

/// Owner-only for a file: `rw` for the owner, nothing for anyone else.
pub const FILE_MODE: u32 = 0o600;

/// `create_dir_all`, except the LEAF lands [`DIR_MODE`].
///
/// The parent chain is created at the process default and the leaf alone
/// carries the mode. A recursive [`std::fs::DirBuilder`] with a mode set
/// would be one line shorter and wrong: std applies that builder's mode to
/// every component it creates, so the first ingest on a machine whose
/// `~/.local/share` did not exist yet would leave `~/.local/share` itself 0700
/// and every other tool's data under it unreachable from the user's own
/// group-shared workflows. terminus narrows its own directory and no directory
/// above it.
///
/// A leaf that is already there is success and keeps whatever mode it has:
/// narrowing an existing directory is the repair in `store::open`, which runs
/// once per open over a known list, and not a side effect of every writer that
/// happens to call this. A path occupied by something that is not a directory
/// is still the error `std::fs::create_dir_all` would have returned.
pub fn create_dir_all(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut builder = std::fs::DirBuilder::new();
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(DIR_MODE);
    }

    match builder.create(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => Ok(()),
        Err(e) => Err(e),
    }
}

/// [`std::fs::OpenOptions`] already carrying [`FILE_MODE`], for the caller to
/// finish.
///
/// The mode applies only when the open creates the file, which is what makes
/// this safe to hand to every writer regardless of its other flags: the caller
/// adds its own `create`/`create_new`/`truncate`/`read`/`write`, so the ingest
/// lock's never-truncate open and the injection scratch's exclusive create both
/// fit without a second helper. Opening a file that already exists changes no
/// mode here.
pub fn options() -> std::fs::OpenOptions {
    let mut options = std::fs::OpenOptions::new();
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(FILE_MODE);
    }
    options
}

/// A new file at `path`, created exclusively and [`FILE_MODE`] from the open.
///
/// The mode rides the creating syscall, for the reason the module header gives.
/// `create_new` is `O_EXCL`, so anything already at the path, a symlink
/// included, fails with [`std::io::ErrorKind::AlreadyExists`] and is never
/// followed and truncated. A taken name is for the caller to answer with
/// another name, never to reuse.
pub fn create_new(path: &Path) -> std::io::Result<std::fs::File> {
    options().write(true).create_new(true).open(path)
}

/// [`create_new`] at the first free name in `dir`, and the path it landed at.
///
/// `name` gives the file name for each attempt number, counting from 0, and the
/// caller sets the limit, so each site keeps the names it has always made. A
/// taken name moves on to the next one: whatever is at it is not ours, so it is
/// never opened, and a stale symlink there is stepped over rather than followed
/// (`O_EXCL`, as [`create_new`]). Any other error returns at once. When every
/// name was taken the error is [`std::io::ErrorKind::AlreadyExists`], so a
/// caller can tell exhaustion from every other failure.
pub fn create_temporary(
    dir: &Path,
    attempts: u32,
    name: impl Fn(u32) -> String,
) -> std::io::Result<(PathBuf, std::fs::File)> {
    first_free(dir, attempts, name, create_new)
}

/// [`create_temporary`] for a symlink to `target`.
///
/// `symlink` also fails on a name that is taken, so each attempt is as
/// exclusive as a [`create_new`], and the same rule steps over what is there.
pub fn create_temporary_symlink(
    dir: &Path,
    attempts: u32,
    name: impl Fn(u32) -> String,
    target: &Path,
) -> std::io::Result<PathBuf> {
    first_free(dir, attempts, name, |path| {
        std::os::unix::fs::symlink(target, path)
    })
    .map(|(path, ())| path)
}

fn first_free<T>(
    dir: &Path,
    attempts: u32,
    name: impl Fn(u32) -> String,
    create: impl Fn(&Path) -> std::io::Result<T>,
) -> std::io::Result<(PathBuf, T)> {
    for attempt in 0..attempts {
        let path = dir.join(name(attempt));
        match create(&path) {
            Ok(made) => return Ok((path, made)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!("all {attempts} names in {} are taken", dir.display()),
    ))
}

/// A new directory at `path`, [`DIR_MODE`] from the `mkdir`.
///
/// The parents are made at the process default, as [`create_dir_all`] makes
/// them. The leaf is one exclusive `mkdir`: anything already at it, a directory
/// or a symlink to one, is [`std::io::ErrorKind::AlreadyExists`] and is never
/// taken as the new directory.
pub fn create_dir_new(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut builder = std::fs::DirBuilder::new();
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(DIR_MODE);
    }
    builder.create(path)
}

/// Write `bytes` to `path` so a reader sees the old file or the new one, never
/// a prefix of either, and the new one is [`FILE_MODE`].
///
/// The bytes go into a [`create_temporary`] beside `path`, named
/// `{prefix}{pid}-{n}{suffix}` over 64 attempts, and the rename puts it in
/// place. A failure removes the temporary and returns the error. The caller
/// makes the directory.
///
/// No `fsync`. The rename gives the reader its atomicity, and a flush would put
/// a disk wait on every caller, the hook's state save among them.
pub fn replace(path: &Path, bytes: &[u8], prefix: &str, suffix: &str) -> std::io::Result<()> {
    use std::io::Write as _;

    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let pid = std::process::id();
    let (temporary, mut file) =
        create_temporary(dir, 64, |n| format!("{prefix}{pid}-{n}{suffix}"))?;
    let written = file.write_all(bytes).and_then(|()| file.flush());
    drop(file);
    let outcome = written.and_then(|()| std::fs::rename(&temporary, path));
    if outcome.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    /// The leaf is ours to narrow and the parents are not.
    #[test]
    fn only_the_leaf_directory_is_owner_only() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("a/b/leaf");

        create_dir_all(&leaf).unwrap();

        assert_eq!(mode(&leaf), 0o700, "the leaf terminus owns");
        assert_eq!(
            mode(&root.path().join("a")),
            0o755,
            "a parent created on the way keeps the umask's own default"
        );
        assert_eq!(
            mode(&root.path().join("a/b")),
            0o755,
            "and so does the next"
        );
    }

    /// An existing directory is left exactly as it was found: the repair in
    /// `store::open` narrows one, not every writer that passes through here.
    ///
    /// The wide leaf is made by plain [`std::fs::create_dir`] at the process
    /// umask - 0755 under the umask 022 these tests run at - rather than by a
    /// `chmod`, because widening then narrowing is the very thing this module
    /// exists to avoid and no mode is re-applied to an existing path anywhere
    /// in this file.
    #[test]
    fn an_existing_leaf_is_success_and_keeps_its_mode() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("leaf");
        std::fs::create_dir(&leaf).unwrap();
        let before = mode(&leaf);

        create_dir_all(&leaf).unwrap();

        assert_eq!(mode(&leaf), before, "an existing leaf is not re-moded");
    }

    /// A file sitting where a directory should be is still an error, the same
    /// one `std::fs::create_dir_all` returns.
    #[test]
    fn a_file_in_the_leafs_place_is_still_an_error() {
        let root = tempfile::tempdir().unwrap();
        let occupied = root.path().join("occupied");
        std::fs::write(&occupied, b"not a directory").unwrap();

        let error = create_dir_all(&occupied).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn a_created_file_is_owner_only() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("secret");

        options().write(true).create_new(true).open(&path).unwrap();

        assert_eq!(mode(&path), 0o600);
    }

    /// The lock file's shape: created if missing, never truncated, and still
    /// 0600 the moment it exists.
    #[test]
    fn a_create_without_truncate_is_owner_only_too() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("LOCK");

        options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .unwrap();

        assert_eq!(mode(&path), 0o600);
    }

    /// The single create lands 0600, and refuses what is already there without
    /// following it.
    #[test]
    fn a_single_create_is_owner_only_and_refuses_anything_taken() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("new");

        create_new(&path).unwrap();
        assert_eq!(mode(&path), 0o600);
        assert_eq!(
            create_new(&path).unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );

        let victim = root.path().join("victim");
        std::fs::write(&victim, b"keep me").unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&victim, &link).unwrap();
        assert_eq!(
            create_new(&link).unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(std::fs::read(&victim).unwrap(), b"keep me");
    }

    fn numbered(n: u32) -> String {
        format!("temp-{n}")
    }

    /// A file and then a stale link at the first two names: the third is the
    /// one made, and the link's target is untouched.
    #[test]
    fn the_retry_steps_over_a_file_and_a_stale_symlink() {
        let root = tempfile::tempdir().unwrap();
        let victim = root.path().join("victim");
        std::fs::write(&victim, b"keep me").unwrap();
        std::fs::write(root.path().join("temp-0"), b"taken").unwrap();
        std::os::unix::fs::symlink(&victim, root.path().join("temp-1")).unwrap();

        let (path, _file) = create_temporary(root.path(), 64, numbered).unwrap();

        assert_eq!(path, root.path().join("temp-2"));
        assert_eq!(mode(&path), 0o600);
        assert_eq!(std::fs::read(&victim).unwrap(), b"keep me");
    }

    #[test]
    fn the_retry_reports_exhaustion_as_already_exists() {
        let root = tempfile::tempdir().unwrap();
        for n in 0..3 {
            std::fs::write(root.path().join(numbered(n)), b"taken").unwrap();
        }

        let error = create_temporary(root.path(), 3, numbered).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn the_retry_in_a_missing_directory_is_not_found_at_once() {
        let root = tempfile::tempdir().unwrap();
        let attempts = std::cell::Cell::new(0);

        let error = create_temporary(&root.path().join("absent"), 64, |n| {
            attempts.set(attempts.get() + 1);
            numbered(n)
        })
        .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(attempts.get(), 1);
    }

    #[test]
    fn the_symlink_form_links_at_the_first_free_name() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        std::fs::write(root.path().join("temp-0"), b"taken").unwrap();

        let path = create_temporary_symlink(root.path(), 64, numbered, &target).unwrap();

        assert_eq!(path, root.path().join("temp-1"));
        assert_eq!(std::fs::read_link(&path).unwrap(), target);
        assert_eq!(std::fs::read(root.path().join("temp-0")).unwrap(), b"taken");
    }

    #[test]
    fn a_new_directory_is_owner_only_under_parents_at_the_default() {
        let root = tempfile::tempdir().unwrap();
        let leaf = root.path().join("a/leaf");

        create_dir_new(&leaf).unwrap();

        assert_eq!(mode(&leaf), 0o700);
        assert_eq!(mode(&root.path().join("a")), 0o755);
    }

    #[test]
    fn a_new_directory_refuses_a_directory_and_a_link_to_one() {
        let root = tempfile::tempdir().unwrap();
        let existing = root.path().join("existing");
        std::fs::create_dir(&existing).unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(&existing, &link).unwrap();

        for path in [&existing, &link] {
            assert_eq!(
                create_dir_new(path).unwrap_err().kind(),
                std::io::ErrorKind::AlreadyExists,
                "{}",
                path.display()
            );
        }
    }

    /// A new target lands owner-only and holds exactly the bytes.
    #[test]
    fn a_replace_lands_owner_only_with_exactly_the_bytes() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");

        replace(&target, b"the bytes", ".t-", ".tmp").unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"the bytes");
        assert_eq!(mode(&target), 0o600);
    }

    /// The rename puts a new inode at the target, and nothing else is left in
    /// the directory.
    #[test]
    fn a_replace_renames_over_the_target_and_leaves_nothing_else() {
        use std::os::unix::fs::MetadataExt;

        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        std::fs::write(&target, b"old").unwrap();
        let before = std::fs::metadata(&target).unwrap().ino();

        replace(&target, b"new", ".t-", ".tmp").unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert_ne!(std::fs::metadata(&target).unwrap().ino(), before);
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_replace_steps_over_a_stale_symlink_at_the_first_temporary_name() {
        let root = tempfile::tempdir().unwrap();
        let victim = root.path().join("victim");
        std::fs::write(&victim, b"keep me").unwrap();
        let first = format!(".t-{}-0.tmp", std::process::id());
        std::os::unix::fs::symlink(&victim, root.path().join(first)).unwrap();
        let target = root.path().join("target");

        replace(&target, b"new", ".t-", ".tmp").unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        assert_eq!(std::fs::read(&victim).unwrap(), b"keep me");
    }

    /// A rename that fails removes its temporary.
    #[test]
    fn a_failed_replace_leaves_no_temporary() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("inside"), b"occupied").unwrap();

        replace(&target, b"new", ".t-", ".tmp").unwrap_err();

        let leftovers: Vec<_> = std::fs::read_dir(root.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .filter(|name| name.to_string_lossy().starts_with(".t-"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }
}
