use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const SDD_VERSION: &str = "5.0.0";
const POLICY_PATH: &str = ".specsync/sdd.json";
const CHANGES_PATH: &str = ".specsync/changes";
const ARCHIVE_PATH: &str = ".specsync/archive/changes";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApprovalLedger {
    pub approvals: Vec<ApprovalRecord>,
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
    let content = fs::read_to_string(root.join(POLICY_PATH)).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_default_policy(root: &Path, verification_commands: Vec<String>) -> Result<(), String> {
    let path = root.join(POLICY_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if path.exists() {
        return Ok(());
    }
    let policy = SddPolicy {
        verification_commands,
        ..SddPolicy::default()
    };
    write_json(&path, &policy)
}

pub fn create_change(root: &Path, request: CreateChangeRequest) -> Result<ChangeRecord, String> {
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
    for path in &affected_paths {
        safe_project_path(root, path).map_err(|error| format!("invalid affected path: {error}"))?;
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
    serde_json::from_str(&content)
        .map_err(|error| format!("invalid change state {}: {error}", path.display()))
}

pub fn list_changes(root: &Path) -> Vec<ChangeRecord> {
    let mut records = Vec::new();
    let dir = root.join(CHANGES_PATH);
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return records,
    };
    for entry in entries.flatten() {
        let path = entry.path().join("state.json");
        if let Ok(content) = fs::read_to_string(path)
            && let Ok(record) = serde_json::from_str(&content)
        {
            records.push(record);
        }
    }
    records.sort_by(|left: &ChangeRecord, right: &ChangeRecord| left.id.cmp(&right.id));
    records
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
            for path in &values {
                safe_project_path(root, path)
                    .map_err(|error| format!("invalid affected path: {error}"))?;
            }
            record.affected_paths = values;
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
    let prior_state = record.state;
    validate_definition(root, &record)?;
    let digest = definition_digest(root, &record)?;
    append_approval(root, &record, "definition", actor, digest, note)?;
    record.state = match prior_state {
        ChangeState::Draft | ChangeState::Approved => ChangeState::Approved,
        ChangeState::Implementing | ChangeState::Verifying => ChangeState::Implementing,
        ChangeState::Accepted | ChangeState::Archived => prior_state,
    };
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    Ok(record)
}

pub fn start_implementation(root: &Path, id: &str) -> Result<ChangeRecord, String> {
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
    let mut record = load_change(root, id)?;
    require_state(
        &record,
        &[ChangeState::Implementing, ChangeState::Verifying],
        "verify the change",
    )?;
    ensure_definition_approval_valid(root, &record)?;
    validate_definition(root, &record)?;
    validate_delta_files(root, &record)?;
    ensure_tasks_complete(root, &record)?;
    let policy = load_policy(root).unwrap_or_default();
    let mut commands = Vec::new();
    for configured in policy.verification_commands {
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
    let requirement_ids = collect_requirement_ids(root, &record)?;
    let missing_evidence = requirement_evidence_missing(root, &record, &requirement_ids);
    let passed = commands.iter().all(|command| command.success)
        && acceptance_criteria_have_evidence(&record, &requirement_ids)
        && missing_evidence.is_empty();
    let verification = VerificationRecord {
        timestamp: now(),
        commit: git_output(root, &["rev-parse", "HEAD"]),
        contract_digest: definition_digest(root, &record)?,
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
        let detail = if missing_evidence.is_empty() {
            "configured verification command failed".to_string()
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

pub fn accept_change(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
) -> Result<ChangeRecord, String> {
    let mut record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Verifying], "accept the change")?;
    ensure_definition_approval_valid(root, &record)?;
    let verification = load_verification(root, &record)?;
    if !verification.passed {
        return Err("cannot accept a change with failed verification".into());
    }
    let current_commit = git_output(root, &["rev-parse", "HEAD"]);
    if verification.commit != current_commit {
        return Err(
            "verification is stale because HEAD changed; run `specsync change verify` again".into(),
        );
    }
    if verification.contract_digest != definition_digest(root, &record)? {
        return Err("verification is stale because the approved contract changed".into());
    }
    let mut prepared = prepare_delta_application(root, &record)?;
    let closing_digest = closing_digest(&record, &verification);
    let mut ledger = load_approvals(root, &record)?;
    ledger.approvals.push(ApprovalRecord {
        gate: "acceptance".into(),
        actor: resolve_actor(root, actor)?,
        timestamp: now(),
        digest: closing_digest,
        note,
    });
    let approvals_path = change_dir(root, &record.id).join("approvals.json");
    prepared.push((approvals_path, json_content(&ledger)?));
    record.state = ChangeState::Accepted;
    record.updated_at = now();
    prepared.push((
        change_dir(root, &record.id).join("state.json"),
        json_content(&record)?,
    ));
    prepared.push((
        change_dir(root, &record.id).join("change.md"),
        change_markdown_content(&record),
    ));
    write_prepared_files(&prepared)?;
    Ok(record)
}

pub fn archive_change(root: &Path, id: &str) -> Result<PathBuf, String> {
    let mut record = load_change(root, id)?;
    require_state(&record, &[ChangeState::Accepted], "archive the change")?;
    record.state = ChangeState::Archived;
    record.updated_at = now();
    save_change(root, &record)?;
    write_change_markdown(root, &record)?;
    let source = change_dir(root, &record.id);
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{}", today(), record.id));
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

pub fn check_project(root: &Path) -> SddCheckReport {
    let Some(policy) = load_policy(root) else {
        return SddCheckReport::default();
    };
    if !policy.enabled {
        return SddCheckReport::default();
    }
    let records = list_changes(root);
    let mut report = SddCheckReport {
        enabled: true,
        checked_changes: records.len(),
        ..SddCheckReport::default()
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
        if record.state == ChangeState::Verifying && !is_ci() {
            match load_verification(root, record) {
                Ok(evidence) => {
                    if evidence.commit != git_output(root, &["rev-parse", "HEAD"])
                        || definition_digest(root, record)
                            .map(|digest| digest != evidence.contract_digest)
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
    if is_ci()
        && records.iter().any(|record| {
            matches!(
                record.state,
                ChangeState::Implementing | ChangeState::Verifying | ChangeState::Accepted
            )
        })
    {
        for configured in &policy.verification_commands {
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
                    report.errors.push(format!(
                        "meaningful changed paths are not covered by an active change: {}",
                        paths.join(", ")
                    ));
                }
            }
            Err(error) => report.warnings.push(error),
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
    write_default_policy(root, detect_verification_commands(root))?;
    write_json(
        &root.join(".specsync/adoption-report.json"),
        &serde_json::json!({
            "requirements_needing_ids": requirement_proposals,
            "generated_at": now(),
        }),
    )?;
    if let Some(source) = detected.as_deref() {
        import_foreign(root, source)?;
    }
    Ok(actions)
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
    if !next_questions(record).is_empty() {
        return Err("the deterministic interview is incomplete".into());
    }
    if let Some(policy) = load_policy(root)
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

fn validate_artifacts(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let dir = change_dir(root, &record.id);
    for artifact in &record.selected_artifacts {
        let path = dir.join(artifact.file_name());
        let content = fs::read_to_string(&path)
            .map_err(|_| format!("required artifact is missing: {}", path.display()))?;
        if content.contains("<!-- TODO") || content.trim().is_empty() {
            return Err(format!("artifact is incomplete: {}", path.display()));
        }
    }
    Ok(())
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

fn acceptance_criteria_have_evidence(record: &ChangeRecord, requirement_ids: &[String]) -> bool {
    if record.no_spec_change {
        return !record.acceptance_criteria.is_empty();
    }
    !record.acceptance_criteria.is_empty() && !requirement_ids.is_empty()
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
        let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
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
            matches!(
                record.state,
                ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
            )
        })
        .collect();
    if active.is_empty() {
        return Ok(());
    }
    let mut modules = BTreeSet::new();
    for record in &active {
        modules.extend(record.affected_specs.iter().cloned());
    }
    let temp = std::env::temp_dir().join(format!(
        "specsync-effective-{}-{}",
        std::process::id(),
        now()
    ));
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
            if !record.affected_specs.contains(&module) {
                continue;
            }
            let delta = match fs::read_to_string(delta_path(root, record, &module)) {
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

fn validate_delta_files(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if record.no_spec_change {
        return Ok(());
    }
    let tombstones = removed_requirement_ids(root);
    for module in &record.affected_specs {
        let path = delta_path(root, record, module);
        let content = fs::read_to_string(&path)
            .map_err(|_| format!("missing semantic delta for `{module}`: {}", path.display()))?;
        let items = parse_delta(&content)?;
        if items.is_empty() {
            return Err(format!("semantic delta for `{module}` is empty"));
        }
        for item in items {
            if item.target == DeltaTarget::Requirement && item.operation != DeltaOperation::Removed
            {
                validate_requirement(&item.key, &item.content)?;
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

fn removed_requirement_ids(root: &Path) -> BTreeSet<String> {
    let mut removed = BTreeSet::new();
    for base in [root.join(CHANGES_PATH), root.join(ARCHIVE_PATH)] {
        for entry in walkdir::WalkDir::new(base)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !entry.file_type().is_file()
                || path.extension().and_then(|extension| extension.to_str()) != Some("md")
                || !path
                    .components()
                    .any(|component| component.as_os_str() == "deltas")
            {
                continue;
            }
            if let Ok(content) = fs::read_to_string(path)
                && let Ok(items) = parse_delta(&content)
            {
                for item in items {
                    if item.target == DeltaTarget::Requirement
                        && item.operation == DeltaOperation::Removed
                    {
                        removed.insert(item.key);
                    }
                }
            }
        }
    }
    removed
}

fn collect_requirement_ids(root: &Path, record: &ChangeRecord) -> Result<Vec<String>, String> {
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
                _ => operation,
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
        let delta = fs::read_to_string(delta_path(root, record, module))
            .map_err(|error| error.to_string())?;
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
    let lines: Vec<&str> = source.lines().collect();
    let start = lines.iter().position(|line| line.trim_end() == heading);
    let end = start.map(|index| {
        lines
            .iter()
            .enumerate()
            .skip(index + 1)
            .find(|(_, line)| line.starts_with(prefix))
            .map(|(next, _)| next)
            .unwrap_or(lines.len())
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
        Vec::new()
    } else {
        let mut value = vec![heading, String::new()];
        value.extend(content.lines().map(str::to_string));
        value.push(String::new());
        value
    };
    let mut output: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
    if let (Some(start), Some(end)) = (start, end) {
        output.splice(start..end, replacement);
    } else {
        if output.last().is_some_and(|line| !line.is_empty()) {
            output.push(String::new());
        }
        output.extend(replacement);
    }
    Ok(format!("{}\n", output.join("\n").trim_end()))
}

fn bump_spec_version(content: &str) -> Result<String, String> {
    let mut found = false;
    let mut output = Vec::new();
    for line in content.lines() {
        if !found && line.starts_with("version:") {
            let version = line
                .trim_start_matches("version:")
                .trim()
                .parse::<u64>()
                .map_err(|_| "spec version must be an integer")?;
            output.push(format!("version: {}", version + 1));
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

fn append_changelog(content: &str, id: &str, title: &str) -> String {
    let row = format!("| {} | {id}: {} |", today(), title.replace('|', "\\|"));
    if let Some(position) = content.rfind("## Change Log") {
        let search_start = position + "## Change Log".len();
        let section_end = content[search_start..]
            .find("\n## ")
            .map(|offset| search_start + offset)
            .unwrap_or(content.len());
        let before = content[..section_end].trim_end();
        let after = &content[section_end..];
        return format!("{before}\n{row}\n{after}");
    }
    format!(
        "{}\n## Change Log\n\n| Date | Change |\n|------|--------|\n{row}\n",
        content.trim_end()
    )
}

fn write_prepared_files(prepared: &[(PathBuf, String)]) -> Result<(), String> {
    for (path, _) in prepared {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
    }
    let backups: Vec<(PathBuf, Option<String>)> = prepared
        .iter()
        .map(|(path, _)| (path.clone(), fs::read_to_string(path).ok()))
        .collect();
    for (path, content) in prepared {
        if let Err(error) = fs::write(path, content) {
            for (backup_path, original) in backups {
                if let Some(original) = original {
                    let _ = fs::write(backup_path, original);
                } else {
                    let _ = fs::remove_file(backup_path);
                }
            }
            return Err(format!(
                "atomic delta application failed at {}: {error}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn ensure_no_delta_conflicts(root: &Path, current: &ChangeRecord) -> Result<(), String> {
    if matches!(current.state, ChangeState::Accepted | ChangeState::Archived) {
        return Ok(());
    }
    let current_keys = delta_keys(root, current)?;
    for other in list_changes(root) {
        if other.id == current.id
            || matches!(
                other.state,
                ChangeState::Draft | ChangeState::Accepted | ChangeState::Archived
            )
            || current.dependencies.contains(&other.id)
            || other.dependencies.contains(&current.id)
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
    let dir = change_dir(root, &record.id);
    let mut canonical_record = record.clone();
    canonical_record.state = ChangeState::Draft;
    canonical_record.updated_at = 0;
    let mut hasher = Sha256::new();
    hasher.update(
        serde_json::to_vec(&canonical_record)
            .map_err(|error| format!("failed to hash change state: {error}"))?,
    );
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
    for path in files {
        let content = fs::read(&path)
            .map_err(|error| format!("failed to hash {}: {error}", path.display()))?;
        let portable_path = path.strip_prefix(root).unwrap_or(&path);
        hasher.update(portable_path.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(content);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn closing_digest(record: &ChangeRecord, verification: &VerificationRecord) -> String {
    let mut hasher = Sha256::new();
    hasher.update(record.id.as_bytes());
    hasher.update(verification.contract_digest.as_bytes());
    hasher.update(verification.commit.as_deref().unwrap_or("").as_bytes());
    format!("{:x}", hasher.finalize())
}

fn ensure_definition_approval_valid(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let ledger = load_approvals(root, record)?;
    let approval = ledger
        .approvals
        .iter()
        .rev()
        .find(|approval| approval.gate == "definition")
        .ok_or_else(|| "definition approval is missing".to_string())?;
    if approval.digest != definition_digest(root, record)? {
        return Err(
            "definition approval is stale; approve the current artifact digest again".into(),
        );
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
    let mut ledger = load_approvals(root, record).unwrap_or_default();
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
    let mut args = vec!["diff", "--name-only"];
    let base = pull_request_diff_base(root, records);
    args.push(&base);
    let output = git_output(root, &args)
        .ok_or_else(|| "unable to inspect changed paths for SDD coverage".to_string())?;
    let covered: Vec<&str> = records
        .iter()
        .filter(|record| !matches!(record.state, ChangeState::Draft | ChangeState::Archived))
        .flat_map(|record| record.affected_paths.iter().map(String::as_str))
        .collect();
    let uncovered = output
        .lines()
        .filter(|path| path_is_meaningful(path, policy))
        .filter(|path| !covered.iter().any(|scope| path_matches_scope(path, scope)))
        .map(str::to_string)
        .collect();
    Ok(uncovered)
}

fn pull_request_diff_base(root: &Path, records: &[ChangeRecord]) -> String {
    if let Ok(branch) = std::env::var("GITHUB_BASE_REF")
        && !branch.trim().is_empty()
    {
        return format!("origin/{branch}...HEAD");
    }
    if let Some(remote_default) = git_output(
        root,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    ) {
        return format!("{remote_default}...HEAD");
    }
    for candidate in ["origin/main", "origin/master"] {
        if git_output(root, &["rev-parse", "--verify", candidate]).is_some() {
            return format!("{candidate}...HEAD");
        }
    }
    records
        .iter()
        .filter_map(|record| record.base_commit.clone())
        .min()
        .unwrap_or_else(|| "HEAD~1...HEAD".into())
}

fn path_is_meaningful(path: &str, policy: &SddPolicy) -> bool {
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
    path == scope.trim_end_matches('/') || path.starts_with(scope)
}

fn safe_project_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.is_absolute()
        || relative.contains('\\')
        || relative.chars().any(char::is_control)
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
    Ok(root.join(path))
}

fn run_configured_command(
    root: &Path,
    configured: &str,
) -> Result<std::process::ExitStatus, String> {
    let parts = shell_words(configured)?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| "empty verification command".to_string())?;
    Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map(|output| output.status)
        .map_err(|error| {
            format!("failed to run configured verification command `{configured}`: {error}")
        })
}

fn is_ci() -> bool {
    ["CI", "GITHUB_ACTIONS"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .any(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
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
            if canonical.is_dir() {
                copy_markdown_files(&canonical, &import_root.join("canonical"))?;
            }
        }
        "speckit" => {
            let constitution = root.join(".specify/memory/constitution.md");
            if constitution.is_file() {
                fs::create_dir_all(&import_root).map_err(|error| error.to_string())?;
                fs::copy(&constitution, import_root.join("constitution.md"))
                    .map_err(|error| error.to_string())?;
            }
        }
        _ => {}
    }
    let source_changes = match source {
        "openspec" => root.join("openspec/changes"),
        "speckit" => root.join("specs"),
        _ => return Err(format!("unsupported adoption source `{source}`")),
    };
    if !source_changes.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(source_changes)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        let entry_path = entry.path();
        if !entry_path.is_dir() || entry.file_name() == "archive" {
            continue;
        }
        let is_active_change = match source {
            "openspec" => {
                entry_path.join("proposal.md").exists()
                    || entry_path.join("design.md").exists()
                    || entry_path.join("specs").is_dir()
            }
            "speckit" => {
                entry_path.join("spec.md").exists()
                    || entry_path.join("plan.md").exists()
                    || entry_path.join("tasks.md").exists()
            }
            _ => false,
        };
        if !is_active_change {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let description = format!("Imported {source} change {name}");
        let record = create_change(
            root,
            CreateChangeRequest {
                description,
                kind: ChangeKind::Feature,
                affected_specs: Vec::new(),
                affected_paths: vec![
                    entry_path
                        .strip_prefix(root)
                        .unwrap_or(&entry_path)
                        .to_string_lossy()
                        .to_string(),
                ],
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
        copy_markdown_files(&entry_path, &destination)?;
    }
    Ok(())
}

fn copy_markdown_files(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    for entry in fs::read_dir(source)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            copy_markdown_files(&path, &destination.join(entry.file_name()))?;
        } else if path.extension().and_then(|value| value.to_str()) == Some("md") {
            fs::copy(&path, destination.join(entry.file_name()))
                .map_err(|error| error.to_string())?;
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
    use tempfile::TempDir;

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
        fs::create_dir_all(root.join("specs/auth")).unwrap();
        fs::write(
            root.join("specs/auth/auth.spec.md"),
            "---\nmodule: auth\nversion: 1\nstatus: stable\nfiles: []\n---\n\n# Auth\n\n## Purpose\n\nAuth.\n\n## Requirements\n\nExisting.\n\n## Public API\n\nNone.\n\n## Invariants\n\nStable.\n\n## Behavioral Examples\n\nWorks.\n\n## Error Cases\n\nNone.\n\n## Dependencies\n\nNone.\n\n## Change Log\n\n| Date | Change |\n|------|--------|\n| 2026-01-01 | Initial |\n",
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
            "# Auth delta\n\n## ADDED\n\n### REQUIREMENT REQ-auth-001\n\nThe system SHALL support passkey authentication.\n\nAcceptance Criteria\n- A registered passkey authenticates the user.\n\n## MODIFIED\n\n### SPEC SECTION Requirements\n\n1. Passkey authentication is supported and traced to `REQ-auth-001`.\n",
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
        record = accept_change(root, &record.id, Some("Reviewer".into()), None).unwrap();
        assert_eq!(record.state, ChangeState::Accepted);
        let spec = fs::read_to_string(root.join("specs/auth/auth.spec.md")).unwrap();
        assert!(spec.contains("version: 2"));
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
    fn prepared_write_failure_rolls_back_prior_files() {
        let temp = TempDir::new().unwrap();
        let first = temp.path().join("first.md");
        let second = temp.path().join("second.md");
        fs::write(&first, "original\n").unwrap();
        fs::create_dir(&second).unwrap();
        let result = write_prepared_files(&[
            (first.clone(), "changed\n".into()),
            (second, "cannot write a directory\n".into()),
        ]);
        assert!(result.is_err());
        assert_eq!(fs::read_to_string(first).unwrap(), "original\n");
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
}
