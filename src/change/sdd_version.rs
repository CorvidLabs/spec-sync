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
#[cfg(test)]
thread_local! {
    static TEST_GIT_CHILD_SPAWNED: RefCell<Option<Box<dyn FnOnce(u32)>>> = RefCell::new(None);
    static TEST_GIT_STDIN_WRITER_JOINED: Cell<bool> = const { Cell::new(false) };
    static TRANSACTION_WRITE_FAILURE_INDEX: RefCell<Option<usize>> = const { RefCell::new(None) };
    static TRANSACTION_AFTER_JOURNAL_HOOK: RefCell<Option<Box<dyn FnOnce()>>> =
        RefCell::new(None);
    /// When true, `reconstruct_legacy_at_anchor` pretends `git worktree remove` failed.
    /// Product #511: cleanup hygiene must not discard a successful reconstruction.
    static FORCE_LEGACY_WORKTREE_REMOVE_FAILURE: Cell<bool> = const { Cell::new(false) };
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
    git_evidence: BTreeMap<GitEvidenceCacheKey, Result<GitEvidence, String>>,
    discovered_evidence:
        BTreeMap<DiscoveredEvidenceCacheKey, Result<(Vec<String>, GitEvidence), String>>,
    git_text_queries: BTreeMap<GitTextQueryCacheKey, Option<String>>,
    git_status_queries: BTreeMap<Vec<String>, Result<bool, String>>,
    workflow_version_history: BTreeMap<String, Result<(), String>>,
    historical_sequence_ledgers: BTreeMap<u64, Result<Vec<Vec<u8>>, String>>,
    terminal_evidence: Option<Result<BTreeMap<String, TerminalEvidenceSummary>, String>>,
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
            git_evidence: BTreeMap::new(),
            discovered_evidence: BTreeMap::new(),
            git_text_queries: BTreeMap::new(),
            git_status_queries: BTreeMap::new(),
            workflow_version_history: BTreeMap::new(),
            historical_sequence_ledgers: BTreeMap::new(),
            terminal_evidence: None,
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
}

/// Invocation-scoped snapshot for read-only lifecycle commands.
///
/// List, show, status, and read-only project reports install this scope. Mutating lifecycle APIs
/// do not create one, so their repository and evidence validation remains live. The first evidence
/// collection for each candidate set still performs the complete before/after Git race check.
pub(crate) struct ChangeReadScope {
    root: PathBuf,
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
        });
    }
}

pub(crate) fn begin_change_read_scope(root: &Path) -> ChangeReadScope {
    CHANGE_READ_SCOPES.with(|scopes| {
        scopes.borrow_mut().push(ChangeReadSnapshot::new(root));
    });
    ChangeReadScope {
        root: root.to_path_buf(),
    }
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
    let lock = ProjectLock { file };
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

fn recover_pending_transaction(root: &Path) -> Result<(), String> {
    let journal_path = root.join(TRANSACTION_PATH);
    if !journal_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&journal_path)
        .map_err(|error| format!("failed to read transaction journal: {error}"))?;
    let journal: TransactionJournalRead = serde_json::from_str(&content)
        .map_err(|error| format!("invalid transaction journal: {error}"))?;
    let entries = match journal {
        TransactionJournalRead::Legacy(entries) => entries,
        TransactionJournalRead::Durable(journal) => {
            if journal.schema_version != 1
                || journal.entry_count != journal.entries.len()
                || journal.entries_digest != transaction_entries_digest(&journal.entries)?
            {
                return Err("invalid transaction journal integrity envelope".into());
            }
            journal.entries
        }
    };
    for entry in entries {
        let path = safe_project_path(root, &entry.path)?;
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
            ignored_paths: vec![".specsync/".into(), "specs/".into()],
            verification_commands: Vec::new(),
            custom_artifacts: BTreeMap::new(),
            principles_file: None,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEvidence {
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
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
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| format!("invalid SDD policy {}: {error}", path.display()))
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
    for entry in entries {
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
    write_json(
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
            let historical = matches!(record.state, ChangeState::Accepted | ChangeState::Archived)
                || reopened_change_preserves_sequence_history(root, &record);
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
        || reopening.from_state != ChangeState::Accepted
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
fn sequence_ledger_freeze_next_action(root: &Path) -> Option<String> {
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

pub(crate) fn answer_question_with_snapshot(
    root: &Path,
    id: &str,
    question: &str,
    answer: &str,
) -> Result<DefinitionMutationResult, String> {
    let _lock = acquire_project_lock(root)?;
    let (mut record, corrections) = load_change_for_definition_mutation(root, id)?;
    require_state(&record, &[ChangeState::Draft], "answer interview questions")?;
    match question {
        "acceptance_criteria" => {
            record.acceptance_criteria = acceptance_criteria_values(answer)?;
        }
        "affected_specs" => {
            let values = split_values(answer);
            for module in &values {
                crate::commands::validate_module_name(module)
                    .map_err(|error| format!("invalid affected spec: {error}"))?;
            }
            record.affected_specs = values;
            // The lessons those specs carry may have changed, so the previous citation is stale.
            record.answers.remove("constraints");
        }
        "affected_paths" => {
            let values = split_values(answer);
            let affected_paths: Vec<String> = values
                .iter()
                .map(|path| {
                    normalize_project_path(path)
                        .map_err(|error| format!("invalid affected path: {error}"))
                })
                .collect::<Result<_, _>>()?;
            record.affected_paths = affected_paths;
        }
        "public_contract" => {
            record.answers.insert(question.into(), answer.into());
            if is_yes(answer) {
                add_artifact(&mut record, ArtifactKind::Requirements);
                add_artifact(&mut record, ArtifactKind::Docs);
            }
        }
        "architecture_risk" => {
            record.answers.insert(question.into(), answer.into());
            if is_yes(answer) {
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
            record.answers.insert(question.into(), answer.into());
        }
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
        predecessor_entry_digest,
    )
    .map(|result| result.change)
}

pub(crate) fn add_supersedes_obligation_with_snapshot(
    root: &Path,
    id: &str,
    predecessor: &str,
    path: &str,
    module: &str,
    predecessor_entry_digest: &str,
) -> Result<DefinitionMutationResult, String> {
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
    let summary =
        summarize_change_with_effective(root, &change, false, Some(&effective_definition));
    let strict_summary =
        summarize_change_with_effective(root, &change, true, Some(&effective_definition));
    DefinitionMutationResult {
        change,
        effective_definition,
        corrections,
        summary,
        strict_summary,
    }
}

pub fn approve_definition(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    approve_definition_with_projection(root, id, actor, note, false)
}

pub fn approve_definition_portable_v501(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    approve_definition_with_projection(root, id, actor, note, true)
}

fn approve_definition_with_projection(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
    portable_v501: bool,
) -> Result<ChangeRecord, String> {
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
    if portable_v501 {
        append_portable_definition_approval_v501(root, &record, actor, note)?;
    } else {
        let digest = definition_digest(root, &record)?;
        append_approval(root, &record, "definition", actor, digest, note)?;
    }
    record.state = match prior_state {
        ChangeState::Draft | ChangeState::Approved => ChangeState::Approved,
        ChangeState::Verifying if record.canonical_applied => ChangeState::Verifying,
        ChangeState::Implementing | ChangeState::Verifying => ChangeState::Implementing,
        ChangeState::Accepted | ChangeState::Archived => prior_state,
    };
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
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

fn materialize_change_deltas(root: &Path, id: &str) -> Result<ChangeRecord, String> {
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
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    ensure_tasks_complete(root, &record)?;
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
    prepared.push((
        change_dir(root, &record.id).join("state.json"),
        json_content(&record)?,
    ));
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
    materialize_change_deltas(root, &id)?;
    verify_change_with_strict(root, &id, strict).map(Some)
}

pub fn verify_change(root: &Path, id: &str) -> Result<VerificationRecord, String> {
    verify_change_with_strict(root, id, false)
}

pub fn verify_change_with_strict(
    root: &Path,
    id: &str,
    strict: bool,
) -> Result<VerificationRecord, String> {
    if let Some(error) = crate::verification_recursion_error() {
        return Err(error);
    }
    let _lock = acquire_project_lock(root)?;
    verify_change_locked(root, id, strict)
}

/// Verification body without lock acquisition.
///
/// The caller MUST already hold the project lock. Acceptance re-records evidence
/// while holding its own lock, and `acquire_project_lock` is a non-reentrant
/// `flock`, so calling the public entry point from there would block forever
/// rather than fail.
fn verify_change_locked(root: &Path, id: &str, strict: bool) -> Result<VerificationRecord, String> {
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Implementing, ChangeState::Verifying],
        "verify the change",
    )?;
    ensure_definition_approval_valid(root, &record)?;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    let records = list_changes_checked(root)?;
    // Verification has no non-error output channel; `specsync check` and
    // `specsync change audit` run this same gate and report the suppressions.
    if let Some(errors) = validate_effective_contracts(root, &records).error_text() {
        return Err(errors);
    }
    ensure_tasks_complete(root, &record)?;
    // Evidence completeness is markdown in the change workspace. Fail it before
    // the spec↔code pass so a missing testing.md row is not a 18-minute suite.
    let requirement_ids = collect_requirement_ids(root, &record)?;
    let has_semantic_acceptance_item = semantic_acceptance_item_exists(root, &record)?;
    let missing_evidence = requirement_evidence_missing(root, &record, &requirement_ids);
    let acceptance_evidence_present =
        acceptance_criteria_have_evidence(&record, has_semantic_acceptance_item);
    if !acceptance_evidence_present || !missing_evidence.is_empty() {
        return Err(format!(
            "verification cannot start: {}",
            evidence_gap_detail(&record, acceptance_evidence_present, &missing_evidence)
        ));
    }
    let (sync, findings) = evaluate_spec_code_sync(root, &record, strict)?;
    let commands = vec![sync];
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
    // the only place the persisted record names it: findings are not written to
    // `verification.json`, so without this the stored evidence for a
    // missing-spec failure would not say which module was missing.
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
    for spec in &scope.files {
        let result =
            crate::validator::validate_spec(spec, root, &schema_tables, &schema_columns, &config);
        let inline_ignores = fs::read_to_string(spec)
            .map(|content| crate::ignore::IgnoreRules::parse_inline(&content))
            .unwrap_or_default();
        errors += result.errors.len();
        findings.extend(result.errors);
        for warning in result.warnings {
            if ignore_rules.is_suppressed(&warning, &result.spec_path, &inline_ignores) {
                continue;
            }
            warnings += 1;
            findings.push(format!("warning: {warning}"));
        }
    }
    let failed = errors > 0 || (strict && warnings > 0);
    Ok((
        CommandEvidence {
            command: scoped_check_command(&scope.filters, strict),
            success: !failed,
            exit_code: Some(i32::from(failed)),
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
    let actor = actor.trim();
    if actor.is_empty() {
        return Err("reopen requires a non-empty human actor passed with --actor".into());
    }
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
    let anchored = if tip_matches_closing {
        authenticate_accepted_evidence_with_anchor(root, &record, None)?.1
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
    if !inputs_drifted && anchored && !legacy_unreconstructible {
        return Err(
            "accepted change delivery inputs are current (exact or successor-covered) and its verification commit is still anchored in current history; reopen is allowed only when accepted evidence is stale"
                .into(),
        );
    }
    if tip_matches_closing && anchored && !legacy_unreconstructible {
        let records = list_all_changes_checked(root)?;
        let mut visiting = BTreeSet::new();
        let mut memo = BTreeMap::new();
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
            (
                change_dir(root, &record.id).join("approvals.json"),
                json_content(&ledger)?,
            ),
            (
                change_dir(root, &record.id).join("state.json"),
                json_content(&record)?,
            ),
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
    let actor = actor.trim();
    if actor.is_empty() {
        return Err(
            "acceptance owner correction requires a non-empty human actor passed with --actor"
                .into(),
        );
    }
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
            (dir.join("state.json"), json_content(&record)?),
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
    // already rejects empty specs without that flag). There is then no owner set
    // to resolve against; finalize still enforces production ownership against
    // full delivery evidence for that class.
    if record.affected_specs.is_empty() {
        if !record.no_spec_change {
            return Err(
                "affected specs are required for path ownership validation unless no_spec_change is justified"
                    .into(),
            );
        }
        return Ok(());
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
        if production_source_lacks_canonical_owner_with_evidence(root, record, path, &evidence)? {
            unowned.push(path.as_str());
        }
    }
    if unowned.is_empty() {
        return Ok(());
    }
    Err(format!(
        "no declared module canonically owns {}: {}. Declare the owning module with `--spec`, \
         or add the path to an owning spec's `files:` list",
        if unowned.len() == 1 {
            "this affected path"
        } else {
            "these affected paths"
        },
        unowned.join(", ")
    ))
}
