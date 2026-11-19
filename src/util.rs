use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use regex::RegexBuilder;
use std::fmt;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

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

/// `path` as a path beneath `root`: `root` joined with a relative path, or a relative path given
/// as it is. Anything that would leave the root by its own spelling (`..`, another absolute
/// path) is refused before the filesystem is asked.
fn project_relative_path(root: &Path, path: &Path) -> io::Result<PathBuf> {
    // A path built from the canonical root (`/private/var/...` on macOS) still names a file
    // beneath a root given as `/var/...`. Only the root is canonicalized, never the path, so a
    // link in the path is still looked at, component by component, below.
    let canonical_root = root.canonicalize().ok();
    let beneath = path
        .strip_prefix(root)
        .ok()
        .or_else(|| {
            canonical_root
                .as_deref()
                .and_then(|canonical| path.strip_prefix(canonical).ok())
        })
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

/// Follow `relative` beneath `project` one component at a time and say what it leads to, or why
/// it is refused: a symbolic link that leaves the project (an absolute target included), a
/// dangling link, a non-directory on the way, or a last component that is neither a regular file
/// nor a directory.
///
/// This names the path and the reason. What enforces the rule is that every read and write then
/// goes through `project`, a `cap_std` directory handle that cannot be led outside itself, so a
/// component swapped after this look still cannot take the operation out of the project.
fn inspect_project_path(project: &Dir, relative: &Path) -> io::Result<ProjectTarget> {
    let mut reached = PathBuf::new();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        reached.push(component);
        let last = components.peek().is_none();
        let shown = reached.display();
        let found = match project.symlink_metadata(&reached) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(ProjectTarget::Absent);
            }
            Err(error) => return Err(error),
        };
        let resolved = if found.file_type().is_symlink() {
            match project.metadata(&reached) {
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
            }
        } else {
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

/// A directory handle on the project root, with `path` as a path beneath it.
fn open_project(root: &Path, path: &Path) -> io::Result<(Dir, PathBuf)> {
    let relative = project_relative_path(root, path)?;
    let project = Dir::open_ambient_dir(root, ambient_authority())?;
    Ok((project, relative))
}

/// The path leads to a file, or to nothing yet. A directory there is refused.
fn project_file_exists(project: &Dir, relative: &Path) -> io::Result<bool> {
    match inspect_project_path(project, relative)? {
        ProjectTarget::File => Ok(true),
        ProjectTarget::Absent => Ok(false),
        ProjectTarget::Directory => Err(refused_project_path(format!(
            "{} is not a regular file",
            relative.display()
        ))),
    }
}

/// Create the file with an exclusive create, which never follows a link at its last component,
/// after making the directories on the way.
fn create_new_project_file(project: &Dir, relative: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = relative.parent().filter(|p| !p.as_os_str().is_empty()) {
        project.create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = project.open_with(relative, &options)?;
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

/// Read a file of the project, `None` when nothing is there (REQ-util-002, REQ-util-003).
///
/// `path` is `root` joined with a relative path, or a path relative to `root`. A symbolic link
/// that stays inside the project is followed. A link that leaves it, a dangling link, anything
/// that is not a regular file, and a file larger than `max_bytes` are refused with an error that
/// names the path and the reason. The file is opened without blocking and its type is checked on
/// the open handle, so neither a FIFO nor a device can hold the read, and at most one byte past
/// the bound is ever read.
pub fn read_project_file(root: &Path, path: &Path, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
    let (project, relative) = open_project(root, path)?;
    if !project_file_exists(&project, &relative)? {
        return Ok(None);
    }
    let mut options = OpenOptions::new();
    // Non-blocking, so a FIFO swapped in after the look above cannot hold the open.
    options.read(true)._cap_fs_ext_nonblock(true);
    let file = project.open_with(&relative, &options)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(refused_project_path(format!(
            "{} is not a regular file",
            relative.display()
        )));
    }
    let over_bound = || {
        refused_project_path(format!(
            "{} is larger than {max_bytes} bytes",
            relative.display()
        ))
    };
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
/// A symbolic link that stays inside the project is written through. A link that leaves it, a
/// dangling link and anything that is not a regular file are refused with an error that names the
/// path, and nothing is written or created.
pub fn write_project_file(root: &Path, path: &Path, contents: &[u8]) -> io::Result<()> {
    let (project, relative) = open_project(root, path)?;
    if !project_file_exists(&project, &relative)? {
        return create_new_project_file(&project, &relative, contents);
    }
    // No create flag: an existing file is opened, so a link is never created through.
    let mut options = OpenOptions::new();
    options.write(true).truncate(true);
    let mut file = project.open_with(&relative, &options)?;
    file.write_all(contents)?;
    file.flush()
}

/// Create a file of the project that does not exist yet (REQ-util-002). `Ok(false)`, with nothing
/// changed, when a regular file is already there. The refusals of [`write_project_file`] apply.
pub fn create_project_file(root: &Path, path: &Path, contents: &[u8]) -> io::Result<bool> {
    let (project, relative) = open_project(root, path)?;
    if project_file_exists(&project, &relative)? {
        return Ok(false);
    }
    create_new_project_file(&project, &relative, contents).map(|()| true)
}

/// Remove a file of the project (REQ-util-002). The name removed is inside the project: a link
/// that stays inside is removed as a link, and its target is left. The refusals of
/// [`write_project_file`] apply.
pub fn remove_project_file(root: &Path, path: &Path) -> io::Result<()> {
    let (project, relative) = open_project(root, path)?;
    if !project_file_exists(&project, &relative)? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} does not exist", relative.display()),
        ));
    }
    project.remove_file(&relative)
}

/// Create a directory of the project and the directories on the way to it (REQ-util-002). The
/// refusals of [`write_project_file`] apply, and a file where the directory belongs is refused.
pub fn create_project_dir(root: &Path, path: &Path) -> io::Result<()> {
    let (project, relative) = open_project(root, path)?;
    match inspect_project_path(&project, &relative)? {
        ProjectTarget::Directory => Ok(()),
        ProjectTarget::Absent => project.create_dir_all(&relative),
        ProjectTarget::File => Err(refused_project_path(format!(
            "{} is not a directory",
            relative.display()
        ))),
    }
}

/// Why a project path is refused, in words for the user, without changing anything: a symbolic
/// link that leaves the project, a dangling link, a path outside the project, or something that
/// is neither a regular file nor a directory. `None` for a path the project file functions accept
/// (REQ-util-002).
pub fn project_path_refusal(root: &Path, path: &Path) -> Option<String> {
    let refusal = |error: io::Error| {
        error
            .get_ref()
            .is_some_and(|inner| inner.is::<RefusedProjectPath>())
            .then(|| error.to_string())
    };
    match open_project(root, path) {
        Ok((project, relative)) => inspect_project_path(&project, &relative)
            .err()
            .and_then(refusal),
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
            let reason = "CLAUDE.md is a symbolic link that leaves the project";

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
