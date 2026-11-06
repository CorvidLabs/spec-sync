use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    let output = git_cmd(root)
        .args(["log", "-1", "--format=%H", "--", file])
        .output()
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
    if let Ok(diff) = git_cmd(root)
        .args(["diff", "--quiet", spec_commit, "--", source_file])
        .status()
        && diff.success()
    {
        return 0;
    }
    let output = match git_cmd(root)
        .args([
            "rev-list",
            "--count",
            &format!("{spec_commit}..HEAD"),
            "--",
            source_file,
        ])
        .output()
    {
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
    git_cmd(root)
        .args(["cat-file", "-e", &format!("{since}:./{path}")])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
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

// ─── Repository facts (REQ-git-utils-008) ────────────────────────────────

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
    /// One `git cat-file --batch` child per git command flavor (REQ-git-utils-009).
    batches: std::collections::HashMap<&'static str, ObjectBatch>,
}

thread_local! {
    static REPOSITORY_FACT_SCOPES: RefCell<Vec<RepositoryFactFrame>> = const {
        RefCell::new(Vec::new())
    };
    static LIVE_REPOSITORY_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// While held, every git read on this thread asks git: nothing is answered from memoized facts
/// or a batch reader. Race checks hold one, because they read the repository twice precisely to
/// see a change made by another process during the command (REQ-git-utils-008).
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
        });
    });
    RepositoryFactScope {
        root: root.to_path_buf(),
    }
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
            | ["symbolic-ref", "-q", _]
            | ["symbolic-ref", _]
            | ["config", "--get", ..]
            | ["config", "--get-regexp", ..]
            | ["config", "-z", "--get-regexp", ..]
    )
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
/// write. Call once per git process that ran to completion. Only a successful answer is
/// remembered: a failed query, or a process that could not be started, timed out or was killed,
/// is asked again next time (REQ-git-utils-008).
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
    if !is_repository_fact(args) || discovery_ceiling_is_set() || !output.status.success() {
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

// ─── Batched object reads (REQ-git-utils-009) ────────────────────────────

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
    /// reader's lifetime uses it (REQ-git-utils-009).
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
        let frame = scopes.iter_mut().rev().find(|frame| frame.root == root)?;
        if !frame.batches.contains_key(flavor) {
            let batch = ObjectBatch::start(command()).ok()?;
            #[cfg(test)]
            record_test_batch_start();
            frame.batches.insert(flavor, batch);
        }
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
/// not, sees the same commit (REQ-git-utils-009).
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

#[cfg(unix)]
fn synthetic_status(code: i32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code << 8)
}

/// Answer `args` from the open scope's batch reader when it is a single committed-object read
/// with a known answer shape (REQ-git-utils-009):
///
/// - `show <rev>:<path>` of a blob: the blob's bytes, exactly as `git show` prints a blob.
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
        ["show", object] => {
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
            match batched_object(
                root,
                flavor,
                &command,
                &format!("{revision}^{{tree}}"),
                usize::MAX,
            )? {
                BatchedObject::Found { .. } => {}
                BatchedObject::Missing => return None,
            }
            let stdout = match batched_object(
                root,
                flavor,
                &command,
                &format!("{revision}:{path}"),
                usize::MAX,
            )? {
                BatchedObject::Found { .. } => format!("{path}\0").into_bytes(),
                BatchedObject::Missing => Vec::new(),
            };
            Some(MemoizedGitOutput {
                status: synthetic_status(0),
                stdout,
                stderr: Vec::new(),
            })
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
    let output = git_cmd(root).args(args).output()?;
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

    // ─── REQ-git-utils-008: repository facts ────────────────────────────

    fn head_through_memo(root: &Path) -> String {
        let output = git_cmd_output(root, &["rev-parse", "--verify", "HEAD"]).unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
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

    // ─── REQ-git-utils-009: batched object reads ────────────────────────

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

    #[test]
    fn a_failed_query_is_never_memoized() {
        let tmp = init_repo();
        let root = tmp.path();
        commit_file(root, "a.txt", "a", "first");
        let _scope = begin_repository_fact_scope(root);
        let args = ["rev-parse", "--verify", "refs/heads/not-there"];
        let output = git_cmd_output(root, &args).unwrap();
        assert!(!output.status.success());
        assert!(
            memoized_repository_fact(root, "git_cmd", &args).is_none(),
            "a failed git query is asked again next time (REQ-git-utils-008)"
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
}
