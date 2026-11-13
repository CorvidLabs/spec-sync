use crate::parser::strip_frontmatter;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::borrow::Cow;
#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const SDD_VERSION: &str = "6.0.0";
const INVALID_CORRECTION_LEDGER_TEXT: &str = "correction ledger integrity is invalid; restore corrections.json from trusted history before inspecting lifecycle status";
const POLICY_PATH: &str = ".specsync/sdd.json";
const CHANGES_PATH: &str = ".specsync/changes";
const ARCHIVE_PATH: &str = ".specsync/archive/changes";
pub(crate) const LESSON_BUNDLE_FILE: &str = "lesson-bundle.md";
const LEGACY_BASELINE_PATH: &str = ".specsync/archive/legacy-baseline.json";
const WORKFLOW_V2_BASELINE_PATH: &str = ".specsync/workflow-v2-baseline.json";
const LOCK_PATH: &str = ".specsync/change.lock";
/// How far back to read the ledger's own history when looking for a
/// downward rewrite. Bounded so `check` stays cheap on a long-lived repo; a
/// regression is detected within this many revisions of the ledger, which is
/// far more than any real branch accumulates before it is noticed.
const SEQUENCE_HISTORY_SCAN_LIMIT: usize = 200;

const SEQUENCE_PATH: &str = ".specsync/change-sequence.json";
const BOOTSTRAP_RECORD_PATH: &str = ".specsync/bootstrap.json";
/// Protected SDD paths `specsync init` creates for a fresh project.
const BOOTSTRAP_RECORD_CANDIDATES: [&str; 4] = [
    ".specsync/config.toml",
    ".specsync/config.json",
    ".specsync/version",
    POLICY_PATH,
];
const TRANSACTION_PATH: &str = ".specsync/change-transaction.json";
const CORRECTIONS_FILE: &str = "corrections.json";
const SCOPED_REVIEW_FILE: &str = "review.json";
const SCOPED_REVIEW_ATTEMPTS_FILE: &str = "review-attempts.json";
/// The anchor label `authenticated_accepted_transition_for` returns for closing evidence it
/// admitted from the working tree. Deliberately not a commit: history holds none for it yet.
const WORKING_TREE_CLOSING_EVIDENCE_ANCHOR: &str = "working-tree-closing-evidence";
const SCOPED_REVIEW_REQUIRED_CHECK: &str = "SpecSync scoped review";
const PORTABLE_DEFINITION_PROJECTION_V501: &str = "specsync-5.0.1";
const DEFINITION_APPROVAL_PAIR_DOMAIN: &[u8] = b"specsync.definition-approval-pair.v1";
const CORRECTION_PREFIX_DOMAIN: &[u8] = b"specsync.correction-prefix.v1";
const MAX_CHANGE_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_GIT_EVIDENCE_PATHS: usize = 100_000;
const MAX_GIT_EVIDENCE_PATH_BYTES: usize = 64 * 1024 * 1024;
const GIT_ATTRIBUTE_BATCH_PATHS: usize = 256;
const MAX_GIT_ATTRIBUTE_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_GIT_COMMAND_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_GIT_DIAGNOSTIC_BYTES: usize = 64 * 1024;
const MAX_GIT_INDEX_BYTES: usize = 256 * 1024 * 1024;
const MAX_GIT_EVIDENCE_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;
const MAX_CHANGE_READ_CACHE_ENTRIES: usize = 100_000;
const GIT_COMMAND_DEADLINE: Duration = Duration::from_secs(120);
const CANONICAL_SPEC_COMPANIONS: [&str; 5] = [
    "requirements.md",
    "tasks.md",
    "context.md",
    "testing.md",
    "design.md",
];

/// Companion names for one module, including its optional `<module>.3md`.
///
/// The 3md file is named after the module, so it cannot live in the fixed list
/// above. `change check --commit` stages these paths. Leaving the 3md file off
/// the list left a new lesson file untracked.
fn canonical_companion_names(module: &str) -> Vec<String> {
    let mut names: Vec<String> = CANONICAL_SPEC_COMPANIONS
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    names.push(format!("{module}.3md"));
    names
}
static EFFECTIVE_CONTRACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static TRUSTED_CORRECTION_HISTORY_CACHE: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();
static GIT_BLOB_CACHE: OnceLock<Mutex<BTreeMap<String, Vec<u8>>>> = OnceLock::new();
/// A test hook called with the PID of each spawned Git child.
#[cfg(test)]
type TestGitChildSpawnedHook = Box<dyn FnOnce(u32)>;
#[cfg(test)]
thread_local! {
    static TEST_GIT_CHILD_SPAWNED: RefCell<Option<TestGitChildSpawnedHook>> = RefCell::new(None);
    static TEST_GIT_STDIN_WRITER_JOINED: Cell<bool> = const { Cell::new(false) };
    static TRANSACTION_WRITE_FAILURE_INDEX: RefCell<Option<usize>> = const { RefCell::new(None) };
    static TRANSACTION_AFTER_JOURNAL_HOOK: RefCell<Option<Box<dyn FnOnce()>>> =
        RefCell::new(None);
    /// When true, `reconstruct_legacy_at_anchor` pretends `git worktree remove` failed.
    /// Product #511: cleanup hygiene must not discard a successful reconstruction.
    static FORCE_LEGACY_WORKTREE_REMOVE_FAILURE: Cell<bool> = const { Cell::new(false) };
    /// `(batch, object)`: shrinks the evidence read bounds so a small fixture can exceed them
    /// (#976). `None` keeps the production bound.
    static TEST_GIT_EVIDENCE_CAPS: Cell<(Option<usize>, Option<usize>)> =
        const { Cell::new((None, None)) };
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GitEvidenceCacheKey {
    regular_files_only: bool,
    candidates: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DiscoveredEvidenceCacheKey {
    scopes: Option<Vec<String>>,
    extra_candidates: Vec<String>,
    regular_files_only: bool,
    /// Clean tracked files are named by Git object id instead of their blob bytes.
    identify_clean_blobs: bool,
    /// Validated `verification_ignored_paths` prefixes left out of discovery (empty for every
    /// caller except the project-input digest with a non-empty list).
    excluded_prefixes: Vec<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct GitTextQueryCacheKey {
    allow_empty: bool,
    arguments: Vec<String>,
}

#[derive(Debug)]
struct ChangeReadSnapshot {
    root: PathBuf,
    active_records: Option<Result<ChangeRoster, String>>,
    all_records: Option<Result<BTreeMap<String, ChangeRecord>, String>>,
    repository_present: Option<Result<bool, String>>,
    repository_context: Option<Result<RepositoryContext, String>>,
    checkout_overrides: Option<Result<Vec<String>, String>>,
    project_input_digest: Option<Result<String, String>>,
    verification_ignored_paths: Option<Result<Vec<String>, String>>,
    git_evidence: BTreeMap<GitEvidenceCacheKey, Result<GitEvidence, String>>,
    discovered_evidence:
        BTreeMap<DiscoveredEvidenceCacheKey, Result<(Vec<String>, GitEvidence), String>>,
    git_text_queries: BTreeMap<GitTextQueryCacheKey, Option<String>>,
    git_status_queries: BTreeMap<Vec<String>, Result<bool, String>>,
    workflow_version_history: BTreeMap<String, Result<(), String>>,
    historical_sequence_ledgers: BTreeMap<u64, Result<Vec<Vec<u8>>, String>>,
    terminal_evidence: Option<Result<BTreeMap<String, TerminalEvidenceSummary>, String>>,
    archived_terminal_evidence: BTreeMap<String, TerminalEvidenceSummary>,
    archived_integrity_cache: Option<ArchivedIntegrityCache>,
    scoped_review_texts: BTreeMap<Vec<String>, Result<String, String>>,
    scoped_review_files: BTreeMap<(String, String), Option<Vec<u8>>>,
    archive_index: Option<BTreeMap<String, Vec<PathBuf>>>,
    persistent_cache: PersistentReadCacheFrame,
}

impl ChangeReadSnapshot {
    fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            active_records: None,
            all_records: None,
            repository_present: None,
            repository_context: None,
            checkout_overrides: None,
            project_input_digest: None,
            verification_ignored_paths: None,
            git_evidence: BTreeMap::new(),
            discovered_evidence: BTreeMap::new(),
            git_text_queries: BTreeMap::new(),
            git_status_queries: BTreeMap::new(),
            workflow_version_history: BTreeMap::new(),
            historical_sequence_ledgers: BTreeMap::new(),
            terminal_evidence: None,
            archived_terminal_evidence: BTreeMap::new(),
            archived_integrity_cache: None,
            scoped_review_texts: BTreeMap::new(),
            scoped_review_files: BTreeMap::new(),
            archive_index: None,
            persistent_cache: PersistentReadCacheFrame::default(),
        }
    }
}

thread_local! {
    static CHANGE_READ_SCOPES: RefCell<Vec<ChangeReadSnapshot>> = const {
        RefCell::new(Vec::new())
    };
    #[cfg(test)]
    static TEST_GIT_PROCESS_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
    #[cfg(test)]
    static TEST_CANONICAL_MODULE_QUERY_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
    #[cfg(test)]
    static TEST_DELTA_BINDING_READ_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

/// Invocation-scoped snapshot for read-only lifecycle commands.
///
/// List, show, status, and read-only project reports install this scope. Mutating lifecycle APIs
/// do not create one, so their repository and evidence validation remains live. The first evidence
/// collection for each candidate set still performs the complete before/after Git race check.
pub(crate) struct ChangeReadScope {
    root: PathBuf,
    /// Every read scope memoizes the delta binding, so one invocation reads each delta once and
    /// `summary` and `approved_deltas` are projections of the same bytes (REQ-change-113).
    _delta_bindings: DeltaBindingMemo,
    /// Repository facts git answered during this read (REQ-git-utils-009).
    _facts: crate::git_utils::RepositoryFactScope,
}

impl Drop for ChangeReadScope {
    fn drop(&mut self) {
        CHANGE_READ_SCOPES.with(|scopes| {
            let mut scopes = scopes.borrow_mut();
            let removed = scopes.pop();
            debug_assert_eq!(
                removed.as_ref().map(|scope| scope.root.as_path()),
                Some(self.root.as_path())
            );
            if let Some(scope) = removed.as_ref() {
                flush_persistent_read_cache(&scope.persistent_cache);
            }
        });
    }
}

/// Write the persistent read cache of every open change read scope on this thread.
fn flush_change_read_scope_caches() {
    CHANGE_READ_SCOPES.with(|scopes| {
        if let Ok(scopes) = scopes.try_borrow() {
            for scope in scopes.iter() {
                flush_persistent_read_cache(&scope.persistent_cache);
            }
        }
    });
}

pub(crate) fn begin_change_read_scope(root: &Path) -> ChangeReadScope {
    CHANGE_READ_SCOPES.with(|scopes| {
        scopes.borrow_mut().push(ChangeReadSnapshot::new(root));
    });
    ChangeReadScope {
        root: root.to_path_buf(),
        _delta_bindings: begin_delta_binding_memo(root),
        _facts: crate::git_utils::begin_repository_fact_scope(root),
    }
}

/// Identity of one memoized delta binding: the change and the record fields the read depends on.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DeltaBindingMemoKey {
    id: String,
    affected_specs: Vec<String>,
    no_spec_change: bool,
}

struct DeltaBindingMemoFrame {
    root: PathBuf,
    bindings: BTreeMap<DeltaBindingMemoKey, Result<DeltaBinding, String>>,
}

thread_local! {
    static DELTA_BINDING_MEMOS: RefCell<Vec<DeltaBindingMemoFrame>> = const {
        RefCell::new(Vec::new())
    };
}

/// Drop guard for one delta-binding memo frame (REQ-change-113).
///
/// Read scopes hold one, and `definition_mutation_result` installs one while it builds the
/// mutation's projections under the project lock. The refusing gates (materialization and
/// acceptance) run outside any memo, so they always read the delta files live.
struct DeltaBindingMemo {
    root: PathBuf,
}

impl Drop for DeltaBindingMemo {
    fn drop(&mut self) {
        DELTA_BINDING_MEMOS.with(|memos| {
            let removed = memos.borrow_mut().pop();
            debug_assert_eq!(
                removed.as_ref().map(|frame| frame.root.as_path()),
                Some(self.root.as_path())
            );
        });
    }
}

fn begin_delta_binding_memo(root: &Path) -> DeltaBindingMemo {
    DELTA_BINDING_MEMOS.with(|memos| {
        memos.borrow_mut().push(DeltaBindingMemoFrame {
            root: root.to_path_buf(),
            bindings: BTreeMap::new(),
        });
    });
    DeltaBindingMemo {
        root: root.to_path_buf(),
    }
}

/// The binding memoized by the innermost memo frame, when that frame is for `root`.
fn memoized_delta_binding(
    root: &Path,
    key: &DeltaBindingMemoKey,
) -> Option<Result<DeltaBinding, String>> {
    DELTA_BINDING_MEMOS.with(|memos| {
        let memos = memos.borrow();
        let frame = memos.last().filter(|frame| frame.root == root)?;
        frame.bindings.get(key).cloned()
    })
}

fn memoize_delta_binding(
    root: &Path,
    key: DeltaBindingMemoKey,
    binding: &Result<DeltaBinding, String>,
) {
    DELTA_BINDING_MEMOS.with(|memos| {
        if let Some(frame) = memos
            .borrow_mut()
            .last_mut()
            .filter(|frame| frame.root == root)
        {
            frame.bindings.insert(key, binding.clone());
        }
    });
}

fn ensure_change_read_scope(root: &Path) -> Option<ChangeReadScope> {
    read_scope_value(root, |_| Some(()))
        .is_none()
        .then(|| begin_change_read_scope(root))
}

fn read_scope_value<Value: Clone>(
    root: &Path,
    read: impl FnOnce(&ChangeReadSnapshot) -> Option<Value>,
) -> Option<Value> {
    CHANGE_READ_SCOPES.with(|scopes| {
        let scopes = scopes.borrow();
        let scope = scopes.last()?;
        (scope.root == root).then(|| read(scope)).flatten()
    })
}

fn update_read_scope(root: &Path, update: impl FnOnce(&mut ChangeReadSnapshot)) -> bool {
    CHANGE_READ_SCOPES.with(|scopes| {
        let mut scopes = scopes.borrow_mut();
        let Some(scope) = scopes.last_mut().filter(|scope| scope.root == root) else {
            return false;
        };
        update(scope);
        true
    })
}

#[cfg(test)]
fn record_test_git_process() {
    TEST_GIT_PROCESS_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(not(test))]
fn record_test_git_process() {}

/// The Git processes the current worker thread spawned, for its parent to fold in.
#[cfg(test)]
fn worker_test_git_process_count() -> usize {
    test_git_process_count()
}

#[cfg(not(test))]
fn worker_test_git_process_count() -> usize {
    0
}

#[cfg(test)]
fn add_test_git_processes(spawned: usize) {
    TEST_GIT_PROCESS_COUNT.with(|count| count.set(count.get() + spawned));
}

#[cfg(not(test))]
fn add_test_git_processes(_spawned: usize) {}

#[cfg(test)]
fn reset_test_git_process_count() {
    TEST_GIT_PROCESS_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn test_git_process_count() -> usize {
    TEST_GIT_PROCESS_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
fn record_test_canonical_module_query() {
    TEST_CANONICAL_MODULE_QUERY_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(not(test))]
fn record_test_canonical_module_query() {}

#[cfg(test)]
fn reset_test_canonical_module_query_count() {
    TEST_CANONICAL_MODULE_QUERY_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn test_canonical_module_query_count() -> usize {
    TEST_CANONICAL_MODULE_QUERY_COUNT.with(std::cell::Cell::get)
}

/// Counts each delta body the approval binding reads, so a test can show that one read scope
/// reads each delta once (REQ-change-113).
#[cfg(test)]
fn record_test_delta_binding_read() {
    TEST_DELTA_BINDING_READ_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(not(test))]
fn record_test_delta_binding_read() {}

#[cfg(test)]
fn reset_test_delta_binding_read_count() {
    TEST_DELTA_BINDING_READ_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn test_delta_binding_read_count() -> usize {
    TEST_DELTA_BINDING_READ_COUNT.with(std::cell::Cell::get)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitWorktreeState {
    modified: BTreeSet<String>,
    sparse_absent: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitCapturedEntry {
    kind: AcceptanceInputKind,
    mode: u32,
    object: Option<String>,
    payload: Vec<u8>,
    /// The payload is a Git object id, not file bytes. The workspace digest
    /// frames it under `object-id` so those bytes cannot collide with a dirty
    /// file that happens to contain the same id text.
    object_identity: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitEvidence {
    modes: BTreeMap<String, u32>,
    entries: BTreeMap<String, GitCapturedEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RepositoryContext {
    git: bool,
    identity: String,
}

#[derive(Debug)]
struct BoundedCommandOutput {
    status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl GitEvidence {
    fn entry(&self, path: &str) -> Result<&GitCapturedEntry, String> {
        self.entries
            .get(path)
            .ok_or_else(|| format!("captured evidence is missing candidate `{path}`"))
    }

    fn generated_file_mode(&self, path: &str) -> Result<u32, String> {
        let entry = self.entry(path)?;
        match (&entry.kind, entry.mode) {
            (AcceptanceInputKind::File, mode @ (0o100644 | 0o100755)) => Ok(mode),
            (AcceptanceInputKind::Missing, 0) => Ok(0o100644),
            _ => Err(format!(
                "generated canonical file would replace unsupported topology at `{path}`"
            )),
        }
    }
}

const DEFINITION_DIGEST_DOMAIN: &[u8] = b"specsync.definition-digest.v2";
const SCOPE_DIGEST_DOMAIN: &[u8] = b"specsync.scope-digest.v1";
const PROJECT_DIGEST_DOMAIN: &[u8] = b"specsync.project-input-digest.v4";
const ACCEPTANCE_DIGEST_DOMAIN: &[u8] = b"specsync.acceptance-input-digest.v2";
const ACCEPTANCE_ENTRY_DOMAIN: &[u8] = b"specsync.acceptance-entry.v1";
const ACCEPTANCE_MANIFEST_DOMAIN: &[u8] = b"specsync.acceptance-manifest.v1";
const SEMANTIC_SUCCESSION_DOMAIN: &[u8] = b"specsync.semantic-succession.v1";
const LEGACY_BASELINE_DOMAIN: &[u8] = b"specsync.legacy-archive-baseline.v1";
const LEGACY_SUBTREE_DOMAIN: &[u8] = b"specsync.legacy-archive-subtree.v1";
const CLOSING_DIGEST_DOMAIN: &[u8] = b"specsync.closing-digest.v2";
const FINALIZATION_DIGEST_DOMAIN: &[u8] = b"specsync.finalization-digest.v2";
const CORRECTION_VIEW_DIGEST_DOMAIN: &[u8] = b"specsync.correction-view-digest.v1";
const APPROVED_DELTA_DIGEST_DOMAIN: &[u8] = b"specsync.approved-delta.v1";
const APPROVED_DELTA_BASE_DOMAIN: &[u8] = b"specsync.approved-delta-base.v1";
const EXACT_TEST_OWNER: &str = "@exact:test";
const EXACT_DELIVERY_OWNER: &str = "@exact:delivery";
const MAX_ACCEPTANCE_ENTRIES: usize = 100_000;
const MAX_ACCEPTANCE_PATH_BYTES: usize = 4_096;
const MAX_ACCEPTANCE_OWNERS: usize = 1_024;
const MAX_ACCEPTANCE_OWNER_BYTES: usize = 256;
const MAX_TRUSTED_HISTORY_COMMITS: usize = 10_000;
const CHG_0068_ID: &str =
    "CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara";
const CHG_0068_LEGACY_APPROVAL_DIGEST: &str =
    "20527b1e1f7f7fd2b310aab328ca865759de081624c519cb56184e3e4f816560";
const CHG_0068_ADOPTED_SCOPE_DIGEST: &str =
    "35f68b68d4cdca134f98f514a56c1afdc4c530ef4f43461f75b7898404fae79b";
const CHG_0068_ADOPTION_ANCHOR_COMMIT: &str = "5e58be031875d89d7d45bf0a5effaf0d9e855ad1";
const CHG_0068_ADOPTION_BASE_COMMIT: &str = "fc091c88f72a6d2fb2df168f4baa4370579ff8a2";
const CHG_0068_ADOPTION_ANCHOR_BLOB: &str =
    "a127ce2a5d51059633d79058bbba65eefc820ea8e4d48ecaddf0941bc92c002d";
const CHG_0068_ADOPTION_CHANGES_DIGEST: &str =
    "0927c338d83284ca99244c58e47659f1a9668e8d3756ffa956855267111e5c40";
const CHG_0068_ADOPTION_REASON: &str =
    "Preserve the already-approved stable scope; the legacy definition preimage is unavailable.";

struct FramedDigest {
    hasher: Sha256,
}

#[derive(Debug, Deserialize)]
struct LifecycleValidationLimits {
    git_max_output_bytes: usize,
    git_timeout_seconds: u64,
    scoped_review_max_descendants: usize,
    scoped_review_max_parents: usize,
}

fn lifecycle_validation_limits() -> &'static LifecycleValidationLimits {
    static LIMITS: OnceLock<LifecycleValidationLimits> = OnceLock::new();
    LIMITS.get_or_init(|| {
        let limits: LifecycleValidationLimits =
            serde_json::from_str(include_str!("../lifecycle-validation-limits.json"))
                .expect("bundled lifecycle validation limits must be valid JSON");
        assert!(limits.git_max_output_bytes > 0);
        assert!(limits.git_timeout_seconds > 0);
        assert!(limits.scoped_review_max_descendants > 0);
        assert!(limits.scoped_review_max_parents > 0);
        limits
    })
}

impl FramedDigest {
    fn new(domain: &[u8]) -> Self {
        let mut digest = Self {
            hasher: Sha256::new(),
        };
        digest.frame(b"domain", domain);
        digest
    }

    fn frame(&mut self, tag: &[u8], value: &[u8]) {
        self.hasher.update((tag.len() as u64).to_be_bytes());
        self.hasher.update(tag);
        self.hasher.update((value.len() as u64).to_be_bytes());
        self.hasher.update(value);
    }

    fn frame_reader<Reader: Read>(
        &mut self,
        tag: &[u8],
        length: u64,
        reader: &mut Reader,
    ) -> Result<(), String> {
        self.hasher.update((tag.len() as u64).to_be_bytes());
        self.hasher.update(tag);
        self.hasher.update(length.to_be_bytes());
        let mut remaining = length;
        let mut buffer = [0_u8; 64 * 1024];
        while remaining > 0 {
            let requested = usize::try_from(remaining.min(buffer.len() as u64))
                .map_err(|_| "framed input length is not representable".to_string())?;
            let read = reader
                .read(&mut buffer[..requested])
                .map_err(|error| format!("failed to stream framed input: {error}"))?;
            if read == 0 {
                return Err("framed input was truncated during hashing".into());
            }
            self.hasher.update(&buffer[..read]);
            remaining -= read as u64;
        }
        Ok(())
    }

    fn entry(&mut self, path: &str, kind: &[u8], mode: u32, content: &[u8]) {
        self.frame(b"entry", b"");
        self.frame(b"path", path.as_bytes());
        self.frame(b"kind", kind);
        self.frame(b"mode", &mode.to_be_bytes());
        self.frame(b"content", content);
    }

    /// Same path, kind, and mode as [`Self::entry`], with the bytes framed as a
    /// Git object id. A file whose bytes are that id text still uses `content`.
    fn object_entry(&mut self, path: &str, kind: &[u8], mode: u32, object_id: &[u8]) {
        self.frame(b"entry", b"");
        self.frame(b"path", path.as_bytes());
        self.frame(b"kind", kind);
        self.frame(b"mode", &mode.to_be_bytes());
        self.frame(b"object-id", object_id);
    }

    fn finish(self) -> String {
        format!("{:x}", self.hasher.finalize())
    }
}

struct ProjectLock {
    file: fs::File,
    /// Repository facts git answered while this command holds the lock (REQ-git-utils-009).
    _facts: crate::git_utils::RepositoryFactScope,
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn acquire_project_lock(root: &Path) -> Result<ProjectLock, String> {
    let path = root.join(LOCK_PATH);
    // Capability no-follow opens are not race-safe for concurrent first-time creators of the
    // metadata directory on all platforms. Use the race-safe std directory/file open path while
    // still rejecting symlink components before and after directory creation so a symlinked
    // `.specsync` root cannot host the lifecycle lock (see
    // `change_adopt_rejects_symlinked_metadata_root_before_lock_write`).
    reject_symlink_components_for(root, &path, "lifecycle lock path")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create lifecycle metadata directory: {error}"))?;
    }
    reject_symlink_components_for(root, &path, "lifecycle lock path")?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|error| format!("failed to open lifecycle lock {}: {error}", path.display()))?;
    file.lock_exclusive().map_err(|error| {
        format!(
            "failed to acquire lifecycle lock {}: {error}",
            path.display()
        )
    })?;
    let lock = ProjectLock {
        file,
        _facts: crate::git_utils::begin_repository_fact_scope(root),
    };
    recover_pending_transaction(root)?;
    Ok(lock)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionEntry {
    path: String,
    original: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionJournal {
    schema_version: u32,
    entry_count: usize,
    entries_digest: String,
    entries: Vec<TransactionEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TransactionJournalRead {
    Durable(TransactionJournal),
    Legacy(Vec<TransactionEntry>),
}

/// The refusal for a journal that cannot be read or parsed: the path, and a set-aside
/// command that runs as printed — both paths shell-quoted, so a project whose path contains a
/// space recovers by pasting (#433, t384's review of #1041).
fn invalid_journal_error(journal_path: &Path, detail: &str) -> String {
    let quoted = shell_quote(&journal_path.display().to_string());
    format!(
        "invalid transaction journal at {quoted}: {detail}; the journal records an interrupted \
         multi-file write and cannot be replayed — inspect it for partial writes, then set it \
         aside with `mv {quoted} {quoted}.broken` and re-run the command"
    )
}

fn recover_pending_transaction(root: &Path) -> Result<(), String> {
    let journal_path = root.join(TRANSACTION_PATH);
    if !journal_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&journal_path)
        .map_err(|error| invalid_journal_error(&journal_path, &error.to_string()))?;
    let journal: TransactionJournalRead = serde_json::from_str(&content)
        .map_err(|error| invalid_journal_error(&journal_path, &error.to_string()))?;
    let entries = match journal {
        TransactionJournalRead::Legacy(entries) => entries,
        TransactionJournalRead::Durable(journal) => {
            if journal.schema_version != 1
                || journal.entry_count != journal.entries.len()
                || journal.entries_digest != transaction_entries_digest(&journal.entries)?
            {
                return Err(invalid_journal_error(
                    &journal_path,
                    "the integrity envelope does not hold",
                ));
            }
            journal.entries
        }
    };
    for entry in entries {
        let path = safe_project_path(root, &entry.path).map_err(|error| {
            invalid_journal_error(&journal_path, &format!("entry `{}`: {error}", entry.path))
        })?;
        if let Some(original) = entry.original {
            atomic_write_durable(&path, original.as_bytes())
                .map_err(|error| format!("failed to restore {}: {error}", path.display()))?;
        } else if path.exists() {
            remove_file_durable(&path)
                .map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
            remove_empty_transaction_directories(root, path.parent());
        }
    }
    remove_file_durable(&journal_path)
        .map_err(|error| format!("failed to clear transaction journal: {error}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeState {
    Draft,
    Approved,
    Implementing,
    Verifying,
    Accepted,
    Archived,
    Retired,
}

impl ChangeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Approved => "approved",
            Self::Implementing => "implementing",
            Self::Verifying => "verifying",
            Self::Accepted => "accepted",
            Self::Archived => "archived",
            Self::Retired => "retired",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Feature,
    BugFix,
    Refactor,
    Migration,
    Documentation,
    Operations,
}

impl ChangeKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().replace('-', "_").as_str() {
            "feature" => Ok(Self::Feature),
            "bug" | "fix" | "bug_fix" => Ok(Self::BugFix),
            "refactor" => Ok(Self::Refactor),
            "migration" => Ok(Self::Migration),
            "documentation" | "docs" => Ok(Self::Documentation),
            "operations" | "operational" | "configuration" | "config" => Ok(Self::Operations),
            _ => Err(format!(
                "unknown change type `{value}` (expected feature, bug-fix, refactor, migration, documentation, or operations)"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Feature => "feature",
            Self::BugFix => "bug_fix",
            Self::Refactor => "refactor",
            Self::Migration => "migration",
            Self::Documentation => "documentation",
            Self::Operations => "operations",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Requirements,
    Research,
    Design,
    Plan,
    Tasks,
    Context,
    Testing,
    Docs,
    Custom(String),
}

impl ArtifactKind {
    pub fn parse(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "requirements" => Self::Requirements,
            "research" => Self::Research,
            "design" => Self::Design,
            "plan" => Self::Plan,
            "tasks" => Self::Tasks,
            "context" => Self::Context,
            "testing" => Self::Testing,
            "docs" | "documentation" => Self::Docs,
            other => Self::Custom(slugify(other)),
        }
    }

    pub fn file_name(&self) -> String {
        match self {
            Self::Requirements => "requirements.md".into(),
            Self::Research => "research.md".into(),
            Self::Design => "design.md".into(),
            Self::Plan => "plan.md".into(),
            Self::Tasks => "tasks.md".into(),
            Self::Context => "context.md".into(),
            Self::Testing => "testing.md".into(),
            Self::Docs => "docs.md".into(),
            Self::Custom(name) => format!("{name}.md"),
        }
    }
}

// The other half of the forward-compatibility valve.
//
// Removing `deny_unknown_fields` lets an OLD binary read a file a NEW binary wrote. This is the
// reverse direction: it lets a NEW binary read a file an OLD binary wrote, by supplying the
// `Default` value for any field that did not exist when the file was written.
//
// Without it, not one of these eight fields is optional on deserialize. That works today only
// because SpecSync always writes every field — so the day a ninth field is added in 6.x, every
// `sdd.json` written before it becomes unreadable by the binary that added it. Same one-way
// door as `deny_unknown_fields`, walked from the other side.
//
// `#[serde(default)]` on the container uses the existing `Default` impl below, so a field added
// later needs no per-field attribute to stay readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SddPolicy {
    pub version: u32,
    pub enabled: bool,
    pub require_change_for_meaningful_files: bool,
    pub meaningful_paths: Vec<String>,
    pub ignored_paths: Vec<String>,
    pub verification_commands: Vec<String>,
    pub custom_artifacts: BTreeMap<String, String>,
    pub principles_file: Option<String>,
    /// Directory prefixes (each ending in `/`) whose files are left out of the verification
    /// project-input digest and nothing else (issue #825). Empty by default and omitted from the
    /// written policy when empty, so default and adopted policies keep their bytes. Every
    /// lifecycle verb validates it through `validated_verification_ignored_paths`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verification_ignored_paths: Vec<String>,
}

impl Default for SddPolicy {
    fn default() -> Self {
        Self {
            version: 2,
            enabled: true,
            require_change_for_meaningful_files: true,
            meaningful_paths: vec![
                "src/".into(),
                "tests/".into(),
                "site/".into(),
                ".github/".into(),
                "Cargo.toml".into(),
                "Cargo.lock".into(),
                "action.yml".into(),
                "package.json".into(),
                "bun.lock".into(),
                "package-lock.json".into(),
                "pnpm-lock.yaml".into(),
                "yarn.lock".into(),
                "Package.swift".into(),
                "Package.resolved".into(),
                "go.mod".into(),
                "go.sum".into(),
                "pyproject.toml".into(),
                "uv.lock".into(),
                "requirements.txt".into(),
                ".specsync/sdd.json".into(),
                ".specsync/config.toml".into(),
                ".specsync/config.json".into(),
                ".specsync/registry.toml".into(),
                ".specsync/version".into(),
            ],
            ignored_paths: vec![".specsync/".into()],
            verification_commands: Vec::new(),
            custom_artifacts: BTreeMap::new(),
            principles_file: None,
            verification_ignored_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub schema_version: u32,
    #[serde(
        default = "legacy_workflow_version",
        skip_serializing_if = "is_legacy_workflow_version"
    )]
    pub workflow_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_origin_version: Option<u32>,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub kind: ChangeKind,
    pub state: ChangeState,
    #[serde(default, skip_serializing_if = "is_false")]
    pub canonical_applied: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub correction_count: u64,
    pub base_commit: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub affected_specs: Vec<String>,
    pub affected_paths: Vec<String>,
    pub no_spec_change: bool,
    pub no_spec_change_rationale: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub selected_artifacts: Vec<ArtifactKind>,
    pub dependencies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersedes: Vec<SupersedesEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_owner_corrections: Vec<AcceptanceOwnerCorrection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_archive_baseline_digest: Option<String>,
    pub answers: BTreeMap<String, String>,
}

// Evidence persisted to disk is deliberately TOLERANT of unknown fields.
//
// These structs carried `#[serde(deny_unknown_fields)]`, which meant an older 6.x binary could
// not read a file written by a newer 6.x binary that had added a field — so no evidence shape
// could be extended during 6's lifetime without breaking installations already deployed. That
// is the mechanism by which "just add a field in 6.4" becomes "we need 7.0".
//
// The tolerant structs are where growth has actually happened: `ChangeRecord`, `SddPolicy`,
// `ApprovalLedger`, `VerificationRecord` and `ChangeSequenceLedger` have absorbed
// `workflow_version`, `canonical_applied`, `correction_count`, `supersedes`,
// `acceptance_owner_corrections` and `reopenings` without incident, and `approvals.json` files
// written before `reopenings` existed still parse today.
//
// What this does NOT buy: `ApprovedScopeV1`, `CorrectionRecord` and `ScopedReviewRecord` are
// digest preimages (scope_digest, correction digests, finalization review_digest). Adding a
// field to one of those still changes its serialized bytes and therefore its digest. Tolerance
// lets an older reader PARSE such a file instead of erroring; it does not make field addition
// digest-safe for those three. Read-time tolerance was never part of any preimage, so removing
// it changes no digest.
//
// Where the line actually falls, checked rather than assumed:
//
// - `hash_cache.rs` keeps `deny_unknown_fields`. `.specsync/hashes.json` is gitignored and
//   `HashCache::load` returns `Self::default()` on any parse error, so an unrecognised shape
//   costs one rebuild. That is a real cache.
// - `agents.rs` does NOT keep it. An earlier version of this comment claimed
//   `.specsync/agent-artifacts.json` was the same kind of thing; it is not.
//   `load_agent_artifact_manifest` returns `Err`, the file is git-tracked and shared with the
//   team, and its content — the digest of exactly the bytes SpecSync last generated — is what
//   distinguishes "unchanged since we wrote it" from "the user edited it". Losing it is not
//   free, so it is evidence and it is tolerant.
//
// Two files gain nothing from tolerance and it would be dishonest to imply otherwise:
// `workflow-v2-baseline` and the legacy archive baseline are read through
// `bytes_match_canonical_json`, a round-trip byte-equality gate strictly stronger than this
// attribute. An added field survives `from_slice` and then fails the byte comparison. See
// `a_baseline_is_still_frozen_by_its_canonical_byte_gate`.
//
// Tolerance is the READ side only. On its own it made the failure silent: a typed rewrite then
// dropped every field it had tolerated. The WRITE side is `json_content_keeping_unknown_members`
// (unknown_members.rs): a record or ledger rewritten in place keeps the members this binary does
// not know (#1043, #434).

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceOwnerCorrection {
    pub schema_version: u32,
    pub sequence: u64,
    pub path: String,
    pub module: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyArchiveBaselineV1 {
    pub schema_version: u32,
    pub domain: String,
    pub authority_change_id: String,
    pub cutoff_commit: String,
    pub entries: Vec<LegacyArchiveBaselineEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct WorkflowV2Baseline {
    schema_version: u32,
    domain: String,
    cutoff_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowV2AdoptionGitSnapshot {
    head: Option<String>,
    comparison_reference: Option<String>,
    comparison_tip: Option<String>,
    cutoff_commit: Option<String>,
}

#[derive(Debug, Clone)]
struct WorkflowV2AdoptionCandidate {
    baseline: WorkflowV2Baseline,
    git_snapshot: WorkflowV2AdoptionGitSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyArchiveBaselineEntryV1 {
    pub id: String,
    pub archive_path: String,
    pub introduction_commit: String,
    pub subtree_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuccessionObligation {
    pub path: String,
    pub module: String,
    pub predecessor_entry_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupersedesEdge {
    pub predecessor_id: String,
    pub obligations: Vec<SuccessionObligation>,
}

#[derive(Debug, Clone)]
pub struct CreateChangeRequest {
    pub description: String,
    pub kind: ChangeKind,
    pub affected_specs: Vec<String>,
    pub affected_paths: Vec<String>,
    pub requested_artifacts: Vec<ArtifactKind>,
    pub no_spec_change: bool,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedScopeV1 {
    pub schema_version: u32,
    pub change_id: String,
    pub title: String,
    pub description: String,
    pub kind: ChangeKind,
    pub affected_specs: Vec<String>,
    pub affected_paths: Vec<String>,
    pub no_spec_change: bool,
    pub no_spec_change_rationale: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<String>,
    pub supersedes: Vec<SupersedesEdge>,
    pub answers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonMaterialScopeChangeCategory {
    Implementation,
    TestEvidence,
    CanonicalMaterialization,
    LifecycleMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonMaterialScopeChangeV1 {
    pub path: String,
    pub category: NonMaterialScopeChangeCategory,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeApprovalMigrationV1 {
    pub schema_version: u32,
    pub source_definition_digest: String,
    pub scope_digest: String,
    pub changes: Vec<NonMaterialScopeChangeV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeAdoptionSourcePreimageStatus {
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeAdoptionEquivalenceClaim {
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeAdoptionAnchorV1 {
    pub base_commit: String,
    pub commit: String,
    pub approval_index: u64,
    pub approvals_blob_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeAdoptionAuthorizationV1 {
    pub actor: String,
    pub recorded_at: u64,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeAdoptionV1 {
    pub schema_version: u32,
    pub change_id: String,
    pub source_approval_index: u64,
    pub legacy_approval_digest: String,
    pub source_preimage_status: ScopeAdoptionSourcePreimageStatus,
    pub equivalence_claim: ScopeAdoptionEquivalenceClaim,
    pub adopted_scope: ApprovedScopeV1,
    pub adopted_scope_digest: String,
    pub anchor: ScopeAdoptionAnchorV1,
    pub authorization: ScopeAdoptionAuthorizationV1,
    pub changes: Vec<NonMaterialScopeChangeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub gate: String,
    pub actor: String,
    pub timestamp: u64,
    pub digest: String,
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition_pair: Option<DefinitionApprovalPairV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_scope: Option<ApprovedScopeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_migration: Option<ScopeApprovalMigrationV1>,
    // Delta BODIES were bound by nothing under workflow v2 (#704).
    //
    // The v1 definition digest hashed every delta file's payload through
    // `definition_artifact_snapshot`, so editing `deltas/<module>.md` after approval invalidated
    // the approval. The v2 stable-scope projection deliberately hashes intent and boundary only,
    // and nothing replaced that binding: `validate_delta_files` reads filenames,
    // `project_input_digest` excludes `.specsync/changes/`. A delta swapped between `approve` and
    // materialization therefore rewrote the canonical spec with wording no approver ever saw.
    //
    // `None` is the ONLY honest reading for every approval written before this field existed —
    // 183 archived changes and counting. It means "this approval made no claim about delta
    // bodies", never "the bodies were tampered with". Absent evidence that reads as a violation
    // is the failure mode this repository has already shipped three times (#672, #684, #689), so
    // the check below proceeds on `None` and judges only what an approver actually signed.
    //
    // What `None` may NOT mean is "this approval declines to make a claim" (#719). Every writer of
    // a `definition` gate records the wording it approved — `append_approval`, the normalizing
    // approval in `accept_change`, and the `--portable-5-0-1` pair alike — so within one ledger
    // the field only ever goes from absent to present. `ensure_approved_delta_bodies_unchanged`
    // enforces exactly that: it trusts absence, and refuses a ledger that walks it back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_delta_digests: Option<BTreeMap<String, String>>,
    // The canonical text each `## MODIFIED` / `## REMOVED` item was approved AGAINST (#843).
    //
    // A modified body is a full replacement written against the block as it stood when the
    // approver read it. When the base branch later changes that same block, re-applying the
    // body deletes text nobody approved removing. Recording the block's digest at approval lets
    // materialization and audit tell "the block this change was approved over" from "a block
    // another change has since rewritten", and refuse the second instead of overwriting it.
    //
    // Keyed by `delta_base_key`. `None` is every approval written before this field existed and
    // reads as unknown, never as violated, exactly like `approved_delta_digests` above.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_delta_bases: Option<BTreeMap<String, String>>,
    // A human decision an agent relayed (#633, CHANGE-13). A claim like `actor`, covered by no
    // digest, and omitted when absent so every approval written without one keeps its bytes.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_relayed_decision"
    )]
    pub relayed_by: Option<RelayedDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionApprovalPairRole {
    Current,
    Legacy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionApprovalPairV1 {
    pub schema_version: u32,
    pub projection: String,
    pub pair_id: String,
    pub role: DefinitionApprovalPairRole,
    pub change_id: String,
    pub correction_count: u64,
    pub correction_prefix_digest: String,
    pub current_digest: String,
    pub legacy_digest: String,
    pub event_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReopenRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
    pub from_state: ChangeState,
    pub to_state: ChangeState,
    pub superseded_approval: ApprovalRecord,
    pub prior_verification: VerificationRecord,
    pub stale_acceptance_input_digest: String,
    pub current_acceptance_input_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_evidence_cause: Option<ReopenCauseV1>,
    /// Exact remote-default commit retaining the entire inconsistent original package.
    /// Its old scoped review remains invalid; recovery only downgrades to verifying.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_review_mismatch_anchor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReopenCauseV1 {
    VerificationCommitUnanchored,
    LegacyAcceptanceUnreconstructible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReopenResult {
    pub change: ChangeRecord,
    pub audit: ReopenRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionField {
    PublicContract,
    ArchitectureRisk,
}

impl CorrectionField {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "public_contract" => Ok(Self::PublicContract),
            "architecture_risk" => Ok(Self::ArchitectureRisk),
            _ => Err(format!(
                "unsupported correction field `{value}` (expected public_contract or architecture_risk)"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PublicContract => "public_contract",
            Self::ArchitectureRisk => "architecture_risk",
        }
    }

    fn required_artifacts(self) -> &'static [ArtifactKind] {
        const PUBLIC_CONTRACT: &[ArtifactKind] = &[ArtifactKind::Requirements, ArtifactKind::Docs];
        const ARCHITECTURE_RISK: &[ArtifactKind] = &[
            ArtifactKind::Research,
            ArtifactKind::Design,
            ArtifactKind::Plan,
            ArtifactKind::Tasks,
            ArtifactKind::Testing,
        ];
        match self {
            Self::PublicContract => PUBLIC_CONTRACT,
            Self::ArchitectureRisk => ARCHITECTURE_RISK,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionRecord {
    pub schema_version: u32,
    pub sequence: u64,
    pub change_id: String,
    pub field: CorrectionField,
    pub original_value: String,
    pub prior_effective_value: String,
    pub corrected_value: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
    pub prior_view_digest: String,
    pub corrected_view_digest: String,
    pub added_artifacts: Vec<ArtifactKind>,
    pub superseded_definition_approval: ApprovalRecord,
    pub superseded_closing_approval: ApprovalRecord,
    pub prior_verification: VerificationRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveChangeDefinition {
    pub answers: BTreeMap<String, String>,
    pub selected_artifacts: Vec<ArtifactKind>,
    pub view_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionResult {
    pub change: ChangeRecord,
    pub correction: CorrectionRecord,
    pub effective_definition: EffectiveChangeDefinition,
    pub corrections: Vec<CorrectionRecord>,
    pub summary: ChangeSummary,
}

pub(crate) struct DefinitionMutationResult {
    pub(crate) change: ChangeRecord,
    pub(crate) effective_definition: EffectiveChangeDefinition,
    pub(crate) corrections: Vec<CorrectionRecord>,
    pub(crate) summary: ChangeSummary,
    pub(crate) strict_summary: ChangeSummary,
    /// Built under the project lock with the same delta-binding read as `summary`, so the
    /// mutation's JSON never mixes an in-lock summary with an out-of-lock delta read.
    pub(crate) approved_deltas: Vec<ApprovedDeltaSummary>,
}

/// The item keys one semantic delta declares for one target, split by operation. Each list is
/// sorted and deduplicated.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaDeclarations {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
}

/// What the current definition approval covers for one affected module (REQ-change-113).
///
/// `delta_intact` is `Some(true)` when the delta body matches the digest the approval recorded
/// (CRLF folded), `Some(false)` when it does not or the binding is refused as a whole, and `None`
/// when no definition approval in the ledger recorded a digest. `requirements`, `sections` and
/// `materialized` are present only for an intact body the current delta grammar parses; an intact
/// body the grammar refuses carries `declarations_error` instead. A list is withheld, never
/// emptied, when its provenance is not proven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovedDeltaSummary {
    pub module: String,
    pub delta_intact: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirements: Option<DeltaDeclarations>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sections: Option<DeltaDeclarations>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declarations_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CorrectionLedger {
    schema_version: u32,
    corrections: Vec<CorrectionRecord>,
}

impl Default for CorrectionLedger {
    fn default() -> Self {
        Self {
            schema_version: 1,
            corrections: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalLedger {
    pub approvals: Vec<ApprovalRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope_adoptions: Vec<ScopeAdoptionV1>,
    #[serde(default)]
    pub reopenings: Vec<ReopenRecord>,
    /// Audited send-backs to draft for re-scoping (#844). Each one cancels every approval
    /// recorded before it without removing any of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub send_backs: Vec<SendBackRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEvidence {
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    /// Bounded findings text, recorded on failure so a recorded failure is
    /// classifiable without re-running the command (#640). Omitted on success
    /// so passing records serialize exactly as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// The last 4 KiB of the findings that failed a check, behind a `…` line when cut, so
/// the record names what failed without a verbose failure growing the ledger without
/// bound. `None` when there is no finding text to record.
pub(crate) fn bounded_failure_output(findings: &[String]) -> Option<String> {
    let joined = findings.join("\n");
    if joined.trim().is_empty() {
        return None;
    }
    const CAP: usize = 4096;
    if joined.len() <= CAP {
        return Some(joined);
    }
    // Byte offsets, so the cap can land inside a multi-byte character —
    // findings carry spec text with `↔`, `—`, `…`. Walk to a boundary first.
    let mut start = joined.len() - CAP;
    while !joined.is_char_boundary(start) {
        start += 1;
    }
    // Start the tail at a line boundary when one is near and text follows it; snapping to a
    // newline far into the tail, or to a trailing one, would throw most of it away.
    const SNAP_WINDOW: usize = 512;
    let boundary = joined[start..]
        .find('\n')
        .filter(|offset| *offset < SNAP_WINDOW && start + offset + 1 < joined.len())
        .map(|offset| start + offset + 1)
        .unwrap_or(start);
    Some(format!("…\n{}", &joined[boundary..]))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub timestamp: u64,
    pub commit: Option<String>,
    pub contract_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_digest: Option<String>,
    #[serde(default)]
    pub workspace_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_input_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_manifest: Option<AcceptanceManifestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_succession: Option<SemanticSuccessionEvidenceV1>,
    pub passed: bool,
    pub commands: Vec<CommandEvidence>,
    pub requirement_ids: Vec<String>,
    /// Why a check was refused before verification started (#847). Present only on attempts
    /// that never ran the gate; absent from every record written before the field existed, so
    /// historical ledgers stay valid and an older binary drops it on rewrite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedReviewRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub reviewer: String,
    pub provenance: ScopedReviewProvenanceV1,
    pub verdict: ScopedReviewVerdict,
    pub implementation_commit: String,
    pub contract_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_digest: Option<String>,
    pub workspace_digest: String,
    pub timestamp: u64,
    // A human decision an agent relayed (#633, SHIP-2.e). Left out of the finalization review
    // digest (`scoped_review_digest`), so a 6.x binary that drops the key computes the same one.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_relayed_decision"
    )]
    pub relayed_by: Option<RelayedDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopedReviewProvenanceProvider {
    GithubActionsCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopedReviewProvenanceV1 {
    pub schema_version: u32,
    pub provider: ScopedReviewProvenanceProvider,
    pub required_check: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ScopedReviewAttemptLedger {
    schema_version: u32,
    reviews: Vec<ScopedReviewRecord>,
}

impl Default for ScopedReviewAttemptLedger {
    fn default() -> Self {
        Self {
            schema_version: 1,
            reviews: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopedReviewVerdict {
    Pass,
    Block,
}

impl ScopedReviewVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Block => "block",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pass" => Ok(Self::Pass),
            "block" => Ok(Self::Block),
            _ => Err(format!(
                "invalid scoped review verdict `{value}`; expected `pass` or `block`"
            )),
        }
    }
}

fn scoped_review_provenance_valid(review: &ScopedReviewRecord) -> bool {
    review.provenance.schema_version == 1
        && review.provenance.provider == ScopedReviewProvenanceProvider::GithubActionsCheck
        && review.provenance.required_check == SCOPED_REVIEW_REQUIRED_CHECK
}

fn validate_scoped_reviewer_claim(reviewer: &str) -> Result<&str, String> {
    let reviewer = reviewer.trim();
    if reviewer.is_empty() || reviewer.len() > 128 {
        return Err("scoped reviewer claim must contain between 1 and 128 ASCII characters".into());
    }
    if !reviewer.is_ascii()
        || !reviewer.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, ' ' | '.' | '_' | ':' | '@' | '/' | '-')
        })
    {
        return Err(
            "scoped reviewer claim contains unsupported characters; use a stable ASCII identity"
                .into(),
        );
    }
    Ok(reviewer)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizationRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub implementation_commit: String,
    pub implementation_tree: String,
    pub contract_digest: String,
    pub workspace_digest: String,
    pub closing_digest: String,
    pub review_digest: String,
    pub finalization_digest: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceInputKind {
    File,
    Symlink,
    Gitlink,
    Missing,
    NonFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceInputEntryV1 {
    pub path: String,
    pub kind: AcceptanceInputKind,
    pub mode: u32,
    pub payload_digest: String,
    pub entry_digest: String,
    pub owners: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceManifestV1 {
    pub schema_version: u32,
    pub entries: Vec<AcceptanceInputEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSuccessionTupleV1 {
    pub predecessor_id: String,
    pub path: String,
    pub module: String,
    pub predecessor_entry_digest: String,
    pub successor_entry_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSuccessionEvidenceV1 {
    pub schema_version: u32,
    pub tuples: Vec<SemanticSuccessionTupleV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_reconciliation: Option<HistoricalReconciliationEvidenceV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalReceiptWitnessV1 {
    pub predecessor_id: String,
    pub package_digest: String,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalReconciliationEvidenceV1 {
    pub schema_version: u32,
    pub transition_status: String,
    pub anchor_commit: String,
    pub predecessors: Vec<HistoricalReceiptWitnessV1>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct VerificationAttemptLedger {
    schema_version: u32,
    attempts: Vec<VerificationRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ChangeSequenceCollision {
    sequence: u64,
    ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChangeSequenceLedger {
    schema_version: u32,
    sequence: u64,
    id: String,
    #[serde(default)]
    acknowledged_collisions: Vec<ChangeSequenceCollision>,
}

#[derive(Debug)]
struct LocatedChangeSequence {
    /// `None` for a slug-only ID, which claims no ordinal and therefore takes part in no
    /// numeric collision. Ordinals are only ever read from IDs minted before the allocator
    /// was retired; nothing mints a new one.
    sequence: Option<u64>,
    id: String,
    path: String,
    historical: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct VerificationRouting {
    #[serde(default)]
    component_verification_commands: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    strict_verification_commands: Vec<String>,
    #[serde(default)]
    strict_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterviewQuestion {
    pub id: String,
    pub prompt: String,
    pub choices: Vec<String>,
    pub recommended: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub id: String,
    pub title: String,
    pub state: ChangeState,
    pub approval_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope_expansion: Vec<String>,
    /// Sorted modules whose semantic delta body changed after the definition approval recorded
    /// it; omitted when empty. The reason `approval_valid` is false when `scope_expansion` is empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_approved_deltas: Vec<String>,
    pub artifacts_complete: bool,
    #[serde(default)]
    pub correction_valid: bool,
    #[serde(default)]
    pub correction_count: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub corrected_fields: BTreeMap<String, String>,
    #[serde(default)]
    pub scoped_review_current: bool,
    #[serde(default)]
    pub strict_validation_required: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_commands: Vec<String>,
    /// The validated `.specsync/sdd.json` `verification_ignored_paths` list; omitted when empty
    /// or invalid (an invalid list is the next action instead). Output only, never persisted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_ignored_paths: Vec<String>,
    pub next_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_evidence: Option<TerminalEvidenceSummary>,
    /// Whether clearing context now loses anything, and what to do first if it does.
    pub handoff: HandoffSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalEvidenceSummary {
    pub validity: TerminalEvidenceValidity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalEvidenceValidity {
    Exact,
    SuccessorCovered,
    Stale,
    AuthenticatedHistory,
    HistoricalTransitionUnproven,
    CorruptHistory,
}

impl TerminalEvidenceValidity {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::SuccessorCovered => "successor-covered",
            Self::Stale => "stale",
            Self::AuthenticatedHistory => "authenticated-history",
            Self::HistoricalTransitionUnproven => "historical-transition-unproven",
            Self::CorruptHistory => "corrupt-history",
        }
    }
}

/// Whether an agent can clear its context — or hand this change to a fresh session — without
/// losing something the lifecycle still needs. `Safe`: everything the next session needs is
/// committed or recorded, and `specsync change status <id>` resumes it. `Conditional`: the work
/// is on disk but its intent or lifecycle position is not pinned yet, so do `before_clearing`
/// first or accept redoing a step. `NotYet`: something exists only in this session, and clearing
/// now loses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffReadiness {
    Safe,
    Conditional,
    NotYet,
}

impl HandoffReadiness {
    /// The word the text line prints; `NotYet` reads as two words there, not the JSON hyphen.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Conditional => "conditional",
            Self::NotYet => "not yet",
        }
    }
}

/// The handoff verdict for one change: readiness, one plain-language reason, the command a
/// fresh session resumes with, and the steps to take before clearing when it is not safe.
/// Every string is built from literals and the change ID only — never a digest — so the text
/// line can print it verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffSummary {
    pub readiness: HandoffReadiness,
    pub reason: String,
    pub resume: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub before_clearing: Vec<String>,
}

/// The lifecycle facts a handoff verdict is decided from, gathered once by the caller so the
/// decision itself is a pure function a test can drive without a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffSignals {
    pub state: ChangeState,
    pub workflow_version: u32,
    /// A project-wide sequence-ledger freeze outranks every state-local verdict.
    pub sequence_frozen: bool,
    pub open_questions: bool,
    pub artifacts_complete: bool,
    pub approval_valid: bool,
    pub correction_valid: bool,
    /// Uncommitted edits under `affected_paths`, ignoring `.specsync/`. `None` when Git could
    /// not answer (not a repository) and is read as clean: nothing there is committed anyway.
    pub scoped_edits_uncommitted: Option<bool>,
    pub verification_current: bool,
    pub scoped_review_current: bool,
    /// Legacy accepted evidence that no longer matches the tree, or a closing approval that
    /// no longer validates; either needs a `reopen` with a reason only this session knows.
    pub terminal_evidence_stale: bool,
    /// An archived record whose terminal evidence is `corrupt-history`, and where its bytes can
    /// be restored from. `None` for every other state, and for an archive that authenticates.
    pub corrupt_archive: Option<CorruptArchive>,
}

/// Where an archive that fails its integrity check can be restored from. Holds a path at most,
/// never a digest, so the recovery built from it can reach a text line verbatim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorruptArchive {
    /// The package's tracked files differ from `HEAD`, so `HEAD` still holds committed bytes.
    /// `package` is relative to the project root.
    DiffersFromHead { package: String },
    /// The package matches `HEAD` but not the commit that archived it: the damage is committed,
    /// and that commit holds the trusted bytes.
    DamageCommitted { package: String },
    /// No package restore would help: the package does not resolve to one archive directory,
    /// Git cannot answer, or the package still matches the commit that archived it, so the
    /// evidence that failed lies outside it (the shared legacy baseline, another record).
    OutsidePackage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SddCheckReport {
    pub enabled: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub checked_changes: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub terminal_evidence: Vec<TerminalEvidenceResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalEvidenceResult {
    pub id: String,
    pub evidence: TerminalEvidenceSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeltaOperation {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeltaTarget {
    Requirement,
    SpecSection,
}

#[derive(Debug, Clone)]
struct DeltaItem {
    operation: DeltaOperation,
    target: DeltaTarget,
    key: String,
    content: String,
}

pub fn load_policy(root: &Path) -> Option<SddPolicy> {
    load_policy_checked(root).ok().flatten()
}

fn load_policy_checked(root: &Path) -> Result<Option<SddPolicy>, String> {
    let path = root.join(POLICY_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read SDD policy {}: {error}", path.display()))?;
    let policy: SddPolicy = serde_json::from_str(&content)
        .map_err(|error| format!("invalid SDD policy {}: {error}", path.display()))?;
    disclose_inert_ignored_paths(root, &policy);
    Ok(Some(policy))
}

/// `ignored_paths` entries whose coverage is entirely always-meaningful: the configured specs
/// tree and the protected SDD policy files can never be ignored, so an entry covering only
/// those is inert (#676). An entry that also covers other files (`.specsync/` covers the change
/// workspaces) does real work and stays silent.
fn inert_ignored_paths(policy: &SddPolicy, specs_dir: &str) -> Vec<String> {
    let specs_root = format!("{}/", specs_dir.trim_start_matches("./").trim_matches('/'));
    policy
        .ignored_paths
        .iter()
        .filter(|entry| {
            let entry = entry.trim_start_matches("./");
            (entry.starts_with(specs_root.as_str()) || entry == specs_root.trim_end_matches('/'))
                || PROTECTED_SDD_PATHS.contains(&entry)
        })
        .cloned()
        .collect()
}

/// Disclose inert `ignored_paths` entries where the conflict exists — at policy load, on
/// stderr, once per process — rather than three verbs later at audit (#676). stdout stays
/// clean for `--json` payloads.
fn disclose_inert_ignored_paths(root: &Path, policy: &SddPolicy) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static DISCLOSED: AtomicBool = AtomicBool::new(false);
    if DISCLOSED.swap(true, Ordering::Relaxed) {
        return;
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    for entry in inert_ignored_paths(policy, &specs_dir) {
        eprintln!(
            "note: ignored_paths entry `{entry}` is inert — SDD policy files and the \
             configured specs tree are always meaningful; remove the entry from {POLICY_PATH}"
        );
    }
}

pub fn write_default_policy(root: &Path, verification_commands: Vec<String>) -> Result<(), String> {
    let path = root.join(POLICY_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if path.exists() {
        return Ok(());
    }
    let policy = default_policy(root, verification_commands);
    write_json(&path, &policy)
}

fn default_policy(root: &Path, verification_commands: Vec<String>) -> SddPolicy {
    // Fresh `init` writes SDD *off*. `SddPolicy::default()` stays fail-closed for
    // omitted fields on deserialize (an old file missing `enabled` must not read
    // as disabled). New files set the off-by-default values explicitly.
    let mut policy = SddPolicy {
        verification_commands,
        enabled: false,
        require_change_for_meaningful_files: false,
        ..SddPolicy::default()
    };
    for source_dir in crate::config::load_config(root).source_dirs {
        let normalized = source_dir.replace('\\', "/");
        let scope = if normalized == "." {
            normalized
        } else {
            format!("{}/", normalized.trim_end_matches('/'))
        };
        if !policy.meaningful_paths.contains(&scope) {
            policy.meaningful_paths.push(scope);
        }
    }
    policy
}

pub fn create_change(root: &Path, request: CreateChangeRequest) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    validate_change_sequences(root)?;
    let CreateChangeRequest {
        description,
        kind,
        affected_specs,
        affected_paths,
        requested_artifacts,
        no_spec_change,
        rationale,
    } = request;
    if description.trim().is_empty() {
        return Err("change description cannot be empty".into());
    }
    if no_spec_change && rationale.as_deref().unwrap_or("").trim().is_empty() {
        return Err("--no-spec-change requires --rationale".into());
    }
    for module in &affected_specs {
        crate::commands::validate_module_name(module)
            .map_err(|error| format!("invalid affected spec: {error}"))?;
    }
    ensure_affected_specs_exist(root, &affected_specs)?;
    let affected_paths: Vec<String> = affected_paths
        .iter()
        .map(|path| {
            normalize_project_path(path).map_err(|error| format!("invalid affected path: {error}"))
        })
        .collect::<Result<_, _>>()?;
    let slug = mint_change_slug(&description)?;
    let now = now();
    let mut artifacts = adaptive_artifacts(kind, &affected_specs, &affected_paths);
    for artifact in requested_artifacts {
        if !artifacts.contains(&artifact) {
            artifacts.push(artifact);
        }
    }
    // A baseline means the project already adopted workflow v2. It does not
    // mean a broken policy file can be ignored. Parse the policy either way.
    let policy = load_policy_checked(root)?;
    // Refuse before any workspace exists: an invalid list, or one that would hide a path this
    // change claims (REQ-change-115).
    let verification_ignored = validated_verification_ignored_paths(root)?;
    ensure_affected_paths_clear_of_verification_ignored(&verification_ignored, &affected_paths, None)?;
    let workflow_version = if read_workflow_v2_baseline(root)?.is_some() {
        2
    } else {
        match policy {
            Some(policy) if policy.version >= 2 => 2,
            Some(_) => 1,
            None => 2,
        }
    };
    if workflow_version >= 2 {
        ensure_workflow_v2_baseline(root)?;
    }
    let (id, dir) = allocate_change_workspace(root, &slug)?;
    let record = ChangeRecord {
        schema_version: 1,
        workflow_version,
        workflow_origin_version: Some(workflow_version),
        id: id.clone(),
        slug,
        title: title_from_description(&description),
        description: description.trim().to_string(),
        kind,
        state: ChangeState::Draft,
        canonical_applied: false,
        correction_count: 0,
        base_commit: git_output(root, &["rev-parse", "HEAD"]),
        created_at: now,
        updated_at: now,
        affected_specs,
        affected_paths,
        no_spec_change,
        no_spec_change_rationale: rationale,
        acceptance_criteria: Vec::new(),
        selected_artifacts: artifacts,
        dependencies: Vec::new(),
        supersedes: Vec::new(),
        acceptance_owner_corrections: Vec::new(),
        legacy_archive_baseline_digest: None,
        answers: BTreeMap::new(),
    };
    fs::create_dir_all(dir.join("deltas")).map_err(|error| error.to_string())?;
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    write_json(&dir.join("approvals.json"), &ApprovalLedger::default())?;
    for artifact in &record.selected_artifacts {
        let path = dir.join(artifact.file_name());
        if !path.exists() {
            fs::write(&path, artifact_template(root, artifact, &record))
                .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
        }
    }
    Ok(record)
}

pub fn load_change(root: &Path, id: &str) -> Result<ChangeRecord, String> {
    let path = find_change_dir(root, id)?.join("state.json");
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "active change state file is missing: {}. Restore state.json or remove the stray directory.",
                path.display()
            ));
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let record = serde_json::from_str(&content)
        .map_err(|error| format!("invalid change state {}: {error}", path.display()))?;
    validate_loaded_change(&record, id, &path)?;
    validate_workflow_version_history(root, &record)?;
    Ok(record)
}

/// An active-change workspace that exists on disk but could not be read.
///
/// This type exists so that "there are no changes" and "I could not read the
/// changes" cannot be represented by the same value. Dropping these on the floor
/// is what made one malformed `state.json` print `No active SDD changes.` and
/// exit 0 while healthy siblings sat right beside it (#443).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnreadableChange {
    /// The workspace directory name, which is the change ID for a well-formed
    /// workspace and the only handle we have for a malformed one.
    pub id: String,
    /// Why it could not be read, already carrying the offending path.
    pub reason: String,
}

/// The active-change roster: what could be read, and what could not.
///
/// Callers that act on a record's *absence* must consult [`Self::is_degraded`]
/// first. Absence from `records` means either that the change is not there or
/// that it could not be parsed, and those two demand opposite responses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChangeRoster {
    pub records: Vec<ChangeRecord>,
    pub unreadable: Vec<UnreadableChange>,
}

impl ChangeRoster {
    /// True when at least one workspace could not be read, so the roster is a
    /// partial view and no conclusion may be drawn from a missing record.
    pub fn is_degraded(&self) -> bool {
        !self.unreadable.is_empty()
    }
}

/// The full roster, including workspaces that could not be read.
///
/// `Err` is reserved for failures that leave no partial truth to report — the
/// changes directory itself being unreadable. A single malformed workspace is
/// data, not an error: it lands in [`ChangeRoster::unreadable`] so the caller can
/// report it *and* still show every healthy change.
pub fn list_changes(root: &Path) -> Result<ChangeRoster, String> {
    if let Some(roster) = read_scope_value(root, |scope| scope.active_records.clone()) {
        return roster;
    }
    let result = list_changes_uncached(root);
    update_read_scope(root, |scope| {
        scope.active_records = Some(result.clone());
    });
    result
}

/// The roster as a plain record list, failing closed on any unreadable workspace.
///
/// This is the historical contract every internal caller was written against:
/// one bad workspace aborts the whole read. Those callers compute digests,
/// ledgers and successor sets where a silently short roster is worse than a hard
/// error, so they keep it. Only the presentation layer uses [`list_changes`].
fn list_changes_checked(root: &Path) -> Result<Vec<ChangeRecord>, String> {
    let roster = list_changes(root)?;
    match roster.unreadable.first() {
        Some(unreadable) => Err(unreadable.reason.clone()),
        None => Ok(roster.records),
    }
}

fn list_changes_uncached(root: &Path) -> Result<ChangeRoster, String> {
    let mut roster = ChangeRoster::default();
    let dir = root.join(CHANGES_PATH);
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(roster),
        Err(error) => return Err(format!("failed to read active changes: {error}")),
    };
    for (done, entry) in entries.enumerate() {
        // Loading active changes is a phase with a deadline boundary (REQ-change-149).
        check_read_deadline("active change loading", done, 0)?;
        // Enumeration failures are not scoped to one workspace: we cannot name
        // what we could not enumerate, so there is no partial truth to report.
        let entry =
            entry.map_err(|error| format!("failed to read active change entry: {error}"))?;
        if !entry
            .file_type()
            .map_err(|error| format!("failed to inspect active change entry: {error}"))?
            .is_dir()
        {
            continue;
        }
        let expected_id = match entry.file_name().into_string() {
            Ok(name) => name,
            Err(_) => {
                let path = entry.path();
                roster.unreadable.push(UnreadableChange {
                    id: path.display().to_string(),
                    reason: format!(
                        "active change directory is not valid UTF-8: {}",
                        path.display()
                    ),
                });
                continue;
            }
        };
        let path = entry.path().join("state.json");
        // Git cannot track empty directories, so switching to a branch without this
        // change leaves a husk behind — typically just an empty `deltas/`. Treating
        // that husk as a corrupt change made `change new` fail outright on any
        // branch that did not contain an earlier change, which is an ordinary
        // branching pattern and blocked the first command in the workflow.
        //
        // A husk holds no regular file and is not an active change here. A
        // directory that holds a file but no `state.json` is damaged and still
        // open: report it and keep enumerating. Every other read error is still
        // reported, so an unreadable state.json is never mistaken for an absent one.
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if is_untrackable_husk(&entry.path()) {
                    continue;
                }
                roster.unreadable.push(UnreadableChange {
                    id: expected_id,
                    reason: format!("active change state file is missing: {}", path.display()),
                });
                continue;
            }
            Err(error) => {
                roster.unreadable.push(UnreadableChange {
                    id: expected_id,
                    reason: format!(
                        "failed to read active change state {}: {error}",
                        path.display()
                    ),
                });
                continue;
            }
        };
        let record: ChangeRecord = match serde_json::from_str(&content) {
            Ok(record) => record,
            Err(error) => {
                roster.unreadable.push(UnreadableChange {
                    id: expected_id,
                    reason: format!("invalid active change state {}: {error}", path.display()),
                });
                continue;
            }
        };
        if let Err(reason) = validate_loaded_change(&record, &expected_id, &path)
            .and_then(|()| validate_workflow_version_history(root, &record))
        {
            roster.unreadable.push(UnreadableChange {
                id: expected_id,
                reason,
            });
            continue;
        }
        roster.records.push(record);
    }
    // And once after the last workspace, so a deadline that passed while reading it still stops.
    // Progress counts every completed workspace directory, including the unreadable
    // ones; records alone would under-report them (Codex review of #1005).
    let completed = roster.records.len() + roster.unreadable.len();
    check_read_deadline("active change loading", completed, completed)?;
    roster
        .records
        .sort_by(|left: &ChangeRecord, right: &ChangeRecord| left.id.cmp(&right.id));
    roster
        .unreadable
        .sort_by(|left, right| left.id.cmp(&right.id));
    Ok(roster)
}

fn change_sequence(id: &str) -> Option<u64> {
    let digits = id.strip_prefix("CHG-")?.split('-').next()?;
    if digits.len() < 4 || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let sequence: u64 = digits.parse().ok()?;
    let canonical = if sequence < 10_000 {
        format!("{sequence:04}")
    } else {
        sequence.to_string()
    };
    (digits == canonical).then_some(sequence)
}

/// The ordinal a located record claims, distinguishing "claims none" from "claims one badly".
///
/// A slug-only ID claims no ordinal: `Ok(None)`, and it is simply absent from numeric
/// collision accounting. A `CHG-`-prefixed ID whose leading segment is all digits is
/// claiming one, so a non-canonical width still fails closed — dropping it silently would
/// take it out of the acknowledged-collision ID-set check that guards the archived
/// collision members.
fn located_change_ordinal(id: &str) -> Result<Option<u64>, String> {
    let claims_ordinal = id
        .strip_prefix("CHG-")
        .and_then(|rest| rest.split('-').next())
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()));
    if !claims_ordinal {
        return Ok(None);
    }
    change_sequence(id).map(Some).ok_or_else(|| {
        format!(
            "change ID `{}` claims a CHG ordinal in a non-canonical notation; historical ordinals are four zero-padded digits below 10000 and unpadded decimal digits at or above it",
            id.escape_default()
        )
    })
}

fn load_change_sequence_ledger(root: &Path) -> Result<Option<ChangeSequenceLedger>, String> {
    let path = root.join(SEQUENCE_PATH);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read change sequence ledger: {error}"))?;
    let ledger: ChangeSequenceLedger = serde_json::from_str(&content)
        .map_err(|error| format!("invalid change sequence ledger: {error}"))?;
    if ledger.schema_version != 1 {
        return Err(format!(
            "unsupported change sequence ledger schema version {}",
            ledger.schema_version
        ));
    }
    if change_sequence(&ledger.id) != Some(ledger.sequence) {
        return Err(format!(
            "change sequence ledger claim `{}` does not match sequence {}",
            ledger.id, ledger.sequence
        ));
    }
    Ok(Some(ledger))
}

/// Raise the working-tree sequence ledger to the committed high-water mark, and
/// report whether it had to.
///
/// The ledger records the highest change sequence ever claimed. `change new`
/// writes it into the working tree only, and nothing commits it until a later
/// lifecycle step runs `git add -A`. So a ledger written days earlier, while the
/// shared branch has advanced past it, is staged as a *regression* — the commit
/// records a lower high-water mark than the one it was built on, and the next
/// allocation can hand out an ID that is already taken.
///
/// The floor at allocation time (`maximum_observed_sequence`) does not help
/// here: the value was correct when it was written. It went stale afterwards.
///
/// Raising rather than refusing is deliberate. The author did nothing wrong —
/// their branch simply sat while `main` moved — so blocking the commit would
/// punish the wrong person for a race they cannot see. Returning the old value
/// lets the caller say so, because silently rewriting lifecycle state is how
/// this class of bug survives unnoticed in the first place.
/// The highest sequence this branch's own history has ever recorded.
///
/// Reads every revision of the ledger reachable from HEAD in one `git log -p`
/// and takes the maximum. This is the branch asking a question about itself,
/// which is the only question whose answer can convict it:
///
/// - A branch merely BEHIND the default branch has never recorded anything
///   higher than it holds now, so it is never accused. Comparing against
///   `origin/main` accused every unrebased branch (#533 regression).
/// - A branch that RAISED the ledger and then rewrote it downwards is caught
///   even when the rewrite happened entirely after it diverged — the case a
///   merge-base comparison misses, because the merge-base predates the raise.
///
/// Returns `None` only when git cannot be asked at all; callers must not treat
/// that as evidence of health.
fn branch_sequence_high_water(root: &Path) -> Option<u64> {
    let tracked = git_repo_relative_path(root, SEQUENCE_PATH).ok()?;
    // One invocation, not one per revision: this runs inside `check` and
    // `audit`, and a repo with a few hundred lifecycle commits would otherwise
    // pay a git process per commit that ever touched the ledger.
    let log = git_output(
        root,
        &[
            "log",
            "-p",
            "--format=",
            &format!("-n{SEQUENCE_HISTORY_SCAN_LIMIT}"),
            "HEAD",
            "--",
            &tracked,
        ],
    )?;
    let mut high = None;
    for line in log.lines() {
        // Added lines only: a removed `"sequence": N` is the old value, and
        // counting it would make every ordinary increment look like a rewrite.
        let Some(rest) = line.strip_prefix('+') else {
            continue;
        };
        let rest = rest.trim();
        let Some(value) = rest.strip_prefix("\"sequence\":") else {
            continue;
        };
        let value = value.trim().trim_end_matches(',');
        if let Ok(parsed) = value.parse::<u64>() {
            high = Some(high.map_or(parsed, |current: u64| current.max(parsed)));
        }
    }
    high
}

pub fn floor_sequence_ledger_to_committed(root: &Path) -> Result<Option<(u64, u64)>, String> {
    let Some(local) = load_change_sequence_ledger(root)? else {
        return Ok(None);
    };
    let Ok(tracked) = git_repo_relative_path(root, SEQUENCE_PATH) else {
        return Ok(None);
    };
    let Some(committed) = git_output(root, &["show", &format!("HEAD:{tracked}")]) else {
        return Ok(None);
    };
    let committed: ChangeSequenceLedger = match serde_json::from_str(&committed) {
        Ok(ledger) => ledger,
        // A committed ledger we cannot parse is not evidence of a higher mark.
        // Leave it to the readers that already validate it and report properly.
        Err(_) => return Ok(None),
    };
    if committed.sequence <= local.sequence {
        return Ok(None);
    }
    let mut collisions = committed.acknowledged_collisions;
    for collision in local.acknowledged_collisions {
        if !collisions
            .iter()
            .any(|known| known.sequence == collision.sequence)
        {
            collisions.push(collision);
        }
    }
    collisions.sort_by_key(|collision| collision.sequence);
    write_json_keeping_unknown_members(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence: committed.sequence,
            id: committed.id,
            acknowledged_collisions: collisions,
        },
    )?;
    Ok(Some((local.sequence, committed.sequence)))
}

fn located_change_sequences(root: &Path) -> Result<Vec<LocatedChangeSequence>, String> {
    let mut located = Vec::new();
    for (base, archived) in [
        (root.join(CHANGES_PATH), false),
        (root.join(ARCHIVE_PATH), true),
    ] {
        let entries = match fs::read_dir(&base) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "failed to read {} changes: {error}",
                    if archived { "archived" } else { "active" }
                ));
            }
        };
        for entry in entries {
            let entry = entry.map_err(|error| format!("failed to read change entry: {error}"))?;
            if !entry
                .file_type()
                .map_err(|error| format!("failed to inspect change entry: {error}"))?
                .is_dir()
            {
                continue;
            }
            let state_path = entry.path().join("state.json");
            let content = match fs::read_to_string(&state_path) {
                Ok(content) => content,
                Err(error)
                    if archived
                        && error.kind() == std::io::ErrorKind::NotFound
                        && (is_positive_legacy_tombstone(&entry.path())
                            || is_untrackable_husk(&entry.path())) =>
                {
                    continue;
                }
                // Git cannot track empty directories, so checking out a branch
                // without this change leaves a husk behind — typically just an
                // empty `deltas/`. A husk is not an active change. Treating one
                // as corrupt made `change new` fail on an ordinary checkout.
                // A directory that holds a file but no `state.json` is damaged
                // and must not be skipped.
                Err(error)
                    if !archived
                        && error.kind() == std::io::ErrorKind::NotFound
                        && is_untrackable_husk(&entry.path()) =>
                {
                    continue;
                }
                Err(error) => {
                    return Err(format!(
                        "failed to read {} change state {}: {error}",
                        if archived { "archived" } else { "active" },
                        state_path.display()
                    ));
                }
            };
            let record: ChangeRecord = serde_json::from_str(&content).map_err(|error| {
                format!(
                    "invalid {} change state {}: {error}",
                    if archived { "archived" } else { "active" },
                    state_path.display()
                )
            })?;
            let expected_id = if archived {
                record.id.clone()
            } else {
                entry.file_name().into_string().map_err(|_| {
                    format!(
                        "active change directory is not valid UTF-8: {}",
                        entry.path().display()
                    )
                })?
            };
            validate_loaded_change(&record, &expected_id, &state_path)?;
            let sequence = located_change_ordinal(&record.id)?;
            // Retired is immutable terminal nonacceptance: like Accepted and
            // Archived it freezes the sequence its ledger recorded, so an
            // acknowledged collision stays acknowledged after a member is
            // retired.
            let historical = matches!(
                record.state,
                ChangeState::Accepted | ChangeState::Archived | ChangeState::Retired
            ) || reopened_change_preserves_sequence_history(root, &record);
            located.push(LocatedChangeSequence {
                sequence,
                id: record.id,
                path: portable_project_path(root, &entry.path()),
                historical,
            });
        }
    }
    Ok(located)
}

fn reopened_change_preserves_sequence_history(root: &Path, record: &ChangeRecord) -> bool {
    if record.state != ChangeState::Verifying || !record.canonical_applied {
        return false;
    }
    let Ok(approvals) = load_approvals(root, record) else {
        return false;
    };
    let Some(reopening) = approvals.reopenings.last() else {
        return false;
    };
    if reopening.schema_version != 1
        || reopening.change_id != record.id
        || reopening.actor.trim().is_empty()
        || reopening.reason.trim().is_empty()
        // `reopen_change` reopens out of the archive as well as out of Accepted,
        // so a preserved-sequence reopening may carry either origin.
        || !matches!(
            reopening.from_state,
            ChangeState::Accepted | ChangeState::Archived
        )
        || reopening.to_state != ChangeState::Verifying
        || !reopening.prior_verification.passed
        || (reopening.stale_acceptance_input_digest == reopening.current_acceptance_input_digest
            && !matches!(
                reopening.stale_evidence_cause,
                Some(ReopenCauseV1::VerificationCommitUnanchored)
            )
            && !(reopening.stale_evidence_cause
                == Some(ReopenCauseV1::LegacyAcceptanceUnreconstructible)
                && record.workflow_version < 2
                && reopening.prior_verification.acceptance_manifest.is_none()))
        || reopening
            .prior_verification
            .acceptance_input_digest
            .as_deref()
            != Some(reopening.stale_acceptance_input_digest.as_str())
        || !definition_digest_matches(root, record, &reopening.prior_verification.contract_digest)
            .unwrap_or(false)
        || validate_verification_execution_digest(root, record, &reopening.prior_verification)
            .is_err()
        || !is_terminal_approval(&reopening.superseded_approval)
        || reopening.superseded_approval.digest
            != closing_digest(record, &reopening.prior_verification)
    {
        return false;
    }
    if let Some(manifest) = &reopening.prior_verification.acceptance_manifest
        && acceptance_manifest_digest(manifest).ok().as_deref()
            != Some(reopening.stale_acceptance_input_digest.as_str())
    {
        return false;
    }
    approvals.approvals.iter().any(|approval| {
        approval.gate == reopening.superseded_approval.gate
            && approval.actor == reopening.superseded_approval.actor
            && approval.timestamp == reopening.superseded_approval.timestamp
            && approval.digest == reopening.superseded_approval.digest
            && approval.note == reopening.superseded_approval.note
    })
}

/// Project-wide next_action when the sequence ledger is frozen (invalid acknowledgements,
/// mutable multi-id acks, or unacknowledged duplicates). Returns `None` when the ledger is healthy.
pub(crate) fn sequence_ledger_freeze_next_action(root: &Path) -> Option<String> {
    match validate_change_sequences(root) {
        Ok(()) => None,
        Err(reason)
            if reason.contains("includes a mutable change")
                || reason.contains("only immutable accepted or archived") =>
        {
            // Premature multi-id sequence acknowledgements freeze change new/adopt until every
            // collision member is accepted/archived (or the acknowledgement is corrected).
            Some(
                "accept or archive every member of the acknowledged sequence collision (or remove the premature acknowledgement from `.specsync/change-sequence.json`), then re-run `specsync change status`"
                    .into(),
            )
        }
        Err(reason)
            if reason.contains("no longer matches the exact historical ID set")
                || reason.contains("must contain at least two unique IDs")
                || reason.contains("duplicate acknowledgements for") =>
        {
            Some(format!(
                "fix or remove the invalid acknowledgement in `.specsync/change-sequence.json` ({reason}), then re-run `specsync change status`"
            ))
        }
        Err(reason) if reason.contains("duplicate numeric change sequence") => Some(format!(
            "{reason}; acknowledge the historical collision in `{SEQUENCE_PATH}` once every member is accepted or archived, then re-run `specsync change status`"
        )),
        // Everything else the gate can now report is a hand-edit, a bad merge, or a duplicate
        // identity on a frozen file. Returning `None` here reported a healthy next action
        // while `change new` and `change audit` were both refusing, which is how a bricked
        // repository looked fine in `change status`.
        Err(reason) => Some(format!(
            "resolve the change sequence ledger problem ({reason}), then re-run `specsync change status`"
        )),
    }
}

fn validate_change_sequences(root: &Path) -> Result<(), String> {
    let located = located_change_sequences(root)?;
    let ledger = load_change_sequence_ledger(root)?;
    // Two workspaces claiming one identity. The ordinal used to make this impossible by
    // construction and the numeric gate below caught it as a side effect; a slug-only ID is
    // unique only by convention, and two clones can archive the same slug on different days
    // into differently dated directories that git merges without a conflict. This is the
    // gate that used to be implicit, made explicit — it must not go through the ordinal,
    // because the IDs that need it no longer have one.
    //
    // It has to live here rather than in `list_all_changes_uncached`, which already refuses
    // the same shape: `change audit` runs with `include_archive_integrity = false` and never
    // loads the archive at all, so on a repository with no active change nothing else looks.
    let mut seen: BTreeMap<&str, &LocatedChangeSequence> = BTreeMap::new();
    for change in &located {
        if let Some(previous) = seen.insert(change.id.as_str(), change) {
            return Err(format!(
                "duplicate change ID `{}`: {} and {}; a change ID identifies exactly one workspace, so one of these packages must be removed or re-identified",
                change.id, previous.path, change.path
            ));
        }
    }
    let mut groups: BTreeMap<u64, Vec<&LocatedChangeSequence>> = BTreeMap::new();
    for change in &located {
        if let Some(sequence) = change.sequence {
            groups.entry(sequence).or_default().push(change);
        }
    }
    for changes in groups.values_mut() {
        changes.sort_by(|left, right| left.id.cmp(&right.id));
    }
    if let Some(ledger) = &ledger {
        let mut acknowledged_sequences = BTreeSet::new();
        for collision in &ledger.acknowledged_collisions {
            if !acknowledged_sequences.insert(collision.sequence) {
                return Err(format!(
                    "change sequence ledger contains duplicate acknowledgements for CHG-{:04}",
                    collision.sequence
                ));
            }
            let mut expected = collision.ids.clone();
            expected.sort();
            expected.dedup();
            let actual = groups
                .get(&collision.sequence)
                .map(|changes| {
                    changes
                        .iter()
                        .map(|change| change.id.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if expected.len() < 2 || expected.len() != collision.ids.len() {
                return Err(format!(
                    "acknowledged collision CHG-{:04} must contain at least two unique IDs",
                    collision.sequence
                ));
            }
            if expected != actual {
                return Err(format!(
                    "acknowledged collision CHG-{:04} no longer matches the exact historical ID set: expected [{}], found [{}]",
                    collision.sequence,
                    expected.join(", "),
                    actual.join(", ")
                ));
            }
            if groups
                .get(&collision.sequence)
                .is_some_and(|changes| changes.iter().any(|change| !change.historical))
            {
                return Err(format!(
                    "acknowledged collision CHG-{:04} includes a mutable change; only immutable accepted or archived collisions can be acknowledged",
                    collision.sequence
                ));
            }
        }
    }
    for (sequence, changes) in groups.iter().filter(|(_, changes)| changes.len() > 1) {
        let ids: Vec<String> = changes.iter().map(|change| change.id.clone()).collect();
        let acknowledged = ledger.as_ref().is_some_and(|ledger| {
            ledger.acknowledged_collisions.iter().any(|collision| {
                let mut expected = collision.ids.clone();
                expected.sort();
                collision.sequence == *sequence && expected == ids
            })
        });
        if !acknowledged {
            let conflicts = changes
                .iter()
                .map(|change| format!("{} ({})", change.id, change.path))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "duplicate numeric change sequence CHG-{sequence:04}: {conflicts}; acknowledge the historical collision in `{SEQUENCE_PATH}` once every member is accepted or archived. A change created now claims no ordinal, so recreating one of these is no longer a way out of a collision between two that already have one."
            ));
        }
    }
    if let Some(ledger) = ledger {
        let maximum = located
            .iter()
            .filter_map(|change| change.sequence)
            .max()
            .unwrap_or(0);
        // Disk must never run ahead of the ledger high-water mark.
        if maximum > ledger.sequence {
            return Err(format!(
                "change sequence ledger claims CHG-{:04} but the highest recorded sequence is CHG-{maximum:04}; \
restore it with `git checkout HEAD -- {SEQUENCE_PATH}` (nothing allocates a sequence any more, so this cannot be repaired by minting a higher one)",
                ledger.sequence
            ));
        }
        // Nor may the ledger fall below the highest mark this branch itself has
        // already recorded. Asked of the branch's OWN history, because that is
        // the only history it is accountable for: a branch behind origin has
        // recorded nothing higher and is innocent, while a branch that raised
        // the ledger and then rewrote it downwards is guilty regardless of
        // where it diverged.
        if let Some(recorded) = branch_sequence_high_water(root)
            && recorded > ledger.sequence
        {
            return Err(format!(
                "change sequence ledger claims CHG-{:04} but this branch already recorded CHG-{recorded:04}; \
restore it with `git checkout HEAD -- {SEQUENCE_PATH}` before continuing",
                ledger.sequence
            ));
        }
        let claim_present = located.iter().any(|change| change.id == ledger.id);
        if !claim_present {
            // Allow high-water claims after an abandoned draft (workspace removed
            // but sequence not rolled back). Multi-agent cleanup and aborted
            // `change new` paths hit this case.
            if ledger.sequence > maximum {
                return Ok(());
            }
            return Err(format!(
                "change sequence ledger claim `{}` has no active or archived workspace",
                ledger.id
            ));
        }
    }
    Ok(())
}

pub fn next_questions(record: &ChangeRecord) -> Vec<InterviewQuestion> {
    let mut questions = Vec::new();
    if record.acceptance_criteria.is_empty() {
        questions.push(InterviewQuestion {
            id: "acceptance_criteria".into(),
            prompt: "What observable outcomes prove this change is complete?".into(),
            choices: Vec::new(),
            recommended: None,
        });
    }
    if record.affected_specs.is_empty() && !record.no_spec_change {
        questions.push(InterviewQuestion {
            id: "affected_specs".into(),
            prompt: "Which canonical spec modules does this change affect?".into(),
            choices: Vec::new(),
            recommended: None,
        });
    }
    if record.affected_paths.is_empty() {
        questions.push(InterviewQuestion {
            id: "affected_paths".into(),
            prompt: "Which source, test, documentation, or configuration paths are in scope?"
                .into(),
            choices: Vec::new(),
            recommended: None,
        });
    }
    if !record.answers.contains_key("public_contract") {
        questions.push(InterviewQuestion {
            id: "public_contract".into(),
            prompt: "Does this change alter public behavior or an API?".into(),
            choices: vec!["yes".into(), "no".into()],
            recommended: Some("no".into()),
        });
    }
    if !record.answers.contains_key("architecture_risk") {
        questions.push(InterviewQuestion {
            id: "architecture_risk".into(),
            prompt: "Does this change affect architecture, persisted data, security, or multiple modules?".into(),
            choices: vec!["yes".into(), "no".into()],
            recommended: Some("no".into()),
        });
    }
    questions
}

/// Interview questions, including the draft-only constraints question.
///
/// [`next_questions`] stays the record-only list. This one looks at the working tree, and only
/// after the rest of the interview is answered, so a historical approval that never recorded
/// `constraints` is not sent back to the interview.
pub(crate) fn next_questions_for(root: &Path, record: &ChangeRecord) -> Vec<InterviewQuestion> {
    let mut questions = next_questions(record);
    if questions.is_empty()
        && let Some(question) = constraints_question(root, record)
    {
        questions.push(question);
    }
    questions
}

fn constraints_question(root: &Path, record: &ChangeRecord) -> Option<InterviewQuestion> {
    if record.state != ChangeState::Draft || record.answers.contains_key("constraints") {
        return None;
    }
    if !constraints_apply(root, record) {
        return None;
    }
    Some(InterviewQuestion {
        id: "constraints".into(),
        prompt:
            "Which existing lessons or principles constrain this change? Answer none if none apply."
                .into(),
        choices: vec!["none".into()],
        recommended: Some("none".into()),
    })
}

fn constraints_apply(root: &Path, record: &ChangeRecord) -> bool {
    if !accumulated_lessons(root, &record.affected_specs).is_empty() {
        return true;
    }
    load_policy(root)
        .and_then(|policy| policy.principles_file)
        .is_some()
}

/// Answer one deterministic interview question after validating the existing definition ledger.
#[allow(dead_code)]
pub fn answer_question(
    root: &Path,
    id: &str,
    question: &str,
    answer: &str,
) -> Result<ChangeRecord, String> {
    answer_question_with_snapshot(root, id, question, answer).map(|result| result.change)
}

/// A comma-separated answer for a list that may legitimately be empty: `none` clears it, and an
/// answer that parses to no value is refused rather than silently clearing the list (#426).
fn clearable_list_answer(
    root: &Path,
    question: &str,
    answer: &str,
) -> Result<Vec<String>, String> {
    // `[]` can name no change or module, so it always clears.
    if answer.trim() == "[]" {
        return Ok(Vec::new());
    }
    if answer.trim() == "none" {
        // A change or spec module may itself be called `none`; clearing the list would then
        // silently drop what the answer meant to name (#1026 review).
        let named = match question {
            "dependencies" => find_change_dir(root, "none").is_ok(),
            _ => {
                let specs_dir = crate::config::load_config(root).specs_dir;
                known_module_names(root, &specs_dir).iter().any(|name| name == "none")
            }
        };
        if named {
            return Err(format!(
                "{question} answer `none` is ambiguous because something is named `none`: answer `none,` to name it, or answer the entries to keep; or answer `'[]'` to clear it"
            ));
        }
        return Ok(Vec::new());
    }
    let values = split_values(answer);
    if values.is_empty() {
        return Err(format!(
            "{question} answer names no value; answer `'[]'` to clear it"
        ));
    }
    Ok(values)
}

/// A yes/no interview answer as `yes` or `no`. The spellings the correction ledger accepts
/// (`y`, `true`, `1`, `n`, `false`, `0`, any case, padded) are stored as the choice they name;
/// anything else is refused naming the choices before it is stored, where it used to fail later
/// as a corrupt correction ledger (#1026 review).
fn yes_no_answer(question: &str, answer: &str) -> Result<String, String> {
    let trimmed = answer.trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "yes" | "y" | "true" | "1" => Ok("yes".into()),
        "no" | "n" | "false" | "0" => Ok("no".into()),
        _ => Err(format!(
            "{question} answer `{}` is not one of its choices; answer yes or no",
            trimmed.escape_debug()
        )),
    }
}

/// Every interview question id `change answer` accepts. `next_questions` and
/// `next_questions_for` advertise a subset of these (`dependencies` is answerable
/// but never asked); a unit test keeps every advertised id in this list, so a
/// question added in one place cannot drift from the other (#426).
const INTERVIEW_QUESTION_IDS: &[&str] = &[
    "acceptance_criteria",
    "affected_specs",
    "affected_paths",
    "dependencies",
    "public_contract",
    "architecture_risk",
    "constraints",
];

pub(crate) fn answer_question_with_snapshot(
    root: &Path,
    id: &str,
    question: &str,
    answer: &str,
) -> Result<DefinitionMutationResult, String> {
    let _lock = acquire_project_lock(root)?;
    let (mut record, corrections) = load_change_for_definition_mutation(root, id)?;
    require_state(&record, &[ChangeState::Draft], "answer interview questions")?;
    if !INTERVIEW_QUESTION_IDS.contains(&question) {
        return Err(format!(
            "unknown interview question `{question}`; expected one of: {}",
            INTERVIEW_QUESTION_IDS.join(", ")
        ));
    }
    if answer.trim().is_empty() {
        return Err(if question == "constraints" {
            "constraints answer must not be empty; answer none if none apply".into()
        } else {
            format!("{question} answer must not be empty")
        });
    }
    match question {
        "acceptance_criteria" => {
            let criteria = acceptance_criteria_values(answer)?;
            if criteria.is_empty() {
                return Err("acceptance_criteria answer names no criterion".into());
            }
            record.acceptance_criteria = criteria;
        }
        "affected_specs" => {
            let values = clearable_list_answer(root, question, answer)?;
            for module in &values {
                crate::commands::validate_module_name(module)
                    .map_err(|error| format!("invalid affected spec: {error}"))?;
            }
            ensure_affected_specs_exist(root, &values).map_err(|error| {
                // `None` or `NONE` is a module name, not the clearing keyword (#1026 review).
                if values.iter().any(|module| module.eq_ignore_ascii_case("none")) {
                    format!("{error}; to clear the list, answer exactly `none` or `'[]'`")
                } else {
                    error
                }
            })?;
            record.affected_specs = values;
            // The lessons those specs carry may have changed, so the previous citation is stale.
            record.answers.remove("constraints");
        }
        "affected_paths" => {
            if answer.trim() == "[]" {
                return Err("affected_paths answer `[]` names no path; a change needs at least one path in scope".into());
            }
            let values = split_values(answer);
            if values.is_empty() {
                return Err("affected_paths answer names no path".into());
            }
            let affected_paths: Vec<String> = values
                .iter()
                .map(|path| {
                    normalize_project_path(path)
                        .map_err(|error| format!("invalid affected path: {error}"))
                })
                .collect::<Result<_, _>>()?;
            record.affected_paths = affected_paths;
        }
        "dependencies" => {
            let dependencies = clearable_list_answer(root, question, answer)?;
            for dependency in &dependencies {
                if dependency == id || dependency_reaches(root, dependency, id, &mut BTreeSet::new()) {
                    return Err("dependency rescope would create a change cycle".into());
                }
                load_change(root, dependency)?;
            }
            record.dependencies = dependencies;
            record.dependencies.sort();
            record.dependencies.dedup();
        }
        "public_contract" => {
            let answer = yes_no_answer(question, answer)?;
            record.answers.insert(question.into(), answer.clone());
            if is_yes(&answer) {
                add_artifact(&mut record, ArtifactKind::Requirements);
                add_artifact(&mut record, ArtifactKind::Docs);
            }
        }
        "architecture_risk" => {
            let answer = yes_no_answer(question, answer)?;
            record.answers.insert(question.into(), answer.clone());
            if is_yes(&answer) {
                add_artifact(&mut record, ArtifactKind::Research);
                add_artifact(&mut record, ArtifactKind::Design);
                add_artifact(&mut record, ArtifactKind::Plan);
                add_artifact(&mut record, ArtifactKind::Tasks);
                add_artifact(&mut record, ArtifactKind::Testing);
            }
        }
        "constraints" => {
            let answer = answer.trim();
            if answer.is_empty() {
                return Err(
                    "constraints answer must not be empty; answer none if none apply".into(),
                );
            }
            record.answers.insert(question.into(), answer.to_string());
        }
        _ => {
            return Err(format!(
                "unknown interview question `{question}`; expected one of: {}",
                INTERVIEW_QUESTION_IDS.join(", ")
            ));
        }
    }
    // Every answer validates the list; the per-change overlap is re-judged when the answer is
    // the one that declares paths (REQ-change-115). Nothing is saved on refusal.
    let verification_ignored = validated_verification_ignored_paths(root)?;
    if question == "affected_paths" {
        ensure_affected_paths_clear_of_verification_ignored(
            &verification_ignored,
            &record.affected_paths,
            Some(&record.id),
        )?;
    }
    record.updated_at = now();
    let effective_definition = validate_definition_mutation(&record, &corrections)?;
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    ensure_artifact_files(root, &record)?;
    Ok(definition_mutation_result(
        root,
        record,
        effective_definition,
        corrections,
    ))
}

/// Add an ordering dependency after validating the existing definition ledger.
#[allow(dead_code)]
pub fn add_dependency(root: &Path, id: &str, dependency: &str) -> Result<ChangeRecord, String> {
    add_dependency_with_snapshot(root, id, dependency).map(|result| result.change)
}

pub(crate) fn add_dependency_with_snapshot(
    root: &Path,
    id: &str,
    dependency: &str,
) -> Result<DefinitionMutationResult, String> {
    let _lock = acquire_project_lock(root)?;
    let (mut record, corrections) = load_change_for_definition_mutation(root, id)?;
    require_state(
        &record,
        &[
            ChangeState::Draft,
            ChangeState::Approved,
            ChangeState::Implementing,
            ChangeState::Verifying,
        ],
        "add a change dependency",
    )?;
    if id == dependency {
        return Err("a change cannot depend on itself".into());
    }
    let _dependency_record = load_change(root, dependency)?;
    if dependency_reaches(root, dependency, id, &mut BTreeSet::new()) {
        return Err(format!(
            "adding dependency `{dependency}` would create a change cycle"
        ));
    }
    if !record.dependencies.iter().any(|value| value == dependency) {
        record.dependencies.push(dependency.to_string());
        record.dependencies.sort();
    }
    record.updated_at = now();
    let effective_definition = validate_definition_mutation(&record, &corrections)?;
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(definition_mutation_result(
        root,
        record,
        effective_definition,
        corrections,
    ))
}

/// Add one exact semantic-succession obligation after validating the definition ledger.
#[allow(dead_code)]
pub fn add_supersedes_obligation(
    root: &Path,
    id: &str,
    predecessor: &str,
    path: &str,
    module: &str,
    predecessor_entry_digest: &str,
) -> Result<ChangeRecord, String> {
    add_supersedes_obligation_with_snapshot(
        root,
        id,
        predecessor,
        path,
        module,
        Some(predecessor_entry_digest),
    )
    .map(|result| result.change)
}

pub(crate) fn add_supersedes_obligation_with_snapshot(
    root: &Path,
    id: &str,
    predecessor: &str,
    path: &str,
    module: &str,
    predecessor_entry_digest: Option<&str>,
) -> Result<DefinitionMutationResult, String> {
    let predecessor_entry_digest = predecessor_entry_digest
        .ok_or_else(|| "a predecessor entry digest is required".to_string())?;
    let _lock = acquire_project_lock(root)?;
    let (mut record, corrections) = load_change_for_definition_mutation(root, id)?;
    require_state(
        &record,
        &[ChangeState::Draft],
        "adopt a predecessor obligation",
    )?;
    if id == predecessor {
        return Err("a change cannot supersede itself".into());
    }
    crate::commands::validate_module_name(module)
        .map_err(|error| format!("invalid succession module: {error}"))?;
    let path = normalize_project_path(path)
        .map_err(|error| format!("invalid succession path: {error}"))?;
    validate_sha256_digest(predecessor_entry_digest, "predecessor entry digest")?;
    if !record
        .affected_specs
        .iter()
        .any(|affected| affected == module)
    {
        return Err(format!(
            "successor `{id}` must declare affected module `{module}`"
        ));
    }
    let obligation = SuccessionObligation {
        path,
        module: module.to_string(),
        predecessor_entry_digest: predecessor_entry_digest.to_string(),
    };
    let edge = if let Some(edge) = record
        .supersedes
        .iter_mut()
        .find(|edge| edge.predecessor_id == predecessor)
    {
        edge
    } else {
        record.supersedes.push(SupersedesEdge {
            predecessor_id: predecessor.to_string(),
            obligations: Vec::new(),
        });
        record
            .supersedes
            .last_mut()
            .ok_or_else(|| "failed to create supersedes edge".to_string())?
    };
    if edge
        .obligations
        .iter()
        .any(|existing| existing.path == obligation.path && existing.module == obligation.module)
    {
        return Err(format!(
            "supersedes obligation already exists for `{predecessor}` `{}` `{module}`",
            obligation.path
        ));
    }
    edge.obligations.push(obligation);
    edge.obligations.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.module.cmp(&right.module))
    });
    // Lexicographic, to agree with `approved_scope`, which sorts the same list by
    // `predecessor_id.cmp` and hashes the result into `scope_digest`. The previous numeric
    // key agreed with it only while every ordinal was four digits: at five they invert
    // (`CHG-9999 < CHG-10000` numerically, `CHG-10000 < CHG-9999` lexicographically), so
    // `approved_scope` would emit an order `validate_supersedes_edges` then rejected.
    record
        .supersedes
        .sort_by(|left, right| left.predecessor_id.cmp(&right.predecessor_id));
    validate_supersedes_edges(&record)?;
    validate_supersedes_semantics(root, &record)?;
    record.updated_at = now();
    let effective_definition = validate_definition_mutation(&record, &corrections)?;
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(definition_mutation_result(
        root,
        record,
        effective_definition,
        corrections,
    ))
}

/// Not yet: always no warning (#423, red).
pub fn succession_base_warning(
    _root: &Path,
    _successor_id: &str,
    _predecessor: &str,
    _path: &str,
) -> Result<Option<String>, String> {
    Ok(None)
}

/// Load one existing definition only after its caller has acquired the project lock.
///
/// Answer, dependency, and supersession mutations share this path so correction-ledger
/// validation and persistence are one serialized transaction. The fixed diagnostic keeps
/// correction values, ledger bytes, and digests out of human command output.
fn load_change_for_definition_mutation(
    root: &Path,
    id: &str,
) -> Result<(ChangeRecord, Vec<CorrectionRecord>), String> {
    let record = load_change(root, id)?;
    let ledger = load_correction_ledger(root, &record)
        .map_err(|_| INVALID_CORRECTION_LEDGER_TEXT.to_string())?;
    validate_correction_records(&record, &ledger.corrections)
        .map_err(|_| INVALID_CORRECTION_LEDGER_TEXT.to_string())?;
    Ok((record, ledger.corrections))
}

fn validate_definition_mutation(
    change: &ChangeRecord,
    corrections: &[CorrectionRecord],
) -> Result<EffectiveChangeDefinition, String> {
    validate_correction_records(change, corrections)
        .map_err(|_| INVALID_CORRECTION_LEDGER_TEXT.to_string())
}

/// Build every machine-facing mutation projection before releasing the project lock.
fn definition_mutation_result(
    root: &Path,
    change: ChangeRecord,
    effective_definition: EffectiveChangeDefinition,
    corrections: Vec<CorrectionRecord>,
) -> DefinitionMutationResult {
    // One delta-binding read for every projection below, all under the caller's project lock.
    let _delta_bindings = begin_delta_binding_memo(root);
    let summary =
        summarize_change_with_effective(root, &change, false, Some(&effective_definition));
    let strict_summary =
        summarize_change_with_effective(root, &change, true, Some(&effective_definition));
    let approved_deltas = approved_delta_summaries(root, &change);
    DefinitionMutationResult {
        change,
        effective_definition,
        corrections,
        summary,
        strict_summary,
        approved_deltas,
    }
}

pub fn approve_definition(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    approve_definition_with_relay(root, id, actor, note, false, None)
}

pub fn approve_definition_portable_v501(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    approve_definition_with_relay(root, id, actor, note, true, None)
}

/// Record a definition approval, or the portable 5.0.1 pair, carrying an optional relayed
/// decision (#633). The relay must name the approval's explicit actor as the one who decided,
/// and is refused before anything is written otherwise.
pub fn approve_definition_with_relay(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
    portable_v501: bool,
    relayed_by: Option<RelayedDecision>,
) -> Result<ChangeRecord, String> {
    // The recorded actor is trimmed (#426), so the relay's decider is trimmed the same way:
    // otherwise `actor` and `relayed_by.decided_by` differ by padding and every later read of the
    // ledger refuses the mismatch (#1026 review).
    let relayed_by = relayed_by.map(|mut relay| {
        relay.decided_by = relay.decided_by.trim().to_string();
        relay
    });
    if relayed_by.is_some() {
        let Some(decider) = actor.as_deref() else {
            return Err(
                "a relayed decision names who decided with --actor <who decided>; without it the \
                 actor comes from Git or $USER, which names the recording agent"
                    .into(),
            );
        };
        ensure_relay_names_the_decider(relayed_by.as_ref(), decider.trim(), "--actor")?;
    }
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    ensure_no_sequence_collision(root, &record)?;
    // Accepted records may re-approve the definition when the artifact digest drifted
    // (reopen recovery). Refuse no-op re-approvals that would only rewrite an already-valid
    // definition approval while the change is accepted.
    let allow_accepted_definition_refresh = record.state == ChangeState::Accepted
        && ensure_definition_approval_valid(root, &record).is_err();
    if allow_accepted_definition_refresh {
        // Fall through after the dedicated accepted check.
    } else {
        require_state(
            &record,
            &[
                ChangeState::Draft,
                ChangeState::Approved,
                ChangeState::Implementing,
                ChangeState::Verifying,
            ],
            "approve the definition",
        )?;
    }
    list_changes_checked(root)?;
    bind_legacy_archive_baseline_authority(root, &mut record)?;
    let prior_state = record.state;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    validate_declared_path_ownership(root, &record)?;
    ensure_change_clear_of_verification_ignored(root, &record)?;
    // A workflow-v2 change is raised to the relay-gated version, and `state.json` written,
    // before the relay is, so an interrupted approve never leaves a relay that SpecSync 6.2.0
    // could drop (REQ-change-158). A workflow-v1 change keeps its version and says so below.
    let relayed_on_workflow_v1 = relayed_by.is_some() && record.workflow_version < 2;
    if relayed_by.is_some() && raise_for_relayed_decision(&mut record) {
        save_change(root, &record)?;
    }
    if portable_v501 {
        append_portable_definition_approval_v501_with_relay(root, &record, actor, note, relayed_by)?;
    } else {
        let digest = definition_digest(root, &record)?;
        append_approval_with_relay(root, &record, "definition", actor, digest, note, relayed_by)?;
    }
    record.state = match prior_state {
        ChangeState::Draft | ChangeState::Approved => ChangeState::Approved,
        ChangeState::Verifying if record.canonical_applied => ChangeState::Verifying,
        ChangeState::Implementing | ChangeState::Verifying => ChangeState::Implementing,
        ChangeState::Accepted | ChangeState::Archived => prior_state,
        ChangeState::Retired => return Err("retirement is immutable nonacceptance".into()),
    };
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    if relayed_on_workflow_v1 {
        eprintln!("{}", ungated_relay_note(&record.id));
    }
    Ok(record)
}

fn bind_legacy_archive_baseline_authority(
    root: &Path,
    record: &mut ChangeRecord,
) -> Result<(), String> {
    let path = root.join(LEGACY_BASELINE_PATH);
    let candidates = BTreeSet::from([LEGACY_BASELINE_PATH.to_string()]);
    let evidence = git_regular_file_evidence(root, &candidates)?;
    let baseline_entry = evidence.entry(LEGACY_BASELINE_PATH)?;
    if baseline_entry.kind == AcceptanceInputKind::Missing {
        if record.legacy_archive_baseline_digest.is_some() {
            return Err(
                "legacy archive baseline is missing or dirty; stale binding cannot be retained"
                    .into(),
            );
        }
        if record
            .affected_paths
            .iter()
            .any(|affected| affected == LEGACY_BASELINE_PATH)
        {
            return Err("legacy archive baseline authority cannot bind a missing ledger".into());
        }
        return Ok(());
    }
    if baseline_entry.kind != AcceptanceInputKind::File
        || !matches!(baseline_entry.mode, 0o100644 | 0o100755)
    {
        return Err(format!(
            "legacy archive baseline is not a regular file: {}",
            path.display()
        ));
    }
    let baseline_bytes = baseline_entry.payload.clone();
    let (baseline, digest) = validate_legacy_archive_baseline_bytes(&baseline_bytes)?;
    if baseline.authority_change_id == record.id {
        if !record
            .affected_paths
            .iter()
            .any(|path| path == LEGACY_BASELINE_PATH)
        {
            return Err(format!(
                "legacy archive baseline authority must cover `{LEGACY_BASELINE_PATH}`"
            ));
        }
        validate_legacy_baseline_authority_cutoff(root, record, &baseline.cutoff_commit)?;
        record.legacy_archive_baseline_digest = Some(digest);
    }
    Ok(())
}

fn validate_legacy_baseline_authority_cutoff(
    root: &Path,
    authority: &ChangeRecord,
    cutoff: &str,
) -> Result<(), String> {
    let resolved = git_output(
        root,
        &["rev-parse", "--verify", &format!("{cutoff}^{{commit}}")],
    )
    .ok_or_else(|| "legacy archive baseline cutoff is unavailable".to_string())?;
    if resolved != cutoff {
        return Err("legacy archive baseline cutoff must be a canonical commit ID".into());
    }
    if authority.base_commit.as_deref() != Some(cutoff) {
        return Err(
            "legacy archive baseline cutoff must equal the authority definition base commit".into(),
        );
    }
    ensure_git_ancestor(root, cutoff, "HEAD", "current authority history")
}

pub fn start_implementation(root: &Path, id: &str) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Approved, ChangeState::Implementing],
        "start implementation",
    )?;
    ensure_definition_approval_valid(root, &record)?;
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    if record.state == ChangeState::Implementing {
        return Ok(record);
    }
    record.state = ChangeState::Implementing;
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(record)
}

/// Materialization as a plain `change check <id>` runs it. Production callers name their own
/// invocation through [`materialize_change_deltas_rerunning`]; the unit tests use this.
#[cfg(test)]
fn materialize_change_deltas(root: &Path, id: &str) -> Result<ChangeRecord, String> {
    materialize_change_deltas_rerunning(root, id, &CheckInvocation::default().command(id))
}

/// Whether the latest audited reopening still freezes canonical content: a send-back at or
/// after it superseded the reopening by re-scoping the change (Codex review of #1018).
fn latest_reopening_still_freezes_canonical(ledger: &ApprovalLedger) -> bool {
    ledger.reopenings.last().is_some_and(|reopening| {
        !ledger
            .send_backs
            .iter()
            .any(|sent| sent.timestamp >= reopening.timestamp)
    })
}

/// [`materialize_change_deltas`], naming `rerun` as the command to re-run when open tasks
/// refuse it (REQ-change-131).
fn materialize_change_deltas_rerunning(
    root: &Path,
    id: &str,
    rerun: &str,
) -> Result<ChangeRecord, String> {
    materialize_change_deltas_inner(root, id, rerun).map_err(|error| {
        let frozen = load_change(root, id).is_ok_and(|record| {
            record.canonical_applied
                && load_approvals(root, &record)
                    .is_ok_and(|ledger| latest_reopening_still_freezes_canonical(&ledger))
        });
        if frozen && !error.contains("frozen audited-reopened") {
            let detail = error.split(';').next().unwrap_or(&error);
            format!("frozen audited-reopened change {id}: {detail}; preserve the approved definition and later canonical text; use a separately approved current-intent replacement and audited nonacceptance retirement, or hold while authenticated custody is unavailable")
        } else { error }
    })
}

fn materialize_change_deltas_inner(
    root: &Path,
    id: &str,
    rerun: &str,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[
            ChangeState::Approved,
            ChangeState::Implementing,
            ChangeState::Verifying,
        ],
        "check the change",
    )?;
    ensure_definition_approval_valid(root, &record)?;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    // Deliberately ABOVE the `canonical_applied` short-circuit. Once the deltas are applied this
    // function stops writing specs, so a check placed below would never see a swap on any run
    // after the first — and the workspace would keep shipping a delta that no longer describes
    // the spec it produced. The delta must match its approval for as long as it is evidence.
    ensure_approved_delta_bodies_unchanged(root, &record)?;
    // Also above the short-circuit, and before anything is written: a body approved over text
    // the base branch has since rewritten must be refused, never applied over it (#843).
    ensure_approved_delta_bases_current(root, &record)?;
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    if let Err(error) = ensure_tasks_complete(root, &record, rerun) {
        // #847: a refusal before verification still leaves a record of itself.
        let _ = record_refused_attempt(root, &record, &error);
        return Err(error);
    }
    // `canonical_applied` records that materialization RAN, never that it ran for the delta on
    // disk now. Re-approving a corrected delta satisfies every gate above — the guard directly
    // above asks whether the delta matches its approval, and a new approval signs the new body —
    // and then the bare flag returned here, skipping the applied delta, the version bump and the
    // Change Log row alike (#741). So the outstanding work is computed from the canonical tree
    // itself and the short-circuit is conditional on there being none.
    let prepared = prepare_pending_delta_application(root, &record)?;
    if record.canonical_applied && prepared.pending.is_empty() {
        return Ok(record);
    }
    if is_ci_project(root) {
        return Err(format!(
            "the canonical specs do not carry the approved semantic deltas ({}); run `specsync change check` locally and commit the result before CI",
            prepared
                .pending
                .iter()
                .map(|module| format!("`{module}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let mut prepared = prepared.files;
    record.canonical_applied = true;
    record.state = ChangeState::Implementing;
    record.updated_at = now();
    let state_path = change_dir(root, &record.id).join("state.json");
    let state = json_content_keeping_unknown_members(
        &state_path,
        relay_gated_record(&change_dir(root, &record.id), &record).as_ref(),
    )?;
    prepared.push((state_path, state));
    prepared.push((
        change_dir(root, &record.id).join("change.md"),
        change_markdown_content(&record),
    ));
    write_prepared_files(root, &prepared)?;
    Ok(record)
}

pub fn check_change(root: &Path, id: Option<&str>) -> Result<Option<VerificationRecord>, String> {
    check_change_with_strict(root, id, false)
}

pub fn check_change_with_strict(
    root: &Path,
    id: Option<&str>,
    strict: bool,
) -> Result<Option<VerificationRecord>, String> {
    check_change_as(
        root,
        id,
        CheckInvocation {
            strict,
            ..CheckInvocation::default()
        },
    )
}

/// The flags of one `change check` invocation, so a refusal can name exactly that command to
/// re-run (REQ-change-131). Only `strict` changes what the check does; `commit` and `push` are
/// carried out by the command layer and appear here only so the named command keeps them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CheckInvocation {
    /// `--strict` was given.
    pub strict: bool,
    /// `--commit` was given.
    pub commit: bool,
    /// `--push` was given.
    pub push: bool,
}

impl CheckInvocation {
    /// The command line that repeats this invocation for one change.
    fn command(&self, id: &str) -> String {
        let mut command = format!("specsync change check {id}");
        for (given, flag) in [
            (self.strict, " --strict"),
            (self.commit, " --commit"),
            (self.push, " --push"),
        ] {
            if given {
                command.push_str(flag);
            }
        }
        command
    }
}

/// [`check_change_with_strict`] for a known invocation: a refusal names that invocation's own
/// command line as the one to re-run.
pub fn check_change_as(
    root: &Path,
    id: Option<&str>,
    invocation: CheckInvocation,
) -> Result<Option<VerificationRecord>, String> {
    let selected = if let Some(id) = id {
        Some(id.to_string())
    } else {
        let candidates: Vec<String> = list_changes_checked(root)?
            .into_iter()
            .filter(|record| {
                matches!(
                    record.state,
                    ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
                )
            })
            .map(|record| record.id)
            .collect();
        match candidates.as_slice() {
            [] => None,
            [id] => Some(id.clone()),
            _ => {
                return Err(format!(
                    "multiple changes need checking; pass one ID: {}",
                    candidates.join(", ")
                ));
            }
        }
    };
    let Some(id) = selected else {
        return Ok(None);
    };
    // Refuse before materialization writes canonical specs (REQ-change-115).
    ensure_change_clear_of_verification_ignored(root, &load_change(root, &id)?)?;
    let rerun = invocation.command(&id);
    materialize_change_deltas_rerunning(root, &id, &rerun)?;
    verify_change_rerunning(root, &id, invocation.strict, &rerun).map(Some)
}

pub fn verify_change(root: &Path, id: &str) -> Result<VerificationRecord, String> {
    verify_change_with_strict(root, id, false)
}

pub fn verify_change_with_strict(
    root: &Path,
    id: &str,
    strict: bool,
) -> Result<VerificationRecord, String> {
    let strict_flag = if strict { " --strict" } else { "" };
    verify_change_rerunning(
        root,
        id,
        strict,
        &format!("specsync change verify {id}{strict_flag}"),
    )
}

/// Verification naming `rerun` as the command to re-run when open tasks refuse it.
fn verify_change_rerunning(
    root: &Path,
    id: &str,
    strict: bool,
    rerun: &str,
) -> Result<VerificationRecord, String> {
    if let Some(error) = crate::verification_recursion_error() {
        return Err(error);
    }
    let _lock = acquire_project_lock(root)?;
    verify_change_locked(root, id, strict, rerun)
}

/// Verification body without lock acquisition.
///
/// The caller MUST already hold the project lock. Acceptance re-records evidence
/// while holding its own lock, and `acquire_project_lock` is a non-reentrant
/// `flock`, so calling the public entry point from there would block forever
/// rather than fail.
fn verify_change_locked(
    root: &Path,
    id: &str,
    strict: bool,
    rerun: &str,
) -> Result<VerificationRecord, String> {
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Implementing, ChangeState::Verifying],
        "verify the change",
    )?;
    ensure_change_clear_of_verification_ignored(root, &record)?;
    ensure_definition_approval_valid(root, &record)?;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    ensure_no_orphaned_applied_text(root, &record)?;
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    // The gate judges the change being verified as verifying, the state this attempt records
    // (REQ-change-112). On the first check `materialize_change_deltas` has just set
    // `canonical_applied` and `Implementing`, and the gate's roster filter used to drop exactly
    // that record, so the one run that records a pass was the one run the gate did not ask.
    let mut records = list_changes_checked(root)?;
    for candidate in &mut records {
        if candidate.id == record.id {
            candidate.state = ChangeState::Verifying;
        }
    }
    // Verification has no non-error output channel; `specsync check` and
    // `specsync change audit` run this same gate and report the suppressions.
    // A verdict about a module this change declares is a spec↔code verdict about this change,
    // so it is held and recorded as part of the attempt below. Every other gate failure is
    // refused without running the gate, and the refusal records itself as a `passed: false`
    // attempt carrying its reason, so a reader can see the check was tried and why it failed
    // (#847).
    let gate = validate_effective_contracts(root, &records);
    let (gate_refusals, unattributed) = gate.split_for_declared(&record.affected_specs);
    if !unattributed.is_empty() {
        // A gate failure about another change's module is not a verdict about this change,
        // so unlike the workspace-grounded refusals below it records no attempt (#847 keeps
        // the control at semantic_application_resolves_registry_backed_canonical_paths.rs).
        return Err(gate.error_text().unwrap_or_else(|| unattributed.join("; ")));
    }
    if let Err(error) = ensure_tasks_complete(root, &record, rerun) {
        let _ = record_refused_attempt(root, &record, &error);
        return Err(error);
    }
    // Evidence completeness is markdown in the change workspace. Fail it before
    // the spec↔code pass so a missing testing.md row is not a 18-minute suite.
    let requirement_ids = collect_requirement_ids(root, &record)?;
    let has_semantic_acceptance_item = semantic_acceptance_item_exists(root, &record)?;
    let missing_evidence = requirement_evidence_missing(root, &record, &requirement_ids);
    let acceptance_evidence_present =
        acceptance_criteria_have_evidence(&record, has_semantic_acceptance_item);
    if !acceptance_evidence_present || !missing_evidence.is_empty() {
        let error = format!(
            "verification cannot start: {}",
            evidence_gap_detail(&record, acceptance_evidence_present, &missing_evidence)
        );
        let _ = record_refused_attempt(root, &record, &error);
        return Err(error);
    }
    let (sync, mut findings) = evaluate_spec_code_sync(root, &record, strict)?;
    let mut commands = vec![sync];
    if !gate_refusals.is_empty() {
        // Named for the command that reproduces the gate's verdict, not as a `--strict` run:
        // the gate suppresses unauthored stubs and replays other active deltas, which
        // `specsync check --strict` does not.
        let refusal_texts: Vec<String> = gate_refusals.values().flatten().cloned().collect();
        commands.push(CommandEvidence {
            command: format!(
                "specsync change check {} (effective contract: {})",
                record.id,
                gate_refusals.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
            success: false,
            exit_code: Some(1),
            output: bounded_failure_output(&refusal_texts),
        });
        findings.splice(0..0, refusal_texts);
    }
    let passed = commands.iter().all(|command| command.success);
    let verification = VerificationRecord {
        timestamp: now(),
        commit: git_output(root, &["rev-parse", "HEAD"]),
        contract_digest: definition_digest(root, &record)?,
        execution_digest: (record.workflow_version >= 2)
            .then(|| execution_digest(root, &record))
            .transpose()?,
        workspace_digest: project_input_digest(root)?,
        acceptance_input_digest: None,
        acceptance_manifest: None,
        semantic_succession: None,
        passed,
        commands,
        requirement_ids,
        refusal: None,
    };
    record_verification_attempt(root, &record, &verification)?;
    record.state = ChangeState::Verifying;
    record.updated_at = now();
    save_change(root, &record)?;
    if !verification.passed {
        let summary = if findings.is_empty() {
            "specs are out of sync with code".to_string()
        } else {
            findings.into_iter().take(5).collect::<Vec<_>>().join("; ")
        };
        return Err(format!(
            "verification failed: {summary}; see commands[] in {}",
            portable_project_path(
                root,
                &change_dir(root, &record.id).join("verification.json")
            )
        ));
    }
    Ok(verification)
}

/// Names the artifact and section an author must edit to close an evidence gap.
///
/// The gap is always described by the change workspace rather than by
/// `verification.json`, so pointing at the record itself sends authors to a file
/// that cannot contain the answer.
fn evidence_gap_detail(
    record: &ChangeRecord,
    acceptance_evidence_present: bool,
    missing_evidence: &[String],
) -> String {
    let testing = format!(".specsync/changes/{}/testing.md", record.id);
    if !acceptance_evidence_present {
        return format!(
            "semantic acceptance evidence is missing; record how each acceptance criterion is \
             proven in {testing}"
        );
    }
    format!(
        "no requirement evidence for {}; add a row for {} to the `## Requirement evidence` table in \
         {testing}",
        missing_evidence.join(", "),
        if missing_evidence.len() == 1 {
            "it"
        } else {
            "each"
        }
    )
}

/// The specs one change owns, and the module names that name them.
///
/// Scope is the union of two claims the change already made at approval: the
/// modules in `affected_specs`, and any spec whose `files:` mapping falls inside
/// a declared `affected_paths` scope. The second half is not redundant — a
/// `--no-spec-change` delivery declares no module at all, and the specs mapping
/// its source are still the contracts it can break. Without it that change
/// would verify against nothing.
///
/// Project-wide validation is `specsync check`. `change check` is scoped by
/// construction, so drift in a module this change does not own is not this
/// change's failure to fix.
struct ScopedSpecs {
    /// Spec files this change is verified against.
    files: Vec<PathBuf>,
    /// `--spec` arguments naming that set, as `filter_specs` matches them.
    filters: Vec<String>,
    /// Declared modules with no spec file on disk. A scoping FAILURE, never an
    /// empty scope — see `evaluate_spec_code_sync`.
    unresolved: Vec<String>,
}

/// The conventional spec path `specsync new <module>` writes.
fn specs_dir_conventional_path(specs_dir: &str, module: &str) -> String {
    format!("{}/{module}/{module}.spec.md", specs_dir.trim_end_matches('/'))
}

/// How to create a missing spec at `expected`: `specsync new` only when it would write exactly
/// that path and accept the name; otherwise, create the spec at that path.
fn scaffold_hint(conventional: &str, expected: &str, module: &str) -> String {
    if conventional == expected && crate::commands::validate_scaffold_module_name(module).is_ok() {
        format!("scaffold it first with `specsync new {}`", shell_word(module))
    } else {
        format!("create its spec at `{expected}` first")
    }
}

/// Refuse affected-spec declarations that name no spec on disk, whether the
/// module is registered or conventional — a change cannot affect a spec that
/// does not exist, and the typo surfaces at declaration, not at
/// materialization. A project with no specs at all cannot typo against an
/// empty set, so the declaration is provisional there (bootstrap).
fn ensure_affected_specs_exist(root: &Path, modules: &[String]) -> Result<(), String> {
    if modules.is_empty() {
        return Ok(());
    }
    let config = crate::config::load_config(root);
    // An unreadable registry is reported, never read as "no modules" (which would skip this
    // check entirely in a project whose specs all live in the registry).
    let registry = crate::registry::load_local_registry(root)
        .map_err(|error| format!("the spec registry cannot be read: {error}"))?;
    let spec_files = crate::validator::find_spec_files(&root.join(&config.specs_dir));
    // Bootstrap, exactly as REQ-change-150 says: a project with no spec files and no registry
    // entries cannot typo against an empty set, so the declaration is provisional there.
    if spec_files.is_empty() && registry.as_ref().is_none_or(|registry| registry.specs.is_empty()) {
        return Ok(());
    }
    // One rule: a module is accepted exactly when it is declarable, the same set the error
    // names as known modules and the owner recovery checks (#1026 review).
    let known = known_module_names(root, &config.specs_dir);
    for module in modules {
        if module_has_canonical_spec(root, &config.specs_dir, module) {
            continue;
        }
        let (spec_path, _) = canonical_module_paths(root, &config.specs_dir, module)
            .map_err(|error| format!("affected spec `{module}` cannot be resolved: {error}"))?;
        let expected = portable_project_path(root, &spec_path);
        if spec_path.is_file() && path_traverses_symlink(root, &spec_path) {
            return Err(format!(
                "affected spec `{module}` is reached through a symlink at `{expected}`, and SpecSync does not follow symlinks into specs; put the spec there as a regular file"
            ));
        }
        if let Some(actual) = known.iter().find(|name| name.eq_ignore_ascii_case(module)) {
            return Err(format!(
                "affected spec `{module}` is not a module; module names are case-sensitive, so name it as `{actual}`"
            ));
        }
        let registered = registry
            .as_ref()
            .is_some_and(|registry| registry.specs.iter().any(|(name, _)| name == module));
        if registered {
            return Err(format!(
                "affected spec `{module}` is registered at `{expected}`, but no spec exists there; create the spec at that path or fix its registry entry"
            ));
        }
        // A spec that declares this module somewhere else is moved or registered, never
        // duplicated by a scaffold at the canonical path.
        if let Some(elsewhere) = spec_files.iter().find(|spec| {
            fs::read_to_string(spec)
                .ok()
                .and_then(|content| crate::parser::parse_frontmatter(&content))
                .and_then(|parsed| parsed.frontmatter.module)
                .is_some_and(|declared| &declared == module)
        }) {
            return Err(format!(
                "affected spec `{module}` has a spec at `{}`, but not at its canonical path `{expected}`; move it there or register it in the spec registry",
                portable_project_path(root, elsewhere)
            ));
        }
        let known_clause = if known.is_empty() {
            String::new()
        } else {
            format!(", or name one of the known modules: {}", known.join(", "))
        };
        return Err(format!(
            "affected spec `{module}` has no spec at `{expected}`; {}{known_clause}",
            scaffold_hint(&specs_dir_conventional_path(&config.specs_dir, module), &expected, module),
        ));
    }
    Ok(())
}

/// Whether `module` has a spec at its canonical path: the one rule `change answer
/// affected_specs`, the owner hints and the printed owner answer share (#1026 review).
///
/// - A spec reached through a symlink is not one: verification refuses to traverse it.
/// - A conventional path must spell the module's own directory and file exactly. A case variant
///   that a case-insensitive disk resolves fails on a case-sensitive one, so an entry differing
///   only in ASCII case does not count. The specs directory above it, a registered path, and a
///   difference in Unicode normalization (macOS matches either form) are taken as the disk
///   resolves them.
fn module_has_canonical_spec(root: &Path, specs_dir: &str, module: &str) -> bool {
    let Ok((spec_path, _)) = canonical_module_paths(root, specs_dir, module) else {
        return false;
    };
    if !spec_path.is_file() || path_traverses_symlink(root, &spec_path) {
        return false;
    }
    let registered = matches!(
        crate::registry::load_local_registry(root),
        Ok(Some(registry)) if registry.specs.iter().any(|(name, _)| name == module)
    );
    registered
        || spec_path
            .parent()
            .is_none_or(|directory| entry_is_not_an_ascii_case_variant(directory) && entry_is_not_an_ascii_case_variant(&spec_path))
}

/// Whether some component of `path` below `root` is a symlink.
fn path_traverses_symlink(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let mut current = root.to_path_buf();
    relative.components().any(|component| {
        current.push(component);
        fs::symlink_metadata(&current).is_ok_and(|metadata| metadata.file_type().is_symlink())
    })
}

/// False when `path`'s directory holds an entry differing from its name only in ASCII case and
/// none spelled exactly: the disk resolved a case variant.
fn entry_is_not_an_ascii_case_variant(path: &Path) -> bool {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return true;
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return true;
    };
    let names: Vec<std::ffi::OsString> = entries.flatten().map(|entry| entry.file_name()).collect();
    names.iter().any(|entry| entry == name)
        || !names.iter().any(|entry| {
            entry
                .to_str()
                .zip(name.to_str())
                .is_some_and(|(entry, name)| entry != name && entry.eq_ignore_ascii_case(name))
        })
}

/// Names of every known spec module — registry entries plus modules discovered
/// under the specs directory — for error messages that name the valid set.
fn known_module_names(root: &Path, specs_dir: &str) -> Vec<String> {
    // Only names `affected_specs` would accept: a registry name whose spec exists, or a
    // discovered spec whose stem is a module with that spec at its canonical path. A file stem
    // that is not a module (`main` for a registry entry's `main.spec.md`) is not listed.
    let mut candidates: BTreeSet<String> = BTreeSet::new();
    if let Ok(Some(registry)) = crate::registry::load_local_registry(root) {
        for (module, _) in &registry.specs {
            candidates.insert(module.clone());
        }
    }
    for spec in crate::validator::find_spec_files(&root.join(specs_dir)) {
        let filter = spec_filter_name(&spec);
        if !filter.is_empty() {
            candidates.insert(filter);
        }
    }
    candidates
        .into_iter()
        .filter(|module| module_has_canonical_spec(root, specs_dir, module))
        .collect()
}

/// The `--spec` argument that selects this file.
///
/// `filter_specs` matches a filter against the file stem with a trailing
/// `.spec` removed (`src/commands/mod.rs`), never against the frontmatter
/// `module:` field. Deriving the name any other way records a command that
/// selects nothing on a spec whose declared module and filename differ — a
/// registry-mapped module most of all, where the two routinely differ.
fn spec_filter_name(spec: &Path) -> String {
    spec.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.strip_suffix(".spec").unwrap_or(stem).to_string())
        .unwrap_or_default()
}

fn scoped_spec_files(
    root: &Path,
    record: &ChangeRecord,
    config: &crate::types::SpecSyncConfig,
) -> ScopedSpecs {
    // Declared modules resolve through the local registry first, so a registered
    // non-conventional spec path is in scope exactly as a conventional one is.
    let mut selected: BTreeMap<PathBuf, String> = BTreeMap::new();
    let mut unresolved = Vec::new();
    // A declared module naming a spec that is not on disk is not "nothing to
    // check". Dropping it silently let a change declare a contract, delete or
    // never write its spec, and verify green against zero of the modules it
    // named — while other specs pulled in by path scope made the pass look real.
    let mut unresolved_filters = Vec::new();
    for module in &record.affected_specs {
        match canonical_module_paths(root, &config.specs_dir, module) {
            Ok((spec_path, _)) if spec_path.is_file() => {
                let filter = spec_filter_name(&spec_path);
                selected.insert(spec_path, filter);
            }
            Ok((spec_path, _)) => {
                unresolved.push(format!(
                    "declared module `{module}` has no spec at {}",
                    portable_project_path(root, &spec_path)
                ));
                unresolved_filters.push(module.clone());
            }
            Err(error) => {
                unresolved.push(format!(
                    "declared module `{module}` cannot be resolved: {error}"
                ));
                unresolved_filters.push(module.clone());
            }
        }
    }
    for spec in crate::validator::find_spec_files(&root.join(&config.specs_dir)) {
        if selected.contains_key(&spec) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&spec) else {
            continue;
        };
        let normalized = content.replace("\r\n", "\n");
        let Some(parsed) = crate::parser::parse_frontmatter(&normalized) else {
            continue;
        };
        let owns_a_declared_path = parsed.frontmatter.files.iter().any(|file| {
            let file = file.replace('\\', "/");
            record
                .affected_paths
                .iter()
                .any(|scope| path_matches_scope(&file, scope))
        });
        if !owns_a_declared_path {
            continue;
        }
        let filter = spec_filter_name(&spec);
        selected.insert(spec, filter);
    }
    // An unresolved module keeps its declared name in the filter list. That is
    // the only place the persisted record of a passing check names it: a check's
    // findings reach `verification.json` only as the bounded `output` of a failed
    // command, so without this a missing-spec scope would not say which module it named.
    //
    // What that string does NOT promise is a faithful rerun in every case.
    // `filter_specs` demotes an unmatched filter to a stderr warning as soon as
    // any other filter matches, and `check`'s exit-1 gate fires only when the
    // matched set is empty. So a scope with one module resolved and one not
    // fails HERE and can still exit 0 on rerun. Reproduction is exact when every
    // named spec resolves, or when none does; the authoritative record of a
    // mixed-scope failure is this verdict, not the rerun.
    let mut filters: BTreeSet<String> = selected.values().cloned().collect();
    filters.extend(unresolved_filters);
    ScopedSpecs {
        files: selected.into_keys().collect(),
        filters: filters.into_iter().collect(),
        unresolved,
    }
}

/// The scoped command this verdict was reached under.
///
/// `--spec` is matched against the file stem, so the recorded string names the
/// same scoped set rather than the project-wide one. It reproduces the verdict
/// exactly when every named spec resolves, or when none does — see the note in
/// `scoped_spec_files` for the mixed case, which fails here but can rerun
/// green. An empty scope is named as such instead of being written as a bare
/// `specsync check`, which would claim a project-wide pass that never
/// happened.
fn scoped_check_command(modules: &[String], strict: bool) -> String {
    if modules.is_empty() {
        return "specsync check (no spec in scope)".to_string();
    }
    let mut command = String::from("specsync check");
    for module in modules {
        command.push_str(" --spec ");
        command.push_str(module);
    }
    if strict {
        command.push_str(" --strict");
    }
    command
}

/// Compare this change's specs to source. Does not spawn a process, does not
/// build, does not run the project's tests. `sdd.json` `verification_commands`
/// are not executed.
fn evaluate_spec_code_sync(
    root: &Path,
    record: &ChangeRecord,
    strict: bool,
) -> Result<(CommandEvidence, Vec<String>), String> {
    let config = crate::config::load_config(root);
    let scope = scoped_spec_files(root, record, &config);
    let schema_tables = crate::validator::get_schema_table_names(root, &config);
    // The shared helper `validate_effective_contracts` already uses; building the
    // snapshot twice by hand is how the two drift apart.
    let schema_columns = crate::commands::build_schema_columns(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::load(root);
    let mut findings = Vec::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    // Fail before validating anything else, so the message names the module the
    // author has to write rather than burying it under findings from the specs
    // that happened to resolve.
    errors += scope.unresolved.len();
    findings.extend(scope.unresolved.iter().cloned());
    // What failed the check: errors always, warnings only when --strict makes them failures.
    // The recorded `output` is built from these, so a long run of warnings cannot crowd the
    // error that failed a non-strict check out of the bounded tail (#1027 review).
    let mut failing: Vec<String> = scope.unresolved.clone();
    let mut warning_lines: Vec<String> = Vec::new();
    for spec in &scope.files {
        let result =
            crate::validator::validate_spec(spec, root, &schema_tables, &schema_columns, &config);
        let inline_ignores = fs::read_to_string(spec)
            .map(|content| crate::ignore::IgnoreRules::parse_inline(&content))
            .unwrap_or_default();
        errors += result.errors.len();
        failing.extend(result.errors.iter().cloned());
        findings.extend(result.errors);
        for warning in result.warnings {
            if ignore_rules.is_suppressed(&warning, &result.spec_path, &inline_ignores) {
                continue;
            }
            warnings += 1;
            let line = format!("warning: {warning}");
            warning_lines.push(line.clone());
            findings.push(line);
        }
    }
    let failed = errors > 0 || (strict && warnings > 0);
    if strict {
        failing.extend(warning_lines);
    }
    let output = if failed {
        bounded_failure_output(&failing)
    } else {
        None
    };
    Ok((
        CommandEvidence {
            command: scoped_check_command(&scope.filters, strict),
            success: !failed,
            exit_code: Some(i32::from(failed)),
            output,
        },
        findings,
    ))
}

#[cfg_attr(not(test), allow(dead_code))]
fn verification_commands_for_change(
    root: &Path,
    policy: &SddPolicy,
    record: &ChangeRecord,
    explicit_strict: bool,
) -> Result<Vec<String>, String> {
    let routing = load_verification_routing(root)?;
    let mut commands = Vec::new();
    // A declared module with no component entry is not a module that needs no
    // verification — it is a module nobody routed. Tracking those separately is
    // what keeps this monotonic: previously any single routed module made
    // `commands` non-empty, which suppressed the project-wide list for the whole
    // change, so `--spec routed --spec unrouted` verified LESS than `--spec
    // unrouted` alone. Declaring scope honestly must never cost coverage.
    let mut unrouted_modules = Vec::new();
    for module in &record.affected_specs {
        match routing.component_verification_commands.get(module) {
            Some(component) => commands.extend(component.iter().cloned()),
            None => unrouted_modules.push(module.as_str()),
        }
    }
    if commands.is_empty() || !unrouted_modules.is_empty() {
        commands.extend(policy.verification_commands.iter().cloned());
    }
    let strict_required = explicit_strict || change_requires_strict_validation(record, &routing);
    if strict_required {
        if routing.strict_verification_commands.is_empty() {
            commands.extend(policy.verification_commands.iter().cloned());
        } else {
            commands.extend(routing.strict_verification_commands);
        }
    }
    let mut seen = BTreeSet::new();
    commands.retain(|command| seen.insert(command.clone()));
    if commands.is_empty() {
        return Err(
            "no verification commands are configured for this change; add a component command or a bounded project fallback in .specsync/sdd.json"
                .into(),
        );
    }
    Ok(commands)
}

fn load_verification_routing(root: &Path) -> Result<VerificationRouting, String> {
    let path = root.join(POLICY_PATH);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(VerificationRouting::default());
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid SDD policy {}: {error}", path.display()))
}

fn change_requires_strict_validation(record: &ChangeRecord, routing: &VerificationRouting) -> bool {
    let default_strict_paths = [
        "src/change.rs",
        "src/mcp.rs",
        "src/github.rs",
        ".github/workflows/release.yml",
        ".github/workflows/post-merge-archive.yml",
    ];
    record.affected_specs.iter().any(|module| {
        matches!(
            module.as_str(),
            "change" | "mcp" | "github" | "authentication" | "security"
        )
    }) || record.affected_paths.iter().any(|path| {
        default_strict_paths
            .into_iter()
            .chain(routing.strict_paths.iter().map(String::as_str))
            .any(|strict_path| {
                path_matches_scope(path, strict_path) || path_matches_scope(strict_path, path)
            })
    })
}

pub fn reopen_change(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
) -> Result<ReopenResult, String> {
    // Refused before anything moves: an archived package is not relocated for a bad actor.
    let actor = normalize_actor(&actor, "reopen")?;
    let _lock = acquire_project_lock(root)?;
    let record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Accepted, ChangeState::Archived],
        "reopen accepted evidence",
    )?;
    // `finalize` performs accept and archive in one command, so `Accepted` is never
    // observable and a change is Archived by the time anyone needs to recover it.
    // Reopen therefore has to un-archive first: the rest of this reopen writes to
    // the *active* directory, so reopening in place would leave two directories
    // claiming one change ID in disagreeing states.
    //
    // The origin is audited rather than silent: the ReopenRecord below carries
    // `from_state`, so a reopen out of the archive is distinguishable from a reopen
    // of a still-accepted change without inventing a new event type.
    let reopened_from = record.state;
    let unarchived_from = if record.state == ChangeState::Archived {
        Some(unarchive_change_workspace(root, &record)?)
    } else {
        None
    };
    let outcome = reopen_unarchived_change(root, id, reopened_from, actor, reason);
    // The un-archive above is a move performed *before* the reopen preconditions are
    // known to hold, so a correctly refused reopen would otherwise consume the archive:
    // the package would sit in the active workspace with an `archived` state.json, and
    // no verb puts it back. A refusal must leave the tree exactly as it was found, so
    // the move is undone on every failure, exactly as `archive_change` restores its own
    // source when post-move validation fails.
    let Some(archived_location) = unarchived_from else {
        return outcome;
    };
    let error = match outcome {
        Ok(result) => return Ok(result),
        Err(error) => error,
    };
    match rename_durable(&change_dir(root, id), &archived_location) {
        Ok(()) => Err(format!("{error}; archive restored")),
        Err(restore_error) => Err(format!(
            "{error}; and the un-archived package could not be restored to {} ({restore_error}); move it back by hand before retrying",
            portable_project_path(root, &archived_location)
        )),
    }
}

/// Move an archived change package out of the dated archive and back into the active
/// workspace, returning the archive location it came from so a failed reopen can put it
/// back. The caller owns that restore: this helper only performs the move.
fn unarchive_change_workspace(root: &Path, record: &ChangeRecord) -> Result<PathBuf, String> {
    let archived_location = find_change_dir(root, &record.id)?;
    let active = change_dir(root, &record.id);
    if active.exists() {
        return Err(format!(
            "cannot un-archive {}: an active change directory already exists at {}",
            record.id,
            portable_project_path(root, &active)
        ));
    }
    if let Some(parent) = active.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    rename_durable(&archived_location, &active)
        .map_err(|error| format!("failed to un-archive {}: {error}", record.id))?;
    Ok(archived_location)
}

/// Reopen a change whose workspace is already active. Every failure here is recoverable
/// by the caller, which restores the archive when it was the one that un-archived.
fn reopen_unarchived_change(
    root: &Path,
    id: &str,
    reopened_from: ChangeState,
    actor: String,
    reason: String,
) -> Result<ReopenResult, String> {
    let mut record = load_change(root, id)?;
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("reopen requires a non-empty reason passed with --reason".into());
    }
    ensure_definition_approval_valid(root, &record).map_err(|error| {
        if error.contains("definition approval is stale") {
            format!(
                "{error}; run `specsync change approve {} --actor <name>` while the change is still accepted to refresh the definition digest, then reopen",
                record.id
            )
        } else {
            error
        }
    })?;
    let current_verification = load_verification(root, &record)?;
    let mut ledger = load_approvals(root, &record)?;
    let superseded_approval = latest_terminal_approval(&ledger).cloned().ok_or_else(|| {
        "accepted change is missing closing approval (acceptance or finalization)".to_string()
    })?;
    let prior_verification = verification_for_closing_approval(
        root,
        &record,
        &superseded_approval,
        &current_verification,
    )?;
    if !prior_verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    if !definition_digest_matches(root, &record, &prior_verification.contract_digest)? {
        return Err("accepted change verification contract is stale; restore the accepted definition before reopening delivery evidence, or run `specsync change approve` to refresh a drifted definition while accepted".into());
    }
    validate_verification_execution_digest(root, &record, &prior_verification)?;
    let stale_acceptance_input_digest = prior_verification
        .acceptance_input_digest
        .clone()
        .ok_or_else(|| "accepted change is missing current delivery-input evidence".to_string())?;
    let current_acceptance_input_digest =
        if let Some(manifest) = &prior_verification.acceptance_manifest {
            let current = acceptance_manifest_with_signed_owners(root, &record, &[], manifest)?;
            acceptance_manifest_digest(&current)?
        } else {
            acceptance_input_digest(root, &record, &[])?
        };
    let inputs_drifted = current_acceptance_input_digest != stale_acceptance_input_digest;
    let tip_matches_closing = current_verification.passed
        && superseded_approval.digest == closing_digest(&record, &current_verification);
    let mut recorded_review_mismatch_anchor = None;
    let anchored = if tip_matches_closing {
        match authenticate_accepted_evidence_with_anchor(root, &record, None) {
            Ok((_, anchored)) => anchored,
            Err(error)
                if error == "finalization scoped review does not bind the accepted verification" =>
            {
                recorded_review_mismatch_anchor = Some(recorded_review_workspace_mismatch_anchor(
                    root,
                    &record,
                    &current_verification,
                )?);
                true
            }
            Err(error) => return Err(error),
        }
    } else {
        accepted_evidence_is_anchored(root, &record, &prior_verification)
    };
    // Archive reconstructs manifest-less legacy inputs at acceptance transitions.
    // Matching today's inputs and an anchored commit do not prove that historical
    // reconstruction can succeed (#751). Failure permits an audited refresh, not
    // archival of the old evidence; every authentication check above still applies.
    let legacy_unreconstructible = record.workflow_version < 2
        && prior_verification.acceptance_manifest.is_none()
        && reconstruct_legacy_acceptance_manifest(root, &record, &stale_acceptance_input_digest)
            .is_err();
    if !inputs_drifted
        && anchored
        && !legacy_unreconstructible
        && recorded_review_mismatch_anchor.is_none()
    {
        return Err(
            "accepted change delivery inputs are current (exact or successor-covered) and its verification commit is still anchored in current history; reopen is allowed only when accepted evidence is stale"
                .into(),
        );
    }
    if tip_matches_closing
        && anchored
        && !legacy_unreconstructible
        && recorded_review_mismatch_anchor.is_none()
    {
        let records = list_all_changes_checked(root)?;
        let mut visiting = BTreeSet::new();
        let mut memo = AcceptedInputValidationMemo::default();
        if validate_accepted_inputs_recursive(
            root,
            &record,
            &records,
            &mut visiting,
            &mut memo,
            None,
        )
        .is_ok()
        {
            return Err(
                "accepted change delivery inputs are current (exact or successor-covered) and its verification commit is still anchored in current history; reopen is allowed only when accepted evidence is stale"
                    .into(),
            );
        }
    }
    let audit = ReopenRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: actor.to_string(),
        reason: reason.to_string(),
        timestamp: now(),
        from_state: reopened_from,
        to_state: ChangeState::Verifying,
        superseded_approval,
        prior_verification,
        stale_acceptance_input_digest,
        current_acceptance_input_digest,
        recorded_review_mismatch_anchor,
        stale_evidence_cause: if !anchored {
            Some(ReopenCauseV1::VerificationCommitUnanchored)
        } else if legacy_unreconstructible {
            Some(ReopenCauseV1::LegacyAcceptanceUnreconstructible)
        } else {
            None
        },
    };
    ledger.reopenings.push(audit.clone());
    record.state = ChangeState::Verifying;
    record.canonical_applied = true;
    record.updated_at = now();
    write_prepared_files(
        root,
        &[
            {
                let path = change_dir(root, &record.id).join("approvals.json");
                let content = json_content_keeping_unknown_members(&path, &ledger)?;
                (path, content)
            },
            {
                let workspace = change_dir(root, &record.id);
                let path = workspace.join("state.json");
                // A reopened change that holds a relay stays gated (REQ-change-158).
                let content = json_content_keeping_unknown_members(
                    &path,
                    relay_gated_record(&workspace, &record).as_ref(),
                )?;
                (path, content)
            },
            (
                change_dir(root, &record.id).join("change.md"),
                change_markdown_content(&record),
            ),
        ],
    )?;
    Ok(ReopenResult {
        change: record,
        audit,
    })
}

/// Outcome of a `migrate 5.0` ledger backfill: per-change repair, skip, and failure detail.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReopenBackfillReport {
    pub dry_run: bool,
    pub repaired: Vec<String>,
    pub unchanged: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl ReopenBackfillReport {
    pub fn is_clean(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Backfill the 5.1 reopening digest fields (`stale_acceptance_input_digest` /
/// `current_acceptance_input_digest`) on 5.0.1-era change ledgers. The repair is deterministic
/// and idempotent: `stale` always reproduces the embedded prior verification's signed digest,
/// and `current` comes from the superseding verification's signed digest when one exists, else
/// from a live recomputation over the current inputs. Repaired ledgers must re-parse and
/// re-validate before the write lands; a reopening that cannot be repaired deterministically
/// fails its change without mutating that ledger while other changes still migrate.
pub fn backfill_reopen_digests(root: &Path, dry_run: bool) -> Result<ReopenBackfillReport, String> {
    let mut report = ReopenBackfillReport {
        dry_run,
        ..Default::default()
    };
    for workspace in change_workspaces_for_backfill(root)? {
        let approvals_path = workspace.join("approvals.json");
        let Ok(content) = fs::read_to_string(&approvals_path) else {
            continue;
        };
        let id = workspace
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| workspace.display().to_string());
        if serde_json::from_str::<ApprovalLedger>(&content).is_ok() {
            report.unchanged.push(id);
            continue;
        }
        match backfill_change_ledger(root, &workspace, &content, dry_run)? {
            Ok(true) => report.repaired.push(id),
            Ok(false) => report.unchanged.push(id),
            Err(reason) => report.failed.push((id, reason)),
        }
    }
    report.repaired.sort();
    report.unchanged.sort();
    report.failed.sort();
    Ok(report)
}

fn change_workspaces_for_backfill(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut workspaces = Vec::new();
    for base in [root.join(CHANGES_PATH), root.join(ARCHIVE_PATH)] {
        let Ok(entries) = fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                workspaces.push(path);
            }
        }
    }
    workspaces.sort();
    Ok(workspaces)
}

/// Repairs one change's `approvals.json`, returning `Ok(true)` when reopenings were backfilled,
/// `Ok(false)` when nothing needed repair, or `Err(reason)` when a reopening cannot be repaired
/// deterministically. On `Err` or verification failure the ledger is left byte-identical.
fn backfill_change_ledger(
    root: &Path,
    workspace: &Path,
    content: &str,
    dry_run: bool,
) -> Result<Result<bool, String>, String> {
    let mut ledger: serde_json::Value = match serde_json::from_str(content) {
        Ok(ledger) => ledger,
        Err(error) => return Ok(Err(format!("approvals ledger is not valid JSON: {error}"))),
    };
    let Some(reopenings) = ledger
        .get_mut("reopenings")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(Ok(false));
    };
    let mut repaired = false;
    for reopening in reopenings.iter_mut() {
        let has_stale = reopening
            .get("stale_acceptance_input_digest")
            .and_then(serde_json::Value::as_str)
            .is_some();
        let has_current = reopening
            .get("current_acceptance_input_digest")
            .and_then(serde_json::Value::as_str)
            .is_some();
        if has_stale && has_current {
            continue;
        }
        let Some(stale) = reopening
            .get("prior_verification")
            .and_then(|verification| verification.get("acceptance_input_digest"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
        else {
            return Ok(Err(
                "reopening is missing its embedded prior verification acceptance-input digest"
                    .into(),
            ));
        };
        let reopen_timestamp = reopening
            .get("timestamp")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        let current = match superseding_acceptance_digest(workspace, reopen_timestamp)? {
            Some(digest) => digest,
            None => match live_acceptance_digest(root, workspace) {
                Ok(digest) => digest,
                Err(error) => {
                    return Ok(Err(format!(
                        "cannot recompute a current acceptance-input digest: {error}"
                    )));
                }
            },
        };
        if stale == current {
            return Ok(Err(
                "repaired digests are identical; the recorded drift cannot be proven".into(),
            ));
        }
        reopening["stale_acceptance_input_digest"] = serde_json::Value::String(stale);
        reopening["current_acceptance_input_digest"] = serde_json::Value::String(current);
        repaired = true;
    }
    if !repaired {
        return Ok(Ok(false));
    }
    let bytes = serde_json::to_vec_pretty(&ledger)
        .map_err(|error| format!("failed to encode repaired approvals ledger: {error}"))?;
    if let Err(error) = serde_json::from_slice::<ApprovalLedger>(&bytes) {
        return Ok(Err(format!(
            "repaired approvals ledger failed 5.1 schema verification: {error}"
        )));
    }
    if !dry_run {
        let mut bytes = bytes;
        bytes.push(b'\n');
        fs::write(workspace.join("approvals.json"), bytes)
            .map_err(|error| format!("failed to write repaired approvals ledger: {error}"))?;
    }
    Ok(Ok(true))
}

/// Returns the signed acceptance-input digest of the change's current verification record when
/// it supersedes the reopening (recorded after it), matching the rollout-proven repair.
fn superseding_acceptance_digest(
    workspace: &Path,
    reopen_timestamp: u64,
) -> Result<Option<String>, String> {
    let verification_path = workspace.join("verification.json");
    let Ok(content) = fs::read_to_string(&verification_path) else {
        return Ok(None);
    };
    let verification: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| format!("verification evidence is not valid JSON: {error}"))?;
    let timestamp = verification
        .get("timestamp")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_default();
    if timestamp < reopen_timestamp {
        return Ok(None);
    }
    Ok(verification
        .get("acceptance_input_digest")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string))
}

/// Recomputes the acceptance-input digest for a change that has no superseding verification,
/// using the same manifest-aware path as the 5.1 reopen flow.
fn live_acceptance_digest(root: &Path, workspace: &Path) -> Result<String, String> {
    let state: ChangeRecord = serde_json::from_str(
        &fs::read_to_string(workspace.join("state.json"))
            .map_err(|error| format!("failed to read change state: {error}"))?,
    )
    .map_err(|error| format!("change state is not valid: {error}"))?;
    let record = load_change(root, &state.id)?;
    let verification = load_verification(root, &record)?;
    if let Some(manifest) = &verification.acceptance_manifest {
        let current = acceptance_manifest_with_signed_owners(root, &record, &[], manifest)?;
        acceptance_manifest_digest(&current)
    } else {
        acceptance_input_digest(root, &record, &[])
    }
}

fn validate_acceptance_owner_correction_records(record: &ChangeRecord) -> Result<(), String> {
    if record.acceptance_owner_corrections.len() > MAX_ACCEPTANCE_OWNERS {
        return Err(format!(
            "acceptance owner corrections exceed {MAX_ACCEPTANCE_OWNERS} entries"
        ));
    }
    let mut exact_pairs = BTreeSet::new();
    for (index, correction) in record.acceptance_owner_corrections.iter().enumerate() {
        validate_acceptance_owner_correction_record(
            record,
            correction,
            index as u64 + 1,
            &mut exact_pairs,
        )?;
    }
    Ok(())
}

fn validate_acceptance_owner_correction_record(
    record: &ChangeRecord,
    correction: &AcceptanceOwnerCorrection,
    expected_sequence: u64,
    exact_pairs: &mut BTreeSet<(String, String)>,
) -> Result<(), String> {
    if correction.schema_version != 1 {
        return Err(format!(
            "unsupported acceptance owner correction schema {} at sequence {expected_sequence}",
            correction.schema_version
        ));
    }
    if correction.sequence != expected_sequence {
        return Err(format!(
            "acceptance owner correction sequence is not contiguous: expected {expected_sequence}, found {}",
            correction.sequence
        ));
    }
    if correction.path.len() > MAX_ACCEPTANCE_PATH_BYTES
        || strict_portable_relative_path(&correction.path)? != correction.path
    {
        return Err(format!(
            "invalid acceptance owner correction path `{}`",
            correction.path
        ));
    }
    if correction.module.len() > MAX_ACCEPTANCE_OWNER_BYTES
        || correction.module.starts_with("@exact:")
    {
        return Err(format!(
            "invalid acceptance owner correction module `{}`",
            correction.module
        ));
    }
    crate::commands::validate_module_name(&correction.module)
        .map_err(|error| format!("invalid acceptance owner correction module: {error}"))?;
    if correction.actor.trim().is_empty()
        || correction.actor.trim() != correction.actor
        || correction.reason.trim().is_empty()
        || correction.reason.trim() != correction.reason
    {
        return Err(format!(
            "acceptance owner correction sequence {expected_sequence} has a missing or non-canonical actor/reason"
        ));
    }
    if !record
        .affected_paths
        .iter()
        .any(|scope| path_matches_scope(&correction.path, scope))
    {
        return Err(format!(
            "acceptance owner correction path `{}` is outside the original affected path scope",
            correction.path
        ));
    }
    if record.affected_specs.contains(&correction.module) {
        return Err(format!(
            "acceptance owner `{}` is already represented by the original affected specs",
            correction.module
        ));
    }
    if !exact_pairs.insert((correction.path.clone(), correction.module.clone())) {
        return Err(format!(
            "duplicate acceptance owner correction `{}` for `{}`",
            correction.module, correction.path
        ));
    }
    Ok(())
}

fn canonical_module_source_paths(root: &Path, module: &str) -> Result<BTreeSet<String>, String> {
    record_test_canonical_module_query();
    let config = crate::config::load_config(root);
    let (spec_path, _) = canonical_module_paths(root, &config.specs_dir, module)?;
    let spec_relative = strict_portable_project_path(root, &spec_path)?;
    let candidates = BTreeSet::from([spec_relative.clone()]);
    let evidence = git_regular_file_evidence(root, &candidates)?;
    let entry = evidence.entry(&spec_relative)?;
    let content = String::from_utf8(entry.payload.clone()).map_err(|_| {
        format!("canonical spec for `{module}` is not valid UTF-8: {spec_relative}")
    })?;
    let parsed = crate::parser::parse_frontmatter(&content)
        .ok_or_else(|| format!("canonical spec for `{module}` has invalid frontmatter"))?;
    Ok(parsed
        .frontmatter
        .files
        .iter()
        .filter_map(|path| normalize_project_path(path).ok())
        .collect())
}

fn canonical_module_owns_cached_source_path(
    root: &Path,
    module: &str,
    relative: &str,
    owned_paths: &BTreeSet<String>,
) -> Result<(), String> {
    if !path_is_production_source(root, relative) {
        return Err(format!(
            "acceptance owner correction path `{relative}` is not production source"
        ));
    }
    let source_path = safe_project_path(root, relative)?;
    let metadata = fs::symlink_metadata(&source_path)
        .map_err(|_| format!("acceptance owner correction path `{relative}` does not exist"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "acceptance owner correction path `{relative}` must be a regular file"
        ));
    }
    if !owned_paths.contains(relative) {
        return Err(format!(
            "canonical module `{module}` does not own exact source path `{relative}`"
        ));
    }
    Ok(())
}

fn validate_acceptance_owner_corrections_current(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    validate_acceptance_owner_corrections_current_with_cache(root, record, &mut BTreeMap::new())
}

fn validate_acceptance_owner_corrections_current_with_cache(
    root: &Path,
    record: &ChangeRecord,
    owned_by_module: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), String> {
    validate_acceptance_owner_correction_records(record)?;
    for correction in &record.acceptance_owner_corrections {
        if !owned_by_module.contains_key(&correction.module) {
            owned_by_module.insert(
                correction.module.clone(),
                canonical_module_source_paths(root, &correction.module)?,
            );
        }
        canonical_module_owns_cached_source_path(
            root,
            &correction.module,
            &correction.path,
            &owned_by_module[&correction.module],
        )?;
    }
    Ok(())
}

fn latest_reopen_for_owner_correction<'a>(
    record: &ChangeRecord,
    approvals: &'a ApprovalLedger,
    corrections: &CorrectionLedger,
) -> Result<Option<&'a ReopenRecord>, String> {
    let Some(reopening) = approvals.reopenings.last() else {
        // A change can reach Verifying without ever closing — that is where one
        // waits for finalize. It has no reopen because nothing reopened it, and
        // demanding one closed every exit: finalize refuses the unowned path,
        // reopen accepts only Accepted/Archived, and declaring the owning module
        // would force a semantic delta for a spec the change does not alter.
        //
        // The reopen path proves the definition still matches *closing*
        // verification. A never-closed change has no closing snapshot; the caller
        // instead requires a currently valid definition approval. That is the
        // necessary substitute for reachability, not audit-equivalent provenance
        // to an Accepted→reopen cycle.
        return Ok(None);
    };
    // REQ-change-033 requires the target be "verifying through an audited reopen"
    // and does not constrain the origin state. Since `finalize` performs accept and
    // archive in one command, a change needing owner correction is Archived by the
    // time anyone reaches it, and its reopen legitimately records `from_state:
    // archived`. Accepting only `Accepted` here would make the requirement
    // unsatisfiable through the guided path.
    //
    // The substantive guarantees are unchanged and checked below: the reopen must
    // reference trusted closing evidence, and no metadata correction may follow it.
    if reopening.schema_version != 1
        || reopening.change_id != record.id
        || !matches!(
            reopening.from_state,
            ChangeState::Accepted | ChangeState::Archived
        )
        || reopening.to_state != ChangeState::Verifying
    {
        return Err("latest audited reopen event is invalid for owner correction".into());
    }
    let approval_position = |target: &ApprovalRecord| {
        approvals.approvals.iter().rposition(|approval| {
            approval.gate == target.gate
                && approval.actor == target.actor
                && approval.timestamp == target.timestamp
                && approval.digest == target.digest
                && approval.note == target.note
        })
    };
    let reopen_position = approval_position(&reopening.superseded_approval)
        .ok_or_else(|| "latest reopen does not reference trusted closing evidence".to_string())?;
    if corrections.corrections.last().is_some_and(|correction| {
        approval_position(&correction.superseded_closing_approval)
            .is_some_and(|position| position > reopen_position)
    }) {
        return Err(
            "acceptance owner correction requires delivery reopening after the latest metadata correction"
                .into(),
        );
    }
    Ok(Some(reopening))
}

#[allow(dead_code)] // Public one-entry wrapper; batch path is used by the CLI adapter.
pub fn add_acceptance_owner_correction(
    root: &Path,
    id: &str,
    path: String,
    module: String,
    actor: String,
    reason: String,
) -> Result<ChangeRecord, String> {
    add_acceptance_owner_corrections(root, id, vec![(path, module)], actor, reason)
}

/// Append one or more audited exact canonical owner corrections in a single transactional write.
///
/// Every `(path, module)` entry is validated independently against the same rules as a single
/// [`add_acceptance_owner_correction`]. If any entry is invalid, no corrections from the batch are
/// persisted.
pub fn add_acceptance_owner_corrections(
    root: &Path,
    id: &str,
    entries: Vec<(String, String)>,
    actor: String,
    reason: String,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Verifying],
        "correct an acceptance input owner",
    )?;
    if !record.canonical_applied {
        return Err("acceptance owner correction requires an already-applied change".into());
    }
    if entries.is_empty() {
        return Err(
            "acceptance owner correction batch requires at least one path/module entry".into(),
        );
    }
    let actor = normalize_actor(&actor, "acceptance owner correction")?;
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(
            "acceptance owner correction requires a non-empty reason passed with --reason".into(),
        );
    }

    let mut owned_by_module = BTreeMap::new();
    validate_acceptance_owner_corrections_current_with_cache(root, &record, &mut owned_by_module)?;
    let approvals = load_approvals(root, &record)?;
    let metadata_corrections = load_correction_ledger(root, &record)?;
    let reopening = latest_reopen_for_owner_correction(&record, &approvals, &metadata_corrections)?;
    let mut original = record.clone();
    original.acceptance_owner_corrections.clear();
    match reopening {
        Some(reopening) => {
            if !definition_digest_matches(
                root,
                &original,
                &reopening.prior_verification.contract_digest,
            )? {
                return Err(
                    "cannot correct an owner after the reopened definition changed; restore the accepted definition or use a successor change"
                        .into(),
                );
            }
        }
        None => {
            // Never closed: no reopened closing snapshot exists. Require a live
            // definition approval so corrections cannot run under a drifted def.
            // Weaker provenance than Accepted→reopen; necessary for the guided path.
            ensure_definition_approval_valid(root, &original)?;
        }
    }

    let timestamp = now();
    let mut provisional = record.clone();
    let mut exact_pairs = provisional
        .acceptance_owner_corrections
        .iter()
        .map(|correction| (correction.path.clone(), correction.module.clone()))
        .collect();
    for (index, (path, module)) in entries.iter().enumerate() {
        let path = path.trim();
        let module = module.trim();
        if path.is_empty() || module.is_empty() {
            return Err(format!(
                "acceptance owner correction batch entry {} has an empty path or module",
                index + 1
            ));
        }
        if strict_portable_relative_path(path)? != path {
            return Err(format!(
                "acceptance owner correction path is not canonical: `{path}`"
            ));
        }
        crate::commands::validate_module_name(module)
            .map_err(|error| format!("invalid acceptance owner correction module: {error}"))?;
        if module.starts_with("@exact:") {
            return Err("reserved exact owners cannot be added through correct-owner".into());
        }
        let correction = AcceptanceOwnerCorrection {
            schema_version: 1,
            sequence: provisional.acceptance_owner_corrections.len() as u64 + 1,
            path: path.to_string(),
            module: module.to_string(),
            actor: actor.to_string(),
            reason: reason.to_string(),
            timestamp,
        };
        validate_acceptance_owner_correction_record(
            &provisional,
            &correction,
            correction.sequence,
            &mut exact_pairs,
        )
        .map_err(|error| {
            format!(
                "acceptance owner correction batch entry {} failed: {error}",
                index + 1
            )
        })?;
        if !owned_by_module.contains_key(module) {
            let owned = canonical_module_source_paths(root, module).map_err(|error| {
                format!(
                    "acceptance owner correction batch entry {} failed: {error}",
                    index + 1
                )
            })?;
            owned_by_module.insert(module.to_string(), owned);
        }
        canonical_module_owns_cached_source_path(root, module, path, &owned_by_module[module])
            .map_err(|error| {
                format!(
                    "acceptance owner correction batch entry {} failed: {error}",
                    index + 1
                )
            })?;
        provisional.acceptance_owner_corrections.push(correction);
    }

    record.acceptance_owner_corrections = provisional.acceptance_owner_corrections;
    record.updated_at = timestamp;
    let dir = change_dir(root, &record.id);
    write_prepared_files(
        root,
        &[
            (json_content_keeping_unknown_members(&dir.join("state.json"), relay_gated_record(&dir, &record).as_ref()).map(|content| (dir.join("state.json"), content))?),
            (dir.join("change.md"), change_markdown_content(&record)),
        ],
    )?;
    Ok(record)
}

/// Discover production-source affected paths lacking canonical ownership for `module`, then append
/// one audited correction per path in a single transactional write.
pub fn add_missing_acceptance_owner_corrections(
    root: &Path,
    id: &str,
    module: String,
    actor: String,
    reason: String,
) -> Result<ChangeRecord, String> {
    let module = module.trim();
    if module.is_empty() {
        return Err("--all-missing requires a non-empty --spec module".into());
    }
    let paths = missing_acceptance_owner_paths(root, id, module)?;
    if paths.is_empty() {
        return Err(format!(
            "no production-source affected paths lack canonical ownership for module `{module}`"
        ));
    }
    let entries = paths
        .into_iter()
        .map(|path| (path, module.to_string()))
        .collect();
    add_acceptance_owner_corrections(root, id, entries, actor, reason)
}

fn missing_acceptance_owner_paths(
    root: &Path,
    id: &str,
    module: &str,
) -> Result<Vec<String>, String> {
    let record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Verifying],
        "discover missing acceptance input owners",
    )?;
    crate::commands::validate_module_name(module)
        .map_err(|error| format!("invalid acceptance owner correction module: {error}"))?;
    if module.starts_with("@exact:") {
        return Err("reserved exact owners cannot be discovered through correct-owner".into());
    }
    let candidate_paths = record
        .affected_paths
        .iter()
        .filter_map(|path| strict_portable_relative_path(path).ok())
        .filter(|path| path_is_production_source(root, path))
        .collect::<Vec<_>>();
    if candidate_paths.is_empty() {
        return Ok(Vec::new());
    }
    let owner_evidence = acceptance_owner_spec_evidence(root, &record)?;
    // Preserve the established `--all-missing` behavior: an unavailable requested owner simply
    // discovers no matching paths and the caller emits the existing empty-discovery diagnostic.
    let requested_owner_paths = canonical_module_source_paths(root, module).ok();
    let mut missing = Vec::new();
    for path in candidate_paths {
        if !production_source_lacks_canonical_owner_with_evidence(
            root,
            &record,
            &path,
            &owner_evidence,
        )? {
            continue;
        }
        if requested_owner_paths.as_ref().is_some_and(|owned_paths| {
            canonical_module_owns_cached_source_path(root, module, &path, owned_paths).is_ok()
        }) {
            missing.push(path);
        }
    }
    missing.sort();
    missing.dedup();
    Ok(missing)
}

fn acceptance_owner_spec_evidence(
    root: &Path,
    record: &ChangeRecord,
) -> Result<GitEvidence, String> {
    let config = crate::config::load_config(root);
    let mut candidates = BTreeSet::new();
    for module in &record.affected_specs {
        let (spec_path, _) = canonical_module_paths(root, &config.specs_dir, module)?;
        candidates.insert(strict_portable_project_path(root, &spec_path)?);
    }
    if candidates.is_empty() {
        Ok(GitEvidence {
            modes: BTreeMap::new(),
            entries: BTreeMap::new(),
        })
    } else {
        git_regular_file_evidence(root, &candidates)
    }
}

/// Reject declared paths that no declared module canonically owns.
///
/// Ownership is knowable the moment a path is declared, but was previously
/// resolved only while building the acceptance manifest at finalize. A change
/// naming a path owned by a module it does not declare therefore passed approve
/// and every check, was reviewed, and failed at the terminal step — into a state
/// with no exit, since `correct-owner` is scoped to already-applied changes and
/// `reopen` accepts only Accepted/Archived.
///
/// Every offending path is reported together: discovering them one per
/// verification pass is what made the original failure expensive.
fn validate_declared_path_ownership(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if record.affected_paths.is_empty() {
        return Ok(());
    }
    // Ownership is resolved against the change's declared specs. Empty specs
    // reach here only for justified `no_spec_change` work (validate_definition
    // already rejects empty specs without that flag). With no declared module an
    // existing production-source path has no owner, and finalize refuses it, so
    // it is refused here, before anyone approves or reviews it (#986, #956).
    if record.affected_specs.is_empty() && !record.no_spec_change {
        return Err(
            "affected specs are required for path ownership validation unless no_spec_change is justified"
                .into(),
        );
    }
    let evidence = acceptance_owner_spec_evidence(root, record)?;
    let mut unowned = Vec::new();
    for path in &record.affected_paths {
        // Only paths that already exist. A change routinely declares a file it is
        // about to create, and ownership for a file that does not exist yet cannot
        // be resolved — the owning spec may well be claiming it in this change's
        // own delta. Those are still enforced at finalize, once the file is real.
        if !root.join(path).exists() {
            continue;
        }
        // A declared directory is a scope, not a file: ownership is decided for its
        // descendant files when the acceptance walker collects them at finalize, so the
        // scope itself is never checked by its extension here (#1011).
        if root.join(path).is_dir() {
            continue;
        }
        if production_source_lacks_canonical_owner_with_evidence(root, record, path, &evidence)? {
            unowned.push(path.as_str());
        }
    }
    if unowned.is_empty() {
        return Ok(());
    }
    let index = SpecListingIndex::build(root, &crate::config::load_config(root).specs_dir);
    let hints: Vec<String> = unowned
        .iter()
        .map(|path| format!("  - {path}: {}", ownership_hint(root, record, path, &index)))
        .collect();
    let mut message = format!(
        "no declared module canonically owns {}: {}. Declare the owning module with `--spec`, \
         or add the path to an owning spec's `files:` list:\n{}",
        if unowned.len() == 1 {
            "this affected path"
        } else {
            "these affected paths"
        },
        unowned.join(", "),
        hints.join("\n")
    );
    if record.state == ChangeState::Draft {
        match declare_owners_answer(root, record, &unowned, &index) {
            Some(answer) => message.push_str(&format!("\nOn this draft, declare them with {answer}.")),
            None => {
                if let Some(missing) = declared_modules_without_spec(root, record) {
                    message.push_str(&format!("\n{missing}."));
                }
            }
        }
    }
    Err(message)
}

/// The sentence naming every declared module with no spec at its canonical path, which keeps the
/// owner recovery from printing an `affected_specs` answer (#1026 review); `None` when every
/// declared module has its spec.
fn declared_modules_without_spec(root: &Path, record: &ChangeRecord) -> Option<String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let missing: Vec<String> = record
        .affected_specs
        .iter()
        .filter(|module| !module_has_canonical_spec(root, &specs_dir, module))
        .map(|module| match canonical_module_paths(root, &specs_dir, module) {
            Ok((spec_path, _)) => format!(
                "`{module}` has no spec at `{}`",
                portable_project_path(root, &spec_path)
            ),
            Err(_) => format!("`{module}` has no spec at its canonical path"),
        })
        .collect();
    (!missing.is_empty()).then(|| {
        format!(
            "The declared {} {}, so no `affected_specs` answer is printed: create that spec, or drop the module from `affected_specs`",
            if missing.len() == 1 { "module" } else { "modules" },
            missing.join(" and ")
        )
    })
}

/// Every module that owns `path` and that `record` does not declare: modules whose configured
/// `owns` covers it (sorted by name), then modules whose canonical spec lists it in `files:`
/// (in spec path order), each once. The same modules `ownership_hint` tells the operator to add.
fn undeclared_owning_modules(
    root: &Path,
    record: &ChangeRecord,
    path: &str,
    index: &SpecListingIndex,
) -> Vec<String> {
    let config = crate::config::load_config(root);
    let mut modules: Vec<String> = config
        .modules
        .keys()
        .filter(|module| !record.affected_specs.contains(*module))
        .filter(|module| configured_module_owns_path(&config, module, path))
        .cloned()
        .collect();
    modules.sort();
    for (spec, module) in index.listing(path) {
        if let Some(module) = module
            && !record.affected_specs.contains(&module)
            && !modules.contains(&module)
            && listing_spec_is_canonical(root, &config.specs_dir, &spec, &module)
        {
            modules.push(module);
        }
    }
    modules
}

/// Whether `spec` is `module`'s canonical spec, the only spec whose `files:` give the module
/// ownership; a listing anywhere else owns nothing, so declaring the module would not help.
fn listing_spec_is_canonical(root: &Path, specs_dir: &str, spec: &str, module: &str) -> bool {
    canonical_module_paths(root, specs_dir, module)
        .ok()
        .is_some_and(|(spec_path, _)| portable_project_path(root, &spec_path) == spec)
}

/// `specsync change answer <id> affected_specs <modules>` that declares, beside the modules
/// `record` already declares, every module that owns one of `paths`; `None` when no module owns
/// any of them, so declaring cannot help (#986). The command is only printed when it runs exactly
/// as printed: `change answer` splits the value on commas and newlines, so a comma- or
/// newline-bearing module name cannot round-trip, and a listing that hit its walk bound may be
/// missing the owner (#1011). When the record carries a constraints answer, the guidance renews
/// it: answering `affected_specs` clears it.
fn declare_owners_answer(
    root: &Path,
    record: &ChangeRecord,
    paths: &[&str],
    index: &SpecListingIndex,
) -> Option<String> {
    if index.truncated {
        return None;
    }
    let mut modules = record.affected_specs.clone();
    let before = modules.len();
    for path in paths {
        for module in undeclared_owning_modules(root, record, path, index) {
            if !modules.contains(&module) {
                modules.push(module);
            }
        }
    }
    // Print an answer command only when `change answer affected_specs` would take it as
    // printed: it splits on commas, trims each value and runs `validate_module_name`.
    if modules.len() == before
        || modules.iter().any(|module| {
            module.contains([',', '\n'])
                || module.trim() != module
                || crate::commands::validate_module_name(module).is_err()
        })
    {
        return None;
    }
    // A declared module with no spec at its canonical path fails verification, and outside a
    // project with no specs `change answer affected_specs` refuses it (#426). The answer re-lists
    // every module, already-declared ones included, so no answer is printed while any of them
    // lacks a spec; the per-path hints and `declared_modules_without_spec` say which spec to
    // create first (#1026 review).
    let specs_dir = crate::config::load_config(root).specs_dir;
    if modules
        .iter()
        .any(|module| !module_has_canonical_spec(root, &specs_dir, module))
    {
        return None;
    }
    let mut guidance = String::new();
    guidance.push_str(&format!(
        "`specsync change answer {} affected_specs {}`",
        shell_quote(&record.id),
        shell_quote(&modules.join(","))
    ));
    if record.answers.contains_key("constraints") {
        guidance.push_str(&format!(
            ", then renew the constraints answer with `specsync change answer {} constraints <answer>` — answering affected_specs clears it",
            shell_quote(&record.id)
        ));
    }
    Some(guidance)
}

/// The ship-preflight blocker for production source in the change's delivery that no declared
/// module owns, and the recovery to name as the next action; `None` when every input has an
/// owner (REQ-cmd-change-008).
///
/// Finalize signs the delivery inputs with `acceptance_manifest` and refuses an unowned
/// production-source input. This builds the same manifest read-only, letting such an input fall
/// to the exact delivery owner instead of failing, so it can name every one of them at once:
/// a production-source entry left with only that owner is exactly what finalize refuses.
pub fn ownership_preflight_blocker(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<(String, String)>, String> {
    let unowned = unowned_production_delivery_inputs(root, record)?;
    if unowned.is_empty() {
        return Ok(None);
    }
    let paths: Vec<&str> = unowned.iter().map(String::as_str).collect();
    let index = SpecListingIndex::build(root, &crate::config::load_config(root).specs_dir);
    // An unowned input this branch never modified is in scope only because a path prefix is
    // wider than the work, and declaring its owner would pull an untouched module into the
    // change: the recovery for it is a narrower scope (#1000). Without a usable base commit, a
    // merge base with the remote default branch or Git, nothing is classified as untouched and
    // the declare recovery covers every path.
    // A truncated spec listing cannot say which owners exist, so nothing is classified then.
    let changed = (!index.truncated)
        .then(|| files_changed_since_base(root, record))
        .flatten();
    let (touched, untouched): (Vec<&str>, Vec<&str>) = paths
        .iter()
        .copied()
        .partition(|path| changed.as_ref().is_none_or(|changed| changed.contains(*path)));
    // An untouched input whose every owner is already declared by the touched inputs' answer is
    // fixed by that answer; narrowing it out as well would be redundant advice (#1037 review).
    let declared_by_answer: BTreeSet<String> = record
        .affected_specs
        .iter()
        .cloned()
        .chain(
            touched
                .iter()
                .flat_map(|path| undeclared_owning_modules(root, record, path, &index)),
        )
        .collect();
    let answer_is_printed =
        !touched.is_empty() && declare_owners_answer(root, record, &touched, &index).is_some();
    let untouched: Vec<&str> = untouched
        .into_iter()
        .filter(|path| {
            let owners = undeclared_owning_modules(root, record, path, &index);
            !answer_is_printed
                || owners.is_empty()
                || !owners.iter().all(|owner| declared_by_answer.contains(owner))
        })
        .collect();
    let hints: Vec<String> = paths
        .iter()
        .map(|path| {
            if untouched.contains(path) {
                let owners = undeclared_owning_modules(root, record, path, &index);
                if owners.is_empty() {
                    format!(
                        "{path}: not modified by this change since its base and no module owns it, so leave it out of `affected_paths`"
                    )
                } else {
                    format!(
                        "{path}: owned by {}, but not modified by this change since its base, so leave it out of `affected_paths`",
                        owners
                            .iter()
                            .map(|owner| format!("`{owner}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            } else {
                format!("{path}: {}", ownership_hint(root, record, path, &index))
            }
        })
        .collect();
    let touched: Vec<&str> = paths
        .iter()
        .copied()
        .filter(|path| !untouched.contains(path))
        .collect();
    let recovery = match (touched.is_empty(), untouched.is_empty()) {
        (_, true) => declare_owners_recovery(root, record, &touched, &index),
        (true, false) => narrow_scope_recovery(record, &untouched, changed.as_ref()),
        (false, false) => declare_and_narrow_recovery(
            root,
            record,
            &touched,
            &untouched,
            &index,
            changed.as_ref(),
        ),
    };
    Ok(Some((
        format!(
            "finalize would refuse production source no declared module owns: {}",
            hints.join("; ")
        ),
        recovery,
    )))
}

/// The recovery that declares the owners of unowned inputs the change modified.
fn declare_owners_recovery(
    root: &Path,
    record: &ChangeRecord,
    paths: &[&str],
    index: &SpecListingIndex,
) -> String {
    // Paths no undeclared module claims are not fixed by declaring one; each needs a `files:`
    // entry in the spec that should own it.
    let unclaimed: Vec<&str> = paths
        .iter()
        .copied()
        .filter(|path| undeclared_owning_modules(root, record, path, index).is_empty())
        .collect();
    if index.truncated {
        return format!(
            "send it back with `specsync change send-back {} --actor <name> --reason \"<why>\"`; the spec listing exceeded its 10,000-entry walk bound, so owner discovery is incomplete — check which spec should own each path by hand and add it to that spec's `files:` list",
            shell_quote(&record.id)
        );
    }
    match declare_owners_answer(root, record, paths, index) {
        Some(answer) if !unclaimed.is_empty() => format!(
            "send it back and declare the owners: `specsync change send-back {} --actor <name> --reason \"declare the owning modules\"`, then {answer}, and add {} to its owning spec's `files:` list, then approve, check and review again",
            shell_quote(&record.id),
            unclaimed.join(", ")
        ),
        Some(answer) => format!(
            "send it back and declare the owners: `specsync change send-back {} --actor <name> --reason \"declare the owning modules\"`, then {answer}, then approve, check and review again",
            shell_quote(&record.id)
        ),
        None => format!(
            "send it back with `specsync change send-back {} --actor <name> --reason \"<why>\"` and give each path an owning spec: create the spec its hint names, or add the path to an existing spec's `files:` list, then declare that module{}",
            shell_quote(&record.id),
            declared_modules_without_spec(root, record)
                .map(|missing| format!(". {missing}"))
                .unwrap_or_default()
        ),
    }
}

/// The recovery for unowned inputs the change never modified: narrow `affected_paths` to the
/// files and directories the change edits instead of declaring their owners (#1000).
fn narrow_scope_recovery(
    record: &ChangeRecord,
    untouched: &[&str],
    changed: Option<&BTreeSet<String>>,
) -> String {
    format!(
        "{} — `specsync change send-back {} --actor <name> --reason \"narrow affected_paths\"`, then {}, then approve, check and review again",
        untouched_sentence(untouched),
        shell_quote(&record.id),
        narrow_answer(record, changed)
    )
}

/// One send-back for a change with both modified and untouched unowned inputs: declare the
/// modified inputs' owners and narrow the scope in the same draft, then one approve, check and
/// review (#1037 review: two separate send-backs could not both run).
fn declare_and_narrow_recovery(
    root: &Path,
    record: &ChangeRecord,
    touched: &[&str],
    untouched: &[&str],
    index: &SpecListingIndex,
    changed: Option<&BTreeSet<String>>,
) -> String {
    // A truncated spec listing turns classification off, so this mixed recovery never sees one.
    // Paths no undeclared module claims are not fixed by declaring one; each needs a `files:`
    // entry in the spec that should own it, exactly as in the declare-only recovery.
    let unclaimed: Vec<&str> = touched
        .iter()
        .copied()
        .filter(|path| undeclared_owning_modules(root, record, path, index).is_empty())
        .collect();
    // Without a printable answer (a listed module has no spec, #1026), the modified inputs get
    // the same owning-spec guidance as the declare-only recovery, and a declared module whose
    // spec is missing is named after the recovery.
    let (declare, missing) = match declare_owners_answer(root, record, touched, index) {
        Some(answer) if !unclaimed.is_empty() => (
            format!(
                "{answer}, and add {} to its owning spec's `files:` list",
                unclaimed.join(", ")
            ),
            None,
        ),
        Some(answer) => (answer, None),
        None => (
            format!(
                "give {} an owning spec: create the spec its hint names, or add the path to an existing spec's `files:` list, then declare that module",
                touched.join(", ")
            ),
            declared_modules_without_spec(root, record),
        ),
    };
    format!(
        "send it back once and rescope: `specsync change send-back {} --actor <name> --reason \"declare the owning modules and narrow affected_paths\"`, then {declare}, and {} ({}), then approve, check and review again{}",
        shell_quote(&record.id),
        narrow_answer(record, changed),
        untouched_sentence(untouched),
        missing
            .map(|missing| format!(". {missing}"))
            .unwrap_or_default()
    )
}

/// The `affected_paths` answer that narrows a scope, with a quoted, comma-separated placeholder.
fn narrow_answer(record: &ChangeRecord, changed: Option<&BTreeSet<String>>) -> String {
    // Name the files exactly: every path the change added, edited or deleted inside its current
    // scope. A placeholder invited listing only the edited files and forgetting a deletion, which
    // then shipped and failed the meaningful-path audit (#1037 review). The placeholder is kept
    // only when there is no list or a path cannot round-trip through the comma-separated answer.
    let in_scope: Vec<&String> = changed
        .map(|changed| {
            changed
                .iter()
                .filter(|path| {
                    record
                        .affected_paths
                        .iter()
                        .any(|scope| path_matches_scope(path, scope))
                })
                .collect()
        })
        .unwrap_or_default();
    // Printed only when `change answer` stores exactly these paths: `split_values` splits on
    // commas and newlines and trims each value, `normalize_project_path` rewrites a backslash and refuses
    // control characters, and a list starting with `-` would be read as an option.
    let round_trips = |path: &&String| {
        !path.contains([',', '\\'])
            && !path.chars().any(char::is_control)
            && path.trim() == path.as_str()
    };
    if !in_scope.is_empty()
        && in_scope.iter().all(round_trips)
        && !in_scope.first().is_some_and(|path| path.starts_with('-'))
    {
        let list = in_scope
            .iter()
            .map(|path| path.as_str())
            .collect::<Vec<_>>()
            .join(",");
        return format!(
            "`specsync change answer {} affected_paths {}` (every file the change adds, edits or deletes in its scope)",
            shell_quote(&record.id),
            shell_quote(&list)
        );
    }
    format!(
        "`specsync change answer {} affected_paths '<path>,<path>'` naming every file the change adds, edits or deletes, so the scope no longer covers the untouched paths",
        shell_quote(&record.id)
    )
}

/// Names up to five untouched inputs and why they should leave the scope.
fn untouched_sentence(untouched: &[&str]) -> String {
    const SHOWN: usize = 5;
    let mut named = untouched
        .iter()
        .take(SHOWN)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if untouched.len() > SHOWN {
        named.push_str(&format!(" and {} more", untouched.len() - SHOWN));
    }
    format!(
        "{named} {} not modified by this change since its base, so its `affected_paths` scope is wider than the work: narrow it instead of declaring those owners",
        if untouched.len() == 1 { "is" } else { "are" }
    )
}

/// Every project-relative path the working tree has changed since the change's base commit,
/// committed or not, plus untracked files. `None` without a base commit or when Git cannot say.
fn files_changed_since_base(root: &Path, record: &ChangeRecord) -> Option<BTreeSet<String>> {
    // A change whose record carries no full commit id as `base_commit` is not classified. The
    // value is only a gate: it never reaches Git, where `--output=<path>` read from `state.json`
    // could become an option, least of all from read-only `ship-status` (#1037 review).
    let base_commit = record
        .base_commit
        .as_deref()
        .filter(|base| is_canonical_commit_id(base))?;
    // The branch's own work is what differs from the merge base with the remote default branch,
    // the base the coverage gate diffs against locally (`pull_request_diff_base`; in CI it diffs
    // against the pull request's base branch), plus untracked files.
    // It is not compared with `base_commit`: once the branch merges or rebases the default branch,
    // that diff also holds every upstream edit made since, which this change never touched
    // (#1037 review). Without a remote default branch or a merge base, the branch's work cannot be
    // told from untouched files, so nothing is classified and every input keeps the declare
    // recovery: the safe direction.
    let remote = remote_default_ref(root)?;
    let merge_base = git_output(root, &["merge-base", remote.as_str(), "HEAD"])?;
    let merge_base = merge_base.trim();
    if !is_canonical_commit_id(merge_base) {
        return None;
    }
    // Once the change's own commits are on the remote default branch (it merged with the change
    // still active, the documented recovery state), the merge base no longer separates its work
    // from upstream work: its committed edits sit at or behind the merge base and would all look
    // untouched. Then nothing is classified (#1037 review). That is so when HEAD is the merge
    // base although the change has commits past its base, and when the merge base already holds
    // this change's own workspace, as after a recovery `change check --commit` on the default
    // branch. `base_commit` is only compared here; it still never reaches Git.
    let head = git_output(root, &["rev-parse", "--verify", "HEAD"])?;
    let head = head.trim();
    if !is_canonical_commit_id(head) || (merge_base == head && base_commit != head) {
        return None;
    }
    let own_workspace = format!("{CHANGES_PATH}/{}", record.id);
    let own_workspace_on_the_default_branch = !git_output_allow_empty(
        root,
        &[
            "ls-tree",
            "-d",
            "--name-only",
            "--end-of-options",
            merge_base,
            "--",
            own_workspace.as_str(),
        ],
    )?
    .trim()
    .is_empty();
    if own_workspace_on_the_default_branch {
        return None;
    }
    let mut changed = BTreeSet::new();
    // NUL-separated, so a non-ASCII, quoted or newline-bearing path comes back verbatim rather
    // than C-quoted under git's default `core.quotePath`; `--no-renames`, so a renamed file's old
    // path counts as changed too.
    for args in [
        vec![
            "diff",
            "--name-only",
            "--relative",
            "--no-renames",
            "-z",
            "--end-of-options",
            merge_base,
        ],
        vec!["ls-files", "--others", "--exclude-standard", "-z"],
    ] {
        changed.extend(
            git_output_allow_empty(root, &args)?
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_string),
        );
    }
    // A path another active change on this branch covers is that change's work: it is neither
    // declared for this change nor kept in its narrowed scope (#1037 review). An unreadable
    // workspace leaves that unknown, so nothing is classified.
    let others = list_changes_checked(root).ok()?;
    changed.retain(|path| {
        !others
            .iter()
            .filter(|other| other.id != record.id)
            .any(|other| other.affected_paths.iter().any(|scope| path_matches_scope(path, scope)))
    });
    Some(changed)
}

/// Names what to change for one unowned affected path.
///
/// Every undeclared module that would own the path is named, in a fixed order: modules whose
/// `[modules."<name>"] owns` covers it (sorted by name), then every spec under the specs
/// directory whose `files:` lists it (sorted by spec path). A listing spec that cannot own the
/// path says why. When nothing names a module to declare, the declared spec whose `files:` share
/// the deepest directory with the path is the one to edit; ties name every tied spec, and a path
/// that shares no directory with any declared spec (depth 0) gets the generic message. Nothing here depends on directory iteration or `--spec` order. This only words
/// the refusal; ownership is still decided by `production_source_lacks_canonical_owner_with_evidence`.
fn ownership_hint(
    root: &Path,
    record: &ChangeRecord,
    path: &str,
    index: &SpecListingIndex,
) -> String {
    let config = crate::config::load_config(root);
    let mut declare = Vec::new();
    let mut explain = Vec::new();

    let mut configured: Vec<&String> = config
        .modules
        .keys()
        .filter(|module| !record.affected_specs.contains(*module))
        .filter(|module| configured_module_owns_path(&config, module, path))
        .collect();
    configured.sort();
    for module in configured {
        let has_spec = module_has_canonical_spec(root, &config.specs_dir, module);
        declare.push(if has_spec {
            format!(
                "the project configuration gives it to module `{module}` (`[modules.\"{module}\"] owns`), which this change does not declare; add `--spec {}`",
                shell_word(module)
            )
        } else {
            format!(
                "the project configuration gives it to module `{module}` (`[modules.\"{module}\"] owns`), which has no spec yet; create that module's spec, then declare it"
            )
        });
    }

    for (spec, module) in index.listing(path) {
        match module {
            None => explain.push(format!(
                "`{spec}` lists it but has no `module:` in its frontmatter, so it owns nothing"
            )),
            Some(module)
                if !record.affected_specs.contains(&module)
                    && !listing_spec_is_canonical(root, &config.specs_dir, &spec, &module) =>
            {
                let canonical = canonical_module_paths(root, &config.specs_dir, &module)
                    .ok()
                    .map(|(spec_path, _)| portable_project_path(root, &spec_path))
                    .unwrap_or_else(|| format!("{}/{module}/{module}.spec.md", config.specs_dir));
                explain.push(format!(
                    "`{spec}` lists it for module `{module}`, but it is not that module's canonical spec `{canonical}`, so declaring `{module}` would not own it"
                ));
            }
            Some(module) if !record.affected_specs.contains(&module) => declare.push(format!(
                "`{spec}` lists it for module `{module}`, which this change does not declare; add `--spec {}`",
            shell_word(&module)
            )),
            Some(module) => {
                let canonical = canonical_module_paths(root, &config.specs_dir, &module)
                    .ok()
                    .map(|(spec_path, _)| portable_project_path(root, &spec_path));
                explain.push(match canonical {
                    Some(canonical) if canonical != spec => format!(
                        "`{spec}` lists it for declared module `{module}`, but it is not that module's canonical spec `{canonical}`, so the listing does not count"
                    ),
                    _ => format!(
                        "`{spec}` lists it for declared module `{module}`, but that spec does not yet prove ownership; run `specsync check` for the reason"
                    ),
                });
            }
        }
    }

    if !declare.is_empty() {
        explain.extend(declare);
    } else {
        explain.push(closest_declared_spec_hint(root, &config, record, path));
    }
    if index.truncated {
        explain.push(
            "the spec listing hit its 10,000-entry walk bound, so an owning spec may be missing from this list"
                .to_string(),
        );
    }
    explain.join("; ")
}

#[cfg(test)]
thread_local! {
    /// Spec-tree walks this thread made to find listing specs (#999 review).
    static TEST_SPEC_LISTING_WALKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn reset_test_spec_listing_walks() {
    TEST_SPEC_LISTING_WALKS.with(|walks| walks.set(0));
}

#[cfg(test)]
fn test_spec_listing_walks() -> usize {
    TEST_SPEC_LISTING_WALKS.with(std::cell::Cell::get)
}

/// Every `*.spec.md` under the specs directory, with its frontmatter module and normalized
/// `files:` entries, read in one bounded walk (10,000 entries), plus every registry-registered
/// canonical spec that lives outside the specs directory (#1011). A refusal or a preflight over
/// many paths builds it once and asks it per path, instead of rereading the spec tree per path
/// (#999 review). `truncated` records that the walk hit its bound, so recovery built on this
/// index must say the listing is incomplete instead of presenting it as complete.
struct SpecListingIndex {
    /// `(portable spec path, frontmatter module, files)`, sorted by spec path.
    specs: Vec<(String, Option<String>, BTreeSet<String>)>,
    /// The walk stopped at its entry bound; the listing may be missing specs.
    truncated: bool,
}

impl SpecListingIndex {
    fn build(root: &Path, specs_dir: &str) -> Self {
        #[cfg(test)]
        TEST_SPEC_LISTING_WALKS.with(|walks| walks.set(walks.get() + 1));
        let mut specs = Vec::new();
        let mut stack = vec![root.join(specs_dir)];
        let mut visited = 0usize;
        let mut truncated = false;
        'walk: while let Some(directory) = stack.pop() {
            let Ok(entries) = fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.flatten() {
                visited += 1;
                if visited > 10_000 {
                    truncated = true;
                    break 'walk;
                }
                let entry_path = entry.path();
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                if kind.is_dir() {
                    stack.push(entry_path);
                    continue;
                }
                if !kind.is_file()
                    || !entry_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".spec.md"))
                {
                    continue;
                }
                let Some(parsed) = fs::read_to_string(&entry_path)
                    .ok()
                    .and_then(|content| crate::parser::parse_frontmatter(&content))
                else {
                    continue;
                };
                let files = parsed
                    .frontmatter
                    .files
                    .iter()
                    .filter_map(|file| normalize_project_path(file).ok())
                    .collect();
                specs.push((
                    portable_project_path(root, &entry_path),
                    parsed.frontmatter.module,
                    files,
                ));
            }
        }
        // Canonical resolution honors the local registry first, so a registered canonical spec
        // outside the specs directory owns through its `files:` list exactly like a walked one
        // (#1011). The registry key is the module that resolution binds to the path.
        if let Ok(Some(registry)) = crate::registry::load_local_registry(root) {
            for (module, path) in &registry.specs {
                let Ok(spec_path) = safe_project_path(root, path) else {
                    continue;
                };
                let portable = portable_project_path(root, &spec_path);
                if specs.iter().any(|(spec, _, _)| spec == &portable) {
                    continue;
                }
                let Some(parsed) = fs::read_to_string(&spec_path)
                    .ok()
                    .and_then(|content| crate::parser::parse_frontmatter(&content))
                else {
                    continue;
                };
                let files = parsed
                    .frontmatter
                    .files
                    .iter()
                    .filter_map(|file| normalize_project_path(file).ok())
                    .collect();
                specs.push((portable, Some(module.clone()), files));
            }
        }
        specs.sort();
        Self { specs, truncated }
    }

    /// Every spec whose `files:` lists `path`, as `(portable spec path, frontmatter module)`,
    /// sorted by spec path.
    fn listing(&self, path: &str) -> Vec<(String, Option<String>)> {
        self.specs
            .iter()
            .filter(|(_, _, files)| files.contains(path))
            .map(|(spec, module, _)| (spec.clone(), module.clone()))
            .collect()
    }
}

/// The `files:` edit to suggest when no undeclared module would own `path`.
fn closest_declared_spec_hint(
    root: &Path,
    config: &crate::types::SpecSyncConfig,
    record: &ChangeRecord,
    path: &str,
) -> String {
    let ours: Vec<&str> = path.split('/').collect();
    let our_directory = &ours[..ours.len().saturating_sub(1)];
    // Whole path components only, so `src/foo` and `src/foobar` share `src` (depth 1).
    let shared_depth = |file: &str| {
        let theirs: Vec<&str> = file.split('/').collect();
        let their_directory = &theirs[..theirs.len().saturating_sub(1)];
        our_directory
            .iter()
            .zip(their_directory.iter())
            .take_while(|(left, right)| left == right)
            .count()
    };
    let mut declared: Vec<(usize, String)> = Vec::new();
    for module in &record.affected_specs {
        let Ok((spec_path, _)) = canonical_module_paths(root, &config.specs_dir, module) else {
            continue;
        };
        let depth = fs::read_to_string(&spec_path)
            .ok()
            .and_then(|content| crate::parser::parse_frontmatter(&content))
            .map(|parsed| {
                parsed
                    .frontmatter
                    .files
                    .iter()
                    .filter_map(|file| normalize_project_path(file).ok())
                    .map(|file| shared_depth(&file))
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        declared.push((depth, portable_project_path(root, &spec_path)));
    }
    declared.sort_by(|left, right| left.1.cmp(&right.1));
    declared.dedup_by(|left, right| left.1 == right.1);
    let best = declared.iter().map(|(depth, _)| *depth).max().unwrap_or(0);
    let quoted = |specs: Vec<&String>| {
        specs
            .iter()
            .map(|spec| format!("`{spec}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    // Depth 0 shares no directory at all, so no declared spec is a closer guess than any other.
    if best == 0 {
        return "add it to the `files:` list of the spec that owns it".to_string();
    }
    let closest: Vec<&String> = declared
        .iter()
        .filter(|(depth, _)| *depth == best)
        .map(|(_, spec)| spec)
        .collect();
    if closest.len() == 1 {
        format!("add it to the `files:` list in `{}`", closest[0])
    } else {
        format!(
            "add it to the `files:` list of one of {}",
            quoted(closest)
        )
    }
}
