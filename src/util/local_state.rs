//! Where SpecSync keeps the state it writes for itself about a project: the hash cache, the
//! transaction journal and the remote spec cache (REQ-util-006).
//!
//! None of it is ever kept where a repository, or an archive of one, can put a file: not in the
//! work tree, where any spelling a case-insensitive disk folds onto the real name would do, and
//! not in a Git directory, which a tarball of a working copy carries whole. It lives only in the
//! user's cache folder, in a folder keyed by the canonical project root, and a cache folder that
//! itself lies inside the project is refused.

use cap_primitives::fs::FollowSymlinks;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, Metadata, OpenOptions};
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The folder SpecSync makes for itself in a Git directory or in the user's cache folder.
const STATE_FOLDER: &str = "specsync";
/// Under the user's cache folder, the folder that holds one folder per project.
const PROJECTS_FOLDER: &str = "projects";
/// Under the user's cache folder, the folder that holds one folder per Git repository.
const REPOSITORIES_FOLDER: &str = "repositories";

/// A local state location SpecSync will not use, with the reason as the user is shown it.
/// Carried inside an `io::Error` so a caller can tell it from a failure of the filesystem.
#[derive(Debug)]
pub struct RefusedLocalState(pub String);

impl fmt::Display for RefusedLocalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RefusedLocalState {}

fn refused(reason: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, RefusedLocalState(reason))
}

/// Where the local state of the project at `root` lives: a folder that exists or may be made
/// (`base`), and the names beneath it that SpecSync makes itself.
struct Location {
    base: PathBuf,
    names: Vec<String>,
}

impl Location {
    fn path(&self) -> PathBuf {
        let mut path = self.base.clone();
        path.extend(&self.names);
        path
    }
}

/// The first 16 hex digits of SHA-256 over `kind` and `path`: a folder name that says which
/// project the state belongs to without carrying any of the project's own spelling.
fn key(kind: &str, path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(path.as_os_str().as_encoded_bytes());
    let digest = format!("{:x}", hasher.finalize());
    digest[..16].to_string()
}

/// The user's cache folder: `$XDG_CACHE_HOME` when it is an absolute path, else
/// `~/Library/Caches` on macOS, `%LOCALAPPDATA%` on Windows and `~/.cache` elsewhere. Unit tests
/// use a folder in the system temporary directory, so they never touch the cache folder of
/// whoever runs them.
fn user_cache_folder() -> Option<PathBuf> {
    if cfg!(test) {
        return Some(std::env::temp_dir().join("specsync-unit-test-cache-home"));
    }
    let absolute = |value: std::ffi::OsString| {
        let path = PathBuf::from(value);
        path.is_absolute().then_some(path)
    };
    if let Some(folder) = std::env::var_os("XDG_CACHE_HOME").and_then(absolute) {
        return Some(folder);
    }
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA").and_then(absolute);
    }
    let home = std::env::var_os("HOME").and_then(absolute)?;
    Some(if cfg!(target_os = "macos") {
        home.join("Library").join("Caches")
    } else {
        home.join(".cache")
    })
}

/// `path` as it is on disk: its longest existing ancestor canonicalized, and the rest of the
/// path, which does not exist yet, joined to it.
fn resolved(path: &Path) -> PathBuf {
    let mut existing = path;
    let mut rest = Vec::new();
    loop {
        if let Ok(canonical) = existing.canonicalize() {
            let mut resolved = canonical;
            resolved.extend(rest.iter().rev());
            return resolved;
        }
        match (existing.parent(), existing.file_name()) {
            (Some(parent), Some(name)) => {
                rest.push(name.to_os_string());
                existing = parent;
            }
            _ => return path.to_path_buf(),
        }
    }
}

fn location(root: &Path) -> io::Result<Location> {
    let canonical = root.canonicalize()?;
    let base = user_cache_folder().ok_or_else(|| {
        refused(
            "there is no user cache folder (set HOME or XDG_CACHE_HOME) to keep SpecSync's own \
             state in"
                .to_string(),
        )
    })?;
    // Some CI containers set `HOME` or `XDG_CACHE_HOME` to the checkout. A cache folder inside
    // the project, or inside the folder its files are confined to, is a place a repository can
    // put a file, so it is refused rather than trusted (REQ-util-006).
    let on_disk = resolved(&base);
    if on_disk.starts_with(&canonical) || on_disk.starts_with(super::confinement_top(&canonical)) {
        return Err(refused(format!(
            "the user cache folder {} lies inside the project, where a repository can put a \
             file, so SpecSync keeps none of its own state there; set XDG_CACHE_HOME to a folder \
             outside the project",
            base.display()
        )));
    }
    let names = vec![
        STATE_FOLDER.to_string(),
        PROJECTS_FOLDER.to_string(),
        key("project", &canonical),
    ];
    // The project's own folder is compared too, not only the base: a project at
    // `<cache>/specsync` would otherwise hold its own state (REQ-util-006).
    let folder = names
        .iter()
        .fold(base.clone(), |path, name| path.join(name));
    let folder_on_disk = resolved(&folder);
    if folder_on_disk.starts_with(&canonical)
        || folder_on_disk.starts_with(super::confinement_top(&canonical))
    {
        return Err(refused(format!(
            "SpecSync's own folder for this project, {}, would lie inside the project, where a \
             repository can put a file, so SpecSync keeps none of its own state there; set \
             XDG_CACHE_HOME to a folder outside the project",
            folder.display()
        )));
    }
    Ok(Location { base, names })
}

/// The folder where SpecSync keeps what it knows about the Git repository whose canonical
/// common directory is `common_dir`, for the project at `root`: `specsync/repositories/<key>` in
/// the user's cache folder, shared by the repository's worktrees and never in its Git directory
/// (REQ-util-009). Refused, as for the project's own folder, when the user cache folder lies
/// inside the project.
pub fn repository_state_dir(root: &Path, common_dir: &Path) -> io::Result<PathBuf> {
    let project = location(root)?;
    Ok(project
        .base
        .join(STATE_FOLDER)
        .join(REPOSITORIES_FOLDER)
        .join(key("repository", common_dir)))
}

/// The folder where SpecSync keeps its own state about the project at `root`, in the user's
/// cache folder, whether or not it exists yet (REQ-util-006).
pub fn project_state_dir(root: &Path) -> io::Result<PathBuf> {
    location(root).map(|location| location.path())
}

/// A handle on a project's local state folder, opened without following a link at any name
/// SpecSync made there, and the folder's path for messages.
pub struct LocalState {
    dir: Dir,
    path: PathBuf,
}

/// A name SpecSync gives a file or folder of its own: one plain component.
fn checked_name(name: &str) -> io::Result<&str> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\', '\0']) {
        return Err(refused(format!("`{name}` is not a local state file name")));
    }
    Ok(name)
}

/// `name` beneath `dir`, a folder SpecSync makes: made when missing and `create`, refused when it
/// is a link or not a folder, then opened without following a link.
fn open_child(dir: Dir, name: &str, shown: &Path, create: bool) -> io::Result<Option<Dir>> {
    let mut created = false;
    match dir.symlink_metadata(name) {
        Ok(metadata) if metadata.is_dir() => {}
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(refused(format!(
                "{} is a symbolic link, and SpecSync's own state is never kept through one",
                shown.display()
            )));
        }
        Ok(_) => return Err(refused(format!("{} is not a directory", shown.display()))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if !create {
                return Ok(None);
            }
            match dir.create_dir(name) {
                Ok(()) => created = true,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    }
    // The look above names what is refused; this open enforces it, on a link that appeared since
    // as well.
    let folder = cap_primitives::fs::open_dir_nofollow(&dir.into_std_file(), Path::new(name))?;
    if created {
        // Only the person can write a folder SpecSync makes for itself, whatever the umask.
        private_folder(&folder)?;
    }
    refuse_shared_folder(&folder, shown)?;
    Ok(Some(Dir::from_std_file(folder)))
}

#[cfg(unix)]
fn private_folder(folder: &std::fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    folder.set_permissions(std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn private_folder(_: &std::fs::File) -> io::Result<()> {
    Ok(())
}

/// On Unix, a folder of SpecSync's own state that another user owns, or that its group or others
/// can write, is refused, as `ssh` refuses such a folder: whoever can write it could put a cache
/// or a journal there (REQ-util-006).
#[cfg(unix)]
fn refuse_shared_folder(folder: &std::fs::File, shown: &Path) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let metadata = folder.metadata()?;
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    let user = unsafe { libc::geteuid() };
    if metadata.uid() != user {
        return Err(refused(format!(
            "{} is owned by another user, who could put a file there, so SpecSync keeps none of \
             its own state there",
            shown.display()
        )));
    }
    if metadata.mode() & 0o022 != 0 {
        return Err(refused(format!(
            "{} is writable by its group or by others, who could put a file there, so SpecSync \
             keeps none of its own state there; run `chmod go-w {}` if only you use it",
            shown.display(),
            shown.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn refuse_shared_folder(_: &std::fs::File, _: &Path) -> io::Result<()> {
    Ok(())
}

impl LocalState {
    /// The local state folder of the project at `root`. With `create` it is made, with the
    /// user's cache folder if that is missing too; without, `None` when it does not exist yet.
    pub fn open(root: &Path, create: bool) -> io::Result<Option<LocalState>> {
        let location = location(root)?;
        if create {
            std::fs::create_dir_all(&location.base)?;
        }
        let mut dir = match Dir::open_ambient_dir(&location.base, ambient_authority()) {
            Ok(dir) => dir,
            Err(error) if error.kind() == io::ErrorKind::NotFound && !create => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut shown = location.base.clone();
        for name in &location.names {
            shown.push(name);
            match open_child(dir, name, &shown, create)? {
                Some(child) => dir = child,
                None => return Ok(None),
            }
        }
        Ok(Some(LocalState { dir, path: shown }))
    }

    /// The path of `name` in the folder, for messages.
    pub fn file_path(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }

    /// The folder `name` in this one, made when missing and `create`.
    pub fn subdirectory(&self, name: &str, create: bool) -> io::Result<Option<LocalState>> {
        let name = checked_name(name)?;
        let path = self.path.join(name);
        Ok(open_child(self.dir.try_clone()?, name, &path, create)?
            .map(|dir| LocalState { dir, path }))
    }

    /// What `name` is, without following a link; `None` when nothing is there.
    pub fn metadata(&self, name: &str) -> io::Result<Option<Metadata>> {
        match self.dir.symlink_metadata(checked_name(name)?) {
            Ok(metadata) => Ok(Some(metadata)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// The bytes of `name`, at most `max_bytes` of them; `None` when it is missing. The file is
    /// opened without following a link and without blocking, and the open handle must be a
    /// regular file no larger than the bound.
    pub fn read_file(&self, name: &str, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
        let name = checked_name(name)?;
        let shown = self.path.join(name);
        let mut options = OpenOptions::new();
        options
            .read(true)
            ._cap_fs_ext_nonblock(true)
            ._cap_fs_ext_follow(FollowSymlinks::No);
        let file = match self.dir.open_with(name, &options) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(match self.dir.symlink_metadata(name) {
                    Ok(metadata) if metadata.file_type().is_symlink() => refused(format!(
                        "{} is a symbolic link, and SpecSync's own state is never read through one",
                        shown.display()
                    )),
                    _ => error,
                });
            }
        };
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(refused(format!(
                "{} is not a regular file",
                shown.display()
            )));
        }
        let too_large = || {
            refused(format!(
                "{} is larger than {max_bytes} bytes",
                shown.display()
            ))
        };
        if metadata.len() > max_bytes {
            return Err(too_large());
        }
        let mut contents = Vec::new();
        file.take(max_bytes + 1).read_to_end(&mut contents)?;
        if contents.len() as u64 > max_bytes {
            return Err(too_large());
        }
        Ok(Some(contents))
    }

    /// Replace `name` with `contents`: written to a new file in the folder and renamed over the
    /// name, so nothing is ever written through what the name was. `durable` also syncs the file
    /// and the folder before returning.
    pub fn write_file(&self, name: &str, contents: &[u8], durable: bool) -> io::Result<()> {
        let name = checked_name(name)?;
        let temporary = temporary_name(name);
        let mut options = OpenOptions::new();
        // An exclusive create never follows a link at the new name.
        options.write(true).create_new(true);
        let mut file = self.dir.open_with(&temporary, &options)?;
        let written = file
            .write_all(contents)
            .and_then(|()| file.flush())
            .and_then(|()| if durable { file.sync_all() } else { Ok(()) });
        drop(file);
        // A rename replaces whatever holds the name and never writes through it.
        let published = written.and_then(|()| self.dir.rename(&temporary, &self.dir, name));
        if published.is_err() {
            let _ = self.dir.remove_file(&temporary);
        }
        published?;
        if durable {
            self.sync()?;
        }
        Ok(())
    }

    /// Remove `name` by unlinking it; nothing there is not an error. `durable` also syncs the
    /// folder before returning.
    pub fn remove(&self, name: &str, durable: bool) -> io::Result<()> {
        match self.dir.remove_file(checked_name(name)?) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        }
        if durable {
            self.sync()?;
        }
        Ok(())
    }

    /// Sync the folder itself. On Linux the folder handle is an `O_PATH` descriptor, which
    /// cannot be synced, so the folder is opened again through it, for reading, and that open is
    /// synced.
    fn sync(&self) -> io::Result<()> {
        let mut options = OpenOptions::new();
        options.read(true);
        self.dir.open_with(".", &options)?.sync_all()
    }
}

/// A name in the folder that no other write is using, for the file renamed over `name`.
fn temporary_name(name: &str) -> String {
    static WRITES: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos())
        .unwrap_or_default();
    format!(
        ".{name}.{}.{nanos}.{}.tmp",
        std::process::id(),
        WRITES.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn git(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    /// Verifies REQ-util-006. Two projects in one work tree, and a project with no Git, each get
    /// their own folder in the user's cache folder; nothing is made in the work tree or the Git
    /// directory.
    #[test]
    fn local_state_lives_outside_the_work_tree() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join("packages/app")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        let top = project_state_dir(&repo).unwrap();
        let app = project_state_dir(&repo.join("packages/app")).unwrap();
        assert!(
            top.starts_with(user_cache_folder().unwrap()),
            "{}",
            top.display()
        );
        assert_ne!(top, app, "two projects in one work tree share their state");

        let state = LocalState::open(&repo, true).unwrap().unwrap();
        state.write_file("hashes.json", b"{}", false).unwrap();
        assert_eq!(state.read_file("hashes.json", 16).unwrap().unwrap(), b"{}");
        assert!(!repo.join(".specsync").exists());
        assert!(!repo.join(".git/specsync").exists());

        let plain = dir.path().join("plain");
        fs::create_dir_all(&plain).unwrap();
        let state_dir = project_state_dir(&plain).unwrap();
        assert!(
            state_dir.starts_with(user_cache_folder().unwrap()),
            "{}",
            state_dir.display()
        );
        assert!(!state_dir.starts_with(plain.canonicalize().unwrap()));
    }

    /// Verifies REQ-util-006. A tarball of a working copy carries its `.git`, so a Git directory is
    /// no place for SpecSync's own state: in a Git work tree too, the state lives in the user's
    /// cache folder.
    #[test]
    fn local_state_never_lives_in_a_git_directory() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        let state = project_state_dir(&repo).unwrap();
        assert!(
            !state.starts_with(repo.canonicalize().unwrap()),
            "the state folder is inside the clone: {}",
            state.display()
        );
        assert!(
            state.starts_with(user_cache_folder().unwrap()),
            "{}",
            state.display()
        );
    }

    /// Verifies REQ-util-006. A link where SpecSync makes its own folder is refused, and a FIFO
    /// or an oversized file there is refused without blocking.
    #[cfg(unix)]
    #[test]
    fn local_state_refuses_links_fifos_and_oversized_files() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let elsewhere = dir.path().join("elsewhere");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&elsewhere).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        let folder = project_state_dir(&repo).unwrap();
        fs::create_dir_all(folder.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &folder).unwrap();
        let error = LocalState::open(&repo, true).err().unwrap();
        assert!(error.to_string().contains("symbolic link"), "{error}");
        assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
        fs::remove_file(&folder).unwrap();

        let state = LocalState::open(&repo, true).unwrap().unwrap();
        let fifo = state.file_path("change-transaction.json");
        assert!(
            Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        let error = state.read_file("change-transaction.json", 64).unwrap_err();
        assert!(error.to_string().contains("not a regular file"), "{error}");
        fs::remove_file(&fifo).unwrap();
        state.write_file("hashes.json", &[b'x'; 17], true).unwrap();
        let error = state.read_file("hashes.json", 16).unwrap_err();
        assert!(
            error.to_string().contains("larger than 16 bytes"),
            "{error}"
        );
        std::os::unix::fs::symlink(&elsewhere, state.file_path("linked.json")).unwrap();
        let error = state.read_file("linked.json", 16).unwrap_err();
        assert!(error.to_string().contains("symbolic link"), "{error}");
        state.remove("linked.json", true).unwrap();
        assert!(elsewhere.exists());
        assert!(state.read_file("linked.json", 16).unwrap().is_none());
    }

    /// N2 (6.4.1). A project that would hold its own state folder, such as one at
    /// `<cache>/specsync`, is refused as a cache folder inside the project is.
    #[test]
    fn a_project_that_would_hold_its_own_state_folder_is_refused() {
        let base = user_cache_folder().unwrap();
        let root = base.join(STATE_FOLDER);
        fs::create_dir_all(&root).unwrap();
        let state = project_state_dir(&root);
        assert!(state.is_err(), "{state:?}");
    }
}
