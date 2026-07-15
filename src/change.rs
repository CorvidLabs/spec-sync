use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

pub const SDD_VERSION: &str = "5.0.0";
const POLICY_PATH: &str = ".specsync/sdd.json";
const CHANGES_PATH: &str = ".specsync/changes";
const ARCHIVE_PATH: &str = ".specsync/archive/changes";
const LEGACY_BASELINE_PATH: &str = ".specsync/archive/legacy-baseline.json";
const LOCK_PATH: &str = ".specsync/change.lock";
const SEQUENCE_PATH: &str = ".specsync/change-sequence.json";
const TRANSACTION_PATH: &str = ".specsync/change-transaction.json";
const CORRECTIONS_FILE: &str = "corrections.json";
const MAX_CHANGE_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024;
const CANONICAL_SPEC_COMPANIONS: [&str; 5] = [
    "requirements.md",
    "tasks.md",
    "context.md",
    "testing.md",
    "design.md",
];
static EFFECTIVE_CONTRACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static TRUSTED_CORRECTION_HISTORY_CACHE: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

const DEFINITION_DIGEST_DOMAIN: &[u8] = b"specsync.definition-digest.v2";
const PROJECT_DIGEST_DOMAIN: &[u8] = b"specsync.project-input-digest.v2";
const ACCEPTANCE_DIGEST_DOMAIN: &[u8] = b"specsync.acceptance-input-digest.v2";
const ACCEPTANCE_ENTRY_DOMAIN: &[u8] = b"specsync.acceptance-entry.v1";
const ACCEPTANCE_MANIFEST_DOMAIN: &[u8] = b"specsync.acceptance-manifest.v1";
const SEMANTIC_SUCCESSION_DOMAIN: &[u8] = b"specsync.semantic-succession.v1";
const LEGACY_BASELINE_DOMAIN: &[u8] = b"specsync.legacy-archive-baseline.v1";
const LEGACY_SUBTREE_DOMAIN: &[u8] = b"specsync.legacy-archive-subtree.v1";
const CLOSING_DIGEST_DOMAIN: &[u8] = b"specsync.closing-digest.v2";
const CORRECTION_VIEW_DIGEST_DOMAIN: &[u8] = b"specsync.correction-view-digest.v1";
const EXACT_TEST_OWNER: &str = "@exact:test";
const EXACT_DELIVERY_OWNER: &str = "@exact:delivery";
const MAX_ACCEPTANCE_ENTRIES: usize = 100_000;
const MAX_ACCEPTANCE_PATH_BYTES: usize = 4_096;
const MAX_ACCEPTANCE_OWNERS: usize = 1_024;
const MAX_ACCEPTANCE_OWNER_BYTES: usize = 256;

struct FramedDigest {
    hasher: Sha256,
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
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

#[derive(Debug, Serialize, Deserialize)]
struct TransactionEntry {
    path: String,
    original: Option<String>,
}

fn recover_pending_transaction(root: &Path) -> Result<(), String> {
    let journal = root.join(TRANSACTION_PATH);
    if !journal.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&journal)
        .map_err(|error| format!("failed to read transaction journal: {error}"))?;
    let entries: Vec<TransactionEntry> = serde_json::from_str(&content)
        .map_err(|error| format!("invalid transaction journal: {error}"))?;
    for entry in entries {
        let path = safe_project_path(root, &entry.path)?;
        if let Some(original) = entry.original {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::write(&path, original)
                .map_err(|error| format!("failed to restore {}: {error}", path.display()))?;
        } else if path.exists() {
            fs::remove_file(&path)
                .map_err(|error| format!("failed to remove {}: {error}", path.display()))?;
        }
    }
    fs::remove_file(&journal)
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            version: 1,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_archive_baseline_digest: Option<String>,
    pub answers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyArchiveBaselineV1 {
    pub schema_version: u32,
    pub domain: String,
    pub authority_change_id: String,
    pub cutoff_commit: String,
    pub entries: Vec<LegacyArchiveBaselineEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub gate: String,
    pub actor: String,
    pub timestamp: u64,
    pub digest: String,
    pub note: Option<String>,
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
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    #[serde(default)]
    pub reopenings: Vec<ReopenRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEvidence {
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub timestamp: u64,
    pub commit: Option<String>,
    pub contract_digest: String,
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
    sequence: u64,
    id: String,
    path: String,
    historical: bool,
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
    pub artifacts_complete: bool,
    #[serde(default)]
    pub correction_valid: bool,
    #[serde(default)]
    pub correction_count: usize,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub corrected_fields: BTreeMap<String, String>,
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
    write_json(&path, &policy)
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
    let mut affected_paths: Vec<String> = affected_paths
        .iter()
        .map(|path| {
            normalize_project_path(path).map_err(|error| format!("invalid affected path: {error}"))
        })
        .collect::<Result<_, _>>()?;
    if !affected_paths
        .iter()
        .any(|scope| path_matches_scope(SEQUENCE_PATH, scope))
    {
        affected_paths.push(SEQUENCE_PATH.into());
    }
    let slug = slugify(&description);
    let id = next_change_id(root, &slug)?;
    let now = now();
    let mut artifacts = adaptive_artifacts(kind, &affected_specs, &affected_paths);
    for artifact in requested_artifacts {
        if !artifacts.contains(&artifact) {
            artifacts.push(artifact);
        }
    }
    let record = ChangeRecord {
        schema_version: 1,
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
        legacy_archive_baseline_digest: None,
        answers: BTreeMap::new(),
    };
    let dir = change_dir(root, &record.id);
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
    update_change_sequence_claim(root, &record.id)?;
    Ok(record)
}

pub fn load_change(root: &Path, id: &str) -> Result<ChangeRecord, String> {
    let path = find_change_dir(root, id)?.join("state.json");
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let record = serde_json::from_str(&content)
        .map_err(|error| format!("invalid change state {}: {error}", path.display()))?;
    validate_loaded_change(&record, id, &path)?;
    Ok(record)
}

pub fn list_changes(root: &Path) -> Vec<ChangeRecord> {
    list_changes_checked(root).unwrap_or_default()
}

fn list_changes_checked(root: &Path) -> Result<Vec<ChangeRecord>, String> {
    let mut records = Vec::new();
    let dir = root.join(CHANGES_PATH);
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(format!("failed to read active changes: {error}")),
    };
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to read active change entry: {error}"))?;
        if !entry
            .file_type()
            .map_err(|error| format!("failed to inspect active change entry: {error}"))?
            .is_dir()
        {
            continue;
        }
        let expected_id = entry.file_name().into_string().map_err(|_| {
            format!(
                "active change directory is not valid UTF-8: {}",
                entry.path().display()
            )
        })?;
        let path = entry.path().join("state.json");
        let content = fs::read_to_string(&path).map_err(|error| {
            format!(
                "failed to read active change state {}: {error}",
                path.display()
            )
        })?;
        let record = serde_json::from_str(&content)
            .map_err(|error| format!("invalid active change state {}: {error}", path.display()))?;
        validate_loaded_change(&record, &expected_id, &path)?;
        records.push(record);
    }
    records.sort_by(|left: &ChangeRecord, right: &ChangeRecord| left.id.cmp(&right.id));
    Ok(records)
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

#[cfg(test)]
fn change_id_sorts_after(candidate: &str, predecessor: &str) -> bool {
    match (change_sequence(candidate), change_sequence(predecessor)) {
        (Some(candidate_sequence), Some(predecessor_sequence)) => {
            (candidate_sequence, candidate) > (predecessor_sequence, predecessor)
        }
        _ => false,
    }
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

fn update_change_sequence_claim(root: &Path, id: &str) -> Result<(), String> {
    let sequence = change_sequence(id).ok_or_else(|| format!("invalid change ID `{id}`"))?;
    let acknowledged_collisions = load_change_sequence_ledger(root)?
        .map(|ledger| ledger.acknowledged_collisions)
        .unwrap_or_default();
    write_json(
        &root.join(SEQUENCE_PATH),
        &ChangeSequenceLedger {
            schema_version: 1,
            sequence,
            id: id.to_string(),
            acknowledged_collisions,
        },
    )
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
                        && is_positive_legacy_tombstone(&entry.path()) =>
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
            let sequence = change_sequence(&record.id)
                .ok_or_else(|| format!("invalid change ID `{}`", record.id.escape_default()))?;
            let historical = matches!(record.state, ChangeState::Accepted | ChangeState::Archived);
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

fn validate_change_sequences(root: &Path) -> Result<(), String> {
    let located = located_change_sequences(root)?;
    let ledger = load_change_sequence_ledger(root)?;
    let mut groups: BTreeMap<u64, Vec<&LocatedChangeSequence>> = BTreeMap::new();
    for change in &located {
        groups.entry(change.sequence).or_default().push(change);
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
                "duplicate numeric change sequence CHG-{sequence:04}: {conflicts}; update from the default branch and create a new change ID"
            ));
        }
    }
    if let Some(ledger) = ledger {
        let maximum = located
            .iter()
            .map(|change| change.sequence)
            .max()
            .unwrap_or(0);
        if maximum != ledger.sequence {
            return Err(format!(
                "change sequence ledger claims CHG-{:04} but the highest recorded sequence is CHG-{maximum:04}",
                ledger.sequence
            ));
        }
        if !located.iter().any(|change| change.id == ledger.id) {
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
    if !record
        .affected_paths
        .iter()
        .any(|path| path != SEQUENCE_PATH)
    {
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

pub fn answer_question(
    root: &Path,
    id: &str,
    question: &str,
    answer: &str,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
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
            let mut affected_paths: Vec<String> = values
                .iter()
                .map(|path| {
                    normalize_project_path(path)
                        .map_err(|error| format!("invalid affected path: {error}"))
                })
                .collect::<Result<_, _>>()?;
            if !affected_paths
                .iter()
                .any(|scope| path_matches_scope(SEQUENCE_PATH, scope))
            {
                affected_paths.push(SEQUENCE_PATH.into());
            }
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
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    ensure_artifact_files(root, &record)?;
    Ok(record)
}

pub fn add_dependency(root: &Path, id: &str, dependency: &str) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
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
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(record)
}

pub fn add_supersedes_obligation(
    root: &Path,
    id: &str,
    predecessor: &str,
    path: &str,
    module: &str,
    predecessor_entry_digest: &str,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
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
    record
        .supersedes
        .sort_by_key(|edge| succession_change_key(&edge.predecessor_id));
    validate_supersedes_edges(&record)?;
    validate_supersedes_semantics(root, &record)?;
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(record)
}

pub fn approve_definition(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
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
    list_changes_checked(root)?;
    bind_legacy_archive_baseline_authority(root, &mut record)?;
    let prior_state = record.state;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    let digest = definition_digest(root, &record)?;
    append_approval(root, &record, "definition", actor, digest, note)?;
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
    let baseline_bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("failed to read legacy archive baseline: {error}")),
    };
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
    require_state(&record, &[ChangeState::Approved], "start implementation")?;
    ensure_definition_approval_valid(root, &record)?;
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    record.state = ChangeState::Implementing;
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(record)
}

pub fn verify_change(root: &Path, id: &str) -> Result<VerificationRecord, String> {
    if let Some(error) = crate::verification_recursion_error() {
        return Err(error);
    }
    let _lock = acquire_project_lock(root)?;
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
    validate_effective_contracts(root, &records).map_err(|errors| errors.join("; "))?;
    ensure_tasks_complete(root, &record)?;
    let policy = load_policy_checked(root)?.unwrap_or_default();
    for configured in &policy.verification_commands {
        reject_direct_lifecycle_verification(root, configured)?;
    }
    let mut commands = Vec::new();
    for configured in policy.verification_commands {
        let status = run_configured_command(root, &configured, ConfiguredCommandOutput::Inherit)?;
        commands.push(CommandEvidence {
            command: configured,
            success: status.success(),
            exit_code: status.code(),
        });
        if !status.success() {
            break;
        }
    }
    let requirement_ids = collect_requirement_ids(root, &record)?;
    let has_semantic_acceptance_item = semantic_acceptance_item_exists(root, &record)?;
    let missing_evidence = requirement_evidence_missing(root, &record, &requirement_ids);
    let commands_passed = commands.iter().all(|command| command.success);
    let acceptance_evidence_present =
        acceptance_criteria_have_evidence(&record, has_semantic_acceptance_item);
    let passed = commands_passed && acceptance_evidence_present && missing_evidence.is_empty();
    let verification = VerificationRecord {
        timestamp: now(),
        commit: git_output(root, &["rev-parse", "HEAD"]),
        contract_digest: definition_digest(root, &record)?,
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
    write_change_markdown(root, &record)?;
    if !verification.passed {
        let detail = if !commands_passed {
            "configured verification command failed".to_string()
        } else if !acceptance_evidence_present {
            "semantic acceptance evidence is missing".to_string()
        } else {
            format!(
                "requirement evidence missing for {}",
                missing_evidence.join(", ")
            )
        };
        return Err(format!(
            "verification failed: {detail}; inspect verification.json"
        ));
    }
    Ok(verification)
}

pub fn reopen_change(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
) -> Result<ReopenResult, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Accepted],
        "reopen accepted evidence",
    )?;
    let actor = actor.trim();
    if actor.is_empty() {
        return Err("reopen requires a non-empty human actor passed with --actor".into());
    }
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("reopen requires a non-empty reason passed with --reason".into());
    }
    ensure_definition_approval_valid(root, &record)?;
    let prior_verification = load_verification(root, &record)?;
    if !prior_verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    if !definition_digest_matches(root, &record, &prior_verification.contract_digest)? {
        return Err("accepted change verification contract is stale; restore the accepted definition before reopening delivery evidence".into());
    }
    let stale_acceptance_input_digest = prior_verification
        .acceptance_input_digest
        .clone()
        .ok_or_else(|| "accepted change is missing current delivery-input evidence".to_string())?;
    let expected_closing_digest = closing_digest(&record, &prior_verification);
    let mut ledger = load_approvals(root, &record)?;
    let superseded_approval = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .cloned()
        .ok_or_else(|| "accepted change is missing closing approval".to_string())?;
    if superseded_approval.digest != expected_closing_digest {
        return Err("accepted change closing approval does not match verification evidence".into());
    }
    let current_acceptance_input_digest =
        if let Some(manifest) = &prior_verification.acceptance_manifest {
            let current = acceptance_manifest_with_signed_owners(root, &record, &[], manifest)?;
            acceptance_manifest_digest(&current)?
        } else {
            acceptance_input_digest(root, &record, &[])?
        };
    if current_acceptance_input_digest == stale_acceptance_input_digest {
        return Err(
            "accepted change delivery inputs are current (exact or successor-covered); reopen is allowed only when delivery evidence is stale"
                .into(),
        );
    }
    authenticate_accepted_evidence(root, &record)?;
    let records = list_all_changes_checked(root)?;
    let mut visiting = BTreeSet::new();
    let mut memo = BTreeMap::new();
    if validate_accepted_inputs_recursive(root, &record, &records, &mut visiting, &mut memo).is_ok()
    {
        return Err(
            "accepted change delivery inputs are current (exact or successor-covered); reopen is allowed only when delivery evidence is stale"
                .into(),
        );
    }
    let audit = ReopenRecord {
        schema_version: 1,
        change_id: record.id.clone(),
        actor: actor.to_string(),
        reason: reason.to_string(),
        timestamp: now(),
        from_state: ChangeState::Accepted,
        to_state: ChangeState::Verifying,
        superseded_approval,
        prior_verification,
        stale_acceptance_input_digest,
        current_acceptance_input_digest,
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
            || correction.superseded_closing_approval.gate != "acceptance"
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
    let ledger = if path.exists() {
        let content = read_bounded_change_text(&path, "correction ledger")?;
        serde_json::from_str(&content)
            .map_err(|error| format!("invalid correction ledger {}: {error}", path.display()))?
    } else {
        CorrectionLedger::default()
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
    let reference_args: Vec<&str> = references.iter().map(String::as_str).collect();
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
    let mut correction_probe = Command::new("git");
    correction_probe
        .args(["rev-list", "--full-history", "--max-count=1"])
        .args(&reference_args);
    if let Some(exclusion) = &history_exclusion {
        correction_probe.arg(exclusion);
    }
    let correction_probe = correction_probe
        .arg("--")
        .arg(&active_corrections)
        .arg(&archive_corrections_glob)
        .current_dir(root)
        .output()
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
    let output = Command::new("git")
        .args(["rev-list", "--full-history"])
        .args(&reference_args)
        .args(history_exclusion)
        .arg("--")
        .arg(&active_directory)
        .arg(&archive_glob)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect trusted correction history: {error}"))?;
    if !output.status.success() {
        return Err("failed to enumerate trusted correction history".into());
    }
    let commits: BTreeSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|commit| !commit.is_empty())
        .map(str::to_string)
        .collect();
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
                .status()
                .is_ok_and(|status| status.success())
        });
        if reachable
            && !Command::new("git")
                .args(["merge-base", "--is-ancestor", boundary, base])
                .current_dir(root)
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
    let definition_matches = approvals
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .is_some_and(|approval| approval.digest == verification.contract_digest);
    let closing_matches = approvals
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
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

        let mut digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
        digest.frame(b"record", &record_bytes);
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
            digest.entry(&local_path, kind, mode, &content);
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
            digest.entry(
                &local_path,
                b"file",
                mode,
                json_content(corrections)?.as_bytes(),
            );
        }
        if digest.finish() == expected {
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
    let output = Command::new("git")
        .args(["show", object.as_str()])
        .current_dir(root)
        .output()
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
    let superseded_definition_approval = approvals
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .cloned()
        .ok_or_else(|| "accepted change is missing definition approval".to_string())?;
    let superseded_closing_approval = approvals
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
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
    let approval_ledger = load_approvals(root, record)?;
    let correction_ledger = load_correction_ledger(root, record)?;
    validate_correction_records(record, &correction_ledger.corrections)?;

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

pub fn accept_change(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Verifying], "accept the change")?;
    ensure_definition_approval_valid(root, &record)?;
    let mut verification = load_verification(root, &record)?;
    if !verification.passed {
        return Err("cannot accept a change with failed verification".into());
    }
    let current_commit = git_output(root, &["rev-parse", "HEAD"]);
    if verification.commit != current_commit {
        return Err(
            "verification is stale because HEAD changed; run `specsync change verify` again".into(),
        );
    }
    if !definition_digest_matches(root, &record, &verification.contract_digest)? {
        return Err("verification is stale because the approved contract changed".into());
    }
    if verification.workspace_digest != project_input_digest(root)? {
        return Err(
            "verification is stale because tested working-tree inputs changed; run `specsync change verify` again"
                .into(),
        );
    }
    ensure_dependencies_satisfied(root, &record)?;
    ensure_no_delta_conflicts(root, &record)?;
    validate_delta_files(root, &record)?;
    let records = list_changes_checked(root)?;
    validate_effective_contracts(root, &records).map_err(|errors| errors.join("; "))?;
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
    let actor = resolve_actor(root, actor)?;
    let stable_definition_digest = definition_digest(root, &record)?;
    let latest_definition_digest = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .map(|approval| approval.digest.as_str());
    if latest_definition_digest != Some(stable_definition_digest.as_str()) {
        ledger.approvals.push(ApprovalRecord {
            gate: "definition".into(),
            actor: actor.clone(),
            timestamp: now(),
            digest: stable_definition_digest,
            note: Some(
                "Normalized compatible definition evidence during explicit acceptance".into(),
            ),
        });
    }
    ledger.approvals.push(ApprovalRecord {
        gate: "acceptance".into(),
        actor,
        timestamp: now(),
        digest: closing_digest,
        note,
    });
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    prepared.push((approvals_path, json_content(&ledger)?));
    prepared.push((
        change_dir(root, &record.id).join("verification.json"),
        json_content(&verification)?,
    ));
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

pub fn archive_change(root: &Path, id: &str) -> Result<PathBuf, String> {
    archive_change_with_finalize_failure(root, id, false)
}

fn archive_change_with_finalize_failure(
    root: &Path,
    id: &str,
    force_finalize_failure: bool,
) -> Result<PathBuf, String> {
    let _lock = acquire_project_lock(root)?;
    list_changes_checked(root)?;
    let record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Accepted], "archive the change")?;
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{}", today(), record.id));
    if destination.exists() {
        return Err(format!(
            "archive destination already exists: {}",
            destination.display()
        ));
    }
    ensure_closing_approval_valid(root, &record)?;
    if let Some(policy) = load_policy_checked(root)?
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
    let source = change_dir(root, &record.id);
    let original_state_bytes = fs::read(source.join("state.json"))
        .map_err(|error| format!("failed to preserve accepted state before archive: {error}"))?;
    let original_markdown_bytes = fs::read(source.join("change.md"))
        .map_err(|error| format!("failed to preserve accepted change before archive: {error}"))?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let accepted_snapshot = source.join("accepted-state.json");
    let accepted_state_bytes = authenticated_accepted_transition(root, &record)
        .map(|(_, bytes, _)| bytes)
        .unwrap_or_else(|_| original_state_bytes.clone());
    fs::write(&accepted_snapshot, &accepted_state_bytes)
        .map_err(|error| format!("failed to stage authenticated accepted state: {error}"))?;
    let mut simulated = list_all_changes_checked(root)?;
    let mut archived_projection = record.clone();
    archived_projection.state = ChangeState::Archived;
    if let Err(error) = validate_archived_integrity(root, &archived_projection) {
        let _ = fs::remove_file(&accepted_snapshot);
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
            let _ = fs::remove_file(&accepted_snapshot);
            return Err(format!(
                "archive post-move preflight would invalidate `{}`: {error}",
                candidate.id
            ));
        }
    }
    if let Err(error) = fs::rename(&source, &destination) {
        let _ = fs::remove_file(&accepted_snapshot);
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
        write_json(&destination.join("state.json"), &archived).and_then(|()| {
            fs::write(
                destination.join("change.md"),
                change_markdown_content(&archived),
            )
            .map_err(|error| error.to_string())
        })
    };
    if let Err(error) = finalize {
        let restore = fs::write(destination.join("state.json"), &original_state_bytes)
            .map_err(|error| error.to_string())
            .and_then(|()| {
                fs::write(destination.join("change.md"), &original_markdown_bytes)
                    .map_err(|error| error.to_string())
            })
            .and_then(|()| {
                fs::remove_file(destination.join("accepted-state.json"))
                    .map_err(|error| error.to_string())
            })
            .and_then(|()| fs::rename(&destination, &source).map_err(|error| error.to_string()));
        return match restore {
            Ok(()) => Err(format!(
                "failed to finalize archive; source restored: {error}"
            )),
            Err(restore_error) => Err(format!(
                "failed to finalize archive ({error}) and restore source ({restore_error})"
            )),
        };
    }
    if let Err(error) = validate_archived_integrity(root, &archived) {
        let restore = fs::write(destination.join("state.json"), &original_state_bytes)
            .map_err(|error| error.to_string())
            .and_then(|()| {
                fs::write(destination.join("change.md"), &original_markdown_bytes)
                    .map_err(|error| error.to_string())
            })
            .and_then(|()| {
                fs::remove_file(destination.join("accepted-state.json"))
                    .map_err(|error| error.to_string())
            })
            .and_then(|()| fs::rename(&destination, &source).map_err(|error| error.to_string()));
        return match restore {
            Ok(()) => Err(format!(
                "archived evidence failed post-move validation; source restored: {error}"
            )),
            Err(restore_error) => Err(format!(
                "archived evidence failed validation ({error}) and restore ({restore_error})"
            )),
        };
    }
    Ok(destination)
}

pub fn summarize_change(root: &Path, record: &ChangeRecord) -> ChangeSummary {
    let effective = effective_change_definition(root, record);
    let correction_valid = effective.is_ok();
    let corrected_fields = effective
        .as_ref()
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
    let artifacts_complete = validate_artifacts(root, record).is_ok();
    let verification_current = || {
        let Ok(verification) = load_verification(root, record) else {
            return false;
        };
        verification.passed
            && definition_digest_matches(root, record, &verification.contract_digest)
                .unwrap_or(false)
            && verification.commit == git_output(root, &["rev-parse", "HEAD"])
            && project_input_digest(root)
                .is_ok_and(|digest| verification.workspace_digest == digest)
    };
    let terminal_evidence = matches!(record.state, ChangeState::Accepted | ChangeState::Archived)
        .then(|| terminal_evidence_summary(root, record));
    let next_action = match record.state {
        ChangeState::Draft if next_questions(record).is_empty() => "approve".into(),
        ChangeState::Draft => "answer interview".into(),
        ChangeState::Approved => "start".into(),
        ChangeState::Implementing if !artifacts_complete => "complete artifacts".into(),
        ChangeState::Implementing if !approval_valid => "approve".into(),
        ChangeState::Implementing => "verify".into(),
        ChangeState::Verifying if !artifacts_complete => "complete artifacts".into(),
        ChangeState::Verifying if !approval_valid => "approve".into(),
        ChangeState::Verifying if !verification_current() => "verify".into(),
        ChangeState::Verifying => "accept".into(),
        ChangeState::Accepted if !correction_valid => "repair correction ledger".into(),
        ChangeState::Accepted if ensure_closing_approval_valid(root, record).is_err() => {
            "reopen".into()
        }
        ChangeState::Accepted
            if terminal_evidence
                .as_ref()
                .is_some_and(|evidence| evidence.validity == TerminalEvidenceValidity::Stale) =>
        {
            "reopen".into()
        }
        ChangeState::Accepted => "archive".into(),
        ChangeState::Archived
            if terminal_evidence.as_ref().is_some_and(|evidence| {
                evidence.validity == TerminalEvidenceValidity::CorruptHistory
            }) =>
        {
            "invalid archived evidence".into()
        }
        ChangeState::Archived => "none".into(),
    };
    ChangeSummary {
        id: record.id.clone(),
        title: record.title.clone(),
        state: record.state,
        approval_valid,
        artifacts_complete,
        correction_valid,
        correction_count: record.correction_count as usize,
        corrected_fields,
        next_action,
        terminal_evidence,
    }
}

fn policy_at_comparison_base(root: &Path) -> Result<Option<SddPolicy>, String> {
    let records = list_changes_checked(root).unwrap_or_default();
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

pub fn check_project(root: &Path) -> SddCheckReport {
    check_project_with_command_output(root, ConfiguredCommandOutput::Inherit)
}

/// Check SDD lifecycle state without allowing configured verification commands
/// to write into a machine-consumed report stream.
pub(crate) fn check_project_quiet(root: &Path) -> SddCheckReport {
    check_project_with_command_output(root, ConfiguredCommandOutput::Suppress)
}

fn check_project_with_command_output(
    root: &Path,
    command_output: ConfiguredCommandOutput,
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
    let all_records = match list_all_changes_checked(root) {
        Ok(records) => records,
        Err(error) => {
            return SddCheckReport {
                enabled: true,
                errors: vec![error],
                ..SddCheckReport::default()
            };
        }
    };
    let mut report = SddCheckReport {
        enabled: true,
        checked_changes: all_records.len(),
        terminal_evidence: all_records
            .values()
            .filter(|record| matches!(record.state, ChangeState::Accepted | ChangeState::Archived))
            .map(|record| TerminalEvidenceResult {
                id: record.id.clone(),
                evidence: terminal_evidence_summary_with_records(root, record, &all_records),
            })
            .collect(),
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
            return SddCheckReport::default();
        };
        if !policy.enabled {
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
        if !matches!(record.state, ChangeState::Draft)
            && let Err(error) = ensure_definition_approval_valid(root, record)
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
            && let Err(error) = ensure_closing_approval_valid(root, record)
        {
            report.errors.push(format!(
                "{}: accepted change verification is stale for current delivery inputs: {error}",
                record.id
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
        for dependency in &record.dependencies {
            if dependency_reaches(root, dependency, &record.id, &mut BTreeSet::new()) {
                report.errors.push(format!(
                    "{}: change dependency cycle through `{dependency}`",
                    record.id
                ));
            }
        }
    }
    for record in all_records
        .values()
        .filter(|record| record.state == ChangeState::Archived)
    {
        if let Err(error) = validate_archived_integrity(root, record) {
            report.errors.push(format!(
                "{}: archived change historical integrity is invalid: {error}",
                record.id
            ));
        }
    }
    if let Err(errors) = validate_effective_contracts(root, &records) {
        report.errors.extend(errors);
    }
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
                    } else if !verification_commit_is_current(root, &evidence, is_ci_project(root))
                        || definition_digest_matches(root, record, &evidence.contract_digest)
                            .map(|matches| !matches)
                            .unwrap_or(true)
                        || project_input_digest(root)
                            .map(|digest| digest != evidence.workspace_digest)
                            .unwrap_or(true)
                    {
                        report.errors.push(format!(
                            "{}: verification evidence is stale for the current commit or contract",
                            record.id
                        ));
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
        for configured in &policy.verification_commands {
            match run_configured_command(root, configured, command_output) {
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
                    report.errors.push(format!(
                        "meaningful changed paths are not covered by an active change: {}",
                        paths.join(", ")
                    ));
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
    if let Some(source) = detected.as_deref() {
        validate_foreign_import(root, source)?;
    }
    let policy_existed = root.join(POLICY_PATH).exists();
    let existing_bootstrap = fs::read_to_string(root.join(".specsync/adoption-report.json"))
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|value| value.get("bootstrap_policy").cloned());
    write_default_policy(root, detect_verification_commands(root))?;
    let bootstrap_policy = if policy_existed {
        existing_bootstrap
    } else {
        adoption_bootstrap_record(root)?
    };
    write_json(
        &root.join(".specsync/adoption-report.json"),
        &serde_json::json!({
            "requirements_needing_ids": requirement_proposals,
            "generated_at": now(),
            "bootstrap_policy": bootstrap_policy,
        }),
    )?;
    if let Some(source) = detected.as_deref() {
        import_foreign(root, source)?;
    }
    Ok(actions)
}

fn adoption_bootstrap_record(root: &Path) -> Result<Option<serde_json::Value>, String> {
    let Some(base_commit) = git_output(root, &["rev-parse", "--verify", "HEAD"]) else {
        return Ok(None);
    };
    let content = fs::read(root.join(POLICY_PATH))
        .map_err(|error| format!("failed to read adopted SDD policy: {error}"))?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(Some(serde_json::json!({
        "path": POLICY_PATH,
        "digest": format!("{:x}", hasher.finalize()),
        "base_commit": base_commit,
    })))
}

fn adoption_bootstrap_covers_policy(root: &Path) -> bool {
    let Ok(content) = fs::read_to_string(root.join(".specsync/adoption-report.json")) else {
        return false;
    };
    let Ok(report) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    let Some(bootstrap) = report.get("bootstrap_policy") else {
        return false;
    };
    if bootstrap.get("path").and_then(serde_json::Value::as_str) != Some(POLICY_PATH) {
        return false;
    }
    let Some(base_commit) = bootstrap
        .get("base_commit")
        .and_then(serde_json::Value::as_str)
    else {
        return false;
    };
    if git_output(root, &["rev-parse", "--verify", base_commit]).is_none() {
        return false;
    }
    let ancestor_status = Command::new("git")
        .args(["merge-base", "--is-ancestor", base_commit, "HEAD"])
        .current_dir(root)
        .status();
    if !ancestor_status.is_ok_and(|status| status.success()) {
        return false;
    }
    let Ok(tree_path) = git_repo_relative_path(root, POLICY_PATH) else {
        return false;
    };
    let object = format!("{base_commit}:{tree_path}");
    if git_output_allow_empty(root, &["show", &object]).is_some() {
        return false;
    }
    let Ok(policy) = fs::read(root.join(POLICY_PATH)) else {
        return false;
    };
    let mut hasher = Sha256::new();
    hasher.update(policy);
    let current_digest = format!("{:x}", hasher.finalize());
    bootstrap.get("digest").and_then(serde_json::Value::as_str) == Some(current_digest.as_str())
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
    let mut previous_edge: Option<(u64, &str)> = None;
    for edge in &record.supersedes {
        let key = succession_change_key(&edge.predecessor_id);
        if previous_edge.is_some_and(|previous| previous >= (key.0, key.1.as_str())) {
            return Err("supersedes edges must be strictly sorted by numeric sequence and full predecessor ID".into());
        }
        previous_edge = Some((key.0, edge.predecessor_id.as_str()));
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
        if succession_change_key(&edge.predecessor_id) >= succession_change_key(&record.id) {
            return Err(format!(
                "superseded change `{}` must sort before successor `{}`",
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

fn succession_change_key(id: &str) -> (u64, String) {
    (change_sequence(id).unwrap_or(u64::MAX), id.to_string())
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

fn validate_artifacts(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let dir = find_change_dir(root, &record.id)?;
    let effective = effective_change_definition(root, record)?;
    for artifact in &effective.selected_artifacts {
        let path = dir.join(artifact.file_name());
        let content = read_bounded_change_text(&path, "artifact")?;
        if content.contains("<!-- TODO") || content.trim().is_empty() {
            return Err(format!("artifact is incomplete: {}", path.display()));
        }
    }
    Ok(())
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
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    if content
        .lines()
        .any(|line| line.trim_start().starts_with("- [ ]"))
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

fn validate_effective_contracts(root: &Path, records: &[ChangeRecord]) -> Result<(), Vec<String>> {
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
        return Ok(());
    }
    let active = dependency_ordered_changes(active)
        .map_err(|error| vec![format!("effective contract ordering: {error}")])?;
    let mut modules = BTreeSet::new();
    for record in &active {
        modules.extend(record.affected_specs.iter().cloned());
    }
    let temp = create_effective_contract_workspace().map_err(|error| vec![error])?;
    let config = crate::config::load_config(root);
    let schema_tables = crate::validator::get_schema_table_names(root, &config);
    let schema_columns = crate::commands::build_schema_columns(root, &config);
    let mut errors = Vec::new();
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
        for record in &active {
            if record.canonical_applied || !record.affected_specs.contains(&module) {
                continue;
            }
            let delta_path =
                delta_path_checked(root, record, &module).map_err(|error| vec![error])?;
            let delta = match read_bounded_change_text(&delta_path, "semantic delta") {
                Ok(delta) => delta,
                Err(error) => {
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
                Err(error) => errors.push(format!("{} effective `{module}`: {error}", record.id)),
            }
        }
        let effective_dir = temp.join(&module);
        if let Err(error) = fs::create_dir_all(&effective_dir) {
            errors.push(format!("failed to prepare effective contract: {error}"));
            continue;
        }
        let effective = effective_dir.join(format!("{module}.spec.md"));
        if let Err(error) = fs::write(&effective, spec) {
            errors.push(format!("failed to write effective contract: {error}"));
            continue;
        }
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
                .chain(result.warnings)
                .map(|error| format!("effective contract `{module}`: {error}")),
        );
    }
    let _ = fs::remove_dir_all(temp);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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
    let active_ids: BTreeSet<&str> = changes.iter().map(|record| record.id.as_str()).collect();
    let mut remaining = changes;
    let mut emitted = BTreeSet::new();
    let mut ordered = Vec::new();
    while !remaining.is_empty() {
        let Some(index) = remaining.iter().position(|record| {
            record.dependencies.iter().all(|dependency| {
                !active_ids.contains(dependency.as_str()) || emitted.contains(dependency.as_str())
            })
        }) else {
            return Err("active change dependency cycle prevents deterministic ordering".into());
        };
        let record = remaining.remove(index);
        emitted.insert(record.id.as_str());
        ordered.push(record);
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
        }
    }
    Ok(())
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
                _ => return Err(format!("invalid delta operation heading `## {header}`")),
            };
            continue;
        }
        if let Some(header) = line.strip_prefix("### ") {
            flush(
                &mut items,
                operation,
                current_target,
                &current_key,
                &mut body,
            );
            let (target, key) = if let Some(value) = header.strip_prefix("REQUIREMENT ") {
                (DeltaTarget::Requirement, value)
            } else if let Some(value) = header.strip_prefix("SPEC SECTION ") {
                (DeltaTarget::SpecSection, value)
            } else {
                return Err(format!("invalid delta item heading `### {header}`"));
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
    Ok(items)
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
    let registered = if registry_path.exists() {
        let registry = crate::registry::load_registry(root).ok_or_else(|| {
            format!(
                "failed to parse local registry {} while resolving `{module}`",
                registry_path.display()
            )
        })?;
        registry
            .specs
            .iter()
            .find(|(registered_module, _)| registered_module == module)
            .map(|(_, path)| path.clone())
    } else {
        None
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
        DeltaOperation::Added if start.is_some() => {
            return Err(format!("cannot add existing block `{key}`"));
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
    for (path, _) in prepared {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    let backups: Vec<(PathBuf, Option<String>)> = prepared
        .iter()
        .map(|(path, _)| (path.clone(), fs::read_to_string(path).ok()))
        .collect();
    let journal: Vec<TransactionEntry> = backups
        .iter()
        .map(|(path, original)| {
            Ok(TransactionEntry {
                path: path
                    .strip_prefix(root)
                    .map_err(|_| format!("transaction path escapes project: {}", path.display()))?
                    .to_string_lossy()
                    .replace('\\', "/"),
                original: original.clone(),
            })
        })
        .collect::<Result<_, String>>()?;
    write_json(&root.join(TRANSACTION_PATH), &journal)?;
    for (path, content) in prepared {
        if let Err(error) = fs::write(path, content) {
            recover_pending_transaction(root)?;
            return Err(format!(
                "atomic delta application failed at {}: {error}",
                path.display()
            ));
        }
    }
    fs::remove_file(root.join(TRANSACTION_PATH))
        .map_err(|error| format!("failed to clear transaction journal: {error}"))?;
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
    definition_digest_for_correction_count(root, record, record.correction_count, false)
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
    Ok(definition_digest_with_explicit_false(root, record)? == expected)
}

fn definition_digest_from_record_bytes(
    root: &Path,
    record: &ChangeRecord,
    record_bytes: &[u8],
    corrections: &[CorrectionRecord],
) -> Result<String, String> {
    let dir = find_change_dir(root, &record.id)?;
    let mut digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
    digest.frame(b"record", record_bytes);
    let effective = validate_correction_records_for_prefix(record, corrections)?;
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for artifact in &effective.selected_artifacts {
        files.push((
            format!("{CHANGES_PATH}/{}/{}", record.id, artifact.file_name()),
            dir.join(artifact.file_name()),
        ));
    }
    if let Ok(entries) = fs::read_dir(dir.join("deltas")) {
        files.extend(entries.flatten().filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            Some((
                format!("{CHANGES_PATH}/{}/deltas/{name}", record.id),
                entry.path(),
            ))
        }));
    }
    if let Some(policy) = load_policy(root)
        && let Some(principles) = policy.principles_file
    {
        let path = safe_project_path(root, &principles)?;
        files.push((strict_portable_project_path(root, &path)?, path));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let git_modes = git_index_modes(root)?;
    for (relative, path) in files {
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.len() > MAX_CHANGE_ARTIFACT_BYTES {
            return Err(format!(
                "approval input exceeds {} byte limit: {}",
                MAX_CHANGE_ARTIFACT_BYTES,
                path.display()
            ));
        }
        let content = fs::read(&path)
            .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
        let (kind, mode) = digest_file_kind_and_mode(&relative, &path, &git_modes)?;
        digest.entry(&relative, kind, mode, &content);
    }
    if !corrections.is_empty() {
        let relative = format!("{CHANGES_PATH}/{}/{CORRECTIONS_FILE}", record.id);
        let ledger = CorrectionLedger {
            schema_version: 1,
            corrections: corrections.to_vec(),
        };
        let content = json_content(&ledger)?;
        let mode = git_modes.get(&relative).copied().unwrap_or(0o100644);
        digest.entry(&relative, b"file", mode, content.as_bytes());
    }
    Ok(digest.finish())
}

fn project_input_digest(root: &Path) -> Result<String, String> {
    let mut paths = match git_project_paths(root)? {
        Some(paths) => paths,
        None => strict_walk_project_paths(root)?,
    };
    paths.sort();
    paths.dedup();
    let git_modes = git_index_modes(root)?;
    let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
    for relative in paths {
        if project_input_is_volatile(&relative) {
            continue;
        }
        let path = root.join(&relative);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let target = fs::read_link(&path).map_err(|error| {
                    format!("failed to read symlink {}: {error}", path.display())
                })?;
                let target = target.to_str().ok_or_else(|| {
                    format!(
                        "non-UTF-8 symlink target cannot be hashed portably: {}",
                        path.display()
                    )
                })?;
                digest.entry(&relative, b"symlink", 0o120000, target.as_bytes());
            }
            Ok(metadata) if metadata.is_file() => {
                let content = fs::read(&path)
                    .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
                let (kind, mode) = digest_file_kind_and_mode(&relative, &path, &git_modes)?;
                digest.entry(&relative, kind, mode, &content);
            }
            Ok(_) => digest.entry(&relative, b"non-file", 0, b""),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                digest.entry(&relative, b"missing", 0, b"")
            }
            Err(error) => {
                return Err(format!("failed to inspect {}: {error}", path.display()));
            }
        }
    }
    Ok(digest.finish())
}

fn git_project_paths(root: &Path) -> Result<Option<Vec<String>>, String> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to enumerate Git project paths: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path)
                .map_err(|_| "non-UTF-8 Git path cannot be hashed portably".to_string())?;
            strict_portable_relative_path(path)
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn strict_walk_project_paths(root: &Path) -> Result<Vec<String>, String> {
    walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let path = portable_project_path(root, entry.path());
            path.is_empty() || !project_input_is_volatile(&format!("{path}/"))
        })
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() || entry.file_type().is_symlink())
        .map(|entry| strict_portable_project_path(root, entry.path()))
        .filter(|path| {
            path.as_ref()
                .map_or(true, |path| !project_input_is_volatile(path))
        })
        .collect()
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
    if path.contains('\\') {
        return Err(format!(
            "project path is not portable because it contains a backslash: `{path}`"
        ));
    }
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "project path is not a portable relative path: `{}`",
            path.display()
        ));
    }
    let mut components = Vec::new();
    for component in path.components() {
        let component = component.as_os_str().to_str().ok_or_else(|| {
            format!(
                "non-UTF-8 project path cannot be hashed portably: {}",
                path.display()
            )
        })?;
        components.push(component);
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

fn git_index_modes(root: &Path) -> Result<BTreeMap<String, u32>, String> {
    let output = Command::new("git")
        .args(["ls-files", "--stage", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect Git file modes: {error}"))?;
    if !output.status.success() {
        return Ok(BTreeMap::new());
    }
    let mut modes = BTreeMap::new();
    for entry in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let Some(tab) = entry.iter().position(|byte| *byte == b'\t') else {
            return Err("invalid `git ls-files --stage` output".into());
        };
        let metadata = std::str::from_utf8(&entry[..tab])
            .map_err(|_| "non-UTF-8 Git index metadata".to_string())?;
        let mut fields = metadata.split_whitespace();
        let mode = fields
            .next()
            .ok_or_else(|| "Git index entry is missing a mode".to_string())?;
        let _object = fields
            .next()
            .ok_or_else(|| "Git index entry is missing an object ID".to_string())?;
        let stage = fields
            .next()
            .ok_or_else(|| "Git index entry is missing a stage".to_string())?;
        if stage != "0" {
            return Err("cannot hash a project with unresolved Git index stages".into());
        }
        let mode =
            u32::from_str_radix(mode, 8).map_err(|_| format!("invalid Git file mode `{mode}`"))?;
        let path = std::str::from_utf8(&entry[tab + 1..])
            .map_err(|_| "non-UTF-8 Git path cannot be hashed portably".to_string())?;
        modes.insert(strict_portable_relative_path(path)?, mode);
    }
    Ok(modes)
}

fn digest_file_kind_and_mode(
    relative: &str,
    path: &Path,
    git_modes: &BTreeMap<String, u32>,
) -> Result<(&'static [u8], u32), String> {
    if let Some(mode) = git_modes.get(relative).copied() {
        if mode == 0o120000 {
            return Ok((b"symlink", mode));
        }
        if mode == 0o160000 {
            return Ok((b"gitlink", mode));
        }
        #[cfg(not(unix))]
        return Ok((b"file", mode));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok((b"symlink", 0o120000));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if metadata.permissions().mode() & 0o111 == 0 {
            0o100644
        } else {
            0o100755
        };
        Ok((b"file", mode))
    }
    #[cfg(not(unix))]
    {
        Ok((b"file", 0o100644))
    }
}

fn closing_digest(record: &ChangeRecord, verification: &VerificationRecord) -> String {
    let mut digest = FramedDigest::new(CLOSING_DIGEST_DOMAIN);
    digest.frame(b"record-id", record.id.as_bytes());
    digest.frame(b"contract", verification.contract_digest.as_bytes());
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

fn acceptance_manifest(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(root, record, overrides, None)
}

fn acceptance_manifest_with_signed_owners(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
    signed: &AcceptanceManifestV1,
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(root, record, overrides, Some(signed))
}

fn acceptance_manifest_internal(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
    signed: Option<&AcceptanceManifestV1>,
) -> Result<AcceptanceManifestV1, String> {
    let mut paths = match git_project_paths(root)? {
        Some(paths) => paths,
        None => strict_walk_project_paths(root)?,
    };
    let override_content: BTreeMap<String, &[u8]> = overrides
        .iter()
        .map(|(path, content)| {
            Ok((
                strict_portable_project_path(root, path)?,
                content.as_bytes(),
            ))
        })
        .collect::<Result<_, String>>()?;
    paths.extend(override_content.keys().cloned());
    paths.extend(
        record
            .affected_paths
            .iter()
            .filter_map(|scope| (!scope.ends_with('/')).then_some(scope.clone())),
    );
    paths.extend(record.supersedes.iter().flat_map(|edge| {
        edge.obligations
            .iter()
            .map(|obligation| obligation.path.clone())
    }));
    if let Some(signed) = signed {
        paths.extend(signed.entries.iter().map(|entry| entry.path.clone()));
    }
    paths.sort();
    paths.dedup();
    let git_modes = git_index_modes(root)?;
    let git_objects = git_index_objects(root)?;
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut entries = Vec::new();
    for relative in paths {
        if project_input_is_volatile(&relative)
            || !record_covers_project_path(root, record, &relative)
        {
            continue;
        }
        let (kind, mode, payload) = if let Some(content) = override_content.get(&relative) {
            let mode = git_modes.get(&relative).copied().unwrap_or(0o100644);
            let kind = acceptance_kind_for_mode(mode);
            (kind, mode, content.to_vec())
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            (
                AcceptanceInputKind::File,
                git_modes.get(&relative).copied().unwrap_or(0o100644),
                content.clone(),
            )
        } else if git_modes.get(&relative) == Some(&0o160000) {
            let object = git_objects.get(&relative).ok_or_else(|| {
                format!("gitlink `{relative}` is missing its exact index object ID")
            })?;
            (
                AcceptanceInputKind::Gitlink,
                0o160000,
                object.as_bytes().to_vec(),
            )
        } else {
            let path = root.join(&relative);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    let target = fs::read_link(&path).map_err(|error| {
                        format!("failed to read symlink {}: {error}", path.display())
                    })?;
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
                Ok(metadata) if metadata.is_file() => {
                    let content = fs::read(&path)
                        .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
                    let (_, mode) = digest_file_kind_and_mode(&relative, &path, &git_modes)?;
                    (AcceptanceInputKind::File, mode, content)
                }
                Ok(_) => (AcceptanceInputKind::NonFile, 0, Vec::new()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    (AcceptanceInputKind::Missing, 0, Vec::new())
                }
                Err(error) => {
                    return Err(format!("failed to inspect {}: {error}", path.display()));
                }
            }
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
            acceptance_input_owners(root, record, &relative, overrides)?
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
        || target.contains('\\')
        || target.chars().any(char::is_control)
        || Path::new(target).is_absolute()
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
    tuples.sort_by(|left, right| {
        succession_change_key(&left.predecessor_id)
            .cmp(&succession_change_key(&right.predecessor_id))
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
    let mut previous: Option<(u64, &str, &str, &str)> = None;
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
        let sequence = change_sequence(&tuple.predecessor_id)
            .ok_or_else(|| format!("invalid semantic predecessor ID `{}`", tuple.predecessor_id))?;
        let key = (
            sequence,
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
    let mut previous: Option<(u64, &str, &str, &str)> = None;
    for tuple in &evidence.tuples {
        let sequence = change_sequence(&tuple.predecessor_id)
            .ok_or_else(|| format!("invalid semantic predecessor ID `{}`", tuple.predecessor_id))?;
        let key = (
            sequence,
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
                    fs::read_to_string(&spec_path).ok()
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
    if owners.is_empty() {
        if path_is_production_source(root, relative) {
            return Err(format!(
                "acceptance input `{relative}` is production source without deterministic canonical ownership"
            ));
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

fn git_index_objects(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let output = Command::new("git")
        .args(["ls-files", "--stage", "-z"])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect Git index objects: {error}"))?;
    if !output.status.success() {
        return Ok(BTreeMap::new());
    }
    let mut objects = BTreeMap::new();
    for entry in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let entry = std::str::from_utf8(entry)
            .map_err(|_| "non-UTF-8 Git index entry cannot be hashed portably".to_string())?;
        let (metadata, path) = entry
            .split_once('\t')
            .ok_or_else(|| format!("invalid Git index entry `{entry}`"))?;
        let mut fields = metadata.split_whitespace();
        let _mode = fields.next();
        let object = fields
            .next()
            .ok_or_else(|| format!("Git index entry has no object ID: `{entry}`"))?;
        objects.insert(strict_portable_relative_path(path)?, object.to_string());
    }
    Ok(objects)
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
        let ledger = load_approvals(root, record)?;
        let approval = ledger
            .approvals
            .iter()
            .rev()
            .find(|approval| approval.gate == "acceptance")
            .ok_or_else(|| format!("accepted change `{}` has no closing approval", record.id))?;
        if approval.digest != closing_digest(record, &verification) {
            return Err(format!(
                "accepted change `{}` closing approval does not authenticate its manifest",
                record.id
            ));
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
    for anchor in anchors {
        if let Ok((key, manifest)) =
            reconstruct_legacy_at_anchor(root, record, signed_legacy_digest, &anchor)
        {
            reconstructions.entry(key).or_insert(manifest);
        }
    }
    if reconstructions.len() != 1 {
        return Err(format!(
            "legacy accepted change `{}` requires exactly one distinct valid historical reconstruction, found {}",
            record.id,
            reconstructions.len()
        ));
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
        let closing = ledger
            .approvals
            .iter()
            .rev()
            .find(|approval| approval.gate == "acceptance")
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
        let manifest = acceptance_manifest(&tree, &historical, &[])?;
        let key = serde_json::to_vec(&serde_json::json!({
            "manifest": manifest,
            "verification": verification,
            "closing": closing,
        }))
        .map_err(|error| format!("failed to canonicalize historical evidence: {error}"))?;
        Ok((key, manifest))
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
    if !removed.is_ok_and(|output| output.status.success()) {
        let _ = fs::remove_dir_all(&temporary);
        return Err("failed to remove legacy reconstruction workspace".into());
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
        let output = Command::new("git")
            .args([
                "log",
                "--format=%H",
                &reference,
                "--",
                state_pathspec.as_str(),
            ])
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to inspect accepted history: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "failed to inspect accepted history at `{reference}`"
            ));
        }
        for commit in String::from_utf8_lossy(&output.stdout).lines() {
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

fn authenticated_accepted_transition(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(String, Vec<u8>, ChangeRecord), String> {
    let workspace = find_change_dir(root, &record.id)?;
    let current_verification = fs::read(workspace.join("verification.json"))
        .map_err(|error| format!("failed to read current verification evidence: {error}"))?;
    let current_approvals = fs::read(workspace.join("approvals.json"))
        .map_err(|error| format!("failed to read current approval evidence: {error}"))?;
    let active = format!("{CHANGES_PATH}/{}", record.id);
    let state_path = git_repo_relative_path(root, &format!("{active}/state.json"))?;
    let verification_path = git_repo_relative_path(root, &format!("{active}/verification.json"))?;
    let approvals_path = git_repo_relative_path(root, &format!("{active}/approvals.json"))?;
    let mut eligible = BTreeMap::new();
    for anchor in accepted_transition_anchors(root, record)? {
        let Some(state_bytes) = git_object_bytes(root, &anchor, &state_path) else {
            continue;
        };
        let Some(verification_bytes) = git_object_bytes(root, &anchor, &verification_path) else {
            continue;
        };
        let Some(approval_bytes) = git_object_bytes(root, &anchor, &approvals_path) else {
            continue;
        };
        if verification_bytes != current_verification || approval_bytes != current_approvals {
            continue;
        }
        let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
            continue;
        };
        if accepted.id != record.id || accepted.state != ChangeState::Accepted {
            continue;
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
                .or_insert((anchor, state_bytes, accepted));
        }
    }
    if record.state == ChangeState::Archived {
        let project_directory = strict_portable_project_path(root, &workspace)?;
        let repository_directory = git_repo_relative_path(root, &project_directory)?;
        let accepted_state_path = format!("{repository_directory}/accepted-state.json");
        let verification_path = format!("{repository_directory}/verification.json");
        let approvals_path = format!("{repository_directory}/approvals.json");
        let accepted_state_pathspec = format!(":(top,literal){accepted_state_path}");
        let mut references = vec!["HEAD".to_string()];
        if let Some(remote_default) = remote_default_ref(root)
            && !references.contains(&remote_default)
        {
            references.push(remote_default);
        }
        let mut command = Command::new("git");
        command.args(["log", "--format=%H", "--diff-filter=A"]);
        command.args(&references);
        let output = command
            .arg("--")
            .arg(&accepted_state_pathspec)
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to inspect archived acceptance history: {error}"))?;
        if !output.status.success() {
            return Err("failed to inspect archived acceptance history".into());
        }
        for anchor in String::from_utf8_lossy(&output.stdout).lines() {
            let Some(state_bytes) = git_object_bytes(root, anchor, &accepted_state_path) else {
                continue;
            };
            let Some(verification_bytes) = git_object_bytes(root, anchor, &verification_path)
            else {
                continue;
            };
            let Some(approval_bytes) = git_object_bytes(root, anchor, &approvals_path) else {
                continue;
            };
            if verification_bytes != current_verification || approval_bytes != current_approvals {
                continue;
            }
            let Ok(accepted) = serde_json::from_slice::<ChangeRecord>(&state_bytes) else {
                continue;
            };
            if accepted.id != record.id || accepted.state != ChangeState::Accepted {
                continue;
            }
            let mut projection = record.clone();
            projection.state = ChangeState::Accepted;
            projection.updated_at = accepted.updated_at;
            if projection == accepted {
                let key = accepted_evidence_key(&state_bytes, &verification_bytes, &approval_bytes);
                eligible
                    .entry(key)
                    .or_insert((anchor.to_string(), state_bytes, accepted));
            }
        }
    }
    if eligible.is_empty()
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
    let closing_matches = approvals
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
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
    let output = Command::new("git")
        .args(["show", object.as_str()])
        .current_dir(root)
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

fn git_change_record_at(root: &Path, commit: &str, path: &str) -> Option<ChangeRecord> {
    let object = format!("{commit}:{path}");
    let output = Command::new("git")
        .args(["show", object.as_str()])
        .current_dir(root)
        .output()
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
    let mut paths = match git_project_paths(root)? {
        Some(paths) => paths,
        None => strict_walk_project_paths(root)?,
    };
    let override_content: BTreeMap<String, &[u8]> = overrides
        .iter()
        .map(|(path, content)| {
            Ok((
                strict_portable_project_path(root, path)?,
                content.as_bytes(),
            ))
        })
        .collect::<Result<_, String>>()?;
    paths.extend(override_content.keys().cloned());
    paths.sort();
    paths.dedup();
    let git_modes = git_index_modes(root)?;
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut digest = FramedDigest::new(ACCEPTANCE_DIGEST_DOMAIN);
    for relative in paths {
        if project_input_is_volatile(&relative)
            || !record_covers_project_path(root, record, &relative)
        {
            continue;
        }
        if let Some(content) = override_content.get(&relative) {
            let mode = git_modes.get(&relative).copied().unwrap_or(0o100644);
            let kind: &[u8] = match mode {
                0o120000 => b"symlink",
                0o160000 => b"gitlink",
                _ => b"file",
            };
            digest.entry(&relative, kind, mode, content);
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            let mode = git_modes.get(&relative).copied().unwrap_or(0o100644);
            digest.entry(&relative, b"file", mode, content);
        } else {
            let path = root.join(&relative);
            match fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    let target = fs::read_link(&path).map_err(|error| {
                        format!("failed to read symlink {}: {error}", path.display())
                    })?;
                    let target = target.to_str().ok_or_else(|| {
                        format!(
                            "non-UTF-8 symlink target cannot be hashed portably: {}",
                            path.display()
                        )
                    })?;
                    digest.entry(&relative, b"symlink", 0o120000, target.as_bytes());
                }
                Ok(metadata) if metadata.is_file() => {
                    let content = fs::read(&path)
                        .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
                    let (kind, mode) = digest_file_kind_and_mode(&relative, &path, &git_modes)?;
                    digest.entry(&relative, kind, mode, &content);
                }
                Ok(_) => digest.entry(&relative, b"non-file", 0, b""),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    digest.entry(&relative, b"missing", 0, b"")
                }
                Err(error) => {
                    return Err(format!("failed to inspect {}: {error}", path.display()));
                }
            }
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
    let sequence = change_sequence(&record.id)
        .ok_or_else(|| format!("invalid change ID `{}`", record.id.escape_default()))?;
    if ledger.sequence <= sequence {
        return Ok(None);
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

fn ensure_definition_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    let approval = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    if !definition_digest_matches(root, record, &approval.digest)? {
        return Err(
            "definition approval is stale; approve the current artifact digest again".into(),
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcceptedInputValidity {
    Exact,
    SuccessorCovered,
}

fn terminal_evidence_summary(root: &Path, record: &ChangeRecord) -> TerminalEvidenceSummary {
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

fn terminal_evidence_summary_with_records(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
) -> TerminalEvidenceSummary {
    if record.state == ChangeState::Archived {
        return match validate_archived_integrity(root, record) {
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
    match validate_accepted_inputs_recursive(
        root,
        record,
        records,
        &mut BTreeSet::new(),
        &mut BTreeMap::new(),
    ) {
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

fn authenticate_accepted_evidence(
    root: &Path,
    record: &ChangeRecord,
) -> Result<VerificationRecord, String> {
    if record.state == ChangeState::Archived {
        validate_archived_accepted_snapshot(root, record)?;
    }
    let verification = load_verification(root, record)?;
    if !verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, record)?;
    if !definition_digest_matches(root, record, &verification.contract_digest)? {
        return Err("accepted change verification contract is stale".into());
    }
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
    let expected = closing_digest(record, &verification);
    let ledger = load_approvals(root, record)?;
    let approval = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .ok_or_else(|| "accepted change is missing closing approval".to_string())?;
    if approval.digest != expected {
        return Err("accepted change closing approval does not match verification evidence".into());
    }
    if !verification_commit_is_accepted_current(root, &verification)
        && !accepted_workspace_is_integrated(root, record)
        && !accepted_change_is_recorded_on_remote_default(root, record)
    {
        return Err("accepted change verification commit is not in current history and canonical acceptance is not recorded on the remote default branch".into());
    }
    Ok(verification)
}

fn validate_archived_accepted_snapshot(root: &Path, archived: &ChangeRecord) -> Result<(), String> {
    let workspace = find_change_dir(root, &archived.id)?;
    let path = workspace.join("accepted-state.json");
    let (_, historical_bytes, historical) = authenticated_accepted_transition(root, archived)?;
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

fn validate_archived_integrity(root: &Path, archived: &ChangeRecord) -> Result<(), String> {
    let workspace = find_change_dir(root, &archived.id)?;
    if !workspace.join("accepted-state.json").exists() {
        return authenticate_legacy_archive_baseline(root, archived, &workspace);
    }
    validate_archived_accepted_snapshot(root, archived)?;
    let verification = load_verification(root, archived)?;
    if !verification.passed {
        return Err("archived change has failed verification evidence".into());
    }
    ensure_definition_approval_valid(root, archived)?;
    if !definition_digest_matches(root, archived, &verification.contract_digest)? {
        return Err("archived change verification contract is stale".into());
    }
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
    let ledger = load_approvals(root, archived)?;
    let approval = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "acceptance")
        .ok_or_else(|| "archived change is missing closing approval".to_string())?;
    if approval.digest != closing_digest(archived, &verification) {
        return Err("archived change closing approval does not match verification evidence".into());
    }
    Ok(())
}

fn authenticate_legacy_archive_baseline(
    root: &Path,
    archived: &ChangeRecord,
    workspace: &Path,
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
    let baseline_entry = legacy_baseline_entry(&baseline, &archived.id)?;
    if baseline_entry.archive_path != project_subtree {
        return Err(format!(
            "legacy archive `{}` baseline path does not match its unique workspace",
            archived.id
        ));
    }
    let repo_subtree = git_repo_relative_path(root, &project_subtree)?;
    let current = archive_workspace_snapshot(root, workspace, &project_subtree)?;
    if legacy_archive_subtree_digest(&current)? != baseline_entry.subtree_digest {
        return Err(format!(
            "legacy archive `{}` subtree does not match its baseline digest",
            archived.id
        ));
    }
    let introduction = git_output(
        root,
        &[
            "rev-parse",
            "--verify",
            &format!("{}^{{commit}}", baseline_entry.introduction_commit),
        ],
    )
    .ok_or_else(|| {
        format!(
            "legacy archive `{}` introduction is unavailable",
            archived.id
        )
    })?;
    if introduction != baseline_entry.introduction_commit {
        return Err(format!(
            "legacy archive `{}` introduction must be a canonical commit ID",
            archived.id
        ));
    }
    ensure_git_ancestor(root, &introduction, &cutoff, "legacy archive cutoff")?;
    let pathspec = format!(":(top,literal){repo_subtree}");
    let output = Command::new("git")
        .args([
            "log",
            "--format=%H",
            "--diff-filter=A",
            &cutoff,
            "--",
            &pathspec,
        ])
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to inspect legacy archive introductions: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect legacy archive introductions at cutoff `{cutoff}`"
        ));
    }
    let mut anchors = BTreeSet::new();
    for commit in String::from_utf8_lossy(&output.stdout).lines() {
        let parents =
            git_output(root, &["rev-list", "--parents", "-n", "1", commit]).unwrap_or_default();
        let parent_has_subtree = parents.split_whitespace().skip(1).any(|parent| {
            !git_tree_snapshot(root, parent, &repo_subtree)
                .unwrap_or_default()
                .is_empty()
        });
        if parent_has_subtree {
            continue;
        }
        if git_tree_snapshot(root, commit, &repo_subtree).is_ok_and(|tree| tree == current) {
            anchors.insert(commit.to_string());
        }
    }
    if anchors.len() != 1 || !anchors.contains(&introduction) {
        return Err(format!(
            "legacy archive `{}` requires its one baseline-bound pre-cutoff introduction anchor, found {}",
            archived.id,
            anchors.len()
        ));
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
    if json_content(&baseline)?.as_bytes() != baseline_bytes {
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

fn archive_workspace_snapshot(
    root: &Path,
    workspace: &Path,
    project_subtree: &str,
) -> Result<BTreeMap<String, (u32, Vec<u8>)>, String> {
    let git_modes = git_index_modes(root)?;
    let mut snapshot = BTreeMap::new();
    for entry in walkdir::WalkDir::new(workspace).follow_links(false) {
        let entry = entry.map_err(|error| format!("failed to inspect legacy archive: {error}"))?;
        if entry.path() == workspace {
            continue;
        }
        let file_type = entry.file_type();
        if file_type.is_dir() {
            continue;
        }
        if !file_type.is_file() && !file_type.is_symlink() {
            return Err(format!(
                "legacy archive contains unsupported non-file entry {}",
                entry.path().display()
            ));
        }
        let relative = strict_portable_project_path(workspace, entry.path())?;
        let project_path = format!("{project_subtree}/{relative}");
        let (_, mode) = digest_file_kind_and_mode(&project_path, entry.path(), &git_modes)?;
        let bytes = if file_type.is_symlink() {
            let target = fs::read_link(entry.path())
                .map_err(|error| format!("failed to read legacy archive symlink: {error}"))?;
            let target = target
                .to_str()
                .ok_or_else(|| "legacy archive symlink target is not UTF-8".to_string())?;
            validate_portable_symlink_target(target)?;
            target.as_bytes().to_vec()
        } else {
            fs::read(entry.path())
                .map_err(|error| format!("failed to read legacy archive entry: {error}"))?
        };
        snapshot.insert(relative, (mode, bytes));
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
    let mut snapshot = BTreeMap::new();
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
        let bytes = Command::new("git")
            .args(["cat-file", "blob", object])
            .current_dir(root)
            .output()
            .map_err(|error| format!("failed to read legacy archive blob: {error}"))?;
        if !bytes.status.success() {
            return Err(format!("failed to read legacy archive blob `{object}`"));
        }
        snapshot.insert(
            strict_portable_relative_path(relative)?,
            (mode, bytes.stdout),
        );
    }
    Ok(snapshot)
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
                "accepted change is missing current delivery-input evidence".to_string()
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
                    "accepted input `{}` disappeared from current inventory",
                    expected.path
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
                return Err(format!(
                    "accepted exact-only input `{}` changed and requires audited reopen",
                    expected.path
                ));
            }
            for owner in &expected.owners {
                let mut covered = false;
                for candidate in records.values() {
                    if candidate.id == record.id
                        || !matches!(
                            candidate.state,
                            ChangeState::Accepted | ChangeState::Archived
                        )
                        || candidate.no_spec_change
                        || succession_change_key(&candidate.id) <= succession_change_key(&record.id)
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
                }
                if !covered {
                    return Err(format!(
                        "accepted input obligation `{}` owner `{owner}` has no closing-valid terminal semantic successor",
                        expected.path
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

fn list_all_changes_checked(root: &Path) -> Result<BTreeMap<String, ChangeRecord>, String> {
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
                    && is_positive_legacy_tombstone(&entry.path()) =>
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
        if records.insert(record.id.clone(), record.clone()).is_some() {
            return Err(format!(
                "change `{}` exists in multiple active/archive locations",
                record.id
            ));
        }
    }
    Ok(records)
}

fn is_positive_legacy_tombstone(path: &Path) -> bool {
    let dated_lifecycle_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.contains("-CHG-"));
    if dated_lifecycle_name
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
    ledger.approvals.push(ApprovalRecord {
        gate: gate.into(),
        actor: resolve_actor(root, actor)?,
        timestamp: now(),
        digest,
        note,
    });
    write_json(
        &change_dir(root, &record.id).join("approvals.json"),
        &ledger,
    )
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
    serde_json::from_str(&content).map_err(|error| error.to_string())
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
    if adoption_bootstrap_covers_policy(root) {
        changed.remove(POLICY_PATH);
    }
    let covered: Vec<&str> = records
        .iter()
        .filter(|record| record_is_delivering(record))
        .flat_map(|record| record.affected_paths.iter().map(String::as_str))
        .collect();
    let uncovered = changed
        .into_iter()
        .filter(|path| path_is_meaningful_for_root(root, path, policy))
        .filter(|path| {
            !covered.iter().any(|scope| path_matches_scope(path, scope))
                && !records
                    .iter()
                    .any(|record| record_covers_project_path(root, record, path))
        })
        .collect();
    Ok(uncovered)
}

fn record_is_delivering(record: &ChangeRecord) -> bool {
    matches!(
        record.state,
        ChangeState::Implementing | ChangeState::Verifying | ChangeState::Accepted
    )
}

fn record_covers_path(record: &ChangeRecord, path: &str) -> bool {
    record_is_delivering(record)
        && record
            .affected_paths
            .iter()
            .any(|scope| path_matches_scope(path, scope))
}

fn record_covers_project_path(root: &Path, record: &ChangeRecord, path: &str) -> bool {
    if !record_is_delivering(record) {
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
    recorded_diff_base(records)
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

fn recorded_diff_base(records: &[ChangeRecord]) -> String {
    records
        .iter()
        .filter_map(|record| record.base_commit.clone())
        .next()
        .unwrap_or_else(|| "HEAD~1...HEAD".into())
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

#[derive(Clone, Copy)]
enum ConfiguredCommandOutput {
    Inherit,
    Suppress,
}

fn run_configured_command(
    root: &Path,
    configured: &str,
    output: ConfiguredCommandOutput,
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
    if matches!(output, ConfiguredCommandOutput::Suppress) {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
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

fn import_foreign(root: &Path, source: &str) -> Result<(), String> {
    let provenance = root.join(".specsync/import-provenance.json");
    let value = serde_json::json!({
        "source": source,
        "imported_at": now(),
        "scope": "active_plus_canonical",
        "archives": "preserved_in_place"
    });
    write_json(&provenance, &value)?;
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
                    copy_markdown_files(root, &canonical, &import_root.join("canonical"))?;
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
                    fs::create_dir_all(&import_root).map_err(|error| error.to_string())?;
                    fs::copy(&constitution, import_root.join("constitution.md"))
                        .map_err(|error| error.to_string())?;
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
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    }
    for entry in fs::read_dir(source_changes).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
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
        if list_changes_checked(root)?
            .iter()
            .any(|record| record.description == description)
        {
            continue;
        }
        let record = create_change(
            root,
            CreateChangeRequest {
                description,
                kind: ChangeKind::Feature,
                affected_specs: Vec::new(),
                affected_paths: vec![portable_project_path(root, &entry_path)],
                requested_artifacts: vec![
                    ArtifactKind::Requirements,
                    ArtifactKind::Design,
                    ArtifactKind::Tasks,
                ],
                no_spec_change: true,
                rationale: Some(format!(
                    "Imported from {source}; canonical reconciliation is pending"
                )),
            },
        )?;
        let destination = change_dir(root, &record.id).join("imported");
        copy_markdown_files(root, &entry_path, &destination)?;
    }
    Ok(())
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

fn copy_markdown_files(
    project_root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), String> {
    reject_symlink_components(project_root, source)?;
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "refusing non-directory or symlinked foreign import path: {}",
            source.display()
        ));
    }
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "refusing symlinked foreign import path: {}",
                path.display()
            ));
        }
        if file_type.is_dir() {
            copy_markdown_files(project_root, &path, &destination.join(entry.file_name()))?;
        } else if file_type.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("md")
        {
            fs::copy(&path, destination.join(entry.file_name()))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn reject_symlink_components(project_root: &Path, candidate: &Path) -> Result<(), String> {
    let relative = candidate.strip_prefix(project_root).map_err(|_| {
        format!(
            "foreign import path escapes project root: {}",
            candidate.display()
        )
    })?;
    let mut current = project_root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "refusing symlinked foreign import path: {}",
                    current.display()
                ));
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
    format!(
        "---\nchange: {}\nartifact: {}\n---\n\n# {}\n\n<!-- TODO: complete this artifact or remove it from selected_artifacts before approval. -->\n",
        record.id,
        title,
        title_from_description(&title)
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

fn find_change_dir(root: &Path, id: &str) -> Result<PathBuf, String> {
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

fn validate_change_id(id: &str) -> Result<(), String> {
    let is_single_component = {
        let mut components = Path::new(id).components();
        matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
    };
    if id.starts_with("CHG-")
        && is_single_component
        && !id.contains(['/', '\\'])
        && !id.chars().any(char::is_control)
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

fn verification_commit_is_current(
    root: &Path,
    evidence: &VerificationRecord,
    allow_ancestor: bool,
) -> bool {
    let Some(commit) = evidence.commit.as_deref() else {
        return false;
    };
    if !allow_ancestor {
        return git_output(root, &["rev-parse", "HEAD"]).as_deref() == Some(commit);
    }
    Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(root)
        .status()
        .is_ok_and(|status| status.success())
}

fn verification_commit_is_accepted_current(root: &Path, evidence: &VerificationRecord) -> bool {
    let Some(commit) = evidence.commit.as_deref() else {
        return git_output(root, &["rev-parse", "HEAD"]).is_none();
    };
    Command::new("git")
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .current_dir(root)
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
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let Ok(repo_state) = git_repo_relative_path(root, &state) else {
        return false;
    };
    let top_state = format!(":(top,literal){repo_state}");
    let history = Command::new("git")
        .args(["log", "--format=%H", reference, "--", top_state.as_str()])
        .current_dir(root)
        .output();
    let Ok(history) = history else {
        return false;
    };
    if !history.status.success() {
        return false;
    }
    String::from_utf8_lossy(&history.stdout)
        .lines()
        .any(|commit| {
            let object = format!("{commit}:{repo_state}");
            let snapshot = Command::new("git")
                .args(["show", object.as_str()])
                .current_dir(root)
                .output();
            let Ok(snapshot) = snapshot else {
                return false;
            };
            snapshot.status.success()
                && serde_json::from_slice::<ChangeRecord>(&snapshot.stdout).is_ok_and(
                    |historical| {
                        historical.id == record.id && historical.state == ChangeState::Accepted
                    },
                )
        })
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
                && candidate.created_at >= record.created_at
                && change_id_sorts_after(&candidate.id, &record.id)
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

fn next_change_id(root: &Path, slug: &str) -> Result<String, String> {
    let mut maximum = 0_u64;
    for base in [root.join(CHANGES_PATH), root.join(ARCHIVE_PATH)] {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                for part in name.split('-') {
                    if let Some(number) = part.strip_prefix("CHG")
                        && let Ok(value) = number.parse::<u64>()
                    {
                        maximum = maximum.max(value);
                    }
                }
                if let Some(index) = name.find("CHG-") {
                    let digits: String = name[index + 4..]
                        .chars()
                        .take_while(char::is_ascii_digit)
                        .collect();
                    if let Ok(value) = digits.parse::<u64>() {
                        maximum = maximum.max(value);
                    }
                }
            }
        }
    }
    Ok(format!("CHG-{:04}-{slug}", maximum + 1))
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars().take(80) {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "change".into()
    } else {
        slug.into()
    }
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
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn git_output_allow_empty(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_repo_relative_path(root: &Path, project_path: &str) -> Result<String, String> {
    let prefix = git_output_allow_empty(root, &["rev-parse", "--show-prefix"])
        .ok_or_else(|| "unable to determine project path within Git repository".to_string())?
        .replace('\\', "/");
    Ok(format!("{prefix}{project_path}"))
}

fn write_json<Value: Serialize>(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, json_content(value)?)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn json_content<Value: Serialize>(value: &Value) -> Result<String, String> {
    let content = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    Ok(format!("{content}\n"))
}

fn is_false(value: &bool) -> bool {
    !*value
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
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, mpsc};
    use tempfile::TempDir;

    // Verifies REQ-change-021.
    #[test]
    fn append_changelog_preserves_version_date_changes_schema() {
        let spec = "---\nmodule: canary\nversion: 3\n---\n\n## Change Log\n\n| Version | Date | Changes |\n|---------|------|---------|\n| 2 | 2026-07-13 | Previous |\n";

        let updated = append_changelog(spec, "CHG-0003", "Correct the change log");

        assert!(updated.contains(&format!(
            "| 3 | {} | CHG-0003: Correct the change log |",
            today()
        )));
    }

    #[test]
    fn append_changelog_populates_date_author_change_schema() {
        let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Author | Change |\n|------|--------|--------|\n";

        let updated = append_changelog(spec, "CHG-0002", "Document behavior");

        assert!(updated.contains(&format!(
            "| {} | SpecSync | CHG-0002: Document behavior |",
            today()
        )));
    }

    #[test]
    fn append_changelog_keeps_default_two_column_schema() {
        let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n";

        let updated = append_changelog(spec, "CHG-0002", "Document behavior");

        assert!(updated.contains(&format!("| {} | CHG-0002: Document behavior |", today())));
    }

    #[test]
    fn append_changelog_does_not_treat_similar_headers_as_change_columns() {
        let spec = "---\nmodule: canary\nversion: 2\n---\n\n## Change Log\n\n| Date | Changer | Change |\n|------|---------|--------|\n";

        let updated = append_changelog(spec, "CHG-0002", "Document behavior");

        assert!(updated.contains(&format!("| {} |  | CHG-0002: Document behavior |", today())));
    }

    #[test]
    fn effective_contract_workspaces_are_unique() {
        const WORKERS: usize = 32;
        let barrier = Arc::new(Barrier::new(WORKERS));
        let (sender, receiver) = mpsc::channel();
        std::thread::scope(|scope| {
            for _ in 0..WORKERS {
                let barrier = Arc::clone(&barrier);
                let sender = sender.clone();
                scope.spawn(move || {
                    barrier.wait();
                    sender
                        .send(create_effective_contract_workspace().unwrap())
                        .unwrap();
                });
            }
        });
        drop(sender);
        let paths: BTreeSet<PathBuf> = receiver.into_iter().collect();
        assert_eq!(paths.len(), WORKERS);
        for path in paths {
            fs::remove_dir(path).unwrap();
        }
    }

    fn completed_record(root: &Path) -> ChangeRecord {
        let mut record = create_change(
            root,
            CreateChangeRequest {
                description: "add passkeys".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        record.acceptance_criteria = vec!["Users can authenticate with a passkey".into()];
        record
            .answers
            .insert("public_contract".into(), "yes".into());
        record
            .answers
            .insert("architecture_risk".into(), "no".into());
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        record
    }

    fn completed_no_spec_record(root: &Path) -> ChangeRecord {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/change")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
        fs::write(
            root.join("specs/change/change.spec.md"),
            "---\nmodule: change\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/lib.rs\n---\n\n# Change\n\n## Purpose\n\nLifecycle fixture.\n\n## Public API\n\nNone.\n\n## Invariants\n\nVerification is deterministic.\n\n## Behavioral Examples\n\nChecks pass.\n\n## Error Cases\n\nInvalid evidence fails.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        let mut record = create_change(
            root,
            CreateChangeRequest {
                description: "harden verification".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["change".into()],
                affected_paths: vec!["src/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("No public contract change".into()),
            },
        )
        .unwrap();
        record.acceptance_criteria = vec!["Verification is fresh".into()];
        record.answers.insert("public_contract".into(), "no".into());
        record
            .answers
            .insert("architecture_risk".into(), "no".into());
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        for artifact in &record.selected_artifacts {
            let content = if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Complete\n"
            } else {
                "# Complete\n\nReviewed.\n"
            };
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                content,
            )
            .unwrap();
        }
        record
    }

    fn completed_section_only_record(root: &Path, delta: &str) -> ChangeRecord {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nRetained for compatibility.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        let record = completed_record(root);
        for artifact in &record.selected_artifacts {
            let content = if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Complete the documentation change.\n"
            } else {
                "# Complete\n\nReviewed content.\n"
            };
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                content,
            )
            .unwrap();
        }
        fs::write(delta_path(root, &record, "auth"), delta).unwrap();
        record
    }

    fn accept_completed_record(root: &Path, mut record: ChangeRecord) -> ChangeRecord {
        record =
            approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap()
    }

    #[test]
    fn successor_evidence_fields_are_byte_compatible_when_absent() {
        let temp = TempDir::new().unwrap();
        let record = completed_no_spec_record(temp.path());
        let mut legacy_record = serde_json::to_value(&record).unwrap();
        legacy_record.as_object_mut().unwrap().remove("supersedes");
        let decoded: ChangeRecord = serde_json::from_value(legacy_record.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), legacy_record);

        let verification = VerificationRecord {
            timestamp: 1,
            commit: None,
            contract_digest: "contract".into(),
            workspace_digest: "workspace".into(),
            acceptance_input_digest: None,
            acceptance_manifest: None,
            semantic_succession: None,
            passed: true,
            commands: Vec::new(),
            requirement_ids: Vec::new(),
        };
        let mut legacy_verification = serde_json::to_value(&verification).unwrap();
        let object = legacy_verification.as_object_mut().unwrap();
        object.remove("acceptance_manifest");
        object.remove("semantic_succession");
        let decoded: VerificationRecord =
            serde_json::from_value(legacy_verification.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), legacy_verification);
    }

    #[test]
    fn acceptance_manifest_validation_rejects_topology_and_digest_aliases() {
        let payload = sha256_hex(b"content");
        let entry = AcceptanceInputEntryV1 {
            path: "src/lib.rs".into(),
            kind: AcceptanceInputKind::File,
            mode: 0o100644,
            entry_digest: acceptance_entry_digest(
                "src/lib.rs",
                &AcceptanceInputKind::File,
                0o100644,
                &payload,
            ),
            payload_digest: payload,
            owners: vec!["change".into()],
        };
        let manifest = AcceptanceManifestV1 {
            schema_version: 1,
            entries: vec![entry.clone()],
        };
        assert!(validate_acceptance_manifest(&manifest).is_ok());

        let mut duplicate = manifest.clone();
        duplicate.entries.push(entry.clone());
        assert!(validate_acceptance_manifest(&duplicate).is_err());
        let mut wrong_mode = manifest.clone();
        wrong_mode.entries[0].mode = 0o100755;
        assert!(validate_acceptance_manifest(&wrong_mode).is_err());
        let mut missing = manifest;
        missing.entries[0].kind = AcceptanceInputKind::Missing;
        missing.entries[0].mode = 0;
        missing.entries[0].entry_digest = acceptance_entry_digest(
            "src/lib.rs",
            &AcceptanceInputKind::Missing,
            0,
            &missing.entries[0].payload_digest,
        );
        assert!(validate_acceptance_manifest(&missing).is_err());
    }

    #[test]
    fn portable_symlink_targets_reject_host_specific_or_ambiguous_forms() {
        assert!(validate_portable_symlink_target("../shared/file").is_ok());
        for target in ["", "/etc/passwd", "C:/secret", "dir\\file", "line\nfeed"] {
            assert!(
                validate_portable_symlink_target(target).is_err(),
                "{target:?}"
            );
        }
    }

    #[test]
    fn root_source_files_are_not_misclassified_as_delivery_metadata() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::write(root.join("main.py"), "print('ok')\n").unwrap();
        fs::write(root.join("main.go"), "package main\n").unwrap();
        assert!(path_is_production_source(root, "main.py"));
        assert!(path_is_production_source(root, "main.go"));
        assert!(!path_is_recognized_delivery_metadata("main.py"));
        assert!(!path_is_recognized_delivery_metadata("main.go"));
        for path in [
            "pyproject.toml",
            "go.mod",
            "package.json",
            "pnpm-lock.yaml",
            "action.yml",
        ] {
            assert!(path_is_recognized_delivery_metadata(path), "{path}");
        }
    }

    #[test]
    fn invalid_supersedes_mutation_is_transactional() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let predecessor = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nPredecessor.\n",
        );
        let successor = create_change(
            root,
            CreateChangeRequest {
                description: "Successor".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        let state_path = change_dir(root, &successor.id).join("state.json");
        let markdown_path = change_dir(root, &successor.id).join("change.md");
        let before_state = fs::read(&state_path).unwrap();
        let before_markdown = fs::read(&markdown_path).unwrap();
        let oversized = format!("src/{}.rs", "x".repeat(MAX_ACCEPTANCE_PATH_BYTES + 1));
        assert!(
            add_supersedes_obligation(
                root,
                &successor.id,
                &predecessor.id,
                &oversized,
                "auth",
                &sha256_hex(b"entry"),
            )
            .is_err()
        );
        assert_eq!(fs::read(state_path).unwrap(), before_state);
        assert_eq!(fs::read(markdown_path).unwrap(), before_markdown);
    }

    #[test]
    fn mapped_tests_remain_exact_only() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nTests remain governed.\n",
        );
        assert_eq!(
            acceptance_input_owners(root, &record, "tests/auth.rs", &[]).unwrap(),
            vec![EXACT_TEST_OWNER.to_string()]
        );
    }

    #[test]
    fn signed_directory_deletion_remains_a_missing_manifest_entry() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.affected_paths = vec!["src/".into()];
        record.state = ChangeState::Accepted;
        let payload = sha256_hex(b"old");
        let signed = AcceptanceManifestV1 {
            schema_version: 1,
            entries: vec![AcceptanceInputEntryV1 {
                path: "src/deleted.rs".into(),
                kind: AcceptanceInputKind::File,
                mode: 0o100644,
                entry_digest: acceptance_entry_digest(
                    "src/deleted.rs",
                    &AcceptanceInputKind::File,
                    0o100644,
                    &payload,
                ),
                payload_digest: payload,
                owners: vec!["change".into()],
            }],
        };
        let current = acceptance_manifest_with_signed_owners(root, &record, &[], &signed).unwrap();
        let deleted = current
            .entries
            .iter()
            .find(|entry| entry.path == "src/deleted.rs")
            .unwrap();
        assert_eq!(deleted.kind, AcceptanceInputKind::Missing);
        assert_eq!(deleted.payload_digest, sha256_hex(b""));
    }

    #[test]
    fn duplicate_active_and_archived_locations_fail_closed() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_no_spec_record(root);
        let archived = root
            .join(ARCHIVE_PATH)
            .join(format!("2026-07-14-{}", record.id));
        fs::create_dir_all(&archived).unwrap();
        let mut archived_record = record.clone();
        archived_record.state = ChangeState::Archived;
        write_json(&archived.join("state.json"), &archived_record).unwrap();
        assert!(
            find_change_dir(root, &record.id)
                .unwrap_err()
                .contains("ambiguous")
        );
        assert!(
            list_all_changes_checked(root)
                .unwrap_err()
                .contains("multiple")
        );
        assert_eq!(
            terminal_evidence_summary(root, &archived_record).validity,
            TerminalEvidenceValidity::CorruptHistory
        );
    }

    #[test]
    fn legacy_archive_tombstones_without_lifecycle_state_are_skipped() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(ARCHIVE_PATH).join("legacy/deltas")).unwrap();
        fs::write(
            root.join(ARCHIVE_PATH).join("legacy/deltas/auth.md"),
            "## REMOVED\n### REQUIREMENT REQ-auth-001\nRetired.\n",
        )
        .unwrap();
        assert!(list_all_changes_checked(root).unwrap().is_empty());
    }

    #[test]
    fn legacy_archive_baseline_bytes_are_canonical_sorted_and_definition_digestible() {
        let entry = |id: &str, path: &str| LegacyArchiveBaselineEntryV1 {
            id: id.into(),
            archive_path: path.into(),
            introduction_commit: "1111111111111111111111111111111111111111".into(),
            subtree_digest: "a".repeat(64),
        };
        let baseline = LegacyArchiveBaselineV1 {
            schema_version: 1,
            domain: "specsync.legacy-archive-baseline.v1".into(),
            authority_change_id: "CHG-0042-authority".into(),
            cutoff_commit: "2222222222222222222222222222222222222222".into(),
            entries: vec![
                entry(
                    "CHG-0001-first",
                    ".specsync/archive/changes/2026-07-11-CHG-0001-first",
                ),
                entry(
                    "CHG-0002-second",
                    ".specsync/archive/changes/2026-07-11-CHG-0002-second",
                ),
            ],
        };
        let bytes = json_content(&baseline).unwrap();
        let (_, digest) = validate_legacy_archive_baseline_bytes(bytes.as_bytes()).unwrap();
        validate_sha256_digest(&digest, "baseline digest").unwrap();

        let compact = serde_json::to_vec(&baseline).unwrap();
        assert!(
            validate_legacy_archive_baseline_bytes(&compact)
                .unwrap_err()
                .contains("canonical persisted JSON")
        );

        let mut unsorted = baseline.clone();
        unsorted.entries.reverse();
        let unsorted = json_content(&unsorted).unwrap();
        assert!(
            validate_legacy_archive_baseline_bytes(unsorted.as_bytes())
                .unwrap_err()
                .contains("strictly sorted")
        );

        let mut duplicate = baseline;
        duplicate.entries[1].id = duplicate.entries[0].id.clone();
        let duplicate = json_content(&duplicate).unwrap();
        assert!(
            validate_legacy_archive_baseline_bytes(duplicate.as_bytes())
                .unwrap_err()
                .contains("must each be unique")
        );
    }

    #[test]
    fn definition_approval_binds_the_exact_legacy_baseline_bytes() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("seed"), "seed\n").unwrap();
        git(&["add", "seed"]);
        git(&["commit", "-m", "baseline cutoff"]);
        let cutoff = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        let mut authority = completed_no_spec_record(root);
        authority.base_commit = Some(cutoff.clone());
        authority
            .affected_paths
            .push(LEGACY_BASELINE_PATH.to_string());
        save_change(root, &authority).unwrap();
        write_change_markdown(root, &authority).unwrap();
        fs::create_dir_all(root.join(".specsync/archive")).unwrap();
        let baseline = LegacyArchiveBaselineV1 {
            schema_version: 1,
            domain: "specsync.legacy-archive-baseline.v1".into(),
            authority_change_id: authority.id.clone(),
            cutoff_commit: cutoff,
            entries: Vec::new(),
        };
        let bytes = json_content(&baseline).unwrap();
        fs::write(root.join(LEGACY_BASELINE_PATH), &bytes).unwrap();
        let (_, expected) = validate_legacy_archive_baseline_bytes(bytes.as_bytes()).unwrap();

        let approved =
            approve_definition(root, &authority.id, Some("Reviewer".into()), None).unwrap();

        assert_eq!(
            approved.legacy_archive_baseline_digest.as_deref(),
            Some(expected.as_str())
        );
        assert!(ensure_definition_approval_valid(root, &approved).is_ok());

        let mut changed = baseline;
        fs::write(root.join("later"), "later\n").unwrap();
        git(&["add", "later"]);
        git(&["commit", "-m", "descendant cutoff"]);
        changed.cutoff_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        write_json(&root.join(LEGACY_BASELINE_PATH), &changed).unwrap();
        let changed_bytes = fs::read(root.join(LEGACY_BASELINE_PATH)).unwrap();
        let (_, changed_digest) = validate_legacy_archive_baseline_bytes(&changed_bytes).unwrap();
        assert_ne!(
            approved.legacy_archive_baseline_digest.as_deref(),
            Some(changed_digest.as_str())
        );
        let error = bind_legacy_archive_baseline_authority(root, &mut authority).unwrap_err();
        assert!(error.contains("must equal the authority definition base commit"));
    }

    #[test]
    fn legacy_baseline_cutoff_accepts_only_the_exact_definition_base_in_current_history() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("history"), "root\n").unwrap();
        git(&["add", "history"]);
        git(&["commit", "-m", "root"]);
        let ancestor = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        fs::write(root.join("history"), "base\n").unwrap();
        git(&["commit", "-am", "definition base"]);
        let base = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        let mut authority = completed_no_spec_record(root);
        authority.base_commit = Some(base.clone());

        assert!(validate_legacy_baseline_authority_cutoff(root, &authority, &base).is_ok());
        let earlier =
            validate_legacy_baseline_authority_cutoff(root, &authority, &ancestor).unwrap_err();
        assert!(earlier.contains("must equal the authority definition base commit"));

        fs::write(root.join("history"), "descendant\n").unwrap();
        git(&["commit", "-am", "descendant"]);
        let descendant = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        let later =
            validate_legacy_baseline_authority_cutoff(root, &authority, &descendant).unwrap_err();
        assert!(later.contains("must equal the authority definition base commit"));

        git(&["switch", "-c", "divergent", &ancestor]);
        fs::write(root.join("divergent"), "divergent\n").unwrap();
        git(&["add", "divergent"]);
        git(&["commit", "-m", "divergent"]);
        let divergent = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        git(&["switch", "main"]);
        authority.base_commit = Some(divergent.clone());
        let unrelated =
            validate_legacy_baseline_authority_cutoff(root, &authority, &divergent).unwrap_err();
        assert!(unrelated.contains("not an ancestor of current authority history"));
    }

    #[test]
    fn legacy_baseline_rejects_post_cutoff_archive_additions() {
        let baseline = LegacyArchiveBaselineV1 {
            schema_version: 1,
            domain: "specsync.legacy-archive-baseline.v1".into(),
            authority_change_id: "CHG-0043-authority".into(),
            cutoff_commit: "2222222222222222222222222222222222222222".into(),
            entries: vec![LegacyArchiveBaselineEntryV1 {
                id: "CHG-0001-pre-cutoff".into(),
                archive_path: ".specsync/archive/changes/2026-07-11-CHG-0001-pre-cutoff".into(),
                introduction_commit: "1111111111111111111111111111111111111111".into(),
                subtree_digest: "a".repeat(64),
            }],
        };

        assert!(legacy_baseline_entry(&baseline, "CHG-0001-pre-cutoff").is_ok());
        let error = legacy_baseline_entry(&baseline, "CHG-0044-post-cutoff").unwrap_err();
        assert!(error.contains("not enumerated by the baseline"));
    }

    #[test]
    fn legacy_archive_subtree_digest_binds_path_mode_kind_and_payload() {
        let snapshot = BTreeMap::from([
            ("approvals.json".into(), (0o100644, b"approval".to_vec())),
            ("tool".into(), (0o100755, b"binary".to_vec())),
            ("link".into(), (0o120000, b"tool".to_vec())),
        ]);
        let expected = legacy_archive_subtree_digest(&snapshot).unwrap();
        for changed in [
            BTreeMap::from([
                ("approvals.json".into(), (0o100644, b"tampered".to_vec())),
                ("tool".into(), (0o100755, b"binary".to_vec())),
                ("link".into(), (0o120000, b"tool".to_vec())),
            ]),
            BTreeMap::from([
                ("approvals.json".into(), (0o100644, b"approval".to_vec())),
                ("tool".into(), (0o100644, b"binary".to_vec())),
                ("link".into(), (0o120000, b"tool".to_vec())),
            ]),
            BTreeMap::from([
                ("renamed.json".into(), (0o100644, b"approval".to_vec())),
                ("tool".into(), (0o100755, b"binary".to_vec())),
                ("link".into(), (0o120000, b"tool".to_vec())),
            ]),
        ] {
            assert_ne!(legacy_archive_subtree_digest(&changed).unwrap(), expected);
        }
        let escaped = BTreeMap::from([("link".into(), (0o120000, b"/tool".to_vec()))]);
        assert!(legacy_archive_subtree_digest(&escaped).is_err());
        let gitlink = BTreeMap::from([("nested".into(), (0o160000, vec![0; 20]))]);
        assert!(legacy_archive_subtree_digest(&gitlink).is_err());
    }

    #[test]
    fn dated_lifecycle_archive_missing_state_fails_global_enumeration() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let archived = root
            .join(ARCHIVE_PATH)
            .join("2026-07-15-CHG-0001-missing-state");
        fs::create_dir_all(archived.join("deltas")).unwrap();
        fs::write(
            archived.join("deltas/change.md"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nTampered.\n",
        )
        .unwrap();
        fs::write(archived.join("approvals.json"), "{}\n").unwrap();

        let all_error = list_all_changes_checked(root).unwrap_err();
        assert!(
            all_error.contains("failed to read archived state"),
            "{all_error}"
        );
        let sequence_error = located_change_sequences(root).unwrap_err();
        assert!(
            sequence_error.contains("failed to read archived change state"),
            "{sequence_error}"
        );
    }

    #[test]
    fn status_and_check_share_exact_and_stale_terminal_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        let exact = summarize_change(root, &record).terminal_evidence.unwrap();
        assert_eq!(exact.validity, TerminalEvidenceValidity::Exact);
        assert!(exact.reason.is_none());
        assert_eq!(check_project(root).terminal_evidence[0].evidence, exact);

        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        let stale = summarize_change(root, &record).terminal_evidence.unwrap();
        assert_eq!(stale.validity, TerminalEvidenceValidity::Stale);
        assert!(
            stale
                .reason
                .as_deref()
                .is_some_and(|reason| !reason.is_empty())
        );
        assert_eq!(check_project(root).terminal_evidence[0].evidence, stale);
    }

    #[test]
    fn strict_check_reports_standalone_unprovable_archived_history() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        let source = change_dir(root, &record.id);
        let archived_dir = root
            .join(ARCHIVE_PATH)
            .join(format!("2026-07-14-{}", record.id));
        fs::create_dir_all(archived_dir.parent().unwrap()).unwrap();
        fs::rename(&source, &archived_dir).unwrap();
        record.state = ChangeState::Archived;
        write_json(&archived_dir.join("state.json"), &record).unwrap();

        let report = check_project(root);
        assert!(report.errors.iter().any(|error| {
            error.contains(&record.id) && error.contains("archived change historical integrity")
        }));
        assert_eq!(report.terminal_evidence.len(), 1);
        assert_eq!(
            report.terminal_evidence[0].evidence.validity,
            TerminalEvidenceValidity::CorruptHistory
        );
        assert_eq!(
            summarize_change(root, &record).next_action,
            "invalid archived evidence"
        );
    }

    #[test]
    fn normal_merge_does_not_create_a_duplicate_accepted_transition() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["switch", "-c", "feature"]);
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept feature"]);
        git(&["switch", "main"]);
        git(&["merge", "--no-ff", "feature", "-m", "merge feature"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        let (anchor, _, accepted) = authenticated_accepted_transition(root, &record).unwrap();
        assert_eq!(accepted.id, record.id);
        assert_ne!(anchor, git_output(root, &["rev-parse", "HEAD"]).unwrap());
        assert!(ensure_closing_approval_valid(root, &record).is_ok());
    }

    #[test]
    fn archive_post_move_failure_restores_exact_source_bytes_without_residue() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);

        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept change"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        let source = change_dir(root, &record.id);
        let original_state = fs::read(source.join("state.json")).unwrap();
        let original_markdown = fs::read(source.join("change.md")).unwrap();
        let error = archive_change_with_finalize_failure(root, &record.id, true).unwrap_err();
        assert!(error.contains("source restored"), "{error}");
        assert_eq!(fs::read(source.join("state.json")).unwrap(), original_state);
        assert_eq!(
            fs::read(source.join("change.md")).unwrap(),
            original_markdown
        );
        assert!(!source.join("accepted-state.json").exists());
        assert!(
            !root
                .join(ARCHIVE_PATH)
                .join(format!("{}-{}", today(), record.id))
                .exists()
        );
    }

    #[test]
    fn authenticated_archive_ignores_later_input_drift_but_rejects_snapshot_tampering() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept change"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let archived_dir = archive_change(root, &record.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "archive change"]);

        let snapshot = fs::read(archived_dir.join("accepted-state.json")).unwrap();
        fs::remove_file(archived_dir.join("accepted-state.json")).unwrap();
        let missing_snapshot =
            validate_archived_integrity(root, &load_change(root, &record.id).unwrap()).unwrap_err();
        assert!(missing_snapshot.contains("missing its authenticated accepted-state snapshot"));
        fs::write(archived_dir.join("accepted-state.json"), &snapshot).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "evolve archived input"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let report = check_project(root);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(
            report.terminal_evidence[0].evidence.validity,
            TerminalEvidenceValidity::AuthenticatedHistory
        );

        fs::write(archived_dir.join("accepted-state.json"), b"{}\n").unwrap();
        let report = check_project(root);
        assert!(
            report.errors.iter().any(|error| {
                error.contains(&record.id) && error.contains("historical integrity")
            })
        );
        assert_eq!(
            report.terminal_evidence[0].evidence.validity,
            TerminalEvidenceValidity::CorruptHistory
        );
    }

    #[test]
    fn gitlink_manifest_hashes_the_exact_index_object_id() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("seed"), "seed\n").unwrap();
        git(&["add", "seed"]);
        git(&["commit", "-m", "seed"]);
        let object = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{object},vendor/library"),
        ]);
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Accepted;
        record.affected_paths = vec!["vendor/library".into()];
        let manifest = acceptance_manifest(root, &record, &[]).unwrap();
        let entry = manifest
            .entries
            .iter()
            .find(|entry| entry.path == "vendor/library")
            .unwrap();
        assert_eq!(entry.kind, AcceptanceInputKind::Gitlink);
        assert_eq!(entry.mode, 0o160000);
        assert_eq!(entry.payload_digest, sha256_hex(object.as_bytes()));
    }

    #[test]
    fn persisted_supersedes_cycle_fails_before_predecessor_manifest_use() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut first = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nFirst.\n",
        );
        let mut second = create_change(
            root,
            CreateChangeRequest {
                description: "Second successor".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        let obligation = SuccessionObligation {
            path: "src/auth.rs".into(),
            module: "auth".into(),
            predecessor_entry_digest: sha256_hex(b"entry"),
        };
        first.state = ChangeState::Accepted;
        first.supersedes = vec![SupersedesEdge {
            predecessor_id: second.id.clone(),
            obligations: vec![obligation.clone()],
        }];
        save_change(root, &first).unwrap();
        second.supersedes = vec![SupersedesEdge {
            predecessor_id: first.id.clone(),
            obligations: vec![obligation],
        }];
        let error = validate_supersedes_semantics(root, &second).unwrap_err();
        assert!(error.contains("succession cycle"), "{error}");
    }

    #[test]
    fn explicit_semantic_successor_covers_changed_entry_but_rejects_unchanged_entry() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);

        let delta =
            "## MODIFIED\n### SPEC SECTION Invariants\n\nAuthentication remains governed.\n";
        let mut predecessor = completed_section_only_record(root, delta);
        predecessor =
            approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
        predecessor = start_implementation(root, &predecessor.id).unwrap();
        verify_change(root, &predecessor.id).unwrap();
        predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept predecessor"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let predecessor_manifest = load_verification(root, &predecessor)
            .unwrap()
            .acceptance_manifest
            .unwrap();
        let mut successor = completed_section_only_record(root, delta);
        successor.affected_paths.extend(
            predecessor_manifest
                .entries
                .iter()
                .filter(|entry| {
                    entry.owners.iter().any(|owner| owner == "auth")
                        && entry.path != "specs/auth/requirements.md"
                })
                .map(|entry| entry.path.clone()),
        );
        successor.affected_paths.sort();
        successor.affected_paths.dedup();
        save_change(root, &successor).unwrap();
        write_change_markdown(root, &successor).unwrap();
        for entry in predecessor_manifest.entries.iter().filter(|entry| {
            entry.owners.iter().any(|owner| owner == "auth")
                && entry.path != "specs/auth/requirements.md"
        }) {
            successor = add_supersedes_obligation(
                root,
                &successor.id,
                &predecessor.id,
                &entry.path,
                "auth",
                &entry.entry_digest,
            )
            .unwrap();
        }
        successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
        successor = start_implementation(root, &successor.id).unwrap();
        fs::write(root.join("src/auth.rs"), "// Authentication module v2.\n").unwrap();
        verify_change(root, &successor.id).unwrap();
        successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept semantic successor"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let predecessor_evidence = summarize_change(root, &predecessor)
            .terminal_evidence
            .unwrap();
        assert_eq!(
            predecessor_evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            predecessor_evidence.reason
        );
        let archived_successor = archive_change(root, &successor.id).unwrap();
        assert!(archived_successor.is_dir());
        let recursive_evidence = summarize_change(root, &predecessor)
            .terminal_evidence
            .unwrap();
        assert_eq!(
            recursive_evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            recursive_evidence.reason
        );
        let post_move = check_project(root);
        assert!(post_move.errors.is_empty(), "{:?}", post_move.errors);
        assert!(post_move.terminal_evidence.iter().any(|result| {
            result.id == successor.id
                && result.evidence.validity == TerminalEvidenceValidity::AuthenticatedHistory
        }));

        let successor_manifest = load_verification(root, &successor)
            .unwrap()
            .acceptance_manifest
            .unwrap();
        let current_source = fs::read(root.join("src/auth.rs")).unwrap();
        let current_spec = fs::read(root.join("specs/auth/auth.spec.md")).unwrap();
        let mut unchanged = completed_section_only_record(root, delta);
        fs::write(root.join("src/auth.rs"), current_source).unwrap();
        fs::write(root.join("specs/auth/auth.spec.md"), current_spec).unwrap();
        unchanged.affected_paths.extend(
            successor_manifest
                .entries
                .iter()
                .filter(|entry| {
                    entry.owners.iter().any(|owner| owner == "auth")
                        && entry.path != "specs/auth/requirements.md"
                })
                .map(|entry| entry.path.clone()),
        );
        unchanged.affected_paths.sort();
        unchanged.affected_paths.dedup();
        save_change(root, &unchanged).unwrap();
        write_change_markdown(root, &unchanged).unwrap();
        for entry in successor_manifest.entries.iter().filter(|entry| {
            entry.owners.iter().any(|owner| owner == "auth")
                && entry.path != "specs/auth/requirements.md"
        }) {
            unchanged = add_supersedes_obligation(
                root,
                &unchanged.id,
                &successor.id,
                &entry.path,
                "auth",
                &entry.entry_digest,
            )
            .unwrap();
        }
        unchanged = approve_definition(root, &unchanged.id, Some("Reviewer".into()), None).unwrap();
        start_implementation(root, &unchanged.id).unwrap();
        verify_change(root, &unchanged.id).unwrap();
        let error = accept_change(root, &unchanged.id, Some("Closer".into()), None).unwrap_err();
        assert!(
            error.contains("does not change the predecessor entry"),
            "{error}"
        );
        fs::write(root.join("src/auth.rs"), "// Authentication module v3.\n").unwrap();
        verify_change(root, &unchanged.id).unwrap();
        let recursive_successor =
            accept_change(root, &unchanged.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept recursive semantic successor"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        assert_eq!(recursive_successor.state, ChangeState::Accepted);
        let intermediate_evidence = summarize_change(root, &successor)
            .terminal_evidence
            .unwrap();
        assert_eq!(
            intermediate_evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            intermediate_evidence.reason
        );
        let final_evidence = summarize_change(root, &predecessor)
            .terminal_evidence
            .unwrap();
        assert_eq!(
            final_evidence.validity,
            TerminalEvidenceValidity::SuccessorCovered,
            "{:?}",
            final_evidence.reason
        );
        let recursive_report = check_project(root);
        assert!(
            recursive_report.errors.is_empty(),
            "{:?}",
            recursive_report.errors
        );
    }

    #[test]
    fn legacy_reconstruction_deduplicates_identical_transitions_but_rejects_distinct_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);

        let delta =
            "## MODIFIED\n### SPEC SECTION Invariants\n\nLegacy evidence remains governed.\n";
        let mut record = completed_section_only_record(root, delta);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

        let signed_legacy_digest = acceptance_input_digest(root, &record, &[]).unwrap();
        let mut verification = load_verification(root, &record).unwrap();
        verification.acceptance_input_digest = Some(signed_legacy_digest.clone());
        verification.acceptance_manifest = None;
        verification.semantic_succession = None;
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        let mut ledger = load_approvals(root, &record).unwrap();
        ledger
            .approvals
            .iter_mut()
            .rev()
            .find(|approval| approval.gate == "acceptance")
            .unwrap()
            .digest = closing_digest(&record, &verification);
        write_json(
            &change_dir(root, &record.id).join("approvals.json"),
            &ledger,
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "record legacy acceptance"]);

        let lifecycle_dir = change_dir(root, &record.id);
        let accepted_state = fs::read(lifecycle_dir.join("state.json")).unwrap();
        let accepted_markdown = fs::read(lifecycle_dir.join("change.md")).unwrap();
        let accepted_verification = fs::read(lifecycle_dir.join("verification.json")).unwrap();
        let accepted_approvals = fs::read(lifecycle_dir.join("approvals.json")).unwrap();
        let commit_transition = |message: &str, distinct_evidence: bool| {
            let mut verifying = record.clone();
            verifying.state = ChangeState::Verifying;
            save_change(root, &verifying).unwrap();
            write_change_markdown(root, &verifying).unwrap();
            git(&["add", "."]);
            git(&["commit", "-m", &format!("prepare {message}")]);
            fs::write(lifecycle_dir.join("state.json"), &accepted_state).unwrap();
            fs::write(lifecycle_dir.join("change.md"), &accepted_markdown).unwrap();
            if distinct_evidence {
                let mut distinct_verification = verification.clone();
                distinct_verification.timestamp += 1;
                write_json(
                    &lifecycle_dir.join("verification.json"),
                    &distinct_verification,
                )
                .unwrap();
                let mut distinct_ledger = ledger.clone();
                distinct_ledger
                    .approvals
                    .iter_mut()
                    .rev()
                    .find(|approval| approval.gate == "acceptance")
                    .unwrap()
                    .digest = closing_digest(&record, &distinct_verification);
                write_json(&lifecycle_dir.join("approvals.json"), &distinct_ledger).unwrap();
            } else {
                fs::write(
                    lifecycle_dir.join("verification.json"),
                    &accepted_verification,
                )
                .unwrap();
                fs::write(lifecycle_dir.join("approvals.json"), &accepted_approvals).unwrap();
            }
            git(&["add", "."]);
            git(&["commit", "-m", message]);
        };

        commit_transition("repeat identical legacy acceptance", false);
        assert!(
            reconstruct_legacy_acceptance_manifest(root, &record, &signed_legacy_digest).is_ok()
        );

        commit_transition("repeat distinct legacy acceptance", true);
        let error = reconstruct_legacy_acceptance_manifest(root, &record, &signed_legacy_digest)
            .unwrap_err();
        assert!(error.contains("found 2"), "{error}");
    }

    #[test]
    fn change_ids_are_sequential_and_readable() {
        let temp = TempDir::new().unwrap();
        let first = create_change(
            temp.path(),
            CreateChangeRequest {
                description: "Add passkeys".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec![],
                affected_paths: vec![],
                requested_artifacts: vec![],
                no_spec_change: true,
                rationale: Some("test".into()),
            },
        )
        .unwrap();
        let second = create_change(
            temp.path(),
            CreateChangeRequest {
                description: "Fix login".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec![],
                affected_paths: vec![],
                requested_artifacts: vec![],
                no_spec_change: true,
                rationale: Some("test".into()),
            },
        )
        .unwrap();
        assert_eq!(first.id, "CHG-0001-add-passkeys");
        assert_eq!(second.id, "CHG-0002-fix-login");
    }

    #[test]
    fn concurrent_change_creation_assigns_unique_ids() {
        let temp = TempDir::new().unwrap();
        let root = std::sync::Arc::new(temp.path().to_path_buf());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let handles: Vec<_> = (0..8)
            .map(|index| {
                let root = std::sync::Arc::clone(&root);
                let barrier = std::sync::Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    create_change(
                        &root,
                        CreateChangeRequest {
                            description: format!("Concurrent change {index}"),
                            kind: ChangeKind::Operations,
                            affected_specs: Vec::new(),
                            affected_paths: vec![format!("ops/{index}/")],
                            requested_artifacts: Vec::new(),
                            no_spec_change: true,
                            rationale: Some("Concurrency fixture".into()),
                        },
                    )
                    .unwrap()
                    .id
                })
            })
            .collect();
        let ids: BTreeSet<String> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(ids.len(), 8);
        assert_eq!(list_changes(&root).len(), 8);
    }

    #[test]
    fn sequence_ledger_rejects_unacknowledged_active_and_archived_collisions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let first = create_change(
            root,
            CreateChangeRequest {
                description: "First claim".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/first".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();
        let mut archived = first.clone();
        archived.id = "CHG-0001-archived-claim".into();
        archived.slug = "archived-claim".into();
        archived.state = ChangeState::Archived;
        let archived_dir = root
            .join(ARCHIVE_PATH)
            .join("2026-07-13-CHG-0001-archived-claim");
        fs::create_dir_all(&archived_dir).unwrap();
        write_json(&archived_dir.join("state.json"), &archived).unwrap();

        let error = validate_change_sequences(root).unwrap_err();
        assert!(error.contains("duplicate numeric change sequence CHG-0001"));
        assert!(error.contains(&first.id));
        assert!(error.contains(&archived.id));
        assert!(error.contains(".specsync/changes"));
        assert!(error.contains(".specsync/archive/changes"));
    }

    #[test]
    fn exact_historical_collision_baseline_preserves_immutable_records() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut first = create_change(
            root,
            CreateChangeRequest {
                description: "First claim".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/first".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();
        first.state = ChangeState::Accepted;
        save_change(root, &first).unwrap();
        let mut second = first.clone();
        second.id = "CHG-0001-second-claim".into();
        second.slug = "second-claim".into();
        second.state = ChangeState::Archived;
        let archived_dir = root
            .join(ARCHIVE_PATH)
            .join("2026-07-14-CHG-0001-second-claim");
        fs::create_dir_all(&archived_dir).unwrap();
        write_json(&archived_dir.join("state.json"), &second).unwrap();
        let mut ids = vec![first.id.clone(), second.id.clone()];
        ids.sort();
        write_json(
            &root.join(SEQUENCE_PATH),
            &ChangeSequenceLedger {
                schema_version: 1,
                sequence: 1,
                id: first.id.clone(),
                acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
            },
        )
        .unwrap();

        assert!(validate_change_sequences(root).is_ok());
        let ledger = load_change_sequence_ledger(root).unwrap().unwrap();
        assert_eq!(ledger.id, first.id);

        fs::remove_dir_all(archived_dir).unwrap();
        let error = validate_change_sequences(root).unwrap_err();
        assert!(error.contains("no longer matches the exact historical ID set"));
        assert!(error.contains(&second.id));
    }

    #[test]
    fn acknowledged_collision_rejects_mutable_active_records() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let first = create_change(
            root,
            CreateChangeRequest {
                description: "First mutable claim".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/first".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();
        let mut second = first.clone();
        second.id = "CHG-0001-second-mutable-claim".into();
        second.slug = "second-mutable-claim".into();
        fs::create_dir_all(change_dir(root, &second.id)).unwrap();
        save_change(root, &second).unwrap();
        let mut ids = vec![first.id.clone(), second.id.clone()];
        ids.sort();
        write_json(
            &root.join(SEQUENCE_PATH),
            &ChangeSequenceLedger {
                schema_version: 1,
                sequence: 1,
                id: first.id.clone(),
                acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 1, ids }],
            },
        )
        .unwrap();

        let error = validate_change_sequences(root).unwrap_err();
        assert!(error.contains("includes a mutable change"));
    }

    #[test]
    fn change_sequences_allow_more_than_four_digits() {
        assert_eq!(change_sequence("CHG-9999-last-four-digit"), Some(9999));
        assert_eq!(change_sequence("CHG-10000-first-five-digit"), Some(10000));
        assert_eq!(change_sequence("CHG-123-too-short"), None);
        assert_eq!(change_sequence("CHG-abcd-malformed"), None);
        assert_eq!(change_sequence("CHG-09999-noncanonical-width"), None);
        assert_eq!(change_sequence("CHG-18446744073709551616-overflow"), None);
        assert!(change_id_sorts_after(
            "CHG-10000-first-five-digit",
            "CHG-9999-last-four-digit"
        ));
        assert!(change_id_sorts_after(
            "CHG-9999-second-collision",
            "CHG-9999-first-collision"
        ));
        assert!(!change_id_sorts_after(
            "CHG-09999-noncanonical-width",
            "CHG-9999-last-four-digit"
        ));
        assert!(!change_id_sorts_after(
            "CHG-10000-first-five-digit",
            "not-a-change-id"
        ));
    }

    #[test]
    fn lifecycle_lock_releases_when_owner_drops() {
        let temp = TempDir::new().unwrap();
        let first = acquire_project_lock(temp.path()).unwrap();
        drop(first);
        let second = acquire_project_lock(temp.path()).unwrap();
        drop(second);
    }

    #[test]
    fn archive_waits_until_delivery_diff_no_longer_needs_coverage() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "base"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "feature"]);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        let error = archive_change(root, &record.id).unwrap_err();
        assert!(error.contains("archive after merge"));
        git(&["add", "."]);
        git(&["commit", "-m", "record accepted lifecycle evidence"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        archive_change(root, &record.id).unwrap();
    }

    // Verifies REQ-change-018.
    #[test]
    fn accepted_evidence_survives_integrated_squash_merge_and_archives() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        write_default_policy(root, Vec::new()).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["switch", "-c", "feature"]);

        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement"]);
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept"]);
        let verification = load_verification(root, &record).unwrap();

        git(&["switch", "main"]);
        git(&["merge", "--squash", "feature"]);
        git(&["commit", "-m", "squash feature"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        assert!(!verification_commit_is_accepted_current(
            root,
            &verification
        ));
        assert!(ensure_closing_approval_valid(root, &record).is_ok());

        git(&["switch", "-c", "followup"]);
        git(&["commit", "--allow-empty", "-m", "followup"]);
        assert!(!accepted_workspace_is_integrated(root, &record));
        assert!(accepted_change_is_recorded_in_current_history(
            root, &record
        ));
        assert!(accepted_change_is_recorded_on_remote_default(root, &record));
        assert!(ensure_closing_approval_valid(root, &record).is_ok());

        git(&["switch", "main"]);
        archive_change(root, &record.id).unwrap();
    }

    #[test]
    fn accepted_evidence_survives_squash_merge_from_nested_project_root() {
        let temp = TempDir::new().unwrap();
        let repo_root = temp.path();
        let root = repo_root.join("packages/app");
        fs::create_dir_all(&root).unwrap();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(repo_root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        write_default_policy(&root, Vec::new()).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["switch", "-c", "feature"]);

        let mut record = completed_no_spec_record(&root);
        record = approve_definition(&root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(&root, &record.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement"]);
        verify_change(&root, &record.id).unwrap();
        record = accept_change(&root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept"]);
        let verification = load_verification(&root, &record).unwrap();

        git(&["switch", "main"]);
        git(&["merge", "--squash", "feature"]);
        git(&["commit", "-m", "squash feature"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        assert!(!verification_commit_is_accepted_current(
            &root,
            &verification
        ));
        assert!(ensure_closing_approval_valid(&root, &record).is_ok());
        assert!(accepted_change_is_recorded_in_current_history(
            &root, &record
        ));
        git(&["update-ref", "-d", "refs/remotes/origin/main"]);
        assert!(ensure_closing_approval_valid(&root, &record).is_err());
        let error = reopen_change(
            &root,
            &record.id,
            "Reviewer".into(),
            "The verification commit is off history".into(),
        )
        .unwrap_err();
        assert!(error.contains("delivery inputs are current"), "{error}");
    }

    #[test]
    fn squash_merged_acceptance_reopens_after_a_current_canonical_successor() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        write_default_policy(root, Vec::new()).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["switch", "-c", "feature"]);

        let mut original = completed_no_spec_record(root);
        fs::write(
            root.join("src/auth.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\nNone.\n\n## Invariants\n\nAuthentication remains governed.\n\n## Behavioral Examples\n\nChecks pass.\n\n## Error Cases\n\nInvalid evidence fails.\n\n## Dependencies\n\nNone.\n\n## Legacy Notes\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        original.affected_specs = vec!["auth".into()];
        original.affected_paths = vec!["src/auth.rs".into()];
        save_change(root, &original).unwrap();
        write_change_markdown(root, &original).unwrap();
        original = approve_definition(root, &original.id, Some("Reviewer".into()), None).unwrap();
        original = start_implementation(root, &original.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement original"]);
        verify_change(root, &original.id).unwrap();
        original = accept_change(root, &original.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept original"]);
        let original_verification = load_verification(root, &original).unwrap();

        git(&["switch", "main"]);
        git(&["merge", "--squash", "feature"]);
        git(&["commit", "-m", "squash original"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        let mut no_spec_successor = completed_no_spec_record(root);
        no_spec_successor.affected_specs = original.affected_specs.clone();
        no_spec_successor.affected_paths = original.affected_paths.clone();
        save_change(root, &no_spec_successor).unwrap();
        write_change_markdown(root, &no_spec_successor).unwrap();
        no_spec_successor =
            approve_definition(root, &no_spec_successor.id, Some("Reviewer".into()), None).unwrap();
        no_spec_successor = start_implementation(root, &no_spec_successor.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement no-spec successor"]);
        verify_change(root, &no_spec_successor.id).unwrap();
        no_spec_successor =
            accept_change(root, &no_spec_successor.id, Some("Closer".into()), None).unwrap();
        assert_eq!(no_spec_successor.state, ChangeState::Accepted);
        git(&["add", "."]);
        git(&["commit", "-m", "accept no-spec successor"]);

        assert!(!accepted_change_has_current_canonical_successors(
            root, &original
        ));

        let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nA later semantic change governs authentication.\n";
        let mut successor = completed_section_only_record(root, delta);
        successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
        successor = start_implementation(root, &successor.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement semantic successor"]);
        verify_change(root, &successor.id).unwrap();
        successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept semantic successor"]);

        assert!(!verification_commit_is_accepted_current(
            root,
            &original_verification
        ));
        assert!(!accepted_workspace_is_integrated(root, &original));
        assert!(accepted_change_is_recorded_in_current_history(
            root, &original
        ));
        assert!(accepted_change_has_current_canonical_successors(
            root, &original
        ));
        assert_ne!(
            acceptance_input_digest(root, &original, &[]).unwrap(),
            original_verification
                .acceptance_input_digest
                .clone()
                .unwrap()
        );
        assert!(ensure_closing_approval_valid(root, &original).is_err());
        assert_eq!(summarize_change(root, &original).next_action, "reopen");

        let reopened = reopen_change(
            root,
            &original.id,
            "Release reviewer".into(),
            "A later accepted change superseded the original governed source".into(),
        )
        .unwrap();
        assert_eq!(reopened.change.state, ChangeState::Verifying);
        assert_eq!(
            reopened.audit.reason,
            "A later accepted change superseded the original governed source"
        );
        assert_eq!(
            reopened.audit.prior_verification.contract_digest,
            original_verification.contract_digest
        );
        assert_eq!(
            reopened.audit.prior_verification.acceptance_input_digest,
            original_verification.acceptance_input_digest
        );
        assert_eq!(successor.state, ChangeState::Accepted);
    }

    #[test]
    fn squash_fallback_rejects_unintegrated_or_changed_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        write_default_policy(root, Vec::new()).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git(&["switch", "-c", "feature"]);

        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "implement"]);
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept"]);

        assert!(ensure_closing_approval_valid(root, &record).is_ok());
        git(&["commit", "--allow-empty", "-m", "move head"]);
        let verification = load_verification(root, &record).unwrap();
        assert!(verification_commit_is_accepted_current(root, &verification));

        git(&["switch", "main"]);
        git(&["merge", "--squash", "feature"]);
        git(&["commit", "-m", "squash feature"]);
        assert!(ensure_closing_approval_valid(root, &record).is_err());
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { changed }\n",
        )
        .unwrap();
        let error = ensure_closing_approval_valid(root, &record).unwrap_err();
        assert!(
            error.contains("delivery inputs")
                || error.contains("no closing-valid terminal semantic successor"),
            "{error}"
        );
    }

    #[test]
    fn failed_archive_move_leaves_an_accepted_change_retryable() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        append_approval(
            root,
            &record,
            "definition",
            Some("Reviewer".into()),
            definition_digest(root, &record).unwrap(),
            None,
        )
        .unwrap();
        record.state = ChangeState::Accepted;
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        let verification = VerificationRecord {
            timestamp: now(),
            commit: None,
            contract_digest: definition_digest(root, &record).unwrap(),
            workspace_digest: project_input_digest(root).unwrap(),
            acceptance_input_digest: Some(acceptance_input_digest(root, &record, &[]).unwrap()),
            acceptance_manifest: None,
            semantic_succession: None,
            passed: true,
            commands: Vec::new(),
            requirement_ids: Vec::new(),
        };
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        append_approval(
            root,
            &record,
            "acceptance",
            Some("Reviewer".into()),
            closing_digest(&record, &verification),
            None,
        )
        .unwrap();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["add", "."]);
        git(&["commit", "-m", "record accepted evidence"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let destination = root
            .join(ARCHIVE_PATH)
            .join(format!("{}-{}", today(), record.id));
        fs::create_dir_all(&destination).unwrap();

        let error = archive_change(root, &record.id).unwrap_err();
        assert!(
            error.contains("archive destination already exists"),
            "{error}"
        );
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Accepted
        );
        fs::remove_dir_all(destination).unwrap();
        assert!(archive_change(root, &record.id).is_ok());
    }

    #[test]
    fn semantic_delta_requires_shall_and_criteria() {
        let valid = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL support passkeys.\n\nAcceptance Criteria\n- Works\n";
        let items = parse_delta(valid).unwrap();
        assert_eq!(items.len(), 1);
        validate_requirement(&items[0].key, &items[0].content).unwrap();
        let invalid = "## ADDED\n### REQUIREMENT REQ-auth-001\nSupport passkeys.\n";
        let item = parse_delta(invalid).unwrap().remove(0);
        assert!(validate_requirement(&item.key, &item.content).is_err());
    }

    #[test]
    fn unknown_delta_operation_heading_is_rejected() {
        let typo = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n\n## REMVOED\n### REQUIREMENT REQ-auth-002\nRetired.\n";
        let error = parse_delta(typo).unwrap_err();
        assert!(error.contains("invalid delta operation heading"));
    }

    #[test]
    fn extra_delta_modules_are_rejected() {
        let temp = TempDir::new().unwrap();
        let record = completed_record(temp.path());
        let valid = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n";
        fs::write(delta_path(temp.path(), &record, "auth"), valid).unwrap();
        fs::write(
            change_dir(temp.path(), &record.id).join("deltas/billing.md"),
            valid.replace("REQ-auth", "REQ-billing"),
        )
        .unwrap();
        let error = validate_delta_files(temp.path(), &record).unwrap_err();
        assert!(error.contains("extra: billing"), "{error}");
    }

    #[test]
    fn spec_versions_preserve_integer_or_semantic_format() {
        assert!(
            bump_spec_version("---\nversion: 4\n---\n")
                .unwrap()
                .contains("version: 5")
        );
        assert!(
            bump_spec_version("---\nversion: 1.2.9\n---\n")
                .unwrap()
                .contains("version: 1.2.10")
        );
        assert!(
            bump_spec_version("---\nversion: 1.2.9 # release\n---\n")
                .unwrap()
                .contains("version: 1.2.10 # release")
        );
        assert!(
            bump_spec_version("---\nversion: \"1.2.9\"\n---\n")
                .unwrap()
                .contains("version: \"1.2.10\"")
        );
        assert!(bump_spec_version("---\nversion: one\n---\n").is_err());
    }

    #[test]
    fn stale_definition_approval_is_rejected() {
        let temp = TempDir::new().unwrap();
        let mut record = completed_record(temp.path());
        for artifact in &record.selected_artifacts {
            fs::write(
                change_dir(temp.path(), &record.id).join(artifact.file_name()),
                "complete\n",
            )
            .unwrap();
        }
        fs::write(delta_path(temp.path(), &record, "auth"), "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n").unwrap();
        record =
            approve_definition(temp.path(), &record.id, Some("Reviewer".into()), None).unwrap();
        assert!(ensure_definition_approval_valid(temp.path(), &record).is_ok());
        fs::write(
            change_dir(temp.path(), &record.id).join("context.md"),
            "changed\n",
        )
        .unwrap();
        assert!(ensure_definition_approval_valid(temp.path(), &record).is_err());
        record = approve_definition(
            temp.path(),
            &record.id,
            Some("Reviewer".into()),
            Some("Reapproved updated context".into()),
        )
        .unwrap();
        assert_eq!(record.state, ChangeState::Approved);
        assert!(ensure_definition_approval_valid(temp.path(), &record).is_ok());
    }

    #[test]
    fn false_canonical_application_preserves_legacy_definition_approvals() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        let state_path = change_dir(root, &record.id).join("state.json");

        let draft_json = fs::read_to_string(&state_path).unwrap();
        assert!(!draft_json.contains("canonical_applied"));
        let stable_digest = definition_digest(root, &record).unwrap();
        let explicit_false_digest = definition_digest_with_explicit_false(root, &record).unwrap();
        assert_ne!(stable_digest, explicit_false_digest);
        append_approval(
            root,
            &record,
            "definition",
            Some("Reviewer".into()),
            explicit_false_digest,
            Some("Approved with the transitional explicit-false encoding".into()),
        )
        .unwrap();
        record.state = ChangeState::Approved;
        save_change(root, &record).unwrap();

        let approved_json = fs::read_to_string(&state_path).unwrap();
        assert!(!approved_json.contains("canonical_applied"));
        let loaded = load_change(root, &record.id).unwrap();
        assert!(!loaded.canonical_applied);
        assert!(ensure_definition_approval_valid(root, &loaded).is_ok());

        record.canonical_applied = true;
        save_change(root, &record).unwrap();
        let accepted_json = fs::read_to_string(state_path).unwrap();
        assert!(accepted_json.contains("\"canonical_applied\": true"));
    }

    #[test]
    fn acceptance_normalizes_transitional_definition_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        let stable_digest = definition_digest(root, &record).unwrap();
        let transitional_digest = definition_digest_with_explicit_false(root, &record).unwrap();
        append_approval(
            root,
            &record,
            "definition",
            Some("Original reviewer".into()),
            transitional_digest.clone(),
            Some("Approved with transitional evidence".into()),
        )
        .unwrap();
        record.state = ChangeState::Approved;
        save_change(root, &record).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();

        record = accept_change(
            root,
            &record.id,
            Some("Release reviewer".into()),
            Some("Accepted after verification".into()),
        )
        .unwrap();

        let ledger = load_approvals(root, &record).unwrap();
        assert_eq!(ledger.approvals.len(), 3);
        assert_eq!(ledger.approvals[0].digest, transitional_digest);
        assert_eq!(ledger.approvals[1].gate, "definition");
        assert_eq!(ledger.approvals[1].actor, "Release reviewer");
        assert_eq!(ledger.approvals[1].digest, stable_digest);
        assert_eq!(
            ledger.approvals[1].note.as_deref(),
            Some("Normalized compatible definition evidence during explicit acceptance")
        );
        assert_eq!(ledger.approvals[2].gate, "acceptance");
        assert!(ensure_definition_approval_valid(root, &record).is_ok());
    }

    #[test]
    fn reaccept_accepts_transitional_pre_reopen_definition_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

        let transitional_digest = definition_digest_with_explicit_false(root, &record).unwrap();
        let mut prior_verification = load_verification(root, &record).unwrap();
        prior_verification.contract_digest = transitional_digest.clone();
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &prior_verification,
        )
        .unwrap();
        let mut ledger = load_approvals(root, &record).unwrap();
        let closing = ledger
            .approvals
            .iter_mut()
            .rev()
            .find(|approval| approval.gate == "acceptance")
            .unwrap();
        closing.digest = closing_digest(&record, &prior_verification);
        write_json(
            &change_dir(root, &record.id).join("approvals.json"),
            &ledger,
        )
        .unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_ok());

        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        record = reopen_change(
            root,
            &record.id,
            "Release reviewer".into(),
            "Delivery input changed after legacy acceptance".into(),
        )
        .unwrap()
        .change;
        assert_eq!(
            load_approvals(root, &record).unwrap().reopenings[0]
                .prior_verification
                .contract_digest,
            transitional_digest
        );

        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Accepted);
        assert!(record.canonical_applied);
    }

    #[test]
    fn markdown_blocks_apply_semantically() {
        let source = "# Requirements\n\n### REQ-auth-001\n\nOld\n\n### REQ-auth-002\n\nKeep\n";
        let modified = apply_markdown_block(
            source,
            "### ",
            "REQ-auth-001",
            "New",
            DeltaOperation::Modified,
        )
        .unwrap();
        assert!(modified.contains("### REQ-auth-001\n\nNew"));
        assert!(modified.contains("### REQ-auth-002\n\nKeep"));
        let removed = apply_markdown_block(
            &modified,
            "### ",
            "REQ-auth-001",
            "",
            DeltaOperation::Removed,
        )
        .unwrap();
        assert!(!removed.contains("REQ-auth-001"));
    }

    #[test]
    fn markdown_block_stops_at_higher_level_heading() {
        let source = "# Requirements\n\n## Durable requirements\n\n### REQ-auth-001\n\nOld text.\n\n## Public API\n\n| Name |\n|---|\n| `authenticate` |\n";
        let modified = apply_markdown_block(
            source,
            "### ",
            "REQ-auth-001",
            "New text.",
            DeltaOperation::Modified,
        )
        .unwrap();
        assert!(modified.contains("### REQ-auth-001\n\nNew text."));
        assert!(modified.contains("## Public API\n\n| Name |\n|---|\n| `authenticate` |"));
        let removed =
            apply_markdown_block(source, "### ", "REQ-auth-001", "", DeltaOperation::Removed)
                .unwrap();
        assert!(!removed.contains("REQ-auth-001"));
        assert!(removed.contains("## Public API\n\n| Name |\n|---|\n| `authenticate` |"));
    }

    #[test]
    fn markdown_block_preserves_crlf_and_unrelated_bytes() {
        let source = "# Requirements\r\n\r\n### REQ-auth-001\r\n\r\nOld.\r\n\r\n## Public API  \r\n\r\nKeep trailing spaces.  \r\n";
        let modified = apply_markdown_block(
            source,
            "### ",
            "REQ-auth-001",
            "New.",
            DeltaOperation::Modified,
        )
        .unwrap();
        assert!(!modified.replace("\r\n", "").contains('\n'));
        assert!(modified.ends_with("## Public API  \r\n\r\nKeep trailing spaces.  \r\n"));
    }

    #[test]
    fn malformed_policy_fails_closed() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".specsync")).unwrap();
        fs::write(temp.path().join(POLICY_PATH), "{ invalid json").unwrap();
        let report = check_project(temp.path());
        assert!(report.enabled);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].contains("invalid SDD policy"));
    }

    #[test]
    fn malformed_active_change_state_fails_closed() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let dir = root.join(CHANGES_PATH).join("CHG-0001-corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("state.json"), "{ invalid json").unwrap();

        let report = check_project(root);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("invalid active change state"))
        );
    }

    #[test]
    fn oversized_change_artifacts_are_rejected() {
        let temp = TempDir::new().unwrap();
        let record = completed_no_spec_record(temp.path());
        fs::write(
            change_dir(temp.path(), &record.id).join("context.md"),
            vec![b'x'; MAX_CHANGE_ARTIFACT_BYTES as usize + 1],
        )
        .unwrap();
        let error = validate_artifacts(temp.path(), &record).unwrap_err();
        assert!(error.contains("exceeds") && error.contains("byte limit"));
    }

    #[test]
    fn non_git_policy_disables_only_changed_path_coverage() {
        let temp = TempDir::new().unwrap();
        write_default_policy(temp.path(), Vec::new()).unwrap();
        assert!(
            !load_policy(temp.path())
                .unwrap()
                .require_change_for_meaningful_files
        );
        let report = check_project(temp.path());
        assert!(
            !report
                .errors
                .iter()
                .any(|error| error.contains("changed paths"))
        );
    }

    #[test]
    fn committed_policy_cannot_be_disabled_or_deleted_locally() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        write_default_policy(root, Vec::new()).unwrap();
        fs::write(root.join(".specsync/version"), SDD_VERSION).unwrap();
        git(&["add", ".specsync/sdd.json", ".specsync/version"]);
        git(&["commit", "-m", "enable sdd"]);

        let mut policy = load_policy(root).unwrap();
        policy.enabled = false;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let report = check_project(root);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("changed without")),
            "unexpected report: {report:?}"
        );

        policy.enabled = true;
        policy.require_change_for_meaningful_files = false;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        assert!(
            check_project(root)
                .errors
                .iter()
                .any(|error| error.contains("changed without"))
        );

        policy.require_change_for_meaningful_files = true;
        policy.meaningful_paths.clear();
        policy.ignored_paths.push(POLICY_PATH.into());
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        assert!(
            check_project(root)
                .errors
                .iter()
                .any(|error| error.contains("changed without"))
        );

        fs::remove_file(root.join(POLICY_PATH)).unwrap();
        fs::remove_file(root.join(".specsync/version")).unwrap();
        assert!(
            check_project(root)
                .errors
                .iter()
                .any(|error| error.contains("changed without"))
        );
    }

    #[test]
    fn clean_initial_commit_needs_no_changed_path_coverage() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "clean\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "initial"]);
        assert!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn clean_feature_branch_still_requires_changed_path_coverage() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ]);
        git(&["switch", "-c", "feature"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "feature"]);

        assert_eq!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
            vec!["src/lib.rs"]
        );
    }

    #[test]
    fn approved_change_does_not_cover_delivery_paths_until_started() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "feature"]);
        let mut record = completed_record(root);
        record.affected_paths = vec!["src/".into(), SEQUENCE_PATH.into()];
        record.state = ChangeState::Approved;
        assert_eq!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[record.clone()]).unwrap(),
            vec![SEQUENCE_PATH.to_string(), "src/lib.rs".into()]
        );
        record.state = ChangeState::Implementing;
        assert!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[record])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn local_coverage_unions_staged_unstaged_and_untracked_paths() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "base\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        fs::write(root.join("src/lib.rs"), "unstaged\n").unwrap();
        fs::write(root.join("src/staged.rs"), "staged\n").unwrap();
        git(&["add", "src/staged.rs"]);
        fs::write(root.join("src/untracked.rs"), "untracked\n").unwrap();
        assert_eq!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[]).unwrap(),
            vec!["src/lib.rs", "src/staged.rs", "src/untracked.rs"]
        );
    }

    #[test]
    fn accepted_changes_require_matching_closing_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        append_approval(
            root,
            &record,
            "definition",
            Some("Reviewer".into()),
            definition_digest(root, &record).unwrap(),
            None,
        )
        .unwrap();
        record.state = ChangeState::Accepted;
        save_change(root, &record).unwrap();
        let verification = VerificationRecord {
            timestamp: now(),
            commit: None,
            contract_digest: definition_digest(root, &record).unwrap(),
            workspace_digest: "workspace".into(),
            acceptance_input_digest: None,
            acceptance_manifest: None,
            semantic_succession: None,
            passed: true,
            commands: Vec::new(),
            requirement_ids: Vec::new(),
        };
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_err());
        append_approval(
            root,
            &record,
            "acceptance",
            Some("Reviewer".into()),
            closing_digest(&record, &verification),
            None,
        )
        .unwrap();
        let error = ensure_closing_approval_valid(root, &record).unwrap_err();
        assert!(error.contains("missing current delivery-input evidence"));
        let mut verification = verification;
        verification.acceptance_input_digest =
            Some(acceptance_input_digest(root, &record, &[]).unwrap());
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &verification,
        )
        .unwrap();
        append_approval(
            root,
            &record,
            "acceptance",
            Some("Reviewer".into()),
            closing_digest(&record, &verification),
            Some("Reapprove additive delivery-input evidence".into()),
        )
        .unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_ok());
        let mut ledger = load_approvals(root, &record).unwrap();
        ledger.approvals.last_mut().unwrap().digest = "tampered".into();
        write_json(
            &change_dir(root, &record.id).join("approvals.json"),
            &ledger,
        )
        .unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_err());
    }

    #[test]
    fn working_tree_changes_invalidate_verification() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
        assert!(error.contains("working-tree inputs changed"));
    }

    #[test]
    fn stale_accepted_change_reopens_with_audited_evidence_and_reaccepts() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        let prior_verification = load_verification(root, &record).unwrap();
        let prior_ledger = load_approvals(root, &record).unwrap();
        assert!(check_project(root).errors.is_empty());

        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        let stale_report = check_project(root);
        assert!(stale_report.errors.iter().any(|error| {
            error.contains("accepted change verification is stale for current delivery inputs")
        }));
        assert_eq!(summarize_change(root, &record).next_action, "reopen");

        let reopened = reopen_change(
            root,
            &record.id,
            "Release reviewer".into(),
            "Review fixes changed scoped delivery inputs".into(),
        )
        .unwrap();
        record = reopened.change;
        assert_eq!(record.state, ChangeState::Verifying);
        assert!(record.canonical_applied);
        assert_eq!(reopened.audit.superseded_approval.actor, "Closer");
        assert_eq!(
            reopened.audit.prior_verification.contract_digest,
            prior_verification.contract_digest
        );
        let reopened_ledger = load_approvals(root, &record).unwrap();
        assert_eq!(
            reopened_ledger.approvals.len(),
            prior_ledger.approvals.len()
        );
        assert_eq!(reopened_ledger.reopenings.len(), 1);
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains("verification evidence is stale for the current commit or contract")
        }));

        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Accepted);
        assert!(check_project(root).errors.is_empty());
        let final_ledger = load_approvals(root, &record).unwrap();
        assert_eq!(
            final_ledger.approvals.len(),
            prior_ledger.approvals.len() + 1
        );
        assert_eq!(final_ledger.reopenings.len(), 1);
        assert_eq!(
            final_ledger.reopenings[0]
                .prior_verification
                .contract_digest,
            prior_verification.contract_digest
        );
    }

    #[test]
    fn broad_successor_without_explicit_obligations_cannot_suppress_stale_predecessor() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut predecessor = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nOriginal governed behavior.\n",
        );
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        predecessor =
            approve_definition(root, &predecessor.id, Some("Reviewer".into()), None).unwrap();
        predecessor = start_implementation(root, &predecessor.id).unwrap();
        verify_change(root, &predecessor.id).unwrap();
        predecessor = accept_change(root, &predecessor.id, Some("Closer".into()), None).unwrap();
        assert!(check_project(root).errors.is_empty());

        fs::write(
            root.join("src/auth-extra.rs"),
            "// Existing product surface.\n",
        )
        .unwrap();
        let spec_path = root.join("specs/auth/auth.spec.md");
        let expanded = fs::read_to_string(&spec_path).unwrap().replace(
            "  - src/auth.rs\n",
            "  - src/auth.rs\n  - src/auth-extra.rs\n",
        );
        fs::write(&spec_path, expanded).unwrap();
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));

        let mut successor = create_change(
            root,
            CreateChangeRequest {
                description: "Expand the governed auth surface".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into(), "src/auth-extra.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        successor.acceptance_criteria = vec!["Both auth files remain governed".into()];
        successor
            .answers
            .insert("public_contract".into(), "yes".into());
        successor
            .answers
            .insert("architecture_risk".into(), "no".into());
        save_change(root, &successor).unwrap();
        write_change_markdown(root, &successor).unwrap();
        for artifact in &successor.selected_artifacts {
            fs::write(
                change_dir(root, &successor.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Govern the expanded surface.\n"
                } else {
                    "# Complete\n\nReviewed successor evidence.\n"
                },
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &successor, "auth"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nBoth existing auth files remain governed.\n",
        )
        .unwrap();
        successor = approve_definition(root, &successor.id, Some("Reviewer".into()), None).unwrap();
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));

        successor = start_implementation(root, &successor.id).unwrap();
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));

        let mut policy = load_policy(root).unwrap();
        policy.verification_commands =
            vec!["cargo metadata --manifest-path definitely-missing/Cargo.toml".into()];
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        assert!(verify_change(root, &successor.id).is_err());
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));

        policy.verification_commands.clear();
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        verify_change(root, &successor.id).unwrap();
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));
        successor = accept_change(root, &successor.id, Some("Closer".into()), None).unwrap();
        assert_eq!(successor.state, ChangeState::Accepted);
        assert!(check_project(root).errors.iter().any(|error| {
            error.contains(&predecessor.id) && error.contains("stale for current delivery inputs")
        }));
    }

    #[test]
    fn reopen_rejects_current_evidence_and_requires_explicit_audit_fields() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

        let error =
            reopen_change(root, &record.id, "Reviewer".into(), "Not stale".into()).unwrap_err();
        assert!(error.contains("delivery inputs are current"), "{error}");
        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        let error = reopen_change(root, &record.id, " ".into(), "Review fix".into()).unwrap_err();
        assert!(error.contains("non-empty human actor"), "{error}");
        let error = reopen_change(root, &record.id, "Reviewer".into(), " ".into()).unwrap_err();
        assert!(error.contains("non-empty reason"), "{error}");
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Accepted
        );
    }

    #[test]
    fn reaccept_rejects_definition_changes_after_canonical_application() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closer".into()), None).unwrap();

        fs::write(
            root.join("src/lib.rs"),
            "pub fn ready() -> bool { false }\n",
        )
        .unwrap();
        record = reopen_change(
            root,
            &record.id,
            "Release reviewer".into(),
            "Review fixes changed scoped delivery inputs".into(),
        )
        .unwrap()
        .change;
        fs::write(
            change_dir(root, &record.id).join("testing.md"),
            "# Testing\n\nThe modified definition must not be silently ignored.\n",
        )
        .unwrap();
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Verifying);
        let report = check_project(root);
        assert!(
            report.errors.iter().any(|error| {
                error.contains("modified definition of an already-applied change")
            })
        );
        verify_change(root, &record.id).unwrap();

        let error = accept_change(root, &record.id, Some("Closer".into()), None).unwrap_err();
        assert!(
            error.contains("perform further spec changes in a new change workspace"),
            "{error}"
        );
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Verifying
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn accepted_metadata_correction_preserves_original_evidence_and_adds_artifacts() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let record = accept_completed_record(root, completed_no_spec_record(root));
        let original_answers = record.answers.clone();
        let original_artifacts = record.selected_artifacts.clone();
        let original_approvals =
            serde_json::to_value(load_approvals(root, &record).unwrap()).unwrap();
        let prior_verification = load_verification(root, &record).unwrap();

        let result = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Release reviewer".into(),
            "The accepted change affects persisted lifecycle architecture".into(),
        )
        .unwrap();

        assert_eq!(result.change.state, ChangeState::Verifying);
        assert!(result.change.canonical_applied);
        assert_eq!(result.change.correction_count, 1);
        assert_eq!(result.change.answers, original_answers);
        assert_eq!(result.change.selected_artifacts, original_artifacts);
        assert_eq!(result.correction.original_value, "no");
        assert_eq!(result.correction.prior_effective_value, "no");
        assert_eq!(result.correction.corrected_value, "yes");
        assert_eq!(
            result.correction.prior_verification.contract_digest,
            prior_verification.contract_digest
        );
        assert_eq!(
            result.correction.added_artifacts,
            vec![
                ArtifactKind::Research,
                ArtifactKind::Design,
                ArtifactKind::Plan,
            ]
        );
        assert_eq!(
            result
                .effective_definition
                .answers
                .get("architecture_risk")
                .map(String::as_str),
            Some("yes")
        );
        assert_eq!(result.summary.next_action, "complete artifacts");
        assert!(!result.summary.approval_valid);
        assert_eq!(
            serde_json::to_value(load_approvals(root, &result.change).unwrap()).unwrap(),
            original_approvals
        );
        for artifact in &result.correction.added_artifacts {
            let content =
                fs::read_to_string(change_dir(root, &record.id).join(artifact.file_name()))
                    .unwrap();
            assert!(content.contains("<!-- TODO"));
        }
        let error = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::PublicContract,
            "yes".into(),
            "Release reviewer".into(),
            "A second correction cannot bypass reacceptance".into(),
        )
        .unwrap_err();
        assert!(error.contains("expected accepted"), "{error}");
    }

    #[test]
    fn metadata_correction_rejects_noops_unsupported_fields_and_missing_audit_inputs() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let record = accept_completed_record(root, completed_no_spec_record(root));

        let error = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "no".into(),
            "Reviewer".into(),
            "No change".into(),
        )
        .unwrap_err();
        assert!(error.contains("already `no`"), "{error}");
        let error = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            " ".into(),
            "Missing actor".into(),
        )
        .unwrap_err();
        assert!(error.contains("non-empty human actor"), "{error}");
        let error = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            " ".into(),
        )
        .unwrap_err();
        assert!(error.contains("non-empty reason"), "{error}");
        assert!(CorrectionField::parse("acceptance_criteria").is_err());
        assert!(!change_dir(root, &record.id).join(CORRECTIONS_FILE).exists());
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Accepted
        );
    }

    #[test]
    fn correction_values_preserve_supported_boolean_aliases() {
        for value in ["yes", "y", "true", "1", " YES "] {
            assert_eq!(canonical_correction_value(value).unwrap(), "yes");
        }
        for value in ["no", "n", "false", "0", " NO "] {
            assert_eq!(canonical_correction_value(value).unwrap(), "no");
        }
        assert!(canonical_correction_value("maybe").is_err());
    }

    #[test]
    fn corrected_acceptance_requires_fresh_gates_and_never_replays_canonical_deltas() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let delta = "## MODIFIED\n\n### SPEC SECTION Invariants\n\nCorrected metadata never replays this canonical section.\n";
        let mut record = completed_section_only_record(root, delta);
        record.answers.insert("public_contract".into(), "no".into());
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        record = accept_completed_record(root, record);
        let canonical_path = root.join("specs/auth/auth.spec.md");
        let canonical_after_first_accept = fs::read_to_string(&canonical_path).unwrap();

        let first = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::PublicContract,
            "yes".into(),
            "Release reviewer".into(),
            "The accepted semantic delta changed the public contract".into(),
        )
        .unwrap();
        assert!(first.correction.added_artifacts.is_empty());
        assert_eq!(first.summary.next_action, "approve");
        assert!(ensure_definition_approval_valid(root, &first.change).is_err());
        record =
            approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();
        assert_eq!(
            fs::read_to_string(&canonical_path).unwrap(),
            canonical_after_first_accept
        );
        assert_eq!(correction_history(root, &record).unwrap().len(), 1);

        let second = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::PublicContract,
            "no".into(),
            "Release reviewer".into(),
            "A later audit restored the original classification".into(),
        )
        .unwrap();
        assert_eq!(second.correction.sequence, 2);
        record =
            approve_definition(root, &record.id, Some("Definition reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Closing reviewer".into()), None).unwrap();

        assert_eq!(
            record.answers.get("public_contract").map(String::as_str),
            Some("no")
        );
        assert_eq!(record.correction_count, 2);
        assert_eq!(correction_history(root, &record).unwrap().len(), 2);
        assert_eq!(
            effective_change_definition(root, &record)
                .unwrap()
                .answers
                .get("public_contract")
                .map(String::as_str),
            Some("no")
        );
        assert_eq!(
            fs::read_to_string(canonical_path).unwrap(),
            canonical_after_first_accept
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn trusted_history_rejects_correction_rollback_and_divergent_same_count() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_section_only_record(
            root,
            "## MODIFIED\n\n### SPEC SECTION Invariants\n\nAccepted passkey changes retain lifecycle evidence.\n",
        );
        record.answers.insert("public_contract".into(), "no".into());
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        git(&["add", "."]);
        git(&["commit", "-m", "accept original definition"]);

        let first = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted implementation has architectural risk".into(),
        )
        .unwrap();
        for artifact in &first.correction.added_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept first correction"]);
        let first_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();

        let second = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "no".into(),
            "Reviewer".into(),
            "A follow-up audit removed the architectural risk".into(),
        )
        .unwrap();
        assert_eq!(second.correction.sequence, 2);
        assert!(effective_change_definition(root, &second.change).is_ok());
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept second correction"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let second_commit = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        let workspace = format!("{CHANGES_PATH}/{}", record.id);

        git(&[
            "restore",
            "--source",
            first_commit.as_str(),
            "--",
            &workspace,
        ]);
        let rolled_back = load_change(root, &record.id).unwrap();
        let error = effective_change_definition(root, &rolled_back).unwrap_err();
        assert!(
            error.contains("correction history rollback detected"),
            "{error}"
        );
        git(&[
            "restore",
            "--source",
            second_commit.as_str(),
            "--",
            &workspace,
        ]);

        git(&["switch", "-c", "divergent", first_commit.as_str()]);
        let stale = load_change(root, &record.id).unwrap();
        let error = effective_change_definition(root, &stale).unwrap_err();
        assert!(
            error.contains("correction history rollback detected"),
            "{error}"
        );
        git(&["update-ref", "-d", "refs/remotes/origin/main"]);
        let divergent = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::PublicContract,
            "yes".into(),
            "Reviewer".into(),
            "The public contract classification was corrected independently".into(),
        )
        .unwrap();
        git(&[
            "update-ref",
            "refs/remotes/origin/main",
            second_commit.as_str(),
        ]);
        let error = effective_change_definition(root, &divergent.change).unwrap_err();
        assert!(
            error.contains("correction history divergence detected"),
            "{error}"
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn full_history_finds_a_corrected_anchor_hidden_by_a_treesame_merge_result() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        git(&["add", "."]);
        git(&["commit", "-m", "accept original definition"]);
        let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

        git(&["switch", "-c", "corrected-side-branch"]);
        let correction = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        for artifact in &correction.correction.added_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept corrected definition"]);

        let workspace = format!("{CHANGES_PATH}/{}", record.id);
        git(&[
            "restore",
            "--source",
            original.as_str(),
            "--",
            workspace.as_str(),
        ]);
        git(&["add", "."]);
        git(&["commit", "-m", "roll back corrected workspace"]);
        git(&["switch", "main"]);
        git(&[
            "merge",
            "--no-ff",
            "corrected-side-branch",
            "-m",
            "merge side-branch history",
        ]);

        let rolled_back = load_change(root, &record.id).unwrap();
        assert_eq!(rolled_back.correction_count, 0);
        let error = effective_change_definition(root, &rolled_back).unwrap_err();
        assert!(
            error.contains("correction history rollback detected"),
            "{error}"
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn trusted_history_ignores_a_dangling_remote_default_symbolic_ref() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let record = accept_completed_record(root, completed_no_spec_record(root));
        git(&["add", "."]);
        git(&["commit", "-m", "accept definition"]);
        git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/missing",
        ]);

        assert!(effective_change_definition(root, &record).is_ok());
    }

    // Verifies REQ-change-032.
    #[test]
    fn historical_git_paths_are_nul_safe_in_a_non_ascii_project_directory() {
        let temp = TempDir::new().unwrap();
        let repository = temp.path();
        let fixture = if cfg!(windows) {
            "fixtures/naïve quoted"
        } else {
            "fixtures/naïve \"quoted\""
        };
        let root = repository.join(fixture);
        fs::create_dir_all(&root).unwrap();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(repository)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(&root, Vec::new()).unwrap();
        let mut record = completed_section_only_record(
            &root,
            "## MODIFIED\n\n### SPEC SECTION Invariants\n\nHistorical paths remain byte-delimited.\n",
        );
        record.answers.insert("public_contract".into(), "no".into());
        save_change(&root, &record).unwrap();
        write_change_markdown(&root, &record).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(&root, record);
        git(&["add", "."]);
        git(&["commit", "-m", "accept original definition"]);

        record = correct_interview_metadata(
            &root,
            &record.id,
            CorrectionField::PublicContract,
            "yes".into(),
            "Reviewer".into(),
            "The accepted semantic delta changes a public contract".into(),
        )
        .unwrap()
        .change;
        record = approve_definition(&root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(&root, &record.id).unwrap();
        record = accept_change(&root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept corrected definition"]);

        let commit = git_output(&root, &["rev-parse", "HEAD"]).unwrap();
        let directory =
            git_repo_relative_path(&root, &format!("{CHANGES_PATH}/{}", record.id)).unwrap();
        assert_eq!(
            historical_change_directories(&root, &commit, &record.id).unwrap(),
            vec![directory.clone()]
        );
        assert!(
            closing_authenticated_correction_anchor(&root, &commit, &directory, &record.id)
                .unwrap()
                .is_some()
        );
        assert!(
            git_entry_at_commit(&root, &commit, &format!("{directory}/state.json"))
                .unwrap()
                .is_some()
        );
        assert!(effective_change_definition(&root, &record).is_ok());
    }

    // Verifies REQ-change-032.
    #[test]
    fn archived_change_uses_prior_active_correction_anchor() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["config", "core.autocrlf", "false"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        let correction = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        for artifact in &correction.correction.added_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "accept corrected definition"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let archive = archive_change(root, &record.id).unwrap();
        let archived = load_change(root, &record.id).unwrap();
        assert_eq!(archived.state, ChangeState::Archived);
        assert!(effective_change_definition(root, &archived).is_ok());

        let mut ledger = load_correction_ledger(root, &archived).unwrap();
        ledger.corrections.clear();
        write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
        let mut rolled_back = archived;
        rolled_back.correction_count = 0;
        write_json(&archive.join("state.json"), &rolled_back).unwrap();
        let error = effective_change_definition(root, &rolled_back).unwrap_err();
        assert!(
            error.contains("correction history rollback detected"),
            "{error}"
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn archived_only_corrected_snapshot_remains_a_trusted_anchor() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        git(&["add", "."]);
        git(&["commit", "-m", "accept original definition"]);

        let correction = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        for artifact in &correction.correction.added_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        let archive = archive_change(root, &record.id).unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "record only archived corrected snapshot"]);
        assert!(
            git_output(
                root,
                &[
                    "log",
                    "--format=%H",
                    "--",
                    &format!("{CHANGES_PATH}/{}/corrections.json", record.id),
                ],
            )
            .is_none()
        );

        let archived = load_change(root, &record.id).unwrap();
        assert_eq!(archived.state, ChangeState::Archived);
        let mut ledger = load_correction_ledger(root, &archived).unwrap();
        ledger.corrections.clear();
        write_json(&archive.join(CORRECTIONS_FILE), &ledger).unwrap();
        let mut rolled_back = archived;
        rolled_back.correction_count = 0;
        write_json(&archive.join("state.json"), &rolled_back).unwrap();

        let error = effective_change_definition(root, &rolled_back).unwrap_err();
        assert!(
            error.contains("correction history rollback detected"),
            "{error}"
        );
    }

    // Verifies REQ-change-032.
    #[test]
    fn shallow_history_with_corrections_fails_closed() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        let correction = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        let head = git_output(root, &["rev-parse", "HEAD"]).unwrap();
        fs::write(root.join(".git/shallow"), format!("{head}\n")).unwrap();
        let error = effective_change_definition(root, &correction.change).unwrap_err();
        assert!(error.contains("shallow Git checkout"), "{error}");
    }

    // Verifies REQ-change-032.
    #[test]
    fn shallow_rollback_tip_cannot_hide_a_corrected_acceptance() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("source");
        let clone = temp.path().join("clone");
        fs::create_dir_all(&source).unwrap();
        let git = |root: &Path, args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success(),
                "git {} failed",
                args.join(" ")
            );
        };
        git(&source, &["init", "-b", "main"]);
        git(&source, &["config", "user.email", "test@example.com"]);
        git(&source, &["config", "user.name", "Test"]);
        fs::write(source.join("README.md"), "# Fixture\n").unwrap();
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "base"]);
        write_default_policy(&source, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(&source);
        record = accept_completed_record(&source, record);
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "accept original definition"]);
        let original = git_output(&source, &["rev-parse", "HEAD"]).unwrap();

        let correction = correct_interview_metadata(
            &source,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        for artifact in &correction.correction.added_artifacts {
            fs::write(
                change_dir(&source, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(&source, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(&source, &record.id).unwrap();
        record = accept_change(&source, &record.id, Some("Reviewer".into()), None).unwrap();
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "accept corrected definition"]);

        let workspace = format!("{CHANGES_PATH}/{}", record.id);
        git(
            &source,
            &["restore", "--source", original.as_str(), "--", &workspace],
        );
        git(&source, &["add", "."]);
        git(&source, &["commit", "-m", "rollback lifecycle workspace"]);
        let source_url = format!("file://{}", source.display());
        git(
            temp.path(),
            &[
                "clone",
                "--depth",
                "1",
                source_url.as_str(),
                clone.to_str().unwrap(),
            ],
        );
        let rolled_back = load_change(&clone, &record.id).unwrap();
        assert_eq!(rolled_back.correction_count, 0);
        let error = effective_change_definition(&clone, &rolled_back).unwrap_err();
        assert!(error.contains("incomplete shallow Git checkout"), "{error}");

        let new_record = create_change(
            &clone,
            CreateChangeRequest {
                description: "Add a local shallow-checkout note".into(),
                kind: ChangeKind::Documentation,
                affected_specs: Vec::new(),
                affected_paths: vec!["README.md".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Local documentation only".into()),
            },
        )
        .unwrap();
        assert!(effective_change_definition(&clone, &new_record).is_ok());
    }

    // Verifies REQ-change-032.
    #[test]
    fn accepted_snapshot_with_a_stale_contract_is_not_an_anchor() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        git(&["add", "."]);
        git(&["commit", "-m", "base definition"]);
        record = accept_completed_record(root, record);
        git(&["add", "."]);
        git(&["commit", "-m", "accept original definition"]);
        let original = git_output(root, &["rev-parse", "HEAD"]).unwrap();

        let correction = correct_interview_metadata(
            root,
            &record.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Reviewer".into(),
            "The accepted verification path has architectural risk".into(),
        )
        .unwrap();
        for artifact in &correction.correction.added_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Review the corrected classification.\n"
                } else {
                    "# Complete\n\nThe corrected classification was reviewed.\n"
                },
            )
            .unwrap();
        }
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        fs::write(
            change_dir(root, &record.id).join("testing.md"),
            "# Complete\n\nChanged after the accepted contract was approved.\n",
        )
        .unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "record stale accepted contract snapshot"]);

        let workspace = format!("{CHANGES_PATH}/{}", record.id);
        git(&["restore", "--source", original.as_str(), "--", &workspace]);
        let rolled_back = load_change(root, &record.id).unwrap();
        assert_eq!(rolled_back.correction_count, 0);
        assert!(effective_change_definition(root, &rolled_back).is_ok());
    }

    #[test]
    fn correction_ledgers_fail_closed_and_hash_portably() {
        let first = TempDir::new().unwrap();
        let root = first.path();
        write_default_policy(root, Vec::new()).unwrap();
        let accepted = accept_completed_record(root, completed_no_spec_record(root));
        let result = correct_interview_metadata(
            root,
            &accepted.id,
            CorrectionField::ArchitectureRisk,
            "yes".into(),
            "Release reviewer".into(),
            "Architecture classification was stale".into(),
        )
        .unwrap();
        let record = result.change;
        let ledger_path = change_dir(root, &record.id).join(CORRECTIONS_FILE);
        let valid_ledger = fs::read_to_string(&ledger_path).unwrap();

        fs::remove_file(&ledger_path).unwrap();
        let error = effective_change_definition(root, &record).unwrap_err();
        assert!(
            error.contains("does not match state.json correction_count"),
            "{error}"
        );
        fs::write(&ledger_path, &valid_ledger).unwrap();

        let mut tampered: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
        tampered["corrections"][0]["sequence"] = serde_json::json!(2);
        write_json(&ledger_path, &tampered).unwrap();
        let error = effective_change_definition(root, &record).unwrap_err();
        assert!(error.contains("sequence is not contiguous"), "{error}");
        fs::write(&ledger_path, &valid_ledger).unwrap();

        let mut unsupported: serde_json::Value = serde_json::from_str(&valid_ledger).unwrap();
        unsupported["corrections"][0]["field"] = serde_json::json!("acceptance_criteria");
        write_json(&ledger_path, &unsupported).unwrap();
        let error = effective_change_definition(root, &record).unwrap_err();
        assert!(error.contains("invalid correction ledger"), "{error}");
        fs::write(&ledger_path, &valid_ledger).unwrap();

        let mut tampered_definition: serde_json::Value =
            serde_json::from_str(&valid_ledger).unwrap();
        tampered_definition["corrections"][0]["superseded_definition_approval"]["digest"] =
            serde_json::json!("forged-definition-digest");
        write_json(&ledger_path, &tampered_definition).unwrap();
        let error = effective_change_definition(root, &record).unwrap_err();
        assert!(error.contains("invalid prior gate evidence"), "{error}");
        fs::write(&ledger_path, &valid_ledger).unwrap();

        let second = TempDir::new().unwrap();
        let second_root = second.path();
        let second_dir = change_dir(second_root, &record.id);
        fs::create_dir_all(second_dir.join("deltas")).unwrap();
        save_change(second_root, &record).unwrap();
        fs::write(second_dir.join(CORRECTIONS_FILE), valid_ledger).unwrap();
        let effective = effective_change_definition(root, &record).unwrap();
        for artifact in &effective.selected_artifacts {
            let content =
                fs::read(change_dir(root, &record.id).join(artifact.file_name())).unwrap();
            fs::write(second_dir.join(artifact.file_name()), content).unwrap();
        }
        assert_eq!(
            definition_digest(root, &record).unwrap(),
            definition_digest(second_root, &record).unwrap()
        );
    }

    #[test]
    fn acceptance_rechecks_late_dependency_state() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        let dependency = create_change(
            root,
            CreateChangeRequest {
                description: "unfinished prerequisite".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Ordering fixture".into()),
            },
        )
        .unwrap();
        record = load_change(root, &record.id).unwrap();
        record.dependencies.push(dependency.id.clone());
        save_change(root, &record).unwrap();
        append_approval(
            root,
            &record,
            "definition",
            Some("Reviewer".into()),
            definition_digest(root, &record).unwrap(),
            Some("Approved late ordering change".into()),
        )
        .unwrap();
        let mut evidence = load_verification(root, &record).unwrap();
        evidence.contract_digest = definition_digest(root, &record).unwrap();
        evidence.workspace_digest = project_input_digest(root).unwrap();
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &evidence,
        )
        .unwrap();
        let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
        assert!(error.contains("must be accepted"), "{error}");
    }

    #[test]
    fn failed_evidence_keeps_local_check_red() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        write_default_policy(root, Vec::new()).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        let mut evidence = verify_change(root, &record.id).unwrap();
        evidence.passed = false;
        write_json(
            &change_dir(root, &record.id).join("verification.json"),
            &evidence,
        )
        .unwrap();
        let report = check_project(root);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("latest verification evidence failed"))
        );
    }

    #[test]
    fn no_spec_change_with_module_scope_needs_no_delta() {
        let temp = TempDir::new().unwrap();
        let record = completed_no_spec_record(temp.path());
        assert!(
            collect_requirement_ids(temp.path(), &record)
                .unwrap()
                .is_empty()
        );
        assert!(validate_effective_contracts(temp.path(), &[record]).is_ok());
    }

    #[test]
    fn reopened_canonical_change_validates_current_canonical_contract() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let delta = "## ADDED\n\n### SPEC SECTION Invariants\n\nDuplicate invariant.\n";
        let mut record = completed_section_only_record(root, delta);
        record.state = ChangeState::Verifying;
        record.canonical_applied = true;
        save_change(root, &record).unwrap();

        let canonical = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
        let item = parse_delta(delta).unwrap().remove(0);
        assert!(
            apply_markdown_block(&canonical, "## ", &item.key, &item.content, item.operation,)
                .is_err(),
            "the fixture must fail if an already-applied delta is replayed"
        );
        assert!(validate_effective_contracts(root, &[record.clone()]).is_ok());

        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "# Invalid current contract\n",
        )
        .unwrap();

        let errors = validate_effective_contracts(root, &[record]).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("effective contract `auth`")),
            "{errors:?}"
        );
    }

    #[test]
    fn unsafe_verification_commands_are_refused() {
        assert!(shell_words("cargo test; rm -rf .").is_err());
        assert!(shell_words("cargo test | tee out").is_err());
        assert!(shell_words("cargo test '").is_err());
        assert_eq!(
            shell_words("fledge run test").unwrap(),
            vec!["fledge", "run", "test"]
        );
        assert_eq!(
            shell_words("cargo run --manifest-path 'tools/spec sync/Cargo.toml' -- check # safe")
                .unwrap(),
            vec![
                "cargo",
                "run",
                "--manifest-path",
                "tools/spec sync/Cargo.toml",
                "--",
                "check"
            ]
        );
    }

    #[test]
    fn verification_detection_prefers_portable_project_commands() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        fs::write(temp.path().join("fledge.toml"), "[tasks.test]\n").unwrap();
        assert_eq!(detect_verification_commands(temp.path()), ["cargo test"]);
    }

    #[test]
    fn portable_paths_normalize_windows_separators() {
        assert_eq!(
            portable_project_path(Path::new(""), Path::new(r"openspec\changes\add-passkeys")),
            "openspec/changes/add-passkeys"
        );
    }

    #[test]
    fn definition_digest_is_portable_across_checkout_roots() {
        let first = TempDir::new().unwrap();
        let second = TempDir::new().unwrap();
        let record = completed_record(first.path());
        fs::create_dir_all(change_dir(second.path(), &record.id).join("deltas")).unwrap();
        save_change(second.path(), &record).unwrap();
        for artifact in &record.selected_artifacts {
            let content = format!("# {}\n\nComplete.\n", artifact.file_name());
            fs::write(
                change_dir(first.path(), &record.id).join(artifact.file_name()),
                &content,
            )
            .unwrap();
            fs::write(
                change_dir(second.path(), &record.id).join(artifact.file_name()),
                content,
            )
            .unwrap();
        }
        let delta = "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n";
        fs::write(delta_path(first.path(), &record, "auth"), delta).unwrap();
        fs::write(delta_path(second.path(), &record, "auth"), delta).unwrap();
        assert_eq!(
            definition_digest(first.path(), &record).unwrap(),
            definition_digest(second.path(), &record).unwrap()
        );
    }

    #[test]
    fn optional_companions_are_selected_by_policy() {
        let feature = adaptive_artifacts(
            ChangeKind::Feature,
            &["auth".into()],
            &["src/auth.rs".into()],
        );
        assert!(feature.contains(&ArtifactKind::Requirements));
        assert!(feature.contains(&ArtifactKind::Testing));
        let docs = adaptive_artifacts(ChangeKind::Documentation, &[], &["README.md".into()]);
        assert!(docs.contains(&ArtifactKind::Docs));
        assert!(!docs.contains(&ArtifactKind::Requirements));
    }

    #[test]
    fn full_lifecycle_applies_contract_and_archives() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(root.join("src/auth.rs"), "// Authentication module.\n").unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1.0.0\nstatus: stable\nfiles:\n  - src/auth.rs\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        fs::write(
            root.join("specs/auth/requirements.md"),
            "---\nspec: auth.spec.md\n---\n\n# Requirements\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(
            root.join("tests/auth.rs"),
            "// Verifies REQ-auth-001\n#[test]\nfn passkey_authentication() {}\n",
        )
        .unwrap();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        let mut record = completed_record(root);
        for artifact in &record.selected_artifacts {
            let content = if *artifact == ArtifactKind::Tasks {
                "# Tasks\n\n- [x] Implement passkeys\n"
            } else {
                "# Complete\n\nReviewed content.\n"
            };
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                content,
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &record, "auth"),
            "# Auth delta\n\n## ADDED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL support passkey authentication.\n\nAcceptance Criteria\n- A registered passkey authenticates the user.\n\n## MODIFIED\n\n### SPEC SECTION Invariants\n\n1. Passkey authentication is supported and traced to `REQ-auth-001`.\n",
        )
        .unwrap();
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        let mut verification = verify_change(root, &record.id).unwrap();
        assert!(verification.passed);
        fs::write(
            change_dir(root, &record.id).join("context.md"),
            "# Complete\n\nUpdated during verification.\n",
        )
        .unwrap();
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Implementing);
        verification = verify_change(root, &record.id).unwrap();
        assert!(verification.passed);
        let tombstone = root.join(".specsync/archive/changes/old/deltas");
        fs::create_dir_all(&tombstone).unwrap();
        fs::write(
            tombstone.join("auth.md"),
            "## REMOVED\n### REQUIREMENT REQ-auth-001\nRetired.\n",
        )
        .unwrap();
        let error = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
        assert!(error.contains("permanent tombstone"), "{error}");
        fs::remove_dir_all(root.join(".specsync/archive/changes/old")).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Accepted);
        let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
        assert!(spec.contains("version: 1.0.1"));
        assert!(spec.contains("REQ-auth-001"));
        assert!(spec.contains(&record.id));
        assert!(spec.contains(&format!("| 2026-01-01 | Initial |\n| {} |", today())));
        let requirements = fs::read_to_string(root.join("specs/auth/requirements.md")).unwrap();
        assert!(requirements.contains("### REQ-auth-001"));
        git(&["add", "."]);
        git(&["commit", "-m", "record accepted lifecycle evidence"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        let archived = archive_change(root, &record.id).unwrap();
        assert!(archived.is_dir());
        assert!(!change_dir(root, &record.id).exists());
    }

    #[test]
    fn section_only_semantic_delta_can_satisfy_acceptance_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and explicitly documented.\n",
        );
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();

        let verification = verify_change(root, &record.id).unwrap();

        assert!(verification.passed);
        assert!(verification.requirement_ids.is_empty());
    }

    #[test]
    fn missing_semantic_acceptance_evidence_is_not_reported_as_command_failure() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_section_only_record(
            root,
            "## REMOVED\n### SPEC SECTION Legacy Notes\n\nRetired.\n",
        );
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();

        let error = verify_change(root, &record.id).unwrap_err();

        assert!(
            error.contains("semantic acceptance evidence is missing"),
            "{error}"
        );
        assert!(
            !error.contains("configured verification command failed"),
            "{error}"
        );
    }

    #[test]
    fn direct_recursive_verification_command_is_rejected_before_execution() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut policy = SddPolicy::default();
        policy.require_change_for_meaningful_files = false;
        policy.verification_commands = vec!["specsync check --strict".into()];
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let mut record = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
        );
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();

        let error = verify_change(root, &record.id).unwrap_err();

        assert!(error.contains("recursive lifecycle verification command"));
        assert!(error.contains("specsync check --strict"));
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Implementing
        );
        assert!(
            !change_dir(root, &record.id)
                .join("verification.json")
                .exists()
        );
        assert!(
            !change_dir(root, &record.id)
                .join("verification-attempts.json")
                .exists()
        );
    }

    // Verifies REQ-change-030.
    #[test]
    fn native_cargo_check_argument_is_not_misclassified_as_specsync() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"native-cli\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();

        assert!(reject_direct_lifecycle_verification(root, "cargo run -- check").is_ok());
        assert!(
            reject_direct_lifecycle_verification(root, "cargo run --bin specsync -- check")
                .is_err()
        );
        assert!(
            reject_direct_lifecycle_verification(root, "cargo run -p specsync -- check").is_err()
        );
        assert!(
            reject_direct_lifecycle_verification(root, "cargo run --package specsync -- check")
                .is_err()
        );
        assert!(reject_direct_lifecycle_verification(root, "specsync --strict").is_err());

        fs::write(
            root.join("Cargo.toml"),
            "[ package ]\nname = 'specsync' # lifecycle CLI\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert!(reject_direct_lifecycle_verification(root, "cargo run -- check").is_err());
        assert_eq!(
            cargo_package_value(
                "[ package ]\nname = 'specsync' # lifecycle CLI\n[dependencies]\nname = \"ignored\"\n",
                "name"
            ),
            Some("specsync".into())
        );
    }

    // Verifies REQ-change-030.
    #[test]
    fn cargo_manifest_path_detects_recursive_specsync_before_state_mutation() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("tools/specsync")).unwrap();
        fs::write(
            root.join("tools/specsync/Cargo.toml"),
            "[package]\nname = \"specsync\"\nversion = \"5.0.2\"\n",
        )
        .unwrap();

        for command in [
            "cargo run --manifest-path tools/specsync/Cargo.toml -- check",
            "cargo run --manifest-path=tools/specsync/Cargo.toml -- check",
            "cargo --manifest-path tools/specsync/Cargo.toml run -- check",
        ] {
            assert!(reject_direct_lifecycle_verification(root, command).is_err());
        }
        fs::create_dir_all(root.join("tools/spec sync")).unwrap();
        fs::write(
            root.join("tools/spec sync/Cargo.toml"),
            "[package]\nname = \"specsync\" # nested CLI\nversion = \"5.0.2\"\n",
        )
        .unwrap();
        assert!(
            reject_direct_lifecycle_verification(
                root,
                "cargo run --manifest-path 'tools/spec sync/Cargo.toml' -- check # lifecycle"
            )
            .is_err()
        );
        fs::create_dir_all(root.join("tools/default-run")).unwrap();
        fs::write(
            root.join("tools/default-run/Cargo.toml"),
            "[package]\nname = \"wrapper\"\ndefault-run = \"specsync\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert!(
            reject_direct_lifecycle_verification(
                root,
                "cargo run --manifest-path=tools/default-run/Cargo.toml -- check"
            )
            .is_err()
        );

        fs::create_dir_all(root.join("tools/native")).unwrap();
        fs::write(
            root.join("tools/native/Cargo.toml"),
            "[package]\nname = \"native-cli\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert!(
            reject_direct_lifecycle_verification(
                root,
                "cargo run --manifest-path tools/native/Cargo.toml -- check"
            )
            .is_ok()
        );
        assert!(
            reject_direct_lifecycle_verification(
                root,
                "cargo run --manifest-path ../outside/Cargo.toml -- check"
            )
            .is_err()
        );

        let mut policy = SddPolicy::default();
        policy.require_change_for_meaningful_files = false;
        policy.verification_commands =
            vec!["cargo run --manifest-path tools/specsync/Cargo.toml -- check".into()];
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let mut record = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
        );
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();

        let error = verify_change(root, &record.id).unwrap_err();

        assert!(error.contains("recursive lifecycle verification command"));
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Implementing
        );
        assert!(
            !change_dir(root, &record.id)
                .join("verification.json")
                .exists()
        );
        assert!(
            !change_dir(root, &record.id)
                .join("verification-attempts.json")
                .exists()
        );
    }

    // Verifies REQ-change-030.
    #[test]
    fn generated_sequence_scope_does_not_suppress_delivery_scope_question() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Describe real delivery scope".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: Vec::new(),
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();

        assert!(
            next_questions(&record)
                .iter()
                .any(|question| question.id == "affected_paths")
        );
        let answered = answer_question(root, &record.id, "affected_paths", "src/lib.rs").unwrap();
        assert!(answered.affected_paths.contains(&"src/lib.rs".into()));
        assert!(answered.affected_paths.contains(&SEQUENCE_PATH.into()));
    }

    // Verifies REQ-change-031.
    #[test]
    fn interview_preserves_prose_and_requires_explicit_multiple_criteria() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Preserve interview intent".into(),
                kind: ChangeKind::BugFix,
                affected_specs: Vec::new(),
                affected_paths: vec!["src/change.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("Interview-only fixture".into()),
            },
        )
        .unwrap();
        let prose = "Literal strings, spaced headers, and trailing comments\nremain one criterion.";

        let answered = answer_question(root, &record.id, "acceptance_criteria", prose).unwrap();

        assert_eq!(answered.acceptance_criteria, [prose]);
        let persisted = load_change(root, &record.id).unwrap();
        assert_eq!(persisted.acceptance_criteria, [prose]);
        let markdown = fs::read_to_string(change_dir(root, &record.id).join("change.md")).unwrap();
        assert!(markdown.contains(&format!("- {prose}")));

        let answered = answer_question(
            root,
            &record.id,
            "acceptance_criteria",
            r#"["First, exactly", "Second criterion"]"#,
        )
        .unwrap();
        assert_eq!(
            answered.acceptance_criteria,
            ["First, exactly", "Second criterion"]
        );

        let answered =
            answer_question(root, &record.id, "affected_specs", "change, registry\ncli").unwrap();
        assert_eq!(answered.affected_specs, ["change", "registry", "cli"]);

        let answered = answer_question(
            root,
            &record.id,
            "affected_paths",
            "src/change.rs,\ntests/integration/change.rs",
        )
        .unwrap();
        assert!(answered.affected_paths.contains(&"src/change.rs".into()));
        assert!(
            answered
                .affected_paths
                .contains(&"tests/integration/change.rs".into())
        );
        assert!(answered.affected_paths.contains(&SEQUENCE_PATH.into()));
    }

    // Verifies REQ-change-030.
    #[test]
    fn disabled_policy_skips_sequence_validation() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut policy = SddPolicy::default();
        policy.enabled = false;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        fs::write(root.join(SEQUENCE_PATH), "not valid json\n").unwrap();

        let report = check_project(root);

        assert!(!report.enabled);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn failed_native_verification_is_retryable_with_append_only_history() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut policy = SddPolicy::default();
        policy.require_change_for_meaningful_files = false;
        policy.verification_commands =
            vec!["cargo metadata --manifest-path definitely-missing/Cargo.toml".into()];
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let mut record = completed_section_only_record(
            root,
            "## MODIFIED\n### SPEC SECTION Invariants\n\nStable and reviewed.\n",
        );
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();

        let first_error = verify_change(root, &record.id).unwrap_err();
        assert!(first_error.contains("configured verification command failed"));
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Verifying
        );

        policy.verification_commands.clear();
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let successful = verify_change(root, &record.id).unwrap();
        assert!(successful.passed);
        let history: VerificationAttemptLedger = serde_json::from_slice(
            &fs::read(change_dir(root, &record.id).join("verification-attempts.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(history.schema_version, 1);
        assert_eq!(history.attempts.len(), 2);
        assert!(!history.attempts[0].passed);
        assert!(history.attempts[1].passed);
        assert!(load_verification(root, &record).unwrap().passed);
    }

    #[test]
    fn overlapping_active_deltas_are_blocked() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut first = completed_record(root);
        let mut second = create_change(
            root,
            CreateChangeRequest {
                description: "add recovery passkeys".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        second.acceptance_criteria = vec!["Recovery works".into()];
        second
            .answers
            .insert("public_contract".into(), "yes".into());
        second
            .answers
            .insert("architecture_risk".into(), "no".into());
        for record in [&mut first, &mut second] {
            record.state = ChangeState::Approved;
            save_change(root, record).unwrap();
            fs::write(
                delta_path(root, record, "auth"),
                "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
            )
            .unwrap();
        }
        let error = ensure_no_delta_conflicts(root, &first).unwrap_err();
        assert!(error.contains(&second.id));
        first.dependencies.push(second.id.clone());
        assert!(ensure_no_delta_conflicts(root, &first).is_ok());
    }

    #[test]
    fn unified_gate_validates_code_against_effective_delta() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(root.join("src/auth.rs"), "pub fn login() {}\n").unwrap();
        fs::write(root.join("tests/auth.rs"), "// REQ-auth-001\n").unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles:\n  - src/auth.rs\ndb_tables: []\ndepends_on: []\n---\n\n# Auth\n\n## Purpose\n\nAuthentication.\n\n## Public API\n\n| Function | Description |\n|----------|-------------|\n| `login` | Login |\n\n## Invariants\n\n1. Stable.\n\n## Behavioral Examples\n\n### Scenario: Login\n\n- **Given** a user\n- **When** login runs\n- **Then** it succeeds\n\n## Error Cases\n\n| Condition | Behavior |\n|-----------|----------|\n| Invalid | Error |\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        let mut record = completed_record(root);
        for artifact in &record.selected_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                if *artifact == ArtifactKind::Tasks {
                    "# Tasks\n\n- [x] Done\n"
                } else {
                    "# Complete\n\nReviewed.\n"
                },
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL support secure login.\n\nAcceptance Criteria\n- Login is tested.\n\n## MODIFIED\n### SPEC SECTION Public API\n| Function | Description |\n|----------|-------------|\n| `login` | Login |\n| `phantom` | Missing implementation |\n",
        )
        .unwrap();
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Approved);
        let mut policy = SddPolicy::default();
        policy.require_change_for_meaningful_files = false;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let report = check_project(root);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("phantom") && error.contains("effective contract")),
            "expected effective-contract phantom error, got {:?}",
            report.errors
        );
        record = start_implementation(root, &record.id).unwrap();
        let error = verify_change(root, &record.id).unwrap_err();
        assert!(error.contains("phantom") && error.contains("effective contract"));
    }

    #[test]
    fn openspec_adoption_imports_canonical_and_active_but_not_archive() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
        fs::create_dir_all(root.join("openspec/changes/add-passkeys")).unwrap();
        fs::create_dir_all(root.join("openspec/changes/archive/old-change")).unwrap();
        fs::write(root.join("openspec/specs/auth/spec.md"), "# Auth\n").unwrap();
        fs::write(
            root.join("openspec/changes/add-passkeys/proposal.md"),
            "# Add passkeys\n",
        )
        .unwrap();
        fs::write(
            root.join("openspec/changes/archive/old-change/proposal.md"),
            "# Old\n",
        )
        .unwrap();
        let actions = adopt(root, false, Some("openspec")).unwrap();
        assert!(actions.iter().any(|action| action.contains("openspec")));
        assert!(
            root.join(".specsync/imports/openspec/canonical/auth/spec.md")
                .is_file()
        );
        let records = list_changes(root);
        assert_eq!(records.len(), 1);
        assert!(
            change_dir(root, &records[0].id)
                .join("imported/proposal.md")
                .is_file()
        );
    }

    #[test]
    fn speckit_adoption_imports_constitution_and_feature_workspaces_only() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specify/memory")).unwrap();
        fs::create_dir_all(root.join("specs/001-passkeys")).unwrap();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(
            root.join(".specify/memory/constitution.md"),
            "# Constitution\n",
        )
        .unwrap();
        fs::write(root.join("specs/001-passkeys/spec.md"), "# Passkeys\n").unwrap();
        fs::write(root.join("specs/auth/auth.spec.md"), "# Native spec\n").unwrap();
        fs::write(root.join("specs/auth/tasks.md"), "# Native tasks\n").unwrap();
        adopt(root, false, Some("speckit")).unwrap();
        assert!(
            root.join(".specsync/imports/speckit/constitution.md")
                .is_file()
        );
        let records = list_changes(root);
        assert_eq!(records.len(), 1);
        assert!(
            change_dir(root, &records[0].id)
                .join("imported/spec.md")
                .is_file()
        );
    }

    #[test]
    fn custom_artifact_templates_are_scoped_and_rendered() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("templates")).unwrap();
        fs::write(
            root.join("templates/risk.md"),
            "# Risk for {{title}}\n\nChange: {{change_id}}\n",
        )
        .unwrap();
        let mut policy = SddPolicy::default();
        policy
            .custom_artifacts
            .insert("risk".into(), "templates/risk.md".into());
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Assess authentication risk".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: vec![ArtifactKind::parse("../../risk")],
                no_spec_change: true,
                rationale: Some("Risk assessment only".into()),
            },
        )
        .unwrap();
        let rendered = fs::read_to_string(change_dir(root, &record.id).join("risk.md")).unwrap();
        assert!(rendered.contains(&record.id));
        assert!(rendered.contains(&record.title));
        assert!(safe_project_path(root, "../../secret").is_err());
    }

    #[test]
    fn project_principles_are_part_of_the_approval_digest() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::write(
            root.join("PRINCIPLES.md"),
            "# Principles\n\nPrefer safety.\n",
        )
        .unwrap();
        let mut policy = SddPolicy::default();
        policy.principles_file = Some("PRINCIPLES.md".into());
        policy.require_change_for_meaningful_files = false;
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let mut record = completed_record(root);
        for artifact in &record.selected_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                "# Complete\n\nReviewed.\n",
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
        )
        .unwrap();
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert!(ensure_definition_approval_valid(root, &record).is_ok());
        fs::write(
            root.join("PRINCIPLES.md"),
            "# Principles\n\nPrefer speed.\n",
        )
        .unwrap();
        assert!(ensure_definition_approval_valid(root, &record).is_err());
    }

    #[test]
    fn definition_approval_rejects_an_invalid_semantic_delta() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_record(root);
        for artifact in &record.selected_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                "# Complete\n\nReviewed.\n",
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nMissing normative language.\n",
        )
        .unwrap();

        let error =
            approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
        assert!(error.contains("SHALL") || error.contains("Acceptance Criteria"));
        assert!(load_approvals(root, &record).unwrap().approvals.is_empty());
    }

    #[test]
    fn prepared_write_failure_rolls_back_prior_files() {
        let temp = TempDir::new().unwrap();
        let first = temp.path().join("first.md");
        let second = temp.path().join("second.md");
        fs::write(&first, "original\n").unwrap();
        fs::create_dir(&second).unwrap();
        let result = write_prepared_files(
            temp.path(),
            &[
                (first.clone(), "changed\n".into()),
                (second, "cannot write a directory\n".into()),
            ],
        );
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(first).unwrap(), "original\n");
    }

    #[test]
    fn pending_transaction_is_recovered_before_next_lifecycle_write() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        let canonical = root.join("specs/auth/auth.spec.md");
        fs::write(&canonical, "corrupted partial write\n").unwrap();
        write_json(
            &root.join(TRANSACTION_PATH),
            &[TransactionEntry {
                path: "specs/auth/auth.spec.md".into(),
                original: Some("original canonical content\n".into()),
            }],
        )
        .unwrap();
        let lock = acquire_project_lock(root).unwrap();
        drop(lock);
        assert_eq!(
            fs::read_to_string(canonical).unwrap(),
            "original canonical content\n"
        );
        assert!(!root.join(TRANSACTION_PATH).exists());
    }

    #[test]
    fn change_dependencies_reject_cycles() {
        let temp = TempDir::new().unwrap();
        let request = |description: &str| CreateChangeRequest {
            description: description.into(),
            kind: ChangeKind::Operations,
            affected_specs: Vec::new(),
            affected_paths: vec!["ci/".into()],
            requested_artifacts: Vec::new(),
            no_spec_change: true,
            rationale: Some("Operational ordering".into()),
        };
        let first = create_change(temp.path(), request("First operation")).unwrap();
        let second = create_change(temp.path(), request("Second operation")).unwrap();
        add_dependency(temp.path(), &first.id, &second.id).unwrap();
        let error = add_dependency(temp.path(), &second.id, &first.id).unwrap_err();
        assert!(error.contains("cycle"));
    }

    #[test]
    fn removed_requirement_ids_cannot_be_reused() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync/archive/changes/old/deltas")).unwrap();
        fs::write(
            root.join(".specsync/archive/changes/old/deltas/auth.md"),
            "## REMOVED\n### REQUIREMENT REQ-auth-007\nRetired requirement.\n",
        )
        .unwrap();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Reuse retired requirement".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-007\nThe system SHALL reuse an ID.\n\nAcceptance Criteria\n- Reused.\n",
        )
        .unwrap();
        let error = validate_delta_files(root, &record).unwrap_err();
        assert!(error.contains("permanent tombstone"));
    }

    #[test]
    fn draft_requirement_removals_are_not_permanent_tombstones() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let removal = create_change(
            root,
            CreateChangeRequest {
                description: "Consider retiring requirement".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            delta_path(root, &removal, "auth"),
            "## REMOVED\n### REQUIREMENT REQ-auth-007\nRetired requirement.\n",
        )
        .unwrap();
        let addition = create_change(
            root,
            CreateChangeRequest {
                description: "Add active requirement".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            delta_path(root, &addition, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-007\nThe system SHALL add an active requirement.\n\nAcceptance Criteria\n- Active.\n",
        )
        .unwrap();

        assert!(validate_delta_files(root, &addition).is_ok());
    }

    #[test]
    fn requirement_ids_must_match_their_delta_module() {
        let temp = TempDir::new().unwrap();
        let record = completed_record(temp.path());
        fs::write(
            delta_path(temp.path(), &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-billing-001\nThe system SHALL authenticate.\n\nAcceptance Criteria\n- Works.\n",
        )
        .unwrap();
        let error = validate_delta_files(temp.path(), &record).unwrap_err();
        assert!(error.contains("must match affected module `auth`"));
    }

    #[test]
    fn change_identifiers_and_scope_cannot_escape_project_root() {
        let temp = TempDir::new().unwrap();
        assert!(load_change(temp.path(), "../../Cargo.toml").is_err());
        let result = create_change(
            temp.path(),
            CreateChangeRequest {
                description: "Escape scope".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["../outside".into()],
                affected_paths: vec!["../../secret".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        );
        assert!(result.is_err());
        assert!(!temp.path().join("../outside").exists());
    }

    #[test]
    fn windows_paths_are_normalized_without_allowing_traversal() {
        assert_eq!(
            normalize_project_path(r"src\auth\mod.rs").unwrap(),
            "src/auth/mod.rs"
        );
        assert!(normalize_project_path(r"..\secret.txt").is_err());
        assert!(normalize_project_path(r"C:\secret.txt").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn safe_project_paths_reject_symlink_escapes() {
        let root = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked")).unwrap();
        let error = safe_project_path(root.path(), "linked/secret.md").unwrap_err();
        assert!(error.contains("through a symlink"));
    }

    #[test]
    fn path_scopes_match_component_boundaries() {
        assert!(path_matches_scope("src", "src"));
        assert!(path_matches_scope("src/auth.rs", "src"));
        assert!(path_matches_scope("src/auth.rs", "src/"));
        assert!(!path_matches_scope("src-old/auth.rs", "src"));
        assert!(!path_matches_scope("src2.rs", "src"));
        assert!(!path_matches_scope("Src/auth.rs", "src"));
    }

    #[test]
    fn default_policy_covers_root_action_and_dependency_lockfiles() {
        let policy = SddPolicy::default();
        for path in [
            "action.yml",
            "Cargo.lock",
            "bun.lock",
            "package-lock.json",
            "Package.resolved",
            "go.sum",
            "uv.lock",
        ] {
            assert!(
                path_is_meaningful(path, &policy),
                "{path} should be meaningful"
            );
        }
        let mut hostile = policy;
        hostile.ignored_paths.push(".specsync/".into());
        assert!(path_is_meaningful(".specsync/sdd.json", &hostile));
        assert!(path_is_meaningful(".specsync/config.toml", &hostile));
        assert!(path_is_meaningful(".specsync/registry.toml", &hostile));
        assert!(path_is_meaningful("specsync-registry.toml", &hostile));
        assert!(path_is_meaningful(SEQUENCE_PATH, &hostile));
        assert!(!path_is_meaningful(
            ".specsync/adoption-report.json",
            &hostile
        ));
        assert!(path_matches_scope("root.rs", "."));
    }

    #[test]
    fn workspace_digest_tracks_unicode_and_space_paths() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src/space dir")).unwrap();
        let path = root.join("src/space dir/naïve.rs");
        fs::write(&path, "first\n").unwrap();
        let first = project_input_digest(root).unwrap();
        fs::write(path, "second\n").unwrap();
        let second = project_input_digest(root).unwrap();
        assert_ne!(first, second);
    }

    // Verifies REQ-change-029.
    #[test]
    fn valid_later_sequence_claim_preserves_historical_acceptance_input() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Implementing;
        record.affected_paths = vec![".specsync".into()];
        save_change(root, &record).unwrap();
        let first_workspace = project_input_digest(root).unwrap();
        let first_acceptance = acceptance_input_digest(root, &record, &[]).unwrap();

        let successor = create_change(
            root,
            CreateChangeRequest {
                description: "Later sequence owner".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/later".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();
        let second_workspace = project_input_digest(root).unwrap();
        let second_acceptance = acceptance_input_digest(root, &record, &[]).unwrap();

        assert!(change_sequence(&successor.id) > change_sequence(&record.id));
        assert_ne!(first_workspace, second_workspace);
        assert_eq!(first_acceptance, second_acceptance);
        assert!(!project_input_is_volatile(SEQUENCE_PATH));
    }

    // Verifies REQ-change-029.
    #[test]
    fn later_collision_acknowledgements_do_not_stale_earlier_sequence_evidence() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut predecessor = completed_no_spec_record(root);
        predecessor.state = ChangeState::Implementing;
        predecessor.affected_paths = vec![SEQUENCE_PATH.into()];
        save_change(root, &predecessor).unwrap();
        let before = acceptance_input_digest(root, &predecessor, &[]).unwrap();

        let mut successor = create_change(
            root,
            CreateChangeRequest {
                description: "Later sequence owner".into(),
                kind: ChangeKind::Operations,
                affected_specs: Vec::new(),
                affected_paths: vec!["ops/later".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: true,
                rationale: Some("fixture".into()),
            },
        )
        .unwrap();
        successor.state = ChangeState::Accepted;
        save_change(root, &successor).unwrap();
        let mut duplicate = successor.clone();
        duplicate.id = "CHG-0002-archived-collision".into();
        duplicate.slug = "archived-collision".into();
        duplicate.state = ChangeState::Archived;
        let archived_dir = root
            .join(ARCHIVE_PATH)
            .join("2026-07-14-CHG-0002-archived-collision");
        fs::create_dir_all(&archived_dir).unwrap();
        write_json(&archived_dir.join("state.json"), &duplicate).unwrap();
        let mut ids = vec![successor.id.clone(), duplicate.id.clone()];
        ids.sort();
        write_json(
            &root.join(SEQUENCE_PATH),
            &ChangeSequenceLedger {
                schema_version: 1,
                sequence: 2,
                id: successor.id,
                acknowledged_collisions: vec![ChangeSequenceCollision { sequence: 2, ids }],
            },
        )
        .unwrap();

        assert!(validate_change_sequences(root).is_ok());
        assert_eq!(
            before,
            acceptance_input_digest(root, &predecessor, &[]).unwrap()
        );
    }

    #[test]
    fn current_sequence_owner_binds_exact_ledger_content() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Implementing;
        record.affected_paths = vec![".specsync".into()];
        save_change(root, &record).unwrap();
        let canonical = acceptance_input_digest(root, &record, &[]).unwrap();
        let ledger = load_change_sequence_ledger(root).unwrap().unwrap();
        fs::write(
            root.join(SEQUENCE_PATH),
            serde_json::to_string(&ledger).unwrap(),
        )
        .unwrap();

        assert!(validate_change_sequences(root).is_ok());
        assert_ne!(
            canonical,
            acceptance_input_digest(root, &record, &[]).unwrap()
        );
    }

    #[test]
    fn invalid_later_sequence_claim_cannot_replace_historical_ledger_input() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Implementing;
        record.affected_paths = vec![".specsync".into()];
        save_change(root, &record).unwrap();
        write_json(
            &root.join(SEQUENCE_PATH),
            &ChangeSequenceLedger {
                schema_version: 1,
                sequence: 2,
                id: "CHG-0002-missing-owner".into(),
                acknowledged_collisions: Vec::new(),
            },
        )
        .unwrap();

        let error = acceptance_input_digest(root, &record, &[]).unwrap_err();
        assert!(error.contains("highest recorded sequence"));
    }

    #[test]
    fn framed_workspace_digest_resists_nul_entry_boundary_collisions() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a"), b"X\0b\0Y").unwrap();
        let one_file = project_input_digest(temp.path()).unwrap();

        fs::write(temp.path().join("a"), b"X").unwrap();
        fs::write(temp.path().join("b"), b"Y").unwrap();
        let two_files = project_input_digest(temp.path()).unwrap();

        assert_ne!(one_file, two_files);
    }

    #[test]
    fn framed_acceptance_digest_resists_nul_entry_boundary_collisions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Implementing;
        record.affected_paths = vec![".".into()];
        fs::write(root.join("a"), b"X\0b\0Y").unwrap();
        let one_file = acceptance_input_digest(root, &record, &[]).unwrap();

        fs::write(root.join("a"), b"X").unwrap();
        fs::write(root.join("b"), b"Y").unwrap();
        let two_files = acceptance_input_digest(root, &record, &[]).unwrap();

        assert_ne!(one_file, two_files);
    }

    #[test]
    fn workspace_digest_preserves_binary_bytes_and_line_endings() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("fixture.bin");
        fs::write(&path, b"first\r\nsecond\0\xff").unwrap();
        let binary = project_input_digest(temp.path()).unwrap();
        fs::write(&path, b"first\r\nsecond\0\xfe").unwrap();
        let changed_binary = project_input_digest(temp.path()).unwrap();
        assert_ne!(binary, changed_binary);

        fs::write(&path, b"first\nsecond\n").unwrap();
        let lf = project_input_digest(temp.path()).unwrap();
        fs::write(&path, b"first\r\nsecond\r\n").unwrap();
        let crlf = project_input_digest(temp.path()).unwrap();
        assert_ne!(lf, crlf, "line endings remain byte-exact digest inputs");
    }

    #[test]
    fn workspace_digest_includes_git_executable_mode() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "core.autocrlf", "false"]);
        fs::write(root.join("tool.sh"), b"#!/bin/sh\nexit 0\n").unwrap();
        git(&["add", "tool.sh"]);
        let regular = project_input_digest(root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(root.join("tool.sh")).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(root.join("tool.sh"), permissions).unwrap();
        }
        #[cfg(not(unix))]
        git(&["update-index", "--chmod=+x", "tool.sh"]);
        let executable = project_input_digest(root).unwrap();
        assert_ne!(regular, executable);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_digest_distinguishes_symlinks_files_and_targets() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::write(root.join("first"), b"same").unwrap();
        fs::write(root.join("second"), b"same").unwrap();
        symlink("first", root.join("entry")).unwrap();
        let first_target = project_input_digest(root).unwrap();
        fs::remove_file(root.join("entry")).unwrap();
        symlink("second", root.join("entry")).unwrap();
        let second_target = project_input_digest(root).unwrap();
        fs::remove_file(root.join("entry")).unwrap();
        fs::write(root.join("entry"), b"second").unwrap();
        let regular_file = project_input_digest(root).unwrap();

        assert_ne!(first_target, second_target);
        assert_ne!(second_target, regular_file);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn workspace_digest_rejects_non_utf8_paths_instead_of_lossy_aliasing() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let temp = TempDir::new().unwrap();
        let path = temp.path().join(OsString::from_vec(vec![b'f', 0xff]));
        fs::write(path, b"content").unwrap();
        let error = project_input_digest(temp.path()).unwrap_err();
        assert!(error.contains("non-UTF-8"), "{error}");
    }

    #[test]
    fn recorded_base_uses_oldest_change_order_not_hash_order() {
        let mut first = completed_record(TempDir::new().unwrap().path());
        first.base_commit = Some("ffffffff".into());
        let mut second = first.clone();
        second.id = "CHG-0002-second".into();
        second.base_commit = Some("00000000".into());
        assert_eq!(recorded_diff_base(&[first, second]), "ffffffff");
    }

    #[test]
    fn dependent_changes_are_topologically_ordered() {
        let temp = TempDir::new().unwrap();
        let mut dependent = completed_record(temp.path());
        dependent.id = "CHG-0001-dependent".into();
        dependent.dependencies = vec!["CHG-0002-prerequisite".into()];
        let mut prerequisite = dependent.clone();
        prerequisite.id = "CHG-0002-prerequisite".into();
        prerequisite.dependencies.clear();
        let ordered = dependency_ordered_changes(vec![&dependent, &prerequisite]).unwrap();
        assert_eq!(ordered[0].id, prerequisite.id);
        assert_eq!(ordered[1].id, dependent.id);
    }

    #[test]
    fn transitive_dependencies_order_overlapping_deltas() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut prerequisite = completed_record(root);
        let mut middle = completed_record(root);
        let mut dependent = completed_record(root);
        prerequisite.state = ChangeState::Implementing;
        middle.state = ChangeState::Implementing;
        dependent.state = ChangeState::Implementing;
        middle.dependencies = vec![prerequisite.id.clone()];
        dependent.dependencies = vec![middle.id.clone()];
        for (record, requirement) in [
            (&prerequisite, "REQ-auth-900"),
            (&middle, "REQ-auth-901"),
            (&dependent, "REQ-auth-900"),
        ] {
            save_change(root, record).unwrap();
            fs::write(
                delta_path(root, record, "auth"),
                format!("## ADDED\n### REQUIREMENT {requirement}\nThe system SHALL work.\n\nAcceptance Criteria\n- Works.\n"),
            )
            .unwrap();
        }
        assert!(ensure_no_delta_conflicts(root, &dependent).is_ok());
    }

    #[test]
    fn semantic_application_respects_custom_specs_directory() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/config.json"),
            r#"{"specsDir":"contracts","sourceDirs":["src"]}"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("contracts/auth")).unwrap();
        fs::write(
            root.join("contracts/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# Auth\n\n## Purpose\nAuth.\n\n## Public API\nNone.\n\n## Invariants\nStable.\n\n## Behavioral Examples\nWorks.\n\n## Error Cases\nNone.\n\n## Dependencies\nNone.\n\n## Change Log\nInitial.\n",
        )
        .unwrap();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Add auth requirement".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL authenticate.\n\nAcceptance Criteria\n- Works.\n",
        )
        .unwrap();
        let prepared = prepare_delta_application(root, &record).unwrap();
        assert!(
            prepared
                .iter()
                .all(|(path, _)| path.starts_with(root.join("contracts")))
        );
    }

    #[test]
    fn semantic_application_resolves_registry_backed_canonical_paths() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(root.join("specs/client")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/client.rs"), "// client API\n").unwrap();
        fs::write(
            root.join(".specsync/registry.toml"),
            "[registry]\nname = \"fixture\"\n\n[specs]\nclient-api = \"specs/client/client-api.spec.md\"\n",
        )
        .unwrap();
        fs::write(
            root.join("specs/client/client-api.spec.md"),
            "---\nmodule: client-api\nversion: 1\nstatus: stable\nfiles:\n  - src/client.rs\n---\n\n# Client API\n\n## Purpose\n\nClient API.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
        )
        .unwrap();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Update client API".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["client-api".into()],
                affected_paths: vec!["src/client.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            root.join("specs/client/requirements.md"),
            "# Requirements\n\nOriginal.\n",
        )
        .unwrap();
        for companion in ["tasks.md", "context.md", "testing.md", "design.md"] {
            fs::write(
                root.join("specs/client").join(companion),
                format!("# {companion}\n\nOriginal.\n"),
            )
            .unwrap();
        }
        fs::write(
            root.join("specs/client/other.spec.md"),
            "# Unrelated canonical spec\n",
        )
        .unwrap();
        let mut delivering = record.clone();
        delivering.state = ChangeState::Implementing;
        assert!(record_covers_project_path(
            root,
            &delivering,
            "specs/client/client-api.spec.md"
        ));
        assert!(record_covers_project_path(
            root,
            &delivering,
            "specs/client/requirements.md"
        ));
        for companion in ["tasks.md", "context.md", "testing.md", "design.md"] {
            assert!(record_covers_project_path(
                root,
                &delivering,
                &format!("specs/client/{companion}")
            ));
        }
        assert!(!record_covers_project_path(
            root,
            &delivering,
            "specs/client/unrelated.md"
        ));
        assert!(!record_covers_project_path(
            root,
            &delivering,
            "specs/client/other.spec.md"
        ));
        let first_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
        fs::write(
            root.join("specs/client/context.md"),
            "# Context\n\nUpdated.\n",
        )
        .unwrap();
        let companion_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
        assert_ne!(first_acceptance, companion_acceptance);
        fs::write(
            root.join("specs/client/unrelated.md"),
            "# Unrelated\n\nUpdated.\n",
        )
        .unwrap();
        let second_acceptance = acceptance_input_digest(root, &delivering, &[]).unwrap();
        assert_eq!(companion_acceptance, second_acceptance);
        fs::write(
            delta_path(root, &record, "client-api"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nRegistry-backed behavior is stable.\n",
        )
        .unwrap();

        let prepared = prepare_delta_application(root, &record).unwrap();

        assert!(
            prepared
                .iter()
                .any(|(path, _)| { path == &root.join("specs/client/client-api.spec.md") })
        );
        assert!(
            prepared
                .iter()
                .any(|(path, _)| path == &root.join("specs/client/requirements.md"))
        );
        assert!(
            !prepared
                .iter()
                .any(|(path, _)| path.starts_with(root.join("specs/client-api")))
        );

        let mut effective_record = record;
        effective_record.state = ChangeState::Approved;
        assert!(validate_effective_contracts(root, &[effective_record]).is_ok());
    }

    #[test]
    fn semantic_application_rejects_unsafe_registry_paths() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::write(
            root.join(".specsync/registry.toml"),
            "[registry]\nname = \"fixture\"\n\n[specs]\nauth = \"../../outside/auth.spec.md\"\n",
        )
        .unwrap();
        let record = create_change(
            root,
            CreateChangeRequest {
                description: "Update auth".into(),
                kind: ChangeKind::BugFix,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/auth.rs".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        fs::write(
            delta_path(root, &record, "auth"),
            "## MODIFIED\n### SPEC SECTION Invariants\n\nStable.\n",
        )
        .unwrap();

        let error = prepare_delta_application(root, &record).unwrap_err();

        assert!(error.contains("unsafe registry path"));
        assert!(error.contains("escapes the project root"));

        let mut effective_record = record;
        effective_record.state = ChangeState::Approved;
        let errors = validate_effective_contracts(root, &[effective_record]).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("cannot resolve canonical spec")
                    && error.contains("unsafe registry path"))
        );
    }

    #[test]
    fn path_coverage_uses_current_remote_base_after_rebase() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .unwrap();
            assert!(status.success(), "git command failed: {args:?}");
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        let original_base = git_output(root, &["rev-parse", "HEAD"]).unwrap();

        fs::create_dir_all(root.join(".github/workflows")).unwrap();
        fs::write(root.join(".github/workflows/ci.yml"), "name: CI\n").unwrap();
        git(&["add", ".github/workflows/ci.yml"]);
        git(&["commit", "-m", "upstream workflow"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        git(&["switch", "-c", "feature"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn feature() {}\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "feature"]);

        let mut record = completed_record(root);
        record.base_commit = Some(original_base);
        record.state = ChangeState::Implementing;
        record.affected_paths = vec!["src/".into(), SEQUENCE_PATH.into()];
        let policy = SddPolicy::default();
        assert!(
            uncovered_meaningful_paths(root, &policy, &[record])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn path_coverage_uses_non_main_remote_default_branch() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "trunk"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/trunk", "HEAD"]);
        git(&[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/trunk",
        ]);
        git(&["switch", "-c", "feature"]);
        assert_eq!(pull_request_diff_base(root, &[]), "origin/trunk...HEAD");
    }

    #[test]
    fn detached_head_verification_and_acceptance_are_supported() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "base"]);
        git(&["switch", "--detach"]);
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Accepted);
    }

    #[test]
    fn loaded_change_rejects_mismatched_or_unsafe_persisted_identity() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_record(root);
        let state_path = change_dir(root, &record.id).join("state.json");
        let original = fs::read_to_string(&state_path).unwrap();
        let mut state: serde_json::Value = serde_json::from_str(&original).unwrap();

        state["id"] = serde_json::Value::String("CHG-9999-other-workspace".into());
        fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        assert!(
            load_change(root, &record.id)
                .unwrap_err()
                .contains("does not match workspace")
        );
        assert!(
            list_changes_checked(root)
                .unwrap_err()
                .contains("does not match workspace")
        );

        state["id"] = serde_json::Value::String("../../escape".into());
        fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        assert!(
            load_change(root, &record.id)
                .unwrap_err()
                .contains("invalid change ID")
        );
    }

    #[test]
    fn loaded_change_rejects_unsafe_persisted_spec_and_artifact_scopes() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_record(root);
        let state_path = change_dir(root, &record.id).join("state.json");
        let original = fs::read_to_string(&state_path).unwrap();
        let mut state: serde_json::Value = serde_json::from_str(&original).unwrap();

        state["affected_specs"] = serde_json::json!(["../../escape"]);
        fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        assert!(
            load_change(root, &record.id)
                .unwrap_err()
                .contains("invalid affected spec")
        );

        state = serde_json::from_str(&original).unwrap();
        state["selected_artifacts"] = serde_json::json!([{"custom": "../../escape"}]);
        fs::write(&state_path, serde_json::to_vec_pretty(&state).unwrap()).unwrap();
        assert!(
            load_change(root, &record.id)
                .unwrap_err()
                .contains("unsafe custom artifact")
        );
    }

    #[test]
    fn historical_tombstone_corruption_fails_closed() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_record(root);
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
        )
        .unwrap();
        let historical = root
            .join(ARCHIVE_PATH)
            .join("2026-01-01-CHG-0000-old/deltas");
        fs::create_dir_all(&historical).unwrap();
        fs::write(historical.join("auth.md"), [0xff, 0xfe]).unwrap();

        let error = validate_delta_files(root, &record).unwrap_err();
        assert!(
            error.contains("historical semantic delta"),
            "unexpected error: {error}"
        );

        fs::write(historical.join("auth.md"), "plain garbage\n").unwrap();
        let error = validate_delta_files(root, &record).unwrap_err();
        assert!(error.contains("historical semantic delta is empty"));

        fs::write(
            historical.join("auth.md"),
            "## REMVOED\n### REQUIREMENT REQ-auth-000\nRetired.\n",
        )
        .unwrap();
        let error = validate_delta_files(root, &record).unwrap_err();
        assert!(
            error.contains("invalid delta operation heading"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn definition_approval_preserves_a_corrupt_ledger() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let record = completed_record(root);
        for artifact in &record.selected_artifacts {
            fs::write(
                change_dir(root, &record.id).join(artifact.file_name()),
                "# Complete\n\nReviewed.\n",
            )
            .unwrap();
        }
        fs::write(
            delta_path(root, &record, "auth"),
            "## ADDED\n### REQUIREMENT REQ-auth-001\nThe system SHALL work.\n\nAcceptance Criteria\n- Works\n",
        )
        .unwrap();
        let ledger_path = change_dir(root, &record.id).join("approvals.json");
        fs::write(&ledger_path, b"{corrupt").unwrap();

        assert!(approve_definition(root, &record.id, Some("Reviewer".into()), None).is_err());
        assert_eq!(fs::read(&ledger_path).unwrap(), b"{corrupt");
        assert_eq!(
            load_change(root, &record.id).unwrap().state,
            ChangeState::Draft
        );
    }

    #[test]
    fn verifying_state_requires_recorded_evidence_in_unified_checks() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut policy = SddPolicy::default();
        policy.require_change_for_meaningful_files = false;
        policy.verification_commands.clear();
        write_json(&root.join(POLICY_PATH), &policy).unwrap();
        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        record.state = ChangeState::Verifying;
        save_change(root, &record).unwrap();

        let report = check_project(root);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("verification evidence is missing"))
        );
    }

    #[test]
    fn ci_verification_accepts_evidence_from_an_ancestor_commit() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("one.txt"), "one\n").unwrap();
        git(&["add", "one.txt"]);
        git(&["commit", "-m", "one"]);
        let verified_commit = git_output(root, &["rev-parse", "HEAD"]);
        fs::write(root.join("two.txt"), "two\n").unwrap();
        git(&["add", "two.txt"]);
        git(&["commit", "-m", "two"]);
        let evidence = VerificationRecord {
            timestamp: now(),
            commit: verified_commit,
            contract_digest: "contract".into(),
            workspace_digest: "workspace".into(),
            acceptance_input_digest: None,
            acceptance_manifest: None,
            semantic_succession: None,
            passed: true,
            commands: Vec::new(),
            requirement_ids: Vec::new(),
        };

        assert!(verification_commit_is_current(root, &evidence, true));
        assert!(!verification_commit_is_current(root, &evidence, false));
    }

    #[test]
    fn no_spec_change_rejects_a_declared_public_contract_change() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record
            .answers
            .insert("public_contract".into(), "yes".into());
        save_change(root, &record).unwrap();

        let error =
            approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap_err();
        assert!(error.contains("no_spec_change"));
        assert!(load_approvals(root, &record).unwrap().approvals.is_empty());
        record.state = ChangeState::Accepted;
        let error = validate_definition(root, &record).unwrap_err();
        assert!(error.contains("no_spec_change"));
    }

    #[test]
    fn accepted_evidence_tracks_scoped_post_acceptance_inputs() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);

        let mut record = completed_no_spec_record(root);
        record = approve_definition(root, &record.id, Some("Reviewer".into()), None).unwrap();
        record = start_implementation(root, &record.id).unwrap();
        fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "implement"]);
        verify_change(root, &record.id).unwrap();
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();

        let evidence = load_verification(root, &record).unwrap();
        assert!(evidence.acceptance_input_digest.is_some());
        assert!(ensure_closing_approval_valid(root, &record).is_ok());
        fs::write(root.join("notes.txt"), "unrelated\n").unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_ok());
        git(&["add", "notes.txt"]);
        git(&["commit", "-m", "unrelated"]);
        assert!(ensure_closing_approval_valid(root, &record).is_ok());
        fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 3 }\n").unwrap();
        assert!(ensure_closing_approval_valid(root, &record).is_err());
        assert!(archive_change(root, &record.id).is_err());
        assert!(change_dir(root, &record.id).is_dir());
    }

    #[test]
    fn subproject_policy_and_diff_paths_are_project_relative() {
        let temp = TempDir::new().unwrap();
        let repository = temp.path();
        let root = repository.join("packages/app");
        let git = |dir: &Path, args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(dir)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(repository, &["init", "-b", "main"]);
        git(repository, &["config", "user.email", "test@example.com"]);
        git(repository, &["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("contracts/auth")).unwrap();
        fs::create_dir_all(root.join(".specsync")).unwrap();
        fs::create_dir_all(repository.join("other")).unwrap();
        write_json(&root.join(POLICY_PATH), &SddPolicy::default()).unwrap();
        fs::write(root.join("src/lib.rs"), "base\n").unwrap();
        fs::write(
            root.join(".specsync/config.toml"),
            "specs_dir = \"contracts\"\n",
        )
        .unwrap();
        fs::write(root.join("contracts/auth/auth.spec.md"), "base spec\n").unwrap();
        fs::write(repository.join("other/file.rs"), "base\n").unwrap();
        git(repository, &["add", "."]);
        git(repository, &["commit", "-m", "base"]);
        git(
            repository,
            &["update-ref", "refs/remotes/origin/main", "HEAD"],
        );

        let mut weakened = SddPolicy::default();
        weakened.enabled = false;
        write_json(&root.join(POLICY_PATH), &weakened).unwrap();
        assert!(policy_at_comparison_base(&root).unwrap().unwrap().enabled);
        write_json(&root.join(POLICY_PATH), &SddPolicy::default()).unwrap();
        fs::write(root.join("src/lib.rs"), "changed\n").unwrap();
        fs::write(root.join("contracts/auth/auth.spec.md"), "changed spec\n").unwrap();
        fs::write(repository.join("other/file.rs"), "outside\n").unwrap();

        let uncovered = uncovered_meaningful_paths(&root, &SddPolicy::default(), &[]).unwrap();
        assert_eq!(
            uncovered,
            vec![
                "contracts/auth/auth.spec.md".to_string(),
                "src/lib.rs".to_string()
            ]
        );
        let mut record = create_change(
            &root,
            CreateChangeRequest {
                description: "Update auth".into(),
                kind: ChangeKind::Feature,
                affected_specs: vec!["auth".into()],
                affected_paths: vec!["src/".into()],
                requested_artifacts: Vec::new(),
                no_spec_change: false,
                rationale: None,
            },
        )
        .unwrap();
        record.state = ChangeState::Implementing;
        assert!(
            uncovered_meaningful_paths(&root, &SddPolicy::default(), &[record])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn adoption_bootstrap_covers_only_the_original_policy() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::write(root.join("README.md"), "base\n").unwrap();
        git(&["add", "README.md"]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);

        adopt(root, false, None).unwrap();
        let policy = load_policy(root).unwrap();
        assert!(check_project(root).errors.is_empty());
        assert!(
            uncovered_meaningful_paths(root, &policy, &[])
                .unwrap()
                .is_empty()
        );
        let report_path = root.join(".specsync/adoption-report.json");
        let report = fs::read(&report_path).unwrap();
        fs::remove_file(&report_path).unwrap();
        assert!(
            uncovered_meaningful_paths(root, &policy, &[])
                .unwrap()
                .contains(&POLICY_PATH.to_string())
        );
        fs::write(&report_path, report).unwrap();
        let tree = git_output(root, &["rev-parse", "HEAD^{tree}"]).unwrap();
        let unrelated = git_output(root, &["commit-tree", &tree, "-m", "unrelated"]).unwrap();
        let mut parsed: serde_json::Value =
            serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
        parsed["bootstrap_policy"]["base_commit"] = serde_json::Value::String(unrelated);
        write_json(&report_path, &parsed).unwrap();
        assert!(
            uncovered_meaningful_paths(root, &policy, &[])
                .unwrap()
                .contains(&POLICY_PATH.to_string())
        );
        adopt(root, false, None).unwrap();
        let mut changed = policy;
        changed.meaningful_paths.push("private/".into());
        write_json(&root.join(POLICY_PATH), &changed).unwrap();
        assert!(
            uncovered_meaningful_paths(root, &changed, &[])
                .unwrap()
                .contains(&POLICY_PATH.to_string())
        );
    }

    #[test]
    fn overlapping_changes_cannot_lend_archive_attribution() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let git = |args: &[&str]| {
            assert!(
                Command::new("git")
                    .args(args)
                    .current_dir(root)
                    .status()
                    .unwrap()
                    .success()
            );
        };
        git(&["init", "-b", "main"]);
        git(&["config", "user.email", "test@example.com"]);
        git(&["config", "user.name", "Test"]);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "base\n").unwrap();
        git(&["add", "."]);
        git(&["commit", "-m", "base"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        write_default_policy(root, Vec::new()).unwrap();

        let mut records = vec![
            completed_no_spec_record(root),
            completed_no_spec_record(root),
        ];
        for record in &mut records {
            append_approval(
                root,
                record,
                "definition",
                Some("Reviewer".into()),
                definition_digest(root, record).unwrap(),
                None,
            )
            .unwrap();
            record.state = ChangeState::Accepted;
            save_change(root, record).unwrap();
            let verification = VerificationRecord {
                timestamp: now(),
                commit: git_output(root, &["rev-parse", "HEAD"]),
                contract_digest: definition_digest(root, record).unwrap(),
                workspace_digest: project_input_digest(root).unwrap(),
                acceptance_input_digest: None,
                acceptance_manifest: None,
                semantic_succession: None,
                passed: true,
                commands: Vec::new(),
                requirement_ids: Vec::new(),
            };
            write_json(
                &change_dir(root, &record.id).join("verification.json"),
                &verification,
            )
            .unwrap();
            append_approval(
                root,
                record,
                "acceptance",
                Some("Reviewer".into()),
                closing_digest(record, &verification),
                None,
            )
            .unwrap();
        }
        fs::write(root.join("src/lib.rs"), "delivery\n").unwrap();
        git(&["add", "src/lib.rs"]);
        git(&["commit", "-m", "delivery"]);
        for record in &records {
            let mut verification = load_verification(root, record).unwrap();
            verification.acceptance_input_digest =
                Some(acceptance_input_digest(root, record, &[]).unwrap());
            write_json(
                &change_dir(root, &record.id).join("verification.json"),
                &verification,
            )
            .unwrap();
            append_approval(
                root,
                record,
                "acceptance",
                Some("Reviewer".into()),
                closing_digest(record, &verification),
                Some("Bind current delivery inputs".into()),
            )
            .unwrap();
        }

        assert!(archive_change(root, &records[0].id).is_err());
        git(&["add", "."]);
        git(&["commit", "-m", "record accepted lifecycle evidence"]);
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        archive_change(root, &records[0].id).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn foreign_import_rejects_symlinked_markdown() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let outside = root.join("outside.md");
        fs::write(&outside, "secret\n").unwrap();
        fs::create_dir_all(root.join("openspec/specs/auth")).unwrap();
        symlink(&outside, root.join("openspec/specs/auth/spec.md")).unwrap();

        let error = adopt(root, false, Some("openspec")).unwrap_err();
        assert!(error.contains("symlinked foreign import"));
        assert!(!root.join(POLICY_PATH).exists());
        assert!(!root.join(".specsync/adoption-report.json").exists());
        assert!(
            !root
                .join(".specsync/imports/openspec/canonical/auth/spec.md")
                .exists()
        );
    }

    #[cfg(unix)]
    #[test]
    fn foreign_import_rejects_symlinked_ancestor_directories() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("project");
        let outside = temp.path().join("outside");
        fs::create_dir_all(outside.join("openspec/specs/auth")).unwrap();
        fs::write(
            outside.join("openspec/specs/auth/spec.md"),
            "external contract\n",
        )
        .unwrap();
        fs::create_dir_all(&root).unwrap();
        symlink(outside.join("openspec"), root.join("openspec")).unwrap();

        let error = adopt(&root, false, Some("openspec")).unwrap_err();
        assert!(error.contains("symlinked foreign import"));
        assert!(!root.join(POLICY_PATH).exists());
        assert!(!root.join(".specsync/adoption-report.json").exists());
        assert!(
            !root
                .join(".specsync/imports/openspec/canonical/auth/spec.md")
                .exists()
        );
    }

    #[cfg(unix)]
    #[test]
    fn speckit_import_rejects_a_symlinked_constitution_ancestor() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let root = temp.path().join("project");
        let outside = temp.path().join("outside");
        fs::create_dir_all(outside.join("memory")).unwrap();
        fs::write(outside.join("memory/constitution.md"), "external rules\n").unwrap();
        fs::create_dir_all(&root).unwrap();
        symlink(&outside, root.join(".specify")).unwrap();

        let error = adopt(&root, false, Some("speckit")).unwrap_err();
        assert!(error.contains("symlinked foreign import"));
        assert!(!root.join(POLICY_PATH).exists());
        assert!(!root.join(".specsync/adoption-report.json").exists());
        assert!(
            !root
                .join(".specsync/imports/speckit/constitution.md")
                .exists()
        );
    }
}
