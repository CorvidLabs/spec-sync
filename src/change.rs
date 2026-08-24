use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

pub const SDD_VERSION: &str = "5.0.0";
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
static EFFECTIVE_CONTRACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static TRUSTED_CORRECTION_HISTORY_CACHE: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();
static GIT_BLOB_CACHE: OnceLock<Mutex<BTreeMap<String, Vec<u8>>>> = OnceLock::new();
#[cfg(test)]
thread_local! {
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
const PROJECT_DIGEST_DOMAIN: &[u8] = b"specsync.project-input-digest.v2";
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
        let limits: LifecycleValidationLimits = serde_json::from_str(include_str!(concat!(
            "../.github/scripts/",
            "lifecycle-validation-limits.json"
        )))
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
    let mut policy = SddPolicy {
        verification_commands,
        ..SddPolicy::default()
    };
    policy.require_change_for_meaningful_files =
        git_output(root, &["rev-parse", "--verify", "HEAD"]).is_some();
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
    let workflow_version = if read_workflow_v2_baseline(root)?.is_some() {
        2
    } else {
        load_policy(root)
            .map(|policy| if policy.version >= 2 { 2 } else { 1 })
            .unwrap_or(2)
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
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
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
        // A directory with no `state.json` is not an active change *here*. Skip it.
        // Every other read error is still reported: an unreadable state.json is a
        // real problem and must never be mistaken for an absent one.
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
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
                // empty `deltas/`. Treating it as a corrupt change made
                // `change new` fail outright on any branch that did not contain
                // an earlier change, which is an ordinary branching pattern.
                // A directory with no `state.json` is not an active change here.
                Err(error) if !archived && error.kind() == std::io::ErrorKind::NotFound => {
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
            && reopening.stale_evidence_cause != Some(ReopenCauseV1::VerificationCommitUnanchored))
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
restore it with `git checkout HEAD -- {SEQUENCE_PATH}` (nothing writes this file any more, so it cannot be repaired by allocating)",
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
    if record.canonical_applied {
        return Ok(record);
    }
    if is_ci_project(root) {
        return Err(
            "approved canonical deltas are not materialized; run `specsync change check` locally and commit the implementation before CI"
                .into(),
        );
    }
    let mut prepared = prepare_delta_application(root, &record)?;
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
    let policy = load_policy_checked(root)?.unwrap_or_default();
    let verification_commands = verification_commands_for_change(root, &policy, &record, strict)?;
    for configured in &verification_commands {
        reject_direct_lifecycle_verification(root, configured)?;
    }
    // Evidence completeness is derived from committed artifacts alone, so it is
    // resolved before the verification commands run. Discovering a missing
    // requirement-evidence row only after a full suite costs an entire
    // re-verification cycle for a defect the workspace already described.
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
    let mut commands = Vec::new();
    for configured in verification_commands {
        let status = run_configured_command(root, &configured)?;
        commands.push(CommandEvidence {
            command: configured,
            success: status.success(),
            exit_code: status.code(),
        });
        if !status.success() {
            break;
        }
    }
    let commands_passed = commands.iter().all(|command| command.success);
    let passed = commands_passed;
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
        let failed = verification
            .commands
            .iter()
            .find(|command| !command.success)
            .map(|command| {
                format!(
                    "`{}` exited with {}",
                    command.command,
                    command
                        .exit_code
                        .map_or_else(|| "a signal".to_string(), |code| code.to_string())
                )
            })
            .unwrap_or_else(|| "a configured verification command failed".to_string());
        return Err(format!(
            "verification failed: {failed}; see commands[] in {}",
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
        authenticate_accepted_evidence_with_anchor(root, &record)?.1
    } else {
        accepted_evidence_is_anchored(root, &record, &prior_verification)
    };
    if !inputs_drifted && anchored {
        return Err(
            "accepted change delivery inputs are current (exact or successor-covered) and its verification commit is still anchored in current history; reopen is allowed only when accepted evidence is stale"
                .into(),
        );
    }
    if tip_matches_closing && anchored {
        let records = list_all_changes_checked(root)?;
        let mut visiting = BTreeSet::new();
        let mut memo = BTreeMap::new();
        if validate_accepted_inputs_recursive(root, &record, &records, &mut visiting, &mut memo)
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
        stale_evidence_cause: (!anchored).then_some(ReopenCauseV1::VerificationCommitUnanchored),
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

fn production_source_lacks_canonical_owner_with_evidence(
    root: &Path,
    record: &ChangeRecord,
    relative: &str,
    evidence: &GitEvidence,
) -> Result<bool, String> {
    if !path_is_production_source(root, relative) {
        return Ok(false);
    }
    let owners = acceptance_input_owners(
        root,
        record,
        relative,
        &[],
        evidence,
        UnownedProductionSource::Reject,
    );
    match owners {
        Ok(owners) => Ok(owners.iter().all(|owner| owner.starts_with("@exact:"))),
        Err(error) if error.contains("without deterministic canonical ownership") => Ok(true),
        Err(error) => Err(error),
    }
}

#[derive(Serialize)]
struct CorrectionDigestPayload<'a> {
    schema_version: u32,
    sequence: u64,
    change_id: &'a str,
    field: CorrectionField,
    original_value: &'a str,
    prior_effective_value: &'a str,
    corrected_value: &'a str,
    actor: &'a str,
    reason: &'a str,
    timestamp: u64,
    added_artifacts: &'a [ArtifactKind],
    superseded_definition_approval: &'a ApprovalRecord,
    superseded_closing_approval: &'a ApprovalRecord,
    prior_verification: &'a VerificationRecord,
}

fn canonical_correction_value(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "y" | "true" | "1" => Ok("yes".into()),
        "no" | "n" | "false" | "0" => Ok("no".into()),
        _ => Err(format!(
            "unsupported correction value `{value}` (expected yes or no)"
        )),
    }
}

fn initial_correction_view_digest(record: &ChangeRecord) -> Result<String, String> {
    let mut digest = FramedDigest::new(CORRECTION_VIEW_DIGEST_DOMAIN);
    digest.frame(b"change-id", record.id.as_bytes());
    for field in [
        CorrectionField::PublicContract,
        CorrectionField::ArchitectureRisk,
    ] {
        let value = record
            .answers
            .get(field.as_str())
            .map(String::as_str)
            .unwrap_or("<missing>");
        if value != "<missing>" {
            canonical_correction_value(value)?;
        }
        digest.frame(field.as_str().as_bytes(), value.as_bytes());
    }
    let artifacts = serde_json::to_vec(&record.selected_artifacts)
        .map_err(|error| format!("failed to hash original artifact selection: {error}"))?;
    digest.frame(b"selected-artifacts", &artifacts);
    Ok(digest.finish())
}

fn corrected_view_digest(previous: &str, record: &CorrectionRecord) -> Result<String, String> {
    let payload = CorrectionDigestPayload {
        schema_version: record.schema_version,
        sequence: record.sequence,
        change_id: &record.change_id,
        field: record.field,
        original_value: &record.original_value,
        prior_effective_value: &record.prior_effective_value,
        corrected_value: &record.corrected_value,
        actor: &record.actor,
        reason: &record.reason,
        timestamp: record.timestamp,
        added_artifacts: &record.added_artifacts,
        superseded_definition_approval: &record.superseded_definition_approval,
        superseded_closing_approval: &record.superseded_closing_approval,
        prior_verification: &record.prior_verification,
    };
    let payload = serde_json::to_vec(&payload)
        .map_err(|error| format!("failed to hash correction event: {error}"))?;
    let mut digest = FramedDigest::new(CORRECTION_VIEW_DIGEST_DOMAIN);
    digest.frame(b"prior-view", previous.as_bytes());
    digest.frame(b"correction", &payload);
    Ok(digest.finish())
}

fn validate_correction_records(
    record: &ChangeRecord,
    corrections: &[CorrectionRecord],
) -> Result<EffectiveChangeDefinition, String> {
    if record.correction_count != corrections.len() as u64 {
        return Err(format!(
            "correction ledger length {} does not match state.json correction_count {}",
            corrections.len(),
            record.correction_count
        ));
    }
    let mut answers = record.answers.clone();
    let mut selected_artifacts = record.selected_artifacts.clone();
    let mut view_digest = initial_correction_view_digest(record)?;

    for (index, correction) in corrections.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if correction.schema_version != 1 {
            return Err(format!(
                "unsupported correction schema version {} at sequence {expected_sequence}",
                correction.schema_version
            ));
        }
        if correction.sequence != expected_sequence {
            return Err(format!(
                "correction sequence is not contiguous: expected {expected_sequence}, found {}",
                correction.sequence
            ));
        }
        if correction.change_id != record.id {
            return Err(format!(
                "correction sequence {expected_sequence} belongs to `{}` instead of `{}`",
                correction.change_id, record.id
            ));
        }
        if correction.actor.trim().is_empty() || correction.reason.trim().is_empty() {
            return Err(format!(
                "correction sequence {expected_sequence} is missing its human actor or reason"
            ));
        }
        if correction.superseded_definition_approval.gate != "definition"
            || !is_terminal_approval(&correction.superseded_closing_approval)
            || !correction.prior_verification.passed
            || correction.superseded_definition_approval.digest
                != correction.prior_verification.contract_digest
        {
            return Err(format!(
                "correction sequence {expected_sequence} contains invalid prior gate evidence"
            ));
        }
        if correction.superseded_closing_approval.digest
            != closing_digest(record, &correction.prior_verification)
        {
            return Err(format!(
                "correction sequence {expected_sequence} prior closing evidence is inconsistent"
            ));
        }

        let field = correction.field.as_str();
        let original = record.answers.get(field).ok_or_else(|| {
            format!("accepted change is missing original `{field}` interview metadata")
        })?;
        canonical_correction_value(original)?;
        if correction.original_value != *original {
            return Err(format!(
                "correction sequence {expected_sequence} original `{field}` value does not match state.json"
            ));
        }
        let prior = answers.get(field).ok_or_else(|| {
            format!("correction sequence {expected_sequence} has no prior `{field}` value")
        })?;
        if correction.prior_effective_value != *prior {
            return Err(format!(
                "correction sequence {expected_sequence} prior `{field}` value breaks the append-only chain"
            ));
        }
        let corrected = canonical_correction_value(&correction.corrected_value)?;
        if corrected != correction.corrected_value
            || corrected == canonical_correction_value(prior)?
        {
            return Err(format!(
                "correction sequence {expected_sequence} must contain a changed canonical yes/no value"
            ));
        }

        let expected_added: Vec<ArtifactKind> = if corrected == "yes" {
            correction
                .field
                .required_artifacts()
                .iter()
                .filter(|artifact| !selected_artifacts.contains(artifact))
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        if correction.added_artifacts != expected_added {
            return Err(format!(
                "correction sequence {expected_sequence} has a non-monotonic or incomplete artifact selection"
            ));
        }
        if correction.prior_view_digest != view_digest {
            return Err(format!(
                "correction sequence {expected_sequence} prior view digest is invalid"
            ));
        }
        let expected_digest = corrected_view_digest(&view_digest, correction)?;
        if correction.corrected_view_digest != expected_digest {
            return Err(format!(
                "correction sequence {expected_sequence} corrected view digest is invalid"
            ));
        }

        answers.insert(field.into(), corrected);
        selected_artifacts.extend(expected_added);
        view_digest = expected_digest;
    }

    Ok(EffectiveChangeDefinition {
        answers,
        selected_artifacts,
        view_digest,
    })
}

fn validate_correction_records_for_prefix(
    record: &ChangeRecord,
    corrections: &[CorrectionRecord],
) -> Result<EffectiveChangeDefinition, String> {
    let mut prefix_record = record.clone();
    prefix_record.correction_count = corrections.len() as u64;
    validate_correction_records(&prefix_record, corrections)
}

fn load_correction_ledger(root: &Path, record: &ChangeRecord) -> Result<CorrectionLedger, String> {
    let path = find_change_dir(root, &record.id)?.join(CORRECTIONS_FILE);
    let ledger = match fs::symlink_metadata(&path) {
        Ok(_) => {
            let relative = strict_portable_project_path(root, &path)?;
            let evidence = git_regular_file_evidence(root, &BTreeSet::from([relative.clone()]))?;
            let entry = evidence.entry(&relative)?;
            if entry.payload.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
                return Err(format!(
                    "correction ledger exceeds byte limit: {}",
                    path.display()
                ));
            }
            serde_json::from_slice(&entry.payload)
                .map_err(|error| format!("invalid correction ledger {}: {error}", path.display()))?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => CorrectionLedger::default(),
        Err(error) => {
            return Err(format!(
                "failed to inspect correction ledger {}: {error}",
                path.display()
            ));
        }
    };
    if ledger.schema_version != 1 {
        return Err(format!(
            "unsupported correction ledger schema version {}",
            ledger.schema_version
        ));
    }
    validate_trusted_correction_history(root, record, &ledger)?;
    Ok(ledger)
}

pub fn effective_change_definition(
    root: &Path,
    record: &ChangeRecord,
) -> Result<EffectiveChangeDefinition, String> {
    let ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &ledger.corrections)
}

fn validate_trusted_correction_history(
    root: &Path,
    record: &ChangeRecord,
    current: &CorrectionLedger,
) -> Result<(), String> {
    let Some(head) = git_output(root, &["rev-parse", "--verify", "HEAD"]) else {
        // A project that has not entered Git history has no trusted historical
        // anchor yet. Its local correction chain is still validated below by
        // `validate_correction_records`.
        return Ok(());
    };
    let mut references = vec![head];
    if let Some(remote_default) = remote_default_ref(root) {
        let remote_default_commit = format!("{remote_default}^{{commit}}");
        if let Some(resolved) = git_output(root, &["rev-parse", "--verify", &remote_default_commit])
        {
            references.push(resolved);
        }
    }
    references.sort();
    references.dedup();
    let shallow = git_output(root, &["rev-parse", "--is-shallow-repository"])
        .is_some_and(|value| value == "true");
    // A correction-free shallow tip is indistinguishable from a correction
    // that was accepted and rolled back below the shallow boundary. Require
    // history through the recorded base before trusting either case; only a
    // demonstrably new change at or after that boundary can safely pass.
    if shallow && !shallow_history_is_complete_for_change(root, record, &references)? {
        return Err(format!(
            "cannot validate append-only correction history for {} from an incomplete shallow Git checkout; fetch through its recorded base commit",
            record.id
        ));
    }
    let ledger_bytes = serde_json::to_vec(current)
        .map_err(|error| format!("failed to cache correction history validation: {error}"))?;
    let mut cache_digest = FramedDigest::new(b"specsync.trusted-correction-cache.v1");
    cache_digest.frame(b"root", root.to_string_lossy().as_bytes());
    cache_digest.frame(b"change-id", record.id.as_bytes());
    cache_digest.frame(b"references", references.join("\n").as_bytes());
    cache_digest.frame(b"shallow", if shallow { b"true" } else { b"false" });
    cache_digest.frame(b"ledger", &ledger_bytes);
    let cache_key = cache_digest.finish();
    let cache = TRUSTED_CORRECTION_HISTORY_CACHE.get_or_init(|| Mutex::new(BTreeSet::new()));
    if cache
        .lock()
        .map_err(|_| "trusted correction history cache is unavailable".to_string())?
        .contains(&cache_key)
    {
        return Ok(());
    }

    let active_directory = git_repo_relative_path(root, &format!("{CHANGES_PATH}/{}", record.id))?;
    let archive_root = git_repo_relative_path(root, ARCHIVE_PATH)?;
    let archive_glob = format!(
        ":(glob,top){}/**/*-{}/**",
        archive_root.trim_end_matches('/'),
        record.id
    );
    let active_corrections = format!("{active_directory}/{CORRECTIONS_FILE}");
    let archive_corrections_glob = format!(
        ":(glob,top){}/**/*-{}/{}",
        archive_root.trim_end_matches('/'),
        record.id,
        CORRECTIONS_FILE
    );
    let history_exclusion = shallow
        .then(|| record.base_commit.as_ref().map(|base| format!("^{base}")))
        .flatten();
    let mut probe_args = vec![
        "rev-list".to_string(),
        "--full-history".to_string(),
        "--max-count=1".to_string(),
    ];
    probe_args.extend(references.iter().cloned());
    if let Some(exclusion) = &history_exclusion {
        probe_args.push(exclusion.clone());
    }
    probe_args.extend([
        "--".to_string(),
        active_corrections.clone(),
        archive_corrections_glob.clone(),
    ]);
    let probe_refs: Vec<&str> = probe_args.iter().map(String::as_str).collect();
    let correction_probe = run_git_bounded(root, &probe_refs, None, 1024)
        .map_err(|error| format!("failed to probe trusted correction history: {error}"))?;
    if !correction_probe.status.success() {
        return Err("failed to probe trusted correction history".into());
    }
    if correction_probe.stdout.is_empty() {
        let mut cache = cache
            .lock()
            .map_err(|_| "trusted correction history cache is unavailable".to_string())?;
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(cache_key);
        return Ok(());
    }
    let mut history_args = vec![
        "rev-list".to_string(),
        "--full-history".to_string(),
        format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1),
    ];
    history_args.extend(references.iter().cloned());
    if let Some(exclusion) = &history_exclusion {
        history_args.push(exclusion.clone());
    }
    history_args.extend([
        "--".to_string(),
        active_directory.clone(),
        archive_glob.clone(),
    ]);
    let history_refs: Vec<&str> = history_args.iter().map(String::as_str).collect();
    let output = run_git_bounded(root, &history_refs, None, MAX_GIT_COMMAND_OUTPUT_BYTES)
        .map_err(|error| format!("failed to inspect trusted correction history: {error}"))?;
    if !output.status.success() {
        return Err("failed to enumerate trusted correction history".into());
    }
    let commits: BTreeSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|commit| !commit.is_empty())
        .map(str::to_string)
        .collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err(format!(
            "trusted correction history exceeds the deterministic {}-commit bound",
            MAX_TRUSTED_HISTORY_COMMITS
        ));
    }
    for commit in commits {
        for directory in historical_change_directories(root, &commit, &record.id)? {
            let Some(anchor) =
                closing_authenticated_correction_anchor(root, &commit, &directory, &record.id)?
            else {
                continue;
            };
            if current.corrections.len() < anchor.corrections.len() {
                return Err(format!(
                    "correction history rollback detected for {}: trusted commit {} preserves {} correction(s), but the current ledger has {}",
                    record.id,
                    commit,
                    anchor.corrections.len(),
                    current.corrections.len()
                ));
            }
            for (index, historical) in anchor.corrections.iter().enumerate() {
                let historical = serde_json::to_vec(historical).map_err(|error| {
                    format!("failed to canonicalize trusted correction history: {error}")
                })?;
                let candidate =
                    serde_json::to_vec(&current.corrections[index]).map_err(|error| {
                        format!("failed to canonicalize current correction history: {error}")
                    })?;
                if candidate != historical {
                    return Err(format!(
                        "correction history divergence detected for {} at sequence {} from trusted commit {}",
                        record.id,
                        index + 1,
                        commit
                    ));
                }
            }
        }
    }
    let mut cache = cache
        .lock()
        .map_err(|_| "trusted correction history cache is unavailable".to_string())?;
    if cache.len() >= 4096 {
        cache.clear();
    }
    cache.insert(cache_key);
    Ok(())
}

fn shallow_history_is_complete_for_change(
    root: &Path,
    record: &ChangeRecord,
    references: &[String],
) -> Result<bool, String> {
    let Some(base) = record.base_commit.as_deref() else {
        return Ok(false);
    };
    let commit_object = format!("{base}^{{commit}}");
    if !Command::new("git")
        .args(["cat-file", "-e", commit_object.as_str()])
        .current_dir(root)
        .output()
        .is_ok_and(|output| output.status.success())
    {
        return Ok(false);
    }
    if !historical_change_directories(root, base, &record.id)?.is_empty() {
        return Ok(false);
    }
    if !references.iter().all(|reference| {
        Command::new("git")
            .args(["merge-base", "--is-ancestor", base, reference])
            .current_dir(root)
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }) {
        return Ok(false);
    }
    let Some(shallow_path) = git_output(root, &["rev-parse", "--git-path", "shallow"]) else {
        return Ok(false);
    };
    let shallow_path = Path::new(&shallow_path);
    let shallow_path = if shallow_path.is_absolute() {
        shallow_path.to_path_buf()
    } else {
        root.join(shallow_path)
    };
    let boundaries = fs::read_to_string(&shallow_path).map_err(|error| {
        format!(
            "failed to read shallow Git boundaries {}: {error}",
            shallow_path.display()
        )
    })?;
    for boundary in boundaries.lines().filter(|line| !line.trim().is_empty()) {
        let reachable = references.iter().any(|reference| {
            Command::new("git")
                .args(["merge-base", "--is-ancestor", boundary, reference])
                .current_dir(root)
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        });
        if reachable
            && !Command::new("git")
                .args(["merge-base", "--is-ancestor", boundary, base])
                .current_dir(root)
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn historical_change_directories(
    root: &Path,
    commit: &str,
    change_id: &str,
) -> Result<Vec<String>, String> {
    let active_state =
        git_repo_relative_path(root, &format!("{CHANGES_PATH}/{change_id}/state.json"))?;
    let archive_root = git_repo_relative_path(root, ARCHIVE_PATH)?;
    let active_state_pathspec = format!(":(top,literal){active_state}");
    let archive_root_pathspec = format!(":(top,literal){archive_root}");
    let output = Command::new("git")
        .args([
            "ls-tree",
            "-z",
            "-r",
            "--full-name",
            "--name-only",
            commit,
            "--",
            active_state_pathspec.as_str(),
            archive_root_pathspec.as_str(),
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect correction history at {commit}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect trusted correction paths at commit {commit}"
        ));
    }
    let active_directory = active_state
        .strip_suffix("/state.json")
        .unwrap_or(active_state.as_str());
    let archive_prefix = format!("{}/", archive_root.trim_end_matches('/'));
    let archive_suffix = format!("-{change_id}/state.json");
    let mut directories = BTreeSet::new();
    for path in nul_terminated_git_paths(&output.stdout, "trusted correction paths")? {
        if path == active_state {
            directories.insert(active_directory.to_string());
        } else if path.starts_with(&archive_prefix)
            && path.ends_with(&archive_suffix)
            && let Some(directory) = path.strip_suffix("/state.json")
        {
            directories.insert(directory.to_string());
        }
    }
    Ok(directories.into_iter().collect())
}

fn closing_authenticated_correction_anchor(
    root: &Path,
    commit: &str,
    directory: &str,
    change_id: &str,
) -> Result<Option<CorrectionLedger>, String> {
    let Some(state_bytes) = git_file_at_commit(root, commit, &format!("{directory}/state.json"))?
    else {
        return Ok(None);
    };
    let Ok(state) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
        return Ok(None);
    };
    if state.id != change_id
        || !matches!(state.state, ChangeState::Accepted | ChangeState::Archived)
        || !state.canonical_applied
        || state.correction_count == 0
    {
        return Ok(None);
    }
    let Some(correction_bytes) =
        git_file_at_commit(root, commit, &format!("{directory}/{CORRECTIONS_FILE}"))?
    else {
        return Ok(None);
    };
    let Ok(corrections) = serde_json::from_slice::<CorrectionLedger>(&correction_bytes) else {
        return Ok(None);
    };
    if corrections.schema_version != 1
        || corrections.corrections.len() as u64 != state.correction_count
        || validate_correction_records(&state, &corrections.corrections).is_err()
    {
        return Ok(None);
    }
    let Some(approval_bytes) =
        git_file_at_commit(root, commit, &format!("{directory}/approvals.json"))?
    else {
        return Ok(None);
    };
    let Some(verification_bytes) =
        git_file_at_commit(root, commit, &format!("{directory}/verification.json"))?
    else {
        return Ok(None);
    };
    let (Ok(approvals), Ok(verification)) = (
        serde_json::from_slice::<ApprovalLedger>(&approval_bytes),
        serde_json::from_slice::<VerificationRecord>(&verification_bytes),
    ) else {
        return Ok(None);
    };
    if !verification.passed || verification.acceptance_input_digest.is_none() {
        return Ok(None);
    }
    let correction_prefix = correction_prefix_digest(&state, &corrections.corrections)?;
    let definition_matches = resolve_definition_approval_event(
        &state,
        &approvals,
        &verification.contract_digest,
        None,
        &correction_prefix,
    )
    .is_ok();
    let closing_matches = latest_terminal_approval(&approvals)
        .is_some_and(|approval| approval.digest == closing_digest(&state, &verification));
    let contract_matches = historical_definition_digest_matches(
        root,
        commit,
        directory,
        &state,
        &corrections,
        &verification.contract_digest,
    )?;
    if !definition_matches || !closing_matches || !contract_matches {
        return Ok(None);
    }
    Ok(Some(corrections))
}

fn historical_definition_digest_matches(
    root: &Path,
    commit: &str,
    directory: &str,
    record: &ChangeRecord,
    corrections: &CorrectionLedger,
    expected: &str,
) -> Result<bool, String> {
    for explicit_false in [false, true] {
        let mut canonical_record = record.clone();
        canonical_record.state = ChangeState::Draft;
        canonical_record.canonical_applied = false;
        canonical_record.updated_at = 0;
        let mut record_bytes = serde_json::to_vec(&canonical_record)
            .map_err(|error| format!("failed to hash historical change state: {error}"))?;
        if explicit_false {
            let state = b"\"state\":\"draft\"";
            let Some(state_start) = record_bytes
                .windows(state.len())
                .position(|window| window == state)
            else {
                return Ok(false);
            };
            record_bytes.splice(
                state_start + state.len()..state_start + state.len(),
                b",\"canonical_applied\":false".iter().copied(),
            );
        }
        let effective = validate_correction_records(record, &corrections.corrections)?;
        let repo_prefix = git_repo_relative_path(root, "")?;
        let active_local_directory = format!("{CHANGES_PATH}/{}", record.id);
        let local_directory = if record.state == ChangeState::Archived {
            active_local_directory.as_str()
        } else {
            directory.strip_prefix(&repo_prefix).unwrap_or(directory)
        };
        let mut files = Vec::new();
        for artifact in &effective.selected_artifacts {
            files.push((
                format!("{directory}/{}", artifact.file_name()),
                format!("{local_directory}/{}", artifact.file_name()),
            ));
        }
        let delta_directory = format!("{directory}/deltas");
        let delta_pathspec = format!(":(top,literal){delta_directory}");
        let output = Command::new("git")
            .args([
                "ls-tree",
                "-z",
                "-r",
                "--full-name",
                "--name-only",
                commit,
                "--",
                delta_pathspec.as_str(),
            ])
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to inspect historical deltas: {error}"))?;
        if !output.status.success() {
            return Ok(false);
        }
        for path in nul_terminated_git_paths(&output.stdout, "historical delta paths")? {
            let local_path = path
                .strip_prefix(directory)
                .and_then(|suffix| suffix.strip_prefix('/'))
                .map(|suffix| format!("{local_directory}/{suffix}"))
                .unwrap_or_else(|| path.strip_prefix(&repo_prefix).unwrap_or(&path).to_string());
            files.push((path.clone(), local_path));
        }
        let policy_path = git_repo_relative_path(root, POLICY_PATH)?;
        if let Some(policy_bytes) = git_file_at_commit(root, commit, &policy_path)?
            && let Ok(policy) = serde_json::from_slice::<SddPolicy>(&policy_bytes)
            && let Some(principles) = policy.principles_file
        {
            let local_path = strict_portable_relative_path(&principles)?;
            files.push((git_repo_relative_path(root, &local_path)?, local_path));
        }
        files.sort_by(|left, right| left.1.cmp(&right.1));
        files.dedup_by(|left, right| left.1 == right.1);

        let mut current_digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
        current_digest.frame(b"record", &record_bytes);
        let mut legacy_digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
        legacy_digest.frame(b"record", &record_bytes);
        let mut complete = true;
        for (repo_path, local_path) in files {
            let Some((mode, content)) = git_entry_at_commit(root, commit, &repo_path)? else {
                complete = false;
                break;
            };
            let kind: &[u8] = match mode {
                0o120000 => b"symlink",
                0o160000 => b"gitlink",
                _ => b"file",
            };
            let canonical = canonical_definition_artifact_payload(&local_path, &content);
            current_digest.entry(&local_path, kind, mode, &canonical);
            legacy_digest.entry(&local_path, kind, mode, &content);
        }
        if !complete {
            continue;
        }
        if !corrections.corrections.is_empty() {
            let repo_path = format!("{directory}/{CORRECTIONS_FILE}");
            let local_path = format!("{local_directory}/{CORRECTIONS_FILE}");
            let Some((mode, _)) = git_entry_at_commit(root, commit, &repo_path)? else {
                continue;
            };
            current_digest.entry(
                &local_path,
                b"file",
                mode,
                json_content(corrections)?.as_bytes(),
            );
            legacy_digest.entry(
                &local_path,
                b"file",
                mode,
                json_content(corrections)?.as_bytes(),
            );
        }
        if current_digest.finish() == expected || legacy_digest.finish() == expected {
            return Ok(true);
        }
    }
    Ok(false)
}

fn git_entry_at_commit(
    root: &Path,
    commit: &str,
    path: &str,
) -> Result<Option<(u32, Vec<u8>)>, String> {
    let pathspec = format!(":(top,literal){path}");
    let output = Command::new("git")
        .args([
            "ls-tree",
            "-z",
            "--full-name",
            commit,
            "--",
            pathspec.as_str(),
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect trusted historical entry: {error}"))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Ok(None);
    }
    let mut entries = output
        .stdout
        .split(|byte| *byte == b'\0')
        .filter(|entry| !entry.is_empty());
    let Some(entry) = entries.next() else {
        return Ok(None);
    };
    if entries.next().is_some() {
        return Ok(None);
    }
    let Some(separator) = entry.iter().position(|byte| *byte == b'\t') else {
        return Ok(None);
    };
    let metadata = std::str::from_utf8(&entry[..separator])
        .map_err(|_| "invalid UTF-8 in historical Git entry metadata".to_string())?;
    let listed_path = &entry[separator + 1..];
    if listed_path != path.as_bytes() {
        return Ok(None);
    }
    let mut fields = metadata.split_whitespace();
    let Some(mode) = fields.next() else {
        return Ok(None);
    };
    let Some(kind) = fields.next() else {
        return Ok(None);
    };
    if kind != "blob" {
        return Ok(None);
    }
    let mode = u32::from_str_radix(mode, 8)
        .map_err(|_| format!("invalid historical Git mode `{mode}`"))?;
    Ok(git_file_at_commit(root, commit, path)?.map(|content| (mode, content)))
}

fn nul_terminated_git_paths(output: &[u8], context: &str) -> Result<Vec<String>, String> {
    output
        .split(|byte| *byte == b'\0')
        .filter(|path| !path.is_empty())
        .map(|path| {
            std::str::from_utf8(path)
                .map(str::to_string)
                .map_err(|_| format!("invalid UTF-8 in {context}"))
        })
        .collect()
}

fn git_file_at_commit(root: &Path, commit: &str, path: &str) -> Result<Option<Vec<u8>>, String> {
    let object = format!("{commit}:{path}");
    let output = run_git_bounded(
        root,
        &["show", object.as_str()],
        None,
        MAX_CHANGE_ARTIFACT_BYTES as usize,
    )
    .map_err(|error| format!("failed to read trusted correction history: {error}"))?;
    if output.status.success() {
        Ok(Some(output.stdout))
    } else {
        Ok(None)
    }
}

pub fn correction_history(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<CorrectionRecord>, String> {
    let ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &ledger.corrections)?;
    Ok(ledger.corrections)
}

pub fn correct_interview_metadata(
    root: &Path,
    id: &str,
    field: CorrectionField,
    value: String,
    actor: String,
    reason: String,
) -> Result<CorrectionResult, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Accepted],
        "correct accepted interview metadata",
    )?;
    if !record.canonical_applied {
        return Err("accepted metadata correction requires canonical application evidence".into());
    }
    let actor = actor.trim();
    if actor.is_empty() {
        return Err(
            "metadata correction requires a non-empty human actor passed with --actor".into(),
        );
    }
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("metadata correction requires a non-empty reason passed with --reason".into());
    }
    let corrected_value = canonical_correction_value(&value)?;
    if record.no_spec_change && field == CorrectionField::PublicContract && corrected_value == "yes"
    {
        return Err(
            "cannot correct public_contract to yes on a no-spec change; use a successor change"
                .into(),
        );
    }

    ensure_definition_approval_valid(root, &record)?;
    ensure_closing_approval_valid(root, &record)?;
    let prior_verification = load_verification(root, &record)?;
    let approvals = load_approvals(root, &record)?;
    let superseded_definition_approval =
        effective_definition_approval(root, &record, &approvals)?.clone();
    let superseded_closing_approval = latest_terminal_approval(&approvals)
        .cloned()
        .ok_or_else(|| "accepted change is missing closing approval".to_string())?;

    let mut ledger = load_correction_ledger(root, &record)?;
    let effective = validate_correction_records(&record, &ledger.corrections)?;
    let original_value = record.answers.get(field.as_str()).cloned().ok_or_else(|| {
        format!(
            "accepted change is missing original `{}` interview metadata",
            field.as_str()
        )
    })?;
    canonical_correction_value(&original_value)?;
    let prior_effective_value = effective
        .answers
        .get(field.as_str())
        .cloned()
        .ok_or_else(|| format!("effective definition is missing `{}`", field.as_str()))?;
    if canonical_correction_value(&prior_effective_value)? == corrected_value {
        return Err(format!(
            "`{}` is already `{corrected_value}`; correction must change the effective value",
            field.as_str()
        ));
    }
    let added_artifacts: Vec<ArtifactKind> = if corrected_value == "yes" {
        field
            .required_artifacts()
            .iter()
            .filter(|artifact| !effective.selected_artifacts.contains(artifact))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let mut correction = CorrectionRecord {
        schema_version: 1,
        sequence: ledger.corrections.len() as u64 + 1,
        change_id: record.id.clone(),
        field,
        original_value,
        prior_effective_value,
        corrected_value,
        actor: actor.into(),
        reason: reason.into(),
        timestamp: now(),
        prior_view_digest: effective.view_digest,
        corrected_view_digest: String::new(),
        added_artifacts,
        superseded_definition_approval,
        superseded_closing_approval,
        prior_verification,
    };
    correction.corrected_view_digest =
        corrected_view_digest(&correction.prior_view_digest, &correction)?;
    ledger.corrections.push(correction.clone());

    record.state = ChangeState::Verifying;
    record.correction_count = ledger.corrections.len() as u64;
    record.updated_at = now();
    let effective_definition = validate_correction_records(&record, &ledger.corrections)?;
    let dir = change_dir(root, &record.id);
    let mut prepared = vec![
        (dir.join(CORRECTIONS_FILE), json_content(&ledger)?),
        (dir.join("state.json"), json_content(&record)?),
        (dir.join("change.md"), change_markdown_content(&record)),
    ];
    for artifact in &correction.added_artifacts {
        let path = dir.join(artifact.file_name());
        if !path.exists() {
            prepared.push((path, artifact_template(root, artifact, &record)));
        }
    }
    write_prepared_files(root, &prepared)?;
    let corrections = ledger.corrections;
    let summary = summarize_change(root, &record);
    Ok(CorrectionResult {
        change: record,
        correction,
        effective_definition,
        corrections,
        summary,
    })
}

fn ensure_reopened_definition_unchanged(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if !record.canonical_applied {
        return Ok(());
    }
    validate_acceptance_owner_correction_records(record)?;
    let approval_ledger = load_approvals(root, record)?;
    let correction_ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &correction_ledger.corrections)?;
    if matches!(
        record.state,
        ChangeState::Implementing | ChangeState::Verifying
    ) && approval_ledger.reopenings.is_empty()
        && correction_ledger.corrections.is_empty()
        && latest_terminal_approval(&approval_ledger).is_none()
    {
        return Ok(());
    }

    let approval_position = |target: &ApprovalRecord| {
        approval_ledger.approvals.iter().rposition(|approval| {
            approval.gate == target.gate
                && approval.actor == target.actor
                && approval.timestamp == target.timestamp
                && approval.digest == target.digest
                && approval.note == target.note
        })
    };
    let correction_recovery = correction_ledger.corrections.last().and_then(|correction| {
        approval_position(&correction.superseded_closing_approval)
            .map(|position| (position, correction))
    });
    let reopen_recovery = approval_ledger.reopenings.last().and_then(|reopening| {
        approval_position(&reopening.superseded_approval).map(|position| (position, reopening))
    });

    match correction_recovery {
        Some((position, correction))
            if reopen_recovery
                .as_ref()
                .is_none_or(|(reopen_position, _)| position > *reopen_position) =>
        {
            let prior_count = correction.sequence.saturating_sub(1);
            let expected = &correction.prior_verification.contract_digest;
            if definition_digest_for_correction_count(root, record, prior_count, false)?
                == *expected
                || definition_digest_for_correction_count(root, record, prior_count, true)?
                    == *expected
            {
                return Ok(());
            }
            return Err(
                "cannot accept a correction after the previously accepted definition changed; restore prior artifacts and deltas or use a successor change"
                    .into(),
            );
        }
        _ => {}
    }

    if let Some((_, reopening)) = reopen_recovery {
        if definition_digest_matches(root, record, &reopening.prior_verification.contract_digest)? {
            return Ok(());
        }
        let mut original = record.clone();
        original.acceptance_owner_corrections.clear();
        if !record.acceptance_owner_corrections.is_empty()
            && definition_digest_matches(
                root,
                &original,
                &reopening.prior_verification.contract_digest,
            )?
        {
            validate_acceptance_owner_corrections_current(root, record)?;
            return Ok(());
        }
        return Err(
            "cannot accept a modified definition of an already-applied change; perform further spec changes in a new change workspace"
                .into(),
        );
    }

    Err(
        "cannot reaccept an already-applied change without audited reopen or correction evidence"
            .into(),
    )
}

fn is_terminal_approval(approval: &ApprovalRecord) -> bool {
    // Workflow v1 writes gate "acceptance". Same-PR finalization (workflow v2) writes
    // gate "finalization" with the same closing digest domain so reopen can supersede it.
    matches!(approval.gate.as_str(), "acceptance" | "finalization")
}

fn latest_terminal_approval(ledger: &ApprovalLedger) -> Option<&ApprovalRecord> {
    ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| is_terminal_approval(approval))
}

fn load_verification_attempts(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<VerificationRecord>, String> {
    let history_path = change_dir(root, &record.id).join("verification-attempts.json");
    if !history_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&history_path)
        .map_err(|error| format!("failed to read verification attempt history: {error}"))?;
    let history: VerificationAttemptLedger = serde_json::from_str(&content)
        .map_err(|error| format!("invalid verification attempt history: {error}"))?;
    if history.schema_version != 1 {
        return Err(format!(
            "unsupported verification attempt history schema version {}",
            history.schema_version
        ));
    }
    Ok(history.attempts)
}

/// Resolve the verification snapshot that authenticates the latest terminal closing approval.
///
/// After reopen→re-verify, `verification.json` is rewritten without acceptance fields and no
/// longer matches the closing approval. Reopen must bind to the historical attempt that the
/// closing approval actually signed, not the latest (pre-accept) re-verify tip.
fn verification_for_closing_approval(
    root: &Path,
    record: &ChangeRecord,
    closing: &ApprovalRecord,
    current: &VerificationRecord,
) -> Result<VerificationRecord, String> {
    if current.passed && closing.digest == closing_digest(record, current) {
        return Ok(current.clone());
    }
    let mut attempts = load_verification_attempts(root, record)?;
    attempts.push(current.clone());
    attempts.reverse();
    for attempt in attempts {
        if attempt.passed && closing.digest == closing_digest(record, &attempt) {
            return Ok(attempt);
        }
    }
    if record.workflow_version >= 2
        && let Ok(finalization) = load_finalization(root, record)
        && finalization.closing_digest == closing.digest
    {
        for attempt in load_verification_attempts(root, record)?
            .into_iter()
            .chain(std::iter::once(current.clone()))
            .rev()
        {
            if attempt.passed
                && finalization.contract_digest == attempt.contract_digest
                && finalization.workspace_digest == attempt.workspace_digest
                && finalization.closing_digest == closing_digest(record, &attempt)
            {
                return Ok(attempt);
            }
        }
    }
    Err(
        "accepted change closing approval does not match verification evidence; restore `verification.json` from trusted history that matches the latest acceptance/finalization approval, or re-accept after a successful reopen once delivery is stale"
            .into(),
    )
}

fn scoped_review_path(root: &Path, record: &ChangeRecord) -> PathBuf {
    change_dir(root, &record.id).join(SCOPED_REVIEW_FILE)
}

fn scoped_review_attempts_path(root: &Path, record: &ChangeRecord) -> PathBuf {
    change_dir(root, &record.id).join(SCOPED_REVIEW_ATTEMPTS_FILE)
}

fn load_scoped_review(root: &Path, record: &ChangeRecord) -> Result<ScopedReviewRecord, String> {
    let workspace = find_change_dir(root, &record.id)?;
    let path = workspace.join(SCOPED_REVIEW_FILE);
    let content = fs::read_to_string(&path)
        .map_err(|_| "independent scoped review evidence is missing".to_string())?;
    let review: ScopedReviewRecord = serde_json::from_str(&content)
        .map_err(|error| format!("invalid independent scoped review evidence: {error}"))?;
    let attempts_path = workspace.join(SCOPED_REVIEW_ATTEMPTS_FILE);
    let attempts: ScopedReviewAttemptLedger = serde_json::from_str(
        &fs::read_to_string(&attempts_path)
            .map_err(|_| "append-only scoped review history is missing".to_string())?,
    )
    .map_err(|error| format!("invalid scoped review attempt history: {error}"))?;
    validate_scoped_review_attempts(root, record, &attempts, &review)?;
    Ok(review)
}

fn validate_scoped_review_attempts(
    root: &Path,
    record: &ChangeRecord,
    attempts: &ScopedReviewAttemptLedger,
    review: &ScopedReviewRecord,
) -> Result<(), String> {
    validate_scoped_review_ledger_contents(root, record, attempts, review)?;
    validate_committed_scoped_review_history(root, record)?;
    let workspace = find_change_dir(root, &record.id)?;
    let mut historical_paths = vec![portable_project_path(
        root,
        &workspace.join(SCOPED_REVIEW_ATTEMPTS_FILE),
    )];
    if record.state == ChangeState::Archived {
        historical_paths.push(format!(
            "{CHANGES_PATH}/{}/{}",
            record.id, SCOPED_REVIEW_ATTEMPTS_FILE
        ));
    }
    for path in historical_paths {
        let repository_path = git_repo_relative_path(root, &path)?;
        let Some(bytes) = git_file_at_commit(root, "HEAD", &repository_path)? else {
            continue;
        };
        let committed: ScopedReviewAttemptLedger = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid committed scoped review history: {error}"))?;
        if committed.schema_version != 1
            || attempts.reviews.len() < committed.reviews.len()
            || attempts.reviews[..committed.reviews.len()] != committed.reviews
        {
            return Err(
                "scoped review attempt history removed or rewrote committed evidence".into(),
            );
        }
        break;
    }
    Ok(())
}

fn validate_scoped_review_ledger_contents(
    root: &Path,
    record: &ChangeRecord,
    attempts: &ScopedReviewAttemptLedger,
    review: &ScopedReviewRecord,
) -> Result<(), String> {
    if attempts.schema_version != 1
        || attempts.reviews.is_empty()
        || attempts.reviews.last() != Some(review)
    {
        return Err(
            "scoped review projection does not match its append-only attempt history".into(),
        );
    }
    let approvals = load_approvals(root, record)?;
    for attempt in &attempts.reviews {
        if attempt.schema_version != 2
            || attempt.change_id != record.id
            || validate_scoped_reviewer_claim(&attempt.reviewer).is_err()
            || !scoped_review_provenance_valid(attempt)
        {
            return Err("scoped review attempt history contains invalid evidence".into());
        }
        let scope_approver = definition_approver_for_review_contract(&approvals, attempt)?;
        if attempt.reviewer.eq_ignore_ascii_case(scope_approver) {
            return Err(
                "scoped review attempt history contains a reviewer who is also the scope approver"
                    .into(),
            );
        }
    }
    Ok(())
}

fn definition_approver_for_review_contract<'a>(
    approvals: &'a ApprovalLedger,
    review: &ScopedReviewRecord,
) -> Result<&'a str, String> {
    if let Some(approval) = approvals.approvals.iter().rev().find(|approval| {
        approval.gate == "definition"
            && (approval.digest == review.contract_digest
                || approval.definition_pair.as_ref().is_some_and(|pair| {
                    pair.current_digest == review.contract_digest
                        || pair.legacy_digest == review.contract_digest
                }))
    }) {
        return Ok(approval.actor.trim());
    }
    for adoption in approvals.scope_adoptions.iter().rev() {
        if adoption.adopted_scope_digest != review.contract_digest {
            continue;
        }
        let approval = approvals
            .approvals
            .get(adoption.source_approval_index as usize)
            .filter(|approval| approval.gate == "definition")
            .ok_or_else(|| {
                "scoped review scope-adoption approval identity is missing".to_string()
            })?;
        return Ok(approval.actor.trim());
    }
    Err("scoped review does not bind a recorded definition approval".into())
}

fn validate_committed_scoped_review_history(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    let active_attempts = git_repo_relative_path(
        root,
        &format!(
            "{CHANGES_PATH}/{}/{}",
            record.id, SCOPED_REVIEW_ATTEMPTS_FILE
        ),
    )?;
    let mut ledger_paths = vec![active_attempts];
    let add_ledger_path = |path: String, paths: &mut Vec<String>| {
        if !paths.contains(&path) {
            paths.push(path);
        }
    };
    if record.state == ChangeState::Archived {
        let workspace = find_change_dir(root, &record.id)?;
        let archive_attempts = git_repo_relative_path(
            root,
            &portable_project_path(root, &workspace.join(SCOPED_REVIEW_ATTEMPTS_FILE)),
        )?;
        add_ledger_path(archive_attempts, &mut ledger_paths);
    }
    // A change has two homes, but over a reopen round trip it occupies several archive
    // DIRECTORIES: `reopen` moves the package out of a dated directory and the next `finalize`
    // creates one for the new close. The walk below reads "the ledger is absent from every path I
    // know" as deleted evidence, so a path set built only from where the package sits right now
    // reports the reopen's own move as a deletion -- which is what refused the second reopen of
    // any change, and what refused the first one from inside `reopen` itself, since `reopen`
    // un-archives BEFORE validating and `find_change_dir` then answers with the active workspace.
    // Every directory this change's package has occupied in reachable history is admitted instead.
    // A repository whose introduction index cannot be built (a shallow clone, say) degrades to the
    // narrower set, which is the behaviour that shipped.
    if let Ok(index) = archive_introduction_index(root)
        && let Some(introductions) = index.get(&record.id)
    {
        for introduction in introductions {
            add_ledger_path(
                format!("{}/{SCOPED_REVIEW_ATTEMPTS_FILE}", introduction.directory),
                &mut ledger_paths,
            );
        }
    }
    let mut history_paths = Vec::with_capacity(ledger_paths.len() * 2);
    for ledger_path in &ledger_paths {
        history_paths.push(ledger_path.clone());
        history_paths.push(
            ledger_path
                .strip_suffix(SCOPED_REVIEW_ATTEMPTS_FILE)
                .map(|prefix| format!("{prefix}{SCOPED_REVIEW_FILE}"))
                .ok_or_else(|| "invalid scoped review history path".to_string())?,
        );
    }

    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let mut arguments = vec![
        "rev-list",
        "--reverse",
        "--full-history",
        max_count.as_str(),
        "HEAD",
        "--",
    ];
    arguments.extend(history_paths.iter().map(String::as_str));
    let commits = scoped_review_git_text(root, &arguments)
        .map_err(|_| "failed to enumerate committed scoped-review history".to_string())?;
    let commits: Vec<&str> = commits.lines().filter(|line| !line.is_empty()).collect();
    if commits.len() > limits.scoped_review_max_descendants {
        return Err(format!(
            "scoped-review history exceeds the deterministic {}-commit bound",
            limits.scoped_review_max_descendants
        ));
    }

    for commit in commits {
        let current = scoped_review_location_at_commit(root, record, commit, &ledger_paths)?;
        let parents = scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", commit])
            .map_err(|_| format!("failed to load parents for review commit {commit}"))?;
        let fields: Vec<&str> = parents.split_whitespace().collect();
        if fields.first().copied() != Some(commit) {
            return Err(format!(
                "scoped-review history commit {commit} has ambiguous identity"
            ));
        }
        if fields.len().saturating_sub(1) > limits.scoped_review_max_parents {
            return Err(format!(
                "scoped-review history commit {commit} exceeds the deterministic {}-parent bound",
                limits.scoped_review_max_parents
            ));
        }
        if fields.len() == 1 {
            let allow_collapsed_archive = collapsed_scoped_review_archive_is_terminal(
                root,
                record,
                commit,
                current.as_ref(),
            )?;
            validate_scoped_review_history_transition(
                None,
                current.as_ref(),
                allow_collapsed_archive,
            )?;
            continue;
        }
        for parent in &fields[1..] {
            let previous = scoped_review_location_at_commit(root, record, parent, &ledger_paths)?;
            let allow_collapsed_archive = previous.is_none()
                && collapsed_scoped_review_archive_is_terminal(
                    root,
                    record,
                    commit,
                    current.as_ref(),
                )?;
            validate_scoped_review_history_transition(
                previous.as_ref(),
                current.as_ref(),
                allow_collapsed_archive,
            )?;
        }
    }
    Ok(())
}

fn scoped_review_location_at_commit(
    root: &Path,
    record: &ChangeRecord,
    commit: &str,
    ledger_paths: &[String],
) -> Result<Option<(String, ScopedReviewAttemptLedger)>, String> {
    let mut found = None;
    for ledger_path in ledger_paths {
        let Some(bytes) = scoped_review_file_at_commit(root, commit, ledger_path)? else {
            continue;
        };
        if found.is_some() {
            return Err(format!(
                "scoped-review history duplicates its ledger at commit {commit}"
            ));
        }
        let attempts: ScopedReviewAttemptLedger = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid committed scoped review history: {error}"))?;
        let review_path = ledger_path
            .strip_suffix(SCOPED_REVIEW_ATTEMPTS_FILE)
            .map(|prefix| format!("{prefix}{SCOPED_REVIEW_FILE}"))
            .ok_or_else(|| "invalid scoped review history path".to_string())?;
        let review_bytes = scoped_review_file_at_commit(root, commit, &review_path)?
            .ok_or_else(|| format!("scoped-review history is missing review.json at {commit}"))?;
        let review: ScopedReviewRecord = serde_json::from_slice(&review_bytes)
            .map_err(|error| format!("invalid committed scoped review evidence: {error}"))?;
        validate_scoped_review_ledger_contents(root, record, &attempts, &review)?;
        found = Some((ledger_path.clone(), attempts));
    }
    Ok(found)
}

fn validate_scoped_review_history_transition(
    previous: Option<&(String, ScopedReviewAttemptLedger)>,
    current: Option<&(String, ScopedReviewAttemptLedger)>,
    allow_collapsed_archive: bool,
) -> Result<(), String> {
    let Some((current_path, current_ledger)) = current else {
        return Err("scoped review history deleted committed evidence".into());
    };
    let Some((previous_path, previous_ledger)) = previous else {
        // First appearance may include multiple reviews when a squash/rebase
        // introduced the ledger in one commit (e.g. post-merge history on main).
        // Append-only transitions still hold from this point forward.
        if !current_ledger.reviews.is_empty() || allow_collapsed_archive {
            return Ok(());
        }
        return Err("scoped review history did not begin with one append".into());
    };
    // A change has exactly two homes: the active workspace and the archive.
    // Evidence moves between them twice in a full round trip — `finalize`
    // carries it active -> archive, and `reopen` carries the same bytes back
    // archive -> active. Only the first direction was admitted, so the walk
    // reached a reopen commit, saw an unchanged ledger at a path it did not
    // recognise, and refused (#540). The refusal surfaced at the NEXT
    // finalize rather than at the reopen, because it comes from a walk over
    // committed history and not from the command performing the move —
    // which is also why re-running `review` could never clear it.
    //
    // A move to any third location is still refused: this admits the two
    // canonical homes, not arbitrary relocation. Archive -> archive is one of
    // them: a reopen round trip closes into a directory dated by the day of the
    // second close, which need not be the first close's directory.
    let moved_within_archive = previous_path.contains(".specsync/archive/changes/")
        && current_path.contains(".specsync/archive/changes/");
    let moved_to_archive = previous_path.contains(".specsync/changes/")
        && current_path.contains(".specsync/archive/changes/");
    let moved_to_active = previous_path.contains(".specsync/archive/changes/")
        && current_path.contains(".specsync/changes/");
    if current_path != previous_path
        && !(moved_to_archive || moved_to_active || moved_within_archive)
    {
        return Err("scoped review history moved evidence outside finalization".into());
    }
    if current_ledger == previous_ledger {
        return Ok(());
    }
    // Appending a review and moving the package can land in ONE commit: the whole
    // `review` + `finalize` pair is routinely committed together, and after a reopen the
    // previous home is already in history, so the append no longer arrives as a first
    // appearance. Growth is still append-only — every review the parent committed has to
    // survive byte-identical — and only the count restriction is relaxed across a move,
    // where a squash can legitimately collapse several attempts into one commit.
    let appended_only = current_ledger.reviews.len() > previous_ledger.reviews.len()
        && current_ledger.reviews[..previous_ledger.reviews.len()] == previous_ledger.reviews;
    if appended_only
        && (current_path != previous_path
            || current_ledger.reviews.len() == previous_ledger.reviews.len() + 1)
    {
        return Ok(());
    }
    Err("scoped review history removed or rewrote committed evidence".into())
}

fn collapsed_scoped_review_archive_is_terminal(
    root: &Path,
    record: &ChangeRecord,
    commit: &str,
    current: Option<&(String, ScopedReviewAttemptLedger)>,
) -> Result<bool, String> {
    let Some((ledger_path, ledger)) = current else {
        return Ok(false);
    };
    if ledger.reviews.len() < 2 || !ledger_path.contains(".specsync/archive/changes/") {
        return Ok(false);
    }
    let Some(directory) = ledger_path.strip_suffix(SCOPED_REVIEW_ATTEMPTS_FILE) else {
        return Ok(false);
    };
    let read =
        |name: &str| scoped_review_file_at_commit(root, commit, &format!("{directory}{name}"));
    let Some(state_bytes) = read("state.json")? else {
        return Ok(false);
    };
    let Some(accepted_bytes) = read("accepted-state.json")? else {
        return Ok(false);
    };
    let Some(finalization_bytes) = read("finalization.json")? else {
        return Ok(false);
    };
    let Some(review_bytes) = read(SCOPED_REVIEW_FILE)? else {
        return Ok(false);
    };
    let Ok(state) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
        return Ok(false);
    };
    let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&accepted_bytes) else {
        return Ok(false);
    };
    let Ok(finalization) = serde_json::from_slice::<FinalizationRecord>(&finalization_bytes) else {
        return Ok(false);
    };
    let Ok(review) = serde_json::from_slice::<ScopedReviewRecord>(&review_bytes) else {
        return Ok(false);
    };
    let review_digest = serde_json::to_vec(&review)
        .map(|bytes| sha256_hex(&bytes))
        .map_err(|error| format!("failed to hash collapsed scoped review: {error}"))?;
    Ok(state.id == record.id
        && state.state == ChangeState::Archived
        && state.workflow_version == 2
        && accepted.id == record.id
        && accepted.state == ChangeState::Accepted
        && accepted.workflow_version == 2
        && finalization.schema_version == 2
        && finalization.change_id == record.id
        && finalization.contract_digest == review.contract_digest
        && finalization.workspace_digest == review.workspace_digest
        && finalization.review_digest == review_digest
        && ledger.reviews.last() == Some(&review))
}

fn load_finalization(root: &Path, record: &ChangeRecord) -> Result<FinalizationRecord, String> {
    let path = find_change_dir(root, &record.id)?.join("finalization.json");
    let content = fs::read_to_string(&path)
        .map_err(|_| "same-PR finalization evidence is missing".to_string())?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid same-PR finalization evidence: {error}"))
}

fn finalization_digest(finalization: &FinalizationRecord) -> String {
    let mut digest = FramedDigest::new(FINALIZATION_DIGEST_DOMAIN);
    digest.frame(b"change-id", finalization.change_id.as_bytes());
    digest.frame(
        b"implementation-commit",
        finalization.implementation_commit.as_bytes(),
    );
    digest.frame(
        b"implementation-tree",
        finalization.implementation_tree.as_bytes(),
    );
    digest.frame(b"contract", finalization.contract_digest.as_bytes());
    digest.frame(b"workspace", finalization.workspace_digest.as_bytes());
    digest.frame(b"closing", finalization.closing_digest.as_bytes());
    digest.frame(b"review", finalization.review_digest.as_bytes());
    digest.finish()
}

fn validate_finalization_evidence(
    root: &Path,
    record: &ChangeRecord,
    verification: &VerificationRecord,
) -> Result<FinalizationRecord, String> {
    let finalization = load_finalization(root, record)?;
    if finalization.schema_version != 2 || finalization.change_id != record.id {
        return Err("same-PR finalization has the wrong schema or change identity".into());
    }
    if finalization.contract_digest != verification.contract_digest
        || finalization.workspace_digest != verification.workspace_digest
        || finalization.closing_digest != closing_digest(record, verification)
    {
        return Err("same-PR finalization does not bind the accepted verification evidence".into());
    }
    let implementation_expression = format!("{}^{{commit}}", finalization.implementation_commit);
    let resolved = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &implementation_expression,
        ],
    );
    let implementation_commit_available = if let Some(resolved) = resolved {
        if resolved != finalization.implementation_commit {
            return Err("finalization implementation commit is not canonical".into());
        }
        let tree_expression = format!("{}^{{tree}}", finalization.implementation_commit);
        let tree = git_output(
            root,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &tree_expression,
            ],
        )
        .ok_or_else(|| "finalization implementation tree is unavailable".to_string())?;
        if tree != finalization.implementation_tree {
            return Err("finalization implementation tree does not match its commit".into());
        }
        true
    } else if record.state == ChangeState::Archived
        && archived_finalization_tree_is_recorded(root, record)?
    {
        false
    } else {
        return Err(
            "finalization implementation commit is unavailable and no exact surviving archive tree authenticates it"
                .into(),
        );
    };
    let review = load_scoped_review(root, record)?;
    if review.schema_version != 2
        || review.change_id != record.id
        || validate_scoped_reviewer_claim(&review.reviewer).is_err()
        || !scoped_review_provenance_valid(&review)
        || review.verdict != ScopedReviewVerdict::Pass
        || review.contract_digest != verification.contract_digest
        || review.execution_digest != verification.execution_digest
        || review.workspace_digest != verification.workspace_digest
    {
        return Err("finalization scoped review does not bind the accepted verification".into());
    }
    if implementation_commit_available {
        let review_ancestor = run_git_bounded(
            root,
            &[
                "merge-base",
                "--is-ancestor",
                &review.implementation_commit,
                &finalization.implementation_commit,
            ],
            None,
            1024,
        )?;
        if !review_ancestor.status.success() {
            return Err("finalization implementation is outside the scoped-review history".into());
        }
    }
    let review_bytes = serde_json::to_vec(&review)
        .map_err(|error| format!("failed to hash scoped review: {error}"))?;
    if sha256_hex(&review_bytes) != finalization.review_digest {
        return Err("finalization scoped-review digest does not match review.json".into());
    }
    if finalization_digest(&finalization) != finalization.finalization_digest {
        return Err("same-PR finalization digest is invalid".into());
    }
    Ok(finalization)
}

fn scoped_review_is_current(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> bool {
    review.schema_version == 2
        && review.change_id == record.id
        && validate_scoped_reviewer_claim(&review.reviewer).is_ok()
        && scoped_review_provenance_valid(review)
        && review.verdict == ScopedReviewVerdict::Pass
        && review_commit_is_current(root, record, review)
        && definition_digest_matches(root, record, &review.contract_digest).unwrap_or(false)
        && (record.workflow_version < 2
            || review.execution_digest.as_deref() == execution_digest(root, record).ok().as_deref())
        && project_input_digest(root).as_deref() == Ok(review.workspace_digest.as_str())
}

fn review_commit_is_current(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> bool {
    review_commit_is_current_checked(root, record, review).is_ok()
}

fn run_scoped_review_git(root: &Path, args: &[&str]) -> Result<BoundedCommandOutput, String> {
    let limits = lifecycle_validation_limits();
    let mut command = configured_git_command(root);
    run_git_command_bounded_with_deadline(
        &mut command,
        args,
        None,
        limits.git_max_output_bytes,
        Duration::from_secs(limits.git_timeout_seconds),
    )
}

fn scoped_review_file_at_commit(
    root: &Path,
    commit: &str,
    path: &str,
) -> Result<Option<Vec<u8>>, String> {
    let object = format!("{commit}:{path}");
    let output = run_scoped_review_git(root, &["show", object.as_str()])?;
    if output.status.success() {
        Ok(Some(output.stdout))
    } else {
        Ok(None)
    }
}

fn scoped_review_git_text(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = run_scoped_review_git(root, args)?;
    if !output.status.success() {
        return Err(format!(
            "scoped-review Git query failed: git {}",
            args.join(" ")
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| "scoped-review Git query returned non-UTF-8 output".to_string())
}

fn review_commit_is_current_checked(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> Result<(), String> {
    let head = scoped_review_git_text(root, &["rev-parse", "--verify", "HEAD^{commit}"])
        .map_err(|_| "HEAD does not resolve to a commit".to_string())?;
    if review.implementation_commit == head {
        return Ok(());
    }
    let ancestor = run_scoped_review_git(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &review.implementation_commit,
            &head,
        ],
    )?;
    if !ancestor.status.success() {
        return Err("scoped-review commit is not an ancestor of HEAD".into());
    }
    let range = format!("{}..{head}", review.implementation_commit);
    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let commits = scoped_review_git_text(root, &["rev-list", "--reverse", &max_count, &range])
        .map_err(|_| "failed to enumerate scoped-review descendants".to_string())?;
    if commits.lines().filter(|line| !line.is_empty()).count()
        > limits.scoped_review_max_descendants
    {
        return Err(format!(
            "scoped-review descendant history exceeds the deterministic {}-commit bound",
            limits.scoped_review_max_descendants
        ));
    }
    let allowed_prefix = git_repo_relative_path(root, &format!("{CHANGES_PATH}/{}/", record.id))?;
    for descendant in commits.lines().filter(|line| !line.is_empty()) {
        let parents =
            scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", descendant])
                .map_err(|_| {
                    format!("failed to load parents for review descendant {descendant}")
                })?;
        let fields: Vec<&str> = parents.split_whitespace().collect();
        if fields.first().copied() != Some(descendant) || fields.len() < 2 {
            return Err(format!(
                "scoped-review descendant {descendant} has ambiguous parent history"
            ));
        }
        if fields.len() - 1 > limits.scoped_review_max_parents {
            return Err(format!(
                "scoped-review descendant {descendant} exceeds the deterministic {}-parent bound",
                limits.scoped_review_max_parents
            ));
        }
        for parent in &fields[1..] {
            let output = run_scoped_review_git(
                root,
                &[
                    "diff-tree",
                    "--no-commit-id",
                    "--name-only",
                    "--no-renames",
                    "-z",
                    "-r",
                    parent,
                    descendant,
                ],
            )?;
            if !output.status.success() {
                return Err(format!(
                    "failed to inspect scoped-review descendant {descendant}"
                ));
            }
            for raw_path in output
                .stdout
                .split(|byte| *byte == 0)
                .filter(|path| !path.is_empty())
            {
                let path = std::str::from_utf8(raw_path)
                    .map_err(|_| "scoped-review descendant contains a non-UTF-8 path")?;
                let relative = path.strip_prefix(&allowed_prefix).ok_or_else(|| {
                    format!("scoped-review descendant changed disallowed path `{path}`")
                })?;
                if !matches!(
                    relative,
                    SCOPED_REVIEW_FILE
                        | SCOPED_REVIEW_ATTEMPTS_FILE
                        | "state.json"
                        | "verification.json"
                        | "verification-attempts.json"
                ) {
                    return Err(format!(
                        "scoped-review descendant changed disallowed path `{path}`"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_verification_for_commit_binding(
    root: &Path,
    record: &ChangeRecord,
    verification: &VerificationRecord,
    require_full_history: bool,
) -> Result<(), String> {
    if require_full_history {
        verification_is_current_checked(root, record, verification)?;
    } else {
        if !verification.passed {
            return Err("latest verification evidence failed".into());
        }
        if !definition_digest_matches(root, record, &verification.contract_digest)? {
            return Err("verification contract digest is stale".into());
        }
        validate_verification_execution_digest(root, record, verification)?;
        if verification.workspace_digest != project_input_digest(root)? {
            return Err("verification project-input digest is stale".into());
        }
    }
    // Commit identity and ancestry are deliberately not checked here. `verification.commit`
    // remains recorded as an informational correlation key — it is what `attest` keys its
    // signed records to — but binding evidence to a commit, and requiring that commit to be
    // an ancestor of the implementation, is a history-trust question rather than a content
    // one. It is also what made squash-merged changes permanently unfinalizable: the squash
    // discards the recorded commit, so the ancestry check could never pass again.
    Ok(())
}

fn validate_verification_execution_digest(
    root: &Path,
    record: &ChangeRecord,
    verification: &VerificationRecord,
) -> Result<(), String> {
    if record.workflow_version < 2 {
        return Ok(());
    }
    let expected = execution_digest(root, record)?;
    match verification.execution_digest.as_deref() {
        Some(actual) if actual == expected => Ok(()),
        Some(_) => Err("verification execution/evidence digest is stale".into()),
        None => Err("verification execution/evidence digest is missing".into()),
    }
}

pub fn record_scoped_review(
    root: &Path,
    id: &str,
    reviewer: String,
) -> Result<ScopedReviewRecord, String> {
    record_scoped_review_with_verdict(root, id, reviewer, ScopedReviewVerdict::Pass)
}

pub fn record_scoped_review_with_verdict(
    root: &Path,
    id: &str,
    reviewer: String,
    verdict: ScopedReviewVerdict,
) -> Result<ScopedReviewRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Verifying], "record scoped review")?;
    ensure_definition_approval_valid(root, &record)?;
    let verification = load_verification(root, &record)?;
    let reviewer = validate_scoped_reviewer_claim(&reviewer)?;
    let approvals = load_approvals(root, &record)?;
    let scope_approver = effective_definition_approval(root, &record, &approvals)?
        .actor
        .trim();
    if reviewer.eq_ignore_ascii_case(scope_approver) {
        return Err(
            "scoped review must be recorded by someone other than the scope approver".into(),
        );
    }
    let current_commit = git_output(root, &["rev-parse", "HEAD"])
        .ok_or_else(|| "scoped review requires a committed implementation".to_string())?;
    validate_verification_for_commit_binding(root, &record, &verification, true).map_err(|error| {
        format!("scoped review cannot bind stale verification ({error}); run `specsync change check` first")
    })?;
    let attempts_path = scoped_review_attempts_path(root, &record);
    let previous_review = attempts_path
        .exists()
        .then(|| load_scoped_review(root, &record))
        .transpose()?;
    let mut attempts = if attempts_path.exists() {
        serde_json::from_str::<ScopedReviewAttemptLedger>(
            &fs::read_to_string(&attempts_path)
                .map_err(|error| format!("failed to read scoped review history: {error}"))?,
        )
        .map_err(|error| format!("invalid scoped review attempt history: {error}"))?
    } else {
        ScopedReviewAttemptLedger::default()
    };
    if attempts.schema_version != 1 {
        return Err("scoped review attempt history has an unsupported schema".into());
    }
    let contract_digest = definition_digest(root, &record)?;
    let execution_digest = execution_digest(root, &record)?;
    let workspace_digest = verification.workspace_digest.clone();
    let implementation_commit = previous_review
        .filter(|previous| {
            previous.schema_version == 2
                && previous.change_id == record.id
                && previous.contract_digest == contract_digest
                && previous.execution_digest.as_deref() == Some(execution_digest.as_str())
                && previous.workspace_digest == workspace_digest
                && review_commit_is_current_checked(root, &record, previous).is_ok()
        })
        .map(|previous| previous.implementation_commit)
        .unwrap_or(current_commit);
    let review = ScopedReviewRecord {
        schema_version: 2,
        change_id: record.id.clone(),
        reviewer: reviewer.to_string(),
        provenance: ScopedReviewProvenanceV1 {
            schema_version: 1,
            provider: ScopedReviewProvenanceProvider::GithubActionsCheck,
            required_check: SCOPED_REVIEW_REQUIRED_CHECK.into(),
        },
        verdict,
        implementation_commit,
        contract_digest,
        execution_digest: Some(execution_digest),
        workspace_digest,
        timestamp: now(),
    };
    attempts.reviews.push(review.clone());
    write_prepared_files(
        root,
        &[
            (scoped_review_path(root, &record), json_content(&review)?),
            (attempts_path, json_content(&attempts)?),
        ],
    )?;
    Ok(review)
}

pub fn accept_change(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    accept_change_with_gate(root, id, actor, note, "acceptance", false, false)
}

fn accept_change_with_gate(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
    gate: &str,
    allow_verified_tree_adoption: bool,
    require_scoped_review: bool,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Verifying], "accept the change")?;
    ensure_definition_approval_valid(root, &record)?;
    let mut verification = load_verification(root, &record)?;
    let current_commit = git_output(root, &["rev-parse", "HEAD"]);
    validate_verification_for_commit_binding(root, &record, &verification, require_scoped_review)
        .map_err(|error| format!("cannot accept stale verification: {error}"))?;
    let mut verification_adopted = false;
    if verification.commit != current_commit {
        if allow_verified_tree_adoption && current_commit.is_some() {
            verification.commit = current_commit.clone();
            verification_adopted = true;
        } else {
            return Err(
                "verification is stale because HEAD changed; run `specsync change check` again"
                    .into(),
            );
        }
    }
    if require_scoped_review {
        let review = load_scoped_review(root, &record)?;
        if !scoped_review_is_current(root, &record, &review) {
            return Err(
                "independent scoped review is stale; open or update the PR so `SpecSync scoped review` can run"
                    .into(),
            );
        }
    }
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    validate_delta_files(root, &record)?;
    // Acceptance is the second place deltas reach the canonical spec (`prepare_delta_application`
    // below, when `check` never materialized them), so it gets the same refusal.
    ensure_approved_delta_bodies_unchanged(root, &record)?;
    let records = list_changes_checked(root)?;
    // Same as verification: acceptance reports only failures, and the project
    // surfaces report every suppression this gate applied.
    if let Some(errors) = validate_effective_contracts(root, &records).error_text() {
        return Err(errors);
    }
    ensure_reopened_definition_unchanged(root, &record)?;
    let mut prepared = if record.canonical_applied {
        Vec::new()
    } else {
        prepare_delta_application(root, &record)?
    };
    let manifest = acceptance_manifest(root, &record, &prepared)?;
    let succession = build_semantic_succession_evidence(root, &record, &manifest)?;
    verification.acceptance_input_digest = Some(acceptance_manifest_digest(&manifest)?);
    verification.acceptance_manifest = Some(manifest);
    verification.semantic_succession = (!succession.tuples.is_empty()).then_some(succession);
    let closing_digest = closing_digest(&record, &verification);
    let mut ledger = load_approvals(root, &record)?;
    if gate == "finalization" {
        if actor.is_some() || note.is_some() {
            return Err(
                "automatic finalization does not accept a second approval actor or note".into(),
            );
        }
        // Persist a terminal ledger entry so reopen can supersede the closing digest after
        // archive-only tips or interrupted finalizations. Gate "finalization" is terminal.
        ledger.approvals.push(ApprovalRecord {
            gate: "finalization".into(),
            actor: "specsync:finalization".into(),
            timestamp: now(),
            digest: closing_digest.clone(),
            note: Some("Same-PR finalization closing digest".into()),
            definition_pair: None,
            approved_scope: None,
            scope_migration: None,
            approved_delta_digests: None,
        });
        let approvals_path = change_dir(root, &record.id).join("approvals.json");
        prepared.push((approvals_path, json_content(&ledger)?));
    } else {
        let actor = resolve_actor(root, actor)?;
        let stable_definition_digest = definition_digest(root, &record)?;
        let definition_is_stable = effective_definition_approval(root, &record, &ledger)?.digest
            == stable_definition_digest;
        if !definition_is_stable {
            ledger.approvals.push(ApprovalRecord {
                gate: "definition".into(),
                actor: actor.clone(),
                timestamp: now(),
                digest: stable_definition_digest,
                note: Some(
                    "Normalized compatible definition evidence during explicit acceptance".into(),
                ),
                definition_pair: None,
                approved_scope: None,
                scope_migration: None,
                // This is a definition gate, so it carries the delta binding forward rather than
                // dropping it. The bodies it names were already checked against the superseded
                // approval earlier in this same call, so it can only record verified wording.
                approved_delta_digests: Some(delta_body_digests(root, &record)?),
            });
        }
        ledger.approvals.push(ApprovalRecord {
            gate: gate.into(),
            actor,
            timestamp: now(),
            digest: closing_digest.clone(),
            note,
            definition_pair: None,
            approved_scope: None,
            scope_migration: None,
            approved_delta_digests: None,
        });
        let approvals_path = change_dir(root, &record.id).join("approvals.json");
        prepared.push((approvals_path, json_content(&ledger)?));
    }
    prepared.push((
        change_dir(root, &record.id).join("verification.json"),
        json_content(&verification)?,
    ));
    // Always append the acceptance-bound verification so reopen can recover when the tip
    // verification.json is later rewritten without the signed acceptance fields.
    {
        let attempts_path = change_dir(root, &record.id).join("verification-attempts.json");
        let mut attempts = if attempts_path.exists() {
            let history: VerificationAttemptLedger = serde_json::from_str(
                &fs::read_to_string(&attempts_path)
                    .map_err(|error| format!("failed to read verification attempts: {error}"))?,
            )
            .map_err(|error| format!("invalid verification attempt history: {error}"))?;
            if history.schema_version != 1 {
                return Err(format!(
                    "unsupported verification attempt history schema version {}",
                    history.schema_version
                ));
            }
            if verification_adopted && history.attempts.is_empty() {
                return Err(
                    "verification attempt history cannot adopt the implementation commit".into(),
                );
            }
            history
        } else {
            VerificationAttemptLedger {
                schema_version: 1,
                attempts: Vec::new(),
            }
        };
        attempts.attempts.push(verification.clone());
        prepared.push((attempts_path, json_content(&attempts)?));
    }
    if gate == "finalization" {
        let review = load_scoped_review(root, &record)?;
        let implementation_commit = current_commit
            .clone()
            .ok_or_else(|| "finalization requires a committed implementation".to_string())?;
        let implementation_tree = git_output(
            root,
            &[
                "rev-parse",
                "--verify",
                &format!("{implementation_commit}^{{tree}}"),
            ],
        )
        .ok_or_else(|| "finalization requires a committed implementation tree".to_string())?;
        let review_bytes = serde_json::to_vec(&review)
            .map_err(|error| format!("failed to hash scoped review: {error}"))?;
        let review_digest = sha256_hex(&review_bytes);
        let mut finalization = FinalizationRecord {
            schema_version: 2,
            change_id: record.id.clone(),
            implementation_commit,
            implementation_tree,
            contract_digest: verification.contract_digest.clone(),
            workspace_digest: verification.workspace_digest.clone(),
            closing_digest,
            review_digest,
            finalization_digest: String::new(),
            timestamp: now(),
        };
        finalization.finalization_digest = finalization_digest(&finalization);
        prepared.push((
            change_dir(root, &record.id).join("finalization.json"),
            json_content(&finalization)?,
        ));
    }
    record.state = ChangeState::Accepted;
    record.canonical_applied = true;
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

pub fn finalize_change(root: &Path, id: &str) -> Result<PathBuf, String> {
    // Recover a process-interrupted archive transaction before inspecting the
    // state that determines whether finalization can resume.
    drop(acquire_project_lock(root)?);
    let record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Verifying, ChangeState::Accepted],
        "finalize the change",
    )?;
    if record.workflow_version < 2 {
        return Err(format!(
            "{} uses the legacy workflow; after reopen run `specsync change accept {} --actor <name>` then `specsync change archive {}` (do not use finalize — it would write finalization evidence without a matching acceptance closing approval)",
            record.id, record.id, record.id
        ));
    }
    if !record.canonical_applied {
        return Err(
            "canonical deltas are not materialized; run `specsync change check` before review"
                .into(),
        );
    }
    if record.state == ChangeState::Verifying {
        accept_change_with_gate(root, id, None, None, "finalization", true, true)?;
    } else {
        let verification = load_verification(root, &record)?;
        validate_finalization_evidence(root, &record, &verification)
            .map_err(|error| format!("cannot resume interrupted same-PR finalization: {error}"))?;
    }
    let archive = archive_change_with_options(root, id, false, true)?;
    // Archival is the only place the system COMPOUNDS rather than merely records: knowledge
    // moves from the change into the spec. Assemble the material here; the agent that just ran
    // `finalize` writes the lessons, guided by `next_action`. A bundle failure must not undo a
    // completed archival, so this is best-effort.
    let _ = write_lesson_bundle(root, &record, &archive);
    Ok(archive)
}

pub fn archive_change(root: &Path, id: &str) -> Result<PathBuf, String> {
    archive_change_with_options(root, id, false, false)
}

#[cfg(test)]
fn archive_change_with_finalize_failure(
    root: &Path,
    id: &str,
    force_finalize_failure: bool,
) -> Result<PathBuf, String> {
    archive_change_with_options(root, id, force_finalize_failure, false)
}

#[cfg(test)]
fn archive_change_with_same_pr_finalize_failure(root: &Path, id: &str) -> Result<PathBuf, String> {
    archive_change_with_options(root, id, true, true)
}

/// The body of a Markdown artifact, with YAML frontmatter removed.
///
/// This is NO LONGER identical to `view::strip_frontmatter`: that one is LF-only and rejects a
/// closer at EOF, both of which this accepts. Unifying them (#696) is therefore a behaviour
/// change for `view`, not the no-op it would have been before CRLF support landed here. Said
/// plainly because the previous version of this comment claimed the no-op and stopped being true
/// the moment this function changed.
///
/// Frontmatter ends at its CLOSING delimiter, never at the next `---` anywhere in the document.
/// Splitting on the delimiter and taking the third field looks equivalent and is not: a body with
/// a horizontal rule loses everything after it, and lost lessons read exactly like lessons nobody
/// ever wrote.
fn strip_frontmatter(text: &str) -> &str {
    // A leading UTF-8 BOM must not hide the opening delimiter.
    let text = text.trim_start_matches('\u{feff}');
    // BOTH line endings, because a Windows-authored companion otherwise keeps its frontmatter and
    // every untouched scaffold reports itself as recorded knowledge.
    //
    // Note this diverges from the repository's usual convention, which is normalize-then-parse:
    // `parser.rs` is LF-only (`^---\n`) and ~28 call sites do `.replace("\r\n", "\n")` before
    // reaching it. Handling CRLF here instead keeps the borrowed `&str` return — normalizing
    // would force an allocation and a signature change — but it does mean this is a parser with
    // its own dialect. #696 should decide which convention wins repo-wide.
    let after_open = if let Some(rest) = text.strip_prefix("---\r\n") {
        rest
    } else if let Some(rest) = text.strip_prefix("---\n") {
        rest
    } else {
        return text;
    };
    // Frontmatter ends at its CLOSING delimiter LINE, never at the next `---` anywhere in the
    // document: `---` is a legal Markdown horizontal rule, and a body that loses everything after
    // one is indistinguishable from a body nobody wrote.
    let mut offset = 0usize;
    for line in after_open.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return &after_open[offset + line.len()..];
        }
        offset += line.len();
    }
    // Unterminated frontmatter: keep the whole document rather than guess where it ended.
    text
}

/// The change a bare lifecycle command acts on: the single active approved/implementing/verifying
/// record — the same states `check_change` selects.
///
/// Policy, so it lives here. The command layer had this resolution inline in one arm and not in
/// another, which is exactly how the build-stage lesson hint came to be silent for the bare
/// `change check` while the success path resolved the ID fine.
pub(crate) fn active_change_id(root: &Path) -> Option<String> {
    list_changes(root)
        .ok()?
        .records
        .into_iter()
        .find(|record| {
            // The SAME states `check_change` selects. A narrower set here is how the hint came
            // to be silent on a failing first check of an approved change: two selections of
            // "the current change" that disagree.
            matches!(
                record.state,
                ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
            )
        })
        .map(|record| record.id)
}

/// Where a module's accumulated lessons live.
///
/// One definition of the convention, so surfacing at `new` and folding at `finalize` can never
/// disagree about which file they mean.
pub(crate) fn module_context_path(module: &str) -> String {
    format!("specs/{module}/context.md")
}

/// What these modules have already learned, as substantive line counts.
///
/// Policy lives in the domain, never in the command layer: `specs/cmd_change/context.md` states
/// that rule three separate ways, and deciding what counts as a lesson IS policy. The command
/// layer renders what this returns and decides nothing.
///
/// A freshly generated scaffold must not advertise itself as knowledge. The scaffold is NOT
/// HTML comments — `specs/<module>/context.md` is generated with plain prompt bullets — so the
/// generator is asked which lines it wrote rather than this module guessing at their shape.
/// Guessing is how the copy drifts from the template it is supposed to track.
///
/// Frontmatter is stripped by its delimiters rather than by "contains a colon" — real lessons are
/// full of colons, and filtering on one would silently drop the very content this exists to
/// surface.
///
/// An unreadable context file yields no entry rather than an error: this is an authoring
/// affordance, and it must never be able to fail a lifecycle command.
pub(crate) fn accumulated_lessons(root: &Path, modules: &[String]) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    for module in modules {
        let relative = module_context_path(module);
        let Ok(text) = fs::read_to_string(root.join(&relative)) else {
            continue;
        };
        let scaffold = crate::generator::generated_context_scaffold(module);
        let scaffold_lines: std::collections::HashSet<&str> =
            scaffold.lines().map(str::trim).collect();
        let body = strip_frontmatter(&text);
        let substantive = body
            .lines()
            .filter(|line| {
                let line = line.trim();
                !line.is_empty()
                    && !line.starts_with("<!--")
                    && !line.starts_with('#')
                    && !scaffold_lines.contains(line)
            })
            .count();
        if substantive > 0 {
            found.push((relative, substantive));
        }
    }
    found
}

/// The context files this change's lessons should be folded into at archival.
///
/// Returns empty when the change cannot be loaded or owns no specs, which the caller renders as
/// the plain merge instruction. Same reason as above: never fail a completed finalize over an
/// authoring affordance.
pub(crate) fn lesson_fold_targets(root: &Path, id: &str) -> Vec<String> {
    load_change(root, id)
        .map(|record| {
            record
                .affected_specs
                .iter()
                .map(|module| module_context_path(module))
                .collect()
        })
        .unwrap_or_default()
}

/// Assemble the material an agent needs to fold this change's lessons into the SPEC's context.
///
/// Lessons belong in `specs/<module>/context.md`, not in the change — a per-change lessons file
/// dies with the change, and the point is that a module accumulates what was learned about it
/// across every change that touched it (docs/6-0-findings.md, finding 10).
///
/// This does not write the lessons. SpecSync must not shell out to a particular agent, and it
/// does not need to: the agent driving the lifecycle just ran `finalize`. So `finalize` assembles
/// the bundle and `next_action` names the step. Neither blocking nor nagging.
///
/// Everything here is read from disk. No network, so `finalize` keeps working offline and in CI.
fn write_lesson_bundle(root: &Path, record: &ChangeRecord, archive: &Path) -> Result<(), String> {
    let mut out = String::new();
    out.push_str(&format!("# Lesson bundle — {}\n\n", record.id));
    out.push_str(
        "Material for folding this change's lessons into the affected specs' `context.md`.\n\
         Synthesise from what actually happened below; do not restate the change description.\n\n",
    );

    out.push_str("## What this change was\n\n");
    out.push_str(&format!("- **Title**: {}\n", record.title));
    out.push_str(&format!("- **Kind**: {:?}\n", record.kind));
    if !record.affected_specs.is_empty() {
        out.push_str(&format!(
            "- **Specs**: {}\n",
            record.affected_specs.join(", ")
        ));
    }
    if !record.affected_paths.is_empty() {
        out.push_str(&format!(
            "- **Paths**: {}\n",
            record.affected_paths.join(", ")
        ));
    }
    for criterion in &record.acceptance_criteria {
        out.push_str(&format!("- **Acceptance**: {criterion}\n"));
    }
    out.push('\n');

    if let Ok(verification) = load_verification(root, record) {
        out.push_str("## Evidence\n\n");
        if let Some(commit) = &verification.commit {
            out.push_str(&format!("- Verification commit: `{commit}`\n"));
        }
        if let Some(base) = &record.base_commit {
            out.push_str(&format!("- Base commit: `{base}`\n"));
        }
        let commands: Vec<&str> = verification
            .commands
            .iter()
            .map(|command| command.command.as_str())
            .collect();
        if !commands.is_empty() {
            out.push_str(&format!("- Verified by: `{}`\n", commands.join("`, `")));
        }
        out.push('\n');
    }

    // The change's own working record is the richest source and is exactly what would otherwise
    // be lost: it is archived with the change and read by nobody afterwards.
    for artifact in ["context", "design", "testing"] {
        let path = archive.join(format!("{artifact}.md"));
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let body = strip_frontmatter(&text).trim().to_string();
        if body.is_empty() {
            continue;
        }
        out.push_str(&format!("## From the change's {artifact}.md\n\n"));
        out.push_str(&body);
        out.push_str("\n\n");
    }

    out.push_str("## Where these lessons go\n\n");
    if record.affected_specs.is_empty() {
        out.push_str(
            "This change declared no affected specs, so there is no module context to fold into.\n",
        );
    } else {
        for module in &record.affected_specs {
            out.push_str(&format!("- `specs/{module}/context.md`\n"));
        }
    }

    // Durable like every other artifact this lifecycle writes: a crash mid-write would leave a
    // TRUNCATED bundle, and truncated lessons read exactly like lessons nobody wrote — the same
    // failure this whole change exists to prevent.
    atomic_write_durable(&archive.join(LESSON_BUNDLE_FILE), out.as_bytes())
        .map_err(|error| format!("failed to write lesson bundle: {error}"))
}

fn archive_change_with_options(
    root: &Path,
    id: &str,
    force_finalize_failure: bool,
    allow_delivery_diff: bool,
) -> Result<PathBuf, String> {
    let _lock = acquire_project_lock(root)?;
    list_changes_checked(root)?;
    let located = find_change_dir(root, id)?;
    let record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Accepted], "archive the change")?;
    validate_acceptance_owner_correction_records(&record)?;
    let source = change_dir(root, &record.id);
    let archive_root = root.join(ARCHIVE_PATH);
    let moved_destination =
        (!source.exists() && located.parent() == Some(archive_root.as_path()) && located.is_dir())
            .then_some(located);
    let resume_post_move = moved_destination.is_some();
    let destination = moved_destination
        .unwrap_or_else(|| archive_root.join(format!("{}-{}", today(), record.id)));
    if destination.exists() && !resume_post_move {
        return Err(format!(
            "archive destination already exists: {}",
            destination.display()
        ));
    }
    ensure_closing_approval_valid(root, &record)?;
    if !allow_delivery_diff
        && let Some(policy) = load_policy_checked(root)?
        && policy.enabled
        && policy.require_change_for_meaningful_files
    {
        let delivery_diff = uncovered_meaningful_paths(root, &policy, &[])?;
        if delivery_diff
            .iter()
            .any(|path| record_covers_project_path(root, &record, path))
        {
            return Err(
                "cannot archive while this delivery diff still depends on the change for path coverage; archive after merge"
                    .into(),
            );
        }
    }
    let current_location = if resume_post_move {
        destination.as_path()
    } else {
        source.as_path()
    };
    let original_state_bytes = fs::read(current_location.join("state.json"))
        .map_err(|error| format!("failed to preserve accepted state before archive: {error}"))?;
    let original_markdown_bytes = fs::read(current_location.join("change.md"))
        .map_err(|error| format!("failed to preserve accepted change before archive: {error}"))?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    // The package is being closed out of the change's own active workspace by this process, so
    // the closing evidence under `current_location` may speak for a generation history has not
    // seen yet -- but only here, and only under `closing_ledger_extends_all_introductions`. A
    // post-move resume found its package already in the archive and gets no such token, because
    // that shape is what an attacker reanimating a committed package presents.
    let pending_close = (!resume_post_move).then_some(PendingArchiveClose {
        package: current_location,
    });
    let accepted_snapshot = current_location.join("accepted-state.json");
    let accepted_state_bytes =
        authenticated_accepted_transition_for(root, &record, pending_close.as_ref())
            .map(|(_, bytes, _)| bytes)
            .unwrap_or_else(|_| original_state_bytes.clone());
    atomic_write_durable(&accepted_snapshot, &accepted_state_bytes)
        .map_err(|error| format!("failed to stage authenticated accepted state: {error}"))?;
    let mut simulated = list_all_changes_checked(root)?;
    let mut archived_projection = record.clone();
    archived_projection.state = ChangeState::Archived;
    if let Err(error) = match pending_close.as_ref() {
        Some(pending) => validate_archived_integrity_closing(root, &archived_projection, pending),
        None => validate_archived_integrity(root, &archived_projection),
    } {
        let _ = remove_file_durable(&accepted_snapshot);
        return Err(format!(
            "archive target historical-integrity preflight failed: {error}"
        ));
    }
    simulated.insert(record.id.clone(), archived_projection);
    for candidate in simulated
        .values()
        .filter(|candidate| candidate.state == ChangeState::Accepted)
    {
        let mut visiting = BTreeSet::new();
        let mut memo = BTreeMap::new();
        if let Err(error) = validate_accepted_inputs_recursive(
            root,
            candidate,
            &simulated,
            &mut visiting,
            &mut memo,
        ) {
            let _ = remove_file_durable(&accepted_snapshot);
            return Err(format!(
                "archive post-move preflight would invalidate `{}`: {error}",
                candidate.id
            ));
        }
    }
    if !resume_post_move && let Err(error) = rename_durable(&source, &destination) {
        let _ = remove_file_durable(&accepted_snapshot);
        return Err(format!(
            "failed to archive {} to {}: {error}",
            source.display(),
            destination.display()
        ));
    }
    let mut archived = record.clone();
    archived.state = ChangeState::Archived;
    archived.updated_at = now();
    let finalize = if force_finalize_failure {
        Err("forced post-move archive finalization failure".to_string())
    } else {
        json_content(&archived).and_then(|state| {
            write_prepared_files(
                root,
                &[
                    (destination.join("state.json"), state),
                    (
                        destination.join("change.md"),
                        change_markdown_content(&archived),
                    ),
                ],
            )
        })
    };
    if let Err(error) = finalize {
        if resume_post_move {
            let restore =
                atomic_write_durable(&destination.join("state.json"), &original_state_bytes)
                    .and_then(|()| {
                        atomic_write_durable(
                            &destination.join("change.md"),
                            &original_markdown_bytes,
                        )
                    });
            return match restore {
                Ok(()) => Err(format!(
                    "failed to finalize archive; post-move destination remains retryable: {error}"
                )),
                Err(restore_error) => Err(format!(
                    "failed to finalize archive ({error}) and restore retryable destination ({restore_error})"
                )),
            };
        }
        let restore = atomic_write_durable(&destination.join("state.json"), &original_state_bytes)
            .and_then(|()| {
                atomic_write_durable(&destination.join("change.md"), &original_markdown_bytes)
            })
            .and_then(|()| remove_file_durable(&destination.join("accepted-state.json")))
            .and_then(|()| rename_durable(&destination, &source));
        return match restore {
            Ok(()) => Err(format!(
                "failed to finalize archive; source restored: {error}"
            )),
            Err(restore_error) => Err(format!(
                "failed to finalize archive ({error}) and restore source ({restore_error})"
            )),
        };
    }
    let moved_close = (!resume_post_move).then_some(PendingArchiveClose {
        package: &destination,
    });
    if let Err(error) = match moved_close.as_ref() {
        Some(pending) => validate_archived_integrity_closing(root, &archived, pending),
        None => validate_archived_integrity(root, &archived),
    } {
        if resume_post_move {
            let restore =
                atomic_write_durable(&destination.join("state.json"), &original_state_bytes)
                    .and_then(|()| {
                        atomic_write_durable(
                            &destination.join("change.md"),
                            &original_markdown_bytes,
                        )
                    });
            return match restore {
                Ok(()) => Err(format!(
                    "archived evidence failed validation; post-move destination remains retryable: {error}"
                )),
                Err(restore_error) => Err(format!(
                    "archived evidence failed validation ({error}) and restore retryable destination ({restore_error})"
                )),
            };
        }
        let restore = atomic_write_durable(&destination.join("state.json"), &original_state_bytes)
            .and_then(|()| {
                atomic_write_durable(&destination.join("change.md"), &original_markdown_bytes)
            })
            .and_then(|()| remove_file_durable(&destination.join("accepted-state.json")))
            .and_then(|()| rename_durable(&destination, &source));
        return match restore {
            Ok(()) => Err(format!(
                "archived evidence failed post-move validation; source restored: {error}"
            )),
            Err(restore_error) => Err(format!(
                "archived evidence failed validation ({error}) and restore ({restore_error})"
            )),
        };
    }
    // Git cannot represent an empty directory, so one committed into an archive
    // package survives a checkout of any commit that predates the package while
    // every tracked sibling disappears. What is left is a husk: a dated
    // directory with no `state.json`, invisible to `git status`. Prune after
    // validation so the rollback paths above still restore an intact source.
    prune_empty_package_directories(&destination);
    Ok(destination)
}

/// Removes directories that hold no regular file at any depth, deepest first.
///
/// An archived package is immutable history; a directory that git could not
/// commit carries no information and is exactly what a later checkout strands.
/// Best effort by design — failing to remove an empty directory must not undo
/// an archive that already validated.
fn prune_empty_package_directories(package: &Path) {
    let mut directories = Vec::new();
    collect_package_directories(package, &mut directories);
    // Deepest first, so a parent emptied by its children is removed in the same pass.
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        let _ = fs::remove_dir(&directory);
    }
}

fn collect_package_directories(directory: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            collect_package_directories(&path, found);
            found.push(path);
        }
    }
}

pub fn summarize_change(root: &Path, record: &ChangeRecord) -> ChangeSummary {
    summarize_change_with_strict(root, record, false)
}

pub fn summarize_change_with_strict(
    root: &Path,
    record: &ChangeRecord,
    explicit_strict: bool,
) -> ChangeSummary {
    let effective = effective_change_definition(root, record).ok();
    summarize_change_with_effective(root, record, explicit_strict, effective.as_ref())
}

fn summarize_change_with_effective(
    root: &Path,
    record: &ChangeRecord,
    explicit_strict: bool,
    effective: Option<&EffectiveChangeDefinition>,
) -> ChangeSummary {
    let correction_valid = effective.is_some();
    let corrected_fields = effective
        .map(|definition| {
            definition
                .answers
                .iter()
                .filter(|(field, value)| record.answers.get(*field) != Some(*value))
                .map(|(field, value)| (field.clone(), value.clone()))
                .collect()
        })
        .unwrap_or_default();
    let approval_valid = ensure_definition_approval_valid(root, record).is_ok();
    let current_definition_digest = definition_digest(root, record).ok();
    let scope_expansion = if record.workflow_version >= 2 {
        load_approvals(root, record)
            .ok()
            .and_then(|ledger| {
                let (index, approval) = ledger
                    .approvals
                    .iter()
                    .enumerate()
                    .rfind(|(_, approval)| approval.gate == "definition")?;
                approval.approved_scope.clone().or_else(|| {
                    ledger
                        .scope_adoptions
                        .iter()
                        .find(|adoption| adoption.source_approval_index == index as u64)
                        .map(|adoption| adoption.adopted_scope.clone())
                })
            })
            .and_then(|approved| {
                approved_scope(root, record)
                    .ok()
                    .map(|current| scope_expansion(&approved, &current))
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let artifacts_complete = validate_artifacts(root, record).is_ok();
    let incomplete_artifacts = effective
        .map(|definition| {
            definition
                .selected_artifacts
                .iter()
                .filter_map(|artifact| {
                    let relative = format!("{CHANGES_PATH}/{}/{}", record.id, artifact.file_name());
                    let path = root.join(&relative);
                    match fs::read_to_string(path) {
                        Ok(content) if !artifact_content_is_incomplete(&content) => None,
                        _ => Some(relative),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let verification_current = || {
        load_verification(root, record)
            .is_ok_and(|verification| verification_is_current(root, record, &verification))
    };
    let scoped_review_current = load_scoped_review(root, record)
        .is_ok_and(|review| scoped_review_is_current(root, record, &review));
    let routing = load_verification_routing(root).unwrap_or_default();
    let strict_validation_required =
        explicit_strict || change_requires_strict_validation(record, &routing);
    let verification_commands = load_policy_checked(root)
        .ok()
        .flatten()
        .and_then(|policy| {
            verification_commands_for_change(root, &policy, record, explicit_strict).ok()
        })
        .unwrap_or_default();
    let terminal_evidence = matches!(record.state, ChangeState::Accepted | ChangeState::Archived)
        .then(|| terminal_evidence_summary(root, record));
    // Sequence-ledger freezes must outrank state-local next actions (including reopen /
    // finalize). Premature multi-id acknowledgements block change new/adopt project-wide.
    let next_action = if let Some(remediation) = sequence_ledger_freeze_next_action(root) {
        remediation
    } else {
        match record.state {
            ChangeState::Draft if !next_questions(record).is_empty() => {
                let question = next_questions(record)
                    .into_iter()
                    .next()
                    .map(|question| question.id)
                    .unwrap_or_else(|| "<question>".into());
                format!(
                    "run `specsync change answer {} {} <answer>`",
                    record.id, question
                )
            }
            // Prefer completing selected artifacts over approve when interview is done
            // but TODO/empty stubs remain (sandbox #16).
            ChangeState::Draft if !artifacts_complete => {
                if incomplete_artifacts.is_empty() {
                    format!(
                        "complete selected artifacts, then run `specsync change status {}`",
                        record.id
                    )
                } else {
                    format!(
                        "complete {}, then run `specsync change status {}`",
                        incomplete_artifacts.join(", "),
                        record.id
                    )
                }
            }
            ChangeState::Draft => {
                format!("run `specsync change approve {} --actor <name>`", record.id)
            }
            ChangeState::Approved if !approval_valid => {
                format!("run `specsync change approve {} --actor <name>`", record.id)
            }
            ChangeState::Approved => {
                format!("run `specsync change check {}`", record.id)
            }
            ChangeState::Implementing if !artifacts_complete => {
                format!(
                    "complete {}, then run `specsync change status {}`",
                    incomplete_artifacts.join(", "),
                    record.id
                )
            }
            ChangeState::Implementing if !approval_valid => {
                format!("run `specsync change approve {} --actor <name>`", record.id)
            }
            ChangeState::Implementing => {
                format!("run `specsync change check {}`", record.id)
            }
            ChangeState::Verifying if !artifacts_complete => {
                format!(
                    "complete {}, then run `specsync change status {}`",
                    incomplete_artifacts.join(", "),
                    record.id
                )
            }
            ChangeState::Verifying if !approval_valid => {
                format!("run `specsync change approve {} --actor <name>`", record.id)
            }
            ChangeState::Verifying if !verification_current() => {
                format!("run `specsync change check {}`", record.id)
            }
            ChangeState::Verifying if !scoped_review_current => {
                format!(
                    "run `specsync change review {} --reviewer <independent-reviewer>` after the PR's scoped review passes",
                    record.id
                )
            }
            ChangeState::Verifying => {
                format!("run `specsync change finalize {}`", record.id)
            }
            ChangeState::Accepted if !correction_valid => format!(
                "restore `.specsync/changes/{}/corrections.json` from trusted history, then run `specsync change status {}`",
                record.id, record.id
            ),
            ChangeState::Accepted if record.workflow_version >= 2 => {
                format!("run `specsync change finalize {}`", record.id)
            }
            ChangeState::Accepted if ensure_closing_approval_valid(root, record).is_err() => {
                format!(
                    "run `specsync change reopen {} --actor <name> --reason <reason>`",
                    record.id
                )
            }
            ChangeState::Accepted
                if terminal_evidence.as_ref().is_some_and(|evidence| {
                    evidence.validity == TerminalEvidenceValidity::Stale
                }) =>
            {
                format!(
                    "run `specsync change reopen {} --actor <name> --reason <reason>`",
                    record.id
                )
            }
            ChangeState::Accepted => {
                format!("run `specsync change archive {}`", record.id)
            }
            ChangeState::Archived
                if terminal_evidence.as_ref().is_some_and(|evidence| {
                    evidence.validity == TerminalEvidenceValidity::CorruptHistory
                }) =>
            {
                format!(
                    "restore the archive for {} from trusted Git history, then run `specsync change check`",
                    record.id
                )
            }
            ChangeState::Archived => "merge the PR on GitHub if it is still open".into(),
        }
    };
    ChangeSummary {
        id: record.id.clone(),
        title: record.title.clone(),
        state: record.state,
        approval_valid,
        definition_digest: current_definition_digest,
        scope_expansion,
        artifacts_complete,
        correction_valid,
        correction_count: record.correction_count as usize,
        corrected_fields,
        scoped_review_current,
        strict_validation_required,
        verification_commands,
        next_action,
        terminal_evidence,
    }
}

fn policy_at_comparison_base(root: &Path) -> Result<Option<SddPolicy>, String> {
    // Fails closed with the rest of the roster readers. An empty list here does not
    // mean "no changes": it also meant "the roster could not be read", and that
    // silently selected a different pull-request diff base below.
    let records = list_changes_checked(root)?;
    let policy_changed_from_head = !is_ci_project(root)
        && Command::new("git")
            .args(["diff", "--quiet", "HEAD", "--", POLICY_PATH])
            .current_dir(root)
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| !status.success());
    let comparison = if policy_changed_from_head {
        "HEAD".to_string()
    } else {
        pull_request_diff_base(root, &records)
    };
    let reference = comparison
        .split("...")
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "unable to determine trusted SDD policy base".to_string())?;
    if git_output(root, &["rev-parse", "--verify", reference]).is_none() {
        if is_ci_project(root)
            && std::env::var("GITHUB_BASE_REF").is_ok_and(|value| !value.trim().is_empty())
        {
            return Err(format!(
                "unable to inspect trusted SDD policy base `{reference}`; check out full base history"
            ));
        }
        return Ok(None);
    }
    let policy_tree_path = git_repo_relative_path(root, POLICY_PATH)?;
    let object = format!("{reference}:{policy_tree_path}");
    let Some(content) = git_output_allow_empty(root, &["show", &object]) else {
        return Ok(None);
    };
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|error| format!("invalid trusted SDD policy `{object}`: {error}"))
}

/// Full lifecycle integrity including archived terminal evidence.
///
/// Product defaults (`change check`, `change audit`, `specsync check`) use
/// [`audit_project`] instead. This remains for unit/integration tests and any
/// intentional full historical walk.
#[cfg_attr(not(test), allow(dead_code))]
pub fn check_project(root: &Path) -> SddCheckReport {
    let _scope = ensure_change_read_scope(root);
    // Full integrity including archive terminal evidence — tests / rare callers.
    check_project_with_command_output(root, true)
}

/// Audit active change workspaces and living SDD policy/spec coherence.
///
/// Does **not** re-validate archived terminal evidence. Archives are history;
/// living truth is active workspaces plus specs/policy.
pub fn audit_project(root: &Path) -> SddCheckReport {
    let _scope = ensure_change_read_scope(root);
    check_project_with_command_output(root, false)
}

fn check_project_with_command_output(
    root: &Path,
    include_archive_integrity: bool,
) -> SddCheckReport {
    if let Some(error) = crate::verification_recursion_error() {
        return SddCheckReport {
            enabled: true,
            errors: vec![error],
            ..SddCheckReport::default()
        };
    }
    let current_policy = match load_policy_checked(root) {
        Ok(policy) => policy,
        Err(error) => {
            return SddCheckReport {
                enabled: true,
                errors: vec![error],
                ..SddCheckReport::default()
            };
        }
    };
    let base_policy = match policy_at_comparison_base(root) {
        Ok(policy) => policy,
        Err(error) => {
            return SddCheckReport {
                enabled: true,
                errors: vec![error],
                ..SddCheckReport::default()
            };
        }
    };
    let is_versioned_sdd = current_policy.is_some()
        || base_policy.is_some()
        || root.join(".specsync/version").exists()
        || root.join(WORKFLOW_V2_BASELINE_PATH).exists()
        || root.join(CHANGES_PATH).is_dir()
        || root.join(ARCHIVE_PATH).is_dir()
        || git_repo_relative_path(root, POLICY_PATH)
            .ok()
            .and_then(|path| git_output(root, &["ls-tree", "--name-only", "HEAD", path.as_str()]))
            .is_some();
    if !is_versioned_sdd {
        return SddCheckReport::default();
    }
    let records = match list_changes_checked(root) {
        Ok(records) => records,
        Err(error) => {
            return SddCheckReport {
                enabled: true,
                errors: vec![error],
                ..SddCheckReport::default()
            };
        }
    };
    let all_records = if include_archive_integrity {
        match list_all_changes_checked(root) {
            Ok(records) => records,
            Err(error) => {
                return SddCheckReport {
                    enabled: true,
                    errors: vec![error],
                    ..SddCheckReport::default()
                };
            }
        }
    } else {
        // Active-only audit: skip loading archives entirely.
        records
            .iter()
            .cloned()
            .map(|record| (record.id.clone(), record))
            .collect()
    };
    let (terminal_evidence, mut archived_integrity_cache) = if include_archive_integrity {
        terminal_evidence_results_with_records(root, &all_records)
    } else {
        // Only evaluate terminal evidence for active accepted/archived-in-flight records.
        let active_terminal: BTreeMap<_, _> = records
            .iter()
            .filter(|record| matches!(record.state, ChangeState::Accepted | ChangeState::Archived))
            .cloned()
            .map(|record| (record.id.clone(), record))
            .collect();
        if active_terminal.is_empty() {
            (Vec::new(), Default::default())
        } else {
            terminal_evidence_results_with_records(root, &active_terminal)
        }
    };
    let mut report = SddCheckReport {
        enabled: true,
        // User-facing counts reflect the scope actually audited.
        checked_changes: if include_archive_integrity {
            all_records.len()
        } else {
            records.len()
        },
        terminal_evidence,
        ..SddCheckReport::default()
    };
    let policy = if let Some(base) = base_policy.filter(|policy| policy.enabled)
        && current_policy.as_ref() != Some(&base)
    {
        if !records
            .iter()
            .any(|record| record_covers_path(record, POLICY_PATH))
        {
            report.errors.push(
                "committed SDD policy changed without an implementing, verifying, or accepted change covering `.specsync/sdd.json`"
                    .into(),
            );
        }
        base
    } else {
        let Some(policy) = current_policy else {
            // Active workspaces exist without a policy file: keep list-load results
            // (corrupt/ineligible changes already failed above) and skip policy-gated checks.
            return report;
        };
        if !policy.enabled {
            // Explicitly disabled policy: do not treat SDD as enabled.
            return SddCheckReport::default();
        }
        policy
    };
    if let Err(error) = validate_change_sequences(root) {
        return SddCheckReport {
            enabled: true,
            errors: vec![error],
            ..SddCheckReport::default()
        };
    }
    for record in &records {
        if let Err(error) = validate_definition(root, record) {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if matches!(
            record.state,
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
        ) && let Err(error) = ensure_definition_approval_valid(root, record)
        {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if matches!(
            record.state,
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
        ) && let Err(error) = validate_delta_files(root, record)
        {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if let Err(error) = ensure_no_delta_conflicts(root, record) {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if matches!(
            record.state,
            ChangeState::Implementing | ChangeState::Verifying
        ) && let Err(error) = ensure_dependencies_satisfied(root, record)
        {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if record.state == ChangeState::Accepted
            && let Some(evidence) = report
                .terminal_evidence
                .iter()
                .find(|evidence| evidence.id == record.id)
            && evidence.evidence.validity == TerminalEvidenceValidity::Stale
        {
            report.errors.push(format!(
                "{}: accepted change verification is stale for current delivery inputs: {}",
                record.id,
                evidence
                    .evidence
                    .reason
                    .as_deref()
                    .unwrap_or("unknown reason")
            ));
        }
        if record.canonical_applied
            && matches!(
                record.state,
                ChangeState::Implementing | ChangeState::Verifying
            )
            && let Err(error) = ensure_reopened_definition_unchanged(root, record)
        {
            report.errors.push(format!("{}: {error}", record.id));
        }
        if let Err(error) = ensure_no_sequence_collision(root, record) {
            report.errors.push(error);
        }
        for dependency in &record.dependencies {
            if dependency_reaches(root, dependency, &record.id, &mut BTreeSet::new()) {
                report.errors.push(format!(
                    "{}: change dependency cycle through `{dependency}`",
                    record.id
                ));
            }
        }
    }
    if include_archive_integrity {
        let mut shared_legacy_error_reported = false;
        for record in all_records
            .values()
            .filter(|record| record.state == ChangeState::Archived)
        {
            if let Err(error) =
                validate_archived_integrity_with_cache(root, record, &mut archived_integrity_cache)
            {
                let is_shared_legacy_error = archived_integrity_cache
                    .legacy
                    .as_ref()
                    .and_then(|result| result.as_ref().err())
                    == Some(&error);
                if is_shared_legacy_error {
                    if !shared_legacy_error_reported {
                        report.errors.push(format!(
                            "legacy archive baseline historical integrity is invalid: {error}"
                        ));
                        shared_legacy_error_reported = true;
                    }
                } else {
                    report.errors.push(format!(
                        "{}: archived change historical integrity is invalid: {error}",
                        record.id
                    ));
                }
            }
        }
    }
    let effective = validate_effective_contracts(root, &records);
    report.warnings.extend(effective.suppressions);
    report.errors.extend(effective.errors);
    let verifying_project_digest = if records
        .iter()
        .any(|record| record.state == ChangeState::Verifying)
    {
        Some(project_input_digest(root))
    } else {
        None
    };
    let mut project_digest_error_reported = false;
    for record in &records {
        if matches!(
            record.state,
            ChangeState::Implementing | ChangeState::Verifying
        ) {
            match collect_requirement_ids(root, record) {
                Ok(ids) => {
                    let missing = requirement_evidence_missing(root, record, &ids);
                    if !missing.is_empty() {
                        report.errors.push(format!(
                            "{}: requirement evidence missing for {}",
                            record.id,
                            missing.join(", ")
                        ));
                    }
                }
                Err(error) => report.errors.push(format!("{}: {error}", record.id)),
            }
        }
        if record.state == ChangeState::Verifying {
            match load_verification(root, record) {
                Ok(evidence) => {
                    if !evidence.passed {
                        report.errors.push(format!(
                            "{}: latest verification evidence failed",
                            record.id
                        ));
                    } else if let Some(result) = &verifying_project_digest {
                        match result {
                            Ok(project_digest) => {
                                if verification_is_current_checked_with_project_digest(
                                    root,
                                    record,
                                    &evidence,
                                    project_digest,
                                )
                                .is_err()
                                {
                                    report.errors.push(format!(
                                        "{}: verification evidence is stale for the current commit or contract",
                                        record.id
                                    ));
                                }
                            }
                            Err(error) if !project_digest_error_reported => {
                                report.errors.push(format!(
                                    "failed to capture shared verification project inputs: {error}"
                                ));
                                project_digest_error_reported = true;
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(error) => report.errors.push(format!("{}: {error}", record.id)),
            }
        }
    }
    if is_ci_project(root)
        && records.iter().any(|record| {
            matches!(
                record.state,
                ChangeState::Implementing | ChangeState::Verifying | ChangeState::Accepted
            )
        })
    {
        let mut configured_commands = Vec::new();
        for record in records.iter().filter(|record| {
            matches!(
                record.state,
                ChangeState::Implementing | ChangeState::Verifying | ChangeState::Accepted
            )
        }) {
            match verification_commands_for_change(root, &policy, record, false) {
                Ok(commands) => configured_commands.extend(commands),
                Err(error) => report.errors.push(format!("{}: {error}", record.id)),
            }
        }
        let mut seen = BTreeSet::new();
        configured_commands.retain(|command| seen.insert(command.clone()));
        for configured in &configured_commands {
            match run_configured_command(root, configured) {
                Ok(status) if status.success() => {}
                Ok(status) => report.errors.push(format!(
                    "CI verification command `{configured}` failed with exit code {:?}",
                    status.code()
                )),
                Err(error) => report.errors.push(error),
            }
        }
    }
    if policy.require_change_for_meaningful_files {
        match uncovered_meaningful_paths(root, &policy, &records) {
            Ok(paths) => {
                if !paths.is_empty() {
                    report.errors.push(uncovered_paths_error(&policy, &paths));
                }
            }
            Err(error) => report.errors.push(error),
        }
    }
    report
}

pub fn adopt(root: &Path, dry_run: bool, source: Option<&str>) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();
    actions.push(format!("enable SDD policy at {POLICY_PATH}"));
    actions.push(
        "adopt the workflow-v2 baseline for new changes while preserving workflow-v1 evidence"
            .into(),
    );
    let requirement_proposals = requirement_id_proposals(root);
    let detected = source
        .map(str::to_string)
        .or_else(|| detect_foreign_source(root));
    if let Some(source) = detected.as_deref() {
        actions.push(format!(
            "import active and canonical artifacts from {source}"
        ));
    }
    actions.push(format!(
        "propose stable requirement IDs for {} canonical companion(s)",
        requirement_proposals.len()
    ));
    actions.push("preserve existing companion files without making them mandatory".into());
    if dry_run {
        return Ok(actions);
    }
    let _lock = acquire_project_lock(root)?;
    validate_change_sequences(root)?;
    if let Some(source) = detected.as_deref() {
        validate_foreign_import(root, source)?;
    }
    let baseline_candidate = prepare_workflow_v2_adoption_candidate(root)?;
    let policy_existed = root.join(POLICY_PATH).exists();
    let existing_bootstrap = fs::read_to_string(root.join(".specsync/adoption-report.json"))
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|value| value.get("bootstrap_policy").cloned());
    let policy_content = if policy_existed {
        None
    } else {
        Some(json_content(&default_policy(
            root,
            detect_verification_commands(root),
        ))?)
    };
    let bootstrap_policy = if policy_existed {
        existing_bootstrap
    } else {
        policy_content
            .as_deref()
            .map(|content| adoption_bootstrap_record_for_content(root, content.as_bytes()))
            .transpose()?
            .flatten()
    };
    let baseline_content = baseline_candidate
        .as_ref()
        .map(|candidate| json_content(&candidate.baseline))
        .transpose()?;
    let mut prepared = Vec::new();
    if let Some(policy_content) = policy_content {
        prepared.push((root.join(POLICY_PATH), policy_content));
    }
    if let Some(source) = detected.as_deref() {
        prepared.extend(prepare_foreign_import(root, source)?);
    }
    prepared.push((
        root.join(".specsync/adoption-report.json"),
        json_content(&serde_json::json!({
            "requirements_needing_ids": requirement_proposals,
            "generated_at": now(),
            "bootstrap_policy": bootstrap_policy,
        }))?,
    ));
    if let Some(baseline_content) = baseline_content {
        prepared.push((root.join(WORKFLOW_V2_BASELINE_PATH), baseline_content));
    }
    if let Some(candidate) = baseline_candidate {
        write_prepared_files_checked(root, &prepared, || {
            validate_workflow_v2_adoption_git_snapshot(root, &candidate.git_snapshot)
        })?;
    } else {
        write_prepared_files(root, &prepared)?;
    }
    Ok(actions)
}

fn adoption_bootstrap_record_for_content(
    root: &Path,
    content: &[u8],
) -> Result<Option<serde_json::Value>, String> {
    let Some(base_commit) = git_output(root, &["rev-parse", "--verify", "HEAD"]) else {
        return Ok(None);
    };
    Ok(Some(serde_json::json!({
        "path": POLICY_PATH,
        "digest": bootstrap_digest(POLICY_PATH, content),
        "base_commit": base_commit,
    })))
}

/// Record the protected SDD files this bootstrap just created.
///
/// `specsync init` writes `.specsync/config.toml`, `.specsync/version`, and
/// `.specsync/sdd.json`. All three are protected SDD paths, so the first commit
/// after initialization used to land as uncovered meaningful delivery — a gate
/// no change workspace could have satisfied, because none existed when the
/// files were written. Recording them here lets the coverage gate recognize
/// spec-sync's own output without weakening the guard on later edits.
pub fn record_bootstrap_paths(root: &Path) -> Result<(), String> {
    let Some(base_commit) = git_output(root, &["rev-parse", "--verify", "HEAD"]) else {
        // Without Git evidence the coverage gate is disabled anyway
        // (see `default_policy`), so there is nothing to exempt.
        return Ok(());
    };
    let mut paths = Vec::new();
    for candidate in BOOTSTRAP_RECORD_CANDIDATES {
        let Ok(content) = fs::read(root.join(candidate)) else {
            continue;
        };
        paths.push(serde_json::json!({
            "path": candidate,
            "digest": bootstrap_digest(candidate, &content),
        }));
    }
    if paths.is_empty() {
        return Ok(());
    }
    write_json(
        &root.join(BOOTSTRAP_RECORD_PATH),
        &serde_json::json!({
            "version": 1,
            "created_at": now(),
            "base_commit": base_commit,
            "paths": paths,
        }),
    )
}

/// Protected SDD paths a bootstrap record exempts from lifecycle path coverage.
///
/// The exemption is deliberately hard to forge. A recorded path is honored only
/// when all four hold:
///
/// 1. it is a protected SDD path — a record can never exempt product source;
/// 2. it is absent at the delivery comparison base, so a *modification* of an
///    already-tracked policy file is never exempt, only its creation;
/// 3. the recorded base commit is a real ancestor of `HEAD`; and
/// 4. the file still hashes to the digest recorded when spec-sync wrote it
///    (see [`bootstrap_digest`] for what the policy digest covers).
///
/// Editing a bootstrapped file therefore revokes its own exemption and the
/// normal change workflow applies from that point on.
fn bootstrap_exempt_paths(root: &Path, comparison_base: &str) -> BTreeSet<String> {
    let mut exempt = BTreeSet::new();
    let records = bootstrap_records(root);
    if records.is_empty() {
        return exempt;
    }
    let Some(base_commit) = comparison_base_commit(root, comparison_base) else {
        return exempt;
    };
    for record in records {
        if !is_protected_sdd_path(&record.path) {
            continue;
        }
        if !commit_is_ancestor_of_head(root, &record.base_commit) {
            continue;
        }
        if !project_path_is_absent_at(root, &base_commit, &record.path) {
            continue;
        }
        if !project_path_matches_digest(root, &record.path, &record.digest) {
            continue;
        }
        exempt.insert(record.path);
    }
    exempt
}

struct BootstrapRecord {
    path: String,
    digest: String,
    base_commit: String,
}

/// Bootstrap records written by `specsync init` and by `change adopt`.
///
/// Adoption keeps its original single-path `bootstrap_policy` shape so reports
/// written by earlier versions keep covering the policy they created.
fn bootstrap_records(root: &Path) -> Vec<BootstrapRecord> {
    let mut records = Vec::new();
    if let Some(report) = read_json_value(root, ".specsync/adoption-report.json")
        && let Some(bootstrap) = report.get("bootstrap_policy")
        && let Some(record) = bootstrap_record_entry(bootstrap, bootstrap)
    {
        records.push(record);
    }
    if let Some(report) = read_json_value(root, BOOTSTRAP_RECORD_PATH)
        && let Some(entries) = report.get("paths").and_then(serde_json::Value::as_array)
    {
        records.extend(
            entries
                .iter()
                .filter_map(|entry| bootstrap_record_entry(entry, &report)),
        );
    }
    records
}

fn bootstrap_record_entry(
    entry: &serde_json::Value,
    base_source: &serde_json::Value,
) -> Option<BootstrapRecord> {
    Some(BootstrapRecord {
        path: entry.get("path")?.as_str()?.to_string(),
        digest: entry.get("digest")?.as_str()?.to_string(),
        base_commit: base_source.get("base_commit")?.as_str()?.to_string(),
    })
}

fn read_json_value(root: &Path, relative: &str) -> Option<serde_json::Value> {
    let content = fs::read_to_string(root.join(relative)).ok()?;
    serde_json::from_str(&content).ok()
}

/// Resolve the single commit a delivery is compared against.
///
/// [`pull_request_diff_base`] yields either a `<ref>...HEAD` range or a bare
/// commit; both reduce to the merge base with `HEAD`.
fn comparison_base_commit(root: &Path, comparison_base: &str) -> Option<String> {
    let left = comparison_base
        .split("...")
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(comparison_base);
    git_output(root, &["merge-base", left, "HEAD"]).or_else(|| {
        git_output(
            root,
            &["rev-parse", "--verify", &format!("{left}^{{commit}}")],
        )
    })
}

fn commit_is_ancestor_of_head(root: &Path, commit: &str) -> bool {
    if git_output(root, &["rev-parse", "--verify", commit]).is_none() {
        return false;
    }
    Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn project_path_is_absent_at(root: &Path, commit: &str, path: &str) -> bool {
    let Ok(tree_path) = git_repo_relative_path(root, path) else {
        return false;
    };
    git_output_allow_empty(root, &["show", &format!("{commit}:{tree_path}")]).is_none()
}

fn project_path_matches_digest(root: &Path, path: &str, digest: &str) -> bool {
    let Ok(content) = fs::read(root.join(path)) else {
        return false;
    };
    if bootstrap_digest(path, &content) == digest || content_digest(&content) == digest {
        return true;
    }
    // `.specsync/config.toml` and `.specsync/version` are not covered by the
    // JSON `eol=lf` attribute, so a Windows autocrlf checkout can rewrite the
    // bytes without changing the content. Accept that rewrite rather than
    // revoking the bootstrap on a platform that never edited the file.
    let lf_only: Vec<u8> = content
        .iter()
        .copied()
        .filter(|byte| *byte != b'\r')
        .collect();
    lf_only != content && content_digest(&lf_only) == digest
}

/// Digest recorded for one bootstrapped path.
///
/// The policy is pinned by its *enforcement surface*, not its bytes:
/// `verification_commands` is cleared before hashing. `init` writes an empty
/// list whenever it cannot detect a test command and tells the author to fill
/// it in — pinning that field would revoke the bootstrap for doing exactly what
/// the tool just asked for. Everything that decides whether the gate bites —
/// `enabled`, `require_change_for_meaningful_files`, `meaningful_paths`,
/// `ignored_paths`, custom artifacts, principles — stays pinned, and a policy
/// that does not parse falls back to a byte digest.
fn bootstrap_digest(path: &str, content: &[u8]) -> String {
    if path == POLICY_PATH
        && let Some(projection) = policy_enforcement_projection(content)
    {
        return content_digest(&projection);
    }
    content_digest(content)
}

fn policy_enforcement_projection(content: &[u8]) -> Option<Vec<u8>> {
    let mut policy: SddPolicy = serde_json::from_slice(content).ok()?;
    policy.verification_commands.clear();
    serde_json::to_vec(&policy).ok()
}

fn content_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn requirement_id_proposals(root: &Path) -> Vec<serde_json::Value> {
    let mut proposals = Vec::new();
    let specs = root.join(crate::config::load_config(root).specs_dir);
    for entry in walkdir::WalkDir::new(specs)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.file_name().and_then(|name| name.to_str()) != Some("requirements.md")
        {
            continue;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.contains("### REQ-") {
            continue;
        }
        let module = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("module");
        proposals.push(serde_json::json!({
            "path": path.strip_prefix(root).unwrap_or(path),
            "suggested_first_id": format!("REQ-{}-001", slugify(module)),
            "action": "review and assign one stable ID per durable requirement",
        }));
    }
    proposals
}

pub fn detect_verification_commands(root: &Path) -> Vec<String> {
    if root.join("Cargo.toml").exists() {
        return vec!["cargo test".into()];
    }
    if root.join("bun.lock").exists() || root.join("bun.lockb").exists() {
        return vec!["bun test".into()];
    }
    if root.join("Package.swift").exists() {
        return vec!["swift test".into()];
    }
    if root.join("fledge.toml").exists() {
        return vec!["fledge run test".into()];
    }
    if root.join("go.mod").exists() {
        return vec!["go test ./...".into()];
    }
    if root.join("pyproject.toml").exists() || root.join("pytest.ini").exists() {
        return vec!["pytest".into()];
    }
    if root.join("package.json").exists() {
        return vec!["npm test".into()];
    }
    Vec::new()
}

fn adaptive_artifacts(kind: ChangeKind, specs: &[String], paths: &[String]) -> Vec<ArtifactKind> {
    let mut artifacts = vec![ArtifactKind::Context];
    match kind {
        ChangeKind::Feature => artifacts.extend([
            ArtifactKind::Requirements,
            ArtifactKind::Plan,
            ArtifactKind::Tasks,
            ArtifactKind::Testing,
            ArtifactKind::Docs,
        ]),
        ChangeKind::BugFix => artifacts.extend([ArtifactKind::Testing, ArtifactKind::Tasks]),
        ChangeKind::Refactor => artifacts.extend([ArtifactKind::Plan, ArtifactKind::Testing]),
        ChangeKind::Migration => artifacts.extend([
            ArtifactKind::Research,
            ArtifactKind::Design,
            ArtifactKind::Plan,
            ArtifactKind::Tasks,
            ArtifactKind::Testing,
            ArtifactKind::Docs,
        ]),
        ChangeKind::Documentation => artifacts.push(ArtifactKind::Docs),
        ChangeKind::Operations => artifacts.extend([ArtifactKind::Plan, ArtifactKind::Testing]),
    }
    if specs.len() > 1 || paths.len() > 4 {
        if !artifacts.contains(&ArtifactKind::Design) {
            artifacts.push(ArtifactKind::Design);
        }
        if !artifacts.contains(&ArtifactKind::Tasks) {
            artifacts.push(ArtifactKind::Tasks);
        }
    }
    artifacts
}

fn add_artifact(record: &mut ChangeRecord, artifact: ArtifactKind) {
    if !record.selected_artifacts.contains(&artifact) {
        record.selected_artifacts.push(artifact);
    }
}

fn validate_definition(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let effective = effective_change_definition(root, record)?;
    validate_acceptance_owner_correction_records(record)?;
    if !record.acceptance_owner_corrections.is_empty()
        && matches!(
            record.state,
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
        )
    {
        validate_acceptance_owner_corrections_current(root, record)?;
    }
    if record.description.trim().is_empty() {
        return Err("description is empty".into());
    }
    if record.acceptance_criteria.is_empty() {
        return Err("at least one acceptance criterion is required".into());
    }
    if record.affected_paths.is_empty() {
        return Err("at least one affected path is required".into());
    }
    if record.affected_specs.is_empty() && !record.no_spec_change {
        return Err("affected specs are required unless no_spec_change is justified".into());
    }
    for module in &record.affected_specs {
        crate::commands::validate_module_name(module)
            .map_err(|error| format!("invalid affected spec: {error}"))?;
    }
    validate_supersedes_edges(record)?;
    validate_supersedes_semantics(root, record)?;
    if record.no_spec_change
        && record
            .no_spec_change_rationale
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err("no_spec_change requires a rationale".into());
    }
    if record.no_spec_change
        && !is_legacy_self_adoption_record(record)
        && effective
            .answers
            .get("public_contract")
            .is_some_and(|answer| is_yes(answer))
    {
        return Err(
            "no_spec_change cannot be used when the interview declares a public contract change"
                .into(),
        );
    }
    if !next_questions(record).is_empty() {
        return Err("the deterministic interview is incomplete".into());
    }
    if let Some(policy) = load_policy_checked(root)?
        && let Some(principles) = policy.principles_file
    {
        let path = safe_project_path(root, &principles)?;
        let content = fs::read_to_string(&path)
            .map_err(|_| format!("configured principles file is missing: {}", path.display()))?;
        if content.trim().is_empty() {
            return Err(format!(
                "configured principles file is empty: {}",
                path.display()
            ));
        }
    }
    validate_artifacts(root, record)
}

fn validate_supersedes_edges(record: &ChangeRecord) -> Result<(), String> {
    const MAX_SUCCESSION_TUPLES: usize = 100_000;
    let mut count = 0_usize;
    let mut previous_edge: Option<&str> = None;
    for edge in &record.supersedes {
        if previous_edge.is_some_and(|previous| previous >= edge.predecessor_id.as_str()) {
            return Err(
                "supersedes edges must be strictly sorted by predecessor ID and must not repeat"
                    .into(),
            );
        }
        previous_edge = Some(edge.predecessor_id.as_str());
        if edge.predecessor_id.len() > 256 || edge.obligations.is_empty() {
            return Err(format!("invalid supersedes edge `{}`", edge.predecessor_id));
        }
        let mut previous_obligation: Option<(&str, &str)> = None;
        for obligation in &edge.obligations {
            count += 1;
            if count > MAX_SUCCESSION_TUPLES {
                return Err(format!(
                    "supersedes obligations exceed {MAX_SUCCESSION_TUPLES}"
                ));
            }
            if obligation.path.len() > 4096 || obligation.module.len() > 256 {
                return Err("supersedes obligation field exceeds its size limit".into());
            }
            let normalized = normalize_project_path(&obligation.path)
                .map_err(|error| format!("invalid succession path: {error}"))?;
            if normalized != obligation.path {
                return Err(format!(
                    "succession path is not canonical: {}",
                    obligation.path
                ));
            }
            crate::commands::validate_module_name(&obligation.module)
                .map_err(|error| format!("invalid succession module: {error}"))?;
            validate_sha256_digest(
                &obligation.predecessor_entry_digest,
                "predecessor entry digest",
            )?;
            if previous_obligation.is_some_and(|previous| {
                previous >= (obligation.path.as_str(), obligation.module.as_str())
            }) {
                return Err(format!(
                    "supersedes obligations for `{}` must be strictly sorted and unique",
                    edge.predecessor_id
                ));
            }
            previous_obligation = Some((&obligation.path, &obligation.module));
        }
    }
    Ok(())
}

fn validate_supersedes_semantics(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    for edge in &record.supersedes {
        let predecessor = load_change(root, &edge.predecessor_id)?;
        if !matches!(
            predecessor.state,
            ChangeState::Accepted | ChangeState::Archived
        ) {
            return Err(format!(
                "superseded change `{}` must be accepted or archived",
                edge.predecessor_id
            ));
        }
        // The predecessor record is loaded immediately above, so comparing when the two were
        // created costs nothing the ID comparison saved.
        //
        // What this guard is actually for is narrower than it looks. `:7511` already requires
        // the predecessor to be accepted or archived, `supersedes_reaches` below is a real
        // cycle check over the edge graph, and a successor declares `supersede` before
        // definition approval — so an honest backwards edge cannot be declared at all, since
        // `load_change` would fail on a predecessor that did not yet exist. What remains is
        // resistance to a hand-edited `supersedes` edge, and `created_at` serves that exactly
        // as well as an ordinal did while meaning the right thing.
        if !happens_after(record, &predecessor) {
            return Err(format!(
                "superseded change `{}` must have been created before successor `{}`",
                edge.predecessor_id, record.id
            ));
        }
        if supersedes_reaches(root, &edge.predecessor_id, &record.id, &mut BTreeSet::new()) {
            return Err(format!(
                "supersedes edge `{}` -> `{}` creates a succession cycle",
                record.id, edge.predecessor_id
            ));
        }
        let manifest = resolved_acceptance_manifest(root, &predecessor)?;
        for obligation in &edge.obligations {
            if !record.affected_specs.contains(&obligation.module) {
                return Err(format!(
                    "successor `{}` does not declare affected module `{}`",
                    record.id, obligation.module
                ));
            }
            if !record
                .affected_paths
                .iter()
                .any(|scope| path_matches_scope(&obligation.path, scope))
            {
                return Err(format!(
                    "successor `{}` does not declare affected path `{}`",
                    record.id, obligation.path
                ));
            }
            let mut entries = manifest
                .entries
                .iter()
                .filter(|entry| entry.path == obligation.path);
            let entry = entries.next().ok_or_else(|| {
                format!(
                    "predecessor `{}` has no signed acceptance entry for `{}`",
                    edge.predecessor_id, obligation.path
                )
            })?;
            if entries.next().is_some() {
                return Err(format!(
                    "predecessor `{}` has ambiguous acceptance entries for `{}`",
                    edge.predecessor_id, obligation.path
                ));
            }
            if entry.entry_digest != obligation.predecessor_entry_digest {
                return Err(format!(
                    "predecessor entry digest mismatch for `{}` `{}`",
                    edge.predecessor_id, obligation.path
                ));
            }
            if obligation.module.starts_with("@exact:")
                || !entry.owners.contains(&obligation.module)
            {
                return Err(format!(
                    "module `{}` is not a successor-eligible signed owner of predecessor path `{}`",
                    obligation.module, obligation.path
                ));
            }
        }
    }
    Ok(())
}

fn supersedes_reaches(
    root: &Path,
    from: &str,
    target: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if from == target {
        return true;
    }
    if !visited.insert(from.to_string()) {
        return false;
    }
    load_change(root, from)
        .map(|record| {
            record.supersedes.iter().any(|edge| {
                edge.predecessor_id == target
                    || supersedes_reaches(root, &edge.predecessor_id, target, visited)
            })
        })
        .unwrap_or(false)
}

fn validate_sha256_digest(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!(
            "{label} must be 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn is_canonical_commit_id(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Whether `later` happened after `earlier`.
///
/// This replaced `succession_change_key`, which derived a total temporal order from the ID
/// string alone — `(change_sequence(id).unwrap_or(u64::MAX), id)`. That worked only because
/// IDs carry a monotonically allocated ordinal, so "supersedes" silently meant "has a bigger
/// number". Under any identity scheme without one it would have degraded to *alphabetical*
/// order with no error, no compile failure and no failing test: `retire-auth` would have
/// "happened before" `add-billing` because `a` sorts before `r`.
///
/// `created_at` says what the ordinal was standing in for. The tiebreak on ID keeps the
/// relation a total order when two changes share a timestamp, which matters because callers
/// use it to enforce strict sorting.
///
/// Both records are already loaded at every call site, so this costs no I/O the ordinal saved.
fn happens_after(later: &ChangeRecord, earlier: &ChangeRecord) -> bool {
    (later.created_at, later.id.as_str()) > (earlier.created_at, earlier.id.as_str())
}

fn is_legacy_self_adoption_record(record: &ChangeRecord) -> bool {
    record.schema_version == 1
        && record.id == "CHG-0001-bootstrap-and-ship-the-verified-specsync-5-0-full-sdd-lifecycle"
        && record.state == ChangeState::Accepted
        && record.no_spec_change_rationale.as_deref()
            == Some(
                "The lifecycle implementation and canonical contracts predate self-adoption in this branch; this bootstrap record verifies the completed work without reapplying already-canonical semantic deltas.",
            )
}

/// Lightweight artifact completeness for human next-action guidance.
///
/// Uses the **persisted** `selected_artifacts` list only — does **not** load
/// correction ledgers or digests. That keeps text-mode `change status` free of
/// `validate_trusted_correction_history` (CodeQL `rust/cleartext-logging`).
/// Definition approve still calls [`validate_artifacts`], which re-checks against
/// the effective (correction-applied) selection.
pub fn artifacts_complete_for_guidance(root: &Path, record: &ChangeRecord) -> bool {
    validate_artifact_bodies(root, &record.id, &record.selected_artifacts).is_ok()
}

fn validate_artifacts(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let effective = effective_change_definition(root, record)?;
    validate_artifact_bodies(root, &record.id, &effective.selected_artifacts)
}

fn validate_artifact_bodies(
    root: &Path,
    id: &str,
    selected: &[ArtifactKind],
) -> Result<(), String> {
    let dir = find_change_dir(root, id)?;
    for artifact in selected {
        let path = dir.join(artifact.file_name());
        let content = read_bounded_change_text(&path, "artifact")?;
        if artifact_content_is_incomplete(&content) {
            return Err(format!("artifact is incomplete: {}", path.display()));
        }
    }
    Ok(())
}

/// True when an artifact body is empty or only placeholder TODO content.
///
/// Treats HTML `<!-- TODO` comments, bare `TODO` lines, and markdown headings
/// that are only `TODO` / `TODO: …` as incomplete (product #495 / sandbox #22).
/// Real prose or checklist items make the artifact complete even if a TODO remains
/// elsewhere — except HTML TODO comments, which always mark incomplete.
fn artifact_content_is_incomplete(content: &str) -> bool {
    if content.contains("<!-- TODO") {
        return true;
    }
    let body = strip_yaml_frontmatter(content);
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return true;
    }
    let mut saw_non_empty = false;
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        saw_non_empty = true;
        if is_placeholder_todo_line(line) {
            continue;
        }
        return false;
    }
    saw_non_empty || trimmed.is_empty()
}

fn strip_yaml_frontmatter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---") else {
        return content;
    };
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .unwrap_or(rest);
    if let Some(end) = rest.find("\n---\n") {
        return &rest[end + "\n---\n".len()..];
    }
    if let Some(end) = rest.find("\n---\r\n") {
        return &rest[end + "\n---\r\n".len()..];
    }
    if let Some(end) = rest.find("\r\n---\r\n") {
        return &rest[end + "\r\n---\r\n".len()..];
    }
    content
}

fn is_placeholder_todo_line(line: &str) -> bool {
    let mut text = line.trim();
    while let Some(stripped) = text.strip_prefix('#') {
        text = stripped.trim_start();
    }
    let lower = text.to_ascii_lowercase();
    lower == "todo"
        || lower.starts_with("todo:")
        || lower.starts_with("todo ")
        || lower == "[ ] todo"
        || lower.starts_with("- [ ] todo")
        || lower.starts_with("* [ ] todo")
        || lower == "- todo"
        || lower == "* todo"
}

fn read_bounded_change_text(path: &Path, kind: &str) -> Result<String, String> {
    let metadata = fs::metadata(path)
        .map_err(|_| format!("required {kind} is missing: {}", path.display()))?;
    if metadata.len() > MAX_CHANGE_ARTIFACT_BYTES {
        return Err(format!(
            "{kind} exceeds {} byte limit: {}",
            MAX_CHANGE_ARTIFACT_BYTES,
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn ensure_tasks_complete(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let effective = effective_change_definition(root, record)?;
    if !effective.selected_artifacts.contains(&ArtifactKind::Tasks) {
        return Ok(());
    }
    let path = change_dir(root, &record.id).join("tasks.md");
    let content = fs::read(&path).map_err(|error| error.to_string())?;
    if markdown_task_checkbox_offsets(&content)
        .into_iter()
        .any(|offset| content[offset] == b' ')
    {
        return Err("tasks.md contains incomplete tasks".into());
    }
    Ok(())
}

fn acceptance_criteria_have_evidence(
    record: &ChangeRecord,
    has_semantic_acceptance_item: bool,
) -> bool {
    if record.no_spec_change {
        return !record.acceptance_criteria.is_empty();
    }
    !record.acceptance_criteria.is_empty() && has_semantic_acceptance_item
}

fn semantic_acceptance_item_exists(root: &Path, record: &ChangeRecord) -> Result<bool, String> {
    if record.no_spec_change {
        return Ok(true);
    }
    for module in &record.affected_specs {
        let content =
            read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")?;
        if parse_delta(&content)?.iter().any(|item| {
            matches!(
                item.target,
                DeltaTarget::Requirement | DeltaTarget::SpecSection
            ) && item.operation != DeltaOperation::Removed
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn requirement_evidence_missing(root: &Path, record: &ChangeRecord, ids: &[String]) -> Vec<String> {
    let testing =
        fs::read_to_string(change_dir(root, &record.id).join("testing.md")).unwrap_or_default();
    let mut evidence = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = portable_project_path(root, path);
        if rel.starts_with(".git/")
            || rel.starts_with(".specsync/")
            || rel.starts_with("specs/")
            || rel.starts_with("target/")
        {
            continue;
        }
        if crate::exports::is_test_file(path, root) {
            evidence.push(path.to_path_buf());
        }
    }
    ids.iter()
        .filter(|id| {
            !testing.contains(id.as_str())
                && !evidence.iter().any(|path| {
                    fs::read_to_string(path)
                        .map(|content| content.contains(id.as_str()))
                        .unwrap_or(false)
                })
        })
        .cloned()
        .collect()
}

/// The section name carried by a stub-section validator warning.
///
/// Deliberately coupled to the exact text `validator::validate_spec` emits for
/// an unfinished section, the same way `SCAFFOLD_BOILERPLATE_PREFIXES` is
/// coupled to the scaffold it detects. Classification is not reused here:
/// `WarningCategory::classify` matches `requirements` first, so the stub warning
/// for `## Requirements` never reaches its stub-section arm.
fn stub_section_warning(warning: &str) -> Option<&str> {
    warning
        .strip_prefix("Section ## ")?
        .strip_suffix(" contains only unfinished draft text")
}

/// Hard errors plus every suppression one effective-contract validation applied.
///
/// Suppressions are carried alongside errors rather than instead of them, so a
/// module that both fails and suppresses still reports what it let through.
#[derive(Debug, Default)]
struct EffectiveContractOutcome {
    errors: Vec<String>,
    suppressions: Vec<String>,
}

impl EffectiveContractOutcome {
    fn failed(error: String) -> Self {
        Self {
            errors: vec![error],
            suppressions: Vec::new(),
        }
    }

    /// Join the errors for gates whose only output channel is a failure string.
    fn error_text(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| self.errors.join("; "))
    }
}

/// Validate every canonical contract with the active semantic deltas replayed.
///
/// Reports the suppressions applied, so no exemption is silent; `check_project`
/// and `audit_project` surface them as warnings.
///
/// Every validator warning is promoted to a hard error here, which is what makes
/// an emptied `## MODIFIED` block fatal — the stub warning is the only gate that
/// sees a section a delta blanked out. Two suppressions are scoped narrowly
/// around that:
///
/// - `.specsyncignore` and inline `<!-- specsync-ignore: … -->` directives, which
///   apply at every other validation surface and now apply here as well.
/// - Stub sections **no active change authored**. `specsync new` writes scaffold
///   placeholders into `## Purpose` and `## Dependencies`, and the first change
///   against a fresh module was blocked by text the tool itself generated. The
///   applied `SpecSection` delta keys name exactly the sections a change is
///   responsible for; anything else is pre-existing canonical text. Authorship
///   is read from the delta file even when the delta is already applied to the
///   canonical spec (`canonical_applied` while verifying, where nothing is
///   replayed), and authorship that cannot be established suppresses nothing.
fn validate_effective_contracts(root: &Path, records: &[ChangeRecord]) -> EffectiveContractOutcome {
    let active: Vec<&ChangeRecord> = records
        .iter()
        .filter(|record| {
            !record.no_spec_change
                && matches!(
                    record.state,
                    ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
                )
                && (!record.canonical_applied || record.state == ChangeState::Verifying)
        })
        .collect();
    if active.is_empty() {
        return EffectiveContractOutcome::default();
    }
    let active = match dependency_ordered_changes(active) {
        Ok(active) => active,
        Err(error) => {
            return EffectiveContractOutcome::failed(format!(
                "effective contract ordering: {error}"
            ));
        }
    };
    let mut modules = BTreeSet::new();
    for record in &active {
        modules.extend(record.affected_specs.iter().cloned());
    }
    let temp = match create_effective_contract_workspace() {
        Ok(temp) => temp,
        Err(error) => return EffectiveContractOutcome::failed(error),
    };
    let config = crate::config::load_config(root);
    let schema_tables = crate::validator::get_schema_table_names(root, &config);
    let schema_columns = crate::commands::build_schema_columns(root, &config);
    let ignore_rules = crate::ignore::IgnoreRules::load(root);
    let mut errors = Vec::new();
    let mut suppressions = Vec::new();
    for module in modules {
        let canonical = match canonical_module_paths(root, &config.specs_dir, &module) {
            Ok((spec_path, _)) => spec_path,
            Err(error) => {
                errors.push(format!(
                    "effective contract `{module}` cannot resolve canonical spec: {error}"
                ));
                continue;
            }
        };
        let mut spec = match fs::read_to_string(&canonical) {
            Ok(spec) => spec,
            Err(error) => {
                errors.push(format!(
                    "effective contract `{module}` cannot read canonical spec: {error}"
                ));
                continue;
            }
        };
        // Sections written by an active change, lowercased. A change owns every
        // section its delta names, whether or not that delta is replayed here.
        let mut authored_sections: BTreeSet<String> = BTreeSet::new();
        // An unreadable delta leaves authorship unknown, which suppresses nothing.
        let mut authorship_known = true;
        for record in &active {
            if !record.affected_specs.contains(&module) {
                continue;
            }
            // An applied delta is already part of the canonical spec, so it is read
            // for authorship only. Its absence is not a new hard error at this gate.
            let applied = record.canonical_applied;
            let delta_path = match delta_path_checked(root, record, &module) {
                Ok(path) => path,
                Err(_) if applied => {
                    authorship_known = false;
                    continue;
                }
                Err(error) => return EffectiveContractOutcome::failed(error),
            };
            let delta = match read_bounded_change_text(&delta_path, "semantic delta") {
                Ok(delta) => delta,
                Err(error) => {
                    if applied {
                        authorship_known = false;
                        continue;
                    }
                    errors.push(format!(
                        "{} effective delta for `{module}`: {error}",
                        record.id
                    ));
                    continue;
                }
            };
            match parse_delta(&delta) {
                Ok(items) => {
                    for item in items
                        .into_iter()
                        .filter(|item| item.target == DeltaTarget::SpecSection)
                    {
                        authored_sections.insert(item.key.to_ascii_lowercase());
                        if applied {
                            continue;
                        }
                        match apply_markdown_block(
                            &spec,
                            "## ",
                            &item.key,
                            &item.content,
                            item.operation,
                        ) {
                            Ok(updated) => spec = updated,
                            Err(error) => {
                                errors.push(format!("{} effective `{module}`: {error}", record.id))
                            }
                        }
                    }
                }
                Err(error) => {
                    if applied {
                        authorship_known = false;
                        continue;
                    }
                    errors.push(format!("{} effective `{module}`: {error}", record.id));
                }
            }
        }
        let effective_dir = temp.join(&module);
        if let Err(error) = fs::create_dir_all(&effective_dir) {
            errors.push(format!("failed to prepare effective contract: {error}"));
            continue;
        }
        let effective = effective_dir.join(format!("{module}.spec.md"));
        let inline_ignores = crate::ignore::IgnoreRules::parse_inline(&spec);
        if let Err(error) = fs::write(&effective, spec) {
            errors.push(format!("failed to write effective contract: {error}"));
            continue;
        }
        // Ignore rules are written against the canonical spec path, not the
        // temporary effective copy this gate validates.
        let spec_rel_path = portable_project_path(root, &canonical);
        let result = crate::validator::validate_spec(
            &effective,
            root,
            &schema_tables,
            &schema_columns,
            &config,
        );
        errors.extend(
            result
                .errors
                .into_iter()
                .map(|error| format!("effective contract `{module}`: {error}")),
        );
        for warning in result.warnings {
            if let Some((category, source)) =
                ignore_rules.suppression_source(&warning, &spec_rel_path, &inline_ignores)
            {
                suppressions.push(format!(
                    "effective contract `{module}`: suppressed `{}` warning by {source} ignore rule: {warning}",
                    category.as_str()
                ));
                continue;
            }
            if authorship_known
                && let Some(section) = stub_section_warning(&warning)
                && !authored_sections.contains(&section.to_ascii_lowercase())
            {
                suppressions.push(format!(
                    "effective contract `{module}`: {warning}; no active change authored ## {section}, \
                     so it is not blocking — complete it in {spec_rel_path} or suppress it with \
                     `stub-section:{spec_rel_path}` in .specsyncignore"
                ));
                continue;
            }
            errors.push(format!("effective contract `{module}`: {warning}"));
        }
    }
    let _ = fs::remove_dir_all(temp);
    EffectiveContractOutcome {
        errors,
        suppressions,
    }
}

fn create_effective_contract_workspace() -> Result<PathBuf, String> {
    for _ in 0..1024 {
        let sequence = EFFECTIVE_CONTRACT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "specsync-effective-{}-{}-{sequence}",
            std::process::id(),
            now()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to claim effective-contract workspace {}: {error}",
                    path.display()
                ));
            }
        }
    }
    Err("failed to claim a unique effective-contract workspace after 1024 attempts".into())
}

fn dependency_ordered_changes(changes: Vec<&ChangeRecord>) -> Result<Vec<&ChangeRecord>, String> {
    let mut by_id = BTreeMap::new();
    for record in changes {
        if by_id.insert(record.id.as_str(), record).is_some() {
            return Err(format!(
                "duplicate active change `{}` prevents deterministic ordering",
                record.id
            ));
        }
    }
    let active_ids: BTreeSet<&str> = by_id.keys().copied().collect();
    let mut indegree = by_id
        .keys()
        .map(|id| (*id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (id, record) in &by_id {
        let dependencies = record
            .dependencies
            .iter()
            .map(String::as_str)
            .filter(|dependency| active_ids.contains(dependency))
            .collect::<BTreeSet<_>>();
        indegree.insert(id, dependencies.len());
        for dependency in dependencies {
            dependents.entry(dependency).or_default().insert(id);
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(*id))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(by_id.len());
    while let Some(id) = ready.pop_first() {
        ordered.push(by_id[id]);
        if let Some(children) = dependents.get(id) {
            for child in children {
                let count = indegree
                    .get_mut(child)
                    .expect("dependent change has an indegree");
                *count -= 1;
                if *count == 0 {
                    ready.insert(child);
                }
            }
        }
    }
    if ordered.len() != by_id.len() {
        return Err("active change dependency cycle prevents deterministic ordering".into());
    }
    Ok(ordered)
}

fn validate_delta_files(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let directory = change_dir(root, &record.id).join("deltas");
    let mut actual_modules = BTreeSet::new();
    if directory.exists() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| format!("failed to inspect semantic deltas: {error}"))?
        {
            let entry =
                entry.map_err(|error| format!("failed to inspect semantic delta: {error}"))?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("md") {
                return Err(format!(
                    "unexpected semantic delta entry `{}`; expected one .md file per affected spec",
                    entry.file_name().to_string_lossy()
                ));
            }
            let module = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| "semantic delta filename is not portable UTF-8".to_string())?;
            actual_modules.insert(module.to_string());
        }
    }
    let expected_modules: BTreeSet<String> = record.affected_specs.iter().cloned().collect();
    if record.no_spec_change {
        if !actual_modules.is_empty() {
            return Err("no-spec change contains semantic delta files".into());
        }
        return Ok(());
    }
    if actual_modules != expected_modules {
        let extra: Vec<&str> = actual_modules
            .difference(&expected_modules)
            .map(String::as_str)
            .collect();
        let missing: Vec<&str> = expected_modules
            .difference(&actual_modules)
            .map(String::as_str)
            .collect();
        return Err(format!(
            "semantic delta modules must exactly match affected specs (missing: {}; extra: {})",
            if missing.is_empty() {
                "none".into()
            } else {
                missing.join(", ")
            },
            if extra.is_empty() {
                "none".into()
            } else {
                extra.join(", ")
            }
        ));
    }
    let tombstones = removed_requirement_ids(root)?;
    for module in &record.affected_specs {
        let path = delta_path_checked(root, record, module)?;
        let content = read_bounded_change_text(&path, "semantic delta")
            .map_err(|error| format!("semantic delta for {module}: {error}"))?;
        let items = parse_delta(&content)?;
        if items.is_empty() {
            return Err(format!("semantic delta for `{module}` is empty"));
        }
        for item in items {
            if item.target == DeltaTarget::Requirement && item.operation != DeltaOperation::Removed
            {
                validate_requirement(&item.key, &item.content)?;
                let requirement_module = module.replace('_', "-");
                let expected_prefix = format!("REQ-{requirement_module}-");
                if !item.key.starts_with(&expected_prefix) {
                    return Err(format!(
                        "requirement ID `{}` must match affected module `{module}` as `{expected_prefix}<number>`",
                        item.key
                    ));
                }
            }
            if item.target == DeltaTarget::Requirement
                && item.operation == DeltaOperation::Added
                && tombstones.contains(&item.key)
            {
                return Err(format!(
                    "removed requirement ID `{}` is a permanent tombstone and cannot be reused",
                    item.key
                ));
            }
            // A block already present with exactly the declared content means this
            // delta is applied, not conflicting. Rejecting it here would make an
            // applied change permanently unverifiable.
            if item.target == DeltaTarget::Requirement
                && item.operation == DeltaOperation::Added
                && living_requirement_conflicts(root, module, &item.key, &item.content)?
            {
                return Err(format!(
                    "cannot add existing block `{}` with different content; use ## MODIFIED for requirements already present in the living tree",
                    item.key
                ));
            }
        }
    }
    Ok(())
}

/// Per-module digest over the EXACT bytes of every semantic delta file this change owns.
///
/// Keyed by module because that is what a refusal has to name: "the delta changed" is not an
/// actionable message when a change owns nine specs. The module name is framed into the digest
/// as well, so moving a body from `deltas/a.md` to `deltas/b.md` cannot preserve a digest.
///
/// A `no_spec_change` record yields an empty map, which is the truth: `validate_delta_files`
/// already refuses any delta file at all for such a change, so there are no bodies to bind.
fn delta_body_digests(
    root: &Path,
    record: &ChangeRecord,
) -> Result<BTreeMap<String, String>, String> {
    let mut digests = BTreeMap::new();
    if record.no_spec_change {
        return Ok(digests);
    }
    for module in &record.affected_specs {
        let path = delta_path_checked(root, record, module)?;
        let body = read_bounded_change_text(&path, "semantic delta")
            .map_err(|error| format!("semantic delta for {module}: {error}"))?;
        let mut digest = FramedDigest::new(APPROVED_DELTA_DIGEST_DOMAIN);
        digest.frame(b"module", module.as_bytes());
        digest.frame(b"body", body.as_bytes());
        digests.insert(module.clone(), digest.finish());
    }
    Ok(digests)
}

/// Refuses to apply a semantic delta whose body is not the body the approver signed (#704).
///
/// ABSENT EVIDENCE IS NOT A VIOLATION. Every approval written before `approved_delta_digests`
/// existed carries `None`, and every one of the 183 archived changes in this repository is in
/// that position. `None` means the approval made no claim about delta wording, so this returns
/// `Ok(())` and leaves the judgement to the gates that did exist at the time. Reading a missing
/// digest as tampering would fail all of recorded history on evidence nobody could have written
/// — the exact shape of #672, #684 and #689's first design.
fn ensure_approved_delta_bodies_unchanged(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    let approval = effective_definition_approval(root, record, &ledger)?;
    let Some(approved) = approval.approved_delta_digests.as_ref() else {
        return Ok(());
    };
    let current = delta_body_digests(root, record)?;
    let mut changed: Vec<&str> = approved
        .iter()
        .filter(|(module, digest)| current.get(*module) != Some(*digest))
        .map(|(module, _)| module.as_str())
        .collect();
    changed.extend(
        current
            .keys()
            .filter(|module| !approved.contains_key(*module))
            .map(String::as_str),
    );
    changed.sort_unstable();
    changed.dedup();
    if changed.is_empty() {
        return Ok(());
    }
    Err(format!(
        "semantic delta for {} changed after approval; the approved wording is what rewrites the canonical spec, so re-run `specsync change approve {}` to approve the current delta bodies (or restore them)",
        changed
            .iter()
            .map(|module| format!("`{module}`"))
            .collect::<Vec<_>>()
            .join(", "),
        record.id
    ))
}

/// True when living `requirements.md` already has a `### {key}` requirement heading.
/// True when living `requirements.md` already has a `### {key}` requirement whose
/// content differs from what an `## ADDED` delta declares.
///
/// A byte-equivalent block is an already-applied delta and is not a conflict, so
/// re-deriving the canonical tree converges instead of failing.
fn living_requirement_conflicts(
    root: &Path,
    module: &str,
    key: &str,
    declared: &str,
) -> Result<bool, String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let (_, requirements_path) = canonical_module_paths(root, &specs_dir, module)?;
    let Ok(source) = fs::read_to_string(&requirements_path) else {
        return Ok(false);
    };
    let heading = format!("### {key}");
    let Some(start) = source.find(&heading) else {
        return Ok(false);
    };
    let body = &source[start..];
    let end = body
        .match_indices("\n### ")
        .next()
        .map_or(body.len(), |(index, _)| index);
    Ok(!markdown_block_matches(&body[..end], &heading, declared))
}

fn removed_requirement_ids(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut removed = BTreeSet::new();
    let mut delta_roots = Vec::new();
    delta_roots.extend(
        list_changes_checked(root)?
            .into_iter()
            .filter(|record| record.state == ChangeState::Accepted || record.canonical_applied)
            .map(|record| change_dir(root, &record.id).join("deltas")),
    );
    delta_roots.push(root.join(ARCHIVE_PATH));
    for base in delta_roots {
        match fs::symlink_metadata(&base) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(format!(
                    "historical semantic delta root is not a directory: {}",
                    base.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "failed to inspect historical semantic delta root {}: {error}",
                    base.display()
                ));
            }
        }
        for entry in walkdir::WalkDir::new(&base) {
            let entry = entry.map_err(|error| {
                format!(
                    "failed to inspect historical semantic deltas under {}: {error}",
                    base.display()
                )
            })?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("md")
                || !path
                    .components()
                    .any(|component| component.as_os_str() == "deltas")
            {
                continue;
            }
            if !entry.file_type().is_file() {
                return Err(format!(
                    "historical semantic delta is not a regular file: {}",
                    path.display()
                ));
            }
            let content =
                read_bounded_change_text(path, "historical semantic delta").map_err(|error| {
                    format!(
                        "failed to read historical semantic delta {}: {error}",
                        path.display()
                    )
                })?;
            let items = parse_delta(&content).map_err(|error| {
                format!(
                    "invalid historical semantic delta {}: {error}",
                    path.display()
                )
            })?;
            if items.is_empty() {
                return Err(format!(
                    "historical semantic delta is empty: {}",
                    path.display()
                ));
            }
            for item in items {
                if item.target == DeltaTarget::Requirement
                    && item.operation == DeltaOperation::Removed
                {
                    removed.insert(item.key);
                }
            }
        }
    }
    Ok(removed)
}

fn collect_requirement_ids(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    if record.no_spec_change {
        return Ok(Vec::new());
    }
    let mut ids = BTreeSet::new();
    for module in &record.affected_specs {
        let path = delta_path_checked(root, record, module)?;
        let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        for item in parse_delta(&content)? {
            if item.target == DeltaTarget::Requirement && item.operation != DeltaOperation::Removed
            {
                ids.insert(item.key);
            }
        }
    }
    Ok(ids.into_iter().collect())
}

fn validate_requirement(id: &str, content: &str) -> Result<(), String> {
    if !id.starts_with("REQ-")
        || id.split('-').count() < 3
        || !id.rsplit('-').next().is_some_and(|number| {
            !number.is_empty() && number.chars().all(|value| value.is_ascii_digit())
        })
    {
        return Err(format!(
            "requirement ID `{id}` must use REQ-<module>-<number>"
        ));
    }
    if !content.contains(" SHALL ") && !content.trim_start().starts_with("SHALL ") {
        return Err(format!(
            "requirement `{id}` must contain a normative SHALL statement"
        ));
    }
    if !content.contains("Acceptance Criteria") {
        return Err(format!(
            "requirement `{id}` must include Acceptance Criteria"
        ));
    }
    Ok(())
}

fn parse_delta(content: &str) -> Result<Vec<DeltaItem>, String> {
    let mut operation: Option<DeltaOperation> = None;
    let mut current_target: Option<DeltaTarget> = None;
    let mut current_key = String::new();
    let mut body = Vec::new();
    let mut items = Vec::new();
    let flush = |items: &mut Vec<DeltaItem>,
                 operation: Option<DeltaOperation>,
                 target: Option<DeltaTarget>,
                 key: &str,
                 body: &mut Vec<String>| {
        if let (Some(operation), Some(target)) = (operation, target) {
            items.push(DeltaItem {
                operation,
                target,
                key: key.trim().to_string(),
                content: body.join("\n").trim().to_string(),
            });
        }
        body.clear();
    };
    for line in content.lines() {
        if let Some(header) = line.strip_prefix("## ") {
            flush(
                &mut items,
                operation,
                current_target,
                &current_key,
                &mut body,
            );
            current_target = None;
            current_key.clear();
            operation = match header.trim().to_ascii_uppercase().as_str() {
                "ADDED" => Some(DeltaOperation::Added),
                "MODIFIED" => Some(DeltaOperation::Modified),
                "REMOVED" => Some(DeltaOperation::Removed),
                _ => {
                    return Err(format!(
                        "invalid delta operation heading `## {header}` — expected one of: ## Added, ## Modified, ## Removed"
                    ));
                }
            };
            continue;
        }
        if let Some(header) = line.strip_prefix("### ") {
            // Classify BEFORE flushing. #564 taught this grammar that a `###` inside an open
            // item is content, but left the flush above the classification, so every content
            // heading still ended the item: one `## MODIFIED / ### SPEC SECTION X` carrying
            // `### Scenario` subheadings became SEVERAL items keyed X, each holding one
            // fragment, and application kept only the last. A spec section silently lost
            // everything above its final subheading — including behaviour the change never
            // touched. Fixing the classification without moving the flush was half a fix.
            let item_heading = strip_ascii_prefix_ignore_case(header, "REQUIREMENT ")
                .map(|value| (DeltaTarget::Requirement, value))
                .or_else(|| {
                    strip_ascii_prefix_ignore_case(header, "SPEC SECTION ")
                        .map(|value| (DeltaTarget::SpecSection, value))
                });
            let (target, key) = if let Some(parsed) = item_heading {
                flush(
                    &mut items,
                    operation,
                    current_target,
                    &current_key,
                    &mut body,
                );
                parsed
            } else if current_target.is_some() {
                // A `###` that is not one of this grammar's item headings, met
                // while inside an item, is section CONTENT — not a malformed
                // item. Rejecting it made a scaffolded spec impossible to
                // change through the lifecycle: `scaffold` writes
                // `### Structs & Enums`, `### Traits`, `### Functions` inside
                // `## Public API` and `### Consumes` inside `## Dependencies`,
                // so `approve` refused the section it had just generated
                // (#564). The grammar identifies its own items by keyword, so
                // depth was never what distinguished them.
                body.push(line.to_string());
                continue;
            } else {
                return Err(format!(
                    "invalid delta item heading `### {header}` — a delta item must be `### REQUIREMENT <id>` or `### SPEC SECTION <name>`; subheadings are only content once an item has been opened"
                ));
            };
            current_target = Some(target);
            current_key = key.trim().to_string();
            continue;
        }
        if current_target.is_some() {
            body.push(line.to_string());
        }
    }
    flush(
        &mut items,
        operation,
        current_target,
        &current_key,
        &mut body,
    );
    if items.iter().any(|item| item.key.is_empty()) {
        return Err("semantic delta contains an item with an empty key".into());
    }
    // Two items with the same operation/target/key under `## MODIFIED` silently overwrite:
    // application keeps the last and the earlier bodies vanish with no diagnostic. Be precise
    // about the scope — duplicate `## ADDED` already fails loudly ("cannot add existing block
    // ... with different content") and duplicate `## REMOVED` fails as a missing block. MODIFIED
    // is the one that resolves silently, so it is the one worth refusing.
    //
    // COUPLED TO THE FLUSH ORDERING ABOVE — do not ship this guard on its own. Two archived
    // deltas (CHG-0121 types.md, CHG-0131 deps.md) contain duplicate MODIFIED keys that exist
    // ONLY because the old ordering split one section into several items. With the reordering
    // they parse as single items and pass; without it, this guard would refuse them and their
    // changes could no longer be re-materialized.
    for (index, item) in items.iter().enumerate() {
        if let Some(earlier) = items[..index].iter().find(|candidate| {
            candidate.operation == item.operation
                && candidate.target == item.target
                && candidate.key == item.key
        }) {
            return Err(format!(
                "semantic delta declares {:?} {:?} `{}` more than once; applying it would keep \
                 only the last and silently discard the earlier body ({} bytes)",
                item.operation,
                item.target,
                item.key,
                earlier.content.len()
            ));
        }
    }
    if items.is_empty() && !content.trim().is_empty() {
        if operation.is_some() {
            return Err(
                "semantic delta contains no items under a recognized operation heading; each ## Added, ## Modified, or ## Removed section must contain ### REQUIREMENT <id> or ### SPEC SECTION <name>"
                    .into(),
            );
        }
        return Err(
            "semantic delta contains no recognized operation headings; expected one of: ## Added, ## Modified, ## Removed"
                .into(),
        );
    }
    Ok(items)
}

fn strip_ascii_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix_len = prefix.len();
    if value.len() >= prefix_len
        && value.as_bytes()[..prefix_len].eq_ignore_ascii_case(prefix.as_bytes())
    {
        Some(&value[prefix_len..])
    } else {
        None
    }
}

fn prepare_delta_application(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<(PathBuf, String)>, String> {
    if record.no_spec_change {
        return Ok(Vec::new());
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    let mut prepared = Vec::new();
    for module in &record.affected_specs {
        let delta =
            read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")?;
        let items = parse_delta(&delta)?;
        let (spec_path, requirements_path) = canonical_module_paths(root, &specs_dir, module)?;
        let mut spec = fs::read_to_string(&spec_path)
            .map_err(|error| format!("failed to read {}: {error}", spec_path.display()))?;
        let mut requirements = fs::read_to_string(&requirements_path)
            .unwrap_or_else(|_| format!("---\nspec: {module}.spec.md\n---\n\n# Requirements\n"));
        for item in items {
            match item.target {
                DeltaTarget::Requirement => {
                    requirements = apply_markdown_block(
                        &requirements,
                        "### ",
                        &item.key,
                        &item.content,
                        item.operation,
                    )?;
                }
                DeltaTarget::SpecSection => {
                    spec = apply_markdown_block(
                        &spec,
                        "## ",
                        &item.key,
                        &item.content,
                        item.operation,
                    )?;
                }
            }
        }
        spec = bump_spec_version(&spec)?;
        spec = append_changelog(&spec, &record.id, &record.title);
        prepared.push((spec_path, spec));
        prepared.push((requirements_path, requirements));
    }
    Ok(prepared)
}

fn canonical_module_paths(
    root: &Path,
    specs_dir: &str,
    module: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let registry_path = crate::registry::local_registry_path(root);
    let registered = match crate::registry::load_local_registry(root) {
        Ok(Some(registry)) => registry
            .specs
            .iter()
            .find(|(registered_module, _)| registered_module == module)
            .map(|(_, path)| path.clone()),
        Ok(None) => None,
        Err(_) => {
            return Err(format!(
                "failed to parse local registry {} while resolving `{module}`",
                registry_path.display()
            ));
        }
    };
    let spec_path = if let Some(path) = registered {
        safe_project_path(root, &path).map_err(|error| {
            format!("unsafe registry path for canonical module `{module}`: {error}")
        })?
    } else {
        root.join(specs_dir)
            .join(module)
            .join(format!("{module}.spec.md"))
    };
    let requirements_path = spec_path
        .parent()
        .ok_or_else(|| format!("canonical spec path has no parent: {}", spec_path.display()))?
        .join("requirements.md");
    Ok((spec_path, requirements_path))
}

/// True when an existing markdown block already carries exactly the declared
/// heading and body, ignoring line-ending style and surrounding blank lines.
///
/// Used to decide whether an `## ADDED` delta has already been applied. Only an
/// exact content match counts: a block that exists with different text is a real
/// conflict and must be declared `## MODIFIED`.
fn markdown_block_matches(existing: &str, heading: &str, content: &str) -> bool {
    fn normalize(value: &str) -> String {
        value
            .replace("\r\n", "\n")
            .trim_matches(['\n', ' ', '\t'])
            .to_string()
    }
    let existing = normalize(existing);
    let Some(body) = existing.strip_prefix(&normalize(heading)) else {
        return false;
    };
    normalize(body) == normalize(content)
}

fn apply_markdown_block(
    source: &str,
    prefix: &str,
    key: &str,
    content: &str,
    operation: DeltaOperation,
) -> Result<String, String> {
    let heading = format!("{prefix}{key}");
    let line_ending = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines = Vec::new();
    let mut offset = 0;
    for raw in source.split_inclusive('\n') {
        let line = raw
            .strip_suffix('\n')
            .unwrap_or(raw)
            .strip_suffix('\r')
            .unwrap_or_else(|| raw.strip_suffix('\n').unwrap_or(raw));
        lines.push((offset, line));
        offset += raw.len();
    }
    if offset < source.len() {
        lines.push((offset, &source[offset..]));
    }
    let start = lines
        .iter()
        .position(|(_, line)| line.trim_end() == heading);
    let target_level = prefix
        .chars()
        .take_while(|character| *character == '#')
        .count();
    let end = start.map(|index| {
        lines
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, (_, line))| {
                markdown_heading_level(line).is_some_and(|level| level <= target_level)
            })
            .map(|(_, (next_offset, _))| *next_offset)
            .unwrap_or(source.len())
    });
    match operation {
        // An ADDED block that is already present with exactly the declared content
        // is an applied delta, not a conflict. Re-deriving the canonical tree must
        // converge, otherwise the delta can never be reconciled against the tree
        // and a partially-applied run leaves the change permanently unverifiable.
        DeltaOperation::Added if start.is_some() => {
            if let (Some(start_index), Some(end_offset)) = (start, end) {
                let existing = &source[lines[start_index].0..end_offset];
                if markdown_block_matches(existing, &heading, content) {
                    return Ok(source.to_string());
                }
            }
            return Err(format!(
                "cannot add existing block `{key}` with different content; use ## MODIFIED to \
                 change a block already present in the living tree"
            ));
        }
        DeltaOperation::Modified | DeltaOperation::Removed if start.is_none() => {
            return Err(format!("cannot modify/remove missing block `{key}`"));
        }
        _ => {}
    }
    let replacement = if operation == DeltaOperation::Removed {
        String::new()
    } else {
        let normalized_content = content
            .trim_end_matches(['\r', '\n'])
            .replace("\r\n", "\n")
            .replace('\n', line_ending);
        format!("{heading}{line_ending}{line_ending}{normalized_content}{line_ending}{line_ending}")
    };
    if let (Some(start_index), Some(end_offset)) = (start, end) {
        let start_offset = lines[start_index].0;
        let mut output =
            String::with_capacity(source.len() - (end_offset - start_offset) + replacement.len());
        output.push_str(&source[..start_offset]);
        output.push_str(&replacement);
        output.push_str(&source[end_offset..]);
        Ok(output)
    } else {
        let mut output = source.to_string();
        if !output.is_empty() && !output.ends_with(line_ending) {
            output.push_str(line_ending);
        }
        if !output.is_empty() && !output.ends_with(&format!("{line_ending}{line_ending}")) {
            output.push_str(line_ending);
        }
        output.push_str(&replacement);
        Ok(output)
    }
}

fn markdown_heading_level(line: &str) -> Option<usize> {
    let level = line
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if level == 0 || line.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    Some(level)
}

fn bump_spec_version(content: &str) -> Result<String, String> {
    let mut found = false;
    let mut output = Vec::new();
    for line in content.lines() {
        if !found && line.starts_with("version:") {
            let version = line.trim_start_matches("version:").trim();
            let bumped = bump_version_scalar(version)?;
            output.push(format!("version: {bumped}"));
            found = true;
        } else {
            output.push(line.to_string());
        }
    }
    if !found {
        return Err("spec frontmatter is missing version".into());
    }
    Ok(format!("{}\n", output.join("\n")))
}

fn bump_version_scalar(raw: &str) -> Result<String, String> {
    let (value, prefix, suffix) = if let Some(quote) = raw
        .chars()
        .next()
        .filter(|value| *value == '\'' || *value == '"')
    {
        let close = raw[quote.len_utf8()..]
            .find(quote)
            .map(|index| index + quote.len_utf8())
            .ok_or_else(|| "spec version has an unterminated quoted scalar".to_string())?;
        let suffix = &raw[close + quote.len_utf8()..];
        if !suffix.trim().is_empty() && !suffix.trim_start().starts_with('#') {
            return Err("spec version has unsupported YAML scalar syntax".into());
        }
        (
            &raw[quote.len_utf8()..close],
            quote.to_string(),
            format!("{quote}{suffix}"),
        )
    } else {
        let comment = raw.find(" #").unwrap_or(raw.len());
        (&raw[..comment], String::new(), raw[comment..].to_string())
    };
    let bumped = if let Ok(integer) = value.trim().parse::<u64>() {
        (integer + 1).to_string()
    } else {
        let components: Vec<&str> = value.trim().split('.').collect();
        if components.len() != 3 {
            return Err("spec version must be an integer or major.minor.patch".into());
        }
        let major = components[0]
            .parse::<u64>()
            .map_err(|_| "spec version must be an integer or major.minor.patch")?;
        let minor = components[1]
            .parse::<u64>()
            .map_err(|_| "spec version must be an integer or major.minor.patch")?;
        let patch = components[2]
            .parse::<u64>()
            .map_err(|_| "spec version must be an integer or major.minor.patch")?;
        format!("{major}.{minor}.{}", patch + 1)
    };
    Ok(format!("{prefix}{bumped}{suffix}"))
}

fn append_changelog(content: &str, id: &str, title: &str) -> String {
    let description = format!("{id}: {}", title.replace('|', "\\|"));
    let default_row = format!("| {} | {description} |", today());
    if let Some(position) = content.rfind("## Change Log") {
        let search_start = position + "## Change Log".len();
        let section_end = content[search_start..]
            .find("\n## ")
            .map(|offset| search_start + offset)
            .unwrap_or(content.len());
        let row = changelog_table_row(&content[search_start..section_end], content, &description)
            .unwrap_or(default_row);
        let before = content[..section_end].trim_end();
        let after = &content[section_end..];
        return format!("{before}\n{row}\n{after}");
    }
    format!(
        "{}\n## Change Log\n\n| Date | Change |\n|------|--------|\n{row}\n",
        content.trim_end(),
        row = default_row,
    )
}

fn changelog_table_row(section: &str, spec: &str, description: &str) -> Option<String> {
    let header = section
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('|') && line.ends_with('|'))?;
    let columns: Vec<&str> = header.trim_matches('|').split('|').map(str::trim).collect();
    if columns.len() < 2 {
        return None;
    }

    let version = spec.lines().find_map(|line| {
        let raw = line.strip_prefix("version:")?.trim();
        let without_comment = raw.split_once(" #").map_or(raw, |(value, _)| value);
        Some(without_comment.trim().trim_matches(['\'', '"']).to_string())
    });
    let mut recognized_description = false;
    let cells: Vec<String> = columns
        .iter()
        .map(|column| {
            let normalized = column.to_ascii_lowercase();
            if normalized == "version" {
                version.clone().unwrap_or_default()
            } else if normalized == "date" {
                today()
            } else if matches!(
                normalized.as_str(),
                "change" | "changes" | "description" | "notes"
            ) {
                recognized_description = true;
                description.to_string()
            } else if normalized == "author" {
                "SpecSync".to_string()
            } else {
                String::new()
            }
        })
        .collect();
    recognized_description.then(|| format!("| {} |", cells.join(" | ")))
}

fn write_prepared_files(root: &Path, prepared: &[(PathBuf, String)]) -> Result<(), String> {
    write_prepared_files_checked(root, prepared, || Ok(()))
}

fn write_prepared_files_checked<Validate>(
    root: &Path,
    prepared: &[(PathBuf, String)],
    mut validate: Validate,
) -> Result<(), String>
where
    Validate: FnMut() -> Result<(), String>,
{
    let mut targets = BTreeSet::new();
    for (path, _) in prepared {
        validate_prepared_transaction_target(root, path)?;
        if !targets.insert(path) {
            return Err(format!(
                "transaction contains duplicate target {}",
                path.display()
            ));
        }
    }
    let backups: Vec<(PathBuf, Option<String>)> = prepared
        .iter()
        .map(|(path, _)| {
            let original = match fs::read_to_string(path) {
                Ok(content) => Some(content),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    return Err(format!(
                        "failed to preserve transaction target {}: {error}",
                        path.display()
                    ));
                }
            };
            Ok((path.clone(), original))
        })
        .collect::<Result<_, String>>()?;
    let journal: Vec<TransactionEntry> = backups
        .iter()
        .map(|(path, original)| {
            Ok(TransactionEntry {
                path: transaction_relative_path(root, path)?,
                original: original.clone(),
            })
        })
        .collect::<Result<_, String>>()?;
    let journal = TransactionJournal {
        schema_version: 1,
        entry_count: journal.len(),
        entries_digest: transaction_entries_digest(&journal)?,
        entries: journal,
    };
    validate_prepared_transaction_targets(root, prepared)?;
    validate()?;
    let journal_path = root.join(TRANSACTION_PATH);
    atomic_write_durable(&journal_path, json_content(&journal)?.as_bytes())
        .map_err(|error| format!("failed to publish transaction journal: {error}"))?;
    #[cfg(test)]
    run_transaction_after_journal_hook();
    if let Err(error) =
        validate_prepared_transaction_targets(root, prepared).and_then(|()| validate())
    {
        recover_pending_transaction(root)?;
        return Err(error);
    }
    for (index, (path, content)) in prepared.iter().enumerate() {
        let _ = index;
        #[cfg(test)]
        if transaction_write_failure_is_due(index) {
            recover_pending_transaction(root)?;
            return Err(format!(
                "injected atomic publication failure at {}",
                path.display()
            ));
        }
        if let Err(error) = validate_prepared_transaction_target(root, path) {
            recover_pending_transaction(root)?;
            return Err(error);
        }
        if let Err(error) = atomic_write_durable(path, content.as_bytes()) {
            recover_pending_transaction(root)?;
            return Err(format!(
                "atomic delta application failed at {}: {error}",
                path.display()
            ));
        }
    }
    if let Err(error) =
        validate_prepared_transaction_targets(root, prepared).and_then(|()| validate())
    {
        recover_pending_transaction(root)?;
        return Err(error);
    }
    remove_file_durable(&journal_path)
        .map_err(|error| format!("failed to clear transaction journal: {error}"))?;
    Ok(())
}

fn validate_prepared_transaction_targets(
    root: &Path,
    prepared: &[(PathBuf, String)],
) -> Result<(), String> {
    for (path, _) in prepared {
        validate_prepared_transaction_target(root, path)?;
    }
    Ok(())
}

fn validate_prepared_transaction_target(root: &Path, path: &Path) -> Result<(), String> {
    transaction_relative_path(root, path)?;
    reject_symlink_components_for(root, path, "transaction target")
}

fn transaction_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "transaction target escapes project root: {}",
            path.display()
        )
    })?;
    let relative = relative.to_str().ok_or_else(|| {
        format!(
            "transaction target is not valid UTF-8 and cannot be journaled losslessly: {}",
            path.display()
        )
    })?;
    #[cfg(unix)]
    if relative.contains('\\') {
        return Err(format!(
            "transaction target contains a Unix filename component with `\\` and cannot be journaled losslessly: {}",
            path.display()
        ));
    }
    let relative = relative.replace('\\', "/");
    if normalize_project_path(&relative)? != relative {
        return Err(format!(
            "transaction target is not a canonical project path: {}",
            path.display()
        ));
    }
    if relative == TRANSACTION_PATH {
        return Err("transaction payload cannot overwrite its own journal".into());
    }
    Ok(relative)
}

#[cfg(test)]
fn inject_transaction_write_failure(index: usize) {
    TRANSACTION_WRITE_FAILURE_INDEX.with(|target| {
        *target.borrow_mut() = Some(index);
    });
}

#[cfg(test)]
fn inject_transaction_after_journal_hook(hook: impl FnOnce() + 'static) {
    TRANSACTION_AFTER_JOURNAL_HOOK.with(|target| {
        *target.borrow_mut() = Some(Box::new(hook));
    });
}

#[cfg(test)]
fn run_transaction_after_journal_hook() {
    TRANSACTION_AFTER_JOURNAL_HOOK.with(|target| {
        if let Some(hook) = target.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(test)]
fn transaction_write_failure_is_due(index: usize) -> bool {
    TRANSACTION_WRITE_FAILURE_INDEX.with(|target| {
        let mut target = target.borrow_mut();
        if *target == Some(index) {
            *target = None;
            true
        } else {
            false
        }
    })
}

fn remove_empty_transaction_directories(root: &Path, start: Option<&Path>) {
    let mut current = start.map(Path::to_path_buf);
    while let Some(directory) = current {
        if directory == root || !directory.starts_with(root) {
            break;
        }
        let parent = directory.parent().map(Path::to_path_buf);
        if fs::remove_dir(&directory).is_err() {
            break;
        }
        current = parent;
    }
}

fn transaction_entries_digest(entries: &[TransactionEntry]) -> Result<String, String> {
    serde_json::to_vec(entries)
        .map(|bytes| sha256_hex(&bytes))
        .map_err(|error| format!("failed to hash transaction journal: {error}"))
}

fn atomic_write_durable(path: &Path, content: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("transaction target has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".specsync-transaction-")
        .tempfile_in(parent)
        .map_err(|error| format!("failed to stage {}: {error}", path.display()))?;
    let permissions = fs::metadata(path)
        .map(|metadata| metadata.permissions())
        .unwrap_or(default_transaction_permissions(temporary.path())?);
    fs::set_permissions(temporary.path(), permissions)
        .map_err(|error| format!("failed to preserve {} permissions: {error}", path.display()))?;
    temporary
        .write_all(content)
        .and_then(|()| temporary.flush())
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| format!("failed to durably stage {}: {error}", path.display()))?;
    temporary
        .persist(path)
        .map_err(|error| format!("failed to publish {}: {}", path.display(), error.error))?;
    sync_parent_directory(parent)
}

#[cfg(unix)]
fn default_transaction_permissions(_temporary: &Path) -> Result<fs::Permissions, String> {
    use std::os::unix::fs::PermissionsExt;
    Ok(fs::Permissions::from_mode(0o644))
}

#[cfg(not(unix))]
fn default_transaction_permissions(temporary: &Path) -> Result<fs::Permissions, String> {
    let mut permissions = fs::metadata(temporary)
        .map(|metadata| metadata.permissions())
        .map_err(|error| format!("failed to read staged file permissions: {error}"))?;
    // Clear the Windows read-only bit after staging. This is not the Unix
    // 0o666 world-writable case that clippy::permissions_set_readonly_false warns about.
    #[allow(clippy::permissions_set_readonly_false)]
    {
        permissions.set_readonly(false);
    }
    Ok(permissions)
}

fn remove_file_durable(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| error.to_string())?;
    let parent = path
        .parent()
        .ok_or_else(|| format!("transaction target has no parent: {}", path.display()))?;
    sync_parent_directory(parent)
}

#[cfg(unix)]
fn sync_parent_directory(directory: &Path) -> Result<(), String> {
    fs::File::open(directory)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("failed to sync directory {}: {error}", directory.display()))
}

#[cfg(windows)]
fn sync_parent_directory(directory: &Path) -> Result<(), String> {
    use std::io::ErrorKind;
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    // Directory FlushFileBuffers can fail with ERROR_ACCESS_DENIED while a
    // sibling handle (lifecycle lock, staged temp file) remains open. File bytes
    // are already synced; treat metadata durability as best-effort in that case.
    match OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(directory)
        .and_then(|file| file.sync_all())
    {
        Ok(()) => Ok(()),
        Err(error)
            if error.raw_os_error() == Some(5) || error.kind() == ErrorKind::PermissionDenied =>
        {
            Ok(())
        }
        Err(error) => Err(format!(
            "failed to sync directory {}: {error}",
            directory.display()
        )),
    }
}

#[cfg(not(any(unix, windows)))]
fn sync_parent_directory(_directory: &Path) -> Result<(), String> {
    Ok(())
}

fn rename_durable(source: &Path, destination: &Path) -> Result<(), String> {
    let source_parent = source
        .parent()
        .ok_or_else(|| format!("rename source has no parent: {}", source.display()))?;
    let destination_parent = destination.parent().ok_or_else(|| {
        format!(
            "rename destination has no parent: {}",
            destination.display()
        )
    })?;
    fs::rename(source, destination).map_err(|error| error.to_string())?;
    sync_parent_directory(source_parent)?;
    if destination_parent != source_parent {
        sync_parent_directory(destination_parent)?;
    }
    Ok(())
}

fn ensure_no_delta_conflicts(root: &Path, current: &ChangeRecord) -> Result<(), String> {
    if current.canonical_applied
        || matches!(current.state, ChangeState::Accepted | ChangeState::Archived)
    {
        return Ok(());
    }
    let current_keys = delta_keys(root, current)?;
    for other in list_changes_checked(root)? {
        if other.id == current.id
            || matches!(
                other.state,
                ChangeState::Draft | ChangeState::Accepted | ChangeState::Archived
            )
            || other.canonical_applied
            || change_depends_on(root, current, &other.id)
            || change_depends_on(root, &other, &current.id)
        {
            continue;
        }
        let overlap: Vec<String> = current_keys
            .intersection(&delta_keys(root, &other)?)
            .cloned()
            .collect();
        if !overlap.is_empty() {
            return Err(format!(
                "semantic delta conflicts with {} on {}; add a dependency or rebase",
                other.id,
                overlap.join(", ")
            ));
        }
    }
    Ok(())
}

fn ensure_dependencies_satisfied(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    for dependency in &record.dependencies {
        let dependency_record = load_change(root, dependency)?;
        if !matches!(
            dependency_record.state,
            ChangeState::Accepted | ChangeState::Archived
        ) {
            return Err(format!(
                "dependency `{dependency}` is {}; it must be accepted before {} can start",
                dependency_record.state.as_str(),
                record.id
            ));
        }
    }
    Ok(())
}

fn dependency_reaches(
    root: &Path,
    from: &str,
    target: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if from == target {
        return true;
    }
    if !visited.insert(from.to_string()) {
        return false;
    }
    load_change(root, from)
        .map(|record| {
            record
                .dependencies
                .iter()
                .any(|dependency| dependency_reaches(root, dependency, target, visited))
        })
        .unwrap_or(false)
}

fn change_depends_on(root: &Path, record: &ChangeRecord, target: &str) -> bool {
    record.dependencies.iter().any(|dependency| {
        dependency == target || dependency_reaches(root, dependency, target, &mut BTreeSet::new())
    })
}

fn delta_keys(root: &Path, record: &ChangeRecord) -> Result<BTreeSet<String>, String> {
    let mut keys = BTreeSet::new();
    if record.no_spec_change {
        return Ok(keys);
    }
    for module in &record.affected_specs {
        let path = delta_path_checked(root, record, module)?;
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) if record.state == ChangeState::Draft => continue,
            Err(error) => return Err(error.to_string()),
        };
        for item in parse_delta(&content)? {
            keys.insert(format!("{module}:{:?}:{}", item.target, item.key));
        }
    }
    Ok(keys)
}

fn definition_digest(root: &Path, record: &ChangeRecord) -> Result<String, String> {
    if record.workflow_version >= 2 {
        return scope_digest(root, record);
    }
    definition_digest_for_correction_count(root, record, record.correction_count, false)
}

fn execution_digest(root: &Path, record: &ChangeRecord) -> Result<String, String> {
    definition_digest_for_correction_count(root, record, record.correction_count, false)
}

fn approved_scope(root: &Path, record: &ChangeRecord) -> Result<ApprovedScopeV1, String> {
    let ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &ledger.corrections)?;
    if record.correction_count > ledger.corrections.len() as u64 {
        return Err(format!(
            "requested scope correction view {} exceeds ledger length {}",
            record.correction_count,
            ledger.corrections.len()
        ));
    }
    let effective = validate_correction_records_for_prefix(
        record,
        &ledger.corrections[..record.correction_count as usize],
    )?;
    let mut affected_specs = record.affected_specs.clone();
    affected_specs.sort();
    let mut affected_paths = record.affected_paths.clone();
    affected_paths.sort();
    let mut acceptance_criteria = record.acceptance_criteria.clone();
    acceptance_criteria.sort();
    let mut dependencies = record.dependencies.clone();
    dependencies.sort();
    let mut supersedes = record.supersedes.clone();
    for edge in &mut supersedes {
        edge.obligations.sort_by(|left, right| {
            (&left.path, &left.module, &left.predecessor_entry_digest).cmp(&(
                &right.path,
                &right.module,
                &right.predecessor_entry_digest,
            ))
        });
    }
    supersedes.sort_by(|left, right| left.predecessor_id.cmp(&right.predecessor_id));
    Ok(ApprovedScopeV1 {
        schema_version: 1,
        change_id: record.id.clone(),
        title: record.title.clone(),
        description: record.description.clone(),
        kind: record.kind,
        affected_specs,
        affected_paths,
        no_spec_change: record.no_spec_change,
        no_spec_change_rationale: record.no_spec_change_rationale.clone(),
        acceptance_criteria,
        dependencies,
        supersedes,
        answers: effective.answers,
    })
}

fn scope_digest_from_approved(scope: &ApprovedScopeV1) -> Result<String, String> {
    let bytes = serde_json::to_vec(scope)
        .map_err(|error| format!("failed to hash approved scope: {error}"))?;
    let mut digest = FramedDigest::new(SCOPE_DIGEST_DOMAIN);
    digest.frame(b"scope", &bytes);
    Ok(digest.finish())
}

fn scope_digest(root: &Path, record: &ChangeRecord) -> Result<String, String> {
    scope_digest_from_approved(&approved_scope(root, record)?)
}

fn scope_expansion(approved: &ApprovedScopeV1, current: &ApprovedScopeV1) -> Vec<String> {
    let mut changes = Vec::new();
    if approved.change_id != current.change_id {
        changes.push(format!(
            "change identity changed from `{}` to `{}`",
            approved.change_id, current.change_id
        ));
    }
    if approved.title != current.title {
        changes.push(format!(
            "title changed from `{}` to `{}`",
            approved.title, current.title
        ));
    }
    if approved.description != current.description {
        changes.push(format!(
            "intent changed from `{}` to `{}`",
            approved.description, current.description
        ));
    }
    if approved.kind != current.kind {
        changes.push(format!(
            "change kind changed from `{}` to `{}`",
            approved.kind.as_str(),
            current.kind.as_str()
        ));
    }
    if approved.no_spec_change != current.no_spec_change
        || approved.no_spec_change_rationale != current.no_spec_change_rationale
    {
        changes.push("spec-impact declaration changed".into());
    }
    if approved.answers != current.answers {
        changes.push("public-contract or architecture-risk declaration changed".into());
    }
    append_scope_changes(
        &mut changes,
        "affected canonical specs",
        &approved.affected_specs,
        &current.affected_specs,
    );
    append_scope_changes(
        &mut changes,
        "affected paths",
        &approved.affected_paths,
        &current.affected_paths,
    );
    append_scope_changes(
        &mut changes,
        "acceptance criteria",
        &approved.acceptance_criteria,
        &current.acceptance_criteria,
    );
    append_scope_changes(
        &mut changes,
        "dependencies",
        &approved.dependencies,
        &current.dependencies,
    );
    let approved_supersedes: BTreeSet<String> = approved
        .supersedes
        .iter()
        .filter_map(|edge| serde_json::to_string(edge).ok())
        .collect();
    let current_supersedes: BTreeSet<String> = current
        .supersedes
        .iter()
        .filter_map(|edge| serde_json::to_string(edge).ok())
        .collect();
    for edge in &current.supersedes {
        if let Ok(encoded) = serde_json::to_string(edge)
            && !approved_supersedes.contains(&encoded)
        {
            changes.push(format!(
                "semantic predecessor obligation added for `{}`",
                edge.predecessor_id
            ));
        }
    }
    for edge in &approved.supersedes {
        if let Ok(encoded) = serde_json::to_string(edge)
            && !current_supersedes.contains(&encoded)
        {
            changes.push(format!(
                "semantic predecessor obligation removed for `{}`",
                edge.predecessor_id
            ));
        }
    }
    changes
}

fn append_scope_changes(
    changes: &mut Vec<String>,
    label: &str,
    approved: &[String],
    current: &[String],
) {
    let approved: BTreeSet<&str> = approved.iter().map(String::as_str).collect();
    let current: BTreeSet<&str> = current.iter().map(String::as_str).collect();
    let added: Vec<&str> = current
        .iter()
        .copied()
        .filter(|value| !approved.contains(value))
        .collect();
    if !added.is_empty() {
        changes.push(format!("{label} added: {}", added.join(", ")));
    }
    let removed: Vec<&str> = approved
        .iter()
        .copied()
        .filter(|value| !current.contains(value))
        .collect();
    if !removed.is_empty() {
        changes.push(format!("{label} removed: {}", removed.join(", ")));
    }
}

fn definition_digest_for_correction_count(
    root: &Path,
    record: &ChangeRecord,
    correction_count: u64,
    explicit_false: bool,
) -> Result<String, String> {
    let ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &ledger.corrections)?;
    if correction_count > ledger.corrections.len() as u64 {
        return Err(format!(
            "requested correction view {correction_count} exceeds ledger length {}",
            ledger.corrections.len()
        ));
    }
    let mut canonical_record = record.clone();
    canonical_record.state = ChangeState::Draft;
    canonical_record.canonical_applied = false;
    canonical_record.correction_count = correction_count;
    canonical_record.updated_at = 0;
    let mut record_bytes = serde_json::to_vec(&canonical_record)
        .map_err(|error| format!("failed to hash change state: {error}"))?;
    if explicit_false {
        let state = b"\"state\":\"draft\"";
        let state_start = record_bytes
            .windows(state.len())
            .position(|window| window == state)
            .ok_or_else(|| "failed to locate canonical change state while hashing".to_string())?;
        let insert_at = state_start + state.len();
        record_bytes.splice(
            insert_at..insert_at,
            b",\"canonical_applied\":false".iter().copied(),
        );
    }
    definition_digest_from_record_bytes(
        root,
        record,
        &record_bytes,
        &ledger.corrections[..correction_count as usize],
    )
}

fn legacy_task_definition_digest_for_correction_count(
    root: &Path,
    record: &ChangeRecord,
    correction_count: u64,
    explicit_false: bool,
) -> Result<String, String> {
    let ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &ledger.corrections)?;
    if correction_count > ledger.corrections.len() as u64 {
        return Err(format!(
            "requested legacy correction view {correction_count} exceeds ledger length {}",
            ledger.corrections.len()
        ));
    }
    let mut canonical_record = record.clone();
    canonical_record.state = ChangeState::Draft;
    canonical_record.canonical_applied = false;
    canonical_record.correction_count = correction_count;
    canonical_record.updated_at = 0;
    let mut record_bytes = serde_json::to_vec(&canonical_record)
        .map_err(|error| format!("failed to hash legacy change state: {error}"))?;
    if explicit_false {
        let state = b"\"state\":\"draft\"";
        let state_start = record_bytes
            .windows(state.len())
            .position(|window| window == state)
            .ok_or_else(|| "failed to locate canonical legacy change state".to_string())?;
        let insert_at = state_start + state.len();
        record_bytes.splice(
            insert_at..insert_at,
            b",\"canonical_applied\":false".iter().copied(),
        );
    }
    let snapshot = definition_artifact_snapshot(
        root,
        record,
        &ledger.corrections[..correction_count as usize],
    )?;
    definition_digest_from_snapshot_with_task_mode(
        record,
        &record_bytes,
        &ledger.corrections[..correction_count as usize],
        &snapshot,
        false,
    )
}

fn definition_digest_with_explicit_false(
    root: &Path,
    record: &ChangeRecord,
) -> Result<String, String> {
    definition_digest_for_correction_count(root, record, record.correction_count, true)
}

fn definition_digest_matches(
    root: &Path,
    record: &ChangeRecord,
    expected: &str,
) -> Result<bool, String> {
    if definition_digest(root, record)? == expected {
        return Ok(true);
    }
    if record.workflow_version >= 2 {
        return Ok(false);
    }
    if definition_digest_with_explicit_false(root, record)? == expected {
        return Ok(true);
    }
    for explicit_false in [false, true] {
        if legacy_task_definition_digest_for_correction_count(
            root,
            record,
            record.correction_count,
            explicit_false,
        )? == expected
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn definition_digest_from_record_bytes(
    root: &Path,
    record: &ChangeRecord,
    record_bytes: &[u8],
    corrections: &[CorrectionRecord],
) -> Result<String, String> {
    let snapshot = definition_artifact_snapshot(root, record, corrections)?;
    definition_digest_from_snapshot(record, record_bytes, corrections, &snapshot)
}

#[derive(Debug, Clone)]
struct DefinitionArtifactSnapshot {
    entries: Vec<(String, u32, Vec<u8>, Option<String>)>,
    correction_mode: Option<u32>,
}

fn definition_artifact_snapshot(
    root: &Path,
    record: &ChangeRecord,
    corrections: &[CorrectionRecord],
) -> Result<DefinitionArtifactSnapshot, String> {
    let dir = find_change_dir(root, &record.id)?;
    let effective = validate_correction_records_for_prefix(record, corrections)?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for artifact in &effective.selected_artifacts {
        files.push((
            format!("{CHANGES_PATH}/{}/{}", record.id, artifact.file_name()),
            dir.join(artifact.file_name()),
        ));
    }
    let deltas = dir.join("deltas");
    match fs::read_dir(&deltas) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!(
                        "failed to enumerate definition deltas {}: {error}",
                        deltas.display()
                    )
                })?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| "definition delta name is not portable UTF-8".to_string())?;
                if name.len() > MAX_ACCEPTANCE_PATH_BYTES || files.len() >= MAX_ACCEPTANCE_ENTRIES {
                    return Err("definition delta inventory exceeds deterministic bounds".into());
                }
                strict_portable_relative_path(&name)?;
                files.push((
                    format!("{CHANGES_PATH}/{}/deltas/{name}", record.id),
                    entry.path(),
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "failed to enumerate definition deltas {}: {error}",
                deltas.display()
            ));
        }
    }
    if let Some(policy) = load_policy(root)
        && let Some(principles) = policy.principles_file
    {
        let path = safe_project_path(root, &principles)?;
        files.push((strict_portable_project_path(root, &path)?, path));
    }
    if !corrections.is_empty() {
        files.push((
            format!("{CHANGES_PATH}/{}/{CORRECTIONS_FILE}", record.id),
            dir.join(CORRECTIONS_FILE),
        ));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let candidate_paths = files
        .iter()
        .map(|(_, path)| strict_portable_project_path(root, path))
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = candidate_paths.iter().cloned().collect();
    let evidence = git_regular_file_evidence(root, &candidates)?;
    let mut correction_mode = None;
    let mut captured = Vec::new();
    for ((relative, path), candidate) in files.into_iter().zip(candidate_paths) {
        let entry = evidence.entry(&candidate)?;
        if entry.kind != AcceptanceInputKind::File || !matches!(entry.mode, 0o100644 | 0o100755) {
            return Err(format!(
                "selected definition artifact is not a regular file: {}",
                path.display()
            ));
        }
        if entry.payload.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
            return Err(format!(
                "approval input exceeds {} byte limit: {}",
                MAX_CHANGE_ARTIFACT_BYTES,
                path.display()
            ));
        }
        if relative.ends_with(&format!("/{CORRECTIONS_FILE}")) {
            correction_mode = Some(entry.mode);
        } else {
            captured.push((
                relative,
                entry.mode,
                entry.payload.clone(),
                entry.object.clone(),
            ));
        }
    }
    Ok(DefinitionArtifactSnapshot {
        entries: captured,
        correction_mode,
    })
}

fn definition_digest_from_snapshot(
    record: &ChangeRecord,
    record_bytes: &[u8],
    corrections: &[CorrectionRecord],
    snapshot: &DefinitionArtifactSnapshot,
) -> Result<String, String> {
    definition_digest_from_snapshot_with_task_mode(
        record,
        record_bytes,
        corrections,
        snapshot,
        true,
    )
}

fn definition_digest_from_snapshot_with_task_mode(
    record: &ChangeRecord,
    record_bytes: &[u8],
    corrections: &[CorrectionRecord],
    snapshot: &DefinitionArtifactSnapshot,
    normalize_task_progress: bool,
) -> Result<String, String> {
    let mut digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
    digest.frame(b"record", record_bytes);
    for (relative, mode, payload, _) in &snapshot.entries {
        let canonical_payload = if normalize_task_progress {
            canonical_definition_artifact_payload(relative, payload)
        } else {
            payload.clone()
        };
        digest.entry(relative, b"file", *mode, &canonical_payload);
    }
    if !corrections.is_empty() {
        let relative = format!("{CHANGES_PATH}/{}/{CORRECTIONS_FILE}", record.id);
        let ledger = CorrectionLedger {
            schema_version: 1,
            corrections: corrections.to_vec(),
        };
        let content = json_content(&ledger)?;
        let mode = snapshot
            .correction_mode
            .ok_or_else(|| "captured correction ledger mode is missing".to_string())?;
        digest.entry(&relative, b"file", mode, content.as_bytes());
    }
    Ok(digest.finish())
}

fn canonical_definition_artifact_payload(relative: &str, payload: &[u8]) -> Vec<u8> {
    if !relative.ends_with("/tasks.md") {
        return payload.to_vec();
    }

    let mut canonical = payload.to_vec();
    for offset in markdown_task_checkbox_offsets(&canonical) {
        canonical[offset] = b' ';
    }
    canonical
}

fn markdown_task_checkbox_offsets(payload: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut line_start = 0;
    while line_start < payload.len() {
        let line_end = payload[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| line_start + offset)
            .unwrap_or(payload.len());
        let mut marker = line_start;
        while marker < line_end && matches!(payload[marker], b' ' | b'\t') {
            marker += 1;
        }
        if marker < line_end && matches!(payload[marker], b'`' | b'~') {
            let fence_byte = payload[marker];
            let fence_length = payload[marker..line_end]
                .iter()
                .take_while(|byte| **byte == fence_byte)
                .count();
            if fence_length >= 3 {
                match fence {
                    Some((open_byte, open_length))
                        if open_byte == fence_byte && fence_length >= open_length =>
                    {
                        fence = None;
                    }
                    None => fence = Some((fence_byte, fence_length)),
                    _ => {}
                }
                line_start = line_end.saturating_add(1);
                continue;
            }
        }
        if fence.is_none()
            && marker + 4 < line_end
            && matches!(payload[marker], b'-' | b'+' | b'*')
            && payload[marker + 1] == b' '
            && payload[marker + 2] == b'['
            && matches!(payload[marker + 3], b' ' | b'x' | b'X')
            && payload[marker + 4] == b']'
            && (marker + 5 == line_end || matches!(payload[marker + 5], b' ' | b'\t' | b'\r'))
        {
            offsets.push(marker + 3);
        }
        line_start = line_end.saturating_add(1);
    }
    offsets
}

#[derive(Serialize)]
struct DefinitionProjectionV501<'a> {
    schema_version: u32,
    id: &'a str,
    slug: &'a str,
    title: &'a str,
    description: &'a str,
    kind: ChangeKind,
    state: ChangeState,
    base_commit: &'a Option<String>,
    created_at: u64,
    updated_at: u64,
    affected_specs: &'a [String],
    affected_paths: &'a [String],
    no_spec_change: bool,
    no_spec_change_rationale: &'a Option<String>,
    acceptance_criteria: &'a [String],
    selected_artifacts: &'a [ArtifactKind],
    dependencies: &'a [String],
    answers: &'a BTreeMap<String, String>,
}

fn portable_definition_digest_pair_v501(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(String, String, String), String> {
    portable_definition_digest_pair_v501_with_task_mode(root, record, true)
}

fn portable_definition_digest_pair_v501_with_task_mode(
    root: &Path,
    record: &ChangeRecord,
    normalize_task_progress: bool,
) -> Result<(String, String, String), String> {
    if record.legacy_archive_baseline_digest.is_none() {
        return Err(
            "SpecSync 5.0.1 portable approval requires a versioned legacy archive baseline binding"
                .into(),
        );
    }
    if record.workflow_version != 1
        || record.canonical_applied
        || record.correction_count != 0
        || !record.supersedes.is_empty()
        || !record.acceptance_owner_corrections.is_empty()
    {
        return Err(
            "SpecSync 5.0.1 portable projection rejects unsupported nonempty post-5.0.1 definition fields"
                .into(),
        );
    }
    let correction_ledger = load_correction_ledger(root, record)?;
    if correction_ledger.schema_version != 1 || !correction_ledger.corrections.is_empty() {
        return Err(
            "SpecSync 5.0.1 portable projection rejects unsupported correction history".into(),
        );
    }
    let mut canonical = record.clone();
    canonical.state = ChangeState::Draft;
    canonical.canonical_applied = false;
    canonical.correction_count = 0;
    canonical.updated_at = 0;
    let current_bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("failed to hash current change state: {error}"))?;
    let legacy_bytes = definition_projection_bytes_v501(record)?;
    let snapshot = definition_artifact_snapshot(root, record, &[])?;
    for (relative, _, captured_payload, object) in &snapshot.entries {
        let working_payload = fs::read(root.join(relative)).map_err(|error| {
            format!(
                "SpecSync 5.0.1 portable projection cannot read working-tree definition artifact `{relative}`: {error}"
            )
        })?;
        let canonical_payload = match object {
            Some(object) => git_blob_bytes(root, object)?,
            None => captured_payload.clone(),
        };
        if working_payload != canonical_payload {
            return Err(format!(
                "SpecSync 5.0.1 portable projection requires canonical working-tree bytes for `{relative}`; use a canonical LF release checkout"
            ));
        }
    }
    let current = definition_digest_from_snapshot_with_task_mode(
        record,
        &current_bytes,
        &[],
        &snapshot,
        normalize_task_progress,
    )?;
    let projected = definition_digest_from_snapshot_with_task_mode(
        record,
        &legacy_bytes,
        &[],
        &snapshot,
        normalize_task_progress,
    )?;
    if current == projected {
        return Err("portable definition pair digests must be distinct".into());
    }
    let correction_prefix = correction_prefix_digest(record, &[])?;
    Ok((current, projected, correction_prefix))
}

fn definition_projection_bytes_v501(record: &ChangeRecord) -> Result<Vec<u8>, String> {
    let mut canonical = record.clone();
    canonical.state = ChangeState::Draft;
    canonical.updated_at = 0;
    let legacy = DefinitionProjectionV501 {
        schema_version: canonical.schema_version,
        id: &canonical.id,
        slug: &canonical.slug,
        title: &canonical.title,
        description: &canonical.description,
        kind: canonical.kind,
        state: canonical.state,
        base_commit: &canonical.base_commit,
        created_at: canonical.created_at,
        updated_at: canonical.updated_at,
        affected_specs: &canonical.affected_specs,
        affected_paths: &canonical.affected_paths,
        no_spec_change: canonical.no_spec_change,
        no_spec_change_rationale: &canonical.no_spec_change_rationale,
        acceptance_criteria: &canonical.acceptance_criteria,
        selected_artifacts: &canonical.selected_artifacts,
        dependencies: &canonical.dependencies,
        answers: &canonical.answers,
    };
    serde_json::to_vec(&legacy)
        .map_err(|error| format!("failed to hash SpecSync 5.0.1 projection: {error}"))
}

fn correction_prefix_digest(
    record: &ChangeRecord,
    corrections: &[CorrectionRecord],
) -> Result<String, String> {
    let mut digest = FramedDigest::new(CORRECTION_PREFIX_DOMAIN);
    digest.frame(b"change-id", record.id.as_bytes());
    digest.frame(b"count", corrections.len().to_string().as_bytes());
    for correction in corrections {
        let bytes = serde_json::to_vec(correction)
            .map_err(|error| format!("failed to hash correction prefix: {error}"))?;
        digest.frame(b"correction", &bytes);
    }
    Ok(digest.finish())
}

fn project_input_digest(root: &Path) -> Result<String, String> {
    if let Some(digest) = read_scope_value(root, |scope| scope.project_input_digest.clone()) {
        return digest;
    }
    let result = project_input_digest_uncached(root);
    update_read_scope(root, |scope| {
        scope.project_input_digest = Some(result.clone());
    });
    result
}

fn project_input_digest_uncached(root: &Path) -> Result<String, String> {
    let (paths, evidence) = stable_discovered_evidence(root, None, &BTreeSet::new(), false)?;
    let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
    for relative in paths {
        if project_input_is_volatile(&relative) {
            continue;
        }
        let entry = evidence.entry(&relative)?;
        digest.entry(
            &relative,
            acceptance_kind_bytes(&entry.kind),
            entry.mode,
            &entry.payload,
        );
    }
    Ok(digest.finish())
}

fn git_project_paths(root: &Path) -> Result<Option<Vec<String>>, String> {
    if !git_repository_present(root)? {
        return Ok(None);
    }
    let output = run_git_required(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        None,
        MAX_GIT_EVIDENCE_PATH_BYTES + MAX_GIT_EVIDENCE_PATHS,
    )?;
    let mut paths = Vec::new();
    let mut total = 0_usize;
    for path in output
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        if git_path_bytes_are_volatile(path) {
            continue;
        }
        if paths.len() >= MAX_GIT_EVIDENCE_PATHS || path.len() > 4096 {
            return Err("Git project path inventory exceeds deterministic bounds".into());
        }
        total = total
            .checked_add(path.len())
            .ok_or_else(|| "Git project path inventory exceeds deterministic bounds".to_string())?;
        if total > MAX_GIT_EVIDENCE_PATH_BYTES {
            return Err("Git project path inventory exceeds deterministic bounds".into());
        }
        let path = std::str::from_utf8(path)
            .map_err(|_| "non-UTF-8 Git path cannot be hashed portably".to_string())?;
        paths.push(strict_portable_relative_path(path)?);
    }
    Ok(Some(paths))
}

fn git_scoped_project_paths(
    root: &Path,
    scopes: &BTreeSet<String>,
) -> Result<Option<Vec<String>>, String> {
    if !git_repository_present(root)? {
        return Ok(None);
    }
    let mut paths = Vec::new();
    let mut total = 0_usize;
    let preserve_volatile = scopes.iter().any(|scope| project_input_is_volatile(scope));
    let repo_prefix = git_repo_prefix(root)?;
    for batch in candidate_argument_batches(scopes) {
        let args = literal_candidate_git_args(
            &repo_prefix,
            &[
                "ls-files",
                "--cached",
                "--others",
                "--exclude-standard",
                "-z",
            ],
            &batch,
        );
        let args = borrowed_git_args(&args);
        let output = run_git_required(
            root,
            &args,
            None,
            MAX_GIT_EVIDENCE_PATH_BYTES + MAX_GIT_EVIDENCE_PATHS,
        )?;
        for path in output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            if !preserve_volatile && git_path_bytes_are_volatile(path) {
                continue;
            }
            if paths.len() >= MAX_GIT_EVIDENCE_PATHS || path.len() > 4096 {
                return Err("scoped Git path inventory exceeds deterministic bounds".into());
            }
            total = total.checked_add(path.len()).ok_or_else(|| {
                "scoped Git path inventory exceeds deterministic bounds".to_string()
            })?;
            if total > MAX_GIT_EVIDENCE_PATH_BYTES {
                return Err("scoped Git path inventory exceeds deterministic bounds".into());
            }
            let path = std::str::from_utf8(path)
                .map_err(|_| "non-UTF-8 governed Git path cannot be hashed portably".to_string())?;
            paths.push(strict_portable_relative_path(path)?);
        }
    }
    paths.sort();
    paths.dedup();
    Ok(Some(paths))
}

fn git_path_bytes_are_volatile(path: &[u8]) -> bool {
    [
        b".git/".as_slice(),
        b"target/".as_slice(),
        b"node_modules/".as_slice(),
        b"site/node_modules/".as_slice(),
        b"site/dist/".as_slice(),
        b"site/.astro/".as_slice(),
        b".specsync/changes/".as_slice(),
        b".specsync/archive/".as_slice(),
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
}

fn acceptance_discovery_scopes(
    root: &Path,
    record: &ChangeRecord,
) -> Result<BTreeSet<String>, String> {
    let mut scopes = BTreeSet::new();
    for affected in &record.affected_paths {
        let scope = affected.trim_end_matches('/');
        if !scope.is_empty() {
            scopes.insert(if scope == "." {
                scope.to_string()
            } else {
                strict_portable_relative_path(scope)?
            });
        }
    }
    let config = crate::config::load_config(root);
    for module in &record.affected_specs {
        let (spec, _) = canonical_module_paths(root, &config.specs_dir, module)?;
        let parent = spec
            .parent()
            .ok_or_else(|| format!("canonical spec path has no parent: {}", spec.display()))?;
        scopes.insert(strict_portable_project_path(root, parent)?);
    }
    for obligation in record
        .supersedes
        .iter()
        .flat_map(|edge| edge.obligations.iter())
    {
        scopes.insert(obligation.path.clone());
    }
    Ok(scopes)
}

fn strict_walk_project_paths(root: &Path) -> Result<Vec<String>, String> {
    let mut prior = None;
    for attempt in 0..2 {
        let mut paths = Vec::new();
        let mut total = 0_usize;
        let entries = walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let relative = portable_project_path(root, entry.path());
                relative.is_empty() || !project_input_is_volatile(&relative)
            });
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("failed to enumerate project tree: {error}"))?;
            let relative = strict_portable_project_path(root, entry.path())?;
            if !relative.is_empty() && project_input_is_volatile(&relative) {
                continue;
            }
            if !entry.file_type().is_file() && !entry.file_type().is_symlink() {
                continue;
            }
            if relative.len() > 4096 || paths.len() >= MAX_GIT_EVIDENCE_PATHS {
                return Err(
                    "filesystem project path inventory exceeds deterministic bounds".into(),
                );
            }
            total = total.checked_add(relative.len()).ok_or_else(|| {
                "filesystem project path inventory exceeds deterministic bounds".to_string()
            })?;
            if total > MAX_GIT_EVIDENCE_PATH_BYTES {
                return Err(
                    "filesystem project path inventory exceeds deterministic bounds".into(),
                );
            }
            paths.push(relative);
        }
        paths.sort();
        paths.dedup();
        if prior.as_ref().is_some_and(|prior| prior == &paths) || attempt == 1 && prior.is_none() {
            return Ok(paths);
        }
        prior = Some(paths);
    }
    Err("filesystem project inventory changed during evidence inspection".into())
}

fn strict_walk_scoped_project_paths(
    root: &Path,
    scopes: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let mut prior = None;
    for attempt in 0..2 {
        let mut paths = Vec::new();
        let mut total = 0_usize;
        for scope in scopes {
            let scoped_root = root.join(scope);
            match fs::symlink_metadata(&scoped_root) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(format!(
                        "failed to inspect scoped project path `{scope}`: {error}"
                    ));
                }
                Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {
                    paths.push(scope.clone());
                    total = total.saturating_add(scope.len());
                }
                Ok(metadata) if metadata.is_dir() => {
                    let preserve_volatile = project_input_is_volatile(scope);
                    let entries = walkdir::WalkDir::new(&scoped_root)
                        .follow_links(false)
                        .into_iter()
                        .filter_entry(|entry| {
                            preserve_volatile
                                || entry.path() == scoped_root
                                || strict_portable_project_path(root, entry.path())
                                    .is_ok_and(|path| !project_input_is_volatile(&path))
                        });
                    for entry in entries {
                        let entry = entry.map_err(|error| {
                            format!("failed to enumerate scoped project tree `{scope}`: {error}")
                        })?;
                        if entry.path() == scoped_root
                            || (!entry.file_type().is_file() && !entry.file_type().is_symlink())
                        {
                            continue;
                        }
                        let path = strict_portable_project_path(root, entry.path())?;
                        total = total.checked_add(path.len()).ok_or_else(|| {
                            "scoped filesystem inventory exceeds deterministic bounds".to_string()
                        })?;
                        paths.push(path);
                    }
                }
                Ok(_) => {
                    return Err(format!(
                        "scoped project path is not a file or directory: `{scope}`"
                    ));
                }
            }
            if paths.len() > MAX_GIT_EVIDENCE_PATHS || total > MAX_GIT_EVIDENCE_PATH_BYTES {
                return Err("scoped filesystem inventory exceeds deterministic bounds".into());
            }
        }
        paths.sort();
        paths.dedup();
        if prior.as_ref().is_some_and(|prior| prior == &paths) || attempt == 1 && prior.is_none() {
            return Ok(paths);
        }
        prior = Some(paths);
    }
    Err("scoped filesystem inventory changed during evidence inspection".into())
}

fn stable_discovered_evidence(
    root: &Path,
    scopes: Option<&BTreeSet<String>>,
    extra_candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<(Vec<String>, GitEvidence), String> {
    let key = DiscoveredEvidenceCacheKey {
        scopes: scopes.map(|values| values.iter().cloned().collect()),
        extra_candidates: extra_candidates.iter().cloned().collect(),
        regular_files_only,
    };
    if let Some(evidence) =
        read_scope_value(root, |scope| scope.discovered_evidence.get(&key).cloned())
    {
        return evidence;
    }
    let result = stable_discovered_evidence_with_hook_internal(
        root,
        scopes,
        extra_candidates,
        regular_files_only,
        |_, _| {},
    );
    update_read_scope(root, |scope| {
        if scope.discovered_evidence.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            scope.discovered_evidence.insert(key, result.clone());
        }
    });
    result
}

#[cfg(test)]
fn stable_discovered_evidence_with_hook<Hook>(
    root: &Path,
    scopes: Option<&BTreeSet<String>>,
    extra_candidates: &BTreeSet<String>,
    regular_files_only: bool,
    hook: Hook,
) -> Result<(Vec<String>, GitEvidence), String>
where
    Hook: FnMut(usize, &Path),
{
    stable_discovered_evidence_with_hook_internal(
        root,
        scopes,
        extra_candidates,
        regular_files_only,
        hook,
    )
}

fn stable_discovered_evidence_with_hook_internal<Hook>(
    root: &Path,
    scopes: Option<&BTreeSet<String>>,
    extra_candidates: &BTreeSet<String>,
    regular_files_only: bool,
    mut hook: Hook,
) -> Result<(Vec<String>, GitEvidence), String>
where
    Hook: FnMut(usize, &Path),
{
    let initial_context = repository_context(root)?;
    for attempt in 0..2 {
        let capture =
            |candidates: &BTreeSet<String>| -> Result<(Option<String>, GitEvidence), String> {
                if initial_context.git {
                    let index = git_index_fingerprint(root)?;
                    let inspected = inspect_git_candidates(root, candidates, regular_files_only)?;
                    Ok((
                        Some(index),
                        GitEvidence {
                            modes: inspected.modes,
                            entries: inspected.entries,
                        },
                    ))
                } else {
                    Ok((
                        None,
                        GitEvidence {
                            modes: BTreeMap::new(),
                            entries: capture_non_git_candidates(
                                root,
                                candidates,
                                regular_files_only,
                            )?,
                        },
                    ))
                }
            };
        let discover = || -> Result<Vec<String>, String> {
            match scopes {
                Some(scopes) => match git_scoped_project_paths(root, scopes)? {
                    Some(paths) => Ok(paths),
                    None => strict_walk_scoped_project_paths(root, scopes),
                },
                None => match git_project_paths(root)? {
                    Some(paths) => Ok(paths),
                    None => strict_walk_project_paths(root),
                },
            }
        };
        let mut before_paths = discover()?;
        before_paths.retain(|path| {
            !project_input_is_volatile(path) || explicitly_scoped_path(scopes, path)
        });
        before_paths.extend(extra_candidates.iter().cloned());
        before_paths.sort();
        before_paths.dedup();
        let before_candidates = before_paths.iter().cloned().collect();
        let (before_index, before) = capture(&before_candidates)?;
        hook(attempt, root);

        let mut after_paths = discover()?;
        after_paths.retain(|path| {
            !project_input_is_volatile(path) || explicitly_scoped_path(scopes, path)
        });
        after_paths.extend(extra_candidates.iter().cloned());
        after_paths.sort();
        after_paths.dedup();
        let after_candidates = after_paths.iter().cloned().collect();
        let (after_index, after) = capture(&after_candidates)?;
        let after_context = repository_context(root)?;
        if initial_context == after_context
            && before_index == after_index
            && before_paths == after_paths
            && before == after
        {
            return Ok((before_paths, before));
        }
        if attempt == 1 {
            return Err(
                "repository identity, discovered inventory, or captured evidence changed during inspection"
                    .into(),
            );
        }
    }
    unreachable!()
}

fn explicitly_scoped_path(scopes: Option<&BTreeSet<String>>, path: &str) -> bool {
    scopes.is_some_and(|scopes| {
        scopes.iter().any(|scope| {
            path == scope
                || path
                    .strip_prefix(scope)
                    .is_some_and(|suffix| suffix.starts_with('/'))
        })
    })
}

fn project_input_is_volatile(path: &str) -> bool {
    [
        ".git/",
        "target/",
        "node_modules/",
        "site/node_modules/",
        "site/dist/",
        "site/.astro/",
        ".specsync/changes/",
        ".specsync/archive/",
        ".specsync/hashes.json",
        LOCK_PATH,
        TRANSACTION_PATH,
    ]
    .iter()
    .any(|prefix| path == prefix.trim_end_matches('/') || path.starts_with(prefix))
}

fn strict_portable_relative_path(path: &str) -> Result<String, String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path
            .as_bytes()
            .get(1)
            .is_some_and(|byte| *byte == b':' && path.as_bytes()[0].is_ascii_alphabetic())
    {
        return Err(format!(
            "project path is not a portable relative path: `{path}`"
        ));
    }
    let components: Vec<&str> = path.split('/').collect();
    if components
        .iter()
        .any(|component| component.is_empty() || *component == "." || *component == "..")
    {
        return Err(format!(
            "project path is not a portable relative path: `{path}`"
        ));
    }
    Ok(components.join("/"))
}

fn strict_portable_project_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "project path is not a portable relative path: {}",
            path.display()
        ));
    }
    let mut components = Vec::new();
    for component in relative.components() {
        let component = component.as_os_str().to_str().ok_or_else(|| {
            format!(
                "non-UTF-8 project path cannot be hashed portably: {}",
                path.display()
            )
        })?;
        if component.contains('\\') {
            return Err(format!(
                "project path is not portable because a component contains a backslash: {}",
                path.display()
            ));
        }
        components.push(component);
    }
    Ok(components.join("/"))
}

fn closing_digest(record: &ChangeRecord, verification: &VerificationRecord) -> String {
    let mut digest = FramedDigest::new(CLOSING_DIGEST_DOMAIN);
    digest.frame(b"record-id", record.id.as_bytes());
    digest.frame(b"contract", verification.contract_digest.as_bytes());
    if let Some(execution) = &verification.execution_digest {
        digest.frame(b"execution", execution.as_bytes());
    }
    digest.frame(b"workspace", verification.workspace_digest.as_bytes());
    digest.frame(
        b"commit",
        verification.commit.as_deref().unwrap_or("").as_bytes(),
    );
    if let Some(acceptance) = &verification.acceptance_input_digest {
        // Presence is explicit: an absent field and an empty field cannot alias.
        let mut value = Vec::with_capacity(acceptance.len() + 1);
        value.push(1);
        value.extend_from_slice(acceptance.as_bytes());
        digest.frame(b"acceptance", &value);
    } else {
        digest.frame(b"acceptance", &[0]);
    }
    if let Some(manifest) = &verification.acceptance_manifest {
        let value = serde_json::to_vec(manifest).unwrap_or_default();
        digest.frame(b"acceptance-input-manifest-v1", &value);
    }
    if let Some(succession) = &verification.semantic_succession {
        let value = semantic_succession_digest(succession).unwrap_or_else(|_| "invalid".into());
        digest.frame(b"semantic-succession-v1", value.as_bytes());
    }
    digest.finish()
}

/// Governs how owner resolution treats production-source inputs with no deterministic
/// canonical owner. Current acceptance stays fail-closed; legacy acceptance-manifest
/// reconstruction assigns the exact delivery owner so adoption-era archived ledgers validate
/// without per-repo remediation.
#[derive(Clone, Copy, PartialEq, Eq)]
enum UnownedProductionSource {
    Reject,
    AssignExactDelivery,
}

fn acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(
        root,
        record,
        overrides,
        None,
        UnownedProductionSource::Reject,
    )
}

fn acceptance_manifest_legacy(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(
        root,
        record,
        overrides,
        None,
        UnownedProductionSource::AssignExactDelivery,
    )
}

fn acceptance_manifest_with_signed_owners(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
    signed: &AcceptanceManifestV1,
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(
        root,
        record,
        overrides,
        Some(signed),
        UnownedProductionSource::Reject,
    )
}

fn acceptance_manifest_internal(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
    signed: Option<&AcceptanceManifestV1>,
    unowned_source: UnownedProductionSource,
) -> Result<AcceptanceManifestV1, String> {
    let discovery_scopes = acceptance_discovery_scopes(root, record)?;
    let override_content: BTreeMap<String, &[u8]> = overrides
        .iter()
        .map(|(path, content)| {
            Ok((
                strict_portable_project_path(root, path)?,
                content.as_bytes(),
            ))
        })
        .collect::<Result<_, String>>()?;
    let mut extra_candidates = override_content.keys().cloned().collect::<BTreeSet<_>>();
    extra_candidates.extend(
        record
            .affected_paths
            .iter()
            .filter_map(|scope| (!scope.ends_with('/')).then_some(scope.clone())),
    );
    extra_candidates.extend(record.supersedes.iter().flat_map(|edge| {
        edge.obligations
            .iter()
            .map(|obligation| obligation.path.clone())
    }));
    if let Some(signed) = signed {
        extra_candidates.extend(signed.entries.iter().map(|entry| entry.path.clone()));
    }
    let (mut paths, evidence) =
        stable_discovered_evidence(root, Some(&discovery_scopes), &extra_candidates, false)?;
    paths.retain(|path| {
        (!project_input_is_volatile(path) || path == LEGACY_BASELINE_PATH)
            && record_covers_project_path(root, record, path)
    });
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut entries = Vec::new();
    for relative in paths {
        let (kind, mode, payload) = if let Some(content) = override_content.get(&relative) {
            let mode = evidence.generated_file_mode(&relative)?;
            let kind = acceptance_kind_for_mode(mode);
            (kind, mode, content.to_vec())
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            (
                AcceptanceInputKind::File,
                evidence.generated_file_mode(&relative)?,
                content.clone(),
            )
        } else {
            let captured = evidence.entry(&relative)?;
            (
                captured.kind.clone(),
                captured.mode,
                captured.payload.clone(),
            )
        };
        let payload_digest = sha256_hex(&payload);
        let owners = if let Some(signed) = signed {
            signed
                .entries
                .iter()
                .find(|entry| entry.path == relative)
                .map(|entry| entry.owners.clone())
                .unwrap_or_else(|| vec![EXACT_DELIVERY_OWNER.to_string()])
        } else {
            acceptance_input_owners(
                root,
                record,
                &relative,
                overrides,
                &evidence,
                unowned_source,
            )?
        };
        let entry_digest = acceptance_entry_digest(&relative, &kind, mode, &payload_digest);
        entries.push(AcceptanceInputEntryV1 {
            path: relative,
            kind,
            mode,
            payload_digest,
            entry_digest,
            owners,
        });
    }
    let manifest = AcceptanceManifestV1 {
        schema_version: 1,
        entries,
    };
    validate_acceptance_manifest(&manifest)?;
    Ok(manifest)
}

fn validate_portable_symlink_target(target: &str) -> Result<(), String> {
    if target.is_empty()
        || target.starts_with('/')
        || target.contains('\\')
        || target.chars().any(char::is_control)
        || target
            .as_bytes()
            .get(1)
            .is_some_and(|byte| *byte == b':' && target.as_bytes()[0].is_ascii_alphabetic())
    {
        return Err(format!(
            "symlink target is not portable project-relative UTF-8: `{}`",
            target.escape_default()
        ));
    }
    Ok(())
}

fn acceptance_kind_for_mode(mode: u32) -> AcceptanceInputKind {
    match mode {
        0o120000 => AcceptanceInputKind::Symlink,
        0o160000 => AcceptanceInputKind::Gitlink,
        _ => AcceptanceInputKind::File,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn acceptance_entry_digest(
    path: &str,
    kind: &AcceptanceInputKind,
    mode: u32,
    payload_digest: &str,
) -> String {
    let mut digest = FramedDigest::new(ACCEPTANCE_ENTRY_DOMAIN);
    digest.frame(b"path", path.as_bytes());
    digest.frame(b"kind", acceptance_kind_bytes(kind));
    digest.frame(b"mode", &mode.to_be_bytes());
    digest.frame(b"payload-digest", payload_digest.as_bytes());
    digest.finish()
}

fn acceptance_kind_bytes(kind: &AcceptanceInputKind) -> &'static [u8] {
    match kind {
        AcceptanceInputKind::File => b"file",
        AcceptanceInputKind::Symlink => b"symlink",
        AcceptanceInputKind::Gitlink => b"gitlink",
        AcceptanceInputKind::Missing => b"missing",
        AcceptanceInputKind::NonFile => b"non-file",
    }
}

fn acceptance_manifest_digest(manifest: &AcceptanceManifestV1) -> Result<String, String> {
    validate_acceptance_manifest(manifest)?;
    let mut digest = FramedDigest::new(ACCEPTANCE_MANIFEST_DOMAIN);
    digest.frame(b"schema-version", &manifest.schema_version.to_be_bytes());
    for entry in &manifest.entries {
        digest.frame(b"entry", b"");
        digest.frame(b"path", entry.path.as_bytes());
        digest.frame(b"kind", acceptance_kind_bytes(&entry.kind));
        digest.frame(b"mode", &entry.mode.to_be_bytes());
        digest.frame(b"payload-digest", entry.payload_digest.as_bytes());
        digest.frame(b"entry-digest", entry.entry_digest.as_bytes());
        for owner in &entry.owners {
            digest.frame(b"owner", owner.as_bytes());
        }
    }
    Ok(digest.finish())
}

fn validate_acceptance_manifest(manifest: &AcceptanceManifestV1) -> Result<(), String> {
    if manifest.schema_version != 1 {
        return Err(format!(
            "unsupported acceptance manifest schema {}",
            manifest.schema_version
        ));
    }
    if manifest.entries.len() > MAX_ACCEPTANCE_ENTRIES {
        return Err(format!(
            "acceptance manifest exceeds {MAX_ACCEPTANCE_ENTRIES} entries"
        ));
    }
    let mut previous: Option<&str> = None;
    for entry in &manifest.entries {
        if previous.is_some_and(|path| path >= entry.path.as_str()) {
            return Err("acceptance manifest paths must be strictly sorted and unique".into());
        }
        previous = Some(&entry.path);
        if entry.path.len() > MAX_ACCEPTANCE_PATH_BYTES
            || strict_portable_relative_path(&entry.path)? != entry.path
        {
            return Err(format!("invalid acceptance manifest path `{}`", entry.path));
        }
        let valid_mode = matches!(
            (&entry.kind, entry.mode),
            (AcceptanceInputKind::File, 0o100644 | 0o100755)
                | (AcceptanceInputKind::Symlink, 0o120000)
                | (AcceptanceInputKind::Gitlink, 0o160000)
                | (
                    AcceptanceInputKind::Missing | AcceptanceInputKind::NonFile,
                    0
                )
        );
        if !valid_mode {
            return Err(format!("invalid acceptance kind/mode for `{}`", entry.path));
        }
        validate_sha256_digest(&entry.payload_digest, "acceptance payload digest")?;
        validate_sha256_digest(&entry.entry_digest, "acceptance entry digest")?;
        if acceptance_entry_digest(&entry.path, &entry.kind, entry.mode, &entry.payload_digest)
            != entry.entry_digest
        {
            return Err(format!(
                "acceptance entry digest mismatch for `{}`",
                entry.path
            ));
        }
        if matches!(
            entry.kind,
            AcceptanceInputKind::Missing | AcceptanceInputKind::NonFile
        ) && entry.payload_digest != sha256_hex(b"")
        {
            return Err(format!(
                "acceptance {:?} entry `{}` must use the empty payload digest",
                entry.kind, entry.path
            ));
        }
        if entry.owners.is_empty() || entry.owners.len() > MAX_ACCEPTANCE_OWNERS {
            return Err(format!("invalid acceptance owners for `{}`", entry.path));
        }
        let mut previous_owner: Option<&str> = None;
        for owner in &entry.owners {
            if owner.len() > MAX_ACCEPTANCE_OWNER_BYTES
                || previous_owner.is_some_and(|previous| previous >= owner.as_str())
            {
                return Err(format!(
                    "acceptance owners for `{}` must be bounded, sorted, and unique",
                    entry.path
                ));
            }
            if owner.starts_with("@exact:")
                && owner != EXACT_TEST_OWNER
                && owner != EXACT_DELIVERY_OWNER
            {
                return Err(format!("unknown reserved acceptance owner `{owner}`"));
            }
            if !owner.starts_with("@exact:") {
                crate::commands::validate_module_name(owner)
                    .map_err(|error| format!("invalid acceptance owner: {error}"))?;
            }
            previous_owner = Some(owner);
        }
    }
    Ok(())
}

fn build_semantic_succession_evidence(
    root: &Path,
    record: &ChangeRecord,
    successor_manifest: &AcceptanceManifestV1,
) -> Result<SemanticSuccessionEvidenceV1, String> {
    validate_supersedes_semantics(root, record)?;
    if record.supersedes.is_empty() {
        return Ok(SemanticSuccessionEvidenceV1 {
            schema_version: 1,
            tuples: Vec::new(),
        });
    }
    let base = record.base_commit.as_deref().ok_or_else(|| {
        "semantic succession requires a definition-signed base commit".to_string()
    })?;
    let ancestor = Command::new("git")
        .args(["merge-base", "--is-ancestor", base, "HEAD"])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to validate succession base ancestry: {error}"))?;
    if !ancestor.success() {
        return Err(format!(
            "semantic succession base commit `{base}` is not an ancestor of HEAD"
        ));
    }
    let mut tuples = Vec::new();
    for edge in &record.supersedes {
        for obligation in &edge.obligations {
            let base_digest = acceptance_entry_digest_at_commit(
                root,
                base,
                &edge.predecessor_id,
                &obligation.path,
            )?;
            if base_digest != obligation.predecessor_entry_digest {
                return Err(format!(
                    "definition-signed base tree does not contain adopted predecessor entry `{}` `{}`",
                    edge.predecessor_id, obligation.path
                ));
            }
            let successor = successor_manifest
                .entries
                .iter()
                .find(|entry| entry.path == obligation.path)
                .ok_or_else(|| {
                    format!(
                        "successor acceptance manifest has no entry for `{}`",
                        obligation.path
                    )
                })?;
            if !successor.owners.contains(&obligation.module) {
                return Err(format!(
                    "successor acceptance entry `{}` is not owned by obligation module `{}`",
                    obligation.path, obligation.module
                ));
            }
            if successor.entry_digest == obligation.predecessor_entry_digest {
                return Err(format!(
                    "semantic succession obligation `{}` `{}` does not change the predecessor entry",
                    obligation.path, obligation.module
                ));
            }
            if !semantic_acceptance_item_exists_for_module(root, record, &obligation.module)? {
                return Err(format!(
                    "semantic succession module `{}` has no non-removed semantic delta",
                    obligation.module
                ));
            }
            tuples.push(SemanticSuccessionTupleV1 {
                predecessor_id: edge.predecessor_id.clone(),
                path: obligation.path.clone(),
                module: obligation.module.clone(),
                predecessor_entry_digest: obligation.predecessor_entry_digest.clone(),
                successor_entry_digest: successor.entry_digest.clone(),
            });
        }
    }
    // Any deterministic total order will do here — this feeds a digest, so only stability
    // matters, not chronology. Lexicographic keeps it consistent with every other ordering in
    // the succession path now that the ordinal is gone.
    tuples.sort_by(|left, right| {
        left.predecessor_id
            .cmp(&right.predecessor_id)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.module.cmp(&right.module))
    });
    let evidence = SemanticSuccessionEvidenceV1 {
        schema_version: 1,
        tuples,
    };
    validate_semantic_succession(record, &evidence)?;
    Ok(evidence)
}

fn validate_semantic_succession(
    record: &ChangeRecord,
    evidence: &SemanticSuccessionEvidenceV1,
) -> Result<(), String> {
    if evidence.schema_version != 1 {
        return Err(format!(
            "unsupported semantic succession schema {}",
            evidence.schema_version
        ));
    }
    if evidence.tuples.len() > MAX_ACCEPTANCE_ENTRIES {
        return Err(format!(
            "semantic succession exceeds {MAX_ACCEPTANCE_ENTRIES} tuples"
        ));
    }
    let expected: BTreeSet<_> = record
        .supersedes
        .iter()
        .flat_map(|edge| {
            edge.obligations.iter().map(|obligation| {
                (
                    edge.predecessor_id.clone(),
                    obligation.path.clone(),
                    obligation.module.clone(),
                    obligation.predecessor_entry_digest.clone(),
                )
            })
        })
        .collect();
    let mut actual = BTreeSet::new();
    let mut previous: Option<(&str, &str, &str)> = None;
    for tuple in &evidence.tuples {
        if tuple.predecessor_id.len() > 256
            || tuple.path.len() > MAX_ACCEPTANCE_PATH_BYTES
            || tuple.module.len() > MAX_ACCEPTANCE_OWNER_BYTES
        {
            return Err("semantic succession tuple exceeds a field limit".into());
        }
        if strict_portable_relative_path(&tuple.path)? != tuple.path {
            return Err(format!("invalid semantic succession path `{}`", tuple.path));
        }
        crate::commands::validate_module_name(&tuple.module)
            .map_err(|error| format!("invalid semantic succession module: {error}"))?;
        validate_sha256_digest(
            &tuple.predecessor_entry_digest,
            "semantic predecessor entry digest",
        )?;
        validate_sha256_digest(
            &tuple.successor_entry_digest,
            "semantic successor entry digest",
        )?;
        // Keyed on the ID alone. The ordinal was here to make `CHG-10000` follow `CHG-9999`,
        // which lexicographic order gets wrong — but a slug-only predecessor has no ordinal to
        // parse and this hard-errored on one. Every ID that ever reached this key is
        // `CHG-` plus exactly four digits, for which the two orders coincide, so no recorded
        // evidence changes. The digest below never framed the ordinal, so no digest changes.
        let key = (
            tuple.predecessor_id.as_str(),
            tuple.path.as_str(),
            tuple.module.as_str(),
        );
        if previous.is_some_and(|previous| previous >= key) {
            return Err("semantic succession tuples must be strictly sorted and unique".into());
        }
        previous = Some(key);
        let obligation = (
            tuple.predecessor_id.clone(),
            tuple.path.clone(),
            tuple.module.clone(),
            tuple.predecessor_entry_digest.clone(),
        );
        if !actual.insert(obligation) {
            return Err("semantic succession contains a duplicate obligation".into());
        }
    }
    if actual != expected {
        return Err(
            "semantic succession is not one-to-one with approved supersedes obligations".into(),
        );
    }
    Ok(())
}

fn semantic_succession_digest(evidence: &SemanticSuccessionEvidenceV1) -> Result<String, String> {
    let mut digest = FramedDigest::new(SEMANTIC_SUCCESSION_DOMAIN);
    digest.frame(b"schema-version", &evidence.schema_version.to_be_bytes());
    let mut previous: Option<(&str, &str, &str)> = None;
    for tuple in &evidence.tuples {
        // See `validate_semantic_succession`: the ordinal is not framed into the digest and
        // is not needed for the order, so a slug-only predecessor is orderable here.
        let key = (
            tuple.predecessor_id.as_str(),
            tuple.path.as_str(),
            tuple.module.as_str(),
        );
        if previous.is_some_and(|previous| previous >= key) {
            return Err("semantic succession tuples must be strictly sorted and unique".into());
        }
        previous = Some(key);
        digest.frame(b"tuple", b"");
        digest.frame(b"predecessor-id", tuple.predecessor_id.as_bytes());
        digest.frame(b"path", tuple.path.as_bytes());
        digest.frame(b"module", tuple.module.as_bytes());
        digest.frame(
            b"predecessor-entry-digest",
            tuple.predecessor_entry_digest.as_bytes(),
        );
        digest.frame(
            b"successor-entry-digest",
            tuple.successor_entry_digest.as_bytes(),
        );
    }
    Ok(digest.finish())
}

fn semantic_acceptance_item_exists_for_module(
    root: &Path,
    record: &ChangeRecord,
    module: &str,
) -> Result<bool, String> {
    let content =
        read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")?;
    Ok(parse_delta(&content)?.iter().any(|item| {
        matches!(
            item.target,
            DeltaTarget::Requirement | DeltaTarget::SpecSection
        ) && item.operation != DeltaOperation::Removed
            && !item.content.trim().is_empty()
    }))
}

fn acceptance_entry_digest_at_commit(
    root: &Path,
    commit: &str,
    predecessor_id: &str,
    path: &str,
) -> Result<String, String> {
    let temporary = create_effective_contract_workspace()?;
    let tree = temporary.join("tree");
    let added = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            tree.to_string_lossy().as_ref(),
            commit,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect succession base tree: {error}"))?;
    if !added.status.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!(
            "failed to inspect succession base tree: {}",
            String::from_utf8_lossy(&added.stderr).trim()
        ));
    }
    let result = (|| {
        let mut predecessor = load_change(&tree, predecessor_id)?;
        predecessor.state = ChangeState::Accepted;
        predecessor.affected_paths = vec![path.to_string()];
        let manifest = acceptance_manifest(&tree, &predecessor, &[])?;
        manifest
            .entries
            .iter()
            .find(|entry| entry.path == path)
            .map(|entry| entry.entry_digest.clone())
            .ok_or_else(|| {
                format!(
                    "succession base tree predecessor `{predecessor_id}` has no entry for `{path}`"
                )
            })
    })();
    let removed = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            tree.to_string_lossy().as_ref(),
        ])
        .current_dir(root)
        .output();
    let _ = fs::remove_dir_all(&temporary);
    if !removed.is_ok_and(|output| output.status.success()) {
        return Err("failed to remove succession base workspace".into());
    }
    result
}

fn semantic_tuple_transition_is_valid(
    root: &Path,
    successor: &ChangeRecord,
    tuple: &SemanticSuccessionTupleV1,
) -> Result<bool, String> {
    let Some(base) = successor.base_commit.as_deref() else {
        return Ok(false);
    };
    let base_old =
        acceptance_entry_digest_at_commit(root, base, &tuple.predecessor_id, &tuple.path)?;
    if base_old != tuple.predecessor_entry_digest {
        return Ok(false);
    }
    let (anchor, _, _) = match authenticated_accepted_transition(root, successor) {
        Ok(transition) => transition,
        Err(_) => return Ok(false),
    };
    let ancestor = Command::new("git")
        .args(["merge-base", "--is-ancestor", base, &anchor])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to validate accepted transition ancestry: {error}"))?;
    if !ancestor.success() {
        return Ok(false);
    }
    Ok(
        acceptance_entry_digest_at_commit(root, &anchor, &successor.id, &tuple.path)
            .is_ok_and(|digest| digest == tuple.successor_entry_digest),
    )
}

fn acceptance_input_owners(
    root: &Path,
    record: &ChangeRecord,
    relative: &str,
    overrides: &[(PathBuf, String)],
    evidence: &GitEvidence,
    unowned_source: UnownedProductionSource,
) -> Result<Vec<String>, String> {
    if path_is_governed_test_or_fixture(relative) {
        return Ok(vec![EXACT_TEST_OWNER.to_string()]);
    }
    if path_is_recognized_delivery_metadata(relative) {
        return Ok(vec![EXACT_DELIVERY_OWNER.to_string()]);
    }
    let config = crate::config::load_config(root);
    let override_content: BTreeMap<String, &str> = overrides
        .iter()
        .map(|(path, content)| Ok((strict_portable_project_path(root, path)?, content.as_str())))
        .collect::<Result<_, String>>()?;
    let mut owners = Vec::new();
    for module in &record.affected_specs {
        let (spec_path, requirements_path) =
            canonical_module_paths(root, &config.specs_dir, module)?;
        let spec_relative = strict_portable_project_path(root, &spec_path)?;
        let parent = spec_path
            .parent()
            .ok_or_else(|| format!("canonical spec path has no parent: {}", spec_path.display()))?;
        let canonical_owned = relative == spec_relative
            || relative == strict_portable_project_path(root, &requirements_path)?
            || CANONICAL_SPEC_COMPANIONS.iter().any(|name| {
                strict_portable_project_path(root, &parent.join(name))
                    .is_ok_and(|path| path == relative)
            });
        let source_owned =
            if canonical_owned {
                false
            } else {
                let content = if let Some(content) = override_content.get(&spec_relative) {
                    Some((*content).to_string())
                } else {
                    evidence
                        .entry(&spec_relative)
                        .ok()
                        .and_then(|entry| String::from_utf8(entry.payload.clone()).ok())
                };
                content
                    .as_deref()
                    .and_then(crate::parser::parse_frontmatter)
                    .is_some_and(|parsed| {
                        parsed.frontmatter.files.iter().any(|path| {
                            normalize_project_path(path).is_ok_and(|path| path == relative)
                        })
                    })
            };
        if canonical_owned || source_owned {
            owners.push(module.clone());
        }
    }
    owners.extend(
        record
            .acceptance_owner_corrections
            .iter()
            .filter(|correction| correction.path == relative)
            .map(|correction| correction.module.clone()),
    );
    if owners.is_empty() {
        if path_is_production_source(root, relative) {
            if unowned_source == UnownedProductionSource::AssignExactDelivery {
                // Legacy reconstruction only: adoption-era records predate canonical
                // ownership, so unowned production source routes to the exact delivery
                // owner instead of aborting the immutable archived ledger.
                owners.push(EXACT_DELIVERY_OWNER.to_string());
            } else {
                return Err(format!(
                    "acceptance input `{relative}` is production source without deterministic canonical ownership"
                ));
            }
        } else {
            owners.push(EXACT_DELIVERY_OWNER.to_string());
        }
    }
    owners.sort();
    owners.dedup();
    Ok(owners)
}

fn path_is_recognized_delivery_metadata(path: &str) -> bool {
    if path.starts_with(".github/") || path.starts_with(".specsync/") || path.starts_with("docs/") {
        return true;
    }
    is_protected_sdd_path(path)
        || (!path.contains('/')
            && matches!(
                path,
                "fledge.toml"
                    | ".trust.toml"
                    | "README.md"
                    | "LICENSE"
                    | "AGENTS.md"
                    | "Cargo.toml"
                    | "Cargo.lock"
                    | "package.json"
                    | "package-lock.json"
                    | "bun.lock"
                    | "bun.lockb"
                    | "pnpm-lock.yaml"
                    | "yarn.lock"
                    | "pyproject.toml"
                    | "uv.lock"
                    | "requirements.txt"
                    | "go.mod"
                    | "go.sum"
                    | "Package.swift"
                    | "Package.resolved"
                    | "action.yml"
                    | "action.yaml"
            ))
}

fn path_is_governed_test_or_fixture(path: &str) -> bool {
    path.starts_with("tests/")
        || path.starts_with("test/")
        || path.starts_with("Tests/")
        || path.contains("/tests/")
        || path.contains("/test/")
        || path.contains("/fixtures/")
        || path.contains("/__fixtures__/")
}

fn path_is_production_source(root: &Path, path: &str) -> bool {
    crate::exports::is_source_file(Path::new(path))
        && crate::config::load_config(root)
            .source_dirs
            .iter()
            .any(|source| {
                let normalized = source.trim_matches('/');
                normalized == "."
                    || path == normalized
                    || path.starts_with(&format!("{normalized}/"))
            })
        && !path_is_governed_test_or_fixture(path)
}

fn rooted_git_command(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.current_dir(root);
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_PREFIX",
        "GIT_CEILING_DIRECTORIES",
        "GIT_DISCOVERY_ACROSS_FILESYSTEM",
        "GIT_EXEC_PATH",
        "GIT_NAMESPACE",
        "GIT_SHALLOW_FILE",
        "GIT_GRAFT_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ] {
        command.env_remove(key);
    }
    command
}

fn configured_git_command(root: &Path) -> Command {
    let mut command = rooted_git_command(root);
    command.env_remove("GIT_CONFIG");
    command.env_remove("GIT_CONFIG_PARAMETERS");
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_CONFIG_KEY_")
            || key.to_string_lossy().starts_with("GIT_CONFIG_VALUE_")
        {
            command.env_remove(key);
        }
    }
    command.env_remove("GIT_CONFIG_COUNT");
    command.env("GIT_CONFIG_NOSYSTEM", "1");
    command.env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    );
    if let Some(index) = std::env::var_os("GIT_INDEX_FILE") {
        let index = PathBuf::from(index);
        let effective = if index.is_absolute() {
            index
        } else {
            root.join(index)
        };
        command.env("GIT_INDEX_FILE", effective);
    } else {
        command.env_remove("GIT_INDEX_FILE");
    }
    command
}

fn read_bounded_pipe<Reader: Read>(
    mut reader: Reader,
    cap: usize,
    overflow: Arc<AtomicBool>,
) -> std::io::Result<Vec<u8>> {
    let mut retained = Vec::with_capacity(cap.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = cap.saturating_sub(retained.len());
        let keep = remaining.min(read);
        retained.extend_from_slice(&buffer[..keep]);
        if keep != read {
            overflow.store(true, Ordering::Release);
        }
    }
    Ok(retained)
}

fn run_git_bounded(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<BoundedCommandOutput, String> {
    let mut command = configured_git_command(root);
    run_git_command_bounded(&mut command, args, input, stdout_cap)
}

fn run_git_command_bounded(
    command: &mut Command,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<BoundedCommandOutput, String> {
    run_git_command_bounded_with_deadline(command, args, input, stdout_cap, GIT_COMMAND_DEADLINE)
}

fn run_git_command_bounded_with_deadline(
    command: &mut Command,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
    deadline: Duration,
) -> Result<BoundedCommandOutput, String> {
    command
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    record_test_git_process();
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to run `git {}`: {error}", args.join(" ")))?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to capture Git stdout".to_string());
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            drop(stdout);
            let _ = child.kill();
            let _ = child.wait();
            return Err("failed to capture Git stderr".to_string());
        }
    };
    let overflow = Arc::new(AtomicBool::new(false));
    let stdout_overflow = Arc::clone(&overflow);
    let stderr_overflow = Arc::clone(&overflow);
    let stdout_reader =
        std::thread::spawn(move || read_bounded_pipe(stdout, stdout_cap, stdout_overflow));
    let stderr_reader = std::thread::spawn(move || {
        read_bounded_pipe(stderr, MAX_GIT_DIAGNOSTIC_BYTES, stderr_overflow)
    });
    let writer = input.map(|input| {
        let mut stdin = child.stdin.take();
        std::thread::spawn(move || -> std::io::Result<()> {
            if let Some(stdin) = stdin.as_mut() {
                stdin.write_all(&input)?;
            }
            Ok(())
        })
    });
    let started = Instant::now();
    let mut command_error = None;
    let status = loop {
        if overflow.load(Ordering::Acquire) {
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("failed to reap Git subprocess: {error}"));
        }
        if started.elapsed() >= deadline {
            command_error = Some(format!(
                "`git {}` exceeded its {:?} wall-clock deadline",
                args.join(" "),
                deadline
            ));
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("failed to reap timed-out Git subprocess: {error}"));
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => {}
            Err(error) => {
                command_error = Some(format!("failed to inspect Git subprocess: {error}"));
                let _ = child.kill();
                break child
                    .wait()
                    .map_err(|wait_error| format!("failed to reap Git subprocess: {wait_error}"));
            }
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "Git stdout reader panicked".to_string())
        .and_then(|result| result.map_err(|error| format!("failed to read Git stdout: {error}")));
    let stderr = stderr_reader
        .join()
        .map_err(|_| "Git stderr reader panicked".to_string())
        .and_then(|result| result.map_err(|error| format!("failed to read Git stderr: {error}")));
    let write_result = writer.map(|writer| {
        writer
            .join()
            .map_err(|_| "Git stdin writer panicked".to_string())
            .and_then(|result| {
                result.map_err(|error| format!("failed to write Git stdin: {error}"))
            })
    });
    let status = status?;
    let stdout = stdout?;
    let stderr = stderr?;
    if !overflow.load(Ordering::Acquire)
        && command_error.is_none()
        && let Some(write_result) = write_result
    {
        write_result?;
    }
    if overflow.load(Ordering::Acquire) {
        return Err(format!(
            "`git {}` output exceeds deterministic bounds",
            args.join(" ")
        ));
    }
    if let Some(error) = command_error {
        return Err(error);
    }
    Ok(BoundedCommandOutput {
        status,
        stdout,
        stderr,
    })
}

fn run_git_required(
    root: &Path,
    args: &[&str],
    input: Option<Vec<u8>>,
    stdout_cap: usize,
) -> Result<Vec<u8>, String> {
    let output = run_git_bounded(root, args, input, stdout_cap)?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// The four `core.*` keys the checkout overrides are derived from, read in one query.
///
/// These were four separate `git config --get` spawns. On a lifecycle-heavy run that is the
/// single largest source of subprocess cost: one suite run issued 15,359 `git config` spawns,
/// which this takes to 3,842. At ~15 ms per spawn that is the dominant term in every command
/// that inspects the checkout, not only in tests.
///
/// Deliberately NOT cached. Every call still spawns, so a configuration change between calls is
/// still observed — the saving comes from asking once for four answers, never from remembering
/// an answer.
///
/// `core.fsmonitor` is deliberately excluded. It is read through `configured_git_command`, which
/// scrubs system, global and injected configuration; folding it into this query — which is built
/// on `rooted_git_command`, and must be, to preserve the injected/global precedence the callers
/// rely on — would silently change how fsmonitor resolves.
const CHECKOUT_CORE_KEYS: [&str; 4] = [
    "core.autocrlf",
    "core.eol",
    "core.symlinks",
    "core.filemode",
];

/// Reads the four keys in a single `--get-regexp`, keeping the LAST value per key.
///
/// Equivalence to four `--get` calls was verified against git 2.50.1 rather than assumed:
/// a multi-valued key lists in order and `--get` returns the last, so last-wins matches;
/// a valueless key yields a record with no `\n`, i.e. the empty value `--get` reports at rc=0;
/// a mixed-case section (`[CORE] FileMode`) is emitted lowercased, as the callers already
/// expect; no matching key gives rc=1 with empty stdout and stderr, exactly as `--get` does for
/// an unset key; and a malformed config gives rc=128 with stderr on both, so the fail-loudly
/// rule is preserved rather than collapsed into "unset".
fn core_config_snapshot_from_command(
    command: &mut Command,
) -> Result<BTreeMap<String, String>, String> {
    command.env("GIT_PAGER", "cat");
    let output = run_git_command_bounded(
        command,
        &[
            "config",
            "-z",
            "--get-regexp",
            "^core[.](autocrlf|eol|symlinks|filemode)$",
        ],
        None,
        // Four `--get` calls each returned about six bytes, so the 128-byte bound they shared
        // was never near the limit. `--get-regexp` returns every occurrence of all four keys
        // across every scope, so the ordinary global-plus-local layout is already 144 bytes and
        // tripped the deterministic-bounds guard — turning a routine read into a hard error and
        // breaking every git-evidence capture on that machine. Bounded like the sibling
        // `core.fsmonitor` read, which faces the same "one query, unknown number of records"
        // shape.
        16 * 1024,
    )?;
    let mut snapshot = BTreeMap::new();
    if !output.status.success() {
        // No matching key is rc=1 with nothing on either stream — the same shape `--get`
        // reports for an unset key. Anything else (a malformed config is rc=128 with stderr)
        // must still fail loudly; reading it as "unset" would turn a broken repository into a
        // silently default one.
        if output.stdout.is_empty() && output.stderr.is_empty() {
            return Ok(snapshot);
        }
        return Err(format!(
            "failed to inspect effective Git core configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| "effective Git core configuration is not UTF-8".to_string())?;
    for record in text.split('\0').filter(|record| !record.is_empty()) {
        // `key\nvalue`, or a bare `key` when the key is present with no value.
        let (key, value) = match record.split_once('\n') {
            Some((key, value)) => (key, value),
            None => (record, ""),
        };
        // Later occurrences win, matching what `--get` returns for a multi-valued key.
        snapshot.insert(key.trim().to_string(), value.to_string());
    }
    Ok(snapshot)
}

fn normalize_checkout_core_value(key: &str, raw: &str) -> Result<String, String> {
    let value = raw.trim().to_ascii_lowercase();
    let normalized = match key {
        "core.autocrlf" => match value.as_str() {
            "" | "true" | "yes" | "on" | "1" => "true",
            "false" | "no" | "off" | "0" => "false",
            "input" => "input",
            _ => {
                return Err(format!(
                    "effective Git core.autocrlf has unsupported value `{value}`"
                ));
            }
        },
        "core.eol" => match value.as_str() {
            "lf" | "crlf" | "native" => return Ok(value),
            _ => {
                return Err(format!(
                    "effective Git {key} has unsupported value `{value}`"
                ));
            }
        },
        _ => match value.as_str() {
            "" | "true" | "yes" | "on" | "1" => "true",
            "false" | "no" | "off" | "0" => "false",
            _ => {
                return Err(format!(
                    "effective Git {key} has unsupported value `{value}`"
                ));
            }
        },
    };
    Ok(normalized.to_string())
}

/// Derives just `core.autocrlf` from the shared snapshot.
///
/// Production reads all four keys together through `effective_checkout_overrides_uncached`, so
/// this now exists for the tests that assert local/global/injected precedence on a `Command`
/// the caller builds. Keeping it means those tests exercise the same snapshot path production
/// uses, rather than a parallel one that could drift from it.
#[cfg(test)]
fn checkout_autocrlf_from_command(command: &mut Command) -> Result<Option<String>, String> {
    let snapshot = core_config_snapshot_from_command(command)?;
    let Some(raw) = snapshot.get("core.autocrlf") else {
        return Ok(None);
    };
    return normalize_checkout_core_value("core.autocrlf", raw).map(Some);
}

fn effective_checkout_overrides(root: &Path) -> Result<Vec<String>, String> {
    if let Some(overrides) = read_scope_value(root, |scope| scope.checkout_overrides.clone()) {
        return overrides;
    }
    let result = effective_checkout_overrides_uncached(root);
    update_read_scope(root, |scope| {
        scope.checkout_overrides = Some(result.clone());
    });
    result
}

fn effective_checkout_overrides_uncached(root: &Path) -> Result<Vec<String>, String> {
    // One spawn for four keys. This used to be four `git config --get` invocations, which
    // dominated the subprocess cost of every checkout inspection.
    let mut command = rooted_git_command(root);
    let snapshot = core_config_snapshot_from_command(&mut command)?;
    let mut overrides = Vec::new();
    for key in CHECKOUT_CORE_KEYS {
        // A key absent from the snapshot is unset, which is what rc=1 meant per-key before.
        let Some(raw) = snapshot.get(key) else {
            continue;
        };
        overrides.push(format!(
            "{key}={}",
            normalize_checkout_core_value(key, raw)?
        ));
    }
    Ok(overrides)
}

fn git_repository_present(root: &Path) -> Result<bool, String> {
    if let Some(present) = read_scope_value(root, |scope| scope.repository_present.clone()) {
        return present;
    }
    let result = git_repository_present_uncached(root);
    update_read_scope(root, |scope| {
        scope.repository_present = Some(result.clone());
    });
    result
}

fn git_repository_present_uncached(root: &Path) -> Result<bool, String> {
    let output = run_git_bounded(root, &["rev-parse", "--is-inside-work-tree"], None, 32)?;
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|_| "Git repository detection output is not UTF-8".to_string())?
        .trim();
    if output.status.success() {
        return match stdout {
            "true" => Ok(true),
            "false" => Err(
                "Git metadata was detected, but the governed root is not inside a work tree".into(),
            ),
            _ => Err(format!(
                "Git repository detection returned an invalid response `{stdout}`"
            )),
        };
    }
    let markers = git_metadata_markers_present(root)?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let ordinary_non_repository = stderr.to_ascii_lowercase().contains("not a git repository");
    if !markers && stdout.is_empty() && ordinary_non_repository {
        return Ok(false);
    }
    let diagnostic = stderr.trim();
    Err(if diagnostic.is_empty() {
        "Git repository detection failed without a diagnostic".into()
    } else {
        format!("Git repository detection failed: {diagnostic}")
    })
}

fn git_metadata_markers_present(root: &Path) -> Result<bool, String> {
    for ancestor in root.ancestors() {
        match fs::symlink_metadata(ancestor.join(".git")) {
            Ok(_) => return Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to inspect Git metadata marker at {}: {error}",
                    ancestor.display()
                ));
            }
        }
    }
    let head = root.join("HEAD");
    let objects = root.join("objects");
    Ok(fs::symlink_metadata(head).is_ok() && fs::symlink_metadata(objects).is_ok())
}

fn repository_context(root: &Path) -> Result<RepositoryContext, String> {
    if let Some(context) = read_scope_value(root, |scope| scope.repository_context.clone()) {
        return context;
    }
    let result = repository_context_uncached(root);
    update_read_scope(root, |scope| {
        scope.repository_context = Some(result.clone());
    });
    result
}

fn repository_context_uncached(root: &Path) -> Result<RepositoryContext, String> {
    if !git_repository_present(root)? {
        let canonical = root
            .canonicalize()
            .map_err(|error| format!("failed to resolve governed root identity: {error}"))?;
        return Ok(RepositoryContext {
            git: false,
            identity: canonical.to_string_lossy().into_owned(),
        });
    }
    let mut digest = FramedDigest::new(b"specsync.repository-context.v1");
    for (tag, args) in [
        (b"git-dir".as_slice(), ["rev-parse", "--absolute-git-dir"]),
        (b"common-dir".as_slice(), ["rev-parse", "--git-common-dir"]),
        (b"worktree".as_slice(), ["rev-parse", "--show-toplevel"]),
    ] {
        let output = run_git_required(root, &args, None, 16 * 1024)?;
        digest.frame(tag, &output);
    }
    Ok(RepositoryContext {
        git: true,
        identity: digest.finish(),
    })
}

fn git_evidence(root: &Path, candidates: &BTreeSet<String>) -> Result<GitEvidence, String> {
    cached_git_evidence(root, candidates, false)
}

fn git_regular_file_evidence(
    root: &Path,
    candidates: &BTreeSet<String>,
) -> Result<GitEvidence, String> {
    cached_git_evidence(root, candidates, true)
}

fn cached_git_evidence(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<GitEvidence, String> {
    let key = GitEvidenceCacheKey {
        regular_files_only,
        candidates: candidates.iter().cloned().collect(),
    };
    if let Some(evidence) = read_scope_value(root, |scope| scope.git_evidence.get(&key).cloned()) {
        return evidence;
    }
    let result = git_evidence_with_policy(root, candidates, regular_files_only, |_, _| {});
    update_read_scope(root, |scope| {
        if scope.git_evidence.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            scope.git_evidence.insert(key, result.clone());
        }
    });
    result
}

#[cfg(test)]
fn git_evidence_with_hook<Hook>(
    root: &Path,
    candidates: &BTreeSet<String>,
    after_inspection: Hook,
) -> Result<GitEvidence, String>
where
    Hook: FnMut(usize, &Path),
{
    git_evidence_with_policy(root, candidates, false, after_inspection)
}

fn git_evidence_with_policy<Hook>(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
    mut after_inspection: Hook,
) -> Result<GitEvidence, String>
where
    Hook: FnMut(usize, &Path),
{
    if candidates.len() > MAX_GIT_EVIDENCE_PATHS
        || candidates.iter().map(String::len).sum::<usize>() > MAX_GIT_EVIDENCE_PATH_BYTES
        || candidates.iter().any(|path| path.len() > 4096)
    {
        return Err("Git evidence candidate inventory exceeds deterministic bounds".into());
    }
    if !git_repository_present(root)? {
        let entries = capture_non_git_candidates(root, candidates, regular_files_only)?;
        return Ok(GitEvidence {
            modes: BTreeMap::new(),
            entries,
        });
    }
    for attempt in 0..2 {
        let before_index = git_index_fingerprint(root)?;
        let before = inspect_git_candidates(root, candidates, regular_files_only)?;
        after_inspection(attempt, root);
        let after = inspect_git_candidates(root, candidates, regular_files_only)?;
        let after_index = git_index_fingerprint(root)?;
        if before_index == after_index && before == after {
            return Ok(GitEvidence {
                modes: before.modes,
                entries: before.entries,
            });
        }
        if attempt == 1 {
            return Err(
                "Git index changed or candidate state changed during evidence inspection".into(),
            );
        }
    }
    unreachable!()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InspectedGitCandidates {
    modes: BTreeMap<String, u32>,
    objects: BTreeMap<String, String>,
    worktree: GitWorktreeState,
    entries: BTreeMap<String, GitCapturedEntry>,
}

fn git_index_fingerprint(root: &Path) -> Result<String, String> {
    let index = run_git_required(root, &["rev-parse", "--git-path", "index"], None, 16 * 1024)?;
    let index = parse_git_path(root, &index, "effective Git index")?;
    let shared = run_git_required(root, &["rev-parse", "--shared-index-path"], None, 16 * 1024)?;
    let mut paths = vec![index];
    if !shared.iter().all(u8::is_ascii_whitespace) {
        paths.push(parse_git_path(root, &shared, "split Git index")?);
    }
    paths.sort();
    paths.dedup();
    fingerprint_git_index_paths(paths)
}

fn fingerprint_git_index_paths(paths: Vec<PathBuf>) -> Result<String, String> {
    let mut digest = FramedDigest::new(b"specsync.git-index-generation.v1");
    let mut total = 0_usize;
    for path in paths {
        match fs::symlink_metadata(&path) {
            Ok(before) => {
                if !before.is_file() || before.file_type().is_symlink() {
                    return Err(format!(
                        "Git index dependency is not a regular file: {}",
                        path.display()
                    ));
                }
                let length = usize::try_from(before.len()).map_err(|_| {
                    "Git index dependencies exceed deterministic bounds".to_string()
                })?;
                total = total.checked_add(length).ok_or_else(|| {
                    "Git index dependencies exceed deterministic bounds".to_string()
                })?;
                if total > MAX_GIT_INDEX_BYTES {
                    return Err("Git index dependencies exceed deterministic bounds".into());
                }
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(&path)
                    .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
                let opened = file
                    .metadata()
                    .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
                if !same_file_metadata(&before, &opened) {
                    return Err(format!(
                        "Git index dependency changed before hashing: {}",
                        path.display()
                    ));
                }
                digest.frame(b"path", path.to_string_lossy().as_bytes());
                digest.frame_reader(b"bytes", before.len(), &mut file)?;
                let after = fs::symlink_metadata(&path)
                    .map_err(|error| format!("failed to re-inspect {}: {error}", path.display()))?;
                if !same_file_metadata(&before, &after) {
                    return Err(format!(
                        "Git index dependency changed during hashing: {}",
                        path.display()
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && total == 0 => {
                digest.frame(b"missing-index", path.to_string_lossy().as_bytes());
            }
            Err(error) => return Err(format!("failed to fingerprint {}: {error}", path.display())),
        }
    }
    Ok(digest.finish())
}

fn same_file_metadata(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    if left.len() != right.len()
        || left.file_type() != right.file_type()
        || left.modified().ok() != right.modified().ok()
    {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        left.dev() == right.dev() && left.ino() == right.ino()
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn parse_git_path(root: &Path, output: &[u8], description: &str) -> Result<PathBuf, String> {
    let value = std::str::from_utf8(output)
        .map_err(|_| format!("{description} path is not UTF-8"))?
        .trim();
    if value.is_empty() || value.as_bytes().contains(&0) {
        return Err(format!("{description} path is invalid"));
    }
    let path = PathBuf::from(value);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

fn candidate_argument_batches(candidates: &BTreeSet<String>) -> Vec<Vec<&str>> {
    let mut batches = Vec::new();
    let mut batch = Vec::new();
    let mut bytes = 0_usize;
    for candidate in candidates {
        if !batch.is_empty()
            && (batch.len() >= GIT_ATTRIBUTE_BATCH_PATHS
                || bytes.saturating_add(candidate.len()) > 64 * 1024)
        {
            batches.push(batch);
            batch = Vec::new();
            bytes = 0;
        }
        batch.push(candidate.as_str());
        bytes = bytes.saturating_add(candidate.len() + 1);
    }
    if !batch.is_empty() {
        batches.push(batch);
    }
    batches
}

fn literal_candidate_git_args(repo_prefix: &str, prefix: &[&str], batch: &[&str]) -> Vec<String> {
    let mut args = Vec::with_capacity(prefix.len() + batch.len() + 1);
    args.extend(prefix.iter().map(|value| (*value).to_string()));
    args.push("--".into());
    for path in batch {
        args.push(format!(":(top,literal){repo_prefix}{path}"));
    }
    args
}

fn borrowed_git_args(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

fn record_git_stage_zero_entry(
    entries: &mut BTreeMap<String, (u32, String)>,
    path: String,
    mode: u32,
    object: String,
) -> Result<(), String> {
    if let Some((existing_mode, existing_object)) = entries.get(&path) {
        if *existing_mode == mode && existing_object == &object {
            return Ok(());
        }
        return Err(format!(
            "conflicting duplicate Git index stage-zero entry for `{path}`"
        ));
    }
    entries.insert(path, (mode, object));
    Ok(())
}

/// Is `path` inside the requested candidate scope?
///
/// A candidate naming a directory expands under `:(top,literal)` to every tracked
/// file beneath it, so those files were requested just as much as an exact
/// candidate was. Rejecting them made `finalize` fail in any repository that had
/// ever archived a change — every project past its first — while passing on the
/// fresh fixtures the suite is built on.
///
/// Compares at the path separator so `a/b` cannot be admitted by `a/bc`.
fn candidate_scope_admits(candidates: &BTreeSet<String>, path: &str) -> bool {
    candidates.contains(path)
        || candidates.iter().any(|candidate| {
            path.len() > candidate.len()
                && path.as_bytes()[candidate.len()] == b'/'
                && path.starts_with(candidate.as_str())
        })
}

fn inspect_git_candidates(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<InspectedGitCandidates, String> {
    let mut stage_entries = BTreeMap::new();
    let batches = candidate_argument_batches(candidates);
    let repo_prefix = git_repo_prefix(root)?;
    let mut stage_output_bytes = 0_usize;
    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "--stage", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_INDEX_BYTES)?;
        stage_output_bytes = stage_output_bytes
            .checked_add(output.len())
            .ok_or_else(|| "scoped Git index output exceeds deterministic bounds".to_string())?;
        if stage_output_bytes > MAX_GIT_INDEX_BYTES {
            return Err("scoped Git index output exceeds deterministic bounds".into());
        }
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            let tab = record
                .iter()
                .position(|byte| *byte == b'\t')
                .ok_or_else(|| "invalid `git ls-files --stage` record".to_string())?;
            let metadata = std::str::from_utf8(&record[..tab])
                .map_err(|_| "non-UTF-8 Git index metadata".to_string())?;
            let mut fields = metadata.split_whitespace();
            let mode_text = fields
                .next()
                .ok_or_else(|| "Git index entry is missing a mode".to_string())?;
            let object = fields
                .next()
                .ok_or_else(|| "Git index entry is missing an object ID".to_string())?;
            let stage = fields
                .next()
                .ok_or_else(|| "Git index entry is missing a stage".to_string())?;
            if fields.next().is_some()
                || object.is_empty()
                || !object.bytes().all(|byte| byte.is_ascii_hexdigit())
                || !matches!(stage, "0" | "1" | "2" | "3")
            {
                return Err("invalid Git index metadata".into());
            }
            let mode = u32::from_str_radix(mode_text, 8)
                .map_err(|_| format!("invalid Git file mode `{mode_text}`"))?;
            let path = std::str::from_utf8(&record[tab + 1..])
                .map_err(|_| "non-UTF-8 scoped Git index path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!("Git returned an out-of-scope index path `{path}`"));
            }
            if stage != "0" {
                return Err(format!(
                    "cannot hash relevant path `{path}` with unresolved Git index stages"
                ));
            }
            let object = object.to_ascii_lowercase();
            record_git_stage_zero_entry(&mut stage_entries, path, mode, object)?;
        }
    }
    let mut modes = BTreeMap::new();
    let mut objects = BTreeMap::new();
    for (path, (mode, object)) in stage_entries {
        modes.insert(path.clone(), mode);
        objects.insert(path, object);
    }

    let fsmonitor = run_git_bounded(
        root,
        &["config", "--get", "core.fsmonitor"],
        None,
        16 * 1024,
    )?;
    let fsmonitor_value = std::str::from_utf8(&fsmonitor.stdout)
        .map_err(|_| "Git core.fsmonitor value is not UTF-8".to_string())?
        .trim();
    if !fsmonitor.status.success() && (!fsmonitor.stdout.is_empty() || !fsmonitor.stderr.is_empty())
    {
        return Err(format!(
            "failed to inspect Git fsmonitor configuration: {}",
            String::from_utf8_lossy(&fsmonitor.stderr).trim()
        ));
    }
    let fsmonitor_active = !fsmonitor_value.is_empty()
        && !matches!(
            fsmonitor_value.to_ascii_lowercase().as_str(),
            "false" | "no" | "off" | "0"
        );
    if fsmonitor_active
        && candidates
            .iter()
            .any(|path| fs::symlink_metadata(root.join(path)).is_ok())
    {
        return Err("Git core.fsmonitor cannot supply canonical working-tree evidence".into());
    }

    let checkout_overrides = effective_checkout_overrides(root)?;
    let mut modified = BTreeSet::new();
    for batch in &batches {
        let mut prefix = Vec::new();
        for setting in &checkout_overrides {
            prefix.push("-c".to_string());
            prefix.push(setting.clone());
        }
        prefix.extend(
            ["diff-files", "--name-only", "-z"]
                .iter()
                .map(|value| (*value).to_string()),
        );
        let prefix = borrowed_git_args(&prefix);
        let args = literal_candidate_git_args(&repo_prefix, &prefix, batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for path in output
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
        {
            let path =
                std::str::from_utf8(path).map_err(|_| "non-UTF-8 Git diff path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope modified path `{path}`"
                ));
            }
            modified.insert(path);
        }
    }

    let mut sparse_absent = BTreeSet::new();
    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "-v", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            if record.len() < 3 || record[1] != b' ' {
                return Err("invalid `git ls-files -v` output".into());
            }
            let tag = record[0];
            let path = std::str::from_utf8(&record[2..])
                .map_err(|_| "non-UTF-8 Git visibility path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope visibility path `{path}`"
                ));
            }
            if tag.is_ascii_lowercase() {
                return Err(format!(
                    "Git assume-unchanged hides working-tree evidence for `{path}`"
                ));
            }
            if tag == b'S' {
                match fs::symlink_metadata(root.join(&path)) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        sparse_absent.insert(path);
                    }
                    Ok(_) => {
                        return Err(format!(
                            "materialized skip-worktree path cannot use canonical index evidence: `{path}`"
                        ));
                    }
                    Err(error) => {
                        return Err(format!("failed to inspect sparse path `{path}`: {error}"));
                    }
                }
            }
        }
    }

    for batch in &batches {
        let args = literal_candidate_git_args(&repo_prefix, &["ls-files", "-f", "-z"], batch);
        let args = borrowed_git_args(&args);
        let output = run_git_required(root, &args, None, MAX_GIT_COMMAND_OUTPUT_BYTES)?;
        for record in output
            .split(|byte| *byte == 0)
            .filter(|entry| !entry.is_empty())
        {
            if record.len() < 3 || record[1] != b' ' {
                return Err("invalid `git ls-files -f` output".into());
            }
            let path = std::str::from_utf8(&record[2..])
                .map_err(|_| "non-UTF-8 Git fsmonitor path".to_string())?;
            let path = strict_portable_relative_path(path)?;
            if !candidate_scope_admits(candidates, &path) {
                return Err(format!(
                    "Git returned an out-of-scope fsmonitor path `{path}`"
                ));
            }
            if record[0].is_ascii_lowercase() && fs::symlink_metadata(root.join(&path)).is_ok() {
                return Err(format!(
                    "Git fsmonitor-valid hides working-tree evidence for `{path}`"
                ));
            }
        }
    }

    let regular_for_attributes = candidates
        .iter()
        .filter(|path| {
            matches!(modes.get(*path), Some(0o100644 | 0o100755))
                && !modified.contains(*path)
                && !sparse_absent.contains(*path)
                && fs::symlink_metadata(root.join(*path)).is_ok_and(|metadata| metadata.is_file())
        })
        .cloned()
        .collect();
    validate_canonical_git_attributes(root, &regular_for_attributes)?;

    let worktree = GitWorktreeState {
        modified,
        sparse_absent,
    };
    let clean_objects: BTreeSet<_> = candidates
        .iter()
        .filter_map(|path| {
            let mode = modes.get(path)?;
            let object = objects.get(path)?;
            (!worktree.modified.contains(path) && matches!(mode, 0o100644 | 0o100755 | 0o120000))
                .then_some(object.as_str())
        })
        .collect();
    let clean_object_refs: Vec<_> = clean_objects.iter().copied().collect();
    let clean_payloads = git_blob_bytes_batch(root, &clean_object_refs)?;
    let prefetched_blobs: BTreeMap<_, _> = clean_objects
        .into_iter()
        .map(str::to_string)
        .zip(clean_payloads)
        .collect();
    let mut entries = BTreeMap::new();
    let mut payload_bytes = 0_usize;
    for path in candidates {
        if regular_files_only {
            let working = fs::symlink_metadata(root.join(path));
            let non_regular = matches!(modes.get(path), Some(0o120000 | 0o160000))
                || working.as_ref().is_ok_and(|metadata| !metadata.is_file());
            if non_regular {
                return Err(format!(
                    "selected definition artifact is not a regular file: {path}"
                ));
            }
        }
        let entry = capture_git_candidate(
            root,
            path,
            modes.get(path).copied(),
            objects.get(path),
            &worktree,
            Some(&prefetched_blobs),
        )?;
        payload_bytes = payload_bytes
            .checked_add(entry.payload.len())
            .ok_or_else(|| "Git evidence payload exceeds deterministic bounds".to_string())?;
        if payload_bytes > MAX_GIT_EVIDENCE_PAYLOAD_BYTES {
            return Err("Git evidence payload exceeds deterministic bounds".into());
        }
        entries.insert(path.clone(), entry);
    }
    Ok(InspectedGitCandidates {
        modes,
        objects,
        worktree,
        entries,
    })
}

fn capture_git_candidate(
    root: &Path,
    relative: &str,
    index_mode: Option<u32>,
    object: Option<&String>,
    worktree: &GitWorktreeState,
    prefetched_blobs: Option<&BTreeMap<String, Vec<u8>>>,
) -> Result<GitCapturedEntry, String> {
    if index_mode == Some(0o160000) {
        let object = object
            .ok_or_else(|| format!("gitlink `{relative}` has no exact index object ID"))?
            .clone();
        return Ok(GitCapturedEntry {
            kind: AcceptanceInputKind::Gitlink,
            mode: 0o160000,
            payload: object.as_bytes().to_vec(),
            object: Some(object),
        });
    }
    let clean = object.is_some() && !worktree.modified.contains(relative);
    if clean {
        let mode = index_mode.ok_or_else(|| format!("tracked path `{relative}` has no mode"))?;
        if worktree.sparse_absent.contains(relative)
            || matches!(mode, 0o100644 | 0o100755 | 0o120000)
        {
            let object = object.expect("clean tracked object").clone();
            let payload = match prefetched_blobs.and_then(|blobs| blobs.get(&object)) {
                Some(payload) => payload.clone(),
                None => git_blob_bytes(root, &object)?,
            };
            if mode == 0o120000 {
                let target = std::str::from_utf8(&payload)
                    .map_err(|_| format!("symlink target is not UTF-8: `{relative}`"))?;
                validate_portable_symlink_target(target)?;
            }
            return Ok(GitCapturedEntry {
                kind: acceptance_kind_for_mode(mode),
                mode,
                object: Some(object),
                payload,
            });
        }
        return Err(format!(
            "unsupported Git index mode `{mode:o}` for `{relative}`"
        ));
    }
    capture_working_candidate(root, relative, object.cloned())
}

fn capture_working_candidate(
    root: &Path,
    relative: &str,
    object: Option<String>,
) -> Result<GitCapturedEntry, String> {
    let path = root.join(relative);
    let (kind, mode, payload) = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let target = fs::read_link(&path)
                .map_err(|error| format!("failed to read symlink {}: {error}", path.display()))?;
            let target = target.to_str().ok_or_else(|| {
                format!(
                    "non-UTF-8 symlink target cannot be hashed portably: {}",
                    path.display()
                )
            })?;
            validate_portable_symlink_target(target)?;
            (
                AcceptanceInputKind::Symlink,
                0o120000,
                target.as_bytes().to_vec(),
            )
        }
        Ok(metadata) if metadata.is_file() => (
            AcceptanceInputKind::File,
            working_file_mode(&metadata),
            fs::read(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?,
        ),
        Ok(_) => (AcceptanceInputKind::NonFile, 0, Vec::new()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidFilename
            ) =>
        {
            (AcceptanceInputKind::Missing, 0, Vec::new())
        }
        Err(error) => return Err(format!("failed to inspect {}: {error}", path.display())),
    };
    Ok(GitCapturedEntry {
        kind,
        mode,
        object,
        payload,
    })
}

fn capture_non_git_candidates(
    root: &Path,
    candidates: &BTreeSet<String>,
    regular_files_only: bool,
) -> Result<BTreeMap<String, GitCapturedEntry>, String> {
    let mut entries = BTreeMap::new();
    let mut payload_bytes = 0_usize;
    for path in candidates {
        if regular_files_only
            && fs::symlink_metadata(root.join(path)).is_ok_and(|metadata| !metadata.is_file())
        {
            return Err(format!(
                "selected definition artifact is not a regular file: {path}"
            ));
        }
        let entry = capture_working_candidate(root, path, None)?;
        payload_bytes = payload_bytes
            .checked_add(entry.payload.len())
            .ok_or_else(|| {
                "filesystem evidence payload exceeds deterministic bounds".to_string()
            })?;
        if payload_bytes > MAX_GIT_EVIDENCE_PAYLOAD_BYTES {
            return Err("filesystem evidence payload exceeds deterministic bounds".into());
        }
        entries.insert(path.clone(), entry);
    }
    Ok(entries)
}

#[cfg(test)]
fn git_worktree_state(root: &Path) -> Result<Option<GitWorktreeState>, String> {
    let candidates = git_project_paths(root)?.unwrap_or_default();
    let candidates = candidates.into_iter().collect::<BTreeSet<_>>();
    if !git_repository_present(root)? {
        return Ok(None);
    }
    Ok(Some(
        inspect_git_candidates(root, &candidates, false)?.worktree,
    ))
}

fn validate_canonical_git_attributes(root: &Path, paths: &BTreeSet<String>) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let paths = paths.iter().collect::<Vec<_>>();
    for batch in paths.chunks(GIT_ATTRIBUTE_BATCH_PATHS) {
        let mut input = Vec::new();
        for path in batch {
            input.extend_from_slice(path.as_bytes());
            input.push(0);
        }
        let output = run_git_required(
            root,
            &[
                "check-attr",
                "-z",
                "--stdin",
                "filter",
                "working-tree-encoding",
                "ident",
            ],
            Some(input),
            MAX_GIT_ATTRIBUTE_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect Git content attributes: {error}"))?;
        validate_git_attribute_output(batch, &output)?;
    }
    Ok(())
}

fn validate_git_attribute_output(paths: &[&String], output: &[u8]) -> Result<(), String> {
    if !output.ends_with(&[0]) {
        return Err("invalid unterminated NUL-delimited `git check-attr` output".into());
    }
    let fields = output[..output.len() - 1]
        .split(|byte| *byte == 0)
        .collect::<Vec<_>>();
    if fields.iter().any(|field| field.is_empty()) || fields.len() % 3 != 0 {
        return Err("invalid NUL-delimited `git check-attr` output".into());
    }
    let attributes = ["filter", "working-tree-encoding", "ident"];
    let expected = paths
        .iter()
        .flat_map(|path| {
            attributes
                .iter()
                .map(move |attribute| (path.as_str(), *attribute))
        })
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    for record in fields.chunks_exact(3) {
        let path = record[0];
        let attribute = record[1];
        let value = record[2];
        let path =
            std::str::from_utf8(path).map_err(|_| "non-UTF-8 Git attribute path".to_string())?;
        let attribute = std::str::from_utf8(attribute)
            .map_err(|_| "non-UTF-8 Git attribute name".to_string())?;
        let value =
            std::str::from_utf8(value).map_err(|_| "non-UTF-8 Git attribute value".to_string())?;
        if !expected.contains(&(path, attribute)) {
            return Err(format!(
                "Git returned an unrequested attribute pair `{path}` / `{attribute}`"
            ));
        }
        if !seen.insert((path, attribute)) {
            return Err(format!(
                "Git returned a duplicate attribute pair `{path}` / `{attribute}`"
            ));
        }
        if value != "unspecified" && value != "unset" {
            return Err(format!(
                "Git `{attribute}` attribute is not supported for canonical evidence: `{path}`"
            ));
        }
    }
    if seen != expected {
        return Err("Git attribute output is missing a requested path/attribute pair".into());
    }
    Ok(())
}

fn git_blob_bytes(root: &Path, object: &str) -> Result<Vec<u8>, String> {
    if let Some(bytes) = GIT_BLOB_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .map_err(|_| "Git blob cache lock is poisoned".to_string())?
        .get(object)
        .cloned()
    {
        return Ok(bytes);
    }
    let stdout = run_git_required(
        root,
        &["cat-file", "blob", object],
        None,
        MAX_GIT_EVIDENCE_PAYLOAD_BYTES,
    )?;
    if stdout.len() <= MAX_CHANGE_ARTIFACT_BYTES as usize {
        let mut cache = GIT_BLOB_CACHE
            .get_or_init(|| Mutex::new(BTreeMap::new()))
            .lock()
            .map_err(|_| "Git blob cache lock is poisoned".to_string())?;
        if cache.len() >= 4_096 {
            cache.clear();
        }
        cache.insert(object.to_string(), stdout.clone());
    }
    Ok(stdout)
}

fn working_file_mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            0o100644
        } else {
            0o100755
        }
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        0o100644
    }
}

fn resolved_acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
) -> Result<AcceptanceManifestV1, String> {
    let verification = load_verification(root, record)?;
    let signed = verification
        .acceptance_input_digest
        .clone()
        .ok_or_else(|| {
            format!(
                "accepted change `{}` has no acceptance input digest",
                record.id
            )
        })?;
    if let Some(manifest) = &verification.acceptance_manifest {
        if !verification.passed {
            return Err(format!(
                "accepted change `{}` has failed verification",
                record.id
            ));
        }
        ensure_definition_approval_valid(root, record)?;
        if !definition_digest_matches(root, record, &verification.contract_digest)? {
            return Err(format!(
                "accepted change `{}` verification contract is stale",
                record.id
            ));
        }
        validate_verification_execution_digest(root, record, &verification)?;
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != signed {
            return Err(format!(
                "accepted change `{}` manifest does not reproduce its signed aggregate",
                record.id
            ));
        }
        if let Some(evidence) = &verification.semantic_succession {
            validate_semantic_succession(record, evidence)?;
        } else if !record.supersedes.is_empty() {
            return Err(format!(
                "accepted change `{}` is missing semantic succession evidence",
                record.id
            ));
        }
        if record.workflow_version >= 2 {
            validate_finalization_evidence(root, record, &verification)?;
        } else {
            let ledger = load_approvals(root, record)?;
            let approval = latest_terminal_approval(&ledger).ok_or_else(|| {
                format!("accepted change `{}` has no closing approval", record.id)
            })?;
            if approval.digest != closing_digest(record, &verification) {
                return Err(format!(
                    "accepted change `{}` closing approval does not authenticate its manifest",
                    record.id
                ));
            }
        }
        return Ok(manifest.clone());
    }
    reconstruct_legacy_acceptance_manifest(root, record, &signed)
}

fn reconstruct_legacy_acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
    signed_legacy_digest: &str,
) -> Result<AcceptanceManifestV1, String> {
    let anchors = accepted_transition_anchors(root, record)?;
    let mut reconstructions: BTreeMap<Vec<u8>, AcceptanceManifestV1> = BTreeMap::new();
    let mut first_failure = None;
    for anchor in anchors {
        match reconstruct_legacy_at_anchor(root, record, signed_legacy_digest, &anchor) {
            Ok((key, manifest)) => {
                reconstructions.entry(key).or_insert(manifest);
            }
            Err(error) => {
                first_failure.get_or_insert(error);
            }
        }
    }
    if reconstructions.len() != 1 {
        let mut error = format!(
            "legacy accepted change `{}` requires exactly one distinct valid historical reconstruction, found {}",
            record.id,
            reconstructions.len()
        );
        if let Some(first_failure) = first_failure {
            error.push_str(&format!("; first reconstruction failure: {first_failure}"));
        }
        return Err(error);
    }
    reconstructions
        .into_values()
        .next()
        .ok_or_else(|| "legacy reconstruction disappeared unexpectedly".to_string())
}

fn reconstruct_legacy_at_anchor(
    root: &Path,
    record: &ChangeRecord,
    signed_legacy_digest: &str,
    anchor: &str,
) -> Result<(Vec<u8>, AcceptanceManifestV1), String> {
    let temporary = create_effective_contract_workspace()?;
    let tree = temporary.join("tree");
    let added = Command::new("git")
        .args([
            "worktree",
            "add",
            "--detach",
            tree.to_string_lossy().as_ref(),
            anchor,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to reconstruct legacy acceptance tree: {error}"))?;
    if !added.status.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!(
            "failed to reconstruct legacy acceptance tree: {}",
            String::from_utf8_lossy(&added.stderr).trim()
        ));
    }
    let result = (|| {
        let historical = load_change(&tree, &record.id)?;
        if historical.state != ChangeState::Accepted {
            return Err(format!(
                "trusted transition `{anchor}` does not contain accepted state for `{}`",
                record.id
            ));
        }
        ensure_definition_approval_valid(&tree, &historical)?;
        let verification = load_verification(&tree, &historical)?;
        if !verification.passed {
            return Err("historical accepted verification did not pass".into());
        }
        let ledger = load_approvals(&tree, &historical)?;
        let closing = latest_terminal_approval(&ledger)
            .ok_or_else(|| "historical accepted state has no closing approval".to_string())?;
        if closing.digest != closing_digest(&historical, &verification) {
            return Err("historical accepted closing approval is invalid".into());
        }
        let aggregate = acceptance_input_digest(&tree, &historical, &[])?;
        if aggregate != signed_legacy_digest {
            return Err(format!(
                "legacy accepted change `{}` cannot reproduce its signed raw-content aggregate",
                record.id
            ));
        }
        let manifest = acceptance_manifest_legacy(&tree, &historical, &[])?;
        let key = serde_json::to_vec(&serde_json::json!({
            "manifest": manifest,
            "verification": verification,
            "closing": closing,
        }))
        .map_err(|error| format!("failed to canonicalize historical evidence: {error}"))?;
        Ok((key, manifest))
    })();
    // Best-effort cleanup of the disposable scratch worktree. A successful
    // reconstruction must never be discarded because hygiene cleanup failed
    // (product #511 / flaky CI under worktree contention).
    let force_remove_failure = {
        #[cfg(test)]
        {
            FORCE_LEGACY_WORKTREE_REMOVE_FAILURE.get()
        }
        #[cfg(not(test))]
        {
            false
        }
    };
    let removed = if force_remove_failure {
        None
    } else {
        Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                tree.to_string_lossy().as_ref(),
            ])
            .current_dir(root)
            .output()
            .ok()
    };
    if !removed.is_some_and(|output| output.status.success()) {
        // Reclaim the registration if remove failed; ignore further hygiene errors.
        let _ = Command::new("git")
            .args(["worktree", "prune"])
            .current_dir(root)
            .output();
    }
    let _ = fs::remove_dir_all(&temporary);
    result
}

fn accepted_transition_anchors(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let state = git_repo_relative_path(root, &state)?;
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let mut anchors = Vec::new();
    for reference in references {
        let state_pathspec = format!(":(top,literal){state}");
        let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
        let output = run_git_bounded(
            root,
            &[
                "log",
                "--format=%H",
                &max_count,
                &reference,
                "--",
                state_pathspec.as_str(),
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect accepted history: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect accepted history at `{reference}`"
            ));
        }
        let history = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<&str> = history
            .lines()
            .filter(|commit| !commit.is_empty())
            .collect();
        if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(format!(
                "accepted history exceeds the deterministic {}-commit bound",
                MAX_TRUSTED_HISTORY_COMMITS
            ));
        }
        for commit in commits {
            let current = git_change_record_at(root, commit, &state);
            if !current.is_some_and(|current| {
                current.id == record.id && current.state == ChangeState::Accepted
            }) {
                continue;
            }
            let parents =
                git_output(root, &["rev-list", "--parents", "-n", "1", commit]).unwrap_or_default();
            let parent_accepted = parents.split_whitespace().skip(1).any(|parent| {
                git_change_record_at(root, parent, &state).is_some_and(|parent| {
                    parent.id == record.id && parent.state == ChangeState::Accepted
                })
            });
            if !parent_accepted {
                anchors.push(commit.to_string());
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    Ok(anchors)
}

/// Lists every commit reachable from `HEAD` or the remote default whose `state.json` records
/// `record` as accepted, regardless of the parent state. Unlike
/// [`accepted_transition_anchors`], this also matches commits that refresh accepted evidence
/// while the change is already accepted, which is the shape squash-merged re-verification
/// produces on the default branch.
fn accepted_recording_anchors(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let state = git_repo_relative_path(root, &state)?;
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let mut anchors = Vec::new();
    for reference in references {
        let state_pathspec = format!(":(top,literal){state}");
        let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
        let output = run_git_bounded(
            root,
            &[
                "log",
                "--format=%H",
                &max_count,
                &reference,
                "--",
                state_pathspec.as_str(),
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| format!("failed to inspect accepted recording history: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect accepted recording history at `{reference}`"
            ));
        }
        let history = String::from_utf8_lossy(&output.stdout);
        let commits: Vec<&str> = history
            .lines()
            .filter(|commit| !commit.is_empty())
            .collect();
        if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(format!(
                "accepted recording history exceeds the deterministic {}-commit bound",
                MAX_TRUSTED_HISTORY_COMMITS
            ));
        }
        for commit in commits {
            if git_change_record_at(root, commit, &state).is_some_and(|current| {
                current.id == record.id && current.state == ChangeState::Accepted
            }) {
                anchors.push(commit.to_string());
            }
        }
    }
    anchors.sort();
    anchors.dedup();
    Ok(anchors)
}

/// Evaluates one candidate anchor commit against the current evidence, recording it in
/// `eligible` when the committed state, verification, and approvals authenticate the change's
/// accepted record.
#[allow(clippy::too_many_arguments)]
fn consider_accepted_evidence_anchor(
    root: &Path,
    record: &ChangeRecord,
    anchor: &str,
    state_path: &str,
    verification_path: &str,
    approvals_path: &str,
    current_verification: &[u8],
    current_approvals: &[u8],
    eligible: &mut BTreeMap<String, (String, Vec<u8>, ChangeRecord)>,
) {
    let Some(state_bytes) = git_object_bytes(root, anchor, state_path) else {
        return;
    };
    let Some(verification_bytes) = git_object_bytes(root, anchor, verification_path) else {
        return;
    };
    let Some(approval_bytes) = git_object_bytes(root, anchor, approvals_path) else {
        return;
    };
    if verification_bytes != current_verification || approval_bytes != current_approvals {
        return;
    }
    let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
        return;
    };
    if accepted.id != record.id || accepted.state != ChangeState::Accepted {
        return;
    }
    let mut projection = record.clone();
    if record.state == ChangeState::Archived {
        projection.state = ChangeState::Accepted;
        projection.updated_at = accepted.updated_at;
    }
    if projection == accepted {
        let key = accepted_evidence_key(&state_bytes, &verification_bytes, &approval_bytes);
        eligible
            .entry(key)
            .or_insert((anchor.to_string(), state_bytes, accepted));
    }
}

/// One reachable commit that introduced a change's archived package into history, together with
/// the repository-relative package directory that commit created and the approval ledger the
/// package carried there.
///
/// The ledger is carried whole rather than reduced to its reopen count. The count is a number the
/// attacker writes next to the evidence it is supposed to qualify; the ledger is evidence already
/// in history, which a later generation must be shown to CONTAIN. See `ledger_succession`.
#[derive(Clone, Debug)]
struct ArchiveIntroduction {
    commit: String,
    directory: String,
    approvals: Option<Vec<u8>>,
}

type ArchiveIntroductionIndex = BTreeMap<String, Vec<ArchiveIntroduction>>;
static ARCHIVE_INTRODUCTION_CACHE: OnceLock<
    Mutex<BTreeMap<String, Arc<ArchiveIntroductionIndex>>>,
> = OnceLock::new();

/// Indexes every reachable commit that ADDS an archived change package, keyed by the change ID
/// recorded inside the committed `state.json` rather than by the directory the package occupies.
///
/// `find_change_dir` resolves an archived package by parsing every `state.json` under the archive
/// root and matching `record.id`, so the directory name is not part of a package's identity. An
/// anchor search keyed by path therefore accepts a commit that merely re-adds an already committed
/// package under a different name, which is what lets a `git mv` mint a fresh anchor for evidence
/// rewritten after archiving. Keying this index the way `find_change_dir` keys its lookup takes the
/// directory name out of the trust decision entirely.
///
/// Rename detection is disabled deliberately. With `diff.renames` on -- the default since Git 2.9
/// -- a relocation is reported as `R` and disappears from `--diff-filter=A`, which would make the
/// hole look closed while resting on a similarity heuristic that an attacker controls. Forcing
/// every first appearance of a path to surface as an addition puts the ordering rule in
/// `admissible_archive_introductions`, not Git's guesswork, in charge of the decision.
fn archive_introduction_index(root: &Path) -> Result<Arc<ArchiveIntroductionIndex>, String> {
    let repo_prefix = git_repo_prefix(root)?;
    let archive_root = format!("{repo_prefix}{ARCHIVE_PATH}");
    let history_pathspec = format!(":(top,literal){archive_root}");
    let directory_prefix = format!("{archive_root}/");
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let resolved: Vec<String> = references
        .iter()
        .map(|reference| {
            git_output(
                root,
                &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
            )
            .unwrap_or_default()
        })
        .collect();
    let cache_key = format!("{}|{archive_root}|{}", root.display(), resolved.join(","));
    if let Some(cached) = ARCHIVE_INTRODUCTION_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .ok()
        .and_then(|cache| cache.get(&cache_key).cloned())
    {
        return Ok(cached);
    }
    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let mut args = vec![
        "log".to_string(),
        "--format=%H".to_string(),
        "--diff-filter=A".to_string(),
        "--no-renames".to_string(),
        max_count,
    ];
    args.extend(references.iter().cloned());
    args.extend(["--".to_string(), history_pathspec.clone()]);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = run_git_bounded(root, &arg_refs, None, MAX_GIT_COMMAND_OUTPUT_BYTES)
        .map_err(|error| format!("failed to inspect archived acceptance history: {error}"))?;
    if !output.status.success() {
        return Err("failed to inspect archived acceptance history".into());
    }
    let history = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err(format!(
            "archived acceptance history exceeds the deterministic {}-commit bound",
            MAX_TRUSTED_HISTORY_COMMITS
        ));
    }
    let mut index: ArchiveIntroductionIndex = BTreeMap::new();
    for commit in commits {
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
                history_pathspec.as_str(),
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
        for path in names.split('\0').filter(|path| !path.is_empty()) {
            let Some(directory) = path.strip_suffix("/state.json") else {
                continue;
            };
            let Some(package) = directory.strip_prefix(&directory_prefix) else {
                continue;
            };
            if package.is_empty() || package.contains('/') {
                continue;
            }
            let Some(archived) = git_change_record_at(root, commit, path) else {
                continue;
            };
            let approvals = git_object_bytes(root, commit, &format!("{directory}/approvals.json"));
            let introductions = index.entry(archived.id).or_default();
            if introductions
                .iter()
                .any(|existing| existing.commit == commit && existing.directory == directory)
            {
                continue;
            }
            introductions.push(ArchiveIntroduction {
                commit: commit.to_string(),
                directory: directory.to_string(),
                approvals,
            });
        }
    }
    for introductions in index.values_mut() {
        introductions.sort_by(|left, right| {
            (&left.commit, &left.directory).cmp(&(&right.commit, &right.directory))
        });
    }
    let index = Arc::new(index);
    if let Ok(mut cache) = ARCHIVE_INTRODUCTION_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        cache.insert(cache_key, Arc::clone(&index));
    }
    Ok(index)
}

/// Whether a package's current approval ledger lawfully continues one already in history.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LedgerSuccession {
    /// The current ledger does not extend the committed one: it is the same generation, it is
    /// shorter, or it rewrote something the committed ledger already recorded.
    NotASuccessor,
    /// The current ledger keeps every approval and every reopen event the committed one recorded,
    /// adds at least one reopen event, and that event supersedes exactly the approval the
    /// committed package closed on.
    LawfulSuccessor,
}

/// The array named `field` in an approval-ledger JSON document, compared as raw JSON.
///
/// Raw values rather than `ApprovalLedger` structs on purpose: round-tripping through the typed
/// form silently drops fields the struct does not know, which is exactly where an attacker would
/// hide a difference between the ledger history committed and the one being presented.
fn ledger_entries(ledger: &serde_json::Value, field: &str) -> Vec<serde_json::Value> {
    ledger
        .get(field)
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// Decide whether `current` lawfully continues the ledger an earlier introduction committed.
///
/// `reopen` is append-only: `reopen_unarchived_change` leaves `approvals` untouched and pushes one
/// `ReopenRecord` whose `superseded_approval` is the package's own terminal approval, and the
/// `finalize` that follows only appends. A genuine later generation therefore CONTAINS the earlier
/// ledger verbatim. A relocation or a delete-and-re-add of the same package does not grow at all,
/// and a package whose committed approvals were rewritten -- the #660 laundering, whose payload is
/// an approval `actor` that no digest covers -- fails the prefix.
///
/// This is what the reopen count was asked to prove and could not. `reopenings.len()` is a number
/// written next to the evidence it is supposed to qualify, so appending one hand-made record used
/// to promote a rewritten package past the introduction that contradicts it. Extension is instead
/// checked against bytes that are already committed, which cannot be changed without rewriting
/// history itself.
///
/// `scope_adoptions` is deliberately not covered: `append_approval` clears it whenever a renewed
/// definition approval lands, so it legitimately shrinks across a reopen. It carries no actor and
/// `validate_scope_adoption` admits it for one hard-coded legacy change only.
fn ledger_succession(earlier: Option<&[u8]>, current: &[u8]) -> LedgerSuccession {
    let Some(earlier) = earlier else {
        return LedgerSuccession::NotASuccessor;
    };
    let (Ok(earlier), Ok(current)) = (
        serde_json::from_slice::<serde_json::Value>(earlier),
        serde_json::from_slice::<serde_json::Value>(current),
    ) else {
        return LedgerSuccession::NotASuccessor;
    };
    let earlier_approvals = ledger_entries(&earlier, "approvals");
    let current_approvals = ledger_entries(&current, "approvals");
    let earlier_reopenings = ledger_entries(&earlier, "reopenings");
    let current_reopenings = ledger_entries(&current, "reopenings");
    if current_reopenings.len() <= earlier_reopenings.len()
        || current_approvals.len() < earlier_approvals.len()
        || current_approvals[..earlier_approvals.len()] != earlier_approvals[..]
        || current_reopenings[..earlier_reopenings.len()] != earlier_reopenings[..]
    {
        return LedgerSuccession::NotASuccessor;
    }
    // The first reopen event the successor adds must supersede exactly the approval the earlier
    // package closed on, so that a new generation names the package it supersedes instead of
    // merely counting past it.
    let Some(superseded) = current_reopenings[earlier_reopenings.len()].get("superseded_approval")
    else {
        return LedgerSuccession::NotASuccessor;
    };
    let earlier_terminal = earlier_approvals.iter().rev().find(|approval| {
        approval
            .get("gate")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|gate| matches!(gate, "acceptance" | "finalization"))
    });
    if earlier_terminal != Some(superseded) {
        return LedgerSuccession::NotASuccessor;
    }
    LedgerSuccession::LawfulSuccessor
}

/// Selects the archived-package introductions that may anchor `record`'s acceptance.
///
/// A change's package can legitimately enter history more than once: `reopen` un-archives an
/// accepted package and a later `finalize` archives it again, so history holds an earlier
/// introduction of superseded evidence and a later introduction of the evidence in the tree.
/// What an attacker produces instead is a re-introduction of the SAME evidence -- a relocation, or
/// a delete-and-re-add -- carrying bytes rewritten after archiving.
///
/// The two are told apart by whether the package being authenticated CONTAINS the ledger the
/// earlier introduction committed. An introduction is admissible only when no strictly earlier
/// introduction of the same change committed a ledger the current package fails to extend. On a
/// package that was never reopened this reduces to "the earliest introduction wins", the property
/// a `git mv` cannot mint; on a reopened one it admits the new generation without ever consulting
/// a count the attacker controls.
fn admissible_archive_introductions(
    root: &Path,
    record: &ChangeRecord,
    current_approvals: &[u8],
) -> Result<Vec<ArchiveIntroduction>, String> {
    let index = archive_introduction_index(root)?;
    let Some(candidates) = index.get(&record.id) else {
        return Ok(Vec::new());
    };
    let mut admissible = Vec::new();
    for candidate in candidates {
        let superseded = candidates.iter().any(|earlier| {
            earlier.commit != candidate.commit
                && ledger_succession(earlier.approvals.as_deref(), current_approvals)
                    == LedgerSuccession::NotASuccessor
                && ensure_git_ancestor(
                    root,
                    &earlier.commit,
                    &candidate.commit,
                    "archived acceptance introduction",
                )
                .is_ok()
        });
        if !superseded {
            admissible.push(candidate.clone());
        }
    }
    Ok(admissible)
}

/// True when `current_approvals` lawfully extends every ledger history holds for this change.
///
/// Used only by the closing-evidence fallback, where the package being authenticated is not in
/// history yet because the commit that would introduce it has not been made. Every generation
/// already committed must be contained in the one about to be, so a reopen may add evidence and
/// may never rewrite what an earlier archive of the same change recorded.
fn closing_ledger_extends_all_introductions(
    root: &Path,
    record: &ChangeRecord,
    current_approvals: &[u8],
) -> Result<bool, String> {
    let index = archive_introduction_index(root)?;
    Ok(index.get(&record.id).is_some_and(|introductions| {
        !introductions.is_empty()
            && introductions.iter().all(|introduction| {
                ledger_succession(introduction.approvals.as_deref(), current_approvals)
                    == LedgerSuccession::LawfulSuccessor
            })
    }))
}

/// True when `anchor` lies in the history that produced one of `introductions`.
///
/// The active-workspace stages authenticate an archived change from `.specsync/changes/<id>/`,
/// where a genuine acceptance transition was recorded before the package was archived -- and where
/// a forged `reopen`/`archive` pair writes the same shape afterwards. Requiring the anchor to
/// precede the package's introduction keeps the first and rejects the second without depending on
/// the directory name, on a diff being empty, or on Git's rename detection.
fn anchor_precedes_an_introduction(
    root: &Path,
    anchor: &str,
    introductions: &[ArchiveIntroduction],
) -> bool {
    introductions.iter().any(|introduction| {
        introduction.commit == anchor
            || ensure_git_ancestor(
                root,
                anchor,
                &introduction.commit,
                "archived acceptance introduction",
            )
            .is_ok()
    })
}

/// The archive this process is creating right now, named by the package directory it is closing.
///
/// `archive_change_with_options` both writes a package and validates it. Only the writing process
/// knows that the closing evidence under `package` is evidence it authenticated out of the
/// change's own ACTIVE workspace moments ago, and that the commit which would introduce the
/// package is the one this command is preparing. Every reading path -- `status`, `audit`, `list`,
/// `ship-status`, the corpus census, the successor and legacy-baseline checks -- passes `None` and
/// is judged entirely by history, which is what #660 requires.
///
/// The token is deliberately NOT minted for a post-move resume whose package was found already
/// sitting in the archive. That shape cannot be told from an attacker flipping a committed
/// package's `state.json` back to `accepted` and re-running `finalize`, so it keeps HEAD's rule.
struct PendingArchiveClose<'a> {
    package: &'a Path,
}

impl PendingArchiveClose<'_> {
    /// True when `workspace` is the very package directory this process is closing.
    fn is_closing(&self, workspace: &Path) -> bool {
        let resolve = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        resolve(self.package) == resolve(workspace)
    }
}

fn authenticated_accepted_transition(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(String, Vec<u8>, ChangeRecord), String> {
    authenticated_accepted_transition_for(root, record, None)
}

fn authenticated_accepted_transition_for(
    root: &Path,
    record: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(String, Vec<u8>, ChangeRecord), String> {
    let workspace = find_change_dir(root, &record.id)?;
    let workspace_relative = strict_portable_project_path(root, &workspace)?;
    let current_verification_path = format!("{workspace_relative}/verification.json");
    let current_approvals_path = format!("{workspace_relative}/approvals.json");
    let candidates = BTreeSet::from([
        current_verification_path.clone(),
        current_approvals_path.clone(),
    ]);
    let evidence = git_evidence(root, &candidates)?;
    let current_verification = evidence.entry(&current_verification_path)?.payload.clone();
    let current_approvals = evidence.entry(&current_approvals_path)?.payload.clone();
    let active = format!("{CHANGES_PATH}/{}", record.id);
    let state_path = git_repo_relative_path(root, &format!("{active}/state.json"))?;
    let verification_path = git_repo_relative_path(root, &format!("{active}/verification.json"))?;
    let approvals_path = git_repo_relative_path(root, &format!("{active}/approvals.json"))?;
    let mut eligible = BTreeMap::new();
    // Once this change's archived package exists in reachable history, that history is the
    // authority for its acceptance. The active-workspace stages accept any commit that merely
    // carries the bytes being checked, which is how a forged `reopen`/`archive` pair mints an
    // anchor without ever touching the archive directory. They are therefore admitted only for
    // commits that PRECEDE the package's introduction -- the genuine pre-archive acceptance
    // transition -- and not at all once history holds the package. The working-tree fallback below
    // is bounded separately, because the one package history cannot hold is the one this command
    // is about to create.
    let introductions = if record.state == ChangeState::Archived {
        admissible_archive_introductions(root, record, &current_approvals)?
    } else {
        Vec::new()
    };
    let archived_package_is_in_history = record.state == ChangeState::Archived
        && archive_introduction_index(root)?.contains_key(&record.id);
    for anchor in accepted_transition_anchors(root, record)? {
        if archived_package_is_in_history
            && !anchor_precedes_an_introduction(root, &anchor, &introductions)
        {
            continue;
        }
        consider_accepted_evidence_anchor(
            root,
            record,
            &anchor,
            &state_path,
            &verification_path,
            &approvals_path,
            &current_verification,
            &current_approvals,
            &mut eligible,
        );
    }
    if record.state == ChangeState::Archived {
        for introduction in &introductions {
            let accepted_state_path = format!("{}/accepted-state.json", introduction.directory);
            let archived_verification_path =
                format!("{}/verification.json", introduction.directory);
            let archived_approvals_path = format!("{}/approvals.json", introduction.directory);
            consider_accepted_evidence_anchor(
                root,
                record,
                &introduction.commit,
                &accepted_state_path,
                &archived_verification_path,
                &archived_approvals_path,
                &current_verification,
                &current_approvals,
                &mut eligible,
            );
        }
    }
    if eligible.is_empty() {
        // Squash merges discard the original acceptance-transition commits while preserving
        // the accepted evidence bytes in later default-branch commits. Trust any in-history
        // commit that records this change as accepted when it carries byte-identical
        // evidence; the per-anchor checks, the introduction bound above and the exactly-one
        // rule still fail closed.
        for anchor in accepted_recording_anchors(root, record)? {
            if archived_package_is_in_history
                && !anchor_precedes_an_introduction(root, &anchor, &introductions)
            {
                continue;
            }
            consider_accepted_evidence_anchor(
                root,
                record,
                &anchor,
                &state_path,
                &verification_path,
                &approvals_path,
                &current_verification,
                &current_approvals,
                &mut eligible,
            );
        }
    }
    // A reopened change's SECOND package is not in history at the moment it is created: the
    // commit that would introduce it is the one this `finalize` is preparing. History holds only
    // the superseded generation, whose bytes stage B refuses exactly as it should, so #663's
    // "once the package is in history, history is the only authority" left a genuine re-finalize
    // with nothing to authenticate against (#540). The fallback is reopened for that case alone,
    // under two conditions no reader of a repository can satisfy together:
    //
    //   * this process is the one WRITING the package (`is_closing`), never true on a read path;
    //   * the ledger it is about to commit contains, unrewritten, every ledger history already
    //     holds for this change -- so a reopen may append evidence and may never restate what an
    //     earlier archive of the same change recorded.
    //
    // Both are required. The first alone would let `finalize` bless a package it merely found in
    // the archive; the second alone would let any working tree speak for a committed package.
    let closing_evidence_may_speak = !archived_package_is_in_history
        || (pending.is_some_and(|pending| pending.is_closing(&workspace))
            && closing_ledger_extends_all_introductions(root, record, &current_approvals)?);
    if eligible.is_empty()
        && closing_evidence_may_speak
        && let Ok(state_bytes) = fs::read(workspace.join("accepted-state.json"))
        && let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes)
    {
        let mut projection = record.clone();
        if record.state == ChangeState::Archived {
            projection.state = ChangeState::Accepted;
            projection.updated_at = accepted.updated_at;
        }
        if projection == accepted
            && accepted.state == ChangeState::Accepted
            && staged_accepted_snapshot_is_closing_authenticated(
                root,
                &accepted,
                &current_verification,
                &current_approvals,
            )?
        {
            let key =
                accepted_evidence_key(&state_bytes, &current_verification, &current_approvals);
            eligible.insert(
                key,
                (
                    "working-tree-closing-evidence".into(),
                    state_bytes,
                    accepted,
                ),
            );
        }
    }
    if eligible.len() != 1 {
        return Err(format!(
            "accepted change `{}` requires exactly one trusted transition matching its state, verification, and closing evidence; found {}",
            record.id,
            eligible.len()
        ));
    }
    eligible
        .into_values()
        .next()
        .ok_or_else(|| "authenticated accepted transition disappeared".to_string())
}

fn staged_accepted_snapshot_is_closing_authenticated(
    root: &Path,
    accepted: &ChangeRecord,
    verification_bytes: &[u8],
    approval_bytes: &[u8],
) -> Result<bool, String> {
    let verification: VerificationRecord = serde_json::from_slice(verification_bytes)
        .map_err(|error| format!("invalid staged accepted verification: {error}"))?;
    let approvals: ApprovalLedger = serde_json::from_slice(approval_bytes)
        .map_err(|error| format!("invalid staged accepted approvals: {error}"))?;
    if !verification.passed
        || !definition_digest_matches(root, accepted, &verification.contract_digest)?
    {
        return Ok(false);
    }
    if validate_verification_execution_digest(root, accepted, &verification).is_err() {
        return Ok(false);
    }
    let Some(signed_inputs) = verification.acceptance_input_digest.as_ref() else {
        return Ok(false);
    };
    if let Some(manifest) = &verification.acceptance_manifest {
        if acceptance_manifest_digest(manifest)? != *signed_inputs {
            return Ok(false);
        }
        match (&accepted.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(accepted, evidence)?,
            _ => return Ok(false),
        }
    } else if verification.semantic_succession.is_some() {
        return Ok(false);
    }
    if accepted.workflow_version >= 2 {
        return Ok(
            validate_finalization_evidence(root, accepted, &verification).is_ok()
                && verification_commit_is_accepted_current(root, &verification),
        );
    }
    let closing_matches = latest_terminal_approval(&approvals)
        .is_some_and(|approval| approval.digest == closing_digest(accepted, &verification));
    Ok(closing_matches && verification_commit_is_accepted_current(root, &verification))
}

fn accepted_evidence_key(state: &[u8], verification: &[u8], approvals: &[u8]) -> String {
    let mut digest = FramedDigest::new(b"specsync.accepted-evidence-key.v1");
    digest.frame(b"state", state);
    digest.frame(b"verification", verification);
    digest.frame(b"approvals", approvals);
    digest.finish()
}

fn git_object_bytes(root: &Path, commit: &str, path: &str) -> Option<Vec<u8>> {
    let object = format!("{commit}:{path}");
    let output = run_git_bounded(
        root,
        &["show", object.as_str()],
        None,
        MAX_CHANGE_ARTIFACT_BYTES as usize,
    )
    .ok()?;
    output.status.success().then_some(output.stdout)
}

fn git_change_record_at(root: &Path, commit: &str, path: &str) -> Option<ChangeRecord> {
    let object = format!("{commit}:{path}");
    let output = run_git_bounded(
        root,
        &["show", object.as_str()],
        None,
        MAX_CHANGE_ARTIFACT_BYTES as usize,
    )
    .ok()?;
    output
        .status
        .success()
        .then(|| serde_json::from_slice(&output.stdout).ok())
        .flatten()
}

fn acceptance_input_digest(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
) -> Result<String, String> {
    let discovery_scopes = acceptance_discovery_scopes(root, record)?;
    let override_content: BTreeMap<String, &[u8]> = overrides
        .iter()
        .map(|(path, content)| {
            Ok((
                strict_portable_project_path(root, path)?,
                content.as_bytes(),
            ))
        })
        .collect::<Result<_, String>>()?;
    let extra_candidates = override_content.keys().cloned().collect();
    let (mut paths, evidence) =
        stable_discovered_evidence(root, Some(&discovery_scopes), &extra_candidates, false)?;
    paths.retain(|path| {
        !project_input_is_volatile(path) && record_covers_project_path(root, record, path)
    });
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut digest = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    for relative in paths {
        if let Some(content) = override_content.get(&relative) {
            let mode = evidence.generated_file_mode(&relative)?;
            let kind: &[u8] = match mode {
                0o120000 => b"symlink",
                0o160000 => b"gitlink",
                _ => b"file",
            };
            digest.entry(&relative, kind, mode, content);
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            let mode = evidence.generated_file_mode(&relative)?;
            digest.entry(&relative, b"file", mode, content);
        } else {
            let entry = evidence.entry(&relative)?;
            digest.entry(
                &relative,
                acceptance_kind_bytes(&entry.kind),
                entry.mode,
                &entry.payload,
            );
        }
    }
    Ok(digest.finish())
}

fn historical_sequence_ledger_acceptance_content(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<Vec<u8>>, String> {
    validate_change_sequences(root)?;
    let Some(ledger) = load_change_sequence_ledger(root)? else {
        return Ok(None);
    };
    // A slug-only change has no earlier ledger revision to reconstruct: it never claimed a
    // sequence, so the frozen file it may have scoped is the same file it signed.
    let Some(sequence) = located_change_ordinal(&record.id)? else {
        return Ok(None);
    };
    if ledger.sequence <= sequence {
        return Ok(None);
    }
    if let Some(content) =
        historical_sequence_ledger_from_history(root, sequence, record.id.as_str())?
    {
        return Ok(Some(content));
    }
    let historical = ChangeSequenceLedger {
        schema_version: ledger.schema_version,
        sequence,
        id: record.id.clone(),
        acknowledged_collisions: ledger
            .acknowledged_collisions
            .into_iter()
            .filter(|collision| collision.sequence <= sequence)
            .collect(),
    };
    Ok(Some(json_content(&historical)?.into_bytes()))
}

fn historical_sequence_ledger_from_history(
    root: &Path,
    sequence: u64,
    record_id: &str,
) -> Result<Option<Vec<u8>>, String> {
    let candidates = if let Some(cached) = read_scope_value(root, |scope| {
        scope.historical_sequence_ledgers.get(&sequence).cloned()
    }) {
        cached?
    } else {
        let result = (|| {
            let mut candidates = Vec::new();
            if !git_repository_present(root)? {
                return Ok(candidates);
            }
            let path = git_repo_relative_path(root, SEQUENCE_PATH)?;
            let limits = lifecycle_validation_limits();
            let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
            let history = scoped_review_git_text(
                root,
                &["rev-list", max_count.as_str(), "HEAD", "--", path.as_str()],
            )
            .map_err(|_| "failed to inspect historical change-sequence ledgers".to_string())?;
            let commits: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
            if commits.len() > limits.scoped_review_max_descendants {
                return Err(format!(
                    "change-sequence history exceeds the deterministic {}-commit bound",
                    limits.scoped_review_max_descendants
                ));
            }
            for commit in commits {
                let Some(bytes) = scoped_review_file_at_commit(root, commit, &path)? else {
                    continue;
                };
                let Ok(candidate) = serde_json::from_slice::<ChangeSequenceLedger>(&bytes) else {
                    continue;
                };
                if candidate.schema_version == 1
                    && candidate.sequence == sequence
                    && change_sequence(&candidate.id) == Some(sequence)
                    && json_content(&candidate)?.as_bytes() == bytes
                {
                    candidates.push(bytes);
                }
            }
            Ok(candidates)
        })();
        update_read_scope(root, |scope| {
            scope
                .historical_sequence_ledgers
                .insert(sequence, result.clone());
        });
        result?
    };
    for bytes in candidates {
        let candidate: ChangeSequenceLedger =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if candidate.acknowledged_collisions.iter().any(|collision| {
            collision.sequence == sequence && collision.ids.iter().any(|id| id == record_id)
        }) {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

fn resolve_definition_approval_event<'a>(
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
    expected_current_digest: &str,
    expected_legacy_digest: Option<&str>,
    expected_correction_prefix: &str,
) -> Result<&'a ApprovalRecord, String> {
    let index = ledger
        .approvals
        .iter()
        .rposition(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    let terminal = &ledger.approvals[index];
    let Some(legacy_metadata) = terminal.definition_pair.as_ref() else {
        if terminal.digest != expected_current_digest {
            return Err(
                "definition approval is stale; approve the current artifact digest again".into(),
            );
        }
        return Ok(terminal);
    };
    if legacy_metadata.schema_version != 1
        || legacy_metadata.projection != PORTABLE_DEFINITION_PROJECTION_V501
        || legacy_metadata.role != DefinitionApprovalPairRole::Legacy
        || legacy_metadata.change_id != record.id
        || legacy_metadata.correction_count != record.correction_count
        || legacy_metadata.correction_prefix_digest != expected_correction_prefix
        || legacy_metadata.event_index.checked_add(1) != Some(index as u64)
    {
        return Err("portable definition approval has invalid terminal metadata".into());
    }
    let current_index = index
        .checked_sub(1)
        .ok_or_else(|| "portable definition approval is missing its current member".to_string())?;
    let current = &ledger.approvals[current_index];
    let current_metadata = current
        .definition_pair
        .as_ref()
        .ok_or_else(|| "portable definition approval current member is unmarked".to_string())?;
    if current.gate != "definition"
        || current_metadata.schema_version != legacy_metadata.schema_version
        || current_metadata.projection != legacy_metadata.projection
        || current_metadata.pair_id != legacy_metadata.pair_id
        || current_metadata.role != DefinitionApprovalPairRole::Current
        || current_metadata.change_id != legacy_metadata.change_id
        || current_metadata.correction_count != legacy_metadata.correction_count
        || current_metadata.correction_prefix_digest != legacy_metadata.correction_prefix_digest
        || current_metadata.current_digest != legacy_metadata.current_digest
        || current_metadata.legacy_digest != legacy_metadata.legacy_digest
        || current_metadata.event_index != legacy_metadata.event_index
        || current_metadata.event_index != current_index as u64
        || current.actor != terminal.actor
        || current.timestamp != terminal.timestamp
        || current.digest == terminal.digest
        || current.digest != expected_current_digest
        || current.digest != current_metadata.current_digest
        || terminal.digest != legacy_metadata.legacy_digest
        || expected_legacy_digest.is_some_and(|expected| terminal.digest != expected)
    {
        return Err("portable definition approval pair is malformed or stale".into());
    }
    let expected_pair_id = definition_approval_pair_id(
        record,
        current_index as u64,
        &current.actor,
        current.timestamp,
        expected_correction_prefix,
        &current.digest,
        &terminal.digest,
    );
    if legacy_metadata.pair_id != expected_pair_id
        || ledger
            .approvals
            .iter()
            .filter(|approval| {
                approval
                    .definition_pair
                    .as_ref()
                    .is_some_and(|metadata| metadata.pair_id == legacy_metadata.pair_id)
            })
            .count()
            != 2
    {
        return Err("portable definition approval pair is duplicated or replayed".into());
    }
    Ok(current)
}

fn validate_scope_adoption<'a>(
    root: &Path,
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
    approval_index: usize,
    approval: &ApprovalRecord,
) -> Result<&'a ApprovedScopeV1, String> {
    let matches: Vec<&ScopeAdoptionV1> = ledger
        .scope_adoptions
        .iter()
        .filter(|adoption| adoption.source_approval_index == approval_index as u64)
        .collect();
    if matches.len() != 1 || ledger.scope_adoptions.len() != 1 {
        return Err("scope adoption must contain one unambiguous allowlisted event".into());
    }
    let adoption = matches[0];
    let immutable_event_matches = record.id == CHG_0068_ID
        && approval_index == 0
        && approval.gate == "definition"
        && approval.actor == "0xLeif"
        && approval.timestamp == 1_785_369_606
        && approval.digest == CHG_0068_LEGACY_APPROVAL_DIGEST
        && approval.note.is_none()
        && approval.definition_pair.is_none()
        && approval.approved_scope.is_none()
        && approval.scope_migration.is_none()
        // The allowlist pins the ENTIRE shape of the one trusted historical event; leaving a
        // field unpinned would let a later rewrite add evidence the allowlist never authorized.
        && approval.approved_delta_digests.is_none();
    if !immutable_event_matches {
        return Err(
            "scope adoption source approval is outside the trusted CHG-0068 allowlist".into(),
        );
    }
    let adoption_shape_matches = adoption.schema_version == 1
        && adoption.change_id == CHG_0068_ID
        && adoption.source_approval_index == 0
        && adoption.legacy_approval_digest == CHG_0068_LEGACY_APPROVAL_DIGEST
        && adoption.source_preimage_status == ScopeAdoptionSourcePreimageStatus::Unavailable
        && adoption.equivalence_claim == ScopeAdoptionEquivalenceClaim::None
        && adoption.adopted_scope_digest == CHG_0068_ADOPTED_SCOPE_DIGEST
        && adoption.anchor.base_commit == CHG_0068_ADOPTION_BASE_COMMIT
        && adoption.anchor.commit == CHG_0068_ADOPTION_ANCHOR_COMMIT
        && adoption.anchor.approval_index == 0
        && adoption.anchor.approvals_blob_sha256 == CHG_0068_ADOPTION_ANCHOR_BLOB
        && adoption.authorization.actor == "0xLeif"
        && adoption.authorization.recorded_at == 1_785_381_022
        && adoption.authorization.reason == CHG_0068_ADOPTION_REASON
        && !adoption.changes.is_empty()
        && adoption.changes.iter().all(|change| {
            !change.path.trim().is_empty()
                && change.path.trim() == change.path
                && !change.summary.trim().is_empty()
                && change.summary.trim() == change.summary
                && strict_portable_relative_path(&change.path).is_ok()
        });
    if !adoption_shape_matches {
        return Err("scope adoption differs from the trusted CHG-0068 allowlist".into());
    }
    if scope_digest_from_approved(&adoption.adopted_scope)? != CHG_0068_ADOPTED_SCOPE_DIGEST {
        return Err("scope adoption projection does not match its allowlisted digest".into());
    }
    let changes = serde_json::to_vec(&adoption.changes)
        .map_err(|error| format!("failed to hash scope adoption classifications: {error}"))?;
    if sha256_hex(&changes) != CHG_0068_ADOPTION_CHANGES_DIGEST {
        return Err("scope adoption classifications differ from the trusted allowlist".into());
    }

    let anchor_expression = format!("{}^{{commit}}", adoption.anchor.commit);
    let resolved_anchor = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &anchor_expression,
        ],
    )
    .ok_or_else(|| {
        "scope adoption anchor is unavailable; fetch full trusted history".to_string()
    })?;
    if resolved_anchor != adoption.anchor.commit {
        return Err("scope adoption anchor is not canonical".into());
    }
    let resolved_base = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{}^{{commit}}", adoption.anchor.base_commit),
        ],
    )
    .ok_or_else(|| "scope adoption base commit is unavailable".to_string())?;
    let anchor_parent = git_output(
        root,
        &["rev-parse", &format!("{}^", adoption.anchor.commit)],
    )
    .ok_or_else(|| "scope adoption anchor parent is unavailable".to_string())?;
    if resolved_base != adoption.anchor.base_commit || anchor_parent != resolved_base {
        return Err("scope adoption anchor is outside its allowlisted base lineage".into());
    }
    let path = git_repo_relative_path(
        root,
        &format!("{CHANGES_PATH}/{}/approvals.json", record.id),
    )?;
    let bytes = git_file_at_commit(root, &resolved_anchor, &path)?
        .ok_or_else(|| "scope adoption anchor approval blob is unavailable".to_string())?;
    if sha256_hex(&bytes) != CHG_0068_ADOPTION_ANCHOR_BLOB {
        return Err("scope adoption anchor approval blob is not allowlisted".into());
    }
    let anchor: ApprovalLedger = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid scope adoption anchor approval ledger: {error}"))?;
    let anchored_approval = anchor
        .approvals
        .get(adoption.anchor.approval_index as usize)
        .ok_or_else(|| "scope adoption anchor approval index is missing".to_string())?;
    let anchored_migration = anchored_approval
        .scope_migration
        .as_ref()
        .ok_or_else(|| "scope adoption anchor migration projection is missing".to_string())?;
    if anchored_approval.gate != approval.gate
        || anchored_approval.actor != approval.actor
        || anchored_approval.timestamp != approval.timestamp
        || anchored_approval.digest != approval.digest
        || anchored_approval.note != approval.note
        || anchored_approval.approved_scope.as_ref() != Some(&adoption.adopted_scope)
        || anchored_migration.source_definition_digest != adoption.legacy_approval_digest
        || anchored_migration.scope_digest != adoption.adopted_scope_digest
        || anchored_migration.changes != adoption.changes
    {
        return Err("scope adoption does not match its immutable anchor projection".into());
    }
    Ok(&adoption.adopted_scope)
}

fn effective_definition_approval<'a>(
    root: &Path,
    record: &ChangeRecord,
    ledger: &'a ApprovalLedger,
) -> Result<&'a ApprovalRecord, String> {
    let latest_index = ledger
        .approvals
        .iter()
        .rposition(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    let latest = &ledger.approvals[latest_index];
    if record.workflow_version >= 2 {
        let approved = if let Some(approved) = latest.approved_scope.as_ref() {
            if latest.scope_migration.is_some() || !ledger.scope_adoptions.is_empty() {
                return Err(
                    "scope approval combines incompatible direct and adopted projections".into(),
                );
            }
            approved
        } else {
            validate_scope_adoption(root, record, ledger, latest_index, latest)?
        };
        if approved.schema_version != 1 || approved.change_id != record.id {
            return Err("scope approval projection has the wrong schema or change identity".into());
        }
        let approved_digest = scope_digest_from_approved(approved)?;
        if latest.approved_scope.is_some() && latest.digest != approved_digest {
            return Err("scope approval digest does not match its approved projection".into());
        }
        let current = approved_scope(root, record)?;
        let expansion = scope_expansion(approved, &current);
        if !expansion.is_empty() {
            return Err(format!(
                "scope approval requires renewal because {}",
                expansion.join("; ")
            ));
        }
        return Ok(latest);
    }
    let corrections = load_correction_ledger(root, record)?;
    let prefix = correction_prefix_digest(record, &corrections.corrections)?;
    if latest.definition_pair.is_some() {
        let (current, legacy, portable_prefix) =
            portable_definition_digest_pair_v501(root, record)?;
        if prefix != portable_prefix {
            return Err("portable definition approval correction prefix is stale".into());
        }
        if let Ok(approval) =
            resolve_definition_approval_event(record, ledger, &current, Some(&legacy), &prefix)
        {
            return Ok(approval);
        }
        let (legacy_current, legacy_projected, legacy_prefix) =
            portable_definition_digest_pair_v501_with_task_mode(root, record, false)?;
        if prefix != legacy_prefix {
            return Err("portable definition approval correction prefix is stale".into());
        }
        return resolve_definition_approval_event(
            record,
            ledger,
            &legacy_current,
            Some(&legacy_projected),
            &prefix,
        );
    }
    let current = definition_digest(root, record)?;
    match resolve_definition_approval_event(record, ledger, &current, None, &prefix) {
        Ok(approval) => Ok(approval),
        Err(_) => {
            let transitional = definition_digest_with_explicit_false(root, record)?;
            if let Ok(approval) =
                resolve_definition_approval_event(record, ledger, &transitional, None, &prefix)
            {
                return Ok(approval);
            }
            for explicit_false in [false, true] {
                let legacy = legacy_task_definition_digest_for_correction_count(
                    root,
                    record,
                    record.correction_count,
                    explicit_false,
                )?;
                if let Ok(approval) =
                    resolve_definition_approval_event(record, ledger, &legacy, None, &prefix)
                {
                    return Ok(approval);
                }
            }
            Err("definition approval is stale; approve the current artifact digest again".into())
        }
    }
}

fn ensure_definition_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    effective_definition_approval(root, record, &ledger).map(|_| ())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcceptedInputValidity {
    Exact,
    SuccessorCovered,
}

fn terminal_evidence_summary(root: &Path, record: &ChangeRecord) -> TerminalEvidenceSummary {
    if let Some(cached) = read_scope_value(root, |scope| scope.terminal_evidence.clone()) {
        return terminal_summary_from_cache(record, cached);
    }
    if read_scope_value(root, |_| Some(())).is_some() {
        let result = list_all_changes_checked(root).map(|records| {
            let (results, _) = terminal_evidence_results_with_records(root, &records);
            results
                .into_iter()
                .map(|result| (result.id, result.evidence))
                .collect()
        });
        update_read_scope(root, |scope| {
            scope.terminal_evidence = Some(result.clone());
        });
        return terminal_summary_from_cache(record, result);
    }
    let result = list_all_changes_checked(root)
        .map(|records| terminal_evidence_summary_with_records(root, record, &records));
    match result {
        Ok(summary) => summary,
        Err(reason) => TerminalEvidenceSummary {
            validity: if record.state == ChangeState::Archived {
                TerminalEvidenceValidity::CorruptHistory
            } else {
                TerminalEvidenceValidity::Stale
            },
            reason: Some(reason),
        },
    }
}

fn terminal_summary_from_cache(
    record: &ChangeRecord,
    cached: Result<BTreeMap<String, TerminalEvidenceSummary>, String>,
) -> TerminalEvidenceSummary {
    match cached {
        Ok(summaries) => {
            summaries
                .get(&record.id)
                .cloned()
                .unwrap_or_else(|| TerminalEvidenceSummary {
                    validity: if record.state == ChangeState::Archived {
                        TerminalEvidenceValidity::CorruptHistory
                    } else {
                        TerminalEvidenceValidity::Stale
                    },
                    reason: Some(format!(
                        "terminal evidence snapshot is missing change `{}`",
                        record.id
                    )),
                })
        }
        Err(reason) => TerminalEvidenceSummary {
            validity: if record.state == ChangeState::Archived {
                TerminalEvidenceValidity::CorruptHistory
            } else {
                TerminalEvidenceValidity::Stale
            },
            reason: Some(reason),
        },
    }
}

fn terminal_evidence_summary_with_records(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
) -> TerminalEvidenceSummary {
    terminal_evidence_summary_with_validation_state(
        root,
        record,
        records,
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
        &mut ArchivedIntegrityCache::default(),
    )
}

fn terminal_evidence_results_with_records(
    root: &Path,
    records: &BTreeMap<String, ChangeRecord>,
) -> (Vec<TerminalEvidenceResult>, ArchivedIntegrityCache) {
    let mut visiting = BTreeSet::new();
    let mut memo = BTreeMap::new();
    let mut archived_cache = ArchivedIntegrityCache::default();
    let results = records
        .values()
        .filter(|record| matches!(record.state, ChangeState::Accepted | ChangeState::Archived))
        .map(|record| TerminalEvidenceResult {
            id: record.id.clone(),
            evidence: terminal_evidence_summary_with_validation_state(
                root,
                record,
                records,
                &mut visiting,
                &mut memo,
                &mut archived_cache,
            ),
        })
        .collect();
    (results, archived_cache)
}

fn terminal_evidence_summary_with_validation_state(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
    archived_cache: &mut ArchivedIntegrityCache,
) -> TerminalEvidenceSummary {
    if record.state == ChangeState::Archived {
        return match validate_archived_integrity_with_cache(root, record, archived_cache) {
            Ok(()) => TerminalEvidenceSummary {
                validity: TerminalEvidenceValidity::AuthenticatedHistory,
                reason: None,
            },
            Err(reason) => TerminalEvidenceSummary {
                validity: TerminalEvidenceValidity::CorruptHistory,
                reason: Some(reason),
            },
        };
    }
    match validate_accepted_inputs_recursive(root, record, records, visiting, memo) {
        Ok(AcceptedInputValidity::Exact) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::Exact,
            reason: None,
        },
        Ok(AcceptedInputValidity::SuccessorCovered) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::SuccessorCovered,
            reason: None,
        },
        Err(reason) => TerminalEvidenceSummary {
            validity: TerminalEvidenceValidity::Stale,
            reason: Some(reason),
        },
    }
}

fn ensure_closing_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let records = list_all_changes_checked(root)?;
    let mut visiting = BTreeSet::new();
    let mut memo = BTreeMap::new();
    validate_accepted_inputs_recursive(root, record, &records, &mut visiting, &mut memo).map(|_| ())
}

fn authenticate_accepted_evidence_with_anchor(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(VerificationRecord, bool), String> {
    validate_acceptance_owner_correction_records(record)?;
    if record.state == ChangeState::Archived {
        validate_archived_accepted_snapshot(root, record, None)?;
    }
    let verification = load_verification(root, record)?;
    if !verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, record)?;
    if !definition_digest_matches(root, record, &verification.contract_digest)? {
        return Err("accepted change verification contract is stale".into());
    }
    validate_verification_execution_digest(root, record, &verification)?;
    let expected_inputs = verification
        .acceptance_input_digest
        .as_ref()
        .ok_or_else(|| "accepted change is missing current delivery-input evidence".to_string())?;
    if let Some(manifest) = &verification.acceptance_manifest {
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != *expected_inputs {
            return Err(
                "accepted change manifest does not reproduce signed delivery inputs".into(),
            );
        }
        match (&record.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(record, evidence)?,
            _ => {
                return Err(
                    "accepted change is missing semantic succession evidence for approved obligations"
                        .into(),
                );
            }
        }
    } else if verification.semantic_succession.is_some() {
        return Err("legacy acceptance cannot carry succession evidence without a manifest".into());
    }
    if record.workflow_version >= 2 {
        validate_finalization_evidence(root, record, &verification)?;
    } else {
        let expected = closing_digest(record, &verification);
        let ledger = load_approvals(root, record)?;
        let approval = latest_terminal_approval(&ledger)
            .ok_or_else(|| "accepted change is missing closing approval".to_string())?;
        if approval.digest != expected {
            return Err(
                "accepted change closing approval does not match verification evidence".into(),
            );
        }
    }
    let anchored = accepted_evidence_is_anchored(root, record, &verification);
    Ok((verification, anchored))
}

/// True when accepted evidence is still tied to history somebody can reach.
fn accepted_evidence_is_anchored(
    root: &Path,
    record: &ChangeRecord,
    evidence: &VerificationRecord,
) -> bool {
    verification_commit_is_accepted_current(root, evidence)
        || accepted_workspace_is_integrated(root, record)
        || accepted_change_is_recorded_on_remote_default(root, record)
}

fn authenticate_accepted_evidence(
    root: &Path,
    record: &ChangeRecord,
) -> Result<VerificationRecord, String> {
    match authenticate_accepted_evidence_with_anchor(root, record)? {
        (verification, true) => Ok(verification),
        (_, false) => Err("accepted change verification commit is not in current history and canonical acceptance is not recorded on the remote default branch".into()),
    }
}

fn validate_archived_accepted_snapshot(
    root: &Path,
    archived: &ChangeRecord,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(), String> {
    let workspace = find_change_dir(root, &archived.id)?;
    let path = workspace.join("accepted-state.json");
    let (_, historical_bytes, historical) =
        authenticated_accepted_transition_for(root, archived, pending)?;
    let accepted = match fs::read(&path) {
        Ok(content) => {
            if content != historical_bytes {
                return Err(format!(
                    "archived change `{}` accepted-state snapshot does not match its trusted transition",
                    archived.id
                ));
            }
            serde_json::from_slice(&content).map_err(|error| {
                format!(
                    "invalid accepted-state snapshot {}: {error}",
                    path.display()
                )
            })?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let verification = load_verification(root, archived)?;
            if verification.acceptance_manifest.is_some() {
                return Err(format!(
                    "archived change `{}` is missing its authenticated accepted-state snapshot",
                    archived.id
                ));
            }
            historical
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    if accepted.id != archived.id || accepted.state != ChangeState::Accepted {
        return Err(format!(
            "archived change `{}` has an invalid accepted-state snapshot identity",
            archived.id
        ));
    }
    let mut projection = archived.clone();
    projection.state = ChangeState::Accepted;
    projection.updated_at = accepted.updated_at;
    if projection != accepted {
        return Err(format!(
            "archived change `{}` does not match its authenticated accepted-state snapshot",
            archived.id
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct LegacyArchiveBaselineContext {
    baseline: LegacyArchiveBaselineV1,
    snapshots: BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
}

#[derive(Debug, Default)]
struct ArchivedIntegrityCache {
    legacy: Option<Result<LegacyArchiveBaselineContext, String>>,
}

fn validate_archived_integrity(root: &Path, archived: &ChangeRecord) -> Result<(), String> {
    validate_archived_integrity_with_cache(root, archived, &mut ArchivedIntegrityCache::default())
}

fn validate_archived_integrity_with_cache(
    root: &Path,
    archived: &ChangeRecord,
    cache: &mut ArchivedIntegrityCache,
) -> Result<(), String> {
    validate_archived_integrity_inner(root, archived, cache, None)
}

/// The archive preflight run by the command that is creating the package, which is the only caller
/// allowed to present closing evidence history has not seen yet. See `PendingArchiveClose`.
fn validate_archived_integrity_closing(
    root: &Path,
    archived: &ChangeRecord,
    pending: &PendingArchiveClose<'_>,
) -> Result<(), String> {
    validate_archived_integrity_inner(
        root,
        archived,
        &mut ArchivedIntegrityCache::default(),
        Some(pending),
    )
}

fn validate_archived_integrity_inner(
    root: &Path,
    archived: &ChangeRecord,
    cache: &mut ArchivedIntegrityCache,
    pending: Option<&PendingArchiveClose<'_>>,
) -> Result<(), String> {
    validate_acceptance_owner_correction_records(archived)?;
    let workspace = find_change_dir(root, &archived.id)?;
    if !workspace.join("accepted-state.json").exists() {
        return authenticate_legacy_archive_baseline(root, archived, &workspace, cache);
    }
    validate_archived_accepted_snapshot(root, archived, pending)?;
    let verification = load_verification(root, archived)?;
    if !verification.passed {
        return Err("archived change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, archived)?;
    if !definition_digest_matches(root, archived, &verification.contract_digest)? {
        return Err("archived change verification contract is stale".into());
    }
    validate_verification_execution_digest(root, archived, &verification)?;
    let expected_inputs = verification
        .acceptance_input_digest
        .as_ref()
        .ok_or_else(|| "archived change is missing delivery-input evidence".to_string())?;
    if let Some(manifest) = &verification.acceptance_manifest {
        validate_acceptance_manifest(manifest)?;
        if acceptance_manifest_digest(manifest)? != *expected_inputs {
            return Err(
                "archived change manifest does not reproduce signed delivery inputs".into(),
            );
        }
        match (&archived.supersedes[..], &verification.semantic_succession) {
            ([], None) => {}
            (_, Some(evidence)) => validate_semantic_succession(archived, evidence)?,
            _ => {
                return Err(
                    "archived change is missing semantic succession evidence for approved obligations"
                        .into(),
                );
            }
        }
    } else {
        if verification.semantic_succession.is_some() {
            return Err(
                "legacy archive cannot carry succession evidence without a manifest".into(),
            );
        }
        reconstruct_legacy_acceptance_manifest(root, archived, expected_inputs)?;
    }
    if archived.workflow_version >= 2 {
        validate_finalization_evidence(root, archived, &verification)?;
    } else {
        let ledger = load_approvals(root, archived)?;
        let approval = latest_terminal_approval(&ledger)
            .ok_or_else(|| "archived change is missing closing approval".to_string())?;
        if approval.digest != closing_digest(archived, &verification) {
            return Err(
                "archived change closing approval does not match verification evidence".into(),
            );
        }
    }
    Ok(())
}

fn authenticate_legacy_archive_baseline(
    root: &Path,
    archived: &ChangeRecord,
    workspace: &Path,
    cache: &mut ArchivedIntegrityCache,
) -> Result<(), String> {
    let verification = load_verification(root, archived)?;
    if verification.acceptance_manifest.is_some() || verification.semantic_succession.is_some() {
        return Err(format!(
            "archived change `{}` is missing its authenticated accepted-state snapshot",
            archived.id
        ));
    }
    if !verification.passed {
        return Err("legacy archive has no passed verification evidence".into());
    }
    let ledger = load_approvals(root, archived)?;
    if !ledger
        .approvals
        .iter()
        .any(|approval| approval.gate == "definition")
    {
        return Err("legacy archive is missing stored definition approval".into());
    }
    ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .ok_or_else(|| "legacy archive is missing stored closing approval".to_string())?;

    if cache.legacy.is_none() {
        cache.legacy = Some(load_legacy_archive_baseline_context(root));
    }
    let context = cache
        .legacy
        .as_ref()
        .expect("legacy archive cache was initialized")
        .as_ref()
        .map_err(Clone::clone)?;
    let baseline = &context.baseline;
    let archive_root = root.join(ARCHIVE_PATH);
    let relative_workspace = workspace.strip_prefix(&archive_root).map_err(|_| {
        format!(
            "legacy archive workspace is outside {}",
            archive_root.display()
        )
    })?;
    let project_subtree = format!(
        "{}/{}",
        ARCHIVE_PATH,
        strict_portable_relative_path(
            relative_workspace
                .to_str()
                .ok_or_else(|| "legacy archive path is not UTF-8".to_string())?
        )?
    );
    let baseline_entry = legacy_baseline_entry(baseline, &archived.id)?;
    if baseline_entry.archive_path != project_subtree {
        return Err(format!(
            "legacy archive `{}` baseline path does not match its unique workspace",
            archived.id
        ));
    }
    let current = context.snapshots.get(&archived.id).ok_or_else(|| {
        format!(
            "legacy archive `{}` has no captured baseline subtree",
            archived.id
        )
    })?;
    if legacy_archive_subtree_digest(current)? != baseline_entry.subtree_digest {
        return Err(format!(
            "legacy archive `{}` subtree does not match its baseline digest",
            archived.id
        ));
    }
    Ok(())
}

fn load_legacy_archive_baseline_context(
    root: &Path,
) -> Result<LegacyArchiveBaselineContext, String> {
    let baseline_path = root.join(LEGACY_BASELINE_PATH);
    let baseline_bytes = fs::read(&baseline_path)
        .map_err(|error| format!("failed to read legacy archive baseline: {error}"))?;
    let (baseline, baseline_digest) = validate_legacy_archive_baseline_bytes(&baseline_bytes)?;

    let records = list_all_changes_checked(root)?;
    let authority = records.get(&baseline.authority_change_id).ok_or_else(|| {
        format!(
            "legacy archive baseline authority `{}` is unavailable",
            baseline.authority_change_id
        )
    })?;
    if authority.legacy_archive_baseline_digest.as_deref() != Some(&baseline_digest) {
        return Err(
            "legacy archive baseline bytes do not match its definition-bound authority".into(),
        );
    }
    ensure_definition_approval_valid(root, authority)
        .map_err(|error| format!("legacy archive baseline authority is not approved: {error}"))?;

    validate_legacy_baseline_authority_cutoff(root, authority, &baseline.cutoff_commit)?;
    let cutoff = baseline.cutoff_commit.clone();
    match authority.state {
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying => {}
        ChangeState::Accepted | ChangeState::Archived => {
            let authority_verification =
                authenticate_accepted_evidence(root, authority).map_err(|error| {
                    format!("legacy archive baseline authority is not closing-valid: {error}")
                })?;
            let manifest = authority_verification
                .acceptance_manifest
                .as_ref()
                .ok_or_else(|| {
                    "accepted legacy archive baseline authority must be manifest-backed".to_string()
                })?;
            let ledger_entry = manifest
                .entries
                .iter()
                .find(|entry| entry.path == LEGACY_BASELINE_PATH)
                .ok_or_else(|| {
                    "accepted legacy archive baseline authority did not sign the ledger path"
                        .to_string()
                })?;
            if ledger_entry.kind != AcceptanceInputKind::File
                || !ledger_entry
                    .owners
                    .iter()
                    .any(|owner| owner == EXACT_DELIVERY_OWNER)
                || ledger_entry.payload_digest != sha256_hex(&baseline_bytes)
                || ledger_entry.entry_digest
                    != acceptance_entry_digest(
                        LEGACY_BASELINE_PATH,
                        &AcceptanceInputKind::File,
                        ledger_entry.mode,
                        &ledger_entry.payload_digest,
                    )
            {
                return Err(
                    "accepted legacy archive baseline authority has invalid exact ledger evidence"
                        .into(),
                );
            }
            let (anchor, _, _) = authenticated_accepted_transition(root, authority)?;
            ensure_git_ancestor(root, &cutoff, &anchor, "authority acceptance anchor")?;
        }
        ChangeState::Draft => {
            return Err("legacy archive baseline authority is still draft".into());
        }
    }

    let scopes = baseline
        .entries
        .iter()
        .map(|entry| entry.archive_path.clone())
        .collect();
    let (project_paths, evidence) =
        stable_discovered_evidence(root, Some(&scopes), &BTreeSet::new(), false)?;
    let snapshots = baseline
        .entries
        .iter()
        .map(|entry| {
            archive_snapshot_from_evidence(&project_paths, &evidence, &entry.archive_path)
                .map(|snapshot| (entry.id.clone(), snapshot))
        })
        .collect::<Result<_, _>>()?;
    validate_legacy_archive_introductions(root, &baseline, &snapshots)?;

    Ok(LegacyArchiveBaselineContext {
        baseline,
        snapshots,
    })
}

fn validate_legacy_archive_introductions(
    root: &Path,
    baseline: &LegacyArchiveBaselineV1,
    snapshots: &BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
) -> Result<(), String> {
    let cutoff = baseline.cutoff_commit.as_str();
    let mut repo_subtrees = BTreeMap::new();
    let mut introductions = BTreeSet::new();
    let repo_prefix = git_repo_prefix(root)?;
    for entry in &baseline.entries {
        let repo_subtree = format!("{repo_prefix}{}", entry.archive_path);
        repo_subtrees.insert(entry.id.clone(), repo_subtree);
        introductions.insert(entry.introduction_commit.as_str());
    }
    let archive_root = format!("{repo_prefix}{ARCHIVE_PATH}");
    let history_pathspec = format!(":(top,literal){archive_root}");
    for introduction in introductions {
        let resolved = git_output(
            root,
            &[
                "rev-parse",
                "--verify",
                &format!("{introduction}^{{commit}}"),
            ],
        )
        .ok_or_else(|| format!("legacy archive introduction `{introduction}` is unavailable"))?;
        if resolved != introduction {
            return Err(format!(
                "legacy archive introduction `{introduction}` must be a canonical commit ID"
            ));
        }
        ensure_git_ancestor(root, introduction, cutoff, "legacy archive cutoff")?;
    }

    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let output = run_git_bounded(
        root,
        &[
            "log",
            "--format=%H",
            "--diff-filter=A",
            &max_count,
            cutoff,
            "--",
            &history_pathspec,
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )
    .map_err(|error| format!("failed to inspect legacy archive introductions: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect legacy archive introductions at cutoff `{cutoff}`"
        ));
    }
    let commits: BTreeSet<_> = String::from_utf8(output.stdout)
        .map_err(|_| "legacy archive introduction history is not UTF-8".to_string())?
        .lines()
        .map(str::to_string)
        .collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err(format!(
            "legacy archive introduction history exceeds the deterministic {}-commit bound",
            MAX_TRUSTED_HISTORY_COMMITS
        ));
    }
    let mut candidates: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut parents = BTreeMap::new();
    for commit in commits {
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
                "--diff-filter=A",
                &commit,
                "--",
                &history_pathspec,
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )
        .map_err(|error| {
            format!("failed to inspect legacy archive introduction `{commit}`: {error}")
        })?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect legacy archive introduction `{commit}`"
            ));
        }
        let added = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|raw| !raw.is_empty())
            .map(|raw| {
                std::str::from_utf8(raw)
                    .map(str::to_string)
                    .map_err(|_| "legacy archive introduction path is not UTF-8".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (id, subtree) in &repo_subtrees {
            let prefix = format!("{subtree}/");
            if added
                .iter()
                .any(|path| path == subtree || path.starts_with(&prefix))
            {
                candidates
                    .entry(id.clone())
                    .or_default()
                    .insert(commit.clone());
            }
        }
        let commit_parents: Vec<String> =
            git_output(root, &["rev-list", "--parents", "-n", "1", &commit])
                .unwrap_or_default()
                .split_whitespace()
                .skip(1)
                .map(str::to_string)
                .collect();
        parents.insert(commit, commit_parents);
    }

    for entry in &baseline.entries {
        let current = snapshots.get(&entry.id).ok_or_else(|| {
            format!(
                "legacy archive `{}` has no captured baseline subtree",
                entry.id
            )
        })?;
        let repo_subtree = repo_subtrees
            .get(&entry.id)
            .ok_or_else(|| format!("legacy archive `{}` has no repository subtree", entry.id))?;
        let mut anchors = BTreeSet::new();
        for commit in candidates.get(&entry.id).into_iter().flatten() {
            let parent_has_subtree = parents.get(commit).into_iter().flatten().any(|parent| {
                !git_tree_snapshot(root, parent, repo_subtree)
                    .unwrap_or_default()
                    .is_empty()
            });
            if parent_has_subtree {
                continue;
            }
            if git_tree_snapshot(root, commit, repo_subtree).is_ok_and(|tree| &tree == current) {
                anchors.insert(commit.clone());
            }
        }
        if anchors.len() != 1 || !anchors.contains(&entry.introduction_commit) {
            return Err(format!(
                "legacy archive `{}` requires its one baseline-bound pre-cutoff introduction anchor, found {}",
                entry.id,
                anchors.len()
            ));
        }
    }
    Ok(())
}

fn legacy_baseline_entry<'a>(
    baseline: &'a LegacyArchiveBaselineV1,
    archived_id: &str,
) -> Result<&'a LegacyArchiveBaselineEntryV1, String> {
    baseline
        .entries
        .iter()
        .find(|entry| entry.id == archived_id)
        .ok_or_else(|| format!("legacy archive `{archived_id}` is not enumerated by the baseline"))
}

fn validate_legacy_archive_baseline_bytes(
    baseline_bytes: &[u8],
) -> Result<(LegacyArchiveBaselineV1, String), String> {
    if baseline_bytes.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
        return Err("legacy archive baseline exceeds the change artifact byte limit".into());
    }
    let baseline: LegacyArchiveBaselineV1 = serde_json::from_slice(baseline_bytes)
        .map_err(|error| format!("invalid legacy archive baseline: {error}"))?;
    if baseline.schema_version != 1 || baseline.domain != "specsync.legacy-archive-baseline.v1" {
        return Err("unsupported legacy archive baseline schema or domain".into());
    }
    if baseline.entries.len() > MAX_ACCEPTANCE_ENTRIES {
        return Err("legacy archive baseline exceeds the entry limit".into());
    }
    let canonical = json_content(&baseline)?;
    if !bytes_match_canonical_json(baseline_bytes, canonical.as_bytes()) {
        return Err("legacy archive baseline must use canonical persisted JSON bytes".into());
    }
    let mut previous: Option<(&str, &str)> = None;
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for entry in &baseline.entries {
        let key = (entry.id.as_str(), entry.archive_path.as_str());
        if previous.is_some_and(|prior| prior >= key) {
            return Err(
                "legacy archive baseline entries must be strictly sorted and unique".into(),
            );
        }
        previous = Some(key);
        if !ids.insert(entry.id.as_str()) || !paths.insert(entry.archive_path.as_str()) {
            return Err("legacy archive baseline IDs and archive paths must each be unique".into());
        }
        validate_sha256_digest(&entry.subtree_digest, "legacy archive subtree digest")?;
        let portable = strict_portable_relative_path(&entry.archive_path)?;
        if portable != entry.archive_path || !portable.starts_with(&format!("{ARCHIVE_PATH}/")) {
            return Err(format!(
                "legacy archive baseline path is not canonical: `{}`",
                entry.archive_path
            ));
        }
    }
    let mut digest = FramedDigest::new(LEGACY_BASELINE_DOMAIN);
    digest.frame(b"ledger", baseline_bytes);
    let baseline_digest = digest.finish();
    Ok((baseline, baseline_digest))
}

fn ensure_git_ancestor(
    root: &Path,
    ancestor: &str,
    descendant: &str,
    label: &str,
) -> Result<(), String> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", ancestor, descendant])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to inspect {label} ancestry: {error}"))?;
    if !status.success() {
        return Err(format!(
            "`{ancestor}` is not an ancestor of {label} `{descendant}`"
        ));
    }
    Ok(())
}

fn legacy_archive_subtree_digest(
    snapshot: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<String, String> {
    if snapshot.is_empty() {
        return Err("legacy archive baseline subtree must not be empty".into());
    }
    let mut digest = FramedDigest::new(LEGACY_SUBTREE_DOMAIN);
    digest.frame(b"schema-version", &1_u32.to_be_bytes());
    for (path, (mode, payload)) in snapshot {
        let kind: &[u8] = match mode {
            0o100644 | 0o100755 => b"file",
            0o120000 => {
                let target = std::str::from_utf8(payload)
                    .map_err(|_| format!("legacy archive symlink `{path}` is not UTF-8"))?;
                validate_portable_symlink_target(target)?;
                b"symlink"
            }
            _ => {
                return Err(format!(
                    "legacy archive subtree contains unsupported mode `{mode:o}` at `{path}`"
                ));
            }
        };
        digest.frame(b"entry", b"");
        digest.frame(b"path", path.as_bytes());
        digest.frame(b"kind", kind);
        digest.frame(b"mode", &mode.to_be_bytes());
        digest.frame(b"payload", payload);
    }
    Ok(digest.finish())
}

#[cfg(test)]
fn archive_workspace_snapshot(
    root: &Path,
    workspace: &Path,
    project_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let mut workspace_paths = BTreeSet::new();
    for entry in walkdir::WalkDir::new(workspace).follow_links(false) {
        let entry = entry.map_err(|error| format!("failed to inspect legacy archive: {error}"))?;
        if entry.path() != workspace && !entry.file_type().is_dir() {
            workspace_paths.insert(format!(
                "{project_subtree}/{}",
                strict_portable_project_path(workspace, entry.path())?
            ));
        }
    }
    let scopes = BTreeSet::from([project_subtree.to_string()]);
    let (project_paths, evidence) =
        stable_discovered_evidence(root, Some(&scopes), &workspace_paths, false)?;
    archive_snapshot_from_evidence(&project_paths, &evidence, project_subtree)
}

fn archive_snapshot_from_evidence(
    project_paths: &[String],
    evidence: &GitEvidence,
    project_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let prefix = format!("{project_subtree}/");
    let mut snapshot = BTreeMap::new();
    for project_path in project_paths {
        let Some(relative) = project_path.strip_prefix(&prefix) else {
            continue;
        };
        let entry = evidence.entry(project_path)?;
        match entry.kind {
            AcceptanceInputKind::File | AcceptanceInputKind::Symlink => {
                snapshot.insert(relative.to_string(), (entry.mode, entry.payload.clone()));
            }
            AcceptanceInputKind::Missing => {}
            AcceptanceInputKind::Gitlink | AcceptanceInputKind::NonFile => {
                return Err(format!(
                    "legacy archive contains unsupported non-file entry {project_path}"
                ));
            }
        }
    }
    Ok(snapshot)
}

fn git_tree_snapshot(
    root: &Path,
    commit: &str,
    repo_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let pathspec = format!(":(top,literal){repo_subtree}");
    let output = Command::new("git")
        .args([
            "ls-tree",
            "-r",
            "-z",
            "--full-name",
            commit,
            "--",
            &pathspec,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect legacy archive tree: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect legacy archive tree at `{commit}`"
        ));
    }
    let prefix = format!("{repo_subtree}/");
    let mut entries = Vec::new();
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|raw| !raw.is_empty())
    {
        let raw = std::str::from_utf8(raw)
            .map_err(|_| "legacy archive tree contains non-UTF-8 metadata".to_string())?;
        let (metadata, path) = raw
            .split_once('\t')
            .ok_or_else(|| format!("invalid legacy archive tree entry `{raw}`"))?;
        let mut fields = metadata.split_whitespace();
        let mode = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no mode".to_string())?;
        let kind = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no kind".to_string())?;
        let object = fields
            .next()
            .ok_or_else(|| "legacy archive tree entry has no object".to_string())?;
        let mode = u32::from_str_radix(mode, 8)
            .map_err(|_| "legacy archive tree entry has invalid mode".to_string())?;
        if kind != "blob" || !matches!(mode, 0o100644 | 0o100755 | 0o120000) {
            return Err(format!(
                "legacy archive tree contains unsupported `{kind}` mode `{mode:o}`"
            ));
        }
        let relative = path
            .strip_prefix(&prefix)
            .ok_or_else(|| format!("legacy archive tree path `{path}` escaped its subtree"))?;
        entries.push((
            strict_portable_relative_path(relative)?,
            mode,
            object.to_string(),
        ));
    }
    let objects: Vec<_> = entries
        .iter()
        .map(|(_, _, object)| object.as_str())
        .collect();
    let blobs = git_blob_bytes_batch(root, &objects)?;
    let mut snapshot = BTreeMap::new();
    for ((relative, mode, _object), bytes) in entries.into_iter().zip(blobs) {
        snapshot.insert(relative, (mode, bytes));
    }
    Ok(snapshot)
}

fn git_blob_bytes_batch(root: &Path, objects: &[&str]) -> Result<Vec<Vec<u8>>, String> {
    if objects.is_empty() {
        return Ok(Vec::new());
    }
    let mut input = Vec::new();
    for object in objects {
        if object.is_empty() || object.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(format!("invalid legacy archive blob object `{object}`"));
        }
        input.extend_from_slice(object.as_bytes());
        input.push(b'\n');
    }
    let output = run_git_required(
        root,
        &["cat-file", "--batch"],
        Some(input),
        MAX_GIT_EVIDENCE_PAYLOAD_BYTES + MAX_GIT_EVIDENCE_PATH_BYTES,
    )?;
    let mut cursor = 0;
    let mut blobs = Vec::with_capacity(objects.len());
    for expected in objects {
        let header_end = output[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
            .ok_or_else(|| "legacy archive batch output has no header terminator".to_string())?;
        let header = std::str::from_utf8(&output[cursor..header_end])
            .map_err(|_| "legacy archive batch output has non-UTF-8 metadata".to_string())?;
        let mut fields = header.split_whitespace();
        let object = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no object".to_string())?;
        let kind = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no kind".to_string())?;
        let size = fields
            .next()
            .ok_or_else(|| "legacy archive batch output has no size".to_string())?
            .parse::<usize>()
            .map_err(|_| "legacy archive batch output has an invalid size".to_string())?;
        if fields.next().is_some() || object != *expected || kind != "blob" {
            return Err(format!(
                "unexpected legacy archive batch header `{header}` for `{expected}`"
            ));
        }
        let payload_start = header_end + 1;
        let payload_end = payload_start
            .checked_add(size)
            .ok_or_else(|| "legacy archive blob size overflowed".to_string())?;
        if payload_end >= output.len() || output[payload_end] != b'\n' {
            return Err(format!(
                "legacy archive batch output is truncated for blob `{expected}`"
            ));
        }
        blobs.push(output[payload_start..payload_end].to_vec());
        cursor = payload_end + 1;
    }
    if cursor != output.len() {
        return Err("legacy archive batch output has unexpected trailing bytes".into());
    }
    Ok(blobs)
}

fn validate_accepted_inputs_recursive(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
) -> Result<AcceptedInputValidity, String> {
    if let Some(result) = memo.get(&record.id) {
        return result.clone();
    }
    if !matches!(record.state, ChangeState::Accepted | ChangeState::Archived) {
        return Err(format!(
            "successor `{}` is not accepted or archived",
            record.id
        ));
    }
    if !visiting.insert(record.id.clone()) {
        return Err(format!(
            "semantic succession cycle detected at `{}`",
            record.id
        ));
    }
    let result = (|| {
        let verification = authenticate_accepted_evidence(root, record)?;
        let expected_digest = verification
            .acceptance_input_digest
            .as_deref()
            .ok_or_else(|| {
                format!(
                    "accepted change is missing current delivery-input evidence; run `specsync change reopen {}` to record fresh acceptance evidence",
                    record.id
                )
            })?;
        let signed = if let Some(manifest) = &verification.acceptance_manifest {
            manifest.clone()
        } else {
            let legacy_current =
                acceptance_input_digest(root, &terminal_delivery_projection(record), &[])?;
            if legacy_current == expected_digest {
                return Ok(AcceptedInputValidity::Exact);
            }
            reconstruct_legacy_acceptance_manifest(root, record, expected_digest)?
        };
        let current = acceptance_manifest_with_signed_owners(
            root,
            &terminal_delivery_projection(record),
            &[],
            &signed,
        )?;
        let current_by_path: BTreeMap<_, _> = current
            .entries
            .iter()
            .map(|entry| (entry.path.as_str(), entry))
            .collect();
        let mut successor_covered = false;
        for expected in &signed.entries {
            let current = current_by_path.get(expected.path.as_str()).ok_or_else(|| {
                format!(
                    "delivery input `{}` disappeared from the current inventory; restore the file or run `specsync change reopen {}` to re-verify the accepted change",
                    expected.path, record.id
                )
            })?;
            if expected.kind == current.kind
                && expected.mode == current.mode
                && expected.payload_digest == current.payload_digest
                && expected.entry_digest == current.entry_digest
            {
                continue;
            }
            if expected
                .owners
                .iter()
                .any(|owner| owner.starts_with("@exact:"))
            {
                if expected.path == SEQUENCE_PATH
                    && later_sequence_owner_covers_historical_input(
                        root, record, records, visiting, memo,
                    )?
                {
                    successor_covered = true;
                    continue;
                }
                return Err(format!(
                    "exact-only delivery input `{}` changed after acceptance and requires an audited reopen; run `specsync change reopen {}` to re-verify the accepted change",
                    expected.path, record.id
                ));
            }
            for owner in &expected.owners {
                let mut covered = false;
                let mut stale_covering_successors = BTreeSet::new();
                for candidate in records.values() {
                    if candidate.id == record.id
                        || !matches!(
                            candidate.state,
                            ChangeState::Accepted | ChangeState::Archived
                        )
                        || candidate.no_spec_change
                        || !happens_after(candidate, record)
                    {
                        continue;
                    }
                    let candidate_verification =
                        match authenticate_accepted_evidence(root, candidate) {
                            Ok(verification) => verification,
                            Err(_) => continue,
                        };
                    let candidate_manifest = if let Some(manifest) =
                        candidate_verification.acceptance_manifest.as_ref()
                    {
                        manifest.clone()
                    } else {
                        match resolved_acceptance_manifest(root, candidate) {
                            Ok(manifest) => manifest,
                            Err(_) => continue,
                        }
                    };
                    let tuple = if let Some(evidence) =
                        candidate_verification.semantic_succession.as_ref()
                    {
                        evidence
                            .tuples
                            .iter()
                            .find(|tuple| {
                                semantic_tuple_matches_obligation(
                                    tuple, &record.id, expected, owner,
                                )
                            })
                            .cloned()
                    } else if candidate_verification.acceptance_manifest.is_none() {
                        legacy_semantic_successor_tuple(root, record, expected, owner, candidate)
                            .ok()
                            .flatten()
                    } else {
                        None
                    };
                    let Some(tuple) = tuple else {
                        continue;
                    };
                    if !candidate_manifest.entries.iter().any(|entry| {
                        entry.path == expected.path
                            && entry.entry_digest == tuple.successor_entry_digest
                            && entry.owners.contains(owner)
                    }) || !semantic_acceptance_item_exists_for_module(root, candidate, owner)
                        .unwrap_or(false)
                        || !semantic_tuple_transition_is_valid(root, candidate, &tuple)
                            .unwrap_or(false)
                    {
                        continue;
                    }
                    if validate_accepted_inputs_recursive(root, candidate, records, visiting, memo)
                        .is_ok()
                    {
                        covered = true;
                        successor_covered = true;
                        break;
                    }
                    stale_covering_successors.insert(candidate.id.clone());
                }
                if !covered {
                    return Err(stale_input_remediation_reason(
                        &record.id,
                        &expected.path,
                        owner,
                        &stale_covering_successors,
                    ));
                }
            }
        }
        Ok(if successor_covered {
            AcceptedInputValidity::SuccessorCovered
        } else {
            AcceptedInputValidity::Exact
        })
    })();
    visiting.remove(&record.id);
    memo.insert(record.id.clone(), result.clone());
    result
}

fn stale_input_remediation_reason(
    record_id: &str,
    path: &str,
    owner: &str,
    stale_covering_successors: &BTreeSet<String>,
) -> String {
    if stale_covering_successors.is_empty() {
        format!(
            "delivery input `{path}` (owner `{owner}`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen {record_id}` to re-verify the accepted change"
        )
    } else {
        let successors = stale_covering_successors
            .iter()
            .map(|id| format!("`{id}`"))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "delivery input `{path}` (owner `{owner}`) changed after acceptance; covering successor change(s) {successors} have stale delivery-input evidence of their own; verify and accept a covering successor, or run `specsync change reopen {record_id}` to re-verify the accepted change"
        )
    }
}

fn later_sequence_owner_covers_historical_input(
    root: &Path,
    predecessor: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
) -> Result<bool, String> {
    validate_change_sequences(root)?;
    let Some(ledger) = load_change_sequence_ledger(root)? else {
        return Ok(false);
    };
    let Some(owner) = records.get(&ledger.id) else {
        return Ok(false);
    };
    if owner.id == predecessor.id
        || !matches!(owner.state, ChangeState::Accepted | ChangeState::Archived)
        || !happens_after(owner, predecessor)
    {
        return Ok(false);
    }
    let verification = match authenticate_accepted_evidence(root, owner) {
        Ok(verification) => verification,
        Err(_) => return Ok(false),
    };
    let manifest = if let Some(manifest) = verification.acceptance_manifest.as_ref() {
        manifest
    } else {
        return Ok(false);
    };
    if !manifest.entries.iter().any(|entry| {
        entry.path == SEQUENCE_PATH
            && entry
                .owners
                .iter()
                .any(|owner| owner.starts_with("@exact:"))
    }) {
        return Ok(false);
    }
    Ok(validate_accepted_inputs_recursive(root, owner, records, visiting, memo).is_ok())
}

fn semantic_tuple_matches_obligation(
    tuple: &SemanticSuccessionTupleV1,
    predecessor_id: &str,
    predecessor_entry: &AcceptanceInputEntryV1,
    owner: &str,
) -> bool {
    tuple.predecessor_id == predecessor_id
        && tuple.path == predecessor_entry.path
        && tuple.module == owner
        && tuple.predecessor_entry_digest == predecessor_entry.entry_digest
}

fn legacy_semantic_successor_tuple(
    root: &Path,
    predecessor: &ChangeRecord,
    predecessor_entry: &AcceptanceInputEntryV1,
    owner: &str,
    candidate: &ChangeRecord,
) -> Result<Option<SemanticSuccessionTupleV1>, String> {
    let obligation_exists = candidate.supersedes.iter().any(|edge| {
        edge.predecessor_id == predecessor.id
            && edge.obligations.iter().any(|obligation| {
                obligation.path == predecessor_entry.path
                    && obligation.module == owner
                    && obligation.predecessor_entry_digest == predecessor_entry.entry_digest
            })
    });
    if !obligation_exists || !semantic_acceptance_item_exists_for_module(root, candidate, owner)? {
        return Ok(None);
    }
    let manifest = resolved_acceptance_manifest(root, candidate)?;
    let Some(successor_entry) = manifest.entries.iter().find(|entry| {
        entry.path == predecessor_entry.path
            && entry.owners.iter().any(|entry_owner| entry_owner == owner)
    }) else {
        return Ok(None);
    };
    Ok(Some(SemanticSuccessionTupleV1 {
        predecessor_id: predecessor.id.clone(),
        path: predecessor_entry.path.clone(),
        module: owner.to_string(),
        predecessor_entry_digest: predecessor_entry.entry_digest.clone(),
        successor_entry_digest: successor_entry.entry_digest.clone(),
    }))
}

fn terminal_delivery_projection(record: &ChangeRecord) -> ChangeRecord {
    let mut projection = record.clone();
    if projection.state == ChangeState::Archived {
        projection.state = ChangeState::Accepted;
    }
    projection
}

/// The `CHG-NNNN` ordinal of a change ID, when it has one.
fn change_sequence_number(id: &str) -> Option<u32> {
    id.strip_prefix("CHG-")?
        .split('-')
        .next()?
        .parse::<u32>()
        .ok()
}

/// Rejects two distinct changes that claimed the same `CHG-NNNN` ordinal from the
/// same base commit.
///
/// Independent worktrees and clones allocate from their own sequence ledger, so
/// two can hand out the same ordinal to different work. They only become visible
/// to each other once both land, and by then the ordinal no longer identifies a
/// single change. Differing base commits are legitimate — a branch re-cut from a
/// later tip may reuse an ordinal — so only a shared base is a genuine collision.
fn ensure_no_sequence_collision(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let Some(ordinal) = change_sequence_number(&record.id) else {
        return Ok(());
    };
    // An unknown base cannot establish that two changes were cut from the same
    // point, so it never raises a collision rather than guessing.
    let Some(base) = record.base_commit.as_deref() else {
        return Ok(());
    };
    let conflicting: Vec<String> = list_all_changes_checked(root)?
        .values()
        .filter(|other| {
            other.id != record.id
                && other.base_commit.as_deref() == Some(base)
                && change_sequence_number(&other.id) == Some(ordinal)
        })
        .map(|other| other.id.clone())
        .collect();
    if conflicting.is_empty() {
        return Ok(());
    }
    Err(format!(
        "change ordinal CHG-{ordinal:04} is claimed by {} and {} from the same base commit {base}. \
         Two workspaces allocated it independently while ordinals were still minted. Acknowledge the \
         collision in `.specsync/change-sequence.json` once both are accepted or archived, or recreate \
         one of them with `specsync change new` — a change created now claims no ordinal and so cannot \
         collide.",
        record.id,
        conflicting.join(", ")
    ))
}

fn list_all_changes_checked(root: &Path) -> Result<BTreeMap<String, ChangeRecord>, String> {
    if let Some(records) = read_scope_value(root, |scope| scope.all_records.clone()) {
        return records;
    }
    let result = list_all_changes_uncached(root);
    update_read_scope(root, |scope| {
        scope.all_records = Some(result.clone());
    });
    result
}

fn list_all_changes_uncached(root: &Path) -> Result<BTreeMap<String, ChangeRecord>, String> {
    let mut records = BTreeMap::new();
    for record in list_changes_checked(root)? {
        records.insert(record.id.clone(), record);
    }
    let archive = root.join(ARCHIVE_PATH);
    let entries = match fs::read_dir(&archive) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(format!("failed to read archived changes: {error}")),
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read archive entry: {error}"))?;
        if !entry
            .file_type()
            .map_err(|error| format!("failed to inspect archive entry: {error}"))?
            .is_dir()
        {
            continue;
        }
        let path = entry.path().join("state.json");
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound
                    && (is_positive_legacy_tombstone(&entry.path())
                        || is_untrackable_husk(&entry.path())) =>
            {
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "failed to read archived state {}: {error}",
                    path.display()
                ));
            }
        };
        let record: ChangeRecord = serde_json::from_str(&content)
            .map_err(|error| format!("invalid archived state {}: {error}", path.display()))?;
        validate_loaded_change(&record, &record.id, &path)?;
        validate_workflow_version_history(root, &record)?;
        if records.insert(record.id.clone(), record.clone()).is_some() {
            return Err(format!(
                "change `{}` exists in multiple active/archive locations",
                record.id
            ));
        }
    }
    Ok(records)
}

/// True when `path` is a directory holding no regular file at any depth.
///
/// Git cannot track an empty directory. Checking out a commit that predates a
/// change package therefore removes every tracked file in it and strands the
/// directories — a husk that `git status` reports as clean. A husk is the
/// absence of a change, not a damaged one, so enumeration skips it.
///
/// A directory that does hold files but no `state.json` is damaged, not a husk,
/// and its caller still refuses it: this predicate cannot be satisfied by
/// ignoring corruption.
fn is_untrackable_husk(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            return false;
        };
        if kind.is_dir() {
            if !is_untrackable_husk(&entry.path()) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

/// Whether an archive directory name carries a `CHG-NNNN` lifecycle ordinal, dated or not.
///
/// `2026-08-19-CHG-0001-foo` and `CHG-0001-foo` both do. The previous test was
/// `name.contains("-CHG-")`, which silently missed the undated form.
fn name_carries_a_lifecycle_ordinal(name: &str) -> bool {
    let segments: Vec<&str> = name.split('-').collect();
    segments.windows(2).any(|pair| {
        pair[0] == "CHG" && pair[1].len() >= 4 && pair[1].chars().all(|c| c.is_ascii_digit())
    })
}

/// Whether a directory holds a regular file at any depth.
///
/// No depth counter: archive packages are a handful of levels, and a symlink loop cannot occur
/// because `read_dir` reports a symlink as neither file nor directory here.
fn directory_holds_a_regular_file(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    entries.flatten().any(|entry| match entry.file_type() {
        Ok(kind) if kind.is_file() => true,
        Ok(kind) if kind.is_dir() => directory_holds_a_regular_file(&entry.path()),
        _ => false,
    })
}

/// A pre-lifecycle archive directory: a `deltas/`-only record with no lifecycle state.
///
/// These are skipped by enumeration. A real lifecycle package that has *lost* its state must
/// NOT be skipped — that is corruption, and hiding it is how a damaged archive reads as an
/// absent one.
///
/// Three signals say "real package, therefore not a tombstone". They are a UNION on purpose:
/// each one only ever moves a directory from skipped to refused, so adding one cannot weaken
/// the gate.
///
/// 1. It holds a regular file outside `deltas/`. All 159 archived packages do; a tombstone
///    holds none. Strictly stronger than the four-file allowlist it joins — a package that
///    kept only `plan.md` was previously misread.
/// 2. It holds one of the four lifecycle marker files.
/// 3. Its name carries a lifecycle ordinal.
///
/// Signal 3 was `name.contains("-CHG-")`, which is wrong today: an undated package named
/// `CHG-0001-foo` does not contain that substring, so a real archived change stripped of its
/// lifecycle files vanished silently. `name_carries_a_lifecycle_ordinal` accepts both forms.
///
/// Signal 3 is also the one that cannot survive an identity scheme without ordinals, and 1 and
/// 2 do not fully replace it: a dated package reduced to `deltas/auth.md` alone is refused
/// today *only* because of its name. Retiring the ordinal therefore needs a provenance signal
/// to take signal 3's place — git history of the package's `state.json` is the obvious
/// candidate. That is a decision for the identity migration, not something to quietly drop here.
fn is_positive_legacy_tombstone(path: &Path) -> bool {
    let holds_a_file_outside_deltas = fs::read_dir(path).is_ok_and(|entries| {
        entries.flatten().any(|entry| match entry.file_type() {
            Ok(kind) if kind.is_file() => true,
            Ok(kind) if kind.is_dir() && entry.file_name() != "deltas" => {
                directory_holds_a_regular_file(&entry.path())
            }
            _ => false,
        })
    });
    let names_a_lifecycle_ordinal = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(name_carries_a_lifecycle_ordinal);
    if holds_a_file_outside_deltas
        || names_a_lifecycle_ordinal
        || [
            "change.md",
            "approvals.json",
            "verification.json",
            "accepted-state.json",
        ]
        .iter()
        .any(|name| path.join(name).exists())
    {
        return false;
    }
    let deltas = path.join("deltas");
    let Ok(entries) = fs::read_dir(deltas) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry.file_type().is_ok_and(|file_type| file_type.is_file())
            && entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("md")
    })
}

fn append_approval(
    root: &Path,
    record: &ChangeRecord,
    gate: &str,
    actor: Option<String>,
    digest: String,
    note: Option<String>,
) -> Result<(), String> {
    let mut ledger = load_approvals(root, record)?;
    let approved_scope = (gate == "definition" && record.workflow_version >= 2)
        .then(|| approved_scope(root, record))
        .transpose()?;
    if gate == "definition" && approved_scope.is_some() {
        // A direct renewed approval supersedes the one-time legacy adoption.
        // Retaining both would make the new approval intentionally ambiguous.
        ledger.scope_adoptions.clear();
    }
    // Only a definition gate approves wording. Closing and finalization gates bind delivery
    // evidence, and claiming they reviewed delta bodies would be a lie recorded in the ledger.
    let approved_delta_digests = (gate == "definition")
        .then(|| delta_body_digests(root, record))
        .transpose()?;
    ledger.approvals.push(ApprovalRecord {
        gate: gate.into(),
        actor: resolve_actor(root, actor)?,
        timestamp: now(),
        digest,
        note,
        definition_pair: None,
        approved_scope,
        scope_migration: None,
        approved_delta_digests,
    });
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
}

fn append_portable_definition_approval_v501(
    root: &Path,
    record: &ChangeRecord,
    actor: Option<String>,
    note: Option<String>,
) -> Result<(), String> {
    let (current_digest, legacy_digest, correction_prefix_digest) =
        portable_definition_digest_pair_v501(root, record)?;
    let actor = resolve_actor(root, actor)?;
    let timestamp = now();
    let path = change_dir(root, &record.id).join("approvals.json");
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut document: serde_json::Value = serde_json::from_str(&content)
        .map_err(|error| format!("invalid approval ledger {}: {error}", path.display()))?;
    let ledger: ApprovalLedger = serde_json::from_value(document.clone())
        .map_err(|error| format!("invalid approval ledger {}: {error}", path.display()))?;
    let event_index = ledger.approvals.len() as u64;
    let pair_id = definition_approval_pair_id(
        record,
        event_index,
        &actor,
        timestamp,
        &correction_prefix_digest,
        &current_digest,
        &legacy_digest,
    );
    let metadata = |role| DefinitionApprovalPairV1 {
        schema_version: 1,
        projection: PORTABLE_DEFINITION_PROJECTION_V501.into(),
        pair_id: pair_id.clone(),
        role,
        change_id: record.id.clone(),
        correction_count: record.correction_count,
        correction_prefix_digest: correction_prefix_digest.clone(),
        current_digest: current_digest.clone(),
        legacy_digest: legacy_digest.clone(),
        event_index,
    };
    let current = ApprovalRecord {
        gate: "definition".into(),
        actor: actor.clone(),
        timestamp,
        digest: current_digest.clone(),
        note,
        definition_pair: Some(metadata(DefinitionApprovalPairRole::Current)),
        approved_scope: None,
        scope_migration: None,
        approved_delta_digests: None,
    };
    let legacy = ApprovalRecord {
        gate: "definition".into(),
        actor,
        timestamp,
        digest: legacy_digest.clone(),
        note: Some("Portable SpecSync 5.0.1 definition projection".into()),
        definition_pair: Some(metadata(DefinitionApprovalPairRole::Legacy)),
        approved_scope: None,
        scope_migration: None,
        approved_delta_digests: None,
    };
    let approvals = document
        .get_mut("approvals")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| "approval ledger approvals must be an array".to_string())?;
    approvals.push(serde_json::to_value(current).map_err(|error| error.to_string())?);
    approvals.push(serde_json::to_value(legacy).map_err(|error| error.to_string())?);
    write_prepared_files(root, &[(path, json_content(&document)?)])
}

fn definition_approval_pair_id(
    record: &ChangeRecord,
    event_index: u64,
    actor: &str,
    timestamp: u64,
    correction_prefix_digest: &str,
    current_digest: &str,
    legacy_digest: &str,
) -> String {
    let mut digest = FramedDigest::new(DEFINITION_APPROVAL_PAIR_DOMAIN);
    digest.frame(b"change-id", record.id.as_bytes());
    digest.frame(b"event-index", event_index.to_string().as_bytes());
    digest.frame(
        b"correction-count",
        record.correction_count.to_string().as_bytes(),
    );
    digest.frame(b"correction-prefix", correction_prefix_digest.as_bytes());
    digest.frame(b"actor", actor.as_bytes());
    digest.frame(b"timestamp", timestamp.to_string().as_bytes());
    digest.frame(b"current", current_digest.as_bytes());
    digest.frame(b"legacy", legacy_digest.as_bytes());
    digest.finish()
}

fn resolve_actor(root: &Path, actor: Option<String>) -> Result<String, String> {
    actor
        .or_else(|| git_output(root, &["config", "user.name"]))
        .or_else(|| std::env::var("USER").ok())
        .or_else(|| std::env::var("USERNAME").ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "approval actor is unknown; pass --actor <name>".to_string())
}

fn load_approvals(root: &Path, record: &ChangeRecord) -> Result<ApprovalLedger, String> {
    let path = find_change_dir(root, &record.id)?.join("approvals.json");
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| {
        let message = error.to_string();
        if message.contains("missing field `stale_acceptance_input_digest`")
            || message.contains("missing field `current_acceptance_input_digest`")
        {
            format!("{message}; run `specsync migrate 5.0` to backfill 5.0.1-era reopening records")
        } else {
            message
        }
    })
}

/// Accepted acceptance-input entries for a change, or an empty list when it has no
/// verification evidence yet.
///
/// `change supersede --digest` requires a `specsync.acceptance-entry.v1` digest and
/// nothing emitted one, so the only way to obtain it was to open
/// `verification.json`, walk `acceptance_manifest.entries`, match on `path`, and
/// read `entry_digest` by hand — at the moment a recovery has already gone wrong.
/// Exposing the entries lets `change show --json` answer the question the CLI asks.
///
/// Missing or unreadable evidence yields an empty list rather than an error: this
/// is a lookup for machine consumers, not a gate.
pub fn acceptance_entries(root: &Path, record: &ChangeRecord) -> Vec<AcceptanceInputEntryV1> {
    load_verification(root, record)
        .ok()
        .and_then(|verification| verification.acceptance_manifest)
        .map(|manifest| manifest.entries)
        .unwrap_or_default()
}

fn load_verification(root: &Path, record: &ChangeRecord) -> Result<VerificationRecord, String> {
    let path = find_change_dir(root, &record.id)?.join("verification.json");
    let content =
        fs::read_to_string(&path).map_err(|_| "verification evidence is missing".to_string())?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid verification evidence: {error}"))
}

fn record_verification_attempt(
    root: &Path,
    record: &ChangeRecord,
    verification: &VerificationRecord,
) -> Result<(), String> {
    let history_path = change_dir(root, &record.id).join("verification-attempts.json");
    let mut history = if history_path.exists() {
        let content = fs::read_to_string(&history_path)
            .map_err(|error| format!("failed to read verification attempt history: {error}"))?;
        let history: VerificationAttemptLedger = serde_json::from_str(&content)
            .map_err(|error| format!("invalid verification attempt history: {error}"))?;
        if history.schema_version != 1 {
            return Err(format!(
                "unsupported verification attempt history schema version {}",
                history.schema_version
            ));
        }
        history
    } else {
        VerificationAttemptLedger {
            schema_version: 1,
            attempts: Vec::new(),
        }
    };
    history.attempts.push(verification.clone());
    write_prepared_files(
        root,
        &[
            (
                change_dir(root, &record.id).join("verification.json"),
                json_content(verification)?,
            ),
            (history_path, json_content(&history)?),
        ],
    )
}

/// The coverage gate is the first hard failure most projects meet, and the way
/// out of it — opening a change workspace, optionally without a spec change —
/// is not discoverable from the bare list of paths. Name the command, and say
/// why an `ignored_paths` entry did not apply when one appears to cover a
/// reported path: protected SDD policy files and the configured specs tree are
/// structurally meaningful and cannot be ignored away.
/// How many `--path` flags the coverage remediation spells out before
/// summarizing the remainder.
const UNCOVERED_PATH_FLAG_LIMIT: usize = 12;

fn uncovered_paths_error(policy: &SddPolicy, paths: &[String]) -> String {
    let mut message = format!(
        "meaningful changed paths are not covered by an active change: {}",
        paths.join(", ")
    );
    // One `--path` per file on one line is runnable but unreadable once a
    // branch touches more than a handful: a wide refactor produced a single
    // line over 8000 characters. Name enough to paste for the common case and
    // say how to get the rest, rather than emitting a wall.
    let mut path_flags = String::new();
    for path in paths.iter().take(UNCOVERED_PATH_FLAG_LIMIT) {
        path_flags.push_str(&format!(" --path {path}"));
    }
    let remaining = paths.len().saturating_sub(UNCOVERED_PATH_FLAG_LIMIT);
    message.push_str(&format!(
        "\n  cover them: specsync change new \"<summary>\" --kind fix --spec <module>{path_flags}"
    ));
    if remaining > 0 {
        message.push_str(&format!(
            "\n  ... and {remaining} more path(s) — add a `--path` for each, or declare a covering prefix such as `--path src/`"
        ));
    }
    message.push_str(
        "\n  no spec text changes: add --no-spec-change --rationale \"<why>\" to that command",
    );
    let shadowed: Vec<&str> = paths
        .iter()
        .filter(|path| {
            policy
                .ignored_paths
                .iter()
                .any(|scope| path_matches_scope(path, scope))
        })
        .map(String::as_str)
        .collect();
    if !shadowed.is_empty() {
        message.push_str(&format!(
            "\n  note: an ignored_paths entry covers {}, but SDD policy files and the configured specs tree are always meaningful",
            shadowed.join(", ")
        ));
    }
    message
}

fn uncovered_meaningful_paths(
    root: &Path,
    policy: &SddPolicy,
    records: &[ChangeRecord],
) -> Result<Vec<String>, String> {
    // A brand-new repository has no meaningful comparison base yet. Only allow
    // the clean-tree shortcut for its first commit; a clean feature branch can
    // still contain committed delivery changes that require lifecycle coverage.
    if !is_ci_project(root)
        && git_worktree_is_clean(root) == Some(true)
        && git_commit_count(root) == Some(1)
    {
        return Ok(Vec::new());
    }
    let mut args = vec!["diff", "--name-only", "--relative"];
    let base = pull_request_diff_base(root, records);
    args.push(&base);
    let output = git_output_allow_empty(root, &args)
        .ok_or_else(|| "unable to inspect changed paths for SDD coverage".to_string())?;
    let mut changed: BTreeSet<String> = output.lines().map(str::to_string).collect();
    if !is_ci_project(root) {
        for command in [
            vec!["diff", "--name-only", "--relative"],
            vec!["diff", "--cached", "--name-only", "--relative"],
            vec!["ls-files", "--others", "--exclude-standard"],
        ] {
            let output = git_output_allow_empty(root, &command).ok_or_else(|| {
                "unable to inspect local changed paths for SDD coverage".to_string()
            })?;
            changed.extend(output.lines().map(str::to_string));
        }
    }
    for path in bootstrap_exempt_paths(root, &base) {
        changed.remove(&path);
    }
    // Same-PR finalize archives the covering change before merge. Product paths
    // remain in the delivery vs base, but the archive package is on the tip.
    // Count only archived records whose package appears in this delivery — not
    // every historical archive (that would silently cover unrelated later PRs).
    let delivery_archives = archived_records_in_delivery(root, &changed)?;
    let mut covering: Vec<&ChangeRecord> = records.iter().collect();
    for archived in &delivery_archives {
        if !covering.iter().any(|record| record.id == archived.id) {
            covering.push(archived);
        }
    }
    let covered: Vec<&str> = covering
        .iter()
        .filter(|record| record_owns_path_coverage(record))
        .flat_map(|record| record.affected_paths.iter().map(String::as_str))
        .collect();
    let uncovered = changed
        .into_iter()
        .filter(|path| path_is_meaningful_for_root(root, path, policy))
        .filter(|path| {
            !covered.iter().any(|scope| path_matches_scope(path, scope))
                && !covering
                    .iter()
                    .any(|record| record_covers_project_path(root, record, path))
        })
        .collect();
    Ok(uncovered)
}

/// Archived packages that appear under the current delivery diff (PR vs base, or
/// local dirty tree including staged archive paths).
fn archived_records_in_delivery(
    root: &Path,
    changed_paths: &BTreeSet<String>,
) -> Result<Vec<ChangeRecord>, String> {
    let prefix = format!("{ARCHIVE_PATH}/");
    let mut dir_names = BTreeSet::new();
    for path in changed_paths {
        let Some(rest) = path.strip_prefix(&prefix) else {
            continue;
        };
        let Some(dir) = rest.split('/').next() else {
            continue;
        };
        if !dir.is_empty() {
            dir_names.insert(dir.to_string());
        }
    }
    if dir_names.is_empty() {
        return Ok(Vec::new());
    }
    let mut records = Vec::new();
    for dir in dir_names {
        let state_path = root.join(ARCHIVE_PATH).join(&dir).join("state.json");
        if !state_path.is_file() {
            continue;
        }
        let content = fs::read_to_string(&state_path).map_err(|error| {
            format!(
                "failed to read delivery archive state {}: {error}",
                state_path.display()
            )
        })?;
        let record: ChangeRecord = serde_json::from_str(&content).map_err(|error| {
            format!(
                "invalid delivery archive state {}: {error}",
                state_path.display()
            )
        })?;
        if record.state == ChangeState::Archived {
            records.push(record);
        }
    }
    Ok(records)
}

fn record_is_delivering(record: &ChangeRecord) -> bool {
    matches!(
        record.state,
        ChangeState::Implementing | ChangeState::Verifying | ChangeState::Accepted
    )
}

/// Active delivery states plus archived packages (same-PR finalize tips).
fn record_owns_path_coverage(record: &ChangeRecord) -> bool {
    record_is_delivering(record) || record.state == ChangeState::Archived
}

fn record_covers_path(record: &ChangeRecord, path: &str) -> bool {
    record_owns_path_coverage(record)
        && record
            .affected_paths
            .iter()
            .any(|scope| path_matches_scope(path, scope))
}

fn record_covers_project_path(root: &Path, record: &ChangeRecord, path: &str) -> bool {
    if !record_owns_path_coverage(record) {
        return false;
    }
    if record
        .affected_paths
        .iter()
        .any(|scope| path_matches_scope(path, scope))
    {
        return true;
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    record.affected_specs.iter().any(|module| {
        canonical_module_paths(root, &specs_dir, module)
            .ok()
            .is_some_and(|(spec_path, _)| {
                if path == portable_project_path(root, &spec_path) {
                    return true;
                }
                spec_path.parent().is_some_and(|parent| {
                    CANONICAL_SPEC_COMPANIONS
                        .iter()
                        .any(|name| path == portable_project_path(root, &parent.join(name)))
                })
            })
    })
}

fn git_worktree_is_clean(root: &Path) -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout.is_empty())
}

fn git_commit_count(root: &Path) -> Option<usize> {
    git_output(root, &["rev-list", "--count", "HEAD"])?
        .parse()
        .ok()
}

fn pull_request_diff_base(root: &Path, records: &[ChangeRecord]) -> String {
    if is_ci_project(root)
        && let Ok(branch) = std::env::var("GITHUB_BASE_REF")
        && !branch.trim().is_empty()
    {
        return format!("origin/{branch}...HEAD");
    }
    if let Some(remote_default) = remote_default_ref(root) {
        return format!("{remote_default}...HEAD");
    }
    recorded_diff_base(root, records)
}

fn remote_default_ref(root: &Path) -> Option<String> {
    if let Some(remote_default) = git_output(
        root,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) {
        return Some(remote_default);
    }
    ["origin/main", "origin/master"]
        .into_iter()
        .find(|candidate| git_output(root, &["rev-parse", "--verify", candidate]).is_some())
        .map(str::to_string)
}

fn recorded_diff_base(root: &Path, records: &[ChangeRecord]) -> String {
    records
        .iter()
        .filter_map(|record| record.base_commit.clone())
        .next()
        .unwrap_or_else(|| {
            // `HEAD~1` does not resolve in a repository whose first commit is
            // also its only one, and the resulting `git diff` failure reported
            // the entire gate as broken ("unable to inspect changed paths").
            // There is no earlier commit to review, so the committed delivery
            // is empty and only the working tree is up for coverage.
            if git_output(root, &["rev-parse", "--verify", "HEAD~1"]).is_some() {
                "HEAD~1...HEAD".into()
            } else {
                "HEAD".into()
            }
        })
}

#[cfg(test)]
fn path_is_meaningful(path: &str, policy: &SddPolicy) -> bool {
    path_is_meaningful_with_specs(path, policy, "specs/")
}

fn path_is_meaningful_for_root(root: &Path, path: &str, policy: &SddPolicy) -> bool {
    path_is_meaningful_with_specs(path, policy, &configured_specs_scope(root))
}

fn configured_specs_scope(root: &Path) -> String {
    let normalized = crate::config::load_config(root)
        .specs_dir
        .replace('\\', "/");
    if normalized == "." {
        normalized
    } else {
        format!("{}/", normalized.trim_matches('/'))
    }
}

fn path_is_meaningful_with_specs(path: &str, policy: &SddPolicy, specs_scope: &str) -> bool {
    if is_protected_sdd_path(path) {
        return true;
    }
    if path_matches_scope(path, specs_scope) {
        return true;
    }
    policy
        .meaningful_paths
        .iter()
        .any(|scope| path_matches_scope(path, scope))
        && !policy
            .ignored_paths
            .iter()
            .any(|scope| path_matches_scope(path, scope))
}

fn path_matches_scope(path: &str, scope: &str) -> bool {
    let normalized_scope = scope.replace('\\', "/");
    let scope = normalized_scope.trim_end_matches('/');
    if scope == "." {
        return true;
    }
    path == scope
        || path
            .strip_prefix(scope)
            .is_some_and(|remainder| remainder.starts_with('/'))
}

fn is_protected_sdd_path(path: &str) -> bool {
    matches!(
        path,
        ".specsync/sdd.json"
            | ".specsync/config.toml"
            | ".specsync/config.json"
            | ".specsync/registry.toml"
            | "specsync-registry.toml"
            | ".specsync/version"
            | ".specsync/change-sequence.json"
    )
}

fn portable_project_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn safe_project_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let normalized = normalize_project_path(relative)?;
    let joined = root.join(normalized);
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("failed to resolve project root {}: {error}", root.display()))?;
    let mut existing = joined.as_path();
    while !existing.exists() {
        existing = existing
            .parent()
            .ok_or_else(|| format!("project policy path escapes the project root: `{relative}`"))?;
    }
    let canonical_existing = existing.canonicalize().map_err(|error| {
        format!(
            "failed to resolve project path {}: {error}",
            existing.display()
        )
    })?;
    if !canonical_existing.starts_with(&canonical_root) {
        return Err(format!(
            "project policy path escapes the project root through a symlink: `{relative}`"
        ));
    }
    Ok(joined)
}

fn normalize_project_path(relative: &str) -> Result<String, String> {
    let normalized = relative.replace('\\', "/");
    let path = Path::new(&normalized);
    let has_windows_prefix = normalized
        .split('/')
        .next()
        .is_some_and(|component| component.ends_with(':'));
    if path.is_absolute()
        || has_windows_prefix
        || normalized.chars().any(char::is_control)
        || path.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "project policy path escapes the project root: `{relative}`"
        ));
    }
    Ok(normalized)
}

fn run_configured_command(
    root: &Path,
    configured: &str,
) -> Result<std::process::ExitStatus, String> {
    let parts = shell_words(configured)?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| "empty verification command".to_string())?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(root)
        .env(crate::VERIFICATION_CONTEXT_ENV, configured);
    command.status().map_err(|error| {
        format!("failed to run configured verification command `{configured}`: {error}")
    })
}

fn reject_direct_lifecycle_verification(root: &Path, configured: &str) -> Result<(), String> {
    let parts = shell_words(configured)?;
    let Some((program, args)) = parts.split_first() else {
        return Ok(());
    };
    let program_name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .trim_end_matches(".exe");
    let invokes_specsync = program_name == "specsync"
        || (program_name == "cargo" && cargo_run_targets_specsync(root, args)?);
    if invokes_specsync {
        return Err(format!(
            "recursive lifecycle verification command `{configured}` is not allowed; run native verification here and keep specsync check as a top-level gate"
        ));
    }
    Ok(())
}

fn cargo_run_targets_specsync(root: &Path, args: &[String]) -> Result<bool, String> {
    let mut manifest_path = None;
    let mut index = 0;
    let run_index = loop {
        let Some(argument) = args.get(index) else {
            return Ok(false);
        };
        if argument == "run" {
            break index;
        }
        if argument == "--manifest-path" {
            manifest_path = Some(
                args.get(index + 1)
                    .ok_or_else(|| "Cargo --manifest-path requires a path".to_string())?
                    .clone(),
            );
            index += 2;
            continue;
        }
        if let Some(path) = argument.strip_prefix("--manifest-path=") {
            manifest_path = Some(path.to_string());
            index += 1;
            continue;
        }
        if matches!(
            argument.as_str(),
            "--color" | "--config" | "--target-dir" | "-Z"
        ) {
            index += 2;
            continue;
        }
        if argument.starts_with('-') {
            index += 1;
            continue;
        }
        return Ok(false);
    };

    let mut selected_target = None;
    index = run_index + 1;
    while index < args.len() {
        let argument = &args[index];
        if argument == "--" {
            break;
        }
        if argument == "--manifest-path" {
            manifest_path = Some(
                args.get(index + 1)
                    .ok_or_else(|| "Cargo --manifest-path requires a path".to_string())?
                    .clone(),
            );
            index += 2;
            continue;
        }
        if let Some(path) = argument.strip_prefix("--manifest-path=") {
            manifest_path = Some(path.to_string());
            index += 1;
            continue;
        }
        if argument == "--bin" {
            selected_target = Some(
                args.get(index + 1)
                    .is_some_and(|target| target == "specsync"),
            );
            index += 2;
            continue;
        }
        if let Some(target) = argument.strip_prefix("--bin=") {
            selected_target = Some(target == "specsync");
            index += 1;
            continue;
        }
        if matches!(argument.as_str(), "-p" | "--package") {
            selected_target = Some(
                args.get(index + 1)
                    .is_some_and(|package| package == "specsync"),
            );
            index += 2;
            continue;
        }
        if let Some(package) = argument
            .strip_prefix("--package=")
            .or_else(|| argument.strip_prefix("-p"))
        {
            selected_target = Some(package == "specsync");
            index += 1;
            continue;
        }
        if matches!(argument.as_str(), "--example" | "--test" | "--bench")
            || argument.starts_with("--example=")
            || argument.starts_with("--test=")
            || argument.starts_with("--bench=")
        {
            selected_target = Some(false);
        }
        index += 1;
    }

    let manifest_path = match manifest_path {
        Some(path) if path.trim().is_empty() => {
            return Err("Cargo --manifest-path requires a non-empty path".into());
        }
        Some(path) => safe_project_path(root, &path)
            .map_err(|error| format!("unsafe Cargo manifest path `{path}`: {error}"))?,
        None => root.join("Cargo.toml"),
    };
    if let Some(selected_target) = selected_target {
        return Ok(selected_target);
    }
    let Ok(manifest) = fs::read_to_string(manifest_path) else {
        return Ok(false);
    };
    Ok(cargo_package_value(&manifest, "default-run")
        .or_else(|| cargo_package_value(&manifest, "name"))
        .is_some_and(|target| target == "specsync"))
}

fn cargo_package_value(manifest: &str, key: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line[1..line.len() - 1].trim() == "package";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((candidate, value)) = line.split_once('=') else {
            continue;
        };
        if candidate.trim() != key {
            continue;
        }
        let value = value
            .split_once('#')
            .map_or(value, |(value, _)| value)
            .trim();
        return Some(
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .or_else(|| {
                    value
                        .strip_prefix('\'')
                        .and_then(|value| value.strip_suffix('\''))
                })
                .unwrap_or(value)
                .to_string(),
        );
    }
    None
}

fn is_ci() -> bool {
    ["CI", "GITHUB_ACTIONS"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .any(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

fn is_ci_project(root: &Path) -> bool {
    if !is_ci() {
        return false;
    }
    let Ok(workspace) = std::env::var("GITHUB_WORKSPACE") else {
        return true;
    };
    let workspace = Path::new(&workspace);
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let canonical_workspace = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    canonical_root.starts_with(canonical_workspace)
}

fn shell_words(command: &str) -> Result<Vec<String>, String> {
    if command.contains([';', '|', '&', '>', '<', '`']) || command.contains("$(") {
        return Err(format!(
            "unsafe shell syntax is not allowed in verification command `{command}`"
        ));
    }
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Quote {
        None,
        Single,
        Double,
    }

    let mut words = Vec::new();
    let mut word = String::new();
    let mut word_started = false;
    let mut quote = Quote::None;
    let mut characters = command.chars();
    while let Some(character) = characters.next() {
        match quote {
            Quote::Single => {
                if character == '\'' {
                    quote = Quote::None;
                } else {
                    word.push(character);
                }
            }
            Quote::Double => {
                if character == '"' {
                    quote = Quote::None;
                } else if character == '\\' {
                    let escaped = characters.next().ok_or_else(|| {
                        format!("unterminated escape in verification command `{command}`")
                    })?;
                    word.push(escaped);
                } else {
                    word.push(character);
                }
            }
            Quote::None => {
                if character.is_whitespace() {
                    if word_started {
                        words.push(std::mem::take(&mut word));
                        word_started = false;
                    }
                } else if character == '#' && !word_started {
                    break;
                } else if character == '\'' {
                    quote = Quote::Single;
                    word_started = true;
                } else if character == '"' {
                    quote = Quote::Double;
                    word_started = true;
                } else if character == '\\' {
                    let escaped = characters.next().ok_or_else(|| {
                        format!("unterminated escape in verification command `{command}`")
                    })?;
                    word.push(escaped);
                    word_started = true;
                } else {
                    word.push(character);
                    word_started = true;
                }
            }
        }
    }
    if quote != Quote::None {
        return Err(format!(
            "unterminated quote in verification command `{command}`"
        ));
    }
    if word_started {
        words.push(word);
    }
    if words.is_empty() {
        return Err("verification command cannot be empty".into());
    }
    Ok(words)
}

fn detect_foreign_source(root: &Path) -> Option<String> {
    if root.join("openspec").is_dir() {
        return Some("openspec".into());
    }
    if root.join(".specify").is_dir() || root.join("specs").join("constitution.md").exists() {
        return Some("speckit".into());
    }
    None
}

fn prepare_foreign_import(root: &Path, source: &str) -> Result<Vec<(PathBuf, String)>, String> {
    let mut prepared = Vec::new();
    let provenance = root.join(".specsync/import-provenance.json");
    let value = serde_json::json!({
        "source": source,
        "imported_at": now(),
        "scope": "active_plus_canonical",
        "archives": "preserved_in_place"
    });
    prepared.push((provenance, json_content(&value)?));
    let import_root = root.join(".specsync/imports").join(source);
    match source {
        "openspec" => {
            let canonical = root.join("openspec/specs");
            reject_symlink_components(root, &canonical)?;
            match fs::symlink_metadata(&canonical) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "refusing symlinked foreign import path: {}",
                        canonical.display()
                    ));
                }
                Ok(metadata) if metadata.is_dir() => {
                    prepare_markdown_files(
                        root,
                        &canonical,
                        &import_root.join("canonical"),
                        &mut prepared,
                    )?;
                }
                Ok(_) => {
                    return Err(format!(
                        "foreign canonical import is not a directory: {}",
                        canonical.display()
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        "speckit" => {
            let constitution = root.join(".specify/memory/constitution.md");
            reject_symlink_components(root, &constitution)?;
            match fs::symlink_metadata(&constitution) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "refusing symlinked foreign import path: {}",
                        constitution.display()
                    ));
                }
                Ok(metadata) if metadata.is_file() => {
                    let content = fs::read_to_string(&constitution)
                        .map_err(|error| format!("failed to read foreign constitution: {error}"))?;
                    prepared.push((import_root.join("constitution.md"), content));
                }
                Ok(_) => {
                    return Err(format!(
                        "foreign constitution is not a regular file: {}",
                        constitution.display()
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        _ => {}
    }
    let source_changes = match source {
        "openspec" => root.join("openspec/changes"),
        "speckit" => root.join("specs"),
        _ => return Err(format!("unsupported adoption source `{source}`")),
    };
    reject_symlink_components(root, &source_changes)?;
    match fs::symlink_metadata(&source_changes) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(format!(
                "refusing symlinked foreign import path: {}",
                source_changes.display()
            ));
        }
        Ok(metadata) if metadata.is_dir() => {}
        Ok(_) => {
            return Err(format!(
                "foreign changes path is not a directory: {}",
                source_changes.display()
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(prepared),
        Err(error) => return Err(error.to_string()),
    }
    let mut entries = fs::read_dir(source_changes)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut known_descriptions: BTreeSet<String> = list_changes_checked(root)?
        .into_iter()
        .map(|record| record.description)
        .collect();
    let mut minted_ids: BTreeSet<String> = BTreeSet::new();
    for entry in entries {
        let entry_path = entry.path();
        reject_symlink_components(root, &entry_path)?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "refusing symlinked foreign import path: {}",
                entry_path.display()
            ));
        }
        if !file_type.is_dir() || entry.file_name() == "archive" {
            continue;
        }
        validate_foreign_tree(&entry_path)?;
        let is_active_change = match source {
            "openspec" => {
                entry_path.join("proposal.md").exists()
                    || entry_path.join("design.md").exists()
                    || entry_path.join("specs").is_dir()
            }
            "speckit" => entry_path.join("spec.md").exists() || entry_path.join("plan.md").exists(),
            _ => false,
        };
        if !is_active_change {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let description = format!("Imported {source} change {name}");
        if known_descriptions.contains(&description) {
            continue;
        }
        let id = mint_change_slug(&description)?;
        if find_change_dir(root, &id).is_ok() || !minted_ids.insert(id.clone()) {
            return Err(change_name_taken_error(root, &id));
        }
        let affected_paths = vec![portable_project_path(root, &entry_path)];
        let mut selected_artifacts = adaptive_artifacts(ChangeKind::Feature, &[], &affected_paths);
        for artifact in [
            ArtifactKind::Requirements,
            ArtifactKind::Design,
            ArtifactKind::Tasks,
        ] {
            if !selected_artifacts.contains(&artifact) {
                selected_artifacts.push(artifact);
            }
        }
        let timestamp = now();
        let record = ChangeRecord {
            schema_version: 1,
            workflow_version: 2,
            workflow_origin_version: Some(2),
            id: id.clone(),
            slug: id.clone(),
            title: title_from_description(&description),
            description: description.clone(),
            kind: ChangeKind::Feature,
            state: ChangeState::Draft,
            canonical_applied: false,
            correction_count: 0,
            base_commit: git_output(root, &["rev-parse", "HEAD"]),
            created_at: timestamp,
            updated_at: timestamp,
            affected_specs: Vec::new(),
            affected_paths,
            no_spec_change: true,
            no_spec_change_rationale: Some(format!(
                "Imported from {source}; canonical reconciliation is pending"
            )),
            acceptance_criteria: Vec::new(),
            selected_artifacts,
            dependencies: Vec::new(),
            supersedes: Vec::new(),
            acceptance_owner_corrections: Vec::new(),
            legacy_archive_baseline_digest: None,
            answers: BTreeMap::new(),
        };
        let record_dir = change_dir(root, &record.id);
        prepared.push((record_dir.join("state.json"), json_content(&record)?));
        prepared.push((
            record_dir.join("change.md"),
            change_markdown_content(&record),
        ));
        prepared.push((
            record_dir.join("approvals.json"),
            json_content(&ApprovalLedger::default())?,
        ));
        for artifact in &record.selected_artifacts {
            prepared.push((
                record_dir.join(artifact.file_name()),
                artifact_template(root, artifact, &record),
            ));
        }
        let destination = change_dir(root, &record.id).join("imported");
        prepare_markdown_files(root, &entry_path, &destination, &mut prepared)?;
        known_descriptions.insert(description);
    }
    Ok(prepared)
}

fn validate_foreign_import(root: &Path, source: &str) -> Result<(), String> {
    let candidates = match source {
        "openspec" => vec![root.join("openspec/specs"), root.join("openspec/changes")],
        "speckit" => vec![
            root.join(".specify/memory/constitution.md"),
            root.join("specs"),
        ],
        _ => return Err(format!("unsupported adoption source `{source}`")),
    };
    for candidate in candidates {
        reject_symlink_components(root, &candidate)?;
        match fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked foreign import path: {}",
                    candidate.display()
                ));
            }
            Ok(metadata) if metadata.is_dir() => validate_foreign_tree(&candidate)?,
            Ok(metadata)
                if source == "speckit"
                    && candidate.ends_with("constitution.md")
                    && metadata.is_file() => {}
            Ok(_) => {
                return Err(format!(
                    "foreign import path is not the expected regular file or directory: {}",
                    candidate.display()
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn prepare_markdown_files(
    project_root: &Path,
    source: &Path,
    destination: &Path,
    prepared: &mut Vec<(PathBuf, String)>,
) -> Result<(), String> {
    reject_symlink_components(project_root, source)?;
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "refusing non-directory or symlinked foreign import path: {}",
            source.display()
        ));
    }
    let mut entries = fs::read_dir(source)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "refusing symlinked foreign import path: {}",
                path.display()
            ));
        }
        if file_type.is_dir() {
            prepare_markdown_files(
                project_root,
                &path,
                &destination.join(entry.file_name()),
                prepared,
            )?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("md")
        {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read foreign Markdown: {error}"))?;
            prepared.push((destination.join(entry.file_name()), content));
        }
    }
    Ok(())
}

fn reject_symlink_components(project_root: &Path, candidate: &Path) -> Result<(), String> {
    reject_symlink_components_for(project_root, candidate, "foreign import path")
}

fn reject_symlink_components_for(
    project_root: &Path,
    candidate: &Path,
    label: &str,
) -> Result<(), String> {
    let relative = candidate
        .strip_prefix(project_root)
        .map_err(|_| format!("{label} escapes project root: {}", candidate.display()))?;
    let mut current = project_root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("refusing symlinked {label}: {}", current.display()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn validate_foreign_tree(root: &Path) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_type().is_symlink() {
            return Err(format!(
                "refusing symlinked foreign import path: {}",
                entry.path().display()
            ));
        }
    }
    Ok(())
}

fn save_change(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    write_json(&change_dir(root, &record.id).join("state.json"), record)
}

fn write_change_markdown(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    fs::write(
        change_dir(root, &record.id).join("change.md"),
        change_markdown_content(record),
    )
    .map_err(|error| error.to_string())
}

fn change_markdown_content(record: &ChangeRecord) -> String {
    let criteria = if record.acceptance_criteria.is_empty() {
        "- <!-- TODO: add observable acceptance criteria -->".into()
    } else {
        record
            .acceptance_criteria
            .iter()
            .map(|criterion| format!("- {criterion}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let specs = if record.affected_specs.is_empty() {
        "- None".into()
    } else {
        record
            .affected_specs
            .iter()
            .map(|spec| format!("- `{spec}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "---\nid: {}\nstate: {}\ntype: {}\nbase_commit: {}\n---\n\n# {}\n\n## Intent\n\n{}\n\n## Affected Canonical Specs\n\n{}\n\n## Acceptance Criteria\n\n{}\n\n## No-spec Rationale\n\n{}\n",
        record.id,
        record.state.as_str(),
        record.kind.as_str(),
        record.base_commit.as_deref().unwrap_or("uncommitted"),
        record.title,
        record.description,
        specs,
        criteria,
        record
            .no_spec_change_rationale
            .as_deref()
            .unwrap_or("Not applicable"),
    )
}

fn ensure_artifact_files(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let dir = change_dir(root, &record.id);
    for artifact in &record.selected_artifacts {
        let path = dir.join(artifact.file_name());
        if !path.exists() {
            fs::write(path, artifact_template(root, artifact, record))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn artifact_template(root: &Path, artifact: &ArtifactKind, record: &ChangeRecord) -> String {
    if let ArtifactKind::Custom(name) = artifact
        && let Some(policy) = load_policy(root)
        && let Some(template) = policy.custom_artifacts.get(name)
        && let Ok(path) = safe_project_path(root, template)
        && let Ok(content) = fs::read_to_string(path)
    {
        return content
            .replace("{{change_id}}", &record.id)
            .replace("{{title}}", &record.title);
    }
    let title = artifact
        .file_name()
        .trim_end_matches(".md")
        .replace('-', " ");
    // Context is the change's own working record: what led here, and what a later
    // agent picking this up mid-flight needs to know. Prompt for that.
    //
    // Lessons belong somewhere else. They are folded into the *spec's* companion
    // context at archival, by the agent, drawn from the change's commits and PR —
    // so a module accumulates what was learned about it across every change that
    // touched it. A per-change lessons file would die with the change, which is the
    // opposite of the point. See docs/6-0-findings.md.
    //
    // Prompts are HTML comments so they guide without counting as content: the
    // artifact must still read as incomplete until an author writes something.
    let prompt = match artifact {
        ArtifactKind::Context => concat!(
            "<!-- What led here: the problem, and how it was noticed. -->\n\n",
            "<!-- What a session picking this up mid-flight needs to know: constraints,\n",
            "     prior attempts, anything already ruled out. -->\n\n",
        ),
        _ => "",
    };
    format!(
        "---\nchange: {}\nartifact: {}\n---\n\n# {}\n\n{}<!-- TODO: complete this artifact or remove it from selected_artifacts before approval. -->\n",
        record.id,
        title,
        title_from_description(&title),
        prompt
    )
}

fn delta_path_checked(root: &Path, record: &ChangeRecord, module: &str) -> Result<PathBuf, String> {
    Ok(find_change_dir(root, &record.id)?
        .join("deltas")
        .join(format!("{module}.md")))
}

#[cfg(test)]
fn delta_path(root: &Path, record: &ChangeRecord, module: &str) -> PathBuf {
    change_dir(root, &record.id)
        .join("deltas")
        .join(format!("{module}.md"))
}

fn change_dir(root: &Path, id: &str) -> PathBuf {
    root.join(CHANGES_PATH).join(id)
}

/// Resolves a change's workspace wherever it currently lives — active or archived.
///
/// This is the single answer to "where are this change's artifacts?". Callers that
/// hard-code `.specsync/changes/<id>/` are correct only until the change is archived,
/// which is exactly how `ship-status` came to report `Verification: none` on a change
/// that had verified (#534). Reuse this rather than growing a third idiom beside it.
pub fn find_change_dir(root: &Path, id: &str) -> Result<PathBuf, String> {
    validate_change_id(id)?;
    let active = change_dir(root, id);
    let mut matches = Vec::new();
    if active.is_dir() {
        matches.push(active);
    }
    let archive = root.join(ARCHIVE_PATH);
    if let Ok(entries) = fs::read_dir(archive) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let state = entry.path().join("state.json");
            let matches_id = fs::read(&state)
                .ok()
                .and_then(|content| serde_json::from_slice::<ChangeRecord>(&content).ok())
                .is_some_and(|record| record.id == id);
            if matches_id {
                matches.push(entry.path());
            }
        }
    }
    match matches.len() {
        0 => Err(format!("change `{id}` was not found")),
        1 => Ok(matches.remove(0)),
        _ => Err(format!(
            "change `{id}` has ambiguous active/archive workspace locations: {}",
            matches
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Longest change ID accepted, in bytes.
///
/// A change ID is a single path component, so the filesystem's 255-byte component limit is the
/// hard ceiling. This is deliberately the *ceiling* and not the slug cap: `MAX_SLUG_BYTES`
/// bounds what SpecSync mints, while this bounds what it will read — an ID minted by a
/// different version, or by hand, must still load if it is legal. The longest ID in this
/// repository's own archive is 90 bytes.
///
/// There was no bound here at all. That was survivable only because every ID began with a
/// generated `CHG-NNNN-` prefix over a capped slug; the moment an arbitrary name is accepted,
/// an unbounded one is a path this process cannot open and a directory it cannot create.
const MAX_CHANGE_ID_BYTES: usize = 255;

/// Validate a change ID as one safe, portable path component.
///
/// The `id.starts_with("CHG-")` test that used to lead this function was doing two jobs at
/// once: proving the ID was well-formed, and proving it was ours. It was never evidence of
/// either — `CHG-` is a prefix anyone can type — and it hard-rejected every identity shape
/// without an ordinal, which is the shape this release is moving to.
///
/// What replaces it is the set of properties that actually matter for something used as a
/// directory name: one component, no separators, no control characters, non-empty, bounded,
/// and not a name a supported platform reserves. Every check here is about what the string
/// *is*; none is about how it begins.
fn validate_change_id(id: &str) -> Result<(), String> {
    let is_single_component = {
        let mut components = Path::new(id).components();
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
    };
    if !id.is_empty()
        && id.len() <= MAX_CHANGE_ID_BYTES
        && is_single_component
        && !id.contains(['/', '\\'])
        && !id.chars().any(char::is_control)
        && !crate::commands::is_reserved_module_name(&id.to_ascii_lowercase())
    {
        return Ok(());
    }
    Err(format!("invalid change ID `{}`", id.escape_default()))
}

fn validate_loaded_change(
    record: &ChangeRecord,
    expected_id: &str,
    state_path: &Path,
) -> Result<(), String> {
    validate_change_id(&record.id)
        .map_err(|error| format!("invalid change state {}: {error}", state_path.display()))?;
    if !matches!(record.workflow_version, 1 | 2) {
        // A version this binary does not know means the record was written by a NEWER
        // SpecSync, not that it is corrupt. Reporting it as an "invalid change state" made
        // those two indistinguishable, so the operator's correct action — upgrade — was the
        // one thing the message did not say. Naming it is what lets a later workflow version
        // exist without every older 6.x install reporting the repository as broken.
        return Err(format!(
            "{} was written by a newer SpecSync (workflow version {}); upgrade specsync to read it",
            state_path.display(),
            record.workflow_version
        ));
    }
    validate_workflow_version_anchor(record)
        .map_err(|error| format!("invalid change state {}: {error}", state_path.display()))?;
    if record.id != expected_id {
        return Err(format!(
            "invalid change state {}: persisted ID `{}` does not match workspace `{expected_id}`",
            state_path.display(),
            record.id
        ));
    }
    for module in &record.affected_specs {
        crate::commands::validate_module_name(module).map_err(|error| {
            format!(
                "invalid change state {}: invalid affected spec: {error}",
                state_path.display()
            )
        })?;
    }
    for artifact in &record.selected_artifacts {
        if let ArtifactKind::Custom(name) = artifact
            && !matches!(ArtifactKind::parse(name), ArtifactKind::Custom(canonical) if canonical == *name)
        {
            return Err(format!(
                "invalid change state {}: unsafe custom artifact `{}`",
                state_path.display(),
                name.escape_default()
            ));
        }
    }
    Ok(())
}

fn validate_workflow_version_anchor(record: &ChangeRecord) -> Result<(), String> {
    match (record.workflow_version, record.workflow_origin_version) {
        (1, None | Some(1)) | (2, Some(2)) => Ok(()),
        (2, None) => {
            Err("workflow-v2 state is missing its immutable workflow_origin_version anchor".into())
        }
        (version, Some(origin)) => Err(format!(
            "workflow version {version} conflicts with immutable origin {origin}"
        )),
        // As above: an unknown version is a newer writer, not corruption.
        _ => Err(format!(
            "change `{}` was written by a newer SpecSync (workflow version {}); upgrade specsync to read it",
            record.id, record.workflow_version
        )),
    }
}

fn validate_historical_workflow_version_anchor(record: &ChangeRecord) -> Result<(), String> {
    match (record.workflow_version, record.workflow_origin_version) {
        (1, None | Some(1)) | (2, None | Some(2)) => Ok(()),
        (version, Some(origin)) => Err(format!(
            "workflow version {version} conflicts with immutable origin {origin}"
        )),
        // As above: an unknown version is a newer writer, not corruption.
        _ => Err(format!(
            "change `{}` was written by a newer SpecSync (workflow version {}); upgrade specsync to read it",
            record.id, record.workflow_version
        )),
    }
}

fn ensure_workflow_v2_baseline(root: &Path) -> Result<(), String> {
    let Some(candidate) = prepare_workflow_v2_adoption_candidate(root)? else {
        return Ok(());
    };
    let expected_snapshot = candidate.git_snapshot.clone();
    write_prepared_files_checked(
        root,
        &[(
            root.join(WORKFLOW_V2_BASELINE_PATH),
            json_content(&candidate.baseline)?,
        )],
        || validate_workflow_v2_adoption_git_snapshot(root, &expected_snapshot),
    )
}

fn prepare_workflow_v2_adoption_candidate(
    root: &Path,
) -> Result<Option<WorkflowV2AdoptionCandidate>, String> {
    if read_workflow_v2_baseline(root)?.is_some() {
        return Ok(None);
    }
    let git_snapshot = workflow_v2_adoption_git_snapshot(root)?;
    validate_workflow_v1_records_at_adoption_cutoff(root, git_snapshot.cutoff_commit.as_deref())?;
    Ok(Some(WorkflowV2AdoptionCandidate {
        baseline: WorkflowV2Baseline {
            schema_version: 1,
            domain: "specsync.workflow-v2-baseline.v1".into(),
            cutoff_commit: git_snapshot.cutoff_commit.clone(),
        },
        git_snapshot,
    }))
}

fn validate_workflow_v1_records_at_adoption_cutoff(
    root: &Path,
    cutoff: Option<&str>,
) -> Result<(), String> {
    let records = list_all_changes_checked(root)?;
    let workflow_v1_records: Vec<&ChangeRecord> = records
        .values()
        .filter(|record| record.workflow_version == 1)
        .collect();
    if workflow_v1_records.is_empty() {
        return Ok(());
    }
    let Some(cutoff) = cutoff else {
        let ids = workflow_v1_records
            .iter()
            .map(|record| record.id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "cannot adopt workflow v2 because workflow-v1 change(s) {ids} have no trusted Git cutoff; commit and integrate or archive them in comparison-base history, then rerun `specsync change adopt`"
        ));
    };
    let mut missing = Vec::new();
    for record in workflow_v1_records {
        if !workflow_v1_record_exists_at_cutoff(root, record, cutoff)? {
            missing.push(record.id.as_str());
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "cannot adopt workflow v2 because workflow-v1 change(s) {} are absent from trusted cutoff {cutoff}; commit and integrate or archive them in comparison-base history, then rerun `specsync change adopt`",
        missing.join(", ")
    ))
}

#[cfg(test)]
fn workflow_v2_baseline_cutoff(root: &Path) -> Option<String> {
    workflow_v2_adoption_git_snapshot(root).ok()?.cutoff_commit
}

fn workflow_v2_adoption_git_snapshot(root: &Path) -> Result<WorkflowV2AdoptionGitSnapshot, String> {
    if !git_repository_present_uncached(root)? {
        return Ok(WorkflowV2AdoptionGitSnapshot {
            head: None,
            comparison_reference: None,
            comparison_tip: None,
            cutoff_commit: None,
        });
    }
    let head = uncached_optional_git_text(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let comparison_reference = uncached_optional_git_text(
        root,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )?
    .or_else(|| {
        ["origin/main", "origin/master"]
            .into_iter()
            .find_map(|candidate| {
                uncached_optional_git_text(
                    root,
                    &["rev-parse", "--verify", &format!("{candidate}^{{commit}}")],
                )
                .ok()
                .flatten()
                .map(|_| candidate.to_string())
            })
    });
    let comparison_tip = comparison_reference
        .as_deref()
        .map(|reference| {
            uncached_optional_git_text(
                root,
                &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
            )
        })
        .transpose()?
        .flatten();
    let cutoff_commit = match (comparison_reference.as_deref(), head.as_deref()) {
        (Some(reference), Some(_)) => {
            uncached_optional_git_text(root, &["merge-base", reference, "HEAD"])?
                .filter(|cutoff| is_canonical_commit_id(cutoff))
                .or_else(|| head.clone())
        }
        _ => head.clone(),
    };
    Ok(WorkflowV2AdoptionGitSnapshot {
        head,
        comparison_reference,
        comparison_tip,
        cutoff_commit,
    })
}

fn uncached_optional_git_text(root: &Path, args: &[&str]) -> Result<Option<String>, String> {
    let output = run_git_bounded(root, args, None, MAX_GIT_DIAGNOSTIC_BYTES)?;
    if !output.status.success() {
        return Ok(None);
    }
    let value = String::from_utf8(output.stdout).map_err(|_| {
        format!(
            "Git query returned non-UTF-8 output: git {}",
            args.join(" ")
        )
    })?;
    let value = value.trim();
    Ok((!value.is_empty()).then(|| value.to_string()))
}

fn validate_workflow_v2_adoption_git_snapshot(
    root: &Path,
    expected: &WorkflowV2AdoptionGitSnapshot,
) -> Result<(), String> {
    let current = workflow_v2_adoption_git_snapshot(root)?;
    if &current == expected {
        return Ok(());
    }
    Err(
        "Git HEAD or remote-default comparison reference changed during workflow-v2 adoption; no migration was published, so retry `specsync change adopt` from a stable checkout"
            .into(),
    )
}

fn read_workflow_v2_baseline(root: &Path) -> Result<Option<(WorkflowV2Baseline, Vec<u8>)>, String> {
    let path = root.join(WORKFLOW_V2_BASELINE_PATH);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if workflow_v2_baseline_exists_in_reachable_history(root)? {
                return Err(format!(
                    "committed workflow-v2 baseline was deleted; restore the exact reachable {} bytes before continuing",
                    WORKFLOW_V2_BASELINE_PATH
                ));
            }
            return Ok(None);
        }
        Err(error) => {
            return Err(format!(
                "failed to inspect workflow-v2 baseline {}: {error}",
                path.display()
            ));
        }
    };
    if !metadata.file_type().is_file() {
        return Err(format!(
            "workflow-v2 baseline is not a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() > MAX_CHANGE_ARTIFACT_BYTES {
        return Err(format!(
            "workflow-v2 baseline exceeds {} byte limit",
            MAX_CHANGE_ARTIFACT_BYTES
        ));
    }
    let bytes =
        fs::read(&path).map_err(|error| format!("failed to read workflow-v2 baseline: {error}"))?;
    let baseline: WorkflowV2Baseline = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid workflow-v2 baseline: {error}"))?;
    if baseline.schema_version != 1 || baseline.domain != "specsync.workflow-v2-baseline.v1" {
        return Err("unsupported workflow-v2 baseline schema or domain".into());
    }
    // Require canonical JSON structure. Git on Windows with autocrlf may rewrite
    // the working tree to CRLF without changing the committed blob; treat that as
    // the same baseline after stripping CR, still rejecting any other divergence.
    let canonical = json_content(&baseline)?;
    if !bytes_match_canonical_json(bytes.as_slice(), canonical.as_bytes()) {
        return Err("workflow-v2 baseline must use canonical persisted JSON bytes".into());
    }
    if let Some(cutoff) = baseline.cutoff_commit.as_deref()
        && !is_canonical_commit_id(cutoff)
    {
        return Err("workflow-v2 baseline cutoff must be a canonical commit ID".into());
    }
    Ok(Some((baseline, bytes)))
}

fn workflow_v2_baseline_exists_in_reachable_history(root: &Path) -> Result<bool, String> {
    if !git_repository_present(root)?
        || git_output(root, &["rev-parse", "--verify", "HEAD"]).is_none()
    {
        return Ok(false);
    }
    let baseline_path = git_repo_relative_path(root, WORKFLOW_V2_BASELINE_PATH)?;
    let history = scoped_review_git_text(
        root,
        &[
            "rev-list",
            "--full-history",
            "--max-count=1",
            "HEAD",
            "--",
            baseline_path.as_str(),
        ],
    )
    .map_err(|_| "failed to inspect reachable workflow-v2 baseline history".to_string())?;
    Ok(history.lines().any(|line| !line.trim().is_empty()))
}

fn validate_workflow_v2_baseline(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let baseline = read_workflow_v2_baseline(root)?;
    let Some((baseline, current_bytes)) = baseline else {
        if record.workflow_version >= 2 {
            return Err(format!(
                "workflow-v2 state requires {}; run `specsync change new` or restore the committed baseline",
                WORKFLOW_V2_BASELINE_PATH
            ));
        }
        return Ok(());
    };
    if !git_repository_present(root)? {
        if baseline.cutoff_commit.is_some() {
            return Err("non-Git workflow-v2 baseline cannot name a cutoff commit".into());
        }
        if record.workflow_version == 1 {
            return Err("workflow-v1 state is not eligible after workflow-v2 adoption".into());
        }
        return Ok(());
    }

    let head = git_output(root, &["rev-parse", "--verify", "HEAD"]);
    let baseline_path = git_repo_relative_path(root, WORKFLOW_V2_BASELINE_PATH)?;
    let Some(head) = head else {
        if baseline.cutoff_commit.is_some() {
            return Err("unborn workflow-v2 baseline cannot name a cutoff commit".into());
        }
        if record.workflow_version == 1 {
            return Err("workflow-v1 state is not eligible after workflow-v2 adoption".into());
        }
        return Ok(());
    };

    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let history = scoped_review_git_text(
        root,
        &[
            "rev-list",
            "--reverse",
            "--full-history",
            max_count.as_str(),
            "HEAD",
            "--",
            baseline_path.as_str(),
        ],
    )
    .map_err(|_| "failed to enumerate workflow-v2 baseline history".to_string())?;
    let history: Vec<&str> = history.lines().filter(|line| !line.is_empty()).collect();
    if history.len() > limits.scoped_review_max_descendants {
        return Err(format!(
            "workflow-v2 baseline history exceeds the deterministic {}-commit bound",
            limits.scoped_review_max_descendants
        ));
    }
    if history.is_empty() {
        if let Some(cutoff) = baseline.cutoff_commit.as_deref()
            && !cached_git_status_success(root, &["merge-base", "--is-ancestor", cutoff, &head])?
        {
            return Err(
                "uncommitted workflow-v2 baseline cutoff is not an ancestor of HEAD".into(),
            );
        }
    } else {
        let introductions = scoped_review_git_text(
            root,
            &[
                "log",
                "--format=%H",
                "--diff-filter=A",
                "HEAD",
                "--",
                baseline_path.as_str(),
            ],
        )
        .map_err(|_| "failed to locate workflow-v2 baseline introduction".to_string())?;
        let introductions: Vec<&str> = introductions
            .lines()
            .filter(|line| !line.is_empty())
            .collect();
        if introductions.len() != 1 {
            return Err("workflow-v2 baseline must have one unique introduction".into());
        }
        let introduction = introductions[0];
        let fields =
            scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", introduction])
                .map_err(|_| "failed to inspect workflow-v2 baseline introduction".to_string())?;
        let fields: Vec<&str> = fields.split_whitespace().collect();
        if fields.first().copied() != Some(introduction)
            || fields.len().saturating_sub(1) > limits.scoped_review_max_parents
        {
            return Err("workflow-v2 baseline introduction has ambiguous parents".into());
        }
        let parents = &fields[1..];
        match (parents.first().copied(), baseline.cutoff_commit.as_deref()) {
            (None, None) => {}
            (Some(first_parent), Some(cutoff))
                if cached_git_status_success(
                    root,
                    &["merge-base", "--is-ancestor", cutoff, first_parent],
                )? => {}
            _ => {
                return Err(
                    "workflow-v2 baseline cutoff must be an ancestor of its introduction's first parent"
                        .into(),
                );
            }
        }
        for parent in parents {
            if scoped_review_file_at_commit(root, parent, &baseline_path)?.is_some() {
                return Err("workflow-v2 baseline existed before its claimed introduction".into());
            }
        }
        let introduced_bytes = scoped_review_file_at_commit(root, introduction, &baseline_path)?
            .ok_or_else(|| {
                "workflow-v2 baseline introduction is missing its exact bytes".to_string()
            })?;
        for commit in &history {
            let bytes =
                scoped_review_file_at_commit(root, commit, &baseline_path)?.ok_or_else(|| {
                    "workflow-v2 baseline was deleted in reachable history".to_string()
                })?;
            if bytes != introduced_bytes {
                return Err("workflow-v2 baseline changed after its introduction".into());
            }
            let fields =
                scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", commit])
                    .map_err(|_| {
                        format!("failed to inspect workflow-v2 baseline commit {commit}")
                    })?;
            let fields: Vec<&str> = fields.split_whitespace().collect();
            if fields.first().copied() != Some(*commit)
                || fields.len().saturating_sub(1) > limits.scoped_review_max_parents
            {
                return Err(format!(
                    "workflow-v2 baseline commit {commit} has ambiguous parent history"
                ));
            }
            for parent in &fields[1..] {
                if let Some(parent_bytes) =
                    scoped_review_file_at_commit(root, parent, &baseline_path)?
                    && parent_bytes != introduced_bytes
                {
                    return Err(
                        "workflow-v2 baseline has a rewritten parent in reachable history".into(),
                    );
                }
            }
        }
        let committed_bytes = scoped_review_file_at_commit(root, "HEAD", &baseline_path)?
            .ok_or_else(|| "committed workflow-v2 baseline was deleted".to_string())?;
        if introduced_bytes != committed_bytes || committed_bytes != current_bytes {
            return Err("workflow-v2 baseline changed after its introduction".into());
        }
    }

    if let Some(cutoff) = baseline.cutoff_commit.as_deref()
        && !cached_git_status_success(root, &["merge-base", "--is-ancestor", cutoff, "HEAD"])?
    {
        return Err("workflow-v2 baseline cutoff is not reachable from HEAD".into());
    }
    if record.workflow_version != 1 {
        return Ok(());
    }
    let cutoff = baseline.cutoff_commit.as_deref().ok_or_else(|| {
        "workflow-v1 state is not eligible after root workflow-v2 adoption".to_string()
    })?;
    if !workflow_v1_record_exists_at_cutoff(root, record, cutoff)? {
        return Err(format!(
            "workflow-v1 change {} was not present at the trusted pre-v2 cutoff {}",
            record.id, cutoff
        ));
    }
    Ok(())
}

fn workflow_v1_record_exists_at_cutoff(
    root: &Path,
    record: &ChangeRecord,
    cutoff: &str,
) -> Result<bool, String> {
    let candidate_paths = workflow_version_state_paths(root, record)?;
    Ok(candidate_paths.iter().any(|path| {
        git_change_record_at(root, cutoff, path).is_some_and(|historical| {
            historical.id == record.id
                && historical.workflow_version == 1
                && matches!(historical.workflow_origin_version, None | Some(1))
        })
    }))
}

fn validate_workflow_version_history(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let key = format!(
        "{}:{}:{:?}:{:?}",
        record.id, record.workflow_version, record.workflow_origin_version, record.state
    );
    if let Some(result) = read_scope_value(root, |scope| {
        scope.workflow_version_history.get(&key).cloned()
    }) {
        return result;
    }
    let result = validate_workflow_version_history_uncached(root, record);
    update_read_scope(root, |scope| {
        scope.workflow_version_history.insert(key, result.clone());
    });
    result
}

fn validate_workflow_version_history_uncached(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    validate_workflow_v2_baseline(root, record)?;
    if !git_repository_present(root)? {
        return Ok(());
    }
    if git_output(root, &["rev-parse", "--verify", "HEAD"]).is_none() {
        return Ok(());
    }
    let state_paths = workflow_version_state_paths(root, record)?;
    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let mut arguments = vec![
        "rev-list",
        "--reverse",
        "--full-history",
        max_count.as_str(),
        "HEAD",
        "--",
    ];
    arguments.extend(state_paths.iter().map(String::as_str));
    let commits = scoped_review_git_text(root, &arguments)
        .map_err(|_| "failed to enumerate workflow-version history".to_string())?;
    let commits: Vec<&str> = commits.lines().filter(|line| !line.is_empty()).collect();
    if commits.len() > limits.scoped_review_max_descendants {
        return Err(format!(
            "workflow-version history exceeds the deterministic {}-commit bound",
            limits.scoped_review_max_descendants
        ));
    }
    for commit in commits {
        let current = workflow_version_at_commit(root, commit, &state_paths)?;
        let parents = scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", commit])
            .map_err(|_| format!("failed to inspect workflow-version commit {commit}"))?;
        let fields: Vec<&str> = parents.split_whitespace().collect();
        if fields.first().copied() != Some(commit)
            || fields.len().saturating_sub(1) > limits.scoped_review_max_parents
        {
            return Err(format!(
                "workflow-version commit {commit} has ambiguous parent history"
            ));
        }
        if fields.len() == 1 {
            if current.is_none() {
                return Err("workflow-version history deleted its creation anchor".into());
            }
            continue;
        }
        for parent in &fields[1..] {
            let previous = workflow_version_at_commit(root, parent, &state_paths)?;
            validate_workflow_version_transition(previous.as_ref(), current.as_ref())?;
        }
    }

    for path in &state_paths {
        let Some(bytes) = scoped_review_file_at_commit(root, "HEAD", path)? else {
            continue;
        };
        let committed: ChangeRecord = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid committed workflow-version state: {error}"))?;
        if committed.workflow_version != record.workflow_version
            || !matches!(
                (
                    committed.workflow_origin_version,
                    record.workflow_origin_version,
                ),
                (previous, current) if previous == current
                    || (previous.is_none()
                        && current == Some(record.workflow_version))
            )
        {
            return Err(
                "workflow version or immutable creation anchor changed from committed state".into(),
            );
        }
        break;
    }
    Ok(())
}

fn workflow_version_state_paths(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
    let active_state =
        git_repo_relative_path(root, &format!("{CHANGES_PATH}/{}/state.json", record.id))?;
    let mut state_paths = vec![active_state];
    if record.state == ChangeState::Archived {
        let workspace = find_change_dir(root, &record.id)?;
        let archive_state = git_repo_relative_path(
            root,
            &portable_project_path(root, &workspace.join("state.json")),
        )?;
        if !state_paths.contains(&archive_state) {
            state_paths.push(archive_state);
        }
    }
    if !git_repository_present(root)?
        || git_output(root, &["rev-parse", "--verify", "HEAD"]).is_none()
    {
        return Ok(state_paths);
    }

    let archive_root = git_repo_relative_path(root, ARCHIVE_PATH)?;
    let pathspec = format!(":(glob){archive_root}/*-{}/state.json", record.id.as_str());
    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let commits = scoped_review_git_text(
        root,
        &[
            "rev-list",
            "--full-history",
            max_count.as_str(),
            "HEAD",
            "--",
            pathspec.as_str(),
        ],
    )
    .map_err(|_| "failed to enumerate archived workflow-version history".to_string())?;
    let commits: Vec<&str> = commits.lines().filter(|line| !line.is_empty()).collect();
    if commits.len() > limits.scoped_review_max_descendants {
        return Err(format!(
            "archived workflow-version history exceeds the deterministic {}-commit bound",
            limits.scoped_review_max_descendants
        ));
    }
    let archived_paths = scoped_review_git_text(
        root,
        &[
            "log",
            "--full-history",
            "--format=",
            "--name-only",
            "-z",
            max_count.as_str(),
            "HEAD",
            "--",
            pathspec.as_str(),
        ],
    )
    .map_err(|_| "failed to enumerate archived workflow-version paths".to_string())?;
    for raw_path in archived_paths.split('\0') {
        let path = raw_path.trim_matches('\n');
        if path.is_empty() {
            continue;
        }
        if !is_canonical_archive_state_path(path, &archive_root, &record.id) {
            return Err(format!(
                "workflow-version history contains a non-canonical archive path `{}`",
                path.escape_default()
            ));
        }
        if !state_paths.iter().any(|candidate| candidate == path) {
            state_paths.push(path.to_string());
        }
    }
    Ok(state_paths)
}

fn is_canonical_archive_state_path(path: &str, archive_root: &str, id: &str) -> bool {
    let prefix = format!("{archive_root}/");
    let suffix = format!("-{id}/state.json");
    let Some(relative) = path.strip_prefix(&prefix) else {
        return false;
    };
    let Some(date) = relative.strip_suffix(&suffix) else {
        return false;
    };
    let bytes = date.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn workflow_version_at_commit(
    root: &Path,
    commit: &str,
    state_paths: &[String],
) -> Result<Option<(String, u32, Option<u32>)>, String> {
    let mut found = None;
    for path in state_paths {
        let Some(bytes) = scoped_review_file_at_commit(root, commit, path)? else {
            continue;
        };
        if found.is_some() {
            return Err(format!(
                "workflow-version history duplicates state at commit {commit}"
            ));
        }
        let historical: ChangeRecord = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid workflow-version history at {commit}: {error}"))?;
        validate_historical_workflow_version_anchor(&historical)?;
        found = Some((
            path.clone(),
            historical.workflow_version,
            historical.workflow_origin_version,
        ));
    }
    Ok(found)
}

fn validate_workflow_version_transition(
    previous: Option<&(String, u32, Option<u32>)>,
    current: Option<&(String, u32, Option<u32>)>,
) -> Result<(), String> {
    let Some((_, current_version, current_origin)) = current else {
        return Err("workflow-version history deleted its immutable creation anchor".into());
    };
    let Some((_, previous_version, previous_origin)) = previous else {
        return Ok(());
    };
    if current_version != previous_version
        || !matches!(
            (previous_origin, current_origin),
            (previous, current)
                if previous == current
                    || (previous.is_none() && *current == Some(*current_version))
        )
    {
        return Err(format!(
            "workflow-version history changed immutable identity from {previous_version}/{previous_origin:?} to {current_version}/{current_origin:?}"
        ));
    }
    Ok(())
}

fn verification_is_current(
    root: &Path,
    record: &ChangeRecord,
    evidence: &VerificationRecord,
) -> bool {
    verification_is_current_checked(root, record, evidence).is_ok()
}

/// Whether a change's recorded verification is current, loading the evidence itself.
///
/// One export rather than two: callers outside this module need the answer, not the record.
/// Missing or unreadable evidence is not current, which keeps this total for an inspection
/// command — a strict `?` here would turn `ship-status` from rc=0 into rc=1 on a workspace whose
/// evidence is already damaged, and the fix for an inspection command must not brick inspection.
///
/// Deliberately NOT "is the recorded commit an ancestor of HEAD". That predicate was removed from
/// the currency paths above with the reasoning recorded inline: it is a history-trust question,
/// it is `attest`'s job, and it is what made squash-merged changes permanently unfinalizable.
/// `ship-status` was the one caller that never got the change, which is #689.
pub fn recorded_verification_is_current(root: &Path, record: &ChangeRecord) -> bool {
    load_verification(root, record)
        .map(|evidence| verification_is_current(root, record, &evidence))
        .unwrap_or(false)
}

fn verification_is_current_checked(
    root: &Path,
    record: &ChangeRecord,
    evidence: &VerificationRecord,
) -> Result<(), String> {
    let project_digest = project_input_digest(root)?;
    verification_is_current_checked_with_project_digest(root, record, evidence, &project_digest)
}

fn verification_is_current_checked_with_project_digest(
    root: &Path,
    record: &ChangeRecord,
    evidence: &VerificationRecord,
    project_digest: &str,
) -> Result<(), String> {
    if !evidence.passed {
        return Err("latest verification evidence failed".into());
    }
    if !definition_digest_matches(root, record, &evidence.contract_digest)? {
        return Err("verification contract digest is stale".into());
    }
    validate_verification_execution_digest(root, record, evidence)?;
    if project_digest != evidence.workspace_digest {
        return Err("verification project-input digest is stale".into());
    }
    // Currency is a content question only: the evidence passed, the plan on disk is the
    // plan that was verified, and the tree on disk is the tree that was verified.
    //
    // The git-ancestry walk that used to follow — descendants of the verification commit,
    // filtered by the REQ-change-016 path allowlist — answered a different question: "can
    // this evidence be trusted as un-tampered history?" That is `attest`'s job, keyed to
    // commit SHAs in git notes rather than reconstructed from a working tree. Its side
    // effect was the documented deadlock where the lifecycle instructed an author to make a
    // commit its own gate then refused.
    Ok(())
}

fn verification_commit_is_accepted_current(root: &Path, evidence: &VerificationRecord) -> bool {
    let Some(commit) = evidence.commit.as_deref() else {
        return git_output(root, &["rev-parse", "HEAD"]).is_none();
    };
    Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn accepted_workspace_is_integrated(root: &Path, record: &ChangeRecord) -> bool {
    let Some(remote_default) = remote_default_ref(root) else {
        return false;
    };
    let head_is_integrated = Command::new("git")
        .args([
            "merge-base",
            "--is-ancestor",
            "HEAD",
            remote_default.as_str(),
        ])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !head_is_integrated {
        return false;
    }
    let workspace = format!("{CHANGES_PATH}/{}", record.id);
    let state = format!("{workspace}/state.json");
    let Ok(repo_workspace) = git_repo_relative_path(root, &workspace) else {
        return false;
    };
    let Ok(repo_state) = git_repo_relative_path(root, &state) else {
        return false;
    };
    let remote_state = format!("{remote_default}:{repo_state}");
    let repo_workspace_pathspec = format!(":(top,literal){repo_workspace}");
    let state_exists = Command::new("git")
        .args(["cat-file", "-e", remote_state.as_str()])
        .current_dir(root)
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    state_exists
        && Command::new("git")
            .args([
                "diff",
                "--quiet",
                remote_default.as_str(),
                "--",
                repo_workspace_pathspec.as_str(),
            ])
            .current_dir(root)
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
}

#[cfg(test)]
fn accepted_change_is_recorded_in_current_history(root: &Path, record: &ChangeRecord) -> bool {
    accepted_change_is_recorded_in_ref(root, record, "HEAD")
}

fn accepted_change_is_recorded_on_remote_default(root: &Path, record: &ChangeRecord) -> bool {
    remote_default_ref(root).is_some_and(|remote_default| {
        accepted_change_is_recorded_in_ref(root, record, &remote_default)
    })
}

fn accepted_change_is_recorded_in_ref(root: &Path, record: &ChangeRecord, reference: &str) -> bool {
    // A workflow-v2 change is created, verified, and archived inside ONE pull request. Squash-merge
    // that and the default branch receives a single commit in which the workspace is ALREADY under
    // the archive — `.specsync/changes/<id>/state.json` never appears on the default branch at all.
    // Asking only about the active path therefore fails for every squash-merged v2 change: measured
    // on this repository, the active path is present for 83 of 172 archives while the archive path
    // is present for 172 of 172. Ask about the archive too, and require the state each location can
    // actually hold.
    if recorded_in_ref_at(root, record, reference, ChangeState::Accepted) {
        return true;
    }
    if record.state == ChangeState::Archived
        && let Ok(workspace) = find_change_dir(root, &record.id)
    {
        let archived_state = portable_project_path(root, &workspace.join("state.json"));
        if recorded_state_path_in_ref(
            root,
            record,
            reference,
            &archived_state,
            ChangeState::Archived,
        ) {
            return true;
        }
    }
    false
}

fn recorded_in_ref_at(
    root: &Path,
    record: &ChangeRecord,
    reference: &str,
    expected: ChangeState,
) -> bool {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    recorded_state_path_in_ref(root, record, reference, &state, expected)
}

fn recorded_state_path_in_ref(
    root: &Path,
    record: &ChangeRecord,
    reference: &str,
    state: &str,
    expected: ChangeState,
) -> bool {
    let Ok(repo_state) = git_repo_relative_path(root, state) else {
        return false;
    };
    let top_state = format!(":(top,literal){repo_state}");
    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let Ok(history) = run_git_bounded(
        root,
        &[
            "log",
            "--format=%H",
            &max_count,
            reference,
            "--",
            top_state.as_str(),
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    ) else {
        return false;
    };
    if !history.status.success() {
        return false;
    }
    let history_text = String::from_utf8_lossy(&history.stdout);
    let commits: Vec<&str> = history_text
        .lines()
        .filter(|commit| !commit.is_empty())
        .collect();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return false;
    }
    commits.into_iter().any(|commit| {
        git_file_at_commit(root, commit, &repo_state)
            .ok()
            .flatten()
            .is_some_and(|snapshot| {
                serde_json::from_slice::<ChangeRecord>(&snapshot).is_ok_and(|historical| {
                    historical.id == record.id && historical.state == expected
                })
            })
    })
}

fn archived_finalization_tree_is_recorded(
    root: &Path,
    record: &ChangeRecord,
) -> Result<bool, String> {
    if record.state != ChangeState::Archived {
        return Ok(false);
    }
    let workspace = find_change_dir(root, &record.id)?;
    let project_workspace = portable_project_path(root, &workspace);
    let repository_workspace = git_repo_relative_path(root, &project_workspace)?;
    let workspace_pathspec = format!(":(top,literal){repository_workspace}");
    let status = run_git_bounded(
        root,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--",
            &workspace_pathspec,
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Ok(false);
    }
    let current_tree = git_output(
        root,
        &["rev-parse", &format!("HEAD:{repository_workspace}")],
    )
    .ok_or_else(|| "current archive tree is not committed at HEAD".to_string())?;
    let repository_state = format!("{repository_workspace}/state.json");
    let state_pathspec = format!(":(top,literal){repository_state}");
    let mut references = vec!["HEAD".to_string()];
    if let Some(remote_default) = remote_default_ref(root)
        && !references.contains(&remote_default)
    {
        references.push(remote_default);
    }
    let max_count = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let mut introductions = BTreeMap::new();
    for reference in references {
        let history = run_git_bounded(
            root,
            &[
                "log",
                "--format=%H",
                "--diff-filter=A",
                &max_count,
                &reference,
                "--",
                &state_pathspec,
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )?;
        if !history.status.success() {
            continue;
        }
        let commits = String::from_utf8_lossy(&history.stdout);
        let candidates: Vec<&str> = commits.lines().filter(|line| !line.is_empty()).collect();
        if candidates.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(format!(
                "archive history exceeds the deterministic {}-commit bound",
                MAX_TRUSTED_HISTORY_COMMITS
            ));
        }
        for candidate in candidates {
            if introductions.contains_key(candidate) {
                continue;
            }
            let Some(state) = git_file_at_commit(root, candidate, &repository_state)? else {
                continue;
            };
            let Ok(historical) = serde_json::from_slice::<ChangeRecord>(&state) else {
                continue;
            };
            if historical.id != record.id || historical.state != ChangeState::Archived {
                continue;
            }
            let parents =
                scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", candidate])
                    .map_err(|_| format!("failed to inspect archive introduction {candidate}"))?;
            let fields: Vec<&str> = parents.split_whitespace().collect();
            if fields.first().copied() != Some(candidate)
                || fields.len() < 2
                || fields.len().saturating_sub(1)
                    > lifecycle_validation_limits().scoped_review_max_parents
            {
                return Err(format!(
                    "archive introduction {candidate} has ambiguous parent history"
                ));
            }
            let mut absent_from_every_parent = true;
            for parent in &fields[1..] {
                let existing = run_scoped_review_git(
                    root,
                    &["ls-tree", "-z", parent, "--", &repository_workspace],
                )?;
                if !existing.status.success() {
                    return Err(format!(
                        "failed to inspect archive introduction parent {parent}"
                    ));
                }
                if !existing.stdout.is_empty() {
                    absent_from_every_parent = false;
                    break;
                }
            }
            if !absent_from_every_parent {
                continue;
            }
            let candidate_tree = git_output(
                root,
                &["rev-parse", &format!("{candidate}:{repository_workspace}")],
            );
            if let Some(candidate_tree) = candidate_tree {
                introductions.insert(candidate.to_string(), candidate_tree);
            }
        }
    }
    if introductions.len() > 1 {
        return Err(format!(
            "archive has {} reachable introduction commits; expected exactly one",
            introductions.len()
        ));
    }
    Ok(introductions
        .values()
        .next()
        .is_some_and(|tree| tree == &current_tree))
}

#[cfg(test)]
fn accepted_change_has_current_canonical_successors(root: &Path, record: &ChangeRecord) -> bool {
    let Ok(records) = list_changes_checked(root) else {
        return false;
    };
    let successors: Vec<_> = records
        .iter()
        .filter(|candidate| {
            candidate.id != record.id
                && !candidate.no_spec_change
                && (candidate.canonical_applied || candidate.state == ChangeState::Accepted)
                // `happens_after` is the production successor relation (CHG-0160 moved it off
                // the ordinal and onto `(created_at, id)`). This helper kept its own
                // ordinal-parsing copy, which returned false for any ID without an ordinal —
                // a second implementation of one concept, disagreeing with the first.
                && happens_after(candidate, record)
                && accepted_change_is_recorded_in_current_history(root, candidate)
        })
        .collect();
    if successors.is_empty() {
        return false;
    }
    let specs_governed = record.affected_specs.iter().all(|module| {
        successors
            .iter()
            .any(|candidate| candidate.affected_specs.contains(module))
    });
    let paths_governed = record.affected_paths.iter().all(|path| {
        successors
            .iter()
            .any(|candidate| record_covers_project_path(root, candidate, path))
    });
    specs_governed && paths_governed
}

/// Refusal naming the workspace that already owns a change ID.
///
/// A change ID is now derived from the description alone, so a duplicate description is a
/// duplicate identity. Naming the existing package and its description is what makes the
/// refusal actionable: the two changes may share no words if either ID went through the
/// reserved-name escape.
fn change_name_taken_error(root: &Path, id: &str) -> String {
    let location = find_change_dir(root, id)
        .map(|path| portable_project_path(root, &path))
        .unwrap_or_else(|_| portable_project_path(root, &change_dir(root, id)));
    let existing = load_change(root, id)
        .map(|record| {
            format!(
                "\n  it is: {} ({})",
                record.description,
                record.state.as_str()
            )
        })
        .unwrap_or_default();
    format!(
        "a change named `{id}` already exists\n  {location}{existing}\nA change ID is derived from its description, so this description is already taken. Rephrase it, or work on the existing change."
    )
}

/// The change ID a description mints.
///
/// The slug is now the whole path component, so a description that slugifies to nothing can
/// no longer be papered over with a shared fallback: under ordinals every such description
/// became a distinct `CHG-NNNN-untitled-change`, and without them the first one to be created
/// would permanently own the only ID any of them can produce. A team writing descriptions in
/// a non-Latin script would get exactly one change, ever.
fn mint_change_slug(description: &str) -> Result<String, String> {
    if !description.chars().any(|c| c.is_ascii_alphanumeric()) {
        // Not `escape_default()`: it renders every non-ASCII character as `\u{...}`, so the
        // one class of description this rejects is exactly the one it would render unreadable.
        let shown: String = description
            .trim()
            .chars()
            .filter(|character| !character.is_control())
            .take(80)
            .collect();
        return Err(format!(
            "cannot derive a change ID from `{shown}`: a change ID is the description slugified, and this description contains no ASCII letters or digits to slugify. Include a few ASCII words in the description."
        ));
    }
    let slug = slugify(description);
    validate_change_id(&slug)?;
    Ok(slug)
}

/// Create the change workspace for `slug`, which is also its ID.
///
/// The allocator this replaces looped up to 10,000 times looking for a free ordinal. With the
/// ID equal to the slug the candidate directory is loop-invariant, so the loop would have
/// issued 10,000 identical `mkdir`s and then reported `exhausted change sequence allocation
/// retries` for what is simply a taken name.
///
/// Refusing rather than disambiguating with a suffix is deliberate. The archive directory is
/// `<date>-<id>`, so two same-named changes archived on different days produce two archives
/// with one `record.id`; `find_change_dir` then reports ambiguous locations and every command
/// in the repository fails with no clean recovery. Refusing converts an unrecoverable
/// repository state into "retype the sentence". A `-2` suffix would also make the ID worse at
/// its only job, which is being typed into every later command.
///
/// The existence probe covers the archive as well as the active workspace, because archiving
/// empties the active directory and an exclusive `create_dir` cannot see an archived twin. The
/// exclusive `create_dir` is still what serializes two processes sharing one volume.
fn allocate_change_workspace(root: &Path, slug: &str) -> Result<(String, PathBuf), String> {
    let changes = root.join(CHANGES_PATH);
    fs::create_dir_all(&changes).map_err(|error| {
        format!(
            "failed to create active changes directory {}: {error}",
            changes.display()
        )
    })?;
    let id = slug.to_string();
    validate_change_id(&id)?;
    if find_change_dir(root, &id).is_ok() {
        return Err(change_name_taken_error(root, &id));
    }
    let dir = change_dir(root, &id);
    match fs::create_dir(&dir) {
        Ok(()) => Ok((id, dir)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(change_name_taken_error(root, &id))
        }
        Err(error) => Err(format!(
            "failed to create change workspace {}: {error}",
            dir.display()
        )),
    }
}

/// Longest slug this mints, in bytes.
///
/// The binding constraint is Windows `MAX_PATH` (260), not the 255-byte component limit that
/// bounds module names. The deepest path a change produces is
/// `.specsync/archive/changes/<slug>/deltas/<module>.md` — 26 + slug + 8 + up to 20 — so a
/// 120-byte slug yields 174 characters and still clears `MAX_PATH` inside an 80-character
/// repository root. A 255-byte slug yields 309 and exceeds it before any root prefix at all.
///
/// This was 80, and it truncated 82 of the 159 descriptions in this repository's own archive.
/// Raising it to 120 leaves 110 intact. It buys readability and nothing else: slug uniqueness
/// across those 159 saturates at 50 bytes, so every byte above that disambiguates nothing.
const MAX_SLUG_BYTES: usize = 120;

/// Stands in when a description slugifies to nothing.
///
/// Not `"change"`, which was the previous fallback and is itself a reserved directory name
/// under `is_reserved_module_name` — harmless while the directory was `CHG-0007-change`, and a
/// collision with `.specsync/changes/` the moment the slug becomes the whole component.
const EMPTY_SLUG_FALLBACK: &str = "untitled-change";

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            if slug.len() >= MAX_SLUG_BYTES {
                break;
            }
            slug.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !slug.is_empty() {
            if slug.len() >= MAX_SLUG_BYTES {
                break;
            }
            slug.push('-');
            separator = true;
        }
    }
    // Truncate at a word boundary. Counting bytes rather than input characters is what makes
    // the cap actually bound the path component; stopping mid-word is what made the old slugs
    // read like `...preserved-audited-guara`. Only trim back to a boundary when one is near
    // enough that the result stays legible.
    let mut slug = slug.trim_matches('-').to_string();
    if slug.len() >= MAX_SLUG_BYTES
        && let Some(boundary) = slug.rfind('-')
        && boundary * 4 >= MAX_SLUG_BYTES * 3
    {
        slug.truncate(boundary);
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        return EMPTY_SLUG_FALLBACK.into();
    }
    // `NUL`, `CON`, `COM1` and friends cannot be directory components on Windows, and the OS
    // matches them case-insensitively so lowercasing is not an escape. `change` and `specs`
    // are reserved here for a different reason — they would collide with the workspace layout.
    if crate::commands::is_reserved_module_name(slug) {
        return format!("{slug}-change");
    }
    slug.into()
}

fn title_from_description(value: &str) -> String {
    let trimmed = value.trim();
    let mut characters = trimmed.chars();
    match characters.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), characters.as_str()),
        None => "Untitled change".into(),
    }
}

fn split_values(value: &str) -> Vec<String> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn acceptance_criteria_values(value: &str) -> Result<Vec<String>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if matches!(
        serde_json::from_str::<serde_json::Value>(trimmed),
        Ok(serde_json::Value::Array(_))
    ) {
        let values: Vec<String> = serde_json::from_str(trimmed).map_err(|error| {
            format!("acceptance criteria JSON arrays must contain only strings: {error}")
        })?;
        if values.iter().any(|criterion| criterion.trim().is_empty()) {
            return Err("acceptance criteria must not contain empty strings".into());
        }
        return Ok(values
            .into_iter()
            .map(|criterion| criterion.trim().to_string())
            .collect());
    }
    Ok(vec![trimmed.to_string()])
}

fn is_yes(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "yes" | "y" | "true" | "1"
    )
}

fn require_state(
    record: &ChangeRecord,
    states: &[ChangeState],
    action: &str,
) -> Result<(), String> {
    if states.contains(&record.state) {
        return Ok(());
    }
    Err(format!(
        "cannot {action} while {} is {}; expected {}",
        record.id,
        record.state.as_str(),
        states
            .iter()
            .map(|state| state.as_str())
            .collect::<Vec<_>>()
            .join(" or ")
    ))
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    cached_git_text_query(root, args, false)
}

fn git_output_allow_empty(root: &Path, args: &[&str]) -> Option<String> {
    cached_git_text_query(root, args, true)
}

fn cached_git_text_query(root: &Path, args: &[&str], allow_empty: bool) -> Option<String> {
    let key = GitTextQueryCacheKey {
        allow_empty,
        arguments: args
            .iter()
            .map(|argument| (*argument).to_string())
            .collect(),
    };
    if let Some(value) = read_scope_value(root, |scope| scope.git_text_queries.get(&key).cloned()) {
        return value;
    }
    record_test_git_process();
    let output = run_git_bounded(root, args, None, MAX_GIT_COMMAND_OUTPUT_BYTES).ok()?;
    if !output.status.success() {
        update_read_scope(root, |scope| {
            if scope.git_text_queries.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
                scope.git_text_queries.insert(key, None);
            }
        });
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let result = if allow_empty || !value.is_empty() {
        Some(value)
    } else {
        None
    };
    update_read_scope(root, |scope| {
        if scope.git_text_queries.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            scope.git_text_queries.insert(key, result.clone());
        }
    });
    result
}

fn cached_git_status_success(root: &Path, args: &[&str]) -> Result<bool, String> {
    let key: Vec<String> = args
        .iter()
        .map(|argument| (*argument).to_string())
        .collect();
    if let Some(value) = read_scope_value(root, |scope| scope.git_status_queries.get(&key).cloned())
    {
        return value;
    }
    let result = run_git_bounded(root, args, None, 1024).map(|output| output.status.success());
    update_read_scope(root, |scope| {
        if scope.git_status_queries.len() < MAX_CHANGE_READ_CACHE_ENTRIES {
            scope.git_status_queries.insert(key, result.clone());
        }
    });
    result
}

fn git_repo_relative_path(root: &Path, project_path: &str) -> Result<String, String> {
    Ok(format!("{}{project_path}", git_repo_prefix(root)?))
}

fn git_repo_prefix(root: &Path) -> Result<String, String> {
    let prefix = git_output_allow_empty(root, &["rev-parse", "--show-prefix"])
        .ok_or_else(|| "unable to determine project path within Git repository".to_string())?
        .replace('\\', "/");
    Ok(prefix)
}

fn write_json<Value: Serialize>(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, json_content(value)?)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn bytes_match_canonical_json(on_disk: &[u8], canonical: &[u8]) -> bool {
    if on_disk == canonical {
        return true;
    }
    // Accept CRLF-only working-tree rewrites of an otherwise exact LF canonical file.
    let lf_only: Vec<u8> = on_disk
        .iter()
        .copied()
        .filter(|&byte| byte != b'\r')
        .collect();
    lf_only.as_slice() == canonical
}

fn json_content<Value: Serialize>(value: &Value) -> Result<String, String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    Ok(format!("{content}\n"))
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn legacy_workflow_version() -> u32 {
    1
}

fn is_legacy_workflow_version(value: &u32) -> bool {
    *value == legacy_workflow_version()
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn today() -> String {
    let days = (now() / 86_400) as i64;
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i64 + era * 400;
    let day_of_year = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
#[path = "change_tests.rs"]
mod tests;
