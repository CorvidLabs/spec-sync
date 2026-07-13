use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

pub const SDD_VERSION: &str = "5.0.0";
const POLICY_PATH: &str = ".specsync/sdd.json";
const CHANGES_PATH: &str = ".specsync/changes";
const ARCHIVE_PATH: &str = ".specsync/archive/changes";
const LOCK_PATH: &str = ".specsync/change.lock";
const TRANSACTION_PATH: &str = ".specsync/change-transaction.json";
const MAX_CHANGE_ARTIFACT_BYTES: u64 = 4 * 1024 * 1024;
static EFFECTIVE_CONTRACT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

const DEFINITION_DIGEST_DOMAIN: &[u8] = b"specsync.definition-digest.v2";
const PROJECT_DIGEST_DOMAIN: &[u8] = b"specsync.project-input-digest.v2";
const ACCEPTANCE_DIGEST_DOMAIN: &[u8] = b"specsync.acceptance-input-digest.v2";
const CLOSING_DIGEST_DOMAIN: &[u8] = b"specsync.closing-digest.v2";

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
                ".specsync/version".into(),
            ],
            ignored_paths: vec![".specsync/".into(), "specs/".into()],
            verification_commands: Vec::new(),
            custom_artifacts: BTreeMap::new(),
            principles_file: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub answers: BTreeMap<String, String>,
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
    pub passed: bool,
    pub commands: Vec<CommandEvidence>,
    pub requirement_ids: Vec<String>,
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
    pub next_action: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SddCheckReport {
    pub enabled: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub checked_changes: usize,
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

pub fn answer_question(
    root: &Path,
    id: &str,
    question: &str,
    answer: &str,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let mut record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Draft], "answer interview questions")?;
    let values = split_values(answer);
    match question {
        "acceptance_criteria" => record.acceptance_criteria = values,
        "affected_specs" => {
            for module in &values {
                crate::commands::validate_module_name(module)
                    .map_err(|error| format!("invalid affected spec: {error}"))?;
            }
            record.affected_specs = values;
        }
        "affected_paths" => {
            record.affected_paths = values
                .iter()
                .map(|path| {
                    normalize_project_path(path)
                        .map_err(|error| format!("invalid affected path: {error}"))
                })
                .collect::<Result<_, _>>()?;
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
        passed,
        commands,
        requirement_ids,
    };
    write_json(
        &change_dir(root, &record.id).join("verification.json"),
        &verification,
    )?;
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
    if !verification_commit_is_accepted_current(root, &prior_verification)
        && !accepted_workspace_is_integrated(root, &record)
        && !accepted_change_is_recorded_in_current_history(root, &record)
        && !accepted_change_has_current_canonical_successors(root, &record)
    {
        return Err("accepted change verification commit is not in current history, its canonical acceptance is not recorded in current history, and no current canonical successor governs its affected contract".into());
    }
    let current_acceptance_input_digest = acceptance_input_digest(root, &record, &[])?;
    if current_acceptance_input_digest == stale_acceptance_input_digest {
        return Err(
            "accepted change delivery inputs are current; reopen is allowed only when delivery evidence is stale"
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

fn ensure_reopened_definition_unchanged(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if !record.canonical_applied {
        return Ok(());
    }
    let ledger = load_approvals(root, record)?;
    let reopening = ledger.reopenings.last().ok_or_else(|| {
        "cannot reaccept an already-applied change without audited reopen evidence".to_string()
    })?;
    if definition_digest_matches(root, record, &reopening.prior_verification.contract_digest)? {
        return Ok(());
    }
    Err(
        "cannot accept a modified definition of an already-applied change; perform further spec changes in a new change workspace"
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
    verification.acceptance_input_digest = Some(acceptance_input_digest(root, &record, &prepared)?);
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
    let _lock = acquire_project_lock(root)?;
    list_changes_checked(root)?;
    let record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Accepted], "archive the change")?;
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
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{}", today(), record.id));
    if destination.exists() {
        return Err(format!(
            "archive destination already exists: {}",
            destination.display()
        ));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::rename(&source, &destination).map_err(|error| {
        format!(
            "failed to archive {} to {}: {error}",
            source.display(),
            destination.display()
        )
    })?;
    let mut archived = record.clone();
    archived.state = ChangeState::Archived;
    archived.updated_at = now();
    if let Err(error) = write_json(&destination.join("state.json"), &archived).and_then(|()| {
        fs::write(
            destination.join("change.md"),
            change_markdown_content(&archived),
        )
        .map_err(|error| error.to_string())
    }) {
        let restore = write_json(&destination.join("state.json"), &record)
            .and_then(|()| {
                fs::write(
                    destination.join("change.md"),
                    change_markdown_content(&record),
                )
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
    Ok(destination)
}

pub fn summarize_change(root: &Path, record: &ChangeRecord) -> ChangeSummary {
    let approval_valid = ensure_definition_approval_valid(root, record).is_ok();
    let artifacts_complete = validate_artifacts(root, record).is_ok();
    let next_action = match record.state {
        ChangeState::Draft if next_questions(record).is_empty() => "approve".into(),
        ChangeState::Draft => "answer interview".into(),
        ChangeState::Approved => "start".into(),
        ChangeState::Implementing => "verify".into(),
        ChangeState::Verifying => "accept".into(),
        ChangeState::Accepted if ensure_closing_approval_valid(root, record).is_err() => {
            "reopen".into()
        }
        ChangeState::Accepted => "archive".into(),
        ChangeState::Archived => "none".into(),
    };
    ChangeSummary {
        id: record.id.clone(),
        title: record.title.clone(),
        state: record.state,
        approval_valid,
        artifacts_complete,
        next_action,
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
    let mut report = SddCheckReport {
        enabled: true,
        checked_changes: records.len(),
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
            report.errors.push(format!("{}: {error}", record.id));
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
        && record
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
    let dir = change_dir(root, &record.id);
    for artifact in &record.selected_artifacts {
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
    if !record.selected_artifacts.contains(&ArtifactKind::Tasks) {
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
            read_bounded_change_text(&delta_path(root, record, module), "semantic delta")?;
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
        let canonical = root
            .join(&config.specs_dir)
            .join(&module)
            .join(format!("{module}.spec.md"));
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
            let delta_path = delta_path(root, record, &module);
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
        let path = delta_path(root, record, module);
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
        let path = delta_path(root, record, module);
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
        let delta = read_bounded_change_text(&delta_path(root, record, module), "semantic delta")?;
        let items = parse_delta(&delta)?;
        let spec_path = root
            .join(&specs_dir)
            .join(module)
            .join(format!("{module}.spec.md"));
        let requirements_path = root.join(&specs_dir).join(module).join("requirements.md");
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
        let path = delta_path(root, record, module);
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
    let mut canonical_record = record.clone();
    canonical_record.state = ChangeState::Draft;
    canonical_record.canonical_applied = false;
    canonical_record.updated_at = 0;
    let record_bytes = serde_json::to_vec(&canonical_record)
        .map_err(|error| format!("failed to hash change state: {error}"))?;
    definition_digest_from_record_bytes(root, record, &record_bytes)
}

fn definition_digest_with_explicit_false(
    root: &Path,
    record: &ChangeRecord,
) -> Result<String, String> {
    let mut canonical_record = record.clone();
    canonical_record.state = ChangeState::Draft;
    canonical_record.canonical_applied = false;
    canonical_record.updated_at = 0;
    let mut record_bytes = serde_json::to_vec(&canonical_record)
        .map_err(|error| format!("failed to hash change state: {error}"))?;
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
    definition_digest_from_record_bytes(root, record, &record_bytes)
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
) -> Result<String, String> {
    let dir = change_dir(root, &record.id);
    let mut digest = FramedDigest::new(DEFINITION_DIGEST_DOMAIN);
    digest.frame(b"record", record_bytes);
    let mut files = Vec::new();
    for artifact in &record.selected_artifacts {
        files.push(dir.join(artifact.file_name()));
    }
    if let Ok(entries) = fs::read_dir(dir.join("deltas")) {
        files.extend(entries.flatten().map(|entry| entry.path()));
    }
    if let Some(policy) = load_policy(root)
        && let Some(principles) = policy.principles_file
    {
        files.push(safe_project_path(root, &principles)?);
    }
    files.sort();
    let git_modes = git_index_modes(root)?;
    for path in files {
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.len() > MAX_CHANGE_ARTIFACT_BYTES {
            return Err(format!(
                "approval input exceeds {} byte limit: {}",
                MAX_CHANGE_ARTIFACT_BYTES,
                path.display()
            ));
        }
        let relative = strict_portable_project_path(root, &path)?;
        let content = fs::read(&path)
            .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
        let (kind, mode) = digest_file_kind_and_mode(&relative, &path, &git_modes)?;
        digest.entry(&relative, kind, mode, &content);
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
    digest.finish()
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

fn ensure_closing_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let verification = load_verification(root, record)?;
    if !verification.passed {
        return Err("accepted change has failed verification evidence".into());
    }
    if !definition_digest_matches(root, record, &verification.contract_digest)? {
        return Err("accepted change verification contract is stale".into());
    }
    let expected_inputs = verification
        .acceptance_input_digest
        .as_ref()
        .ok_or_else(|| "accepted change is missing current delivery-input evidence".to_string())?;
    let current_inputs = acceptance_input_digest(root, record, &[])?;
    if current_inputs != *expected_inputs {
        return Err("accepted change verification is stale for current delivery inputs".into());
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
        && !accepted_change_is_recorded_in_current_history(root, record)
        && !accepted_change_has_current_canonical_successors(root, record)
    {
        return Err("accepted change verification commit is not in current history, its canonical acceptance is not recorded in current history, and no current canonical successor governs its affected contract".into());
    }
    Ok(())
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
    let path = change_dir(root, &record.id).join("approvals.json");
    let content = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&content).map_err(|error| error.to_string())
}

fn load_verification(root: &Path, record: &ChangeRecord) -> Result<VerificationRecord, String> {
    let path = change_dir(root, &record.id).join("verification.json");
    let content =
        fs::read_to_string(&path).map_err(|_| "verification evidence is missing".to_string())?;
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid verification evidence: {error}"))
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
    let specs_scope = configured_specs_scope(root);
    record.affected_specs.iter().any(|module| {
        let module_scope = if specs_scope == "." {
            format!("{module}/")
        } else {
            format!("{specs_scope}{module}/")
        };
        path_matches_scope(path, &module_scope)
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
            | ".specsync/version"
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
    command.args(args).current_dir(root);
    if matches!(output, ConfiguredCommandOutput::Suppress) {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    command.status().map_err(|error| {
        format!("failed to run configured verification command `{configured}`: {error}")
    })
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
    let words: Vec<String> = command.split_whitespace().map(str::to_string).collect();
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
    if active.is_dir() {
        return Ok(active);
    }
    let archive = root.join(ARCHIVE_PATH);
    if let Ok(entries) = fs::read_dir(archive) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().ends_with(id) {
                return Ok(entry.path());
            }
        }
    }
    Err(format!("change `{id}` was not found"))
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
                repo_workspace.as_str(),
            ])
            .current_dir(root)
            .status()
            .is_ok_and(|status| status.success())
}

fn accepted_change_is_recorded_in_current_history(root: &Path, record: &ChangeRecord) -> bool {
    let state = format!("{CHANGES_PATH}/{}/state.json", record.id);
    let Ok(repo_state) = git_repo_relative_path(root, &state) else {
        return false;
    };
    let top_state = format!(":(top){repo_state}");
    let history = Command::new("git")
        .args(["log", "--format=%H", "--", top_state.as_str()])
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
                && candidate.id > record.id
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

        assert!(updated.contains(&format!(
            "| {} |  | CHG-0002: Document behavior |",
            today()
        )));
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
        fs::write(root.join("src/lib.rs"), "pub fn ready() -> bool { true }\n").unwrap();
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
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        assert!(archive_change(root, &record.id).is_ok());
    }

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
        assert!(accepted_change_is_recorded_in_current_history(root, &record));
        assert!(ensure_closing_approval_valid(root, &record).is_ok());

        git(&["switch", "main"]);
        assert!(archive_change(root, &record.id).is_ok());
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
        assert!(error.contains("delivery inputs"));
    }

    #[test]
    fn failed_archive_move_leaves_an_accepted_change_retryable() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        let mut record = completed_no_spec_record(root);
        record.state = ChangeState::Accepted;
        save_change(root, &record).unwrap();
        write_change_markdown(root, &record).unwrap();
        let verification = VerificationRecord {
            timestamp: now(),
            commit: None,
            contract_digest: definition_digest(root, &record).unwrap(),
            workspace_digest: project_input_digest(root).unwrap(),
            acceptance_input_digest: Some(acceptance_input_digest(root, &record, &[]).unwrap()),
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
        let destination = root
            .join(ARCHIVE_PATH)
            .join(format!("{}-{}", today(), record.id));
        fs::create_dir_all(&destination).unwrap();

        let error = archive_change(root, &record.id).unwrap_err();
        assert!(error.contains("archive destination already exists"));
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
        record.affected_paths = vec!["src/".into()];
        record.state = ChangeState::Approved;
        assert_eq!(
            uncovered_meaningful_paths(root, &SddPolicy::default(), &[record.clone()]).unwrap(),
            vec!["src/lib.rs"]
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
        record.state = ChangeState::Accepted;
        save_change(root, &record).unwrap();
        let verification = VerificationRecord {
            timestamp: now(),
            commit: None,
            contract_digest: definition_digest(root, &record).unwrap(),
            workspace_digest: "workspace".into(),
            acceptance_input_digest: None,
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
        assert_eq!(
            shell_words("fledge run test").unwrap(),
            vec!["fledge", "run", "test"]
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
        assert!(!path_is_meaningful(
            ".specsync/adoption-report.json",
            &hostile
        ));
        assert!(!path_is_meaningful(".specsync/registry.toml", &hostile));
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
        record.affected_paths = vec!["src/".into()];
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
            record.state = ChangeState::Accepted;
            save_change(root, record).unwrap();
            let verification = VerificationRecord {
                timestamp: now(),
                commit: git_output(root, &["rev-parse", "HEAD"]),
                contract_digest: definition_digest(root, record).unwrap(),
                workspace_digest: project_input_digest(root).unwrap(),
                acceptance_input_digest: None,
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
        git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
        assert!(archive_change(root, &records[0].id).is_ok());
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
