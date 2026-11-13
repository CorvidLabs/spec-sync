use std::cell::RefCell;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};

/// Host environment keys a git child is allowed to inherit.
///
/// Secrets (`GITHUB_TOKEN`, `GH_TOKEN`, `SSH_AUTH_SOCK`) and git overrides
/// (`GIT_DIR`, `GIT_ASKPASS`, `GIT_INDEX_FILE`) are intentionally absent.
const GIT_INHERITED_ENV: &[&str] = &[
    "PATH",
    "PATHEXT",
    "HOME",
    "USER",
    "LOGNAME",
    "TMPDIR",
    "TEMP",
    "TMP",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "TZ",
    "SYSTEMROOT",
    "WINDIR",
    "COMSPEC",
];

thread_local! {
    static DISCOVERY_CEILING: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Run `f` so git children spawned on this thread set `GIT_CEILING_DIRECTORIES`
/// to `ceiling` after `env_clear`. Restores the previous slot on return or unwind.
///
/// Default `git_cmd` does not pin a ceiling, so a nested project still discovers
/// its parent work tree. MCP snapshot dispatch is the caller that needs isolation:
/// empirically (Git 2.39.5) ceiling=`parent(snapshot)` stops walk-up into a host
/// worktree, while ceiling=`snapshot` itself still walks up.
pub fn with_discovery_ceiling<R>(ceiling: &Path, f: impl FnOnce() -> R) -> R {
    struct Guard(Option<PathBuf>);
    impl Drop for Guard {
        fn drop(&mut self) {
            DISCOVERY_CEILING.with(|slot| {
                *slot.borrow_mut() = self.0.take();
            });
        }
    }
    let absolute = if ceiling.is_absolute() {
        ceiling.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(ceiling))
            .unwrap_or_else(|_| ceiling.to_path_buf())
    };
    let previous = DISCOVERY_CEILING.with(|slot| slot.replace(Some(absolute)));
    let _guard = Guard(previous);
    f()
}

/// Spawn `git` without inheriting host secrets or repo-override variables.
///
/// The MCP server (and every other caller of this module) used to PATH-resolve
/// `git` with the full parent environment, so a read-only `specsync mcp` process
/// that needed `GITHUB_TOKEN` for issue verification forwarded that token — and
/// `GIT_DIR` / `GIT_ASKPASS` — to whatever `git` was first on `PATH`.
///
/// Keep a small allowlist so git can actually run (PATH, locale, temp, home).
/// Do not set `GIT_CEILING_DIRECTORIES` on the default path: a project whose
/// root is a subdirectory of a git repository (monorepo package, nested
/// checkout) must still discover the parent work tree. `env_clear` already
/// drops any inherited ceiling. Snapshot isolation uses
/// [`with_discovery_ceiling`].
pub(crate) fn git_cmd(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.env_clear();
    for key in GIT_INHERITED_ENV {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    DISCOVERY_CEILING.with(|slot| {
        if let Some(ceiling) = slot.borrow().as_ref() {
            command.env("GIT_CEILING_DIRECTORIES", ceiling);
        }
    });
    command.env("GIT_TERMINAL_PROMPT", "0");
    command.env("GIT_OPTIONAL_LOCKS", "0");
    command.env("LC_ALL", "C");
    command.current_dir(root);
    command
}

/// A self-pipe that wakes readers blocked in `poll` when the direct child has exited.
///
/// Created after the child is spawned so the child cannot inherit it. Both readers poll
/// the read end and neither consumes the wake byte, so one write wakes every waiter.
#[cfg(unix)]
pub(crate) struct OutputWake {
    read: std::os::unix::io::OwnedFd,
    write: std::os::unix::io::OwnedFd,
}

#[cfg(unix)]
impl OutputWake {
    pub(crate) fn new() -> io::Result<Self> {
        use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

        let mut fds = [0; 2];
        // SAFETY: `fds` is a writable two-int buffer. On success `pipe` opens both ends.
        let rc = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if rc != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: both descriptors were just opened by `pipe` and are uniquely owned here.
        let read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let write = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        set_cloexec(read.as_raw_fd())?;
        set_cloexec(write.as_raw_fd())?;
        Ok(Self { read, write })
    }

    pub(crate) fn raw_read(&self) -> std::os::unix::io::RawFd {
        use std::os::unix::io::AsRawFd;
        self.read.as_raw_fd()
    }

    /// `stop` is stored before the write, and `poll` returning is ordered after that write.
    pub(crate) fn signal(&self, stop: &AtomicBool) {
        use std::os::unix::io::AsRawFd;

        stop.store(true, Ordering::SeqCst);
        let byte = [1u8];
        loop {
            // SAFETY: `write` refers to the pipe end this guard owns, and `byte` is one live byte.
            let rc = unsafe { libc::write(self.write.as_raw_fd(), byte.as_ptr().cast(), 1) };
            if rc >= 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                break;
            }
        }
    }
}

#[cfg(unix)]
fn set_cloexec(fd: std::os::unix::io::RawFd) -> io::Result<()> {
    // SAFETY: `fd` is an open descriptor owned by the caller.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fd` is the same open descriptor; `FD_CLOEXEC` only changes its flag word.
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(unix)]
fn set_nonblocking(fd: std::os::unix::io::RawFd) -> io::Result<()> {
    // SAFETY: `fd` is an open descriptor owned by the caller.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fd` is the same open descriptor; `O_NONBLOCK` only changes its status flags.
    let rc = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Read `reader` until EOF or until `stop` is set.
///
/// While the child is still running, each successful read is one buffer and then
/// `stop` is checked again, so a descendant that keeps the pipe readable after the
/// child exits cannot hold this function in a drain loop. When `stop` is observed,
/// one `FIONREAD` snapshot of the bytes already queued is drained and the function
/// returns. Bytes the child had written by the time it was reaped are in that
/// snapshot or were already kept. Later descendant writes are not followed.
/// Past `cap`, further bytes are discarded and `overflow` is set.
#[cfg(unix)]
pub(crate) fn read_pipe_until_stop<R>(
    mut reader: R,
    stop: &AtomicBool,
    wake: std::os::unix::io::RawFd,
    cap: usize,
    overflow: &AtomicBool,
) -> io::Result<Vec<u8>>
where
    R: Read + std::os::unix::io::AsRawFd,
{
    set_nonblocking(reader.as_raw_fd())?;
    let mut retained = Vec::new();
    let mut buffer = [0u8; 16 * 1024];
    loop {
        if stop.load(Ordering::SeqCst) {
            drain_queued_snapshot(&mut reader, &mut buffer, &mut retained, cap, overflow)?;
            return Ok(retained);
        }
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(retained),
            Ok(read) => retain_bytes(&mut retained, &buffer[..read], cap, overflow),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                // The child can write its last bytes and exit in this window.
                // Honor stop only after that queued snapshot is taken.
                if stop.load(Ordering::SeqCst) {
                    drain_queued_snapshot(&mut reader, &mut buffer, &mut retained, cap, overflow)?;
                    return Ok(retained);
                }
                wait_for_pipe_or_wake(reader.as_raw_fd(), wake)?;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(unix)]
fn retain_bytes(retained: &mut Vec<u8>, chunk: &[u8], cap: usize, overflow: &AtomicBool) {
    let remaining = cap.saturating_sub(retained.len());
    let keep = remaining.min(chunk.len());
    retained.extend_from_slice(&chunk[..keep]);
    if keep != chunk.len() {
        overflow.store(true, Ordering::Release);
    }
}

/// Bytes sitting in the pipe right now. A descendant that writes more after this
/// call does not increase the count, so the following read is finite.
#[cfg(unix)]
fn queued_len(fd: std::os::unix::io::RawFd) -> io::Result<usize> {
    let mut available: libc::c_int = 0;
    // SAFETY: `fd` is the caller's open pipe read end. `FIONREAD` writes one int.
    let rc = unsafe { libc::ioctl(fd, libc::FIONREAD, &mut available) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(available.max(0) as usize)
}

#[cfg(unix)]
fn drain_queued_snapshot<R>(
    reader: &mut R,
    buffer: &mut [u8],
    retained: &mut Vec<u8>,
    cap: usize,
    overflow: &AtomicBool,
) -> io::Result<()>
where
    R: Read + std::os::unix::io::AsRawFd,
{
    let mut left = queued_len(reader.as_raw_fd())?;
    while left > 0 {
        let want = left.min(buffer.len());
        match reader.read(&mut buffer[..want]) {
            Ok(0) => return Ok(()),
            Ok(read) => {
                retain_bytes(retained, &buffer[..read], cap, overflow);
                left -= read;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn wait_for_pipe_or_wake(
    pipe: std::os::unix::io::RawFd,
    wake: std::os::unix::io::RawFd,
) -> io::Result<()> {
    let mut fds = [
        libc::pollfd {
            fd: pipe,
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: wake,
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    loop {
        // SAFETY: `fds` holds two open descriptors owned for the duration of this wait.
        let rc = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1) };
        if rc < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        return Ok(());
    }
}

/// Collect stdout and stderr, and return when the direct child exits.
///
/// Stdin is the null device, matching `Command::output`: the child cannot read
/// the caller's input. Stdout and stderr are pipes this process owns, so a
/// grandchild cannot hold the caller's own stdout or stderr. A grandchild that
/// keeps the child's pipe open does not delay the return: readers stop once the
/// child has been reaped, after a finite snapshot of the bytes already queued.
pub(crate) fn output_after_child_exits(command: &mut Command) -> io::Result<Output> {
    use std::process::Stdio;

    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("failed to capture child stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("failed to capture child stderr"))?;

    #[cfg(unix)]
    {
        use std::sync::Arc;
        use std::thread;

        let stop = Arc::new(AtomicBool::new(false));
        let wake = OutputWake::new().inspect_err(|_| {
            let _ = child.kill();
            let _ = child.wait();
        })?;
        let wake_fd = wake.raw_read();
        let stdout_stop = Arc::clone(&stop);
        let stderr_stop = Arc::clone(&stop);
        let stdout_reader = thread::spawn(move || {
            let overflow = AtomicBool::new(false);
            read_pipe_until_stop(stdout, &stdout_stop, wake_fd, usize::MAX, &overflow)
        });
        let stderr_reader = thread::spawn(move || {
            let overflow = AtomicBool::new(false);
            read_pipe_until_stop(stderr, &stderr_stop, wake_fd, usize::MAX, &overflow)
        });
        let status = child.wait();
        wake.signal(&stop);
        let stdout = join_reader(stdout_reader);
        let stderr = join_reader(stderr_reader);
        Ok(Output {
            status: status?,
            stdout: stdout?,
            stderr: stderr?,
        })
    }

    #[cfg(not(unix))]
    {
        collect_output_after_exit_windows(&mut child, stdout, stderr)
    }
}

#[cfg(unix)]
fn join_reader(reader: std::thread::JoinHandle<io::Result<Vec<u8>>>) -> io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| io::Error::other("child output reader panicked"))?
}

/// Run `command` and return when the direct child exits.
///
/// Stdout and stderr are `/dev/null` (or the platform null device), not the
/// caller's pipes. A grandchild of a `status` call therefore cannot keep the
/// caller's output open after this process exits.
pub(crate) fn status_without_inherited_output(command: &mut Command) -> io::Result<ExitStatus> {
    use std::process::Stdio;

    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command.status()
}

#[cfg(not(unix))]
fn collect_output_after_exit_windows(
    child: &mut std::process::Child,
    mut stdout: std::process::ChildStdout,
    mut stderr: std::process::ChildStderr,
) -> io::Result<Output> {
    use std::os::windows::io::AsRawHandle;
    use std::thread;
    use std::time::Duration;

    let mut out = Vec::new();
    let mut err = Vec::new();
    let status = loop {
        drain_available_windows(stdout.as_raw_handle(), &mut stdout, &mut out)?;
        drain_available_windows(stderr.as_raw_handle(), &mut stderr, &mut err)?;
        if let Some(status) = child.try_wait()? {
            drain_available_windows(stdout.as_raw_handle(), &mut stdout, &mut out)?;
            drain_available_windows(stderr.as_raw_handle(), &mut stderr, &mut err)?;
            break status;
        }
        thread::sleep(Duration::from_millis(2));
    };
    Ok(Output {
        status,
        stdout: out,
        stderr: err,
    })
}

#[cfg(not(unix))]
fn drain_available_windows<R: Read>(
    handle: std::os::windows::io::RawHandle,
    reader: &mut R,
    retained: &mut Vec<u8>,
) -> io::Result<()> {
    let available = match windows_pipe_available(handle) {
        Ok(available) => available,
        // ERROR_BROKEN_PIPE and ERROR_NO_DATA: the write end closed after the
        // stream was drained. That is end of stream, not a failed Git call.
        Err(error) if matches!(error.raw_os_error(), Some(109 | 232)) => return Ok(()),
        Err(error) => return Err(error),
    };
    if available == 0 {
        return Ok(());
    }
    let mut limited = reader.take(u64::from(available));
    limited.read_to_end(retained)?;
    Ok(())
}

#[cfg(not(unix))]
fn windows_pipe_available(handle: std::os::windows::io::RawHandle) -> io::Result<u32> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn PeekNamedPipe(
            handle: *mut std::ffi::c_void,
            buffer: *mut std::ffi::c_void,
            buffer_size: u32,
            bytes_read: *mut u32,
            total_bytes_avail: *mut u32,
            bytes_left: *mut u32,
        ) -> i32;
    }

    let mut available = 0u32;
    // SAFETY: `handle` is the raw pipe handle of a `ChildStdout`/`ChildStderr` this
    // process owns. `PeekNamedPipe` with a null buffer only reports the queued byte count.
    let ok = unsafe {
        PeekNamedPipe(
            handle.cast(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            &mut available,
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(available)
}

/// Why a tree cannot be asked how far a spec has fallen behind its source.
///
/// Both variants mean the same thing to a caller measuring drift: there is no
/// history, so the distance is UNKNOWN — never zero. They differ only in what
/// a human should be told to do about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingHistory {
    /// `root` is not inside a git repository.
    NotARepository,
    /// A git repository whose `HEAD` is unborn — `git init` and nothing since.
    NoCommits,
}

impl MissingHistory {
    /// Lowercase reason, for machine payloads and mid-sentence use.
    pub fn reason(self) -> &'static str {
        match self {
            Self::NotARepository => "not a git repository",
            Self::NoCommits => "repository has no commits",
        }
    }

    /// Sentence-cased reason, for the head of a human-readable error line.
    pub fn sentence(self) -> &'static str {
        match self {
            Self::NotARepository => "Not a git repository",
            Self::NoCommits => "Repository has no commits",
        }
    }
}

/// Whether `root` has committed history to measure staleness against.
///
/// `None` means history is usable. `Some(..)` means every spec-behind-source
/// distance in this tree is unknown, and any caller about to report `0` or
/// `false` is reporting an answer to a question git could not be asked.
pub fn missing_history(root: &Path) -> Option<MissingHistory> {
    if !is_git_repo(root) {
        return Some(MissingHistory::NotARepository);
    }
    if !has_commits(root) {
        return Some(MissingHistory::NoCommits);
    }
    None
}

/// The commit a spec's staleness is measured from.
///
/// This type exists to make the bug in #572 unspellable. The old
/// `git_last_commit_hash -> Option<String>` collapsed two unrelated absences
/// into one `None`: "history exists, this spec simply is not in it" (drift is
/// genuinely zero) and "there is no history at all" (drift is UNKNOWN). Four
/// call sites read that `None` as the former and silently reported everything
/// up to date on a tree that had no `.git` — `report`, `check --stale`,
/// `score`, and the lifecycle `no_stale` guard, while only `stale` got it
/// right. Splitting the absence forces every caller — including the next one
/// written — through an exhaustive `match` the compiler checks, instead of
/// relying on each author to remember a guard helper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecBaseline {
    /// The commit that last touched the spec. Drift is measurable from here.
    Commit(String),
    /// History exists, but git has no record of this spec file yet. There is
    /// nothing for the spec to be behind, so drift is genuinely zero.
    Untracked,
    /// There is no history to ask. Drift is unknown, and MUST NOT be reported
    /// as zero.
    Missing(MissingHistory),
}

/// Resolve the commit a spec's staleness is measured from.
///
/// The `missing_history` probe runs only when no commit came back, so a healthy
/// repository still costs exactly one `git log` per spec.
pub fn spec_baseline(root: &Path, spec_file: &str) -> SpecBaseline {
    match last_commit_hash(root, spec_file) {
        Some(hash) => SpecBaseline::Commit(hash),
        // A hash would have proven history exists; none did, so ask why.
        None => match missing_history(root) {
            Some(missing) => SpecBaseline::Missing(missing),
            None => SpecBaseline::Untracked,
        },
    }
}

/// Get the last commit hash that touched a file.
///
/// Deliberately private: its `None` is ambiguous, and every caller outside this
/// module must go through [`spec_baseline`], which disambiguates it.
fn last_commit_hash(root: &Path, file: &str) -> Option<String> {
    let output =
        output_after_child_exits(git_cmd(root).args(["log", "-1", "--format=%H", "--", file]))
            .ok()?;
    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() { None } else { Some(hash) }
}

/// Count commits that touched `source_file` since the given `spec_commit`.
///
/// Takes a precomputed spec commit hash (from [`git_last_commit_hash`]) so
/// callers iterating over a spec's source files spawn one `git rev-list` per
/// file instead of also re-resolving the spec's commit each time.
pub fn git_commits_since(root: &Path, spec_commit: &str, source_file: &str) -> usize {
    // Content first: add-then-revert commit pairs leave the file byte-identical
    // to its state at `spec_commit`, and pure commit counting would report
    // phantom drift. If the working-tree content matches the content at
    // `spec_commit`, there is nothing to catch up on.
    if let Ok(diff) = status_without_inherited_output(git_cmd(root).args([
        "diff",
        "--quiet",
        spec_commit,
        "--",
        source_file,
    ])) && diff.success()
    {
        return 0;
    }
    let output = match output_after_child_exits(git_cmd(root).args([
        "rev-list",
        "--count",
        &format!("{spec_commit}..HEAD"),
        "--",
        source_file,
    ])) {
        Ok(o) => o,
        Err(_) => return 0,
    };

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
}

/// Check if the current directory is inside a git repository.
pub fn is_git_repo(root: &Path) -> bool {
    git_cmd_output(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Whether the repository has at least one commit.
///
/// A repository with an unborn `HEAD` is a git repository by every other test,
/// but there is no history for anything to be newer or older than. Callers that
/// decide staleness from history must treat it the same way they treat "not a
/// git repository" rather than reporting everything current (#558) — it is the
/// state `git init` leaves behind, which is where the quick start begins.
pub fn has_commits(root: &Path) -> bool {
    git_cmd_output(root, &["rev-parse", "--verify", "HEAD"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Staleness info for a single spec relative to its source files.
#[derive(Debug, Clone)]
pub struct StaleInfo {
    /// Relative path to the spec file.
    pub spec_path: String,
    /// Module name from frontmatter.
    pub module_name: String,
    /// Maximum commits behind across all source files.
    pub max_commits_behind: usize,
    /// Per-source-file commit distances (file, commits_behind).
    pub source_details: Vec<(String, usize)>,
    /// Cited files that no longer exist. A spec is stale on this alone,
    /// whatever the drift threshold, and every renderer needs it: a row
    /// reading `0 commits behind` with no drifted files is stale with no
    /// visible cause.
    pub deleted_files: Vec<String>,
}

/// Was this path known to git at `since`, and is it gone now?
///
/// Distinguishes a DELETION — a definite fact git can state, with a commit that
/// performed it — from a path git never knew, which is genuinely unmeasurable.
/// Collapsing the two loses the stronger claim: `stale` would shrug at a
/// deletion that `lifecycle` can name.
pub fn source_was_deleted(root: &Path, since: &str, path: &str) -> bool {
    if root.join(path).exists() {
        return false;
    }
    // `<rev>:./<path>` resolves relative to cwd, which is `root`; the
    // repo-root-relative form would answer the wrong question whenever the
    // project sits in a subdirectory of the repository.
    status_without_inherited_output(git_cmd(root).args([
        "cat-file",
        "-e",
        &format!("{since}:./{path}"),
    ]))
    .map(|s| s.success())
    .unwrap_or(false)
}

/// A spec whose staleness could not be measured at all.
///
/// Distinct from a fresh spec. A fresh spec was compared against its sources
/// and found current; this one had nothing to compare, because every file it
/// cites is absent or is a directory. Counting it fresh spends it against the
/// "up to date" total using evidence that was never gathered.
pub struct UnmeasurableSpec {
    pub spec_path: String,
    pub module_name: String,
    /// Each unmeasurable file and why, so the report names the cause rather
    /// than only the effect.
    pub files: Vec<(String, &'static str)>,
}

// ─── Repository facts (REQ-git-utils-009) ────────────────────────────────

/// What one memoized git query answered.
#[derive(Clone, Debug)]
pub(crate) struct MemoizedGitOutput {
    pub(crate) status: std::process::ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

/// One open repository-fact scope: the facts git already answered for `root` in this command.
struct RepositoryFactFrame {
    root: PathBuf,
    facts: std::collections::HashMap<(&'static str, Vec<String>), MemoizedGitOutput>,
    /// One `git cat-file --batch` child per git command flavor (REQ-git-utils-010).
    batches: std::collections::HashMap<&'static str, ObjectBatch>,
    /// False in a worker thread's seeded scope: it never starts a batch reader, so a command
    /// keeps one `cat-file --batch` child per root however many workers it runs.
    batch_reader: bool,
}

thread_local! {
    static REPOSITORY_FACT_SCOPES: RefCell<Vec<RepositoryFactFrame>> = const {
        RefCell::new(Vec::new())
    };
    static LIVE_REPOSITORY_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// While held, every git read on this thread asks git: nothing is answered from memoized facts
/// or a batch reader. Race checks hold one, because they read the repository twice precisely to
/// see a change made by another process during the command (REQ-git-utils-009).
#[derive(Debug)]
pub(crate) struct LiveRepositoryReads {
    _private: (),
}

impl Drop for LiveRepositoryReads {
    fn drop(&mut self) {
        LIVE_REPOSITORY_READS.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

/// Make every git read on this thread live until the guard drops.
pub(crate) fn live_repository_reads() -> LiveRepositoryReads {
    LIVE_REPOSITORY_READS.with(|depth| depth.set(depth.get() + 1));
    LiveRepositoryReads { _private: () }
}

fn repository_reads_are_live() -> bool {
    LIVE_REPOSITORY_READS.with(|depth| depth.get() > 0)
}

/// Drop guard for one repository-fact scope.
///
/// Lifecycle commands hold one for as long as they hold the project lock, and read-only lifecycle
/// commands for as long as their read scope. Facts never outlive the guard, so a `git commit`
/// made between two commands (or by a test between two calls) is always seen by the next one.
/// Scopes are per thread: worker threads spawned inside a command do not memoize.
#[derive(Debug)]
pub(crate) struct RepositoryFactScope {
    root: PathBuf,
}

impl Drop for RepositoryFactScope {
    fn drop(&mut self) {
        REPOSITORY_FACT_SCOPES.with(|scopes| {
            let removed = scopes.borrow_mut().pop();
            debug_assert_eq!(
                removed.as_ref().map(|frame| frame.root.as_path()),
                Some(self.root.as_path())
            );
        });
    }
}

/// Open a repository-fact scope for `root` on this thread until the guard drops.
pub(crate) fn begin_repository_fact_scope(root: &Path) -> RepositoryFactScope {
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        // A scope nested in an open scope for the same root starts from what the outer one already
        // knows: both live in this command, and invalidation clears every open frame.
        let facts = scopes
            .last()
            .filter(|frame| frame.root == root)
            .map(|frame| frame.facts.clone())
            .unwrap_or_default();
        scopes.push(RepositoryFactFrame {
            root: root.to_path_buf(),
            facts,
            batches: std::collections::HashMap::new(),
            batch_reader: true,
        });
    });
    RepositoryFactScope {
        root: root.to_path_buf(),
    }
}

/// What the open scopes for one root know on this thread, to seed a read-only worker thread.
#[derive(Clone, Debug)]
pub(crate) struct RepositoryFactSeed {
    root: PathBuf,
    facts: std::collections::HashMap<(&'static str, Vec<String>), MemoizedGitOutput>,
}

/// The facts this thread's open scopes for `root` hold, for worker threads that only read
/// (REQ-git-utils-011). `None` when no scope for `root` is open, reads are live, or a discovery
/// ceiling is pinned; a worker then runs without a scope, as before.
pub(crate) fn repository_fact_seed(root: &Path) -> Option<RepositoryFactSeed> {
    if repository_reads_are_live() || discovery_ceiling_is_set() {
        return None;
    }
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        let scopes = scopes.borrow();
        let mut frames = scopes.iter().filter(|frame| frame.root == root).peekable();
        frames.peek()?;
        let mut facts = std::collections::HashMap::new();
        for frame in frames {
            facts.extend(
                frame
                    .facts
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        Some(RepositoryFactSeed {
            root: root.to_path_buf(),
            facts,
        })
    })
}

/// Open a scope on a read-only worker thread that starts from `seed` (REQ-git-utils-011). The
/// worker memoizes what it asks in its own scope, never starts a batch reader, and must not write
/// to the repository; the facts it was seeded with stay true because nothing in the command writes
/// while its workers run.
pub(crate) fn begin_seeded_repository_fact_scope(seed: RepositoryFactSeed) -> RepositoryFactScope {
    let root = seed.root.clone();
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        scopes.borrow_mut().push(RepositoryFactFrame {
            root: seed.root,
            facts: seed.facts,
            batches: std::collections::HashMap::new(),
            batch_reader: false,
        });
    });
    RepositoryFactScope { root }
}

/// Whether `args` asks git a question whose answer only a git write can change.
///
/// These read refs, objects and configuration only, never the working tree or the index, so the
/// files a lifecycle command writes cannot change their answers: `rev-parse` (without `--stdin`),
/// the read forms of `symbolic-ref` and `config`, and the history and object reads `rev-list`,
/// `log`, `show`, `ls-tree`, `merge-base`, `cat-file` (single-object forms), `diff-tree`,
/// `for-each-ref` and `show-ref`. `ls-files`, `diff`, `diff-files`, `diff-index`, `status` and
/// `check-attr` read the working tree or index and are never memoized.
fn is_repository_fact(args: &[&str]) -> bool {
    if args.iter().any(|argument| {
        matches!(
            *argument,
            "--stdin" | "--batch" | "--batch-check" | "--no-index"
        ) || argument.starts_with("--batch")
    }) {
        return false;
    }
    matches!(
        args,
        ["rev-parse", ..]
            | ["rev-list", ..]
            | ["log", ..]
            | ["show", ..]
            | ["ls-tree", ..]
            | ["merge-base", ..]
            | ["cat-file", ..]
            | ["diff-tree", ..]
            | ["for-each-ref", ..]
            | ["show-ref", ..]
            | ["symbolic-ref", "--short", _]
            | ["symbolic-ref", "-q", "--short", _]
            | ["symbolic-ref", "-q", _]
            | ["symbolic-ref", _]
    ) || is_config_read(args)
}

/// Whether `args` is a `git config` lookup: optional scope and format options, then `--get`,
/// `--get-all` or `--get-regexp` and at least one more argument. Anything else may write.
fn is_config_read(args: &[&str]) -> bool {
    let ["config", options @ ..] = args else {
        return false;
    };
    let mut rest = options.iter();
    for option in rest.by_ref() {
        match *option {
            "--local" | "--global" | "--system" | "--worktree" | "-z" | "--null" | "--includes"
            | "--no-includes" | "--bool" | "--int" | "--path" => {}
            "--get" | "--get-all" | "--get-regexp" => return rest.next().is_some(),
            _ => return false,
        }
    }
    false
}

/// The git subcommand in `args`, skipping leading `-c key=value` pairs and other global options.
fn git_subcommand<'a>(args: &[&'a str]) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(argument) = iter.next() {
        match *argument {
            "-c" | "-C" | "--git-dir" | "--work-tree" | "--namespace" => {
                iter.next();
            }
            option if option.starts_with('-') => {}
            subcommand => return Some(subcommand),
        }
    }
    None
}

/// Whether running `args` cannot change any memoized repository fact.
fn is_read_only_git(args: &[&str]) -> bool {
    match git_subcommand(args) {
        Some(
            "rev-parse" | "show" | "ls-files" | "ls-tree" | "rev-list" | "log" | "cat-file"
            | "merge-base" | "check-attr" | "check-ignore" | "diff" | "diff-files" | "diff-tree"
            | "diff-index" | "status" | "for-each-ref" | "show-ref" | "name-rev" | "describe"
            | "var" | "grep" | "blame" | "count-objects" | "hash-object",
        ) => !args.contains(&"-w"),
        Some("symbolic-ref") => is_repository_fact(args),
        Some("config") => is_repository_fact(args),
        Some("worktree") => args.contains(&"list"),
        _ => false,
    }
}

/// Whether running `args` cannot change any memoized repository fact, for callers that run git
/// outside [`note_repository_git`].
pub(crate) fn is_read_only_git_invocation(args: &[&str]) -> bool {
    is_read_only_git(args)
}

/// The memoized answer to `args` for `root` under `flavor`, if a scope for `root` is open and
/// already holds one. Never answers while a discovery ceiling is pinned.
pub(crate) fn memoized_repository_fact(
    root: &Path,
    flavor: &'static str,
    args: &[&str],
) -> Option<MemoizedGitOutput> {
    if !is_repository_fact(args) || discovery_ceiling_is_set() || repository_reads_are_live() {
        return None;
    }
    let key: (&'static str, Vec<String>) = (
        flavor,
        args.iter()
            .map(|argument| (*argument).to_string())
            .collect(),
    );
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        scopes
            .borrow()
            .iter()
            .rev()
            .filter(|frame| frame.root == root)
            .find_map(|frame| frame.facts.get(&key).cloned())
    })
}

/// Record what git answered for `args`, and forget every fact for `root` after anything that may
/// write. Call once per git process that ran to completion; a process that could not be started,
/// timed out or was killed is not an answer and is never remembered.
pub(crate) fn note_repository_git(
    root: &Path,
    flavor: &'static str,
    args: &[&str],
    output: &MemoizedGitOutput,
) {
    if !is_read_only_git(args) {
        invalidate_repository_facts();
        return;
    }
    if !is_repository_fact(args)
        || discovery_ceiling_is_set()
        || !(output.status.success() || is_definite_absence(args, output))
    {
        return;
    }
    let key: (&'static str, Vec<String>) = (
        flavor,
        args.iter()
            .map(|argument| (*argument).to_string())
            .collect(),
    );
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        for frame in scopes
            .borrow_mut()
            .iter_mut()
            .filter(|frame| frame.root == root)
        {
            frame.facts.insert(key.clone(), output.clone());
        }
    });
}

/// Whether a nonzero exit is a definite answer rather than a failure, and so a fact worth keeping
/// for the command: `git config` lookups exit 1 when the key is unset, and `symbolic-ref -q` and
/// `rev-parse --verify --quiet` exit 1 when the ref is absent. Any other nonzero exit is a failed
/// query, which is never memoized, so the next read asks git again (REQ-git-utils-009, #887).
fn is_definite_absence(args: &[&str], output: &MemoizedGitOutput) -> bool {
    let code = output.status.code();
    if code == Some(1)
        && (is_config_read(args)
            || matches!(args, ["symbolic-ref", "-q", ..])
            || (args.first() == Some(&"rev-parse")
                && args.contains(&"--verify")
                && args.contains(&"--quiet")))
    {
        return true;
    }
    // `show <rev>:<path>` of a path the revision does not hold: Git names the path and the
    // revision, and the batch reader reports the name missing after resolving the revision. Any
    // other 128, such as an unreadable object or a failed fetch, stays a failure.
    if code == Some(128) && args.first() == Some(&"show") {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return stderr.contains("does not exist in '")
            || stderr.contains("exists on disk, but not in '")
            || stderr.starts_with("fatal: invalid object name '");
    }
    false
}

/// Forget every memoized repository fact on this thread: something outside the fact rules ran,
/// such as a git write or a configured verification command.
pub(crate) fn invalidate_repository_facts() {
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        for frame in scopes.borrow_mut().iter_mut() {
            frame.facts.clear();
            // A long-lived reader may hold a ref snapshot from before the write.
            frame.batches.clear();
        }
    });
}

fn discovery_ceiling_is_set() -> bool {
    DISCOVERY_CEILING.with(|slot| slot.borrow().is_some())
}

// ─── Batched object reads (REQ-git-utils-010) ────────────────────────────

/// How long one batched lookup may take before its child is killed: the same wall-clock bound a
/// per-file git read has.
const BATCH_READ_DEADLINE: std::time::Duration = std::time::Duration::from_secs(120);

/// A running `git cat-file --batch` child. Dropping it closes stdin and reaps the child.
struct ObjectBatch {
    child: std::sync::Arc<std::sync::Mutex<std::process::Child>>,
    stdin: Option<std::process::ChildStdin>,
    stdout: std::io::BufReader<std::process::ChildStdout>,
    deadline: std::time::Duration,
    watchdog: BatchWatchdog,
    /// The commit `HEAD` named when this reader first resolved it; every later `HEAD` read in the
    /// reader's lifetime uses it (REQ-git-utils-010).
    head: Option<String>,
}

/// Kills the batch child when an armed lookup outlives its deadline, so a stalled object store
/// cannot hang the command; the blocked read then sees the pipe close and fails.
struct BatchWatchdog {
    state: std::sync::Arc<(std::sync::Mutex<WatchdogState>, std::sync::Condvar)>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[derive(Default)]
struct WatchdogState {
    armed_until: Option<std::time::Instant>,
    closed: bool,
}

impl BatchWatchdog {
    fn start(child: std::sync::Arc<std::sync::Mutex<std::process::Child>>) -> Self {
        let state = std::sync::Arc::new((
            std::sync::Mutex::new(WatchdogState::default()),
            std::sync::Condvar::new(),
        ));
        let watched = std::sync::Arc::clone(&state);
        let thread = std::thread::spawn(move || {
            let (lock, wake) = &*watched;
            let Ok(mut current) = lock.lock() else { return };
            loop {
                if current.closed {
                    return;
                }
                current = match current.armed_until {
                    Some(until) => {
                        let now = std::time::Instant::now();
                        if now >= until {
                            if let Ok(mut child) = child.lock() {
                                let _ = child.kill();
                            }
                            return;
                        }
                        match wake.wait_timeout(current, until - now) {
                            Ok((guard, _)) => guard,
                            Err(_) => return,
                        }
                    }
                    None => match wake.wait(current) {
                        Ok(guard) => guard,
                        Err(_) => return,
                    },
                };
            }
        });
        Self {
            state,
            thread: Some(thread),
        }
    }

    fn set(&self, armed_until: Option<std::time::Instant>) {
        let (lock, wake) = &*self.state;
        if let Ok(mut current) = lock.lock() {
            current.armed_until = armed_until;
            wake.notify_all();
        }
    }
}

impl Drop for BatchWatchdog {
    fn drop(&mut self) {
        let (lock, wake) = &*self.state;
        if let Ok(mut current) = lock.lock() {
            current.closed = true;
            wake.notify_all();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ObjectBatch {
    fn drop(&mut self) {
        drop(self.stdin.take());
        if let Ok(mut child) = self.child.lock() {
            if !matches!(child.try_wait(), Ok(Some(_))) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

/// What one batched lookup found.
enum BatchedObject {
    /// The name resolves to this object, of this type, with these bytes.
    Found {
        oid: String,
        kind: String,
        bytes: Vec<u8>,
    },
    /// Git reports the name missing or ambiguous.
    Missing,
}

impl ObjectBatch {
    fn start(command: Command) -> std::io::Result<Self> {
        Self::start_with_deadline(command, BATCH_READ_DEADLINE)
    }

    fn start_with_deadline(
        mut command: Command,
        deadline: std::time::Duration,
    ) -> std::io::Result<Self> {
        use std::process::Stdio;
        let mut child = command
            .args(["cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .map(std::io::BufReader::new)
            .ok_or_else(|| std::io::Error::other("cat-file --batch has no stdout"))?;
        let child = std::sync::Arc::new(std::sync::Mutex::new(child));
        let watchdog = BatchWatchdog::start(std::sync::Arc::clone(&child));
        Ok(Self {
            child,
            stdin,
            stdout,
            deadline,
            watchdog,
            head: None,
        })
    }

    /// The commit `HEAD` names for this reader: the memoized `HEAD` when `known` has it,
    /// otherwise resolved once through the reader and kept for its lifetime.
    fn head(&mut self, known: Option<String>) -> std::io::Result<String> {
        if let Some(head) = &self.head {
            return Ok(head.clone());
        }
        let head = match known {
            Some(head) => head,
            None => match self.read("HEAD", usize::MAX)? {
                BatchedObject::Found { oid, kind, .. } if kind == "commit" => oid,
                _ => return Err(std::io::Error::other("HEAD does not name a commit")),
            },
        };
        self.head = Some(head.clone());
        Ok(head)
    }

    /// Look `name` up, killing the child if it does not answer within the deadline.
    fn read(&mut self, name: &str, max_bytes: usize) -> std::io::Result<BatchedObject> {
        self.watchdog
            .set(Some(std::time::Instant::now() + self.deadline));
        let found = self.read_unwatched(name, max_bytes);
        self.watchdog.set(None);
        found
    }

    /// Look `name` up. `max_bytes` bounds the object: a larger one is an error, raised from the
    /// header before any payload is read, so the caller discards this reader and falls back to
    /// the per-file read and its own bound.
    fn read_unwatched(&mut self, name: &str, max_bytes: usize) -> std::io::Result<BatchedObject> {
        use std::io::{BufRead, Read, Write};
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| std::io::Error::other("cat-file --batch stdin closed"))?;
        stdin.write_all(name.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        let mut header = String::new();
        if self.stdout.read_line(&mut header)? == 0 {
            return Err(std::io::Error::other("cat-file --batch ended"));
        }
        let header = header.trim_end_matches('\n');
        if header.ends_with(" missing") || header.ends_with(" ambiguous") {
            return Ok(BatchedObject::Missing);
        }
        let mut fields = header.split(' ');
        let (Some(oid), Some(kind), Some(size), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(std::io::Error::other(format!(
                "unexpected cat-file --batch header `{header}`"
            )));
        };
        let size: usize = size
            .parse()
            .map_err(|_| std::io::Error::other("unreadable cat-file --batch size"))?;
        if size > max_bytes {
            return Err(std::io::Error::other("object exceeds the read bound"));
        }
        let mut bytes = vec![0; size];
        self.stdout.read_exact(&mut bytes)?;
        let mut newline = [0_u8; 1];
        self.stdout.read_exact(&mut newline)?;
        Ok(BatchedObject::Found {
            oid: oid.to_string(),
            kind: kind.to_string(),
            bytes,
        })
    }
}

/// Look up `name` through the open scope's batch reader for `flavor`, starting it with
/// `command` on first use. `None` when no scope for `root` is open on this thread, a discovery
/// ceiling is pinned, or the reader failed (the reader is then discarded); callers fall back to
/// one git process for the read.
fn batched_object(
    root: &Path,
    flavor: &'static str,
    command: impl FnOnce() -> Command,
    name: &str,
    max_bytes: usize,
) -> Option<BatchedObject> {
    if name.contains('\n') {
        return None;
    }
    with_batch(root, flavor, command, |batch| batch.read(name, max_bytes))
}

/// Run `use_batch` on the open scope's batch reader for `flavor`, starting it with `command` on
/// first use. A reader that fails is discarded, since a half-read answer may still be in its pipe.
fn with_batch<Answer>(
    root: &Path,
    flavor: &'static str,
    command: impl FnOnce() -> Command,
    use_batch: impl FnOnce(&mut ObjectBatch) -> std::io::Result<Answer>,
) -> Option<Answer> {
    if discovery_ceiling_is_set() || repository_reads_are_live() {
        return None;
    }
    REPOSITORY_FACT_SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        if scopes
            .iter()
            .any(|frame| frame.root == root && !frame.batch_reader)
        {
            return None;
        }
        // One reader per root: a nested scope for the same root uses the reader an enclosing
        // scope already started, and a new reader belongs to the outermost scope for the root,
        // so it outlives the nested scope that first needed it (REQ-git-utils-010).
        let index = match scopes
            .iter()
            .position(|frame| frame.root == root && frame.batches.contains_key(flavor))
        {
            Some(index) => index,
            None => {
                let index = scopes.iter().position(|frame| frame.root == root)?;
                let batch = ObjectBatch::start(command()).ok()?;
                #[cfg(test)]
                record_test_batch_start();
                scopes[index].batches.insert(flavor, batch);
                index
            }
        };
        let frame = &mut scopes[index];
        let batch = frame.batches.get_mut(flavor)?;
        match use_batch(batch) {
            Ok(answer) => Some(answer),
            Err(_) => {
                frame.batches.remove(flavor);
                None
            }
        }
    })
}

/// The object id `revision` names for batched reads: a full object id as given, and `HEAD` as the
/// memoized HEAD commit. When the memo does not know HEAD yet, the batch resolves it once and the
/// answer is remembered as `rev-parse --verify HEAD`, so every later read in the scope, batched or
/// not, sees the same commit (REQ-git-utils-010).
#[cfg(unix)]
fn batched_revision(
    root: &Path,
    flavor: &'static str,
    command: impl FnOnce() -> Command,
    revision: &str,
) -> Option<String> {
    if revision != "HEAD" {
        return Some(revision.to_string());
    }
    let verify_head = ["rev-parse", "--verify", "HEAD"];
    let known = [&verify_head[..], &["rev-parse", "HEAD"][..]]
        .iter()
        .find_map(|args| memoized_repository_fact(root, flavor, args))
        .and_then(|fact| String::from_utf8(fact.stdout).ok())
        .map(|stdout| stdout.trim().to_string())
        .filter(|head| batchable_revision(head) && head != "HEAD");
    let learned = known.is_none();
    let head = with_batch(root, flavor, command, |batch| batch.head(known))?;
    if learned {
        note_repository_git(
            root,
            flavor,
            &verify_head,
            &MemoizedGitOutput {
                status: synthetic_status(0),
                stdout: format!("{head}\n").into_bytes(),
                stderr: Vec::new(),
            },
        );
    }
    Some(head)
}

/// Whether a revision is safe to hand to `cat-file --batch` in place of `ls-tree`/`show`.
fn batchable_revision(revision: &str) -> bool {
    revision == "HEAD"
        || (revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

/// Whether a path names exactly one entry for `ls-tree` and `<rev>:<path>` alike.
fn batchable_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with([':', '/', '-'])
        && !path.ends_with('/')
        && !path.contains(['*', '?', '[', '\\', '\n'])
        && !path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
}

/// The mode and hex object id of the entry called `name` in a raw tree object, if it has one.
fn tree_entry<'tree>(
    tree: &'tree [u8],
    name: &str,
    hash_len: usize,
) -> Option<(&'tree [u8], String)> {
    let mut rest = tree;
    while !rest.is_empty() {
        let space = rest.iter().position(|byte| *byte == b' ')?;
        let nul = space + rest[space..].iter().position(|byte| *byte == 0)?;
        let end = nul + 1 + hash_len;
        let hash = rest.get(nul + 1..end)?;
        if &rest[space + 1..nul] == name.as_bytes() {
            let oid = hash.iter().map(|byte| format!("{byte:02x}")).collect();
            return Some((&rest[..space], oid));
        }
        rest = &rest[end..];
    }
    None
}

#[cfg(unix)]
fn synthetic_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code << 8)
}

/// Answer `args` from the open scope's batch reader when it is a single committed-object read
/// with a known answer shape (REQ-git-utils-010):
///
/// - `show <rev>:<path>` (optionally followed by `--`) of a blob: the blob's bytes, exactly as `git show` prints a blob.
///   A missing object fails with status 128, as `git show` does. A tree or other object type is
///   not answered here, because `git show` formats those differently.
/// - `ls-tree --full-tree -z --name-only <rev> -- <path>`: `<path>\0` when the path exists at
///   `<rev>`, otherwise empty, with status 0 either way, as `ls-tree` reports. Only answered when
///   `<rev>` resolves to a tree; otherwise `ls-tree` fails, so the read falls back to it.
///
/// `<rev>` must be `HEAD` or a full object id, and `<path>` one literal entry. Anything else is
/// `None`, and the caller runs git as before.
#[cfg(unix)]
pub(crate) fn batched_git_read(
    root: &Path,
    flavor: &'static str,
    command: impl Fn() -> Command,
    args: &[&str],
    max_bytes: usize,
) -> Option<MemoizedGitOutput> {
    match args {
        ["show", object] | ["show", object, "--"] => {
            let (revision, path) = object.split_once(':')?;
            if !batchable_revision(revision) || !batchable_path(path) {
                return None;
            }
            let revision = batched_revision(root, flavor, &command, revision)?;
            match batched_object(
                root,
                flavor,
                &command,
                &format!("{revision}:{path}"),
                max_bytes,
            )? {
                BatchedObject::Found { kind, bytes, .. } if kind == "blob" => {
                    Some(MemoizedGitOutput {
                        status: synthetic_status(0),
                        stdout: bytes,
                        stderr: Vec::new(),
                    })
                }
                BatchedObject::Found { .. } => None,
                BatchedObject::Missing => Some(MemoizedGitOutput {
                    status: synthetic_status(128),
                    stdout: Vec::new(),
                    stderr: format!("fatal: invalid object name '{object}'.\n").into_bytes(),
                }),
            }
        }
        [
            "ls-tree",
            "--full-tree",
            "-z",
            "--name-only",
            revision,
            "--",
            path,
        ] => {
            if !batchable_revision(revision) || !batchable_path(path) {
                return None;
            }
            let revision = batched_revision(root, flavor, &command, revision)?;
            let listing = |stdout: Vec<u8>| MemoizedGitOutput {
                status: synthetic_status(0),
                stdout,
                stderr: Vec::new(),
            };
            let BatchedObject::Found {
                oid: root_tree,
                kind,
                bytes: mut tree,
            } = batched_object(
                root,
                flavor,
                &command,
                &format!("{revision}^{{tree}}"),
                usize::MAX,
            )?
            else {
                return None;
            };
            if kind != "tree" {
                return None;
            }
            // Walk the parent trees and look for the entry itself, without loading the object
            // it names: a gitlink or a partial-clone blob is listed by `ls-tree` even though its
            // object is not in this repository.
            let hash_len = root_tree.len() / 2;
            let parts: Vec<&str> = path.split('/').collect();
            for (index, part) in parts.iter().enumerate() {
                let Some((mode, oid)) = tree_entry(&tree, part, hash_len) else {
                    return Some(listing(Vec::new()));
                };
                if index + 1 == parts.len() {
                    return Some(listing(format!("{path}\0").into_bytes()));
                }
                if mode != b"40000" {
                    return Some(listing(Vec::new()));
                }
                match batched_object(root, flavor, &command, &oid, usize::MAX)? {
                    BatchedObject::Found { kind, bytes, .. } if kind == "tree" => tree = bytes,
                    // A tree this clone does not hold: let `ls-tree` answer.
                    _ => return None,
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(not(unix))]
pub(crate) fn batched_git_read(
    _root: &Path,
    _flavor: &'static str,
    _command: impl Fn() -> Command,
    _args: &[&str],
    _max_bytes: usize,
) -> Option<MemoizedGitOutput> {
    None
}

#[cfg(test)]
thread_local! {
    static TEST_BATCH_STARTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_test_batch_start() {
    TEST_BATCH_STARTS.with(|count| count.set(count.get() + 1));
}

/// Run `git args` in `root` through [`git_cmd`], answering repository facts from the open scope.
fn git_cmd_output(root: &Path, args: &[&str]) -> std::io::Result<MemoizedGitOutput> {
    if let Some(memoized) = memoized_repository_fact(root, "git_cmd", args) {
        return Ok(memoized);
    }
    let output = output_after_child_exits(git_cmd(root).args(args))?;
    let output = MemoizedGitOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    };
    note_repository_git(root, "git_cmd", args, &output);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Run a git command in `root`, asserting it succeeds.
    fn git(root: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .expect("git command should run");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Initialize an empty git repo with a deterministic identity.
    fn init_repo() -> TempDir {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        git(root, &["init"]);
        git(root, &["config", "user.email", "test@test.com"]);
        git(root, &["config", "user.name", "Test"]);
        git(root, &["config", "commit.gpgsign", "false"]);
        tmp
    }

    /// Write a file (creating parent dirs), stage everything, and commit.
    fn commit_file(root: &Path, rel: &str, contents: &str, message: &str) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        git(root, &["add", "-A"]);
        git(root, &["commit", "-m", message]);
    }

    /// The commit a tracked spec resolves to, for tests that only need the hash.
    fn commit_of(root: &Path, spec: &str) -> String {
        match spec_baseline(root, spec) {
            SpecBaseline::Commit(hash) => hash,
            other => panic!("{spec} should be tracked, got {other:?}"),
        }
    }

    #[test]
    fn baseline_is_untracked_for_a_file_git_has_no_record_of() {
        let tmp = init_repo();
        // The repo HAS history, so "no commit for this path" genuinely means
        // there is nothing for the spec to be behind.
        commit_file(tmp.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(
            spec_baseline(tmp.path(), "specs/missing.spec.md"),
            SpecBaseline::Untracked
        );
    }

    #[test]
    fn baseline_is_a_commit_for_a_tracked_file() {
        let tmp = init_repo();
        commit_file(tmp.path(), "specs/auth.spec.md", "spec v1", "add spec");

        let hash = match spec_baseline(tmp.path(), "specs/auth.spec.md") {
            SpecBaseline::Commit(hash) => hash,
            other => panic!("tracked file should resolve to a commit, got {other:?}"),
        };
        assert_eq!(
            hash.len(),
            40,
            "expected a full 40-char SHA-1, got {hash:?}"
        );
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// The distinction the whole type exists for (#572): a tree with no
    /// history must NOT resolve to the same value as a tracked-but-uncommitted
    /// spec, because the first means "unknown" and the second means "zero".
    #[test]
    fn baseline_separates_no_history_from_no_commit_for_this_path() {
        let plain = TempDir::new().unwrap();
        assert_eq!(
            spec_baseline(plain.path(), "specs/auth.spec.md"),
            SpecBaseline::Missing(MissingHistory::NotARepository)
        );

        let unborn = init_repo();
        assert_eq!(
            spec_baseline(unborn.path(), "specs/auth.spec.md"),
            SpecBaseline::Missing(MissingHistory::NoCommits)
        );

        let healthy = init_repo();
        commit_file(healthy.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(
            spec_baseline(healthy.path(), "specs/auth.spec.md"),
            SpecBaseline::Untracked
        );
    }

    #[test]
    fn missing_history_reports_usable_history_as_none() {
        let plain = TempDir::new().unwrap();
        assert_eq!(
            missing_history(plain.path()),
            Some(MissingHistory::NotARepository)
        );

        let unborn = init_repo();
        assert_eq!(
            missing_history(unborn.path()),
            Some(MissingHistory::NoCommits)
        );

        let healthy = init_repo();
        commit_file(healthy.path(), "src/auth.rs", "fn login() {}", "add source");
        assert_eq!(missing_history(healthy.path()), None);
    }

    #[test]
    fn commits_since_is_zero_when_source_unchanged_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "src/auth.rs", "fn login() {}", "add source");
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");

        let spec_commit = commit_of(root, "specs/auth.spec.md");
        // Source has not changed since the spec was committed.
        assert_eq!(git_commits_since(root, &spec_commit, "src/auth.rs"), 0);
    }

    #[test]
    fn commits_since_counts_source_changes_after_spec() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "specs/auth.spec.md", "spec v1", "add spec");
        let spec_commit = commit_of(root, "specs/auth.spec.md");

        // Three commits touch the source file after the spec was last committed.
        commit_file(root, "src/auth.rs", "v1", "change 1");
        commit_file(root, "src/auth.rs", "v2", "change 2");
        commit_file(root, "src/auth.rs", "v3", "change 3");

        assert_eq!(git_commits_since(root, &spec_commit, "src/auth.rs"), 3);
        // A different, untouched file reports zero.
        assert_eq!(git_commits_since(root, &spec_commit, "src/other.rs"), 0);
    }

    #[test]
    fn commits_since_returns_zero_for_invalid_commit() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "src/auth.rs", "v1", "add source");
        // A bogus commit ref makes `git rev-list` fail; we degrade to zero.
        assert_eq!(
            git_commits_since(
                root,
                "0000000000000000000000000000000000000000",
                "src/auth.rs"
            ),
            0
        );
    }

    #[test]
    fn is_git_repo_detects_repo_and_non_repo() {
        let repo = init_repo();
        assert!(is_git_repo(repo.path()));

        let plain = TempDir::new().unwrap();
        assert!(!is_git_repo(plain.path()));
    }

    #[test]
    fn git_inherited_env_excludes_secrets_and_git_overrides() {
        for denied in [
            "GITHUB_TOKEN",
            "GH_TOKEN",
            "GIT_DIR",
            "GIT_ASKPASS",
            "GIT_AUTHOR_NAME",
            "GIT_AUTHOR_EMAIL",
            "GIT_INDEX_FILE",
            "SSH_AUTH_SOCK",
            "GIT_CEILING_DIRECTORIES",
        ] {
            assert!(
                !GIT_INHERITED_ENV.contains(&denied),
                "{denied} must not be forwarded to git children"
            );
        }
        assert!(
            GIT_INHERITED_ENV.contains(&"PATH"),
            "git must still be able to resolve from PATH"
        );
    }

    #[test]
    fn git_cmd_does_not_forward_sentinel_secrets() {
        let cmd = git_cmd(Path::new("."));
        let keys: Vec<String> = cmd
            .get_envs()
            .filter_map(|(key, value)| value.map(|_| key.to_string_lossy().into_owned()))
            .collect();
        for denied in [
            "GITHUB_TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            "SPECSYNC_TEST_SECRET",
            "GH_TOKEN",
            "GIT_DIR",
            "GIT_ASKPASS",
            "GIT_INDEX_FILE",
            "SSH_AUTH_SOCK",
        ] {
            assert!(
                !keys.iter().any(|k| k == denied),
                "{denied} must not be forwarded to git children; env was {keys:?}"
            );
        }
        assert!(
            keys.iter().any(|k| k == "PATH"),
            "git must still be able to resolve from PATH"
        );
    }

    #[test]
    fn is_git_repo_detects_project_inside_repository_subdirectory() {
        let host = init_repo();
        commit_file(host.path(), "src/auth.rs", "fn login() {}", "add source");
        let nested = host.path().join("packages/foo");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("README"), "pkg\n").unwrap();
        assert!(
            is_git_repo(&nested),
            "a specsync project whose root is a subdirectory of the repository must still see that work tree"
        );
        assert!(is_git_repo(host.path()));
        let plain = TempDir::new().unwrap();
        assert!(!is_git_repo(plain.path()));
    }

    #[test]
    fn discovery_ceiling_isolates_a_subdirectory_from_host_walk_up() {
        let host = init_repo();
        commit_file(host.path(), "src/auth.rs", "fn login() {}", "add source");
        let snap = host.path().join("tmp/snap");
        fs::create_dir_all(&snap).unwrap();
        assert!(
            is_git_repo(&snap),
            "without a ceiling, a tempfile inside the host worktree discovers that work tree"
        );
        let isolated = with_discovery_ceiling(&host.path().join("tmp"), || is_git_repo(&snap));
        assert!(
            !isolated,
            "ceiling=parent(snapshot) must stop walk-up into the host"
        );
        let nested = host.path().join("packages/foo");
        fs::create_dir_all(&nested).unwrap();
        assert!(
            is_git_repo(&nested),
            "nested-project walk-up must still work outside the closure"
        );
    }

    #[test]
    fn discovery_ceiling_does_not_leak_after_the_closure() {
        let host = init_repo();
        commit_file(host.path(), "src/auth.rs", "fn login() {}", "add source");
        let snap = host.path().join("tmp/snap");
        fs::create_dir_all(&snap).unwrap();
        let isolated = with_discovery_ceiling(&host.path().join("tmp"), || is_git_repo(&snap));
        assert!(!isolated);
        assert!(
            is_git_repo(&snap),
            "after the closure, default git_cmd is ceiling-free again"
        );
    }

    // ─── REQ-git-utils-009: repository facts ────────────────────────────

    fn head_through_memo(root: &Path) -> String {
        let output = git_cmd_output(root, &["rev-parse", "--verify", "HEAD"]).unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    /// #887 (REQ-git-utils-009): a failed git query is not memoized, so the next read asks git
    /// again. A definite "absent" answer is a fact, not a failure: `git config` exits 1 for an
    /// unset key, and `symbolic-ref -q` exits 1 for a missing ref.
    #[test]
    fn a_failed_fact_is_not_memoized_but_a_definite_absence_is() {
        let tmp = init_repo();
        let root = tmp.path();
        let _scope = begin_repository_fact_scope(root);
        // No commit yet: `rev-parse --verify HEAD` fails with 128.
        let failed = ["rev-parse", "--verify", "HEAD"];
        let output = git_cmd_output(root, &failed).unwrap();
        assert_eq!(output.status.code(), Some(128));
        assert!(
            memoized_repository_fact(root, "git_cmd", &failed).is_none(),
            "a failed query was memoized"
        );
        let unset = ["config", "--get", "core.fsmonitor"];
        assert_eq!(git_cmd_output(root, &unset).unwrap().status.code(), Some(1));
        assert!(memoized_repository_fact(root, "git_cmd", &unset).is_some());
        let absent = ["symbolic-ref", "-q", "--short", "refs/remotes/origin/HEAD"];
        assert_eq!(
            git_cmd_output(root, &absent).unwrap().status.code(),
            Some(1)
        );
        assert!(memoized_repository_fact(root, "git_cmd", &absent).is_some());
    }

    /// #887: the batch reader's "missing" for `show <rev>:<path>` is cat-file's own answer, and
    /// Git's "path … does not exist in …" names the absent path: both are definite. A process
    /// `git show <rev>:<path> --` that fails says only "bad revision", the same for a missing path
    /// and a missing commit, so it stays a failure and is asked again.
    #[cfg(unix)]
    #[test]
    fn a_missing_path_answer_is_definite_but_a_bad_revision_is_not() {
        let output = |code: i32, stderr: &str| MemoizedGitOutput {
            status: synthetic_status(code),
            stdout: Vec::new(),
            stderr: stderr.as_bytes().to_vec(),
        };
        let show = ["show", "HEAD:missing.txt", "--"];
        assert!(is_definite_absence(
            &show,
            &output(128, "fatal: invalid object name 'HEAD:missing.txt'.\n")
        ));
        assert!(is_definite_absence(
            &show,
            &output(128, "fatal: path 'missing.txt' does not exist in 'HEAD'\n")
        ));
        assert!(!is_definite_absence(
            &show,
            &output(128, "fatal: bad revision 'HEAD:missing.txt'\n")
        ));
        assert!(!is_definite_absence(
            &["rev-parse", "--verify", "HEAD"],
            &output(128, "fatal: Needed a single revision\n")
        ));
    }

    #[test]
    fn a_fact_read_twice_in_one_scope_is_answered_once() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "a", "first");
        let _scope = begin_repository_fact_scope(root);
        let args = ["rev-parse", "--verify", "HEAD"];
        assert!(memoized_repository_fact(root, "git_cmd", &args).is_none());
        let first = head_through_memo(root);
        let memoized = memoized_repository_fact(root, "git_cmd", &args)
            .expect("the first read is remembered for the rest of the scope");
        assert_eq!(String::from_utf8(memoized.stdout).unwrap().trim(), first);
        // A commit made outside SpecSync during an open scope is not seen until a write by the
        // command itself or the scope ends; this is the command-lifetime contract.
        assert_eq!(head_through_memo(root), first);
    }

    #[test]
    fn a_write_inside_the_scope_forgets_every_fact() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "a", "first");
        let _scope = begin_repository_fact_scope(root);
        let before = head_through_memo(root);
        assert!(is_git_repo(root));
        git_cmd_output(root, &["commit", "--allow-empty", "-m", "second"]).unwrap();
        assert!(
            memoized_repository_fact(root, "git_cmd", &["rev-parse", "--verify", "HEAD"]).is_none(),
            "a commit by the command itself clears the memo"
        );
        let after = head_through_memo(root);
        assert_ne!(before, after, "the read after the commit asks git again");
        for write in [
            &["update-ref", "refs/heads/other", "HEAD"][..],
            &["switch", "-c", "elsewhere"][..],
            &["-c", "core.autocrlf=false", "add", "-A"][..],
            &["config", "core.autocrlf", "false"][..],
        ] {
            head_through_memo(root);
            git_cmd_output(root, write).unwrap();
            assert!(
                memoized_repository_fact(root, "git_cmd", &["rev-parse", "--verify", "HEAD"])
                    .is_none(),
                "`git {write:?}` must clear the memo"
            );
        }
        head_through_memo(root);
        invalidate_repository_facts();
        assert!(
            memoized_repository_fact(root, "git_cmd", &["rev-parse", "--verify", "HEAD"]).is_none(),
            "an external process (verification command) clears the memo"
        );
    }

    #[test]
    fn nothing_is_memoized_without_an_open_scope_or_after_it_closes() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "a", "first");
        let args = ["rev-parse", "--verify", "HEAD"];
        head_through_memo(root);
        assert!(memoized_repository_fact(root, "git_cmd", &args).is_none());
        {
            let _scope = begin_repository_fact_scope(root);
            head_through_memo(root);
            assert!(memoized_repository_fact(root, "git_cmd", &args).is_some());
        }
        assert!(memoized_repository_fact(root, "git_cmd", &args).is_none());
        let other = init_repo();
        let _scope = begin_repository_fact_scope(other.path());
        head_through_memo(root);
        assert!(
            memoized_repository_fact(root, "git_cmd", &args).is_none(),
            "a scope for another root does not memoize this one"
        );
    }

    #[test]
    fn working_tree_reads_and_stdin_queries_are_never_facts() {
        for args in [
            &["ls-files", "-z"][..],
            &["status", "--porcelain"][..],
            &["diff-files", "--name-only"][..],
            &["check-attr", "-z", "--stdin", "filter"][..],
            &["cat-file", "--batch"][..],
            &["rev-parse", "--stdin"][..],
            &["config", "core.autocrlf", "false"][..],
            &["symbolic-ref", "HEAD", "refs/heads/x"][..],
        ] {
            assert!(!is_repository_fact(args), "{args:?} must not be memoized");
        }
        for args in [
            &["rev-parse", "--show-prefix"][..],
            &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"][..],
            &["config", "--get", "core.fsmonitor"][..],
            &["log", "--format=%H", "HEAD"][..],
        ] {
            assert!(is_repository_fact(args), "{args:?} is a repository fact");
        }
        assert!(!is_read_only_git(&["-c", "a=b", "commit", "-m", "x"]));
        assert!(is_read_only_git(&[
            "-c",
            "a=b",
            "diff-files",
            "--name-only"
        ]));
        assert!(!is_read_only_git(&["worktree", "add", "x"]));
        assert!(!is_read_only_git(&["hash-object", "-w", "f"]));
    }

    // ─── REQ-git-utils-010: batched object reads ────────────────────────

    fn per_file(root: &Path, args: &[&str]) -> MemoizedGitOutput {
        let output = git_cmd(root).args(args).output().unwrap();
        MemoizedGitOutput {
            status: output.status,
            stdout: output.stdout,
            stderr: output.stderr,
        }
    }

    #[cfg(unix)]
    #[test]
    fn batched_reads_equal_per_file_reads_from_one_child() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "dir/a.txt", "alpha\n", "first");
        fs::write(root.join("empty.txt"), "").unwrap();
        fs::write(root.join("binary.bin"), [0_u8, 159, 146, 150, 0, 10, 13]).unwrap();
        commit_file(root, "dir/b.txt", "beta", "second");
        let head = head_through_memo(root);
        TEST_BATCH_STARTS.with(|count| count.set(0));
        let _scope = begin_repository_fact_scope(root);
        let mut reads = Vec::new();
        for revision in ["HEAD", head.as_str()] {
            for path in [
                "dir/a.txt",
                "dir/b.txt",
                "empty.txt",
                "binary.bin",
                "missing.txt",
            ] {
                reads.push(vec!["show".to_string(), format!("{revision}:{path}")]);
                reads.push(vec![
                    "show".to_string(),
                    format!("{revision}:{path}"),
                    "--".to_string(),
                ]);
                for listed in [path, "dir"] {
                    reads.push(
                        [
                            "ls-tree",
                            "--full-tree",
                            "-z",
                            "--name-only",
                            revision,
                            "--",
                            listed,
                        ]
                        .map(str::to_string)
                        .to_vec(),
                    );
                }
            }
        }
        for read in &reads {
            let args: Vec<&str> = read.iter().map(String::as_str).collect();
            let batched = batched_git_read(root, "git_cmd", || git_cmd(root), &args, usize::MAX)
                .unwrap_or_else(|| panic!("{args:?} should be answered by the batch"));
            let expected = per_file(root, &args);
            assert_eq!(batched.stdout, expected.stdout, "{args:?} stdout");
            assert_eq!(
                batched.status.success(),
                expected.status.success(),
                "{args:?} status"
            );
            assert_eq!(
                batched.status.code(),
                expected.status.code(),
                "{args:?} code"
            );
        }
        assert_eq!(
            TEST_BATCH_STARTS.with(std::cell::Cell::get),
            1,
            "{} reads share one cat-file --batch child",
            reads.len()
        );
        // A tree shown with `git show` is formatted, not raw, so it is left to `git show`.
        assert!(
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", "HEAD:dir"],
                usize::MAX
            )
            .is_none()
        );
        // Unknown shapes, abbreviated revisions and pathspec magic are not batched.
        for args in [
            &["show", "HEAD~1:dir/a.txt"][..],
            &["show", "HEAD:dir/"][..],
            &["show", "HEAD:*.txt"][..],
            &["show", "HEAD"][..],
            &[
                "ls-tree",
                "--full-tree",
                "-z",
                "--name-only",
                "HEAD",
                "--",
                "dir/",
            ][..],
        ] {
            assert!(
                batched_git_read(root, "git_cmd", || git_cmd(root), args, usize::MAX).is_none(),
                "{args:?} must fall back to git"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_batched_head_read_after_a_commit_sees_the_new_commit() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "one", "first");
        let _scope = begin_repository_fact_scope(root);
        let read = |root: &Path| {
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", "HEAD:a.txt"],
                usize::MAX,
            )
            .unwrap()
            .stdout
        };
        assert_eq!(read(root), b"one");
        fs::write(root.join("a.txt"), "two").unwrap();
        git_cmd_output(root, &["commit", "-am", "second"]).unwrap();
        assert_eq!(read(root), b"two");
    }

    #[cfg(unix)]
    #[test]
    fn an_object_over_the_bound_falls_back_to_git() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "big.txt", &"x".repeat(4096), "big");
        let _scope = begin_repository_fact_scope(root);
        assert!(
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", "HEAD:big.txt"],
                1024
            )
            .is_none()
        );
        assert_eq!(
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", "HEAD:big.txt"],
                8192
            )
            .unwrap()
            .stdout
            .len(),
            4096
        );
    }

    #[cfg(unix)]
    #[test]
    fn batched_head_reads_resolve_head_through_the_memo() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "one", "first");
        let read = |root: &Path, path: &str| {
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", &format!("HEAD:{path}")],
                usize::MAX,
            )
            .unwrap()
            .stdout
        };
        {
            // HEAD read first, then a batched read: the batch reads at the memoized HEAD.
            let _scope = begin_repository_fact_scope(root);
            let head = head_through_memo(root);
            fs::write(root.join("a.txt"), "two").unwrap();
            git(root, &["commit", "-am", "elsewhere"]);
            assert_eq!(read(root, "a.txt"), b"one");
            assert_eq!(head_through_memo(root), head);
        }
        {
            // Batched read first: it pins HEAD for the rest of the scope, memo included.
            let _scope = begin_repository_fact_scope(root);
            assert_eq!(read(root, "a.txt"), b"two");
            let head = head_through_memo(root);
            fs::write(root.join("a.txt"), "three").unwrap();
            git(root, &["commit", "-am", "elsewhere again"]);
            assert_eq!(read(root, "a.txt"), b"two");
            assert_eq!(head_through_memo(root), head);
        }
    }

    /// A stand-in for `git cat-file --batch` that runs `script` in `sh`.
    fn fake_batch(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn an_oversized_object_is_refused_from_its_header() {
        // The header claims more than the bound and the payload never comes: the read must
        // refuse from the header instead of waiting for, or allocating, the payload.
        let mut batch = ObjectBatch::start_with_deadline(
            fake_batch("read line; echo \"$line blob 4096\"; exec sleep 30"),
            std::time::Duration::from_secs(60),
        )
        .unwrap();
        let started = std::time::Instant::now();
        assert!(batch.read("HEAD:big.txt", 16).is_err());
        assert!(started.elapsed() < std::time::Duration::from_secs(10));
    }

    #[test]
    fn a_stalled_batch_read_is_killed_at_its_deadline() {
        let mut batch = ObjectBatch::start_with_deadline(
            fake_batch("exec sleep 30"),
            std::time::Duration::from_millis(300),
        )
        .unwrap();
        let started = std::time::Instant::now();
        assert!(batch.read("HEAD:a.txt", usize::MAX).is_err());
        assert!(
            started.elapsed() < std::time::Duration::from_secs(10),
            "the read gave up after {:?}",
            started.elapsed()
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_batched_listing_finds_an_entry_whose_object_is_not_local() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "dir/a.txt", "a", "first");
        // A gitlink names a commit that lives in another repository.
        let absent = "1234567890abcdef1234567890abcdef12345678";
        for path in ["sub", "dir/nested-sub"] {
            git(
                root,
                &[
                    "update-index",
                    "--add",
                    "--cacheinfo",
                    &format!("160000,{absent},{path}"),
                ],
            );
        }
        git(root, &["commit", "-m", "gitlinks"]);
        let _scope = begin_repository_fact_scope(root);
        for path in [
            "sub",
            "dir/nested-sub",
            "dir/a.txt",
            "dir",
            "missing",
            "dir/missing",
            "sub/inside",
            "dir/a.txt/inside",
        ] {
            let args = [
                "ls-tree",
                "--full-tree",
                "-z",
                "--name-only",
                "HEAD",
                "--",
                path,
            ];
            let expected = per_file(root, &args);
            assert!(expected.status.success(), "{path}");
            let batched = batched_git_read(root, "git_cmd", || git_cmd(root), &args, usize::MAX)
                .unwrap_or_else(|| panic!("{path} should be answered by the batch"));
            assert_eq!(batched.stdout, expected.stdout, "{path}");
            assert_eq!(batched.status.code(), expected.status.code(), "{path}");
        }
    }

    #[test]
    fn scoped_config_lookups_are_read_only_facts() {
        for args in [
            &["config", "--local", "--get", "filter.lfs.process"][..],
            &["config", "--local", "-z", "--get-regexp", "^filter"][..],
            &["config", "--get-all", "remote.origin.url"][..],
        ] {
            assert!(is_read_only_git(args), "{args:?} only reads");
            assert!(is_repository_fact(args), "{args:?} is a repository fact");
        }
        for args in [
            &["config", "--local", "filter.lfs.process", "x"][..],
            &["config", "--local", "--unset", "filter.lfs.process"][..],
            &["config", "--get"][..],
        ] {
            assert!(!is_read_only_git(args), "{args:?} may write");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_nested_scope_reuses_the_outer_batch_reader() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "one", "first");
        let read = |root: &Path| {
            batched_git_read(
                root,
                "git_cmd",
                || git_cmd(root),
                &["show", "HEAD:a.txt"],
                usize::MAX,
            )
            .unwrap()
            .stdout
        };
        TEST_BATCH_STARTS.with(|count| count.set(0));
        {
            // Outer read, nested read, outer read again: one reader.
            let _outer = begin_repository_fact_scope(root);
            assert_eq!(read(root), b"one");
            {
                let _nested = begin_repository_fact_scope(root);
                assert_eq!(read(root), b"one");
            }
            assert_eq!(read(root), b"one");
        }
        assert_eq!(TEST_BATCH_STARTS.with(std::cell::Cell::get), 1);
        TEST_BATCH_STARTS.with(|count| count.set(0));
        {
            // First read inside a nested scope: the outer scope keeps that reader.
            let _outer = begin_repository_fact_scope(root);
            {
                let _nested = begin_repository_fact_scope(root);
                assert_eq!(read(root), b"one");
            }
            assert_eq!(read(root), b"one");
        }
        assert_eq!(TEST_BATCH_STARTS.with(std::cell::Cell::get), 1);
    }

    #[test]
    fn a_seeded_worker_scope_starts_from_the_parents_facts() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "one", "first");
        let _scope = begin_repository_fact_scope(root);
        let head = head_through_memo(root);
        let seed = repository_fact_seed(root).expect("an open scope gives a seed");
        // A commit by another process after seeding: the worker answers from the seed.
        git(root, &["commit", "--allow-empty", "-m", "elsewhere"]);
        let root_path = root.to_path_buf();
        let (seen, memoized, batched) = std::thread::spawn(move || {
            let _facts = begin_seeded_repository_fact_scope(seed);
            let memoized =
                memoized_repository_fact(&root_path, "git_cmd", &["rev-parse", "--verify", "HEAD"])
                    .is_some();
            let seen = head_through_memo(&root_path);
            #[cfg(unix)]
            let batched = batched_git_read(
                &root_path,
                "git_cmd",
                || git_cmd(&root_path),
                &["show", "HEAD:a.txt"],
                usize::MAX,
            )
            .is_some();
            #[cfg(not(unix))]
            let batched = false;
            (seen, memoized, batched)
        })
        .join()
        .unwrap();
        assert!(memoized, "the parent's fact is in the worker's scope");
        assert_eq!(seen, head);
        assert!(
            !batched,
            "a seeded worker scope never starts a batch reader"
        );
    }

    #[test]
    fn no_seed_without_a_scope_or_while_reads_are_live() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "one", "first");
        assert!(repository_fact_seed(root).is_none());
        let _scope = begin_repository_fact_scope(root);
        head_through_memo(root);
        {
            let _live = live_repository_reads();
            assert!(repository_fact_seed(root).is_none());
        }
        let other = init_repo();
        assert!(repository_fact_seed(other.path()).is_none());
        assert!(repository_fact_seed(root).is_some());
    }

    #[test]
    fn live_reads_ask_git_even_inside_a_scope() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "a", "first");
        let _scope = begin_repository_fact_scope(root);
        let before = head_through_memo(root);
        // Another process moves HEAD during the command.
        git(root, &["commit", "--allow-empty", "-m", "elsewhere"]);
        assert_eq!(
            head_through_memo(root),
            before,
            "an ordinary read is answered from the scope"
        );
        {
            let _live = live_repository_reads();
            assert!(
                memoized_repository_fact(root, "git_cmd", &["rev-parse", "--verify", "HEAD"])
                    .is_none()
            );
            assert_ne!(
                head_through_memo(root),
                before,
                "a race check sees the move"
            );
            #[cfg(unix)]
            assert!(
                batched_git_read(
                    root,
                    "git_cmd",
                    || git_cmd(root),
                    &["show", "HEAD:a.txt"],
                    usize::MAX
                )
                .is_none()
            );
        }
    }

    /// Same shape as the bounded Git reader: a background shell inherits the
    /// child's stdout and blocks on a fifo. The read must return the child's
    /// bytes while that holder is still alive.
    #[cfg(unix)]
    #[test]
    fn output_returns_while_a_descendant_holds_the_pipe() {
        use std::os::unix::ffi::OsStrExt;

        let temp = TempDir::new().unwrap();
        let fifo = temp.path().join("hold");
        let pidfile = temp.path().join("holder");
        let fifo_c = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) }, 0);
        let script = temp.path().join("child.sh");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\n(\n  trap '' HUP\n  read _ < '{}'\n) &\necho $! > '{}'\nprintf 'payload\\n'\nexit 0\n",
                fifo.display(),
                pidfile.display()
            ),
        )
        .unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let script_for_child = script.clone();
        std::thread::spawn(move || {
            // Through `sh`, not exec of the fresh file: see #885 (ETXTBSY).
            let mut command = Command::new("sh");
            command.arg(script_for_child);
            let _ = tx.send(output_after_child_exits(&mut command));
        });
        let result = rx.recv_timeout(std::time::Duration::from_secs(15));
        let holder = fs::read_to_string(&pidfile)
            .ok()
            .and_then(|text| text.trim().parse::<i32>().ok());
        let holder_alive = holder.is_some_and(|pid| unsafe { libc::kill(pid, 0) } == 0);
        if let Some(pid) = holder {
            unsafe { libc::kill(pid, libc::SIGKILL) };
        }
        let output = result
            .unwrap_or_else(|_| {
                panic!(
                    "still waiting on a descendant's stdout after the child exited (holder alive: {holder_alive})"
                )
            })
            .expect("child should run");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"payload\n");
        assert!(
            holder_alive,
            "the descendant must still be alive when the read returns"
        );
    }

    #[cfg(unix)]
    #[test]
    fn status_does_not_hand_the_caller_stdout_to_the_child() {
        let temp = TempDir::new().unwrap();
        let marker = temp.path().join("stdout-target");
        let script = temp.path().join("child.sh");
        fs::write(
            &script,
            format!(
                "#!/bin/sh\nexec 3>&1\nif [ /dev/fd/3 -ef /dev/null ]; then echo /dev/null; else echo other; fi > '{}'\n",
                marker.display()
            ),
        )
        .unwrap();
        // Run the script through `sh` instead of exec'ing the file just written:
        // another test thread forking while that write was open would make the
        // exec fail with ETXTBSY (#885).
        let mut command = Command::new("sh");
        command.arg(&script);
        let status = status_without_inherited_output(&mut command).unwrap();
        assert!(status.success());
        assert_eq!(
            fs::read_to_string(&marker).unwrap().trim(),
            "/dev/null",
            "a status child must not inherit the caller's stdout"
        );
    }

    /// A piped stdin on the command must not survive. `Command::output` replaces
    /// stdin with the null device. `test -ef` compares device and inode, so the
    /// check also holds on macOS, where `/dev/fd/N` is not a symlink to read.
    #[cfg(unix)]
    #[test]
    fn output_after_child_exits_nulls_stdin() {
        use std::process::Stdio;

        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("if [ /dev/fd/0 -ef /dev/null ]; then echo /dev/null; else echo other; fi")
            .stdin(Stdio::piped());
        let output = output_after_child_exits(&mut command).unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "/dev/null",
            "stdin must be the null device, not the caller's pipe"
        );
    }

    /// The pipe is full and a descendant is blocked in `write` before the read
    /// starts. Stop is already set. An unbounded drain unblocks that write and
    /// follows the flood forever. A finite snapshot keeps the payload at the
    /// head of the pipe and stops at the pipe's capacity.
    #[cfg(target_os = "linux")]
    #[test]
    fn stop_keeps_queued_bytes_and_does_not_follow_a_descendant_flood() {
        use std::fs::File;
        use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd};

        let mut fds = [0; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        // SAFETY: `pipe` just opened both ends and this test owns them.
        let read = unsafe { OwnedFd::from_raw_fd(fds[0]) };
        let write = unsafe { OwnedFd::from_raw_fd(fds[1]) };
        let capacity = pipe_capacity(read.as_raw_fd());
        let payload = b"payload\n";
        assert_eq!(
            unsafe { libc::write(write.as_raw_fd(), payload.as_ptr().cast(), payload.len()) },
            payload.len() as isize
        );
        fill_pipe_until_full(write.as_raw_fd());

        let (tid_tx, tid_rx) = std::sync::mpsc::channel();
        let flood = std::thread::spawn(move || {
            // SAFETY: `gettid` takes no pointer and returns this thread's id.
            let tid = unsafe { libc::syscall(libc::SYS_gettid) };
            tid_tx.send(tid).unwrap();
            let chunk = [b'x'; 8192];
            loop {
                // SAFETY: `write` is the pipe's write end, owned by this thread.
                let rc =
                    unsafe { libc::write(write.as_raw_fd(), chunk.as_ptr().cast(), chunk.len()) };
                if rc < 0 {
                    break;
                }
            }
        });
        let tid = tid_rx.recv().unwrap();
        while !std::fs::read_to_string(format!("/proc/{tid}/wchan"))
            .unwrap_or_default()
            .contains("pipe")
        {
            std::thread::yield_now();
        }

        let stop = AtomicBool::new(true);
        let wake = OutputWake::new().unwrap();
        let overflow = AtomicBool::new(false);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(read_pipe_until_stop(
                File::from(read),
                &stop,
                wake.raw_read(),
                usize::MAX,
                &overflow,
            ));
        });
        let bytes = rx
            .recv_timeout(std::time::Duration::from_secs(15))
            .unwrap_or_else(|_| panic!("a descendant flood kept the read going after stop was set"))
            .unwrap();
        assert!(
            bytes.starts_with(b"payload\n"),
            "bytes queued at stop were dropped"
        );
        assert!(
            bytes.len() <= capacity,
            "post-stop drain followed the flood: {} bytes, pipe capacity {capacity}",
            bytes.len()
        );
        drop(flood);
    }

    #[cfg(target_os = "linux")]
    fn pipe_capacity(fd: std::os::unix::io::RawFd) -> usize {
        // SAFETY: `fd` is an open pipe. `F_GETPIPE_SZ` returns its capacity.
        let size = unsafe { libc::fcntl(fd, libc::F_GETPIPE_SZ) };
        assert!(size > 0, "F_GETPIPE_SZ failed");
        size as usize
    }

    #[cfg(target_os = "linux")]
    fn fill_pipe_until_full(fd: std::os::unix::io::RawFd) {
        set_nonblocking(fd).unwrap();
        let chunk = [b'x'; 8192];
        loop {
            // SAFETY: `fd` is the pipe write end this test owns.
            let rc = unsafe { libc::write(fd, chunk.as_ptr().cast(), chunk.len()) };
            if rc < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    break;
                }
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                panic!("filling the pipe failed: {error}");
            }
        }
        // SAFETY: `fd` is the same write end. Clearing `O_NONBLOCK` makes the
        // descendant's later `write` block inside the kernel once the pipe is full.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        assert!(flags >= 0);
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK) },
            0
        );
    }
}
