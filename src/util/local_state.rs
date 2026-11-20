//! Where SpecSync keeps the state it writes for itself about a project: the hash cache, the
//! transaction journal and the remote spec cache (REQ-util-006).
//!
//! None of it is ever kept in the work tree, where a repository can put a file with any content
//! it likes, under any spelling a case-insensitive disk folds onto the real name, in a ZIP or a
//! tarball with no Git to say it came with the repository. In a Git work tree the state lives in
//! the work tree's own Git directory, which `git clone` never fills and Git never checks a file
//! out into. With no work tree it lives in the user's cache folder. Either way the folder is
//! keyed by the project, so two projects never share it.

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

fn location(root: &Path) -> io::Result<Location> {
    let canonical = root.canonicalize()?;
    if let Some((git_dir, prefix)) = super::git_state_location(&canonical) {
        return Ok(Location {
            base: git_dir,
            names: vec![STATE_FOLDER.to_string(), key("work-tree", &prefix)],
        });
    }
    let base = user_cache_folder().ok_or_else(|| {
        refused(
            "there is no Git work tree here and no user cache folder (set HOME or \
             XDG_CACHE_HOME) to keep SpecSync's own state in"
                .to_string(),
        )
    })?;
    Ok(Location {
        base,
        names: vec![
            STATE_FOLDER.to_string(),
            PROJECTS_FOLDER.to_string(),
            key("project", &canonical),
        ],
    })
}

/// The folder where SpecSync keeps its own state about the project at `root`, outside the work
/// tree, whether or not it exists yet (REQ-util-006).
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
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    }
    // The look above names what is refused; this open enforces it, on a link that appeared since
    // as well.
    cap_primitives::fs::open_dir_nofollow(&dir.into_std_file(), Path::new(name))
        .map(Dir::from_std_file)
        .map(Some)
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

    /// Verifies REQ-util-006. In a Git work tree the state lives in the Git directory, one folder
    /// per project; with no work tree it lives in the user's cache folder. Nothing is made in the
    /// work tree.
    #[test]
    fn local_state_lives_outside_the_work_tree() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(repo.join("packages/app")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        let git_dir = repo.join(".git").canonicalize().unwrap();
        let top = project_state_dir(&repo).unwrap();
        let app = project_state_dir(&repo.join("packages/app")).unwrap();
        assert!(
            top.starts_with(git_dir.join("specsync")),
            "{}",
            top.display()
        );
        assert!(
            app.starts_with(git_dir.join("specsync")),
            "{}",
            app.display()
        );
        assert_ne!(top, app, "two projects in one work tree share their state");

        let state = LocalState::open(&repo, true).unwrap().unwrap();
        state.write_file("hashes.json", b"{}", false).unwrap();
        assert_eq!(state.read_file("hashes.json", 16).unwrap().unwrap(), b"{}");
        assert!(!repo.join(".specsync").exists());

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
        std::os::unix::fs::symlink(&elsewhere, repo.join(".git/specsync")).unwrap();
        let error = LocalState::open(&repo, true).err().unwrap();
        assert!(error.to_string().contains("symbolic link"), "{error}");
        assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
        fs::remove_file(repo.join(".git/specsync")).unwrap();

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
}
