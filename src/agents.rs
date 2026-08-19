use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

// ─── Shared instruction body ─────────────────────────────────────────────────
//
// This is a standalone copy of the workflow prose also found (in slightly
// different phrasing) in hooks.rs's CLAUDE_MD_SNIPPET/AGENTS_MD_SNIPPET. It is
// duplicated rather than shared because hooks.rs's four snippets are not
// identical to each other (Cursor/Copilot use a terser style) and unifying
// them is out of scope here — this body is purely for the new SKILL.md files.

const SKILL_BODY: &str = r#"## Companion files

## Verified change lifecycle (6.0)

For every meaningful source, test, public documentation, schema, or configuration change:

1. Run `specsync change new "<intent>" --json` and conduct the returned interview with the user.
2. Use `specsync change answer <id> <question-id> "<answer>" --json` until no questions remain.
3. Complete the adaptively selected artifacts and semantic deltas. Requirements use stable
   `REQ-<module>-<number>` IDs, a normative SHALL statement, and acceptance criteria.
4. Ask the user for the single scope approval, then run `specsync change approve <id>`.
5. Implement code, canonical specs, and tests on the same branch. Run `specsync change check [<id>]`
   for **scoped** verification of this change only (materialize deltas + targeted tests). Do **not**
   treat check as a full archive integrity walk. Use `specsync change audit` when you need project
   health over **active** workspaces and living specs. Archives are history.
6. Complete ordinary pull-request review. For agent-authored work, have an independent reviewer
   inspect the change package, implementation diff, canonical spec delta, and targeted evidence
   once, then record it with `specsync change review <id> --reviewer "<identity>"`.
7. Run `specsync change finalize <id>` to create the same-PR metadata/archive-only commit, then
   merge through GitHub. SpecSync does not merge the pull request.

## Lifecycle verbs

- `specsync change check [id]` — verify **this** change (materialize + targeted tests). Default daily path.
- `specsync change audit` — project health over **active** workspaces and living specs. Not archive history.
- Archives are history; do not re-validate terminal evidence for every archived CHG on each check.
- Slash commands: `/specsync:check`, `/specsync:audit` (Claude/Cursor/Gemini via `specsync agents install`).

Never invent or self-grant the scope approval or independent review. If an approved definition
changes, its digest becomes stale and must be approved again. `specsync change status` always
prints one explicit next action. Historical repair commands remain available for older evidence,
but new changes use this single workflow.

Each canonical spec may have policy-selected companion files. Read and update the ones present; do not create empty companions only for ceremony:

- **`tasks.md`** — Work items for this module. Check off tasks (`- [x]`) as you complete them. Add new tasks if you discover work needed.
- **`requirements.md`** — Acceptance criteria and user stories. These are permanent invariants, not tasks — do not check them off. Update if requirements change.
- **`context.md`** — Architectural decisions, key files, and current status. Update when you make design decisions or change what's in progress.
- **`testing.md`** — Test strategy: automated test locations, manual QA checklists, and edge cases/boundary conditions.
- **`design.md`** *(opt-in)* — Layout, component hierarchy, design tokens, and asset references. Present when `companions.design` is enabled in config.

## Before modifying any module

1. Read the relevant spec in `specs/<module>/<module>.spec.md`
2. Read whichever companion files are present (`requirements.md`, `tasks.md`, `context.md`, `testing.md`, `design.md`, or project-defined files)
3. After changes, run `specsync check` to verify specs still pass

## After completing work

1. Mark completed items in `tasks.md` — check off finished tasks, add new ones discovered
2. Update `context.md` — record decisions made, update current status
3. If requirements changed, update `requirements.md` acceptance criteria
4. If test coverage changed, update `testing.md` with new test files or edge cases
5. If UI/layout changed, update `design.md` with revised layout, components, or tokens

## Before creating a PR

Run `specsync check --strict` — all specs must pass with zero warnings.

## When adding new modules

Run `specsync scaffold <module-name>` to create a spec, companion files, a registry
entry, and auto-detected source files — or `specsync new <module-name>` for a
minimal spec-only draft. Complete the spec before writing code. The
`/specsync:create-spec` command (or tool-equivalent) runs this for you, and
accepts either a bare module name or a natural-language feature description
(e.g. `/specsync:create-spec "I want a feature that lets users export their
data as CSV"`) — pass a description and it will pick a module name and use
the description to draft the spec's Purpose and Requirements.

## Key commands

- `specsync check` — validate all specs against source code
- `specsync check --json` — machine-readable validation output
- `specsync change check [id]` — scoped verification for one SDD change
- `specsync change audit` — active workspaces + living specs (not archive history)
- `specsync coverage` — show which modules lack specs
- `specsync score` — quality score for each spec (0-100)
- `specsync scaffold <name>` — full scaffold: spec + companions + registry entry + source detection
- `specsync new <name>` — quick-create a minimal spec (add `--full` for companions)
- `specsync resolve --remote` — verify cross-project dependencies
"#;

// ─── Create-spec command body (shared prose, per-tool argument syntax) ───────

const CREATE_SPEC_STEPS_MD: &str = r#"1. Read the complete arguments above. Remove each standalone `--minimal` flag
   (in any position) and remember that minimal mode was requested. Preserve
   the complete remaining input for classification; do not extract a module
   name yet. If nothing remains, ask the user for a module or description.
2. Classify the complete remaining input as one of:
   - **A bare module name** — the entire input is one identifier with no
     whitespace, such as `auth-service` or `billing`. Use it as-is.
   - **A free-text feature description** — any quoted or unquoted sentence or
     phrase describing what to build, e.g. `"I want a feature that lets users
     export their data as CSV"`. Only after making this classification, invent
     a short, kebab-case module name that captures the idea (e.g. `csv-export`).
     Never use only the first word as the module name. If the right name is
     ambiguous, ask the user to confirm or rename it before continuing. Keep
     the complete description at hand — you'll use it in step 5.
   Flag position does not change classification:
   - `--minimal billing` and `billing --minimal` both select the bare module
     `billing` in minimal mode.
   - `--minimal I need CSV export` and `I need CSV export --minimal` both keep
     the complete description and derive a name such as `csv-export`, not `I`.
3. If minimal mode was requested, run:
   ```
   specsync new <module-name>
   ```
   This creates a minimal spec only (no companion files).
4. Otherwise (default), run:
   ```
   specsync scaffold <module-name>
   ```
   This creates the spec, companion files (`tasks.md`, `requirements.md`,
   `context.md`, `testing.md`, and `design.md` if `companions.design` is
   enabled), a registry entry, and auto-detects related source files.
5. Open the newly created `specs/<module-name>/<module-name>.spec.md` and fill
   in the `Purpose`, `Requirements`, and `Public API` sections. If a free-text
   description was given in step 2, use it directly to draft these sections —
   ask clarifying questions if it's underspecified, but do not leave the
   sections as unfilled placeholder text. Do the same for `requirements.md`
   (acceptance criteria) and `tasks.md` (initial task breakdown), if present.
6. Run `specsync check` to confirm the new spec passes validation."#;

const CREATE_SPEC_STEPS_TOML: &str = r#"1. Read the complete arguments above. Remove each standalone --minimal flag
   (in any position) and remember that minimal mode was requested. Preserve
   the complete remaining input for classification; do not extract a module
   name yet. If nothing remains, ask the user for a module or description.
2. Classify the complete remaining input as one of:
   - A bare module name - the entire input is one identifier with no
     whitespace, such as auth-service or billing. Use it as-is.
   - A free-text feature description - any quoted or unquoted sentence or
     phrase describing what to build, e.g. "I want a feature that lets users
     export their data as CSV". Only after making this classification, invent
     a short, kebab-case module name that captures the idea (e.g. csv-export).
     Never use only the first word as the module name. If the right name is
     ambiguous, ask the user to confirm or rename it before continuing. Keep
     the complete description at hand - you'll use it in step 5.
   Flag position does not change classification:
   - --minimal billing and billing --minimal both select the bare module
     billing in minimal mode.
   - --minimal I need CSV export and I need CSV export --minimal both keep the
     complete description and derive a name such as csv-export, not I.
3. If minimal mode was requested, run:
   specsync new <module-name>
   This creates a minimal spec only (no companion files).
4. Otherwise (default), run:
   specsync scaffold <module-name>
   This creates the spec, companion files (tasks.md, requirements.md,
   context.md, testing.md, and design.md if companions.design is enabled),
   a registry entry, and auto-detects related source files.
5. Open the newly created specs/<module-name>/<module-name>.spec.md and fill
   in the Purpose, Requirements, and Public API sections. If a free-text
   description was given in step 2, use it directly to draft these sections -
   ask clarifying questions if it's underspecified, but do not leave the
   sections as unfilled placeholder text. Do the same for requirements.md
   (acceptance criteria) and tasks.md (initial task breakdown), if present.
6. Run specsync check to confirm the new spec passes validation."#;

const CREATE_SPEC_DESCRIPTION: &str = "Scaffold a new spec-sync module spec from a module name or a natural-language feature description (full scaffold by default, or minimal with --minimal)";

const CREATE_CHANGE_DESCRIPTION: &str =
    "Create and guide a verified spec-sync SDD change through its deterministic interview";

const CREATE_CHANGE_STEPS_MD: &str = r#"1. Run `specsync change new "$ARGUMENTS" --json`.
2. Read the returned `questions` array and interview the user one question at a time.
3. Record each answer with `specsync change answer <id> <question-id> "<answer>" --json`.
4. Continue until the question list is empty, then show the selected artifacts and next action.
5. Do not approve, implement, verify, accept, or archive until the corresponding human gate or work stage is reached.
6. After implementation, run scoped verification with `specsync change check <id>` (or `/specsync:check`). Use `specsync change audit` only for active-workspace project health — never expect check to rewalk archived terminal evidence."#;

const CHECK_CHANGE_DESCRIPTION: &str =
    "Run scoped SpecSync change verification for one change (materialize deltas + targeted tests)";

const CHECK_CHANGE_STEPS_MD: &str = r#"1. Prefer `specsync change check $ARGUMENTS` when an id or partial id is provided; otherwise run `specsync change check`.
2. Expect **scoped** verification only — this change's materialization and verification commands. Do not run a full archive integrity walk.
3. Stream/wait for exit. On success, follow the printed **Next:** action (review, PR, or finalize path).
4. Do **not** run `specsync change audit` unless the user asked for project health over active workspaces and living specs."#;

const AUDIT_CHANGE_DESCRIPTION: &str =
    "Audit active SpecSync change workspaces and living specs (not archive history)";

const AUDIT_CHANGE_STEPS_MD: &str = r#"1. Run `specsync change audit`.
2. Report active-workspace and living-spec issues only. Archives are history — do not re-validate every archived CHG's terminal evidence.
3. Use this for "is the SDD workspace healthy?" not "did my feature tests pass?" (that is `change check` / `/specsync:check`)."#;

const SKILL_TRIGGER_DESCRIPTION: &str = "Keep markdown module specs in specs/<module>/ synchronized with source code using spec-sync. Use this whenever creating, editing, or reviewing code in a module that has (or should have) a spec, or whenever the user mentions specs, spec-sync, companion files (tasks.md/requirements.md/context.md/testing.md/design.md), or asks to add/update a module's documentation.";
const AGENT_ARTIFACT_MANIFEST_VERSION: u32 = 1;
const AGENT_ARTIFACT_TEMPLATE_VERSION: u32 = 3;
const AGENT_ARTIFACT_MANIFEST_PATH: &str = ".specsync/agent-artifacts.json";

// `.specsync/agent-artifacts.json` is committed and shared, and `load_agent_artifact_manifest`
// hard-errors rather than rebuilding, so `deny_unknown_fields` here was a 6.x lockout in a
// team-shared file: a manifest written by a newer 6.x would stop `agents install` and `init`
// dead for everyone still on the older binary. The three known fields stay required — nothing
// here fails open — the struct only stops refusing fields it does not need.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AgentArtifactRecord {
    tool: String,
    template_version: u32,
    digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AgentArtifactManifest {
    version: u32,
    artifacts: BTreeMap<String, AgentArtifactRecord>,
}

impl Default for AgentArtifactManifest {
    fn default() -> Self {
        Self {
            version: AGENT_ARTIFACT_MANIFEST_VERSION,
            artifacts: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
struct GeneratedAgentArtifact {
    manifest_key: String,
    tool: AgentTool,
    path: PathBuf,
    content: Vec<u8>,
}

/// AI coding tools that receive native skill/command file installation, as
/// opposed to the prose-instruction-file hooks in `hooks.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTool {
    Claude,
    Cursor,
    Codex,
    Gemini,
}

impl AgentTool {
    pub fn all() -> &'static [AgentTool] {
        &[
            AgentTool::Claude,
            AgentTool::Cursor,
            AgentTool::Codex,
            AgentTool::Gemini,
        ]
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            AgentTool::Claude => "claude",
            AgentTool::Cursor => "cursor",
            AgentTool::Codex => "codex",
            AgentTool::Gemini => "gemini",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AgentTool::Claude => "Claude Code SDD skill + specsync commands",
            AgentTool::Cursor => "Cursor SDD skill + specsync commands",
            AgentTool::Codex => "Codex CLI SDD skill (project-scoped)",
            AgentTool::Gemini => "Gemini CLI SDD skill + specsync commands",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Some(AgentTool::Claude),
            "cursor" => Some(AgentTool::Cursor),
            "codex" => Some(AgentTool::Codex),
            "gemini" => Some(AgentTool::Gemini),
            _ => None,
        }
    }

    /// Directory (relative to project root) containing this tool's skill
    /// folder, e.g. `.claude/skills/spec-sync/`.
    fn skill_dir(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(root.join(".claude").join("skills").join("spec-sync")),
            AgentTool::Cursor => Some(root.join(".cursor").join("skills").join("spec-sync")),
            AgentTool::Codex => Some(root.join(".codex").join("skills").join("spec-sync")),
            AgentTool::Gemini => Some(root.join(".gemini").join("skills").join("spec-sync")),
        }
    }

    /// Full path to this tool's `create-spec` command file. `None` if this
    /// tool has no project-local command mechanism (Codex).
    fn command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("create-spec.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-create-spec.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("create-spec.toml"),
            ),
            AgentTool::Codex => None,
        }
    }

    fn change_command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("create-change.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-create-change.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("create-change.toml"),
            ),
            AgentTool::Codex => None,
        }
    }

    fn check_command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("check.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-check.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("check.toml"),
            ),
            AgentTool::Codex => None,
        }
    }

    fn audit_command_path(&self, root: &Path) -> Option<PathBuf> {
        match self {
            AgentTool::Claude => Some(
                root.join(".claude")
                    .join("commands")
                    .join("specsync")
                    .join("audit.md"),
            ),
            AgentTool::Cursor => Some(
                root.join(".cursor")
                    .join("commands")
                    .join("specsync-audit.md"),
            ),
            AgentTool::Gemini => Some(
                root.join(".gemini")
                    .join("commands")
                    .join("specsync")
                    .join("audit.toml"),
            ),
            AgentTool::Codex => None,
        }
    }

    fn command_paths(&self, root: &Path) -> Vec<PathBuf> {
        [
            self.command_path(root),
            self.change_command_path(root),
            self.check_command_path(root),
            self.audit_command_path(root),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// True if every artifact this tool should have (skill dir and/or command
/// file) already exists.
pub fn is_installed(root: &Path, tool: AgentTool) -> bool {
    let skill_ok = match tool.skill_dir(root) {
        Some(dir) => dir.join("SKILL.md").exists(),
        None => true,
    };
    let command_ok = tool.command_paths(root).iter().all(|path| path.exists());
    skill_ok && command_ok
}

fn agent_artifact_manifest_path(root: &Path) -> PathBuf {
    root.join(AGENT_ARTIFACT_MANIFEST_PATH)
}

fn load_agent_artifact_manifest(root: &Path) -> Result<AgentArtifactManifest, String> {
    let path = agent_artifact_manifest_path(root);
    let content = match fs::read(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Ok(AgentArtifactManifest::default());
        }
        Err(error) => {
            return Err(format!("Failed to read {}: {error}", path.display()));
        }
    };
    let manifest: AgentArtifactManifest = serde_json::from_slice(&content)
        .map_err(|error| format!("Failed to parse {}: {error}", path.display()))?;
    if manifest.version != AGENT_ARTIFACT_MANIFEST_VERSION {
        return Err(format!(
            "Unsupported agent artifact manifest version {} in {}; expected {}",
            manifest.version,
            path.display(),
            AGENT_ARTIFACT_MANIFEST_VERSION
        ));
    }
    for (artifact, record) in &manifest.artifacts {
        if !AgentTool::all()
            .iter()
            .any(|tool| tool.name() == record.tool)
        {
            return Err(format!(
                "Unknown agent tool `{}` for {artifact} in {}",
                record.tool,
                path.display()
            ));
        }
        if record.template_version == 0 {
            return Err(format!(
                "Invalid template version for {artifact} in {}",
                path.display()
            ));
        }
        if record.digest.len() != 64 || !record.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "Invalid SHA-256 digest for {artifact} in {}",
                path.display()
            ));
        }
    }
    Ok(manifest)
}

fn write_agent_artifact_manifest(
    root: &Path,
    manifest: &AgentArtifactManifest,
) -> Result<(), String> {
    let path = agent_artifact_manifest_path(root);
    let parent = path
        .parent()
        .ok_or_else(|| format!("Invalid agent artifact manifest path: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    let mut content = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("Failed to serialize {}: {error}", path.display()))?;
    content.push(b'\n');
    fs::write(&path, content)
        .map_err(|error| format!("Failed to write {}: {error}", path.display()))
}

fn artifact_manifest_key(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!(
            "Generated agent artifact {} is outside project root {}: {error}",
            path.display(),
            root.display()
        )
    })?;
    relative
        .iter()
        .map(|component| {
            component.to_str().ok_or_else(|| {
                format!(
                    "Generated agent artifact path is not valid UTF-8: {}",
                    path.display()
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|components| components.join("/"))
}

fn generated_agent_artifacts(
    root: &Path,
    tool: AgentTool,
) -> Result<Vec<GeneratedAgentArtifact>, String> {
    let mut artifacts = Vec::new();

    if let Some(dir) = tool.skill_dir(root) {
        let path = dir.join("SKILL.md");
        artifacts.push(GeneratedAgentArtifact {
            manifest_key: artifact_manifest_key(root, &path)?,
            tool,
            path,
            content: skill_md_content(tool).into_bytes(),
        });
    }

    if let Some(path) = tool.command_path(root) {
        artifacts.push(GeneratedAgentArtifact {
            manifest_key: artifact_manifest_key(root, &path)?,
            tool,
            path,
            content: create_spec_command_content(tool).into_bytes(),
        });
    }

    if let Some(path) = tool.change_command_path(root) {
        artifacts.push(GeneratedAgentArtifact {
            manifest_key: artifact_manifest_key(root, &path)?,
            tool,
            path,
            content: create_change_command_content(tool).into_bytes(),
        });
    }

    if let Some(path) = tool.check_command_path(root) {
        artifacts.push(GeneratedAgentArtifact {
            manifest_key: artifact_manifest_key(root, &path)?,
            tool,
            path,
            content: check_change_command_content(tool).into_bytes(),
        });
    }

    if let Some(path) = tool.audit_command_path(root) {
        artifacts.push(GeneratedAgentArtifact {
            manifest_key: artifact_manifest_key(root, &path)?,
            tool,
            path,
            content: audit_change_command_content(tool).into_bytes(),
        });
    }

    Ok(artifacts)
}

fn content_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

fn generated_artifact_record(
    artifact: &GeneratedAgentArtifact,
    digest: String,
) -> AgentArtifactRecord {
    AgentArtifactRecord {
        tool: artifact.tool.name().to_string(),
        template_version: AGENT_ARTIFACT_TEMPLATE_VERSION,
        digest,
    }
}

fn recorded_artifact_matches(
    record: &AgentArtifactRecord,
    artifact: &GeneratedAgentArtifact,
    digest: &str,
) -> bool {
    record.tool == artifact.tool.name() && record.digest.eq_ignore_ascii_case(digest)
}

fn write_generated_agent_artifact(artifact: &GeneratedAgentArtifact) -> Result<(), String> {
    let parent = artifact.path.parent().ok_or_else(|| {
        format!(
            "Invalid generated agent artifact path: {}",
            artifact.path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    fs::write(&artifact.path, &artifact.content)
        .map_err(|error| format!("Failed to write {}: {error}", artifact.path.display()))
}

/// Install skill + command files for one tool. Returns Ok(true) if any agent
/// artifact was written, Ok(false) if every artifact was already current.
///
/// A versioned SHA-256 manifest records the exact bytes last generated by
/// spec-sync. Re-running after an upgrade replaces stale content only when its
/// current digest still matches that record. Differing or untracked content is
/// reported as a conflict and left untouched, including non-UTF-8 content.
pub fn install_agent(root: &Path, tool: AgentTool) -> Result<bool, String> {
    let manifest = load_agent_artifact_manifest(root)?;
    let mut updated_manifest = manifest.clone();
    let artifacts = generated_agent_artifacts(root, tool)?;
    let mut pending_writes = Vec::new();
    let mut conflicts = Vec::new();

    for artifact in &artifacts {
        let desired_digest = content_digest(&artifact.content);
        match fs::read(&artifact.path) {
            Ok(existing) if existing == artifact.content => {
                updated_manifest.artifacts.insert(
                    artifact.manifest_key.clone(),
                    generated_artifact_record(artifact, desired_digest),
                );
            }
            Ok(existing) => {
                let existing_digest = content_digest(&existing);
                match manifest.artifacts.get(&artifact.manifest_key) {
                    Some(recorded)
                        if recorded_artifact_matches(recorded, artifact, &existing_digest) =>
                    {
                        pending_writes.push(artifact);
                        updated_manifest.artifacts.insert(
                            artifact.manifest_key.clone(),
                            generated_artifact_record(artifact, desired_digest),
                        );
                    }
                    Some(_) => conflicts.push(format!(
                        "{} (content differs from the recorded generated digest)",
                        artifact.path.display()
                    )),
                    None => conflicts.push(format!(
                        "{} (no trusted generated digest is recorded)",
                        artifact.path.display()
                    )),
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                pending_writes.push(artifact);
                updated_manifest.artifacts.insert(
                    artifact.manifest_key.clone(),
                    generated_artifact_record(artifact, desired_digest),
                );
            }
            Err(error) => {
                return Err(format!(
                    "Failed to inspect generated agent artifact {}: {error}",
                    artifact.path.display()
                ));
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(format!(
            "Refusing to overwrite customized generated agent artifact(s):\n  - {}\n\
             Existing files were left unchanged; reconcile or remove the conflicts and retry.",
            conflicts.join("\n  - ")
        ));
    }

    for artifact in &pending_writes {
        write_generated_agent_artifact(artifact)?;
    }

    let manifest_changed = updated_manifest != manifest;
    if manifest_changed {
        write_agent_artifact_manifest(root, &updated_manifest)?;
    }

    Ok(!pending_writes.is_empty())
}

/// Uninstall a tool's generated artifacts. Returns Ok(true) if anything was
/// removed, Ok(false) if nothing was present.
///
/// Every existing artifact must still match its recorded generated digest.
/// Customized or untracked files are reported together and left untouched.
pub fn uninstall_agent(root: &Path, tool: AgentTool) -> Result<bool, String> {
    let manifest = load_agent_artifact_manifest(root)?;
    let mut updated_manifest = manifest.clone();
    let artifacts = generated_agent_artifacts(root, tool)?;
    let mut pending_removals = Vec::new();
    let mut conflicts = Vec::new();

    for artifact in &artifacts {
        match fs::read(&artifact.path) {
            Ok(existing) => {
                let existing_digest = content_digest(&existing);
                match manifest.artifacts.get(&artifact.manifest_key) {
                    Some(recorded)
                        if recorded_artifact_matches(recorded, artifact, &existing_digest) =>
                    {
                        pending_removals.push(artifact);
                    }
                    Some(_) => conflicts.push(format!(
                        "{} (content differs from the recorded generated digest)",
                        artifact.path.display()
                    )),
                    None => conflicts.push(format!(
                        "{} (no trusted generated digest is recorded)",
                        artifact.path.display()
                    )),
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "Failed to inspect generated agent artifact {}: {error}",
                    artifact.path.display()
                ));
            }
        }
    }

    if !conflicts.is_empty() {
        return Err(format!(
            "Refusing to remove customized generated agent artifact(s):\n  - {}\n\
             Existing files were left unchanged; reconcile or remove the conflicts and retry.",
            conflicts.join("\n  - ")
        ));
    }

    for artifact in &pending_removals {
        fs::remove_file(&artifact.path)
            .map_err(|error| format!("Failed to remove {}: {error}", artifact.path.display()))?;
        updated_manifest.artifacts.remove(&artifact.manifest_key);
        cleanup_generated_namespace(&artifact.path);
    }

    // Missing files no longer need stale ownership entries.
    for artifact in &artifacts {
        if !artifact.path.exists() {
            updated_manifest.artifacts.remove(&artifact.manifest_key);
        }
    }

    if updated_manifest != manifest {
        write_agent_artifact_manifest(root, &updated_manifest)?;
    }

    Ok(!pending_removals.is_empty())
}

// ─── Content builders ─────────────────────────────────────────────────────

fn skill_md_content(_tool: AgentTool) -> String {
    format!(
        "---\nname: spec-sync\ndescription: {SKILL_TRIGGER_DESCRIPTION}\n---\n\n# Spec-Sync Workflow\n\nThis project uses [spec-sync](https://github.com/CorvidLabs/spec-sync) for bidirectional spec-to-code validation. Specs live in `specs/<module>/<module>.spec.md`.\n\n{SKILL_BODY}",
    )
}

fn create_spec_command_content(tool: AgentTool) -> String {
    match tool {
        AgentTool::Claude => claude_create_spec_md(),
        AgentTool::Cursor => cursor_create_spec_md(),
        AgentTool::Gemini => gemini_create_spec_toml(),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn create_change_command_content(tool: AgentTool) -> String {
    let steps = match tool {
        AgentTool::Gemini => CREATE_CHANGE_STEPS_MD.replace("$ARGUMENTS", "{{args}}"),
        _ => CREATE_CHANGE_STEPS_MD.to_string(),
    };

    match tool {
        AgentTool::Claude => format!(
            "---\ndescription: {CREATE_CHANGE_DESCRIPTION}\nargument-hint: <change-description>\n---\n\n{steps}\n"
        ),
        AgentTool::Cursor => {
            format!("Create a verified spec-sync SDD change.\n\nArguments: $ARGUMENTS\n\n{steps}\n")
        }
        AgentTool::Gemini => format!(
            "description = \"{CREATE_CHANGE_DESCRIPTION}\"\n\nprompt = \"\"\"\nArguments: {{{{args}}}}\n\n{steps}\n\"\"\"\n"
        ),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn check_change_command_content(tool: AgentTool) -> String {
    let steps = match tool {
        AgentTool::Gemini => CHECK_CHANGE_STEPS_MD.replace("$ARGUMENTS", "{{args}}"),
        _ => CHECK_CHANGE_STEPS_MD.to_string(),
    };

    match tool {
        AgentTool::Claude => format!(
            "---\ndescription: {CHECK_CHANGE_DESCRIPTION}\nargument-hint: [change-id]\n---\n\nRun scoped SpecSync change verification.\n\nArguments: `$ARGUMENTS`\n\n{steps}\n"
        ),
        AgentTool::Cursor => format!(
            "Run scoped SpecSync change verification.\n\nArguments: $ARGUMENTS\n\n{steps}\n"
        ),
        AgentTool::Gemini => format!(
            "description = \"{CHECK_CHANGE_DESCRIPTION}\"\n\nprompt = \"\"\"\nRun scoped SpecSync change verification.\n\nArguments: {{{{args}}}}\n\n{steps}\n\"\"\"\n"
        ),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn audit_change_command_content(tool: AgentTool) -> String {
    let steps = AUDIT_CHANGE_STEPS_MD;

    match tool {
        AgentTool::Claude => format!(
            "---\ndescription: {AUDIT_CHANGE_DESCRIPTION}\n---\n\nAudit active SpecSync change workspaces and living specs.\n\n{steps}\n"
        ),
        AgentTool::Cursor => {
            format!("Audit active SpecSync change workspaces and living specs.\n\n{steps}\n")
        }
        AgentTool::Gemini => format!(
            "description = \"{AUDIT_CHANGE_DESCRIPTION}\"\n\nprompt = \"\"\"\nAudit active SpecSync change workspaces and living specs.\n\n{steps}\n\"\"\"\n"
        ),
        AgentTool::Codex => unreachable!("Codex has no command file"),
    }
}

fn cleanup_command_namespace(path: &Path) {
    if let Some(parent) = path.parent() {
        let is_our_namespace_dir =
            parent.file_name().and_then(|name| name.to_str()) == Some("specsync");
        if is_our_namespace_dir
            && fs::read_dir(parent)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false)
        {
            let _ = fs::remove_dir(parent);
        }
    }
}

fn cleanup_generated_namespace(path: &Path) {
    cleanup_command_namespace(path);
    if let Some(parent) = path.parent() {
        let is_skill_dir = parent.file_name().and_then(|name| name.to_str()) == Some("spec-sync");
        if is_skill_dir
            && fs::read_dir(parent)
                .map(|mut entries| entries.next().is_none())
                .unwrap_or(false)
        {
            let _ = fs::remove_dir(parent);
        }
    }
}

fn claude_create_spec_md() -> String {
    format!(
        "---\ndescription: {CREATE_SPEC_DESCRIPTION}\nargument-hint: <module-name-or-description> [--minimal]\n---\n\nCreate a new spec-sync module spec.\n\nArguments: `$ARGUMENTS`\n\n{CREATE_SPEC_STEPS_MD}\n",
    )
}

fn cursor_create_spec_md() -> String {
    format!(
        "Create a new spec-sync module spec.\n\nArguments: $ARGUMENTS\n\n{CREATE_SPEC_STEPS_MD}\n",
    )
}

fn gemini_create_spec_toml() -> String {
    format!(
        "description = \"{CREATE_SPEC_DESCRIPTION}\"\n\nprompt = \"\"\"\nCreate a new spec-sync module spec.\n\nArguments: {{{{args}}}}\n\n{CREATE_SPEC_STEPS_TOML}\n\"\"\"\n",
    )
}

// ─── CLI command handlers ────────────────────────────────────────────────────

/// Install agent integrations for the specified tools (or all if empty).
pub fn cmd_install(root: &Path, targets: &[AgentTool]) {
    let targets = if targets.is_empty() {
        AgentTool::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Installing Agent Integrations".bold()
    );

    let mut installed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for target in &targets {
        match install_agent(root, *target) {
            Ok(true) => {
                println!("  {} Installed {}", "✓".green(), target.description());
                installed += 1;
            }
            Ok(false) => {
                println!(
                    "  {} Already installed: {}",
                    "·".dimmed(),
                    target.description()
                );
                skipped += 1;
            }
            Err(e) => {
                println!("  {} {}: {e}", "✗".red(), target.description());
                errors += 1;
            }
        }
    }

    println!();
    if installed > 0 {
        println!("{installed} tool(s) installed.");
    }
    if skipped > 0 {
        println!("{skipped} tool(s) already present.");
    }
    if errors > 0 {
        println!("{errors} tool(s) failed.");
        std::process::exit(1);
    }
}

/// Uninstall agent integrations for the specified tools (or all if empty).
pub fn cmd_uninstall(root: &Path, targets: &[AgentTool]) {
    let targets = if targets.is_empty() {
        AgentTool::all().to_vec()
    } else {
        targets.to_vec()
    };

    println!(
        "\n--- {} ------------------------------------------------",
        "Uninstalling Agent Integrations".bold()
    );

    let mut removed = 0;

    for target in &targets {
        match uninstall_agent(root, *target) {
            Ok(true) => {
                println!("  {} Removed {}", "✓".green(), target.description());
                removed += 1;
            }
            Ok(false) => {
                println!("  {} Not installed: {}", "·".dimmed(), target.description());
            }
            Err(e) => {
                println!("  {} {}: {e}", "!".yellow(), target.description());
            }
        }
    }

    println!();
    if removed > 0 {
        println!("{removed} tool(s) removed.");
    } else {
        println!("No agent integrations to remove.");
    }
}

/// Show installation status of all agent tools.
pub fn cmd_status(root: &Path) {
    println!(
        "\n--- {} ------------------------------------------------",
        "Agent Integration Status".bold()
    );

    for target in AgentTool::all() {
        let installed = is_installed(root, *target);
        let status = if installed {
            "installed".green().to_string()
        } else {
            "not installed".dimmed().to_string()
        };
        println!("  {:45} {}", target.description(), status);
    }

    println!();
    println!("Install all: specsync agents install");
    println!("Install one: specsync agents install --claude --gemini");
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn normalize_checkout_line_endings(content: &str) -> String {
        content.replace("\r\n", "\n")
    }

    // ── AgentTool::all / name / from_str ───────────────────────────

    #[test]
    fn agent_tool_all_returns_four_targets() {
        assert_eq!(AgentTool::all().len(), 4);
    }

    #[test]
    fn agent_tool_all_contains_all_variants() {
        let all = AgentTool::all();
        assert!(all.contains(&AgentTool::Claude));
        assert!(all.contains(&AgentTool::Cursor));
        assert!(all.contains(&AgentTool::Codex));
        assert!(all.contains(&AgentTool::Gemini));
    }

    #[test]
    fn agent_tool_name_returns_expected_strings() {
        assert_eq!(AgentTool::Claude.name(), "claude");
        assert_eq!(AgentTool::Cursor.name(), "cursor");
        assert_eq!(AgentTool::Codex.name(), "codex");
        assert_eq!(AgentTool::Gemini.name(), "gemini");
    }

    #[test]
    fn from_str_parses_all_targets() {
        assert_eq!(AgentTool::from_str("claude"), Some(AgentTool::Claude));
        assert_eq!(AgentTool::from_str("cursor"), Some(AgentTool::Cursor));
        assert_eq!(AgentTool::from_str("codex"), Some(AgentTool::Codex));
        assert_eq!(AgentTool::from_str("gemini"), Some(AgentTool::Gemini));
    }

    #[test]
    fn from_str_is_case_insensitive() {
        assert_eq!(AgentTool::from_str("CLAUDE"), Some(AgentTool::Claude));
        assert_eq!(AgentTool::from_str("Gemini"), Some(AgentTool::Gemini));
    }

    #[test]
    fn from_str_returns_none_for_unknown() {
        assert_eq!(AgentTool::from_str("unknown"), None);
        assert_eq!(AgentTool::from_str(""), None);
        assert_eq!(AgentTool::from_str("windsurf"), None);
    }

    // ── path correctness ────────────────────────────────────────────

    #[test]
    fn claude_paths_are_correct() {
        let tmp = setup();
        assert_eq!(
            AgentTool::Claude.skill_dir(tmp.path()).unwrap(),
            tmp.path().join(".claude/skills/spec-sync")
        );
        assert_eq!(
            AgentTool::Claude.command_path(tmp.path()).unwrap(),
            tmp.path().join(".claude/commands/specsync/create-spec.md")
        );
    }

    #[test]
    fn cursor_command_path_is_flat() {
        let tmp = setup();
        assert_eq!(
            AgentTool::Cursor.command_path(tmp.path()).unwrap(),
            tmp.path().join(".cursor/commands/specsync-create-spec.md")
        );
    }

    #[test]
    fn codex_has_no_command_path() {
        let tmp = setup();
        assert!(AgentTool::Codex.command_path(tmp.path()).is_none());
        assert!(AgentTool::Codex.skill_dir(tmp.path()).is_some());
    }

    #[test]
    fn gemini_has_both_skill_and_command_paths() {
        let tmp = setup();
        assert!(AgentTool::Gemini.skill_dir(tmp.path()).is_some());
        assert!(AgentTool::Gemini.command_path(tmp.path()).is_some());
    }

    // ── is_installed ───────────────────────────────────────────────

    #[test]
    fn is_installed_returns_false_for_empty_dir() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(!is_installed(tmp.path(), *target));
        }
    }

    // ── install_agent ───────────────────────────────────────────────

    #[test]
    fn install_claude_creates_skill_and_command() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());

        let skill =
            fs::read_to_string(tmp.path().join(".claude/skills/spec-sync/SKILL.md")).unwrap();
        assert!(skill.starts_with("---\nname: spec-sync"));
        assert!(skill.contains("tasks.md"));
        assert!(skill.contains("specsync check"));

        let command =
            fs::read_to_string(tmp.path().join(".claude/commands/specsync/create-spec.md"))
                .unwrap();
        assert!(command.starts_with("---\ndescription:"));
        assert!(command.contains("$ARGUMENTS"));
        assert!(command.contains("specsync scaffold"));
        assert!(command.contains("specsync new"));
        let change_command = fs::read_to_string(
            tmp.path()
                .join(".claude/commands/specsync/create-change.md"),
        )
        .unwrap();
        assert!(change_command.contains("specsync change new"));
        assert!(skill.contains("specsync change check"));
        assert!(skill.contains("specsync change audit"));
        assert!(skill.contains("Lifecycle verbs"));

        let check_command =
            fs::read_to_string(tmp.path().join(".claude/commands/specsync/check.md")).unwrap();
        assert!(check_command.contains("specsync change check"));
        assert!(check_command.contains("scoped"));

        let audit_command =
            fs::read_to_string(tmp.path().join(".claude/commands/specsync/audit.md")).unwrap();
        assert!(audit_command.contains("specsync change audit"));
        assert!(audit_command.contains("active"));

        assert!(is_installed(tmp.path(), AgentTool::Claude));
    }

    #[test]
    fn install_cursor_command_has_no_frontmatter() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Cursor).unwrap());
        let command =
            fs::read_to_string(tmp.path().join(".cursor/commands/specsync-create-spec.md"))
                .unwrap();
        assert!(!command.starts_with("---"));
        assert!(command.contains("$ARGUMENTS"));
        assert!(
            tmp.path()
                .join(".cursor/commands/specsync-create-change.md")
                .exists()
        );
        assert!(
            tmp.path()
                .join(".cursor/commands/specsync-check.md")
                .exists()
        );
        assert!(
            tmp.path()
                .join(".cursor/commands/specsync-audit.md")
                .exists()
        );
    }

    #[test]
    fn install_codex_creates_skill_only() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Codex).unwrap());
        assert!(tmp.path().join(".codex/skills/spec-sync/SKILL.md").exists());
        assert!(!tmp.path().join(".codex/commands").exists());
        assert!(is_installed(tmp.path(), AgentTool::Codex));
    }

    #[test]
    fn install_gemini_creates_skill_and_command() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Gemini).unwrap());
        assert!(
            tmp.path()
                .join(".gemini/skills/spec-sync/SKILL.md")
                .exists()
        );

        let path = tmp
            .path()
            .join(".gemini/commands/specsync/create-spec.toml");
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("prompt = \"\"\""));
        assert!(content.contains("description ="));
        assert!(content.contains("{{args}}"));
        // Triple-quoted block must be balanced.
        assert_eq!(content.matches("\"\"\"").count(), 2);

        assert!(is_installed(tmp.path(), AgentTool::Gemini));
    }

    #[test]
    fn create_spec_commands_classify_the_complete_remaining_input() {
        let tmp = setup();

        for tool in [AgentTool::Claude, AgentTool::Cursor, AgentTool::Gemini] {
            install_agent(tmp.path(), tool).unwrap();
            let path = tool.command_path(tmp.path()).unwrap();
            let content = fs::read_to_string(path).unwrap();

            assert!(content.contains("Remove each standalone"));
            assert!(content.contains("the complete remaining input for classification"));
            assert!(content.contains("the entire input is one identifier with no"));
            assert!(content.contains("any quoted or unquoted sentence or"));
            assert!(content.contains("Never use only the first word as the module name"));
            assert!(content.contains("--minimal billing"));
            assert!(content.contains("billing --minimal"));
            assert!(content.contains("--minimal I need CSV export"));
            assert!(content.contains("I need CSV export --minimal"));
            assert!(content.contains("csv-export"));
            assert!(content.contains("not `I`") || content.contains("not I"));
            assert!(!content.contains("first whitespace-separated token"));
        }
    }

    #[test]
    fn checked_in_create_spec_commands_match_generated_assets() {
        let tmp = setup();
        let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        if !manifest_root.join(".git").exists()
            && !manifest_root
                .join(".claude/commands/specsync/create-spec.md")
                .exists()
        {
            // Published crate archives intentionally omit repository-only agent fixtures.
            return;
        }
        let fixtures = [
            (
                AgentTool::Claude,
                ".claude/commands/specsync/create-spec.md",
            ),
            (
                AgentTool::Cursor,
                ".cursor/commands/specsync-create-spec.md",
            ),
            (
                AgentTool::Gemini,
                ".gemini/commands/specsync/create-spec.toml",
            ),
        ];

        for (tool, relative_path) in fixtures {
            let checked_in = fs::read_to_string(manifest_root.join(relative_path)).unwrap();
            assert!(install_agent(tmp.path(), tool).unwrap());
            let generated = fs::read_to_string(tool.command_path(tmp.path()).unwrap()).unwrap();
            assert_eq!(
                generated,
                normalize_checkout_line_endings(&checked_in),
                "{} command drifted",
                tool.name()
            );
            assert!(!install_agent(tmp.path(), tool).unwrap());
        }
    }

    #[test]
    fn checked_in_asset_parity_normalizes_windows_checkout_line_endings() {
        assert_eq!(
            normalize_checkout_line_endings("first\r\nsecond\r\n"),
            "first\nsecond\n"
        );
    }

    #[test]
    fn gemini_create_change_uses_native_args_and_quotes_answers() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Gemini).unwrap();

        let path = AgentTool::Gemini.change_command_path(tmp.path()).unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(content.contains("specsync change new \"{{args}}\" --json"));
        assert!(content.contains("<question-id> \"<answer>\" --json"));
        assert!(!content.contains("$ARGUMENTS"));
    }

    #[test]
    fn every_generated_lifecycle_surface_quotes_free_text_answers() {
        let tmp = setup();

        for tool in AgentTool::all() {
            install_agent(tmp.path(), *tool).unwrap();
            let skill =
                fs::read_to_string(tool.skill_dir(tmp.path()).unwrap().join("SKILL.md")).unwrap();
            assert!(skill.contains("<question-id> \"<answer>\" --json"));

            if let Some(path) = tool.change_command_path(tmp.path()) {
                let command = fs::read_to_string(path).unwrap();
                assert!(command.contains("<question-id> \"<answer>\" --json"));
            }
        }
    }

    fn error_mentions_path(error: &str, path: &Path) -> bool {
        let native = path.display().to_string();
        let forward = native.replace('\\', "/");
        let back = native.replace('/', "\\");
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        error.contains(&native)
            || error.contains(&forward)
            || error.contains(&back)
            || (!file_name.is_empty() && error.contains(file_name))
    }

    #[test]
    fn reinstall_keeps_all_generated_artifacts_byte_identical() {
        let tmp = setup();

        for tool in AgentTool::all() {
            assert!(install_agent(tmp.path(), *tool).unwrap());
            let mut paths = vec![tool.skill_dir(tmp.path()).unwrap().join("SKILL.md")];
            paths.extend(tool.command_path(tmp.path()));
            paths.extend(tool.change_command_path(tmp.path()));
            let before = paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>();

            assert!(!install_agent(tmp.path(), *tool).unwrap());
            let after = paths
                .iter()
                .map(|path| fs::read(path).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(before, after);
        }
    }

    #[test]
    fn install_is_idempotent() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(install_agent(tmp.path(), *target).unwrap());
            assert!(!install_agent(tmp.path(), *target).unwrap());
        }
    }

    #[test]
    fn install_records_versioned_digests_for_generated_artifacts() {
        let tmp = setup();
        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());

        let manifest_path = agent_artifact_manifest_path(tmp.path());
        assert!(manifest_path.exists());
        let manifest = load_agent_artifact_manifest(tmp.path()).unwrap();
        assert_eq!(manifest.version, AGENT_ARTIFACT_MANIFEST_VERSION);

        let artifacts = generated_agent_artifacts(tmp.path(), AgentTool::Claude).unwrap();
        assert_eq!(manifest.artifacts.len(), artifacts.len());
        for artifact in artifacts {
            let record = manifest.artifacts.get(&artifact.manifest_key).unwrap();
            assert_eq!(record.tool, AgentTool::Claude.name());
            assert_eq!(record.template_version, AGENT_ARTIFACT_TEMPLATE_VERSION);
            assert_eq!(record.digest, content_digest(&artifact.content));
        }
    }

    #[test]
    fn install_preserves_untracked_stale_skill_as_conflict() {
        let tmp = setup();
        let skill_path = tmp.path().join(".claude/skills/spec-sync/SKILL.md");
        fs::create_dir_all(skill_path.parent().unwrap()).unwrap();
        let stale = b"stale or customized skill without trusted ownership metadata";
        fs::write(&skill_path, stale).unwrap();

        let error = install_agent(tmp.path(), AgentTool::Claude).unwrap_err();

        assert!(error.contains("Refusing to overwrite customized"));
        assert!(
            error_mentions_path(&error, &skill_path),
            "error should mention skill path: {error}"
        );
        assert!(error.contains("no trusted generated digest"));
        assert_eq!(fs::read(&skill_path).unwrap(), stale);
        assert!(!tmp.path().join(".claude/commands").exists());
        assert!(!agent_artifact_manifest_path(tmp.path()).exists());
    }

    #[test]
    fn install_preserves_untracked_stale_command_as_conflict() {
        let tmp = setup();
        let command_path = tmp.path().join(".claude/commands/specsync/create-spec.md");
        fs::create_dir_all(command_path.parent().unwrap()).unwrap();
        let stale = b"stale or customized command without trusted ownership metadata";
        fs::write(&command_path, stale).unwrap();

        let error = install_agent(tmp.path(), AgentTool::Claude).unwrap_err();

        assert!(
            error_mentions_path(&error, &command_path),
            "error should mention command path: {error}"
        );
        assert_eq!(fs::read(&command_path).unwrap(), stale);
        assert!(!tmp.path().join(".claude/skills/spec-sync").exists());
    }

    #[test]
    fn manifest_allows_unmodified_generated_artifact_upgrade() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        let skill_path = tmp.path().join(".claude/skills/spec-sync/SKILL.md");
        let old_generated_content = b"older generated skill content";
        fs::write(&skill_path, old_generated_content).unwrap();

        let mut manifest = load_agent_artifact_manifest(tmp.path()).unwrap();
        let manifest_key = artifact_manifest_key(tmp.path(), &skill_path).unwrap();
        manifest.artifacts.insert(
            manifest_key.clone(),
            AgentArtifactRecord {
                tool: AgentTool::Claude.name().to_string(),
                template_version: AGENT_ARTIFACT_TEMPLATE_VERSION - 1,
                digest: content_digest(old_generated_content),
            },
        );
        write_agent_artifact_manifest(tmp.path(), &manifest).unwrap();

        assert!(install_agent(tmp.path(), AgentTool::Claude).unwrap());
        let expected = skill_md_content(AgentTool::Claude).into_bytes();
        assert_eq!(fs::read(&skill_path).unwrap(), expected);

        let upgraded_manifest = load_agent_artifact_manifest(tmp.path()).unwrap();
        let upgraded = upgraded_manifest.artifacts.get(&manifest_key).unwrap();
        assert_eq!(upgraded.tool, AgentTool::Claude.name());
        assert_eq!(upgraded.template_version, AGENT_ARTIFACT_TEMPLATE_VERSION);
        assert_eq!(upgraded.digest, content_digest(&expected));
    }

    #[test]
    fn install_reports_and_preserves_all_customized_artifacts() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        let skill_path = tmp.path().join(".claude/skills/spec-sync/SKILL.md");
        let command_path = tmp.path().join(".claude/commands/specsync/create-spec.md");
        let customized_skill = b"customized skill";
        let customized_command = b"customized command";
        fs::write(&skill_path, customized_skill).unwrap();
        fs::write(&command_path, customized_command).unwrap();
        let manifest_before = fs::read(agent_artifact_manifest_path(tmp.path())).unwrap();

        let error = install_agent(tmp.path(), AgentTool::Claude).unwrap_err();

        assert!(
            error_mentions_path(&error, &skill_path),
            "error should mention skill path: {error}"
        );
        assert!(
            error_mentions_path(&error, &command_path),
            "error should mention command path: {error}"
        );
        assert!(error.contains("content differs from the recorded generated digest"));
        assert_eq!(fs::read(&skill_path).unwrap(), customized_skill);
        assert_eq!(fs::read(&command_path).unwrap(), customized_command);
        assert_eq!(
            fs::read(agent_artifact_manifest_path(tmp.path())).unwrap(),
            manifest_before
        );
    }

    #[test]
    fn install_preserves_and_reports_non_utf8_customization() {
        let tmp = setup();
        let command_path = tmp.path().join(".cursor/commands/specsync-create-spec.md");
        fs::create_dir_all(command_path.parent().unwrap()).unwrap();
        let customized = [0xff, 0xfe, 0x00, b'C', b'U', b'S', b'T', b'O', b'M'];
        fs::write(&command_path, customized).unwrap();

        let error = install_agent(tmp.path(), AgentTool::Cursor).unwrap_err();

        assert!(
            error_mentions_path(&error, &command_path),
            "error should mention command path: {error}"
        );
        assert!(error.contains("no trusted generated digest"));
        assert_eq!(fs::read(&command_path).unwrap(), customized);
        assert!(!tmp.path().join(".cursor/skills/spec-sync").exists());
        assert!(!agent_artifact_manifest_path(tmp.path()).exists());
    }

    #[test]
    fn install_adopts_matching_legacy_artifacts_without_rewriting_them() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Cursor).unwrap();
        let command_path = tmp.path().join(".cursor/commands/specsync-create-spec.md");
        let command_before = fs::read(&command_path).unwrap();
        fs::remove_file(agent_artifact_manifest_path(tmp.path())).unwrap();

        assert!(!install_agent(tmp.path(), AgentTool::Cursor).unwrap());
        assert_eq!(fs::read(&command_path).unwrap(), command_before);
        assert!(agent_artifact_manifest_path(tmp.path()).exists());
        assert!(!install_agent(tmp.path(), AgentTool::Cursor).unwrap());
    }

    #[test]
    fn install_does_not_rewrite_unchanged_content() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        // Second install: content is identical, so nothing should be rewritten.
        assert!(!install_agent(tmp.path(), AgentTool::Claude).unwrap());
    }

    // ── uninstall_agent ──────────────────────────────────────────────

    #[test]
    fn uninstall_returns_false_when_not_installed() {
        let tmp = setup();
        for target in AgentTool::all() {
            assert!(!uninstall_agent(tmp.path(), *target).unwrap());
        }
    }

    #[test]
    fn uninstall_claude_removes_skill_and_command() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Claude).unwrap());
        assert!(!tmp.path().join(".claude/skills/spec-sync").exists());
        assert!(!tmp.path().join(".claude/commands/specsync").exists());
        assert!(!is_installed(tmp.path(), AgentTool::Claude));
        assert!(
            load_agent_artifact_manifest(tmp.path())
                .unwrap()
                .artifacts
                .is_empty()
        );
    }

    #[test]
    fn uninstall_preserves_sibling_commands() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        let sibling = tmp.path().join(".claude/commands/other-command.md");
        fs::write(&sibling, "unrelated command").unwrap();

        assert!(uninstall_agent(tmp.path(), AgentTool::Claude).unwrap());

        assert!(!tmp.path().join(".claude/commands/specsync").exists());
        assert!(tmp.path().join(".claude/commands").exists());
        assert!(sibling.exists());
    }

    #[test]
    fn uninstall_cursor_flat_file_does_not_touch_commands_dir() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Cursor).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Cursor).unwrap());
        assert!(
            !tmp.path()
                .join(".cursor/commands/specsync-create-spec.md")
                .exists()
        );
        assert!(tmp.path().join(".cursor/commands").exists());
    }

    #[test]
    fn uninstall_gemini_removes_skill_and_command() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Gemini).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Gemini).unwrap());
        assert!(!tmp.path().join(".gemini/commands/specsync").exists());
        assert!(!tmp.path().join(".gemini/skills/spec-sync").exists());
    }

    #[test]
    fn uninstall_codex_removes_skill_only() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Codex).unwrap();
        assert!(uninstall_agent(tmp.path(), AgentTool::Codex).unwrap());
        assert!(!tmp.path().join(".codex/skills/spec-sync").exists());
    }

    #[test]
    fn uninstall_preserves_all_customized_artifacts_without_partial_removal() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Claude).unwrap();
        let skill_path = tmp.path().join(".claude/skills/spec-sync/SKILL.md");
        let command_path = tmp.path().join(".claude/commands/specsync/create-spec.md");
        let change_path = tmp
            .path()
            .join(".claude/commands/specsync/create-change.md");
        let customized = b"customized generated skill";
        fs::write(&skill_path, customized).unwrap();
        let command_before = fs::read(&command_path).unwrap();
        let change_before = fs::read(&change_path).unwrap();
        let manifest_before = fs::read(agent_artifact_manifest_path(tmp.path())).unwrap();

        let error = uninstall_agent(tmp.path(), AgentTool::Claude).unwrap_err();

        assert!(error.contains("Refusing to remove customized"));
        assert!(
            error_mentions_path(&error, &skill_path),
            "error should mention skill path: {error}"
        );
        assert_eq!(fs::read(&skill_path).unwrap(), customized);
        assert_eq!(fs::read(&command_path).unwrap(), command_before);
        assert_eq!(fs::read(&change_path).unwrap(), change_before);
        assert_eq!(
            fs::read(agent_artifact_manifest_path(tmp.path())).unwrap(),
            manifest_before
        );
    }

    #[test]
    fn uninstall_preserves_untracked_artifact_and_user_skill_sibling() {
        let tmp = setup();
        let skill_dir = tmp.path().join(".codex/skills/spec-sync");
        let skill_path = skill_dir.join("SKILL.md");
        let sibling = skill_dir.join("notes.md");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(&skill_path, "untracked skill").unwrap();
        fs::write(&sibling, "user notes").unwrap();

        let error = uninstall_agent(tmp.path(), AgentTool::Codex).unwrap_err();

        assert!(error.contains("no trusted generated digest"));
        assert!(skill_path.exists());
        assert_eq!(fs::read_to_string(&sibling).unwrap(), "user notes");
    }

    #[test]
    fn uninstall_preserves_user_files_inside_generated_skill_directory() {
        let tmp = setup();
        install_agent(tmp.path(), AgentTool::Codex).unwrap();
        let skill_dir = tmp.path().join(".codex/skills/spec-sync");
        let sibling = skill_dir.join("notes.md");
        fs::write(&sibling, "user notes").unwrap();

        assert!(uninstall_agent(tmp.path(), AgentTool::Codex).unwrap());

        assert!(!skill_dir.join("SKILL.md").exists());
        assert_eq!(fs::read_to_string(&sibling).unwrap(), "user notes");
        assert!(skill_dir.exists());
    }

    /// A manifest written by a newer 6.x must not brick `agents install` for everyone else.
    ///
    /// `.specsync/agent-artifacts.json` is committed, so one teammate on a later 6.x adding a
    /// field used to hard-fail the command for every teammate still on the older binary, in a
    /// file they all share. Read tolerance is the whole fix; the three fields this binary needs
    /// are still required, so nothing here starts accepting a manifest it cannot use.
    #[test]
    fn a_manifest_written_by_a_newer_six_is_still_usable() {
        let extended = serde_json::json!({
            "version": 1,
            "artifacts": {
                "claude:skill": {
                    "tool": "claude",
                    "template_version": 3,
                    "digest": "a".repeat(64),
                    "future_record_field": "written by 6.4"
                }
            },
            "future_manifest_field": {"nested": true}
        });
        let bytes = serde_json::to_vec(&extended).unwrap();
        let manifest: AgentArtifactManifest = serde_json::from_slice(bytes.as_slice())
            .expect("a manifest from a newer 6.x must still be readable");
        let record = manifest
            .artifacts
            .get("claude:skill")
            .expect("the fields this binary needs must survive the unknown ones");
        assert_eq!(record.tool, "claude");
        assert_eq!(record.template_version, 3);

        // Control: a record still missing a field this binary requires is still refused, so the
        // tolerance above cannot be mistaken for "accept any shape".
        let incomplete = serde_json::json!({
            "version": 1,
            "artifacts": {"claude:skill": {"tool": "claude", "digest": "a".repeat(64)}}
        });
        assert!(
            serde_json::from_slice::<AgentArtifactManifest>(
                serde_json::to_vec(&incomplete).unwrap().as_slice()
            )
            .is_err(),
            "a manifest missing a required field must still be refused"
        );
    }
}
