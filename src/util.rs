use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use regex::RegexBuilder;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub mod local_state;

/// Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            curr[j] = if a[i - 1] == b[j - 1] {
                prev[j - 1]
            } else {
                1 + prev[j - 1].min(prev[j]).min(curr[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Maximum allowed size for user-provided regex patterns (in bytes of the compiled DFA).
/// Prevents ReDoS from crafted patterns in config files.
const MAX_REGEX_SIZE: usize = 1 << 16; // 64 KB

/// Compile a user-provided regex pattern with size limits to prevent ReDoS.
/// Returns None if the pattern is invalid or exceeds the size limit.
pub fn safe_regex(pattern: &str) -> Option<regex::Regex> {
    RegexBuilder::new(pattern)
        .size_limit(MAX_REGEX_SIZE)
        .dfa_size_limit(MAX_REGEX_SIZE)
        .build()
        .ok()
}

/// Confine a user-supplied path (e.g. a `depends_on` entry) to the project
/// root. Returns the joined path when `rel` is a relative path with no `..`
/// escape and no absolute/prefix component; `None` otherwise. Use this to
/// keep dependency declarations from validating (or reading) files outside
/// the project — `/etc/passwd` must never pass as a spec dependency.
pub fn confine_path_to_root(root: &std::path::Path, rel: &str) -> Option<std::path::PathBuf> {
    use std::path::Component;
    let path = std::path::Path::new(rel);
    if rel.is_empty() || path.is_absolute() {
        return None;
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            // ParentDir could escape the root; RootDir/Prefix are absolute.
            _ => return None,
        }
    }
    Some(root.join(path))
}

/// A project path the project file functions will not act on (REQ-util-002), with the reason as
/// the user is shown it. Carried inside an `io::Error` so a caller can tell a refusal from a
/// failure of the filesystem.
#[derive(Debug)]
struct RefusedProjectPath(String);

impl fmt::Display for RefusedProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RefusedProjectPath {}

fn refused_project_path(reason: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, RefusedProjectPath(reason))
}

/// Where the project file functions confine a path (REQ-util-004): the Git work tree that holds
/// the project root, so a monorepo subproject can follow a link that stays inside its
/// repository, or the project root itself when it is in no work tree.
struct Confinement {
    /// The directory every operation goes through, canonical.
    top: PathBuf,
    /// The project root as a path beneath `top`; empty when they are the same directory.
    prefix: PathBuf,
    /// The work tree's Git directory, canonical, from `git rev-parse --absolute-git-dir`; `None`
    /// outside a work tree. Nothing beneath it is a project file, whatever it is named
    /// (REQ-util-005).
    git_dir: Option<PathBuf>,
    /// The enclosing work tree's top folder and the project root's path beneath it, whether or
    /// not the confinement widened to that work tree (REQ-util-007); `None` outside a work tree.
    work_tree: Option<(PathBuf, PathBuf)>,
}

/// The confinement of `canonical_root`, worked out once per root for the whole run.
fn confinement(canonical_root: &Path) -> Arc<Confinement> {
    static KNOWN: OnceLock<Mutex<HashMap<PathBuf, Arc<Confinement>>>> = OnceLock::new();
    let known = KNOWN.get_or_init(Default::default);
    if let Some(found) = known
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(canonical_root)
    {
        return Arc::clone(found);
    }
    let computed = Arc::new(
        work_tree_confinement(canonical_root).unwrap_or_else(|| Confinement {
            top: canonical_root.to_path_buf(),
            prefix: PathBuf::new(),
            git_dir: None,
            work_tree: None,
        }),
    );
    known
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(canonical_root.to_path_buf(), Arc::clone(&computed));
    computed
}

/// The Git work tree holding `canonical_root` and its Git directory, from `git rev-parse
/// --show-toplevel --absolute-git-dir`. `None` outside a work tree, when Git is not installed,
/// or when its answer does not hold the root. `GIT_DIR` and `GIT_WORK_TREE` are left out, so a
/// hook's environment cannot move the answer.
///
/// The confinement is that work tree's top folder only when the project root is the top, or
/// when Git tracks a file under the root: a monorepo subproject. A project that is not part of
/// the repository around it, such as a ZIP unpacked inside a home folder kept under Git, is
/// confined to itself, so its links cannot reach the person's own files (REQ-util-007).
fn work_tree_confinement(canonical_root: &Path) -> Option<Confinement> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel", "--absolute-git-dir"])
        .current_dir(canonical_root)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE");
    let output = crate::git_utils::output_after_child_exits(&mut command).ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let mut lines = text.lines();
    let top = Path::new(lines.next()?).canonicalize().ok()?;
    let git_dir = Path::new(lines.next()?).canonicalize().ok()?;
    let prefix = canonical_root.strip_prefix(&top).ok()?.to_path_buf();
    let work_tree = Some((top.clone(), prefix.clone()));
    let part_of_the_work_tree = prefix.as_os_str().is_empty()
        || crate::git_utils::tracked_by_git(&top, &prefix.to_string_lossy().replace('\\', "/"));
    Some(if part_of_the_work_tree {
        Confinement {
            top,
            prefix,
            git_dir: Some(git_dir),
            work_tree,
        }
    } else {
        Confinement {
            top: canonical_root.to_path_buf(),
            prefix: PathBuf::new(),
            git_dir: Some(git_dir),
            work_tree,
        }
    })
}

/// The folder the project file functions confine `canonical_root` to: the enclosing work tree's
/// top folder for a project Git tracks there, otherwise the root itself (REQ-util-007). A user
/// cache folder inside it is refused (REQ-util-006).
pub(crate) fn confinement_top(canonical_root: &Path) -> PathBuf {
    confinement(canonical_root).top.clone()
}

/// The top folder of the Git work tree holding `root`, and its Git directory, both canonical;
/// `None` outside a work tree. The hooks folder is reached from one of them (REQ-hooks-009).
pub(crate) fn work_tree_and_git_dir(root: &Path) -> Option<(PathBuf, PathBuf)> {
    let confinement = confinement(&root.canonicalize().ok()?);
    let git_dir = confinement.git_dir.clone()?;
    let (top, _) = confinement.work_tree.clone()?;
    Some((top, git_dir))
}

/// A Git directory name, as Git and a case-insensitive file system read it: `.git` in any letter
/// case, with any trailing dots and spaces (REQ-util-005).
fn is_git_directory_name(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy()
        .trim_end_matches(['.', ' '])
        .eq_ignore_ascii_case(".git")
}

/// A directory handle on the confinement, the confined path beneath it, and the part of that
/// path that is the project root, which messages leave out.
struct ProjectPath {
    project: Dir,
    top: PathBuf,
    prefix: PathBuf,
    git_dir: Option<PathBuf>,
    relative: PathBuf,
}

impl ProjectPath {
    /// The same confinement, at another path beneath it.
    fn at(&self, relative: PathBuf) -> io::Result<ProjectPath> {
        Ok(ProjectPath {
            project: self.project.try_clone()?,
            top: self.top.clone(),
            prefix: self.prefix.clone(),
            git_dir: self.git_dir.clone(),
            relative,
        })
    }

    /// `location`, a canonical path, lies in the Git directory.
    fn in_git_dir(&self, location: &Path) -> bool {
        self.git_dir
            .as_deref()
            .is_some_and(|git_dir| location.starts_with(git_dir))
    }

    /// `reached` as the user named it: relative to the project root where it lies beneath it.
    fn shown<'a>(&self, reached: &'a Path) -> std::path::Display<'a> {
        reached
            .strip_prefix(&self.prefix)
            .unwrap_or(reached)
            .display()
    }
}

/// `path` as a path beneath `root`: `root` joined with a relative path, or a relative path given
/// as it is. Anything that would leave the root by its own spelling (`..`, another absolute
/// path) is refused before the filesystem is asked.
fn project_relative_path(
    root: &Path,
    canonical_root: Option<&Path>,
    path: &Path,
) -> io::Result<PathBuf> {
    // A path built from the canonical root (`/private/var/...` on macOS) still names a file
    // beneath a root given as `/var/...`. Only the root is canonicalized, never the path, so a
    // link in the path is still looked at, component by component, below.
    let beneath = path
        .strip_prefix(root)
        .ok()
        .or_else(|| canonical_root.and_then(|canonical| path.strip_prefix(canonical).ok()))
        .unwrap_or(path);
    let mut relative = PathBuf::new();
    for component in beneath.components() {
        match component {
            Component::Normal(name) => relative.push(name),
            Component::CurDir => {}
            _ => {
                return Err(refused_project_path(format!(
                    "{} is outside the project",
                    path.display()
                )));
            }
        }
    }
    if relative.as_os_str().is_empty() {
        return Err(refused_project_path(format!(
            "{} does not name a file in the project",
            path.display()
        )));
    }
    Ok(relative)
}

/// What a project path leads to.
enum ProjectTarget {
    /// Nothing is there yet, and nothing on the way is refused.
    Absent,
    /// A regular file inside the project, reached directly or through links that stay inside.
    File,
    /// A directory inside the project, reached the same way.
    Directory,
}

/// Follow the path beneath the confinement one component at a time and say what it leads to, or
/// why it is refused: a symbolic link that leaves the project, an absolute link, a dangling
/// link, a path into the Git directory, a non-directory on the way, or a last component that is
/// neither a regular file nor a directory.
///
/// This names the path and the reason. What enforces confinement is that every read and write
/// then goes through `project`, a `cap_std` directory handle that cannot be led outside itself,
/// so a component swapped after this look still cannot take the operation out of the project.
fn inspect_project_path(path: &ProjectPath) -> io::Result<ProjectTarget> {
    let project = &path.project;
    let mut reached = PathBuf::new();
    // Once a link is on the way, `top` joined with `reached` is no longer where it leads.
    let mut through_link = false;
    let mut components = path.relative.components().peekable();
    while let Some(component) = components.next() {
        reached.push(component);
        let last = components.peek().is_none();
        let shown = path.shown(&reached);
        if reached.starts_with(&path.prefix)
            && let Component::Normal(name) = component
            && is_git_directory_name(name)
        {
            return Err(refused_project_path(format!(
                "{shown} leads into the Git directory"
            )));
        }
        let found = match project.symlink_metadata(&reached) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(ProjectTarget::Absent);
            }
            Err(error) => return Err(error),
        };
        let resolved = if found.file_type().is_symlink() {
            // Reading the link's own text never follows it.
            let target = std::fs::read_link(path.top.join(&reached))?;
            if target.is_absolute() {
                return Err(refused_project_path(format!(
                    "{shown} is an absolute link, which is never followed"
                )));
            }
            let resolved = match project.metadata(&reached) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    return Err(refused_project_path(format!(
                        "{shown} is a dangling symbolic link"
                    )));
                }
                Err(_) => {
                    return Err(refused_project_path(format!(
                        "{shown} is a symbolic link that leaves the project"
                    )));
                }
            };
            // A link that stays inside may still lead into `.git`, or into a Git directory with
            // another name, where Git runs hooks and reads its config (REQ-util-005).
            let canonical = std::fs::canonicalize(path.top.join(&reached))?;
            if path.in_git_dir(&canonical)
                || canonical
                    .strip_prefix(&path.top)
                    .unwrap_or(&canonical)
                    .components()
                    .any(
                        |part| matches!(part, Component::Normal(name) if is_git_directory_name(name)),
                    )
            {
                return Err(refused_project_path(format!(
                    "{shown} leads into the Git directory"
                )));
            }
            through_link = true;
            resolved
        } else {
            let location = if through_link {
                std::fs::canonicalize(path.top.join(&reached))?
            } else {
                path.top.join(&reached)
            };
            if path.in_git_dir(&location) {
                return Err(refused_project_path(format!(
                    "{shown} leads into the Git directory"
                )));
            }
            found
        };
        if last {
            return if resolved.is_file() {
                Ok(ProjectTarget::File)
            } else if resolved.is_dir() {
                Ok(ProjectTarget::Directory)
            } else {
                Err(refused_project_path(format!(
                    "{shown} is not a regular file"
                )))
            };
        }
        if !resolved.is_dir() {
            return Err(refused_project_path(format!("{shown} is not a directory")));
        }
    }
    Ok(ProjectTarget::Absent)
}

/// A handle on the confinement of `root`, with `path` as a path beneath it.
fn open_project(root: &Path, path: &Path) -> io::Result<ProjectPath> {
    let canonical_root = root.canonicalize()?;
    let relative = project_relative_path(root, Some(&canonical_root), path)?;
    let confinement = confinement(&canonical_root);
    let project = Dir::open_ambient_dir(&confinement.top, ambient_authority())?;
    Ok(ProjectPath {
        project,
        top: confinement.top.clone(),
        prefix: confinement.prefix.clone(),
        git_dir: confinement.git_dir.clone(),
        relative: confinement.prefix.join(relative),
    })
}

/// A handle on exactly `directory`, with `path` as a path beneath it: for a directory the person
/// names outside the project, read within itself and nothing wider (REQ-util-004).
fn open_within(directory: &Path, path: &Path) -> io::Result<ProjectPath> {
    let top = directory.canonicalize()?;
    let relative = project_relative_path(directory, Some(&top), path)?;
    let project = Dir::open_ambient_dir(&top, ambient_authority())?;
    Ok(ProjectPath {
        project,
        top,
        prefix: PathBuf::new(),
        git_dir: None,
        relative,
    })
}

/// The path leads to a file, or to nothing yet. A directory there is refused.
fn project_file_exists(path: &ProjectPath) -> io::Result<bool> {
    match inspect_project_path(path)? {
        ProjectTarget::File => Ok(true),
        ProjectTarget::Absent => Ok(false),
        ProjectTarget::Directory => Err(refused_project_path(format!(
            "{} is not a regular file",
            path.shown(&path.relative)
        ))),
    }
}

/// Create the file with an exclusive create, which never follows a link at its last component,
/// after making the directories on the way.
fn create_new_project_file(path: &ProjectPath, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.relative.parent().filter(|p| !p.as_os_str().is_empty()) {
        path.project.create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = path.project.open_with(&path.relative, &options)?;
    file.write_all(contents)?;
    file.flush()
}

/// The bound for policy, config and state files and for change records (REQ-util-003).
pub const MAX_STATE_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// The bound for spec files, companions, history fragments, templates and hook files.
pub const MAX_SPEC_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// The bound for source and schema files, the same as coverage and `watch` use.
pub const MAX_SOURCE_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// The bound for the transaction journal, the hash cache and other whole project inputs.
pub const MAX_PROJECT_INPUT_BYTES: u64 = 64 * 1024 * 1024;

/// The bounded read behind [`read_project_file`] and [`read_file_within`].
fn read_confined(path: &ProjectPath, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
    if !project_file_exists(path)? {
        return Ok(None);
    }
    let shown = path.shown(&path.relative);
    let mut options = OpenOptions::new();
    // Non-blocking, so a FIFO swapped in after the look above cannot hold the open.
    options.read(true)._cap_fs_ext_nonblock(true);
    let file = path.project.open_with(&path.relative, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(refused_project_path(format!(
            "{shown} is not a regular file"
        )));
    }
    let over_bound = || refused_project_path(format!("{shown} is larger than {max_bytes} bytes"));
    if metadata.len() > max_bytes {
        return Err(over_bound());
    }
    let mut contents = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut contents)?;
    if contents.len() as u64 > max_bytes {
        return Err(over_bound());
    }
    Ok(Some(contents))
}

/// Read a file of the project, `None` when nothing is there (REQ-util-002, REQ-util-003).
///
/// `path` is `root` joined with a relative path, or a path relative to `root`. A symbolic link
/// that stays inside the project, or inside the Git work tree that holds it, is followed. A link
/// that leaves it, an absolute or dangling link, a path into the Git directory, anything that is
/// not a regular file, and a file larger than `max_bytes` are refused with an error that names
/// the path and the reason. The file is opened without blocking and its type is checked on the
/// open handle, so neither a FIFO nor a device can hold the read, and at most one byte past the
/// bound is ever read.
pub fn read_project_file(root: &Path, path: &Path, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
    read_confined(&open_project(root, path)?, max_bytes)
}

/// [`read_project_file`] within exactly `directory`, for a directory the person names that may
/// lie outside the project: a link inside it is followed, a link out of it is refused, and the
/// same bound applies (REQ-util-004).
pub fn read_file_within(
    directory: &Path,
    path: &Path,
    max_bytes: u64,
) -> io::Result<Option<Vec<u8>>> {
    read_confined(&open_within(directory, path)?, max_bytes)
}

/// [`read_project_file`] with the contract of `std::fs::read`: a missing file is a not-found
/// error, so a call site keeps the error handling it had (REQ-util-003).
pub fn read_project_bytes(root: &Path, path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    read_project_file(root, path, max_bytes)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not exist", path.display()),
        )
    })
}

/// [`read_project_file`] with the contract of `std::fs::read_to_string`: a missing file is a
/// not-found error and bytes that are not UTF-8 are invalid data (REQ-util-003).
pub fn read_project_string(root: &Path, path: &Path, max_bytes: u64) -> io::Result<String> {
    // The same error text as `std::fs::read_to_string`, so a caller's message does not change.
    String::from_utf8(read_project_bytes(root, path, max_bytes)?).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        )
    })
}

/// Write a file of the project, creating it and the directories on the way when they are missing
/// (REQ-util-002).
///
/// A symbolic link that stays inside the project is written through. A link that leaves it, an
/// absolute or dangling link, a path into the Git directory and anything that is not a regular
/// file are refused with an error that names the path, and nothing is written or created. An
/// existing file is opened without blocking and without truncating, and emptied only once the
/// open handle shows a regular file, so a FIFO swapped in after the look cannot hold the write.
pub fn write_project_file(root: &Path, path: &Path, contents: &[u8]) -> io::Result<()> {
    let path = open_project(root, path)?;
    if !project_file_exists(&path)? {
        return create_new_project_file(&path, contents);
    }
    // No create flag: an existing file is opened, so a link is never created through.
    let mut options = OpenOptions::new();
    options.write(true)._cap_fs_ext_nonblock(true);
    let mut file = path.project.open_with(&path.relative, &options)?;
    if !file.metadata()?.is_file() {
        return Err(refused_project_path(format!(
            "{} is not a regular file",
            path.shown(&path.relative)
        )));
    }
    file.set_len(0)?;
    file.write_all(contents)?;
    file.flush()
}

/// Create a file of the project that does not exist yet (REQ-util-002). `Ok(false)`, with nothing
/// changed, when a regular file is already there. The refusals of [`write_project_file`] apply.
pub fn create_project_file(root: &Path, path: &Path, contents: &[u8]) -> io::Result<bool> {
    let path = open_project(root, path)?;
    if project_file_exists(&path)? {
        return Ok(false);
    }
    create_new_project_file(&path, contents).map(|()| true)
}

/// Remove a file of the project (REQ-util-002). The name removed is inside the project: a link
/// that stays inside is removed as a link, and its target is left. The refusals of
/// [`write_project_file`] apply.
pub fn remove_project_file(root: &Path, path: &Path) -> io::Result<()> {
    let path = open_project(root, path)?;
    if !project_file_exists(&path)? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not exist", path.shown(&path.relative)),
        ));
    }
    path.project.remove_file(&path.relative)
}

/// Create a directory of the project and the directories on the way to it (REQ-util-002). The
/// refusals of [`write_project_file`] apply, and a file where the directory belongs is refused.
pub fn create_project_dir(root: &Path, path: &Path) -> io::Result<()> {
    let path = open_project(root, path)?;
    match inspect_project_path(&path)? {
        ProjectTarget::Directory => Ok(()),
        ProjectTarget::Absent => path.project.create_dir_all(&path.relative),
        ProjectTarget::File => Err(refused_project_path(format!(
            "{} is not a directory",
            path.shown(&path.relative)
        ))),
    }
}

/// Create a directory of the project that must not exist yet, after the directories on the way
/// (REQ-util-002). Something already at the name, a link included, is an `AlreadyExists` error;
/// the refusals of [`write_project_file`] apply on the way.
pub fn create_new_project_dir(root: &Path, path: &Path) -> io::Result<()> {
    let path = open_project(root, path)?;
    if let Some(parent) = path.relative.parent().filter(|p| !p.as_os_str().is_empty()) {
        let parent = path.at(parent.to_path_buf())?;
        match inspect_project_path(&parent)? {
            ProjectTarget::Directory => {}
            ProjectTarget::Absent => parent.project.create_dir_all(&parent.relative)?,
            ProjectTarget::File => {
                return Err(refused_project_path(format!(
                    "{} is not a directory",
                    parent.shown(&parent.relative)
                )));
            }
        }
    }
    // `mkdir` never follows a link at its last component: a name already there, link or not,
    // fails with `AlreadyExists`.
    path.project.create_dir(&path.relative)
}

/// Whether `path` is a regular file of the project the project file functions would read: a
/// refused path counts as absent (REQ-util-002).
pub fn is_project_file(root: &Path, path: &Path) -> bool {
    open_project(root, path)
        .and_then(|path| project_file_exists(&path))
        .unwrap_or(false)
}

/// Remove an empty directory of the project (REQ-util-002). A link at the name is never
/// followed, and a directory reached through a link that leaves the project is refused.
pub fn remove_empty_project_dir(root: &Path, path: &Path) -> io::Result<()> {
    let path = open_project(root, path)?;
    match inspect_project_path(&path)? {
        ProjectTarget::Directory => path.project.remove_dir(&path.relative),
        ProjectTarget::Absent => Ok(()),
        ProjectTarget::File => Err(refused_project_path(format!(
            "{} is not a directory",
            path.shown(&path.relative)
        ))),
    }
}

/// The entries of one directory beneath the project root, sorted, as paths under `path`
/// (REQ-util-010). The directory is reached by the project rule, so one that leaves the project
/// through a link at it or on the way to it, a dangling link, or the Git directory is refused
/// with the path, and it is listed through the project handle. The project root itself is
/// listed as it is.
pub fn read_project_dir(root: &Path, path: &Path) -> io::Result<Vec<PathBuf>> {
    let canonical_root = root.canonicalize()?;
    let entries = if path == root || path == canonical_root {
        Dir::open_ambient_dir(root, ambient_authority())?.read_dir(".")?
    } else {
        let directory = open_project(root, path)?;
        match inspect_project_path(&directory)? {
            ProjectTarget::Directory => {}
            ProjectTarget::Absent => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{} does not exist", directory.shown(&directory.relative)),
                ));
            }
            ProjectTarget::File => {
                return Err(refused_project_path(format!(
                    "{} is not a directory",
                    directory.shown(&directory.relative)
                )));
            }
        }
        directory.project.read_dir(&directory.relative)?
    };
    let mut found = Vec::new();
    for entry in entries {
        found.push(path.join(entry?.file_name()));
    }
    found.sort();
    Ok(found)
}

/// Every regular file under the project directory `directory` whose name ends with `suffix`,
/// sorted, as `root` joined with its path (REQ-util-002).
///
/// `directory` follows the project rule, so a link there that stays inside is followed and one
/// that leaves is refused. Below it, a link is listed but never followed, unless it dangles
/// inside the project or leads to a directory, so a reader refuses one that leaves by name;
/// links to directories are not walked, as before. A `.git` directory is not walked. The walk
/// stops with an error past `max_entries` entries or `max_depth` levels, so no tree can hold it.
pub fn find_project_files(
    root: &Path,
    directory: &Path,
    suffix: &str,
    max_entries: usize,
    max_depth: usize,
) -> io::Result<Vec<PathBuf>> {
    let Ok(canonical_root) = root.canonicalize() else {
        // No project there: nothing to find, as when the directory is missing.
        return Ok(Vec::new());
    };
    let names_root = directory == root || directory == canonical_root;
    let start = if names_root {
        // `specs_dir = "."`: the walk starts at the project root itself.
        let confinement = confinement(&canonical_root);
        ProjectPath {
            project: Dir::open_ambient_dir(&confinement.top, ambient_authority())?,
            top: confinement.top.clone(),
            prefix: confinement.prefix.clone(),
            git_dir: confinement.git_dir.clone(),
            relative: confinement.prefix.clone(),
        }
    } else {
        open_project(root, directory)?
    };
    if !names_root && !matches!(inspect_project_path(&start)?, ProjectTarget::Directory) {
        return Ok(Vec::new());
    }
    let shown_start = if names_root {
        ".".to_string()
    } else {
        start.shown(&start.relative).to_string()
    };
    let mut found = Vec::new();
    let mut entries = 0usize;
    let mut pending = vec![(start.relative.clone(), 0usize)];
    while let Some((current, depth)) = pending.pop() {
        // The root that is the confinement itself is the handle's own directory, `.`.
        let listed = if current.as_os_str().is_empty() {
            Path::new(".")
        } else {
            current.as_path()
        };
        let listing = start.project.read_dir(listed).map_err(|error| {
            // A folder that cannot be listed hides what is below it, so the error names it.
            let shown = if current.as_os_str().is_empty() {
                ".".to_string()
            } else {
                start.shown(&current).to_string()
            };
            io::Error::new(error.kind(), format!("cannot list {shown}: {error}"))
        })?;
        for entry in listing {
            let entry = entry?;
            entries += 1;
            if entries > max_entries {
                return Err(refused_project_path(format!(
                    "{shown_start} holds more than {max_entries} entries"
                )));
            }
            let name = entry.file_name();
            let child = current.join(&name);
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if is_git_directory_name(&name) || start.in_git_dir(&start.top.join(&child)) {
                    continue;
                }
                if depth + 1 > max_depth {
                    return Err(refused_project_path(format!(
                        "{shown_start} is more than {max_depth} directories deep"
                    )));
                }
                pending.push((child, depth + 1));
                continue;
            }
            if !name.to_string_lossy().ends_with(suffix) {
                continue;
            }
            // A link is listed unless it is dangling or leads to a directory inside, as a walk
            // that does not follow links lists it. It is never followed here: one that leaves
            // the project or leads into `.git` is listed for the reader to refuse by name.
            let keep = file_type.is_file()
                || (file_type.is_symlink()
                    && !matches!(
                        start.project.metadata(&child),
                        Err(ref error) if error.kind() == io::ErrorKind::NotFound
                    )
                    && !matches!(
                        inspect_project_path(&start.at(child.clone())?),
                        Ok(ProjectTarget::Directory)
                    ));
            if keep {
                found.push(root.join(child.strip_prefix(&start.prefix).unwrap_or(&child)));
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Why a project path is refused, in words for the user, without changing anything: a symbolic
/// link that leaves the project, an absolute or dangling link, a path into the Git directory, a
/// path outside the project, or something that is neither a regular file nor a directory. `None`
/// for a path the project file functions accept (REQ-util-002).
pub fn project_path_refusal(root: &Path, path: &Path) -> Option<String> {
    let refusal = |error: io::Error| {
        error
            .get_ref()
            .is_some_and(|inner| inner.is::<RefusedProjectPath>())
            .then(|| error.to_string())
    };
    match open_project(root, path) {
        Ok(path) => inspect_project_path(&path).err().and_then(refusal),
        Err(error) => refusal(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("config.ts", "confg.ts"), 1);
    }

    #[test]
    fn test_safe_regex_valid() {
        assert!(safe_regex(r"\bfoo\b").is_some());
        assert!(safe_regex(r"^## \w+").is_some());
    }

    #[test]
    fn test_safe_regex_invalid() {
        assert!(safe_regex(r"[invalid").is_none());
    }

    #[test]
    fn test_confine_path_to_root_accepts_relative() {
        let root = std::path::Path::new("/proj");
        assert_eq!(
            confine_path_to_root(root, "specs/a/a.spec.md"),
            Some(root.join("specs/a/a.spec.md"))
        );
        assert_eq!(
            confine_path_to_root(root, "./specs/a.spec.md"),
            Some(root.join("./specs/a.spec.md"))
        );
    }

    #[test]
    fn test_confine_path_to_root_rejects_escapes() {
        let root = std::path::Path::new("/proj");
        // Absolute paths and `..` traversal must never validate (#444).
        assert_eq!(confine_path_to_root(root, "/etc/passwd"), None);
        assert_eq!(confine_path_to_root(root, "../outside.spec.md"), None);
        assert_eq!(confine_path_to_root(root, "specs/../../etc/passwd"), None);
        assert_eq!(confine_path_to_root(root, ""), None);
    }

    /// A project beside an `outside` directory, both in one temporary directory, so a link that
    /// leaves the project still reaches nothing but this fixture (REQ-util-002).
    #[cfg(unix)]
    fn project_beside_outside() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("project");
        let outside = dir.path().join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        (dir, root, outside)
    }

    #[cfg(unix)]
    fn entries(directory: &Path) -> usize {
        std::fs::read_dir(directory).unwrap().count()
    }

    /// Verifies REQ-util-002. A file linked outside the project is neither written nor read, by
    /// a relative link as a clone carries it or by an absolute one.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_refuse_a_link_that_leaves_the_project() {
        use std::os::unix::fs::symlink;
        for absolute in [false, true] {
            let (_dir, root, outside) = project_beside_outside();
            let victim = outside.join("victim.txt");
            std::fs::write(&victim, "the owner's bytes\n").unwrap();
            if absolute {
                symlink(&victim, root.join("CLAUDE.md")).unwrap();
            } else {
                symlink("../outside/victim.txt", root.join("CLAUDE.md")).unwrap();
            }
            // An absolute link is refused as one before anything asks where it leads (REQ-util-004).
            let reason = if absolute {
                "CLAUDE.md is an absolute link, which is never followed"
            } else {
                "CLAUDE.md is a symbolic link that leaves the project"
            };

            let written = write_project_file(&root, Path::new("CLAUDE.md"), b"block\n");
            assert_eq!(
                std::fs::read_to_string(&victim).unwrap(),
                "the owner's bytes\n"
            );
            assert_eq!(written.unwrap_err().to_string(), reason);
            assert_eq!(
                read_project_file(&root, &root.join("CLAUDE.md"), MAX_SPEC_FILE_BYTES)
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(
                create_project_file(&root, Path::new("CLAUDE.md"), b"block\n")
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(
                remove_project_file(&root, Path::new("CLAUDE.md"))
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(
                project_path_refusal(&root, Path::new("CLAUDE.md")).as_deref(),
                Some(reason)
            );
            assert!(std::fs::symlink_metadata(root.join("CLAUDE.md")).is_ok());
            assert_eq!(entries(&outside), 1);
        }
    }

    /// Verifies REQ-util-004. A write to a FIFO is refused at once; nothing waits on it.
    #[cfg(unix)]
    #[test]
    fn writing_a_fifo_is_refused_without_blocking() {
        let (_dir, root, _outside) = project_beside_outside();
        fifo(&root.join("pipe.md"));
        assert_eq!(
            write_project_file(&root, Path::new("pipe.md"), b"x")
                .unwrap_err()
                .to_string(),
            "pipe.md is not a regular file"
        );
    }

    /// Verifies REQ-util-004. `read_file_within` reads within exactly the directory it is given:
    /// a link inside it is followed, a link out of it is refused, even to a sibling directory.
    #[cfg(unix)]
    #[test]
    fn reading_within_a_named_directory_stays_inside_it() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        std::fs::write(outside.join("notes.md"), "notes\n").unwrap();
        symlink("notes.md", outside.join("again.md")).unwrap();
        std::fs::write(root.join("secret.md"), "secret\n").unwrap();
        symlink("../project/secret.md", outside.join("secret.md")).unwrap();
        let read =
            |name: &str| read_file_within(&outside, &outside.join(name), MAX_SPEC_FILE_BYTES);
        assert_eq!(read("again.md").unwrap().unwrap(), b"notes\n");
        assert_eq!(
            read("secret.md").unwrap_err().to_string(),
            "secret.md is a symbolic link that leaves the project"
        );
    }

    /// Verifies REQ-util-004. An empty directory reached through a link that leaves the project
    /// is never removed; one inside is.
    #[cfg(unix)]
    #[test]
    fn removing_an_empty_directory_never_follows_a_link_out() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        std::fs::create_dir_all(outside.join("specsync")).unwrap();
        symlink("../outside", root.join("commands")).unwrap();
        assert!(remove_empty_project_dir(&root, &root.join("commands/specsync")).is_err());
        assert!(outside.join("specsync").is_dir());
        std::fs::create_dir_all(root.join("own/specsync")).unwrap();
        remove_empty_project_dir(&root, &root.join("own/specsync")).unwrap();
        assert!(!root.join("own/specsync").exists());
    }

    /// Verifies REQ-util-004. The walk lists a file link that leaves the project, without
    /// following it, so its reader refuses it by name; it drops a link dangling inside the
    /// project and does not walk a linked directory or `.git`.
    #[cfg(unix)]
    #[test]
    fn finding_project_files_never_follows_a_link() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        std::fs::create_dir_all(root.join("specs/auth")).unwrap();
        std::fs::create_dir_all(root.join("specs/.git")).unwrap();
        std::fs::create_dir_all(outside.join("more")).unwrap();
        std::fs::write(root.join("specs/auth/auth.spec.md"), "a").unwrap();
        std::fs::write(root.join("specs/.git/x.spec.md"), "a").unwrap();
        std::fs::write(outside.join("out.spec.md"), "a").unwrap();
        std::fs::write(outside.join("more/deep.spec.md"), "a").unwrap();
        symlink("../../outside/out.spec.md", root.join("specs/out.spec.md")).unwrap();
        symlink("nothing.spec.md", root.join("specs/gone.spec.md")).unwrap();
        symlink("../../outside/more", root.join("specs/more")).unwrap();
        let found = find_project_files(&root, &root.join("specs"), ".spec.md", 1000, 16).unwrap();
        assert_eq!(
            found,
            vec![
                root.join("specs/auth/auth.spec.md"),
                root.join("specs/out.spec.md")
            ]
        );
        assert_eq!(
            read_project_file(&root, &found[1], MAX_SPEC_FILE_BYTES)
                .unwrap_err()
                .to_string(),
            "specs/out.spec.md is a symbolic link that leaves the project"
        );
    }

    /// Verifies REQ-util-002. A walk that starts at the project root itself (`specs_dir = "."`)
    /// lists the root's files, outside Git and at the top of a work tree, where the root is the
    /// confinement's own directory.
    #[test]
    fn finding_project_files_from_the_root_itself_lists_its_files() {
        for in_git in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let root = dir.path();
            if in_git {
                let status = std::process::Command::new("git")
                    .args(["init", "-q"])
                    .current_dir(root)
                    .status()
                    .unwrap();
                assert!(status.success());
            }
            std::fs::create_dir_all(root.join("auth")).unwrap();
            std::fs::write(root.join("top.spec.md"), "a").unwrap();
            std::fs::write(root.join("auth/auth.spec.md"), "a").unwrap();
            let found = find_project_files(root, root, ".spec.md", 1000, 16).unwrap();
            assert_eq!(
                found,
                vec![root.join("auth/auth.spec.md"), root.join("top.spec.md")],
                "in_git: {in_git}"
            );
        }
    }

    /// Verifies REQ-util-005. A path into the Git directory is refused, named directly in any
    /// letter case or reached through a link that stays inside the project.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_refuse_the_git_directory() {
        use std::os::unix::fs::symlink;
        let (_dir, root, _outside) = project_beside_outside();
        std::fs::create_dir_all(root.join(".git/hooks")).unwrap();
        std::fs::write(root.join(".git/config"), "[core]\n").unwrap();
        symlink(".git/config", root.join("CLAUDE.md")).unwrap();
        for name in [".git/config", ".GIT/config", ".git./config", "CLAUDE.md"] {
            let error = write_project_file(&root, Path::new(name), b"x").unwrap_err();
            assert!(
                error.to_string().ends_with("leads into the Git directory"),
                "{name}: {error}"
            );
            assert!(read_project_file(&root, Path::new(name), MAX_SPEC_FILE_BYTES).is_err());
        }
        assert_eq!(
            std::fs::read_to_string(root.join(".git/config")).unwrap(),
            "[core]\n"
        );
    }

    /// Verifies REQ-util-005. A Git directory with another name (`git init --separate-git-dir`
    /// inside the work tree) is refused like `.git`: named directly, reached through a link, or
    /// reached through a linked directory on the way. A walk does not enter it.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_refuse_a_git_directory_under_any_name() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let status = std::process::Command::new("git")
            .args(["init", "-q", "--separate-git-dir", "gitstore", "."])
            .current_dir(&root)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .status()
            .unwrap();
        assert!(status.success());
        let hook = root.join("gitstore/hooks/pre-commit");
        std::fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::write(root.join("gitstore/hooks/notes.spec.md"), "a").unwrap();
        symlink("gitstore/hooks/pre-commit", root.join("CLAUDE.md")).unwrap();
        symlink("gitstore", root.join("store")).unwrap();
        for name in [
            "gitstore/hooks/pre-commit",
            "CLAUDE.md",
            "store/hooks/pre-commit",
        ] {
            let error = write_project_file(&root, Path::new(name), b"x").unwrap_err();
            assert!(
                error.to_string().ends_with("leads into the Git directory"),
                "{name}: {error}"
            );
        }
        assert_eq!(
            std::fs::read_to_string(&hook).unwrap(),
            "#!/bin/sh\nexit 0\n"
        );
        assert!(
            find_project_files(&root, &root, ".spec.md", 1000, 16)
                .unwrap()
                .is_empty()
        );
    }

    /// Verifies REQ-util-002. A dangling link creates nothing, wherever it points, and a link to
    /// a link that leaves is refused like a direct one.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_refuse_a_dangling_link_and_a_chain_that_leaves() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        symlink("../outside/not-there-yet.txt", root.join("out.md")).unwrap();
        symlink("not-there-either.md", root.join("in.md")).unwrap();
        for (name, reason) in [
            // A link that would leave is refused for that, whether or not its target exists.
            (
                "out.md",
                "out.md is a symbolic link that leaves the project",
            ),
            ("in.md", "in.md is a dangling symbolic link"),
        ] {
            let reason = reason.to_string();
            assert_eq!(
                write_project_file(&root, Path::new(name), b"x")
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(
                create_project_file(&root, Path::new(name), b"x")
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(project_path_refusal(&root, Path::new(name)), Some(reason));
        }
        assert_eq!(entries(&outside), 0);
        assert!(!root.join("not-there-either.md").exists());

        let victim = outside.join("victim.txt");
        std::fs::write(&victim, "the owner's bytes\n").unwrap();
        symlink("../outside/victim.txt", root.join("hop.md")).unwrap();
        symlink("hop.md", root.join("chain.md")).unwrap();
        assert_eq!(
            write_project_file(&root, Path::new("chain.md"), b"x")
                .unwrap_err()
                .to_string(),
            "chain.md is a symbolic link that leaves the project"
        );
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            "the owner's bytes\n"
        );
    }

    /// Verifies REQ-util-002. A link that stays inside the project is followed: `CLAUDE.md` as a
    /// link to `AGENTS.md` is a common setup.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_follow_a_link_that_stays_inside_the_project() {
        use std::os::unix::fs::symlink;
        let (_dir, root, _outside) = project_beside_outside();
        std::fs::write(root.join("AGENTS.md"), "# Agents\n").unwrap();
        symlink("AGENTS.md", root.join("CLAUDE.md")).unwrap();

        assert_eq!(project_path_refusal(&root, Path::new("CLAUDE.md")), None);
        assert_eq!(
            read_project_file(&root, Path::new("CLAUDE.md"), MAX_SPEC_FILE_BYTES).unwrap(),
            Some(b"# Agents\n".to_vec())
        );
        write_project_file(&root, Path::new("CLAUDE.md"), b"# Agents\n\nblock\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("AGENTS.md")).unwrap(),
            "# Agents\n\nblock\n"
        );
        assert!(
            std::fs::symlink_metadata(root.join("CLAUDE.md"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(!create_project_file(&root, Path::new("CLAUDE.md"), b"other\n").unwrap());

        std::fs::create_dir_all(root.join("kept/claude")).unwrap();
        symlink("kept/claude", root.join(".claude")).unwrap();
        write_project_file(&root, Path::new(".claude/settings.json"), b"{}\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("kept/claude/settings.json")).unwrap(),
            "{}\n"
        );
    }

    /// Verifies REQ-util-002. A directory on the way can be the link, and a path can try to leave
    /// by its own spelling.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_refuse_a_directory_that_leaves_and_a_path_outside() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        std::fs::create_dir_all(outside.join("elsewhere")).unwrap();
        symlink("../outside/elsewhere", root.join(".claude")).unwrap();
        let reason = ".claude is a symbolic link that leaves the project";
        assert_eq!(
            write_project_file(&root, Path::new(".claude/settings.json"), b"{}")
                .unwrap_err()
                .to_string(),
            reason
        );
        assert_eq!(
            create_project_dir(&root, Path::new(".claude/skills"))
                .unwrap_err()
                .to_string(),
            reason
        );
        assert_eq!(entries(&outside.join("elsewhere")), 0);

        assert!(write_project_file(&root, Path::new("../outside/escaped.txt"), b"x").is_err());
        assert!(write_project_file(&root, &outside.join("escaped.txt"), b"x").is_err());
        assert!(create_project_dir(&root, Path::new("../outside/made")).is_err());
        assert_eq!(entries(&outside), 1);
    }

    /// Verifies REQ-util-002. The plain cases: create, replace, read, remove, and a directory
    /// where a file belongs.
    #[cfg(unix)]
    #[test]
    fn project_file_functions_create_replace_read_and_remove_plain_files() {
        let (_dir, root, _outside) = project_beside_outside();
        let path = root.join(".specsync/lock.json");

        assert_eq!(
            read_project_file(&root, &path, MAX_SPEC_FILE_BYTES).unwrap(),
            None
        );
        assert_eq!(project_path_refusal(&root, &path), None);
        assert!(create_project_file(&root, &path, b"one\n").unwrap());
        assert!(!create_project_file(&root, &path, b"two\n").unwrap());
        assert_eq!(
            read_project_file(&root, &path, MAX_SPEC_FILE_BYTES).unwrap(),
            Some(b"one\n".to_vec())
        );
        write_project_file(&root, &path, b"three\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "three\n");
        remove_project_file(&root, &path).unwrap();
        assert!(!path.exists());
        assert_eq!(
            remove_project_file(&root, &path).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );

        create_project_dir(&root, Path::new(".specsync/lifecycle")).unwrap();
        assert!(root.join(".specsync/lifecycle").is_dir());
        assert_eq!(
            write_project_file(&root, Path::new(".specsync/lifecycle"), b"x")
                .unwrap_err()
                .to_string(),
            ".specsync/lifecycle is not a regular file"
        );
        create_project_dir(&root, Path::new(".specsync/archive/nested/deeper")).unwrap();
        assert!(root.join(".specsync/archive/nested/deeper").is_dir());
        std::fs::write(root.join("file"), "x").unwrap();
        assert_eq!(
            write_project_file(&root, Path::new("file/under"), b"x")
                .unwrap_err()
                .to_string(),
            "file is not a directory"
        );
    }

    /// Make a FIFO at `path` that nothing ever writes to.
    #[cfg(unix)]
    fn fifo(path: &Path) {
        let status = std::process::Command::new("mkfifo")
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success());
    }

    /// Verifies REQ-util-003. A link that leaves the root, a dangling link, a FIFO and a directory
    /// are refused with the path and the reason, and the FIFO read returns at once.
    #[cfg(unix)]
    #[test]
    fn project_file_reads_refuse_what_they_must_not_read() {
        use std::os::unix::fs::symlink;
        let (_dir, root, outside) = project_beside_outside();
        std::fs::write(outside.join("victim.txt"), "the owner's bytes\n").unwrap();
        symlink("../outside/victim.txt", root.join("out.json")).unwrap();
        symlink("not-there.json", root.join("dangling.json")).unwrap();
        fifo(&root.join("fifo.json"));
        std::fs::create_dir(root.join("dir.json")).unwrap();

        let started = std::time::Instant::now();
        for (name, reason) in [
            (
                "out.json",
                "out.json is a symbolic link that leaves the project",
            ),
            ("dangling.json", "dangling.json is a dangling symbolic link"),
            ("fifo.json", "fifo.json is not a regular file"),
            ("dir.json", "dir.json is not a regular file"),
        ] {
            assert_eq!(
                read_project_file(&root, Path::new(name), MAX_STATE_FILE_BYTES)
                    .unwrap_err()
                    .to_string(),
                reason
            );
            assert_eq!(
                read_project_string(&root, &root.join(name), MAX_STATE_FILE_BYTES)
                    .unwrap_err()
                    .to_string(),
                reason
            );
        }
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "a read waited on the FIFO"
        );
    }

    /// Verifies REQ-util-003. The bound is exact: a file at the bound is read, one byte more is
    /// refused naming the bound, without being read.
    #[cfg(unix)]
    #[test]
    fn project_file_reads_hold_the_bound() {
        let (_dir, root, _outside) = project_beside_outside();
        std::fs::write(root.join("at.txt"), vec![b'a'; 16]).unwrap();
        assert_eq!(
            read_project_bytes(&root, Path::new("at.txt"), 16)
                .unwrap()
                .len(),
            16
        );
        assert_eq!(
            read_project_bytes(&root, Path::new("at.txt"), 15)
                .unwrap_err()
                .to_string(),
            "at.txt is larger than 15 bytes"
        );
        let sparse = std::fs::File::create(root.join("sparse.json")).unwrap();
        sparse.set_len(MAX_STATE_FILE_BYTES + 1).unwrap();
        assert_eq!(
            read_project_file(&root, Path::new("sparse.json"), MAX_STATE_FILE_BYTES)
                .unwrap_err()
                .to_string(),
            format!("sparse.json is larger than {MAX_STATE_FILE_BYTES} bytes")
        );
    }

    /// Verifies REQ-util-003. A link inside the root is read; a missing file is `None` from
    /// `read_project_file` and not found from the other two; bytes that are not UTF-8 are invalid
    /// data from `read_project_string`.
    #[cfg(unix)]
    #[test]
    fn project_file_reads_follow_inside_links_and_keep_the_std_contracts() {
        let (_dir, root, _outside) = project_beside_outside();
        std::fs::write(root.join("AGENTS.md"), "# Agents\n").unwrap();
        std::os::unix::fs::symlink("AGENTS.md", root.join("CLAUDE.md")).unwrap();
        assert_eq!(
            read_project_string(&root, Path::new("CLAUDE.md"), MAX_SPEC_FILE_BYTES).unwrap(),
            "# Agents\n"
        );
        assert_eq!(
            read_project_file(&root, Path::new("missing.md"), MAX_SPEC_FILE_BYTES).unwrap(),
            None
        );
        for error in [
            read_project_bytes(&root, Path::new("missing.md"), MAX_SPEC_FILE_BYTES).unwrap_err(),
            read_project_string(&root, Path::new("missing.md"), MAX_SPEC_FILE_BYTES).unwrap_err(),
        ] {
            assert_eq!(error.kind(), io::ErrorKind::NotFound);
        }
        std::fs::write(root.join("binary.md"), [0xff, 0xfe]).unwrap();
        assert_eq!(
            read_project_string(&root, Path::new("binary.md"), MAX_SPEC_FILE_BYTES)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
