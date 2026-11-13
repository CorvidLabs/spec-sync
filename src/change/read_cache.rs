// Read-path caches for lifecycle status, list and show.
//
// Two layers. Read-only commands get both through a `ChangeReadScope`; finalize and archive get
// only the persistent, Git-keyed layer through a `HistoryCacheScope` (REQ-change-121). Neither
// command moves `HEAD`, which the per-scope `touched_since` memo relies on.
//
// 1. An invocation-scoped archive index, so resolving one change's workspace no longer parses
//    every archived `state.json` again on each lookup.
// 2. A persistent cache of archived workflow-version history verdicts, stored under the Git
//    common directory so every worktree shares it (never in the working tree, so it is never a
//    project input). An entry is reused only when its content digest still matches the archived
//    `state.json` bytes, the commit it was computed at is an ancestor of the scope's `HEAD`, and
//    no commit between them touched that change's active or archived state paths. Anything else recomputes the verdict live. Only passing verdicts are
//    stored, so a refusal is always re-derived.

// Schema 2 keeps whether an introduced package had an `approvals.json`, not its bytes (#850).
const READ_CACHE_SCHEMA: u32 = 2;
const READ_CACHE_FILE: &str = "specsync/read-cache-v1.json";
/// The most archive-introduction entries the persistent cache keeps; the least recently used go
/// first (#850).
const MAX_CACHED_ARCHIVE_INTRODUCTIONS: usize = 4096;
/// A cache file larger than this is not loaded, so the next write starts the cache fresh (#850).
const MAX_READ_CACHE_FILE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct PersistentReadCache {
    schema: u32,
    specsync_version: String,
    repo_prefix: String,
    /// Archived change id to its cached passing workflow-version history verdict.
    workflow_history: BTreeMap<String, WorkflowHistoryCacheEntry>,
    /// `<commit>\n<archive pathspec>` to the archived packages that commit introduced. A commit
    /// and its parents are immutable, so this needs no invalidation beyond the build identity.
    /// At most `MAX_CACHED_ARCHIVE_INTRODUCTIONS` entries are kept.
    #[serde(default)]
    archive_introductions: BTreeMap<String, CachedArchiveIntroductions>,
    /// Advances on every write; an archive introduction is stamped with it when a command that
    /// writes the cache used that entry, so the least recently used entry is dropped first.
    #[serde(default)]
    clock: u64,
    /// Keys of legacy-baseline introduction proofs that passed (see
    /// `legacy_introductions_cache_key`). A key digests every input of the proof that is not an
    /// immutable commit, so it never names a different verdict. Only passes are stored.
    #[serde(default)]
    legacy_introductions: BTreeSet<String>,
    /// `<cutoff commit>\n<change id>` for each workflow-v1 change shown to have had a workflow-v1
    /// record at that trusted pre-v2 cutoff (REQ-change-136). The cutoff is an immutable commit, so
    /// an entry needs no invalidation beyond the build identity; a refusal is never stored.
    #[serde(default)]
    workflow_v1_at_cutoff: BTreeSet<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct CachedArchiveIntroductions {
    /// The cache clock when a command that wrote the cache last used this entry.
    #[serde(default)]
    last_used: u64,
    packages: Vec<CachedArchiveIntroduction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CachedArchiveIntroduction {
    id: String,
    directory: String,
    /// Whether the introducing commit holds the package's `approvals.json`. Its bytes are read
    /// from that commit when they are compared, never kept here (#850).
    approvals: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct WorkflowHistoryCacheEntry {
    /// SHA-256 of the archived `state.json` bytes the verdict was computed for.
    state_digest: String,
    /// The full `HEAD` commit the verdict was computed at.
    commit: String,
    /// The state path the archive lived at when the verdict was computed.
    archive_state_path: String,
}

#[derive(Debug, Default)]
struct PersistentReadCacheFrame {
    path: Option<PathBuf>,
    loaded: Option<PersistentReadCache>,
    dirty: bool,
    /// Commit to the set of lifecycle state paths touched in `commit..HEAD`, or `None` when that
    /// commit is not an ancestor of `HEAD` (or the walk failed), so its entries are unusable.
    touched_since: BTreeMap<String, Option<BTreeSet<String>>>,
    /// `HEAD` resolved once, to a full commit id, when the frame loaded. Every invalidation walk
    /// and every stamp in this scope uses this id, never the moving symbolic `HEAD`.
    head: Option<String>,
    /// Archive-introduction keys this scope read or stored, stamped as recently used when the
    /// cache is written (#850).
    used_introductions: BTreeSet<String>,
}

fn archive_index_in_scope(root: &Path, id: &str) -> Option<Vec<PathBuf>> {
    let index = read_scope_value(root, |scope| scope.archive_index.clone()).or_else(|| {
        read_scope_value(root, |_| Some(()))?;
        let index = build_archive_index(root);
        update_read_scope(root, |scope| scope.archive_index = Some(index.clone()));
        Some(index)
    })?;
    Some(index.get(id).cloned().unwrap_or_default())
}

/// Every archived workspace keyed by the id its `state.json` declares, exactly as
/// `find_change_dir` matches them (unreadable or unparsable state never matches).
fn build_archive_index(root: &Path) -> BTreeMap<String, Vec<PathBuf>> {
    let mut index: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(root.join(ARCHIVE_PATH)) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let state = entry.path().join("state.json");
            if let Some(record) = fs::read(&state)
                .ok()
                .and_then(|content| serde_json::from_slice::<ChangeRecord>(&content).ok())
            {
                index.entry(record.id).or_default().push(entry.path());
            }
        }
    }
    index
}

/// The package version plus the build fingerprint, so a build that judges history differently
/// never reuses another build's verdicts (REQ-change-134). The version alone named no code: every
/// build of one version shared it, sibling worktrees included, since the cache lives in the Git
/// common directory (#878). The fingerprint covers the bundled validation limits too.
fn read_cache_build_identity() -> String {
    format!(
        "{}+{}",
        env!("CARGO_PKG_VERSION"),
        crate::hash_cache::BUILD_FINGERPRINT
    )
}

/// The cache is disabled whenever the history Git walks is not simply the object graph:
/// a shallow repository (deepening it could reveal history a verdict never saw), replace refs or
/// grafts (they rewrite parents and trees without moving `HEAD`), and a partial clone (a missing
/// blob could make a verdict depend on what happened to be fetched).
fn persistent_read_cache_path(root: &Path) -> Option<PathBuf> {
    let probe = git_output(
        root,
        &["rev-parse", "--is-shallow-repository", "--git-common-dir"],
    )?;
    let mut probe = probe.lines();
    if probe.next() != Some("false") {
        return None;
    }
    let common_dir = PathBuf::from(probe.next()?);
    let common_dir = if common_dir.is_absolute() {
        common_dir
    } else {
        root.join(common_dir)
    };
    // Git reports the main worktree's common directory relative to the root as given, but a
    // linked worktree's as an absolute canonical path; canonicalize so a checkout path that runs
    // through a symlink (macOS `/var` -> `/private/var`) still names the one shared cache file.
    let common_dir = fs::canonicalize(&common_dir).unwrap_or(common_dir);
    if common_dir.join("info").join("grafts").exists() {
        return None;
    }
    let replace_refs = run_git_bounded(
        root,
        &["for-each-ref", "--count=1", "--format=%(refname)", "refs/replace/"],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .ok()?;
    if !replace_refs.status.success() || !replace_refs.stdout.trim_ascii().is_empty() {
        return None;
    }
    let promisor = run_git_bounded(
        root,
        &[
            "config",
            "--get-regexp",
            r"^(extensions\.partialclone|remote\..*\.promisor)$",
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .ok()?;
    // Exit 1 means no such key; any match, or any other failure, keeps the cache off.
    if promisor.status.code() != Some(1) {
        return None;
    }
    // The common directory, not the per-worktree one: worktrees share one object graph, and an
    // entry is only reused after proving it against the reading worktree's own `HEAD`.
    Some(common_dir.join(READ_CACHE_FILE))
}

thread_local! {
    /// History cache frames for mutating commands, which install no `ChangeReadScope`.
    static HISTORY_CACHE_SCOPES: RefCell<Vec<(PathBuf, PersistentReadCacheFrame)>> =
        const { RefCell::new(Vec::new()) };
}

/// Lets a mutating lifecycle command (finalize, archive) reuse the persistent, Git-keyed history
/// verdicts and archive introductions while every other read stays live (REQ-change-121).
///
/// Only the two entries whose validity Git history proves are shared: a workflow-version history
/// verdict reused under the same digest and history rule as a read scope, and the archive
/// packages an immutable commit introduced. No working-tree snapshot is taken, so the command
/// still sees its own writes.
struct HistoryCacheScope {
    root: PathBuf,
}

fn begin_history_cache_scope(root: &Path) -> HistoryCacheScope {
    HISTORY_CACHE_SCOPES.with(|scopes| {
        scopes
            .borrow_mut()
            .push((root.to_path_buf(), PersistentReadCacheFrame::default()));
    });
    IMMUTABLE_GIT_MEMOS.with(|memos| {
        memos
            .borrow_mut()
            .push((root.to_path_buf(), ImmutableGitMemo::default()));
    });
    HistoryCacheScope {
        root: root.to_path_buf(),
    }
}

impl Drop for HistoryCacheScope {
    fn drop(&mut self) {
        IMMUTABLE_GIT_MEMOS.with(|memos| {
            memos.borrow_mut().pop();
        });
        HISTORY_CACHE_SCOPES.with(|scopes| {
            let removed = scopes.borrow_mut().pop();
            debug_assert_eq!(
                removed.as_ref().map(|(root, _)| root.as_path()),
                Some(self.root.as_path())
            );
            if let Some((_, frame)) = removed.as_ref() {
                flush_persistent_read_cache(frame);
            }
        });
    }
}

thread_local! {
    /// Answers to Git queries that name only immutable objects, kept for one history scope.
    static IMMUTABLE_GIT_MEMOS: RefCell<Vec<(PathBuf, ImmutableGitMemo)>> =
        const { RefCell::new(Vec::new()) };
}

/// Inside a history scope, finalize and archive re-ask Git the same questions once per archived
/// change: the bytes of a path at a full commit id, a full commit id's parents, whether one full
/// commit id is an ancestor of another, and the repository prefix. Each answer is a pure function
/// of objects named by full id (or of the checkout location), so it cannot change while the
/// command runs. Queries that name `HEAD`, a branch, or the working tree are never memoized here;
/// the workflow-v2 baseline history verdict is memoized only under the full id `HEAD` resolved to.
#[derive(Default)]
struct ImmutableGitMemo {
    files: BTreeMap<(String, String), Option<Vec<u8>>>,
    texts: BTreeMap<Vec<String>, Result<String, String>>,
    statuses: BTreeMap<Vec<String>, Result<bool, String>>,
    prefix: Option<String>,
    repository_present: bool,
    /// Passing workflow-v2 baseline history verdicts, keyed by everything the verdict reads.
    baseline_history_ok: BTreeSet<BaselineHistoryKey>,
}

/// Resolved head id, baseline path, cutoff commit, and current baseline bytes.
type BaselineHistoryKey = (String, String, Option<String>, Vec<u8>);

fn baseline_history_key(
    head: &str,
    baseline_path: &str,
    cutoff_commit: Option<&str>,
    current_bytes: &[u8],
) -> BaselineHistoryKey {
    (
        head.to_string(),
        baseline_path.to_string(),
        cutoff_commit.map(str::to_string),
        current_bytes.to_vec(),
    )
}

fn memoized_baseline_history_ok(
    root: &Path,
    head: &str,
    baseline_path: &str,
    cutoff_commit: Option<&str>,
    current_bytes: &[u8],
) -> bool {
    if !is_full_object_id(head) {
        return false;
    }
    let key = baseline_history_key(head, baseline_path, cutoff_commit, current_bytes);
    immutable_git_memo(root, |memo| Some(memo.baseline_history_ok.contains(&key))).unwrap_or(false)
}

fn remember_baseline_history_ok(
    root: &Path,
    head: &str,
    baseline_path: &str,
    cutoff_commit: Option<&str>,
    current_bytes: &[u8],
) {
    if !is_full_object_id(head) {
        return;
    }
    let key = baseline_history_key(head, baseline_path, cutoff_commit, current_bytes);
    immutable_git_memo(root, |memo| {
        if memo.baseline_history_ok.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            memo.baseline_history_ok.insert(key);
        }
        Some(())
    });
}

fn memoized_repository_present(root: &Path) -> bool {
    immutable_git_memo(root, |memo| Some(memo.repository_present)).unwrap_or(false)
}

fn remember_repository_present(root: &Path) {
    immutable_git_memo(root, |memo| {
        memo.repository_present = true;
        Some(())
    });
}

fn is_full_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Whether `args` is a query whose answer depends only on objects named by full id.
fn names_only_immutable_objects(args: &[&str]) -> bool {
    match args {
        ["rev-list", "--parents", "-n", "1", commit] => is_full_object_id(commit),
        ["merge-base", "--is-ancestor", ancestor, descendant] => {
            is_full_object_id(ancestor) && is_full_object_id(descendant)
        }
        _ => false,
    }
}

fn immutable_git_memo<Value>(
    root: &Path,
    use_memo: impl FnOnce(&mut ImmutableGitMemo) -> Option<Value>,
) -> Option<Value> {
    IMMUTABLE_GIT_MEMOS.with(|memos| {
        let mut memos = memos.borrow_mut();
        let (memo_root, memo) = memos.last_mut()?;
        if memo_root != root {
            return None;
        }
        use_memo(memo)
    })
}

fn memoized_immutable_file(root: &Path, commit: &str, path: &str) -> Option<Option<Vec<u8>>> {
    if !is_full_object_id(commit) {
        return None;
    }
    let key = (commit.to_string(), path.to_string());
    immutable_git_memo(root, |memo| memo.files.get(&key).cloned())
}

fn remember_immutable_file(root: &Path, commit: &str, path: &str, bytes: &Option<Vec<u8>>) {
    if !is_full_object_id(commit) {
        return;
    }
    immutable_git_memo(root, |memo| {
        if memo.files.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            memo.files
                .insert((commit.to_string(), path.to_string()), bytes.clone());
        }
        Some(())
    });
}

fn memoized_immutable_text(root: &Path, args: &[&str]) -> Option<Result<String, String>> {
    if !names_only_immutable_objects(args) {
        return None;
    }
    let key: Vec<String> = args.iter().map(|argument| (*argument).to_string()).collect();
    immutable_git_memo(root, |memo| memo.texts.get(&key).cloned())
}

fn remember_immutable_text(root: &Path, args: &[&str], result: &Result<String, String>) {
    if !names_only_immutable_objects(args) {
        return;
    }
    let key: Vec<String> = args.iter().map(|argument| (*argument).to_string()).collect();
    immutable_git_memo(root, |memo| {
        if memo.texts.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            memo.texts.insert(key, result.clone());
        }
        Some(())
    });
}

fn memoized_immutable_status(root: &Path, args: &[&str]) -> Option<Result<bool, String>> {
    if !names_only_immutable_objects(args) {
        return None;
    }
    let key: Vec<String> = args.iter().map(|argument| (*argument).to_string()).collect();
    immutable_git_memo(root, |memo| memo.statuses.get(&key).cloned())
}

fn remember_immutable_status(root: &Path, args: &[&str], result: &Result<bool, String>) {
    // A failed run (timeout, spawn error) is not an answer about the objects; ask again.
    if !names_only_immutable_objects(args) || result.is_err() {
        return;
    }
    let key: Vec<String> = args.iter().map(|argument| (*argument).to_string()).collect();
    immutable_git_memo(root, |memo| {
        if memo.statuses.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            memo.statuses.insert(key, result.clone());
        }
        Some(())
    });
}

fn memoized_repo_prefix(root: &Path) -> Option<String> {
    immutable_git_memo(root, |memo| memo.prefix.clone())
}

fn remember_repo_prefix(root: &Path, prefix: &str) {
    immutable_git_memo(root, |memo| {
        memo.prefix = Some(prefix.to_string());
        Some(())
    });
}

fn take_persistent_cache_frame(root: &Path) -> Option<(bool, PersistentReadCacheFrame)> {
    if read_scope_value(root, |_| Some(())).is_some() {
        return CHANGE_READ_SCOPES.with(|scopes| {
            scopes
                .borrow_mut()
                .last_mut()
                .filter(|scope| scope.root == root)
                .map(|scope| (true, std::mem::take(&mut scope.persistent_cache)))
        });
    }
    HISTORY_CACHE_SCOPES.with(|scopes| {
        scopes
            .borrow_mut()
            .last_mut()
            .filter(|(scope_root, _)| scope_root == root)
            .map(|(_, frame)| (false, std::mem::take(frame)))
    })
}

fn return_persistent_cache_frame(root: &Path, in_read_scope: bool, frame: PersistentReadCacheFrame) {
    if in_read_scope {
        CHANGE_READ_SCOPES.with(|scopes| {
            if let Some(scope) = scopes
                .borrow_mut()
                .last_mut()
                .filter(|scope| scope.root == root)
            {
                scope.persistent_cache = frame;
            }
        });
    } else {
        HISTORY_CACHE_SCOPES.with(|scopes| {
            if let Some((_, slot)) = scopes
                .borrow_mut()
                .last_mut()
                .filter(|(scope_root, _)| scope_root == root)
            {
                *slot = frame;
            }
        });
    }
}

fn with_persistent_read_cache<Value>(
    root: &Path,
    use_cache: impl FnOnce(&Path, &mut PersistentReadCacheFrame) -> Value,
) -> Option<Value> {
    let (in_read_scope, mut frame) = take_persistent_cache_frame(root)?;
    if frame.loaded.is_none() {
        frame.path = persistent_read_cache_path(root);
        frame.head = git_output(root, &["rev-parse", "--verify", "HEAD^{commit}"]);
        let prefix = git_repo_prefix(root).unwrap_or_default();
        let fresh = PersistentReadCache {
            schema: READ_CACHE_SCHEMA,
            specsync_version: read_cache_build_identity(),
            repo_prefix: prefix.clone(),
            workflow_history: BTreeMap::new(),
            archive_introductions: BTreeMap::new(),
            clock: 0,
            legacy_introductions: BTreeSet::new(),
            workflow_v1_at_cutoff: BTreeSet::new(),
        };
        let loaded = frame
            .path
            .as_ref()
            .filter(|path| {
                fs::metadata(path).is_ok_and(|metadata| metadata.len() <= MAX_READ_CACHE_FILE_BYTES)
            })
            .and_then(|path| fs::read(path).ok())
            .and_then(|bytes| serde_json::from_slice::<PersistentReadCache>(&bytes).ok())
            .filter(|cache| {
                cache.schema == fresh.schema
                    && cache.specsync_version == fresh.specsync_version
                    && cache.repo_prefix == fresh.repo_prefix
            });
        frame.loaded = Some(loaded.unwrap_or(fresh));
    }
    let value = use_cache(root, &mut frame);
    return_persistent_cache_frame(root, in_read_scope, frame);
    Some(value)
}

/// Lifecycle state paths touched by any commit in `commit..head`, compared against every parent.
/// `head` is a full commit id resolved once per scope, so both Git calls see the same tip.
fn lifecycle_paths_touched_since(root: &Path, commit: &str, head: &str) -> Option<BTreeSet<String>> {
    if !cached_git_status_success(root, &["merge-base", "--is-ancestor", commit, head]).ok()? {
        return None;
    }
    // Repository-relative paths, anchored at the repository root. A project in a subdirectory
    // otherwise makes this invalidation walk match nothing, so a cached pass survives a later
    // edit of the change's own state (#851).
    let changes = format!(":(top,literal){}", git_repo_relative_path(root, CHANGES_PATH).ok()?);
    let archive = format!(":(top,literal){}", git_repo_relative_path(root, ARCHIVE_PATH).ok()?);
    let range = format!("{commit}..{head}");
    let output = scoped_review_git_text(
        root,
        &[
            "log",
            "--full-history",
            "-m",
            "--no-renames",
            "--format=",
            "--name-only",
            "-z",
            range.as_str(),
            "--",
            changes.as_str(),
            archive.as_str(),
        ],
    )
    .ok()?;
    Some(
        output
            .split('\0')
            .map(|path| path.trim_matches('\n'))
            .filter(|path| !path.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// True when `path` is one of the state paths `workflow_version_state_paths` could ever name for
/// `id`: its active state, or `<archive>/<anything without a slash>-<id>/state.json`.
fn path_is_state_candidate(path: &str, changes_root: &str, archive_root: &str, id: &str) -> bool {
    if path == format!("{changes_root}/{id}/state.json") {
        return true;
    }
    let Some(rest) = path.strip_prefix(&format!("{archive_root}/")) else {
        return false;
    };
    let Some(directory) = rest.strip_suffix("/state.json") else {
        return false;
    };
    !directory.contains('/') && directory.ends_with(&format!("-{id}"))
}

/// The digest and repository path of the archived `state.json`, but only when those bytes parse
/// to exactly `record`: a verdict is computed for a record, so it may only be keyed by bytes that
/// say the same thing.
fn archived_state_cache_key(root: &Path, record: &ChangeRecord) -> Option<(String, String)> {
    let workspace = find_change_dir(root, &record.id).ok()?;
    let state = workspace.join("state.json");
    let bytes = fs::read(&state).ok()?;
    if serde_json::from_slice::<ChangeRecord>(&bytes).ok().as_ref() != Some(record) {
        return None;
    }
    let archive_state_path =
        git_repo_relative_path(root, &portable_project_path(root, &state)).ok()?;
    Some((sha256_hex(&bytes), archive_state_path))
}

/// A cached passing workflow-version history verdict that is still provably current.
fn cached_workflow_history_is_current(root: &Path, record: &ChangeRecord) -> bool {
    if record.state != ChangeState::Archived {
        return false;
    }
    let Some((digest, archive_state_path)) = archived_state_cache_key(root, record) else {
        return false;
    };
    let (Ok(changes_root), Ok(archive_root)) = (
        git_repo_relative_path(root, CHANGES_PATH),
        git_repo_relative_path(root, ARCHIVE_PATH),
    ) else {
        return false;
    };
    with_persistent_read_cache(root, |root, frame| {
        if frame.path.is_none() {
            return false;
        }
        let Some(entry) = frame
            .loaded
            .as_ref()
            .and_then(|cache| cache.workflow_history.get(&record.id))
            .cloned()
        else {
            return false;
        };
        if entry.state_digest != digest || entry.archive_state_path != archive_state_path {
            return false;
        }
        let Some(head) = frame.head.clone() else {
            return false;
        };
        let touched = frame
            .touched_since
            .entry(entry.commit.clone())
            .or_insert_with(|| lifecycle_paths_touched_since(root, &entry.commit, &head));
        let current = touched.as_ref().is_some_and(|touched| {
            // The live check also reads the archive's current state path, which need not end
            // in `-<id>` once the directory is renamed; treat it as a candidate too.
            !touched.iter().any(|path| {
                *path == archive_state_path
                    || path_is_state_candidate(path, &changes_root, &archive_root, &record.id)
            })
        });
        // Proven current at `head`: move the entry forward so later reads walk only newer commits.
        if current
            && entry.commit != head
            && let Some(cache) = frame.loaded.as_mut()
        {
            cache.workflow_history.insert(
                record.id.clone(),
                WorkflowHistoryCacheEntry {
                    commit: head,
                    ..entry
                },
            );
            frame.dirty = true;
        }
        current
    })
    .unwrap_or(false)
}

fn remember_workflow_history_pass(root: &Path, record: &ChangeRecord) {
    if record.state != ChangeState::Archived {
        return;
    }
    let Some((digest, archive_state_path)) = archived_state_cache_key(root, record) else {
        return;
    };
    // The verdict was computed against the scope's HEAD; if HEAD has moved since, it is not known
    // to hold at either commit, so record nothing.
    let current = git_output(root, &["rev-parse", "--verify", "HEAD^{commit}"]);
    with_persistent_read_cache(root, |_, frame| {
        let Some(head) = frame.head.clone().filter(|head| current.as_ref() == Some(head)) else {
            return;
        };
        if let Some(cache) = frame.loaded.as_mut() {
            let entry = WorkflowHistoryCacheEntry {
                state_digest: digest,
                commit: head,
                archive_state_path,
            };
            if cache.workflow_history.get(&record.id) != Some(&entry) {
                cache.workflow_history.insert(record.id.clone(), entry);
                frame.dirty = true;
            }
        }
    });
}

const LEGACY_INTRODUCTIONS_CACHE_DOMAIN: &[u8] = b"specsync.read-cache.legacy-introductions.v1";

/// The read-cache key for one proof of a legacy baseline's introductions (#901).
///
/// `validate_legacy_archive_introductions` reads the baseline (bound by its ledger digest), the
/// cutoff commit, the commits reachable from it, the introduction commits and their trees, the
/// repository prefix, and every enumerated archive's current subtree. Commits named by full id
/// and everything reachable from them are immutable while the persistent cache is enabled (it is
/// off for shallow, grafted, replaced and partial clones), the prefix and the build are part of
/// the cache file's identity, and the rest is digested here. `None`, so the proof runs live,
/// when the cutoff is not a full commit id or an archive has no capturable subtree.
fn legacy_introductions_cache_key(
    baseline_digest: &str,
    baseline: &LegacyArchiveBaselineV1,
    snapshots: &BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
) -> Option<String> {
    if !is_full_object_id(&baseline.cutoff_commit) {
        return None;
    }
    let mut digest = FramedDigest::new(LEGACY_INTRODUCTIONS_CACHE_DOMAIN);
    digest.frame(b"baseline", baseline_digest.as_bytes());
    digest.frame(b"cutoff", baseline.cutoff_commit.as_bytes());
    for entry in &baseline.entries {
        let subtree = legacy_archive_subtree_digest(snapshots.get(&entry.id)?).ok()?;
        digest.frame(b"id", entry.id.as_bytes());
        digest.frame(b"subtree", subtree.as_bytes());
    }
    Some(digest.finish())
}

/// Whether the persistent read cache holds a passing proof under `key`.
fn legacy_introductions_proven(root: &Path, key: &str) -> bool {
    with_persistent_read_cache(root, |_, frame| {
        frame.path.is_some()
            && frame
                .loaded
                .as_ref()
                .is_some_and(|cache| cache.legacy_introductions.contains(key))
    })
    .unwrap_or(false)
}

/// Records a passing proof under `key`. Called only after the live proof passed.
fn remember_legacy_introductions_proven(root: &Path, key: String) {
    with_persistent_read_cache(root, |_, frame| {
        if frame.path.is_none() {
            return;
        }
        if let Some(cache) = frame.loaded.as_mut()
            && cache.legacy_introductions.len() < MAX_CHANGE_READ_CACHE_ENTRIES
            && cache.legacy_introductions.insert(key)
        {
            frame.dirty = true;
        }
    });
}

/// Writes the persistent cache when the scope learned something new. Failures are ignored: the
/// cache is an accelerator, and a read-only checkout must still answer.
fn flush_persistent_read_cache(frame: &PersistentReadCacheFrame) {
    if !frame.dirty {
        return;
    }
    let (Some(path), Some(cache)) = (frame.path.as_ref(), frame.loaded.as_ref()) else {
        return;
    };
    let mut cache = cache.clone();
    bound_archive_introductions(&mut cache, &frame.used_introductions);
    let Ok(bytes) = serde_json::to_vec(&cache) else {
        return;
    };
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    // Unique per process and per flush, so concurrent writers (threads or processes) never share
    // a temporary file; the rename stays atomic.
    static FLUSH_NONCE: AtomicU64 = AtomicU64::new(0);
    let temporary = path.with_extension(format!(
        "tmp-{}-{}",
        std::process::id(),
        FLUSH_NONCE.fetch_add(1, Ordering::Relaxed)
    ));
    if fs::write(&temporary, bytes).is_ok() && fs::rename(&temporary, path).is_err() {
        let _ = fs::remove_file(&temporary);
    }
}

/// Stamps the archive introductions this scope used with a new clock value, then drops the least
/// recently used entries beyond `MAX_CACHED_ARCHIVE_INTRODUCTIONS` (#850). Ties go by key.
fn bound_archive_introductions(cache: &mut PersistentReadCache, used: &BTreeSet<String>) {
    cache.clock = cache.clock.saturating_add(1);
    for key in used {
        if let Some(entry) = cache.archive_introductions.get_mut(key) {
            entry.last_used = cache.clock;
        }
    }
    let excess = cache
        .archive_introductions
        .len()
        .saturating_sub(MAX_CACHED_ARCHIVE_INTRODUCTIONS);
    if excess == 0 {
        return;
    }
    let mut by_age: Vec<(u64, String)> = cache
        .archive_introductions
        .iter()
        .map(|(key, entry)| (entry.last_used, key.clone()))
        .collect();
    by_age.sort();
    for (_, key) in by_age.into_iter().take(excess) {
        cache.archive_introductions.remove(&key);
    }
}

/// Whether `commit`'s tree has an entry at the repository-relative `path`, read from tree objects
/// only, so it answers even when the blob itself is unavailable. `None` when Git cannot say.
fn commit_tree_lists_path(root: &Path, commit: &str, path: &str) -> Option<bool> {
    let output = run_git_bounded(
        root,
        &["ls-tree", "--full-tree", "-z", "--name-only", commit, "--", path],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .split('\0')
            .any(|listed| listed == path)
    })
}

/// One archived package a commit introduced: `(id, directory, approvals ledger)`.
type ArchivePackageIntroduction = (String, String, IntroducedApprovals);

/// The archived packages `commit` added under `history_pathspec`, as `(id, directory, approvals)`.
/// Reads the persistent cache inside a read scope, and stores a result only when every Git call
/// behind it answered conclusively.
fn archive_introductions_at_commit(
    root: &Path,
    commit: &str,
    history_pathspec: &str,
    directory_prefix: &str,
    listed: Option<&[String]>,
) -> Result<Vec<ArchivePackageIntroduction>, String> {
    let key = format!("{commit}\n{history_pathspec}");
    if let Some(Some(cached)) = with_persistent_read_cache(root, |_, frame| {
        let cached = frame
            .loaded
            .as_ref()
            .and_then(|cache| cache.archive_introductions.get(&key).cloned());
        if cached.is_some() {
            frame.used_introductions.insert(key.clone());
        }
        cached
    }) {
        return Ok(cached
            .packages
            .into_iter()
            .map(|entry| {
                let approvals = if entry.approvals {
                    IntroducedApprovals::Committed
                } else {
                    IntroducedApprovals::Absent
                };
                (entry.id, entry.directory, approvals)
            })
            .collect());
    }
    let (introduced, conclusive) = archive_introductions_at_commit_uncached(
        root,
        commit,
        history_pathspec,
        directory_prefix,
        listed,
    )?;
    if conclusive {
        let packages = introduced
            .iter()
            .map(|(id, directory, approvals)| CachedArchiveIntroduction {
                id: id.clone(),
                directory: directory.clone(),
                approvals: *approvals != IntroducedApprovals::Absent,
            })
            .collect();
        with_persistent_read_cache(root, |_, frame| {
            if frame.path.is_some()
                && let Some(cache) = frame.loaded.as_mut()
            {
                cache.archive_introductions.insert(
                    key.clone(),
                    CachedArchiveIntroductions {
                        last_used: 0,
                        packages,
                    },
                );
                frame.used_introductions.insert(key);
                frame.dirty = true;
            }
        });
    }
    Ok(introduced)
}

type ArchiveIntroductionsAtCommit = (Vec<ArchivePackageIntroduction>, bool);

fn archive_introductions_at_commit_uncached(
    root: &Path,
    commit: &str,
    history_pathspec: &str,
    directory_prefix: &str,
    listed: Option<&[String]>,
) -> Result<ArchiveIntroductionsAtCommit, String> {
    if let Some(listed) = listed {
        return archive_introductions_from_listing(root, commit, directory_prefix, listed);
    }
    let output = run_git_bounded(
        root,
        &[
            "diff-tree",
            "--root",
            "-m",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-z",
            "--no-renames",
            "--diff-filter=A",
            commit,
            "--",
            history_pathspec,
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .map_err(|error| {
        format!("failed to inspect archived acceptance introduction `{commit}`: {error}")
    })?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect archived acceptance introduction `{commit}`"
        ));
    }
    let names = String::from_utf8_lossy(&output.stdout);
    let listed: Vec<String> = names
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect();
    archive_introductions_from_listing(root, commit, directory_prefix, &listed)
}

/// The archived packages among `listed`, the paths `commit` added under the archive root, with
/// their committed `state.json` and `approvals.json` bytes.
fn archive_introductions_from_listing(
    root: &Path,
    commit: &str,
    directory_prefix: &str,
    listed: &[String],
) -> Result<ArchiveIntroductionsAtCommit, String> {
    let mut conclusive = true;
    let mut introduced = Vec::new();
    for path in listed.iter().map(String::as_str) {
        let Some(directory) = path.strip_suffix("/state.json") else {
            continue;
        };
        let Some(package) = directory.strip_prefix(directory_prefix) else {
            continue;
        };
        if package.is_empty() || package.contains('/') {
            continue;
        }
        let object = format!("{commit}:{path}");
        let Ok(state) = run_git_bounded(
            root,
            &["show", object.as_str(), "--"],
            None,
            MAX_CHANGE_ARTIFACT_BYTES as usize,
        ) else {
            conclusive = false;
            continue;
        };
        if !state.status.success() {
            // diff-tree just listed this path as added, so a failed read means the blob is
            // unavailable (an offline partial clone), not that the archive is absent.
            conclusive = false;
            continue;
        }
        let Ok(archived) = serde_json::from_slice::<ChangeRecord>(&state.stdout) else {
            continue;
        };
        let object = format!("{commit}:{directory}/approvals.json");
        let approvals = match run_git_bounded(
            root,
            &["show", object.as_str(), "--"],
            None,
            MAX_CHANGE_ARTIFACT_BYTES as usize,
        ) {
            Ok(output) if output.status.success() => Some(output.stdout),
            Ok(_) => {
                if commit_tree_lists_path(root, commit, &format!("{directory}/approvals.json"))
                    != Some(false)
                {
                    conclusive = false;
                }
                None
            }
            Err(_) => {
                conclusive = false;
                None
            }
        };
        let approvals = approvals.map_or(IntroducedApprovals::Absent, IntroducedApprovals::Read);
        introduced.push((archived.id, directory.to_string(), approvals));
    }
    Ok((introduced, conclusive))
}

/// Inside a read scope, computes the workflow-version history verdicts that the archive walk is
/// about to need and the persistent cache cannot answer, several at a time, and memoizes them in
/// the scope. The walk still visits every archive in order and returns the same first error; this
/// only moves independent Git reads off the critical path. Outside a read scope it does nothing.
fn prefetch_archived_workflow_histories(root: &Path, entries: &[std::io::Result<fs::DirEntry>]) {
    let in_history_scope = HISTORY_CACHE_SCOPES.with(|scopes| {
        scopes
            .borrow()
            .last()
            .is_some_and(|(scope_root, _)| scope_root == root)
    });
    if read_scope_value(root, |_| Some(())).is_none() && !in_history_scope {
        return;
    }
    let mut pending = Vec::new();
    for entry in entries.iter().flatten() {
        let Ok(content) = fs::read_to_string(entry.path().join("state.json")) else {
            continue;
        };
        let Ok(record) = serde_json::from_str::<ChangeRecord>(&content) else {
            continue;
        };
        if record.state != ChangeState::Archived
            || validate_loaded_change(&record, &record.id, &entry.path().join("state.json")).is_err()
        {
            continue;
        }
        let key = workflow_version_history_key(&record);
        if read_scope_value(root, |scope| scope.workflow_version_history.get(&key).cloned()).is_some()
            || cached_workflow_history_is_current(root, &record)
        {
            continue;
        }
        pending.push((key, record));
    }
    if pending.len() < 2 {
        return;
    }
    // With the command's lifecycle-state history index, each archive's check needs no history
    // walk of its own, and its state-file reads go through this thread's one batch reader. The
    // archive walk then validates them here, in order, with a deadline check between archives
    // (REQ-change-141). Workers have no batch reader, so they would only add processes.
    if lifecycle_state_history(root).is_some() {
        return;
    }
    let workers = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
        .clamp(1, 8)
        .min(pending.len());
    let next = AtomicU64::new(0);
    let results: Mutex<Vec<(usize, Result<(), String>)>> = Mutex::new(Vec::new());
    // Workers only read, so each starts from what this command already knows about the
    // repository and memoizes its own reads (REQ-git-utils-011).
    let seed = crate::git_utils::repository_fact_seed(root);
    let deadline = current_read_deadline();
    let (next_ref, pending_ref, results_ref) = (&next, &pending, &results);
    std::thread::scope(|threads| {
        let workers: Vec<_> = (0..workers)
            .map(|_| {
                let seed = seed.clone();
                threads.spawn(move || {
                    let _facts = seed.map(crate::git_utils::begin_seeded_repository_fact_scope);
                    let _deadline = begin_worker_read_deadline(deadline);
                    loop {
                    if check_read_deadline("archived workflow history", 0, 0).is_err() {
                        break;
                    }
                    let index = next_ref.fetch_add(1, Ordering::Relaxed) as usize;
                    let Some((_, record)) = pending_ref.get(index) else {
                        break;
                    };
                    let result = validate_workflow_version_history_uncached(root, record);
                    if let Ok(mut results) = results_ref.lock() {
                        results.push((index, result));
                    }
                    }
                    worker_test_git_process_count()
                })
            })
            .collect();
        // Fold each worker's Git processes into this thread's test counter, so the Git budget
        // guards see prefetched work. A no-op outside tests.
        for worker in workers {
            if let Ok(spawned) = worker.join() {
                add_test_git_processes(spawned);
            }
        }
    });
    let Ok(results) = results.into_inner() else {
        return;
    };
    let completed = results.len();
    for (index, result) in results {
        let (key, record) = &pending[index];
        if result.is_ok() {
            remember_workflow_history_pass(root, record);
        }
        update_read_scope(root, |scope| {
            scope.workflow_version_history.insert(key.clone(), result);
        });
    }
    // Workers stop at the deadline; what they finished is recorded above, so the report can keep it.
    let _ = check_read_deadline("archived workflow history", completed, pending.len());
}

/// Write every open persistent read cache on this thread, so a read stopped by its deadline keeps
/// the verdicts it completed (REQ-change-137).
fn flush_open_read_caches() {
    HISTORY_CACHE_SCOPES.with(|scopes| {
        if let Ok(scopes) = scopes.try_borrow() {
            for (_, frame) in scopes.iter() {
                flush_persistent_read_cache(frame);
            }
        }
    });
    flush_change_read_scope_caches();
}

/// Whether the persistent read cache already knows that workflow-v1 change `id` had a workflow-v1
/// record at `cutoff` (REQ-change-136).
fn cached_workflow_v1_at_cutoff(root: &Path, cutoff: &str, id: &str) -> bool {
    let key = format!("{cutoff}\n{id}");
    with_persistent_read_cache(root, |_, frame| {
        frame
            .loaded
            .as_ref()
            .is_some_and(|cache| cache.workflow_v1_at_cutoff.contains(&key))
    })
    .unwrap_or(false)
}

/// Remember that workflow-v1 change `id` had a workflow-v1 record at `cutoff`. Only a
/// conclusive presence is stored; an absence stays live so a transient read failure never
/// becomes a lasting refusal.
fn remember_workflow_v1_at_cutoff(root: &Path, cutoff: &str, id: &str) {
    let key = format!("{cutoff}\n{id}");
    with_persistent_read_cache(root, |_, frame| {
        if frame.path.is_some()
            && let Some(cache) = frame.loaded.as_mut()
            && cache.workflow_v1_at_cutoff.insert(key)
        {
            frame.dirty = true;
        }
    });
}

/// The paths each of `commits` added under `history_pathspec`, from one `git diff-tree --stdin`
/// with exactly the flags the per-commit read uses (REQ-change-135). Commits the persistent read
/// cache already answers are skipped. `None` when the walk could not be read; callers then fall
/// back to one read per commit.
fn archive_introduction_listings(
    root: &Path,
    commits: &[&str],
    history_pathspec: &str,
) -> Option<BTreeMap<String, Vec<String>>> {
    let uncached: Vec<&str> = with_persistent_read_cache(root, |_, frame| {
        commits
            .iter()
            .copied()
            .filter(|commit| {
                let key = format!("{commit}\n{history_pathspec}");
                !frame
                    .loaded
                    .as_ref()
                    .is_some_and(|cache| cache.archive_introductions.contains_key(&key))
            })
            .collect()
    })
    .unwrap_or_else(|| commits.to_vec());
    if uncached.len() < 2 {
        return Some(BTreeMap::new());
    }
    let mut input = uncached.join("\n").into_bytes();
    input.push(b'\n');
    let output = run_git_bounded(
        root,
        &[
            "diff-tree",
            "--stdin",
            "--root",
            "-m",
            "--name-only",
            "-r",
            "-z",
            "--no-renames",
            "--diff-filter=A",
            "--",
            history_pathspec,
        ],
        Some(input),
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let wanted: BTreeSet<&str> = uncached.iter().copied().collect();
    let mut listings: BTreeMap<String, Vec<String>> = uncached
        .iter()
        .map(|commit| ((*commit).to_string(), Vec::new()))
        .collect();
    let mut current: Option<String> = None;
    for entry in String::from_utf8_lossy(&output.stdout).split('\0') {
        let entry = entry.trim_matches('\n');
        if entry.is_empty() {
            continue;
        }
        // A header is the commit id; every listed path is under the archive root and has a `/`.
        let token = entry.split_whitespace().next().unwrap_or(entry);
        if !entry.contains('/') && wanted.contains(token) {
            current = Some(token.to_string());
            continue;
        }
        let listing = listings.get_mut(current.as_ref()?)?;
        if !listing.iter().any(|path| path == entry) {
            listing.push(entry.to_string());
        }
    }
    Some(listings)
}
