
fn commit_is_ancestor_of_head(root: &Path, commit: &str) -> bool {
    // Its commits come from committed bootstrap records, so only a full commit id is looked up.
    if !is_canonical_commit_id(commit)
        || git_output(root, &["rev-parse", "--verify", commit]).is_none()
    {
        return false;
    }
    crate::git_utils::status_without_inherited_output(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", commit, "HEAD"])
            .current_dir(root),
    )
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
/// list — pinning that field would revoke the bootstrap for filling it in later.
/// Everything that decides whether the gate bites —
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
    historical_reconciliation_anchor(record)?;
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
        return Err(unfinished_definition_refusal(
            record,
            "at least one acceptance criterion is required",
            "acceptance_criteria",
        ));
    }
    if record.affected_specs.is_empty() && !record.no_spec_change {
        return Err(unfinished_definition_refusal(
            record,
            "affected specs are required unless no_spec_change is justified",
            "affected_specs",
        ));
    }
    if record.affected_paths.is_empty() {
        return Err(unfinished_definition_refusal(
            record,
            "at least one affected path is required",
            "affected_paths",
        ));
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
    let remaining = next_questions_for(root, record);
    if !remaining.is_empty() {
        return Err(if record.state == ChangeState::Draft {
            interview_incomplete_refusal(&record.id, &remaining)
        } else {
            INTERVIEW_INCOMPLETE.into()
        });
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

const INTERVIEW_INCOMPLETE: &str = "the deterministic interview is incomplete";

/// The refusal for a definition missing `question`'s answer (REQ-change-133). Only a draft is
/// told to run `change answer`, which refuses every other state; a record past draft reaches
/// this through check, verification or audit and keeps the bare condition.
fn unfinished_definition_refusal(
    record: &ChangeRecord,
    condition: &str,
    question: &str,
) -> String {
    if record.state == ChangeState::Draft {
        format!(
            "{condition}; run {}",
            answer_command_hint(&record.id, question, &[])
        )
    } else {
        condition.to_string()
    }
}

/// A draft's refusal for an unfinished interview: every question still open, and the
/// `change answer` command for the first one (REQ-change-133).
fn interview_incomplete_refusal(id: &str, remaining: &[InterviewQuestion]) -> String {
    let open = remaining
        .iter()
        .map(|question| format!("`{}`", question.id))
        .collect::<Vec<_>>()
        .join(", ");
    let Some(first) = remaining.first() else {
        return INTERVIEW_INCOMPLETE.into();
    };
    format!(
        "{INTERVIEW_INCOMPLETE} (still to answer: {open}); run {}",
        answer_command_hint(id, &first.id, &first.choices)
    )
}

/// The `change answer` command for one open interview question, shared by approve's refusal
/// and both forms of status's next action so they name the same command (REQ-change-133,
/// REQ-cmd-change-answer-hint-001). Two or more choices are the placeholder (`<yes|no>`). A
/// single choice is the answer itself, so the command runs as printed: `<none>` would be a
/// Bash input redirection (#958). `constraints` is free text with a `none` shortcut, so its
/// hint also says the lessons that apply can be named instead. No choice takes free text, with
/// the scope questions keeping their established placeholders.
pub(crate) fn answer_command_hint(id: &str, question_id: &str, choices: &[String]) -> String {
    let answer = match choices {
        [] => match question_id {
            "acceptance_criteria" => "\"<observable outcome>\"",
            "affected_specs" => "\"<module>\"",
            "affected_paths" => "\"<path>\"",
            _ => "\"<answer>\"",
        }
        .to_string(),
        [only] => only.clone(),
        choices => format!("<{}>", choices.join("|")),
    };
    let command = format!("`specsync change answer {id} {question_id} {answer}`");
    if question_id == "constraints" && choices.len() == 1 {
        format!("{command}, or name the lessons or principles that constrain it instead")
    } else {
        command
    }
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
    let mut owner_evidence: Option<GitEvidence> = None;
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
            if obligation.module.starts_with("@exact:") {
                return Err(format!(
                    "module `{}` is not a successor-eligible signed owner of predecessor path `{}`",
                    obligation.module, obligation.path
                ));
            }
            if entry.owners.contains(&obligation.module) {
                continue;
            }
            // A reserved exact owner is not a module's claim: it records that no module owned
            // the input when the predecessor signed it. That claim is retired by whichever
            // module owns the path now -- the same rule that will sign the successor's own entry
            // for it -- so eligibility is read from the current configuration rather than from
            // the frozen label. A signed module owner's claim stays with that module.
            if !entry
                .owners
                .iter()
                .all(|owner| owner.starts_with("@exact:"))
            {
                return Err(format!(
                    "module `{}` is not a successor-eligible signed owner of predecessor path `{}`",
                    obligation.module, obligation.path
                ));
            }
            if owner_evidence.is_none() {
                owner_evidence = Some(acceptance_owner_spec_evidence(root, record)?);
            }
            let currently_owns = owner_evidence
                .as_ref()
                .map(|evidence| {
                    module_currently_owns_path(
                        root,
                        record,
                        &obligation.path,
                        &obligation.module,
                        evidence,
                    )
                })
                .transpose()?
                .unwrap_or(false);
            if !currently_owns {
                return Err(format!(
                    "module `{}` does not currently own predecessor path `{}` (signed `{}`); grant it with `owns` under `[modules.\"{}\"]` in `.specsync/config.toml`",
                    obligation.module,
                    obligation.path,
                    entry.owners.join("`, `"),
                    obligation.module
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

/// Whether `value` is a full commit id as SpecSync records one: 40 lowercase hex digits (SHA-1).
pub(crate) fn is_canonical_commit_id(value: &str) -> bool {
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
        if artifact_is_incomplete(artifact, &content) {
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
fn artifact_is_incomplete(artifact: &ArtifactKind, content: &str) -> bool {
    artifact_content_is_incomplete(content)
        || (matches!(artifact, ArtifactKind::Plan) && plan_sections_incomplete(content))
}

/// The headings a selected plan must fill before approval.
const PLAN_SECTION_TITLES: [&str; 5] = [
    "approach",
    "out of scope",
    "steps",
    "risks",
    "constraints consulted",
];

/// True when a required plan heading is missing or has no substantive text under it.
///
/// Headings and body lines inside a fenced code block are examples, not the plan.
/// An unclosed fence hides the rest of the file.
fn plan_sections_incomplete(content: &str) -> bool {
    let body = strip_html_comments(strip_frontmatter(content));
    let mut sections: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut fence: Option<(char, usize)> = None;
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some((open_marker, open_length)) = fence {
            // A closing fence may be followed only by whitespace. ```markdown
            // is an opening fence with an info string, not a close.
            if let Some(mark) = fence_mark(line)
                && mark.bare
                && mark.marker == open_marker
                && mark.length >= open_length
            {
                fence = None;
            }
            continue;
        } else if let Some(mark) = fence_mark(line) {
            fence = Some((mark.marker, mark.length));
            continue;
        }
        if let Some(title) = trimmed.strip_prefix("## ") {
            let key = title.trim().to_ascii_lowercase();
            if PLAN_SECTION_TITLES.contains(&key.as_str()) {
                current = Some(key.clone());
                sections.entry(key).or_default();
            } else {
                // An unrelated heading ends the previous section. Leaving `current` set
                // would count that heading's prose as the empty required section above it.
                current = None;
            }
            continue;
        }
        if let Some(key) = &current {
            sections
                .entry(key.clone())
                .or_default()
                .push(line.to_string());
        }
    }
    PLAN_SECTION_TITLES.iter().any(|title| {
        sections.get(*title).is_none_or(|lines| {
            !lines
                .iter()
                .any(|line| plan_section_line_is_substantive(line))
        })
    })
}

struct FenceMark {
    marker: char,
    length: usize,
    /// True when nothing but whitespace follows the marker run.
    bare: bool,
}

/// A CommonMark fence: at most three leading spaces, then a backtick or tilde
/// run of at least three. Four spaces is an indented code block, not a fence.
fn fence_mark(line: &str) -> Option<FenceMark> {
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let marker = rest.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = rest
        .chars()
        .take_while(|character| *character == marker)
        .count();
    if length < 3 {
        return None;
    }
    Some(FenceMark {
        marker,
        length,
        bare: rest[length..].trim().is_empty(),
    })
}

fn plan_section_line_is_substantive(line: &str) -> bool {
    let line = line.trim();
    if line.is_empty() || line.starts_with("<!--") || line.starts_with('#') {
        return false;
    }
    !is_placeholder_todo_line(line)
}

/// Remove HTML comments, including ones that span lines.
///
/// A comment is not a lesson and not a plan. Keeping only the opening line out
/// left the hidden prose looking substantive.
fn strip_html_comments(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("<!--") {
        kept.push_str(&rest[..start]);
        rest = &rest[start + 4..];
        if let Some(end) = rest.find("-->") {
            rest = &rest[end + 3..];
        } else {
            rest = "";
        }
    }
    kept.push_str(rest);
    kept
}

fn artifact_content_is_incomplete(content: &str) -> bool {
    if content.contains("<!-- TODO") {
        return true;
    }
    let body = strip_frontmatter(content);
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

/// How many open tasks a refusal or next action names one by one; the rest are counted.
const OPEN_TASKS_NAMED: usize = 5;
/// How many characters of one task's text a refusal quotes.
const OPEN_TASK_TEXT_CHARS: usize = 80;
/// The start of every open-task refusal. It is the whole refusal text from before #892, kept so
/// anything that matched it still matches.
const OPEN_TASKS_REFUSAL: &str = "tasks.md contains incomplete tasks";

/// An unchecked `[ ]` task outside fenced code: its 1-based line and its quotable text.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenTask {
    line: usize,
    text: String,
}

/// Every open task in `content`, found by the same parser that blanks checkbox state for the
/// definition digest, so what counts as a task cannot differ between the two.
fn open_tasks_in(content: &[u8]) -> Vec<OpenTask> {
    let mut line = 1;
    let mut counted_to = 0;
    markdown_task_checkbox_offsets(content)
        .into_iter()
        .filter(|offset| content[*offset] == b' ')
        .map(|offset| {
            line += content[counted_to..offset]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count();
            counted_to = offset;
            let end = content[offset..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(content.len(), |length| offset + length);
            // `offset` is the box's inner byte and `offset + 1` its `]`; the parser guarantees
            // the line continues at least that far.
            let text = String::from_utf8_lossy(&content[offset + 2..end]);
            OpenTask {
                line,
                text: quotable_task_text(&text),
            }
        })
        .collect()
}

/// Task text as one safe quoted phrase: trimmed, without control characters, with quote marks
/// and backticks made plain, and cut at [`OPEN_TASK_TEXT_CHARS`].
fn quotable_task_text(text: &str) -> String {
    let plain: String = text
        .chars()
        .filter(|character| !character.is_control())
        .map(|character| match character {
            '`' | '"' => '\'',
            other => other,
        })
        .collect();
    let plain = plain.trim();
    if plain.chars().count() <= OPEN_TASK_TEXT_CHARS {
        return plain.to_string();
    }
    let mut cut: String = plain.chars().take(OPEN_TASK_TEXT_CHARS).collect();
    cut.push_str("...");
    cut
}

/// The open tasks of a change whose effective definition selects `tasks.md`; empty otherwise.
fn open_tasks(root: &Path, record: &ChangeRecord) -> Result<Vec<OpenTask>, String> {
    let effective = effective_change_definition(root, record)?;
    open_tasks_selected(root, &record.id, &effective.selected_artifacts)
}

/// [`open_tasks`] for a caller that already holds the effective artifact selection.
fn open_tasks_selected(
    root: &Path,
    id: &str,
    selected: &[ArtifactKind],
) -> Result<Vec<OpenTask>, String> {
    if !selected.contains(&ArtifactKind::Tasks) {
        return Ok(Vec::new());
    }
    let path = change_dir(root, id).join("tasks.md");
    let content = fs::read(&path).map_err(|error| error.to_string())?;
    Ok(open_tasks_in(&content))
}

/// "finish the 1 open task in `<file>` (line 4: "...") and mark it `[x]`": the one sentence
/// every refusal, status line and ship-status blocker uses for open tasks.
fn open_tasks_sentence(id: &str, open: &[OpenTask]) -> String {
    let mut listed = open
        .iter()
        .take(OPEN_TASKS_NAMED)
        .map(|task| format!("line {}: \"{}\"", task.line, task.text))
        .collect::<Vec<_>>()
        .join("; ");
    let unnamed = open.len().saturating_sub(OPEN_TASKS_NAMED);
    if unnamed > 0 {
        listed.push_str(&format!("; and {unnamed} more"));
    }
    let (count, close) = if open.len() == 1 {
        ("1 open task".to_string(), "mark it `[x]`")
    } else {
        (format!("{} open tasks", open.len()), "mark each one `[x]`")
    };
    format!("finish the {count} in `{CHANGES_PATH}/{id}/tasks.md` ({listed}) and {close}")
}

/// Whether an open task blocks this change's next lifecycle verb: an approved, implementing or
/// verifying change, or a workflow-v2 finalization that stopped after acceptance. Once that
/// workspace has moved into the archive its `tasks.md` is no longer read, so nothing is named.
pub fn open_tasks_block(record: &ChangeRecord) -> bool {
    matches!(
        record.state,
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
    ) || (record.state == ChangeState::Accepted && record.workflow_version >= 2)
}

/// What has to happen before this change can be checked, reviewed or finalized: its open tasks
/// by line and text, and how to close them (REQ-change-132). `None` when every selected task is
/// done, when the change does not select `tasks.md`, or when the file cannot be read, which the
/// refusing verbs report themselves.
pub fn open_tasks_next_step(root: &Path, record: &ChangeRecord) -> Option<String> {
    let open = open_tasks(root, record).ok()?;
    (!open.is_empty()).then(|| open_tasks_sentence(&record.id, &open))
}

/// The refusal a verb gives while this change has an open task, naming `rerun`, the exact
/// command to run once the tasks are done (REQ-change-131). `None` when nothing is open.
pub fn open_tasks_refusal(root: &Path, record: &ChangeRecord, rerun: &str) -> Option<String> {
    open_tasks_next_step(root, record)
        .map(|step| format!("{OPEN_TASKS_REFUSAL}: {step}, then re-run `{rerun}`"))
}

/// Refuse while the change's selected `tasks.md` has an open task (REQ-change-131). Unlike
/// [`open_tasks_refusal`], an unreadable `tasks.md` is an error here, never a pass.
fn ensure_tasks_complete(root: &Path, record: &ChangeRecord, rerun: &str) -> Result<(), String> {
    let open = open_tasks(root, record)?;
    if open.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{OPEN_TASKS_REFUSAL}: {}, then re-run `{rerun}`",
        open_tasks_sentence(&record.id, &open)
    ))
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
                DeltaTarget::Requirement
                    | DeltaTarget::SpecSection
                    | DeltaTarget::RequirementsSection
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
///
/// `module_errors` attributes the validator verdicts among `errors` to the module whose effective
/// contract produced them. It is private bookkeeping: every caller still reads `errors`, which is
/// unchanged, and `errors` remains the complete list. Failures that are not a validator verdict
/// about one module (ordering, the temporary workspace, an unresolvable canonical spec, a delta
/// that cannot be read or applied) carry no attribution.
#[derive(Debug, Default)]
struct EffectiveContractOutcome {
    errors: Vec<String>,
    suppressions: Vec<String>,
    module_errors: BTreeMap<String, Vec<String>>,
}

impl EffectiveContractOutcome {
    fn failed(error: String) -> Self {
        Self {
            errors: vec![error],
            ..Self::default()
        }
    }

    /// Join the errors for gates whose only output channel is a failure string.
    fn error_text(&self) -> Option<String> {
        (!self.errors.is_empty()).then(|| self.errors.join("; "))
    }

    /// Split the errors for verification of one change (REQ-change-112): the validator verdicts
    /// about modules `declared` names, keyed by module, and every other error in gate order.
    /// Together the two halves hold every error exactly once.
    fn split_for_declared(
        &self,
        declared: &[String],
    ) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
        let attributed: BTreeMap<String, Vec<String>> = self
            .module_errors
            .iter()
            .filter(|(module, _)| declared.contains(module))
            .map(|(module, errors)| (module.clone(), errors.clone()))
            .collect();
        let mut held: Vec<&String> = attributed.values().flatten().collect();
        let rest = self
            .errors
            .iter()
            .filter(|error| {
                match held.iter().position(|candidate| *candidate == *error) {
                    Some(index) => {
                        held.swap_remove(index);
                        false
                    }
                    None => true,
                }
            })
            .cloned()
            .collect();
        (attributed, rest)
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
    let mut module_errors: BTreeMap<String, Vec<String>> = BTreeMap::new();
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
        let mut verdicts: Vec<String> = result
            .errors
            .into_iter()
            .map(|error| format!("effective contract `{module}`: {error}"))
            .collect();
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
            verdicts.push(format!("effective contract `{module}`: {warning}"));
        }
        if !verdicts.is_empty() {
            errors.extend(verdicts.iter().cloned());
            module_errors.insert(module.clone(), verdicts);
        }
    }
    let _ = fs::remove_dir_all(temp);
    EffectiveContractOutcome {
        errors,
        suppressions,
        module_errors,
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
            if let Some(problem) = requirements_section_item_problem(&item) {
                return Err(problem);
            }
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
            // applied change permanently unverifiable. A sent-back change's own earlier
            // ADDED block is not a foreign one either: materialization replaces it, so
            // approve must accept the revised body too (#1001).
            if item.target == DeltaTarget::Requirement
                && item.operation == DeltaOperation::Added
                && living_requirement_conflicts(root, module, &item.key, &item.content)?
                && !sent_back_added_block_is_own_on_disk(root, record, module, &item)?
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

/// A semantic delta body reduced to the form the APPLIER compares, and no further (#730).
///
/// `\r\n` becomes `\n`, because line-ending style is provably not content in this module: the
/// applier says so in two independent places. `markdown_block_matches` normalizes CRLF before
/// deciding whether an `## ADDED` block is already present, and `apply_markdown_block` re-emits
/// every body in the target file's own style. `parse_delta` reads through `str::lines()`, which
/// discards the `\r` of a CRLF pair outright, so a CRLF delta and an LF delta materialize
/// byte-identical canonical specs. Hashing the raw bytes therefore bound the digest to something
/// materialization never consumes, and a Windows checkout with `core.autocrlf=true` produced a
/// different digest for a delta nobody had edited.
///
/// NOTHING ELSE IS FOLDED, and that limit is the load-bearing half. `markdown_block_matches` also
/// trims surrounding blank lines and horizontal whitespace; copying that would make the binding
/// accept edits it exists to refuse, because trailing whitespace and blank lines inside a body ARE
/// wording a reviewer signed. The digest's job is strictly narrower than the applier's: the
/// applier decides whether an edit is already applied, the digest decides whether an approver saw
/// it. Only the line-ending axis is safe to erase, because it is the only one Git rewrites on its
/// own, without an author.
///
/// A LONE `\r` IS CONTENT and stays. Classic-Mac line endings are not something Git's `text` /
/// `eol` / `core.autocrlf` conversions ever produce — every one of them converts only between LF
/// and CRLF — so no honest cross-OS handoff can introduce one. Both `str::lines()` and
/// `markdown_block_matches` keep a bare `\r` as ordinary text, and `parser::parse_frontmatter`
/// preserves it deliberately for the same reason (#715). Folding it here would erase a byte the
/// applier would carry into the canonical spec.
///
/// Guarded on `contains('\r')` so an LF delta — every delta in a repository with #715's
/// `.gitattributes` pins — borrows and allocates nothing.
fn canonical_delta_body(body: &str) -> Cow<'_, str> {
    if body.contains('\r') {
        Cow::Owned(body.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(body)
    }
}

/// Per-module digest over every semantic delta file this change owns, line endings canonicalized.
///
/// Keyed by module because that is what a refusal has to name: "the delta changed" is not an
/// actionable message when a change owns nine specs. The module name is framed into the digest
/// as well, so moving a body from `deltas/a.md` to `deltas/b.md` cannot preserve a digest.
///
/// The body is framed through `canonical_delta_body`, so the digest asks the same question the
/// applier asks. See that function for why the line-ending axis is the ONLY one normalized.
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
        digests.insert(module.clone(), delta_body_digest(module, &body));
    }
    Ok(digests)
}

/// The framed digest of one module's delta body, CRLF folded. Approval and status both frame
/// through this one function, so the approval-time and status-time constructions cannot drift.
fn delta_body_digest(module: &str, body: &str) -> String {
    let mut digest = FramedDigest::new(APPROVED_DELTA_DIGEST_DOMAIN);
    digest.frame(b"module", module.as_bytes());
    digest.frame(b"body", canonical_delta_body(body).as_bytes());
    digest.finish()
}

/// The ledger key of one `## MODIFIED` / `## REMOVED` item's approval-time base (#843), e.g.
/// `cli: REQUIREMENT REQ-cli-007`. Readable on purpose: it is what a refusal names.
fn delta_base_key(module: &str, item: &DeltaItem) -> String {
    let target = match item.target {
        DeltaTarget::Requirement => "REQUIREMENT",
        DeltaTarget::SpecSection => "SPEC SECTION",
        DeltaTarget::RequirementsSection => "REQUIREMENTS SECTION",
    };
    format!("{module}: {target} {}", item.key)
}

/// Framed digest of the canonical block an item targets, as it stands in `spec`/`requirements`
/// now, or of its absence. Line endings and surrounding blank lines fold exactly as
/// `markdown_block_matches` folds them, so a CRLF checkout never reads as a moved block.
fn canonical_block_digest(module: &str, spec: &str, requirements: &str, item: &DeltaItem) -> String {
    let source = delta_target_source(item.target, spec, requirements);
    let mut digest = FramedDigest::new(APPROVED_DELTA_BASE_DOMAIN);
    digest.frame(b"key", delta_base_key(module, item).as_bytes());
    match delta_block_range(source, item.target, &item.key) {
        None => digest.frame(b"absent", b""),
        Some(range) => {
            let block = source[range].replace("\r\n", "\n");
            digest.frame(b"block", block.trim_matches(['\n', ' ', '\t']).as_bytes());
        }
    }
    digest.finish()
}

/// Every `## MODIFIED` / `## REMOVED` item of every affected module, with the canonical spec and
/// requirements text it applies to, read once per module.
fn for_each_replacing_delta_item(
    root: &Path,
    record: &ChangeRecord,
    mut visit: impl FnMut(&str, &str, &str, &DeltaItem),
) -> Result<(), String> {
    if record.no_spec_change {
        return Ok(());
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    for module in &record.affected_specs {
        let delta = read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")
            .map_err(|error| format!("semantic delta for {module}: {error}"))?;
        let items = parse_delta(&delta)?;
        if !items
            .iter()
            .any(|item| item.operation != DeltaOperation::Added)
        {
            continue;
        }
        let (spec_path, requirements_path) = canonical_module_paths(root, specs_dir.as_str(), module)?;
        // A missing file is a block that is absent, which is a base like any other; whether a
        // MODIFIED item may target it is `apply_markdown_block`'s question, not this one's.
        let spec = read_canonical_or_absent(root, &spec_path)?;
        let requirements = read_canonical_or_absent(root, &requirements_path)?;
        for item in items.iter().filter(|item| {
            item.operation != DeltaOperation::Added && !is_change_log_section(item)
        }) {
            visit(module, &spec, &requirements, item);
        }
    }
    Ok(())
}

/// A canonical file's text, or empty when it does not exist. Any other read failure is an error:
/// reading an unreadable file as empty would report an IO fault as a moved block.
fn read_canonical_or_absent(root: &Path, path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!(
            "could not read canonical `{}` to compare delta bases: {error}",
            portable_project_path(root, path)
        )),
    }
}

/// Whether an item targets the module spec's `## Change Log` section. Materialization appends
/// this change's own row to that section, so after the first check the block never equals the
/// approved body and a recorded base could only ever refuse the change's own text. The Change Log
/// therefore records no base and is not guarded against a stale body (documented in deltas.md).
fn is_change_log_section(item: &DeltaItem) -> bool {
    item.target == DeltaTarget::SpecSection && item.key == "Change Log"
}

/// The approval-time base of every replacing item (#843): what a definition approval records
/// beside the delta bodies it approves.
fn delta_base_digests(
    root: &Path,
    record: &ChangeRecord,
) -> Result<BTreeMap<String, String>, String> {
    let mut bases = BTreeMap::new();
    for_each_replacing_delta_item(root, record, |module, spec, requirements, item| {
        bases.insert(
            delta_base_key(module, item),
            canonical_block_digest(module, spec, requirements, item),
        );
    })?;
    Ok(bases)
}

/// One unchanged, unapplied delta item whose canonical block moved since its approval.
struct MovedBaseCandidate {
    module: String,
    item: DeltaItem,
    key: String,
    previous_base: String,
    current_base: String,
    current_body: String,
}

/// Commits searched, newest first, for the canonical text an earlier approval recorded.
const MOVED_BASE_HISTORY_LIMIT: usize = 200;
/// Lines per side the three-way comparison accepts; a larger block keeps the refusal.
const MOVED_BASE_LINE_LIMIT: usize = 1_000;

/// Whether the approved body already carries every edit the current canonical text made to
/// the approval-time base (#975). The base text is recovered from git by its recorded digest;
/// when it cannot be found the answer is no, and the approval is refused as before.
fn moved_base_is_already_carried(
    root: &Path,
    candidate: &MovedBaseCandidate,
) -> Result<bool, String> {
    let specs_dir = crate::config::load_config(root).specs_dir;
    let (spec_path, requirements_path) =
        canonical_module_paths(root, &specs_dir, &candidate.module)?;
    let path = match candidate.item.target {
        DeltaTarget::Requirement | DeltaTarget::RequirementsSection => requirements_path,
        DeltaTarget::SpecSection => spec_path,
    };
    let Some(old_body) = recover_approved_base_body(root, &path, candidate) else {
        return Ok(false);
    };
    let approved = candidate
        .item
        .content
        .replace("\r\n", "\n")
        .trim_matches(['\n', ' ', '\t'])
        .to_string();
    Ok(three_way_keeps_body(
        &old_body,
        &candidate.current_body,
        &approved,
    ))
}

/// The block body whose digest an earlier approval recorded, read from the newest commit
/// that holds it. `None` outside git, past the history bound, or when no commit matches.
fn recover_approved_base_body(
    root: &Path,
    path: &Path,
    candidate: &MovedBaseCandidate,
) -> Option<String> {
    let prefix = git_repo_prefix(root).ok()?;
    let file = project_relative(root, path)?;
    let limit = MOVED_BASE_HISTORY_LIMIT.to_string();
    let commits = git_output_allow_empty(
        root,
        &["log", "--format=%H", "-n", limit.as_str(), "--", file.as_str()],
    )?;
    for commit in commits.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let object = format!("{commit}:{prefix}{file}");
        let text = git_output_allow_empty(root, &["show", object.as_str()]).unwrap_or_default();
        let (spec, requirements) = match candidate.item.target {
            DeltaTarget::Requirement | DeltaTarget::RequirementsSection => ("", text.as_str()),
            DeltaTarget::SpecSection => (text.as_str(), ""),
        };
        if canonical_block_digest(&candidate.module, spec, requirements, &candidate.item)
            == candidate.previous_base
        {
            let source = if spec.is_empty() { requirements } else { spec };
            return Some(block_body(source, &candidate.item).unwrap_or_default());
        }
    }
    None
}

/// Three-way comparison of an approval-time base, the current canonical text and the approved
/// body, line by line, aligned diff3-style on the base. Applying the approved body overwrites
/// nothing exactly when every region the current text changed is either unchanged there or
/// already identical in the approved body; then the merge of the three is the approved body.
fn three_way_keeps_body(base: &str, current: &str, approved: &str) -> bool {
    let base: Vec<&str> = base.lines().collect();
    let current: Vec<&str> = current.lines().collect();
    let approved: Vec<&str> = approved.lines().collect();
    if [base.len(), current.len(), approved.len()]
        .iter()
        .any(|lines| *lines > MOVED_BASE_LINE_LIMIT)
    {
        return false;
    }
    let to_current = longest_common_subsequence_map(&base, &current);
    let to_approved = longest_common_subsequence_map(&base, &approved);
    let (mut b, mut c, mut a) = (0usize, 0usize, 0usize);
    loop {
        while b < base.len() && to_current[b] == Some(c) && to_approved[b] == Some(a) {
            b += 1;
            c += 1;
            a += 1;
        }
        if b == base.len() && c == current.len() && a == approved.len() {
            return true;
        }
        let next = (b..base.len()).find(|&index| {
            to_current[index].is_some_and(|line| line >= c)
                && to_approved[index].is_some_and(|line| line >= a)
        });
        let (base_end, current_end, approved_end) = match next {
            Some(index) => (
                index,
                to_current[index].unwrap_or(current.len()),
                to_approved[index].unwrap_or(approved.len()),
            ),
            None => (base.len(), current.len(), approved.len()),
        };
        let base_chunk = &base[b..base_end];
        let current_chunk = &current[c..current_end];
        let approved_chunk = &approved[a..approved_end];
        if current_chunk != base_chunk && current_chunk != approved_chunk {
            return false;
        }
        b = base_end;
        c = current_end;
        a = approved_end;
        if next.is_none() {
            return true;
        }
    }
}

/// For each line of `left`, the index of the line it is paired with in one longest common
/// subsequence of `left` and `right`, or `None`.
fn longest_common_subsequence_map(left: &[&str], right: &[&str]) -> Vec<Option<usize>> {
    let (rows, columns) = (left.len(), right.len());
    let mut lengths = vec![0u32; (rows + 1) * (columns + 1)];
    let at = |row: usize, column: usize| row * (columns + 1) + column;
    for row in (0..rows).rev() {
        for column in (0..columns).rev() {
            lengths[at(row, column)] = if left[row] == right[column] {
                lengths[at(row + 1, column + 1)] + 1
            } else {
                lengths[at(row + 1, column)].max(lengths[at(row, column + 1)])
            };
        }
    }
    let mut map = vec![None; rows];
    let (mut row, mut column) = (0usize, 0usize);
    while row < rows && column < columns {
        if left[row] == right[column] {
            map[row] = Some(column);
            row += 1;
            column += 1;
        } else if lengths[at(row + 1, column)] >= lengths[at(row, column + 1)] {
            row += 1;
        } else {
            column += 1;
        }
    }
    map
}

/// The bases a new definition approval records (#843), refusing a re-approval that would
/// launder a stale body.
///
/// For an item whose module body is byte-for-byte the body the previous definition approval
/// signed, and whose base that approval recorded, the previous base is the truth: the body was
/// written against it. If the item is already applied, the base travels unchanged (re-reading the
/// tree would record this change's own text and forget what it replaced). If it is not applied and
/// the block has moved since, re-approving would record the newer text as the base and let the
/// unchanged stale body overwrite it, so the approval is refused, naming the block. Every other
/// item (a rewritten body, or no earlier base) records the canonical block as it stands now.
fn approval_delta_bases(
    root: &Path,
    record: &ChangeRecord,
    ledger: &ApprovalLedger,
) -> Result<BTreeMap<String, String>, String> {
    let previous = effective_definition_approval(root, record, ledger)
        .ok()
        .or_else(|| {
            ledger
                .approvals
                .iter()
                .rev()
                .find(|approval| approval.gate == "definition")
        });
    let (Some(previous_bases), Some(previous_bodies)) = (
        previous.and_then(|approval| approval.approved_delta_bases.as_ref()),
        previous.and_then(|approval| approval.approved_delta_digests.as_ref()),
    ) else {
        return delta_base_digests(root, record);
    };
    let current_bodies = delta_body_digests(root, record)?;
    let mut bases = BTreeMap::new();
    let mut candidates: Vec<MovedBaseCandidate> = Vec::new();
    for_each_replacing_delta_item(root, record, |module, spec, requirements, item| {
        let key = delta_base_key(module, item);
        let current = canonical_block_digest(module, spec, requirements, item);
        let body_unchanged = previous_bodies.get(module).is_some()
            && previous_bodies.get(module) == current_bodies.get(module);
        let base = match previous_bases.get(&key) {
            Some(previous_base) if body_unchanged => {
                if !delta_item_is_applied(spec, requirements, item) && *previous_base != current {
                    let source = delta_target_source(item.target, spec, requirements);
                    candidates.push(MovedBaseCandidate {
                        module: module.to_string(),
                        item: item.clone(),
                        key: key.clone(),
                        previous_base: previous_base.clone(),
                        current_base: current.clone(),
                        current_body: block_body(source, item).unwrap_or_default(),
                    });
                }
                previous_base.clone()
            }
            _ => current,
        };
        bases.insert(key, base);
    })?;
    // A moved base is not by itself a stale body (#975): when the body the human approved
    // already carries every edit the newer canonical text made to the approval-time base, a
    // three-way comparison shows applying it overwrites nothing, so the approval records the
    // new base. Anything else, including an approval-time base git no longer holds, is refused.
    let mut moved: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for candidate in candidates {
        if moved_base_is_already_carried(root, &candidate)? {
            bases.insert(candidate.key.clone(), candidate.current_base.clone());
            continue;
        }
        let target = delta_target_label(&candidate.item);
        moved.entry(candidate.module).or_default().push(target);
    }
    if moved.is_empty() {
        return Ok(bases);
    }
    Err(format!(
        "cannot approve {}: the semantic delta for {} was approved against canonical text that has changed since ({}) and the delta body has not been rewritten; approving it unchanged would let the stale body overwrite text this change never approved, so rewrite each listed delta body onto the current canonical text first, then run `specsync change approve {}` and `specsync change check {}`",
        record.id,
        moved
            .keys()
            .map(|module| format!("`{module}`"))
            .collect::<Vec<_>>()
            .join(", "),
        moved
            .iter()
            .map(|(module, targets)| format!("{module}: {}", targets.join(", ")))
            .collect::<Vec<_>>()
            .join("; "),
        record.id,
        record.id
    ))
}

/// Refuses to apply a `## MODIFIED` or `## REMOVED` item over canonical text that changed after
/// the item was approved (#843).
///
/// A modified body replaces the whole block. When the base branch rewrites that block after
/// approval — another change lands and is merged in — applying the stale body would delete the
/// other change's text with no record that anything was lost, and the tests rarely cover the
/// dropped wording, so verification still passes. The approval recorded the block it was granted
/// over; an item whose block is neither that base nor already this change's own text is refused,
/// naming the block, before anything is written.
///
/// An item that is already applied is never refused: the block carrying exactly this change's
/// approved text is converged, whatever the base was. Absent bases (approvals written before this
/// binding) are unknown, not violated, so such a change keeps materializing as before.
fn ensure_approved_delta_bases_current(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    if record.no_spec_change {
        return Ok(());
    }
    let ledger = load_approvals(root, record)?;
    let approval = effective_definition_approval(root, record, &ledger)?;
    let Some(bases) = approval.approved_delta_bases.as_ref() else {
        // Absence is monotone, exactly as for `approved_delta_digests` (#719): a ledger that has
        // recorded bases for this change is new enough to record them again, so a later
        // definition approval carrying none is a claim being withdrawn, not silence from before
        // the binding, and it must not switch this guard off.
        if ledger.approvals.iter().any(|candidate| {
            candidate.gate == "definition" && candidate.approved_delta_bases.is_some()
        }) {
            return Err(format!(
                "the definition approval for {} records no canonical bases for its MODIFIED or REMOVED delta items although an earlier definition approval on this change recorded them; an approval cannot withdraw a claim an earlier one made, so re-run `specsync change approve {}` to record the canonical text this approval covers, then `specsync change check {}` (or restore the approval ledger)",
                record.id, record.id, record.id
            ));
        }
        return Ok(());
    };
    let mut moved: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for_each_replacing_delta_item(root, record, |module, spec, requirements, item| {
        if delta_item_is_applied(spec, requirements, item) {
            return;
        }
        let key = delta_base_key(module, item);
        let Some(base) = bases.get(&key) else {
            return;
        };
        if *base != canonical_block_digest(module, spec, requirements, item) {
            let target = delta_target_label(item);
            moved.entry(module.to_string()).or_default().push(target);
        }
    })?;
    if moved.is_empty() {
        return Ok(());
    }
    Err(format!(
        "semantic delta for {} was approved against canonical text that has changed since ({}); applying it would overwrite text this change never approved, so rewrite each listed delta body onto the current canonical text, re-run `specsync change approve {}` to approve the rebased bodies, then `specsync change check {}`",
        moved
            .keys()
            .map(|module| format!("`{module}`"))
            .collect::<Vec<_>>()
            .join(", "),
        moved
            .iter()
            .map(|(module, targets)| format!("{module}: {}", targets.join(", ")))
            .collect::<Vec<_>>()
            .join("; "),
        record.id,
        record.id
    ))
}

/// Refuses to apply a semantic delta whose body is not the body the approver signed (#704).
///
/// ABSENT EVIDENCE IS NOT A VIOLATION. Every approval written before `approved_delta_digests`
/// existed carries `None`, and every one of the 183 archived changes in this repository is in
/// that position. `None` means the approval made no claim about delta wording, so this returns
/// `Ok(())` and leaves the judgement to the gates that did exist at the time. Reading a missing
/// digest as tampering would fail all of recorded history on evidence nobody could have written
/// — the exact shape of #672, #684 and #689's first design.
///
/// ABSENCE IS MONOTONE, though, and that is the part #719 found missing. "Predates the binding"
/// is a property of a whole LEDGER, not of one event: a ledger that has recorded a delta digest
/// for this change is demonstrably new enough to record one again, so a later definition approval
/// carrying none is not silence from before the binding — it is a claim being withdrawn. That
/// state is refused, and refusing it costs history nothing: an archived ledger has no digest on
/// ANY of its definition approvals, so the reading above stays the one that applies to it.
fn ensure_approved_delta_bodies_unchanged(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(), String> {
    let changed = changed_approved_delta_modules(root, record)?;
    if changed.is_empty() {
        return Ok(());
    }
    // The remedy names BOTH steps on purpose. Naming only `approve` was actively harmful while
    // #741 was open: re-approving recorded a digest for the corrected body, satisfied this very
    // check, and then the `canonical_applied` short-circuit discarded the correction in silence.
    // Approval binds the wording; only `check` puts it in the canonical spec.
    Err(format!(
        "semantic delta for {} changed after approval; the approved wording is what rewrites the canonical spec, so re-run `specsync change approve {}` to approve the current delta bodies and then `specsync change check {}` to materialize them into the canonical spec (or restore the approved delta bodies)",
        changed
            .iter()
            .map(|module| format!("`{module}`"))
            .collect::<Vec<_>>()
            .join(", "),
        record.id,
        record.id
    ))
}

/// The modules, sorted, whose delta body no longer matches the digest the effective definition
/// approval recorded for it, plus any delta the approval never recorded. Empty when the bodies
/// still match, and also when no definition approval in the ledger ever recorded a digest: that
/// absence is unknown, not violated. `Err` for every refusal that names no module, such as a
/// withdrawn binding or an unreadable delta.
///
/// The list is the reason a bare `bool` would throw away. Status names these modules in
/// `ChangeSummary.changed_approved_deltas`; materialization and acceptance refuse with them.
fn changed_approved_delta_modules(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<String>, String> {
    Ok(match approved_delta_binding(root, record)? {
        DeltaBinding::Unrecorded => Vec::new(),
        DeltaBinding::Recorded { changed, .. } => changed,
    })
}

/// One module's delta body as the binding read it, and whether it matches its approval.
#[derive(Clone, Debug)]
struct BoundDelta {
    intact: bool,
    body: String,
}

/// The delta binding of one change, read once: every affected module's body and its verdict.
///
/// `Unrecorded` is a ledger in which no definition approval recorded a delta digest, which is
/// unknown, not violated (REQ-change-089, REQ-change-090). `changed` is exactly the sorted list
/// `changed_approved_delta_modules` always returned; `per_module` keeps the bytes that were
/// digested, so every projection of the binding is a projection of the same read (REQ-change-113).
#[derive(Clone, Debug)]
enum DeltaBinding {
    Unrecorded,
    Recorded {
        per_module: BTreeMap<String, BoundDelta>,
        changed: Vec<String>,
    },
}

/// The only reader of delta bodies for the approval binding. Inside a delta-binding memo (every
/// read scope, and the definition-mutation transaction) the result is memoized per change, so a
/// status call reads each delta once; outside one, as in the refusing materialization and
/// acceptance gates, every call reads live.
fn approved_delta_binding(root: &Path, record: &ChangeRecord) -> Result<DeltaBinding, String> {
    let key = DeltaBindingMemoKey {
        id: record.id.clone(),
        affected_specs: record.affected_specs.clone(),
        no_spec_change: record.no_spec_change,
    };
    if let Some(memoized) = memoized_delta_binding(root, &key) {
        return memoized;
    }
    let binding = read_approved_delta_binding(root, record);
    memoize_delta_binding(root, key, &binding);
    binding
}

fn read_approved_delta_binding(
    root: &Path,
    record: &ChangeRecord,
) -> Result<DeltaBinding, String> {
    let ledger = load_approvals(root, record)?;
    let approval = effective_definition_approval(root, record, &ledger)?;
    let Some(approved) = approval.approved_delta_digests.as_ref() else {
        if ledger.approvals.iter().any(|candidate| {
            candidate.gate == "definition" && candidate.approved_delta_digests.is_some()
        }) {
            return Err(format!(
                "the definition approval for {} records no semantic delta wording although an earlier definition approval on this change recorded it; an approval cannot withdraw a claim an earlier one made, so re-run `specsync change approve {}` to record the delta bodies this approval covers, then `specsync change check {}` to bring the canonical spec to them (or restore the approval ledger)",
                record.id, record.id, record.id
            ));
        }
        return Ok(DeltaBinding::Unrecorded);
    };
    // A `no_spec_change` record owns no delta bodies (see `delta_body_digests`).
    let mut per_module = BTreeMap::new();
    if !record.no_spec_change {
        for module in &record.affected_specs {
            let path = delta_path_checked(root, record, module)?;
            let body = read_bounded_change_text(&path, "semantic delta")
                .map_err(|error| format!("semantic delta for {module}: {error}"))?;
            record_test_delta_binding_read();
            let intact = approved.get(module) == Some(&delta_body_digest(module, &body));
            per_module.insert(module.clone(), BoundDelta { intact, body });
        }
    }
    let mut changed: Vec<String> = approved
        .keys()
        .filter(|module| !per_module.get(*module).is_some_and(|bound| bound.intact))
        .cloned()
        .collect();
    changed.extend(
        per_module
            .iter()
            .filter(|(_, bound)| !bound.intact)
            .map(|(module, _)| module.clone()),
    );
    changed.sort_unstable();
    changed.dedup();
    Ok(DeltaBinding::Recorded {
        per_module,
        changed,
    })
}

/// Per affected module, what the current definition approval covers (REQ-change-113): the
/// requirement IDs and spec sections the approved delta body declares, a three-valued integrity
/// flag, and whether the canonical tree carries the body.
///
/// Reported only for an approved, implementing or verifying change whose scope approval is
/// current, the states and the scope half `approval_currency` uses. Every field comes from the
/// one memoized binding read the summary also uses, so `delta_intact` and
/// `ChangeSummary.changed_approved_deltas` cannot disagree in one read scope. A list is withheld,
/// never emptied, when its provenance is not proven: an edited body, a refused binding and an
/// unknown binding carry the module and the flag only. Entries are sorted by module.
pub fn approved_delta_summaries(root: &Path, record: &ChangeRecord) -> Vec<ApprovedDeltaSummary> {
    if record.no_spec_change
        || !matches!(
            record.state,
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
        )
        || ensure_definition_approval_valid(root, record).is_err()
    {
        return Vec::new();
    }
    let mut modules: Vec<&String> = record.affected_specs.iter().collect();
    modules.sort_unstable();
    modules.dedup();
    let bare = |module: &str, delta_intact: Option<bool>| ApprovedDeltaSummary {
        module: module.to_string(),
        delta_intact,
        requirements: None,
        sections: None,
        requirements_sections: None,
        materialized: None,
        declarations_error: None,
    };
    let per_module = match approved_delta_binding(root, record) {
        // A withdrawn binding or an unreadable delta: `approval_currency` reads the same `Err`
        // as `approval_valid: false`.
        Err(_) => {
            return modules
                .into_iter()
                .map(|module| bare(module, Some(false)))
                .collect();
        }
        Ok(DeltaBinding::Unrecorded) => {
            return modules
                .into_iter()
                .map(|module| bare(module, None))
                .collect();
        }
        Ok(DeltaBinding::Recorded { per_module, .. }) => per_module,
    };
    let specs_dir = record
        .canonical_applied
        .then(|| crate::config::load_config(root).specs_dir);
    modules
        .into_iter()
        .map(|module| {
            let Some(bound) = per_module.get(module).filter(|bound| bound.intact) else {
                return bare(module, Some(false));
            };
            match parse_delta(&bound.body) {
                Ok(items) => {
                    let (requirements, sections, requirements_sections) =
                        split_delta_declarations(&items);
                    // Before the first materialization the answer is false by definition, and a
                    // first application may raise refusals only a first application should raise.
                    let materialized = specs_dir.as_deref().is_some_and(|specs_dir| {
                        let mut prepared = PreparedMaterialization::default();
                        prepare_module_application(
                            root,
                            record,
                            specs_dir,
                            module,
                            &bound.body,
                            &mut prepared,
                        )
                        .is_ok_and(|()| !prepared.pending.iter().any(|pending| pending == module))
                    });
                    let any = |declarations: &DeltaDeclarations| {
                        !(declarations.added.is_empty()
                            && declarations.modified.is_empty()
                            && declarations.removed.is_empty())
                    };
                    ApprovedDeltaSummary {
                        requirements: Some(requirements),
                        sections: Some(sections),
                        // Present only when the delta has such items, so every existing summary
                        // keeps its bytes (#866).
                        requirements_sections: any(&requirements_sections)
                            .then_some(requirements_sections),
                        materialized: Some(materialized),
                        ..bare(module, Some(true))
                    }
                }
                // The digest matched, so this is the approved body; only its declarations are
                // unavailable under the current grammar, and the diagnostic says why.
                Err(error) => ApprovedDeltaSummary {
                    declarations_error: Some(error),
                    ..bare(module, Some(true))
                },
            }
        })
        .collect()
}

/// Split parsed delta items into requirement, spec-section and requirements-section declarations
/// by operation. Only item headings are items, so prose is never reported.
fn split_delta_declarations(
    items: &[DeltaItem],
) -> (DeltaDeclarations, DeltaDeclarations, DeltaDeclarations) {
    let mut requirements = DeltaDeclarations::default();
    let mut sections = DeltaDeclarations::default();
    let mut requirements_sections = DeltaDeclarations::default();
    for item in items {
        let target = match item.target {
            DeltaTarget::Requirement => &mut requirements,
            DeltaTarget::SpecSection => &mut sections,
            DeltaTarget::RequirementsSection => &mut requirements_sections,
        };
        match item.operation {
            DeltaOperation::Added => target.added.push(item.key.clone()),
            DeltaOperation::Modified => target.modified.push(item.key.clone()),
            DeltaOperation::Removed => target.removed.push(item.key.clone()),
        }
    }
    for declarations in [&mut requirements, &mut sections, &mut requirements_sections] {
        for list in [
            &mut declarations.added,
            &mut declarations.modified,
            &mut declarations.removed,
        ] {
            list.sort_unstable();
            list.dedup();
        }
    }
    (requirements, sections, requirements_sections)
}

/// Approval currency as the gates that consume the approval judge it (REQ-change-111).
///
/// For an approved, implementing or verifying change the approval is current only when the
/// effective definition approval still matches the scope digest AND the delta bodies it recorded
/// still match the delta files. Those are the two refusals materialization and acceptance give,
/// and status reading only the first sent the operator to a `change check` certain to refuse
/// (Chough BUG-0002). Every other state keeps the scope half alone, so draft, accepted and
/// archived verdicts and every archived ledger are not re-judged by the active-state binding.
///
/// The delta half runs only when the scope half holds: it resolves the same effective approval
/// first and would only repeat the scope refusal.
struct ApprovalCurrency {
    current: bool,
    /// Sorted modules whose approved delta body changed; empty unless the scope half holds.
    changed_approved_deltas: Vec<String>,
}

fn approval_currency(root: &Path, record: &ChangeRecord) -> ApprovalCurrency {
    let scope_current = ensure_definition_approval_valid(root, record).is_ok();
    if !scope_current
        || !matches!(
            record.state,
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
        )
    {
        return ApprovalCurrency {
            current: scope_current,
            changed_approved_deltas: Vec::new(),
        };
    }
    match changed_approved_delta_modules(root, record) {
        Ok(changed) => ApprovalCurrency {
            current: changed.is_empty(),
            changed_approved_deltas: changed,
        },
        Err(_) => ApprovalCurrency {
            current: false,
            changed_approved_deltas: Vec::new(),
        },
    }
}

/// Whether the change's definition approval still covers its approved scope and, for an
/// approved, implementing or verifying change, the semantic delta bodies it recorded. This is
/// the predicate behind `ChangeSummary.approval_valid`, exported for human next-action guidance.
/// It returns only a boolean built from comparisons, so no digest can reach a text sink.
pub fn approval_current_for_guidance(root: &Path, record: &ChangeRecord) -> bool {
    approval_currency(root, record).current
}

/// True when living `requirements.md` already has a `### {key}` requirement heading.
/// True when living `requirements.md` already has a `### {key}` requirement whose
/// content differs from what an `## ADDED` delta declares.
///
/// A byte-equivalent block is an already-applied delta and is not a conflict, so
/// re-deriving the canonical tree converges instead of failing.
/// Whether an ADDED block that conflicts with the living tree is this change's own earlier
/// text, judged exactly as `prepare_module_application` judges it before materialization
/// replaces the block: the latest send-backs list it as an applied ADDED item and the
/// canonical tree still carries that item's text. A block whose canonical text someone else
/// changed is not the change's own, so approve still refuses it (#1001).
fn sent_back_added_block_is_own_on_disk(
    root: &Path,
    record: &ChangeRecord,
    module: &str,
    item: &DeltaItem,
) -> Result<bool, String> {
    let ledger = load_approvals(root, record).unwrap_or_default();
    if ledger.send_backs.is_empty() {
        return Ok(false);
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    let (spec_path, requirements_path) = canonical_module_paths(root, &specs_dir, module)?;
    let spec = fs::read_to_string(&spec_path).unwrap_or_default();
    let requirements = fs::read_to_string(&requirements_path).unwrap_or_default();
    Ok(sent_back_added_block_is_own(
        &ledger,
        module,
        item,
        &spec,
        &requirements,
    ))
}

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
            let item_heading = strip_ascii_prefix_ignore_case(header, "REQUIREMENTS SECTION ")
                .map(|value| (DeltaTarget::RequirementsSection, value))
                .or_else(|| {
                    strip_ascii_prefix_ignore_case(header, "REQUIREMENT ")
                        .map(|value| (DeltaTarget::Requirement, value))
                })
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
                    "invalid delta item heading `### {header}` — a delta item must be `### REQUIREMENT <id>`, `### SPEC SECTION <name>` or `### REQUIREMENTS SECTION <name>`; subheadings are only content once an item has been opened"
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
                "semantic delta contains no items under a recognized operation heading; each ## Added, ## Modified, or ## Removed section must contain ### REQUIREMENT <id>, ### SPEC SECTION <name> or ### REQUIREMENTS SECTION <name>"
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

/// What materialization still owes the canonical tree.
///
/// `pending` names the modules whose canonical outputs were NOT all current, and `files` carries
/// rewritten contents for exactly those modules. A module with nothing outstanding contributes no
/// file at all, so re-running a settled materialization writes nothing — which is the property
/// that lets the caller keep short-circuiting instead of rewriting specs on every check.
#[derive(Debug, Default)]
struct PreparedMaterialization {
    files: Vec<(PathBuf, String)>,
    pending: Vec<String>,
}

/// Whether the canonical tree already carries what one delta item declares.
///
/// This is the question `apply_markdown_block` cannot answer, because it APPLIES: re-applying a
/// `## REMOVED` item to a block that is already gone is an error there, and rightly so on a first
/// run — it means the delta names something that was never present. Asked as a question instead,
/// "already removed" is simply true.
fn delta_item_is_applied(spec: &str, requirements: &str, item: &DeltaItem) -> bool {
    let source = delta_target_source(item.target, spec, requirements);
    let heading = format!("{}{}", delta_target_prefix(item.target), item.key);
    match delta_block_range(source, item.target, &item.key) {
        None => item.operation == DeltaOperation::Removed,
        Some(range) => {
            item.operation != DeltaOperation::Removed
                && markdown_block_matches(&source[range], &heading, &item.content)
        }
    }
}

/// Whether a module spec's Change Log already carries a row for this change.
///
/// The version bump and the Change Log row are written together, by the same two lines, exactly
/// once per (change, module). The row is therefore the only durable evidence that BOTH happened:
/// a bumped `version:` leaves nothing behind naming the change that bumped it, so an integer on
/// its own cannot distinguish "this change bumped it" from "some other change did".
///
/// `append_changelog` writes the description as `{id}: {title}` into whichever column the table
/// calls its description, so the row is found by that prefix inside the `## Change Log` section
/// rather than by rebuilding a row and comparing it. A table that `specsync compact` or a human
/// has since reformatted still counts, and the trailing colon keeps one slug from matching
/// another that merely extends it.
fn changelog_records_change(spec: &str, id: &str) -> bool {
    let Some(position) = spec.rfind("## Change Log") else {
        return false;
    };
    let start = position + "## Change Log".len();
    let end = spec[start..]
        .find("\n## ")
        .map_or(spec.len(), |offset| start + offset);
    spec[start..end].contains(&format!("{id}:"))
}

/// Materialization writes THREE outputs per module, and the `canonical_applied` short-circuit
/// above this function used to skip all three (#741): the delta applied to the canonical files,
/// the spec's `version:` bump, and the spec's Change Log row. `bump_spec_version` and
/// `append_changelog` have no other caller anywhere, so a delta corrected after review and
/// re-approved produced changed contract text with neither a bump nor a row — and every gate
/// downstream passed, because nothing below the flag ever ran to ask.
///
/// So the flag alone stopped being the answer to "is the canonical tree current?". The artefacts
/// answer it, per module and per output:
///
/// * the delta is applied when every item it declares is already reflected — a `## REMOVED` block
///   absent, an `## ADDED`/`## MODIFIED` block present with matching content;
/// * the bump and the row were written together, so the Change Log naming this change is the
///   evidence that both of them happened.
///
/// A module with both is not rewritten at all. A module missing either is materialized again, and
/// given only the halves it is missing: a corrected delta re-applies its wording without a second
/// bump and a second row, because one change bumps one module's version exactly once.
///
/// Convergence is scoped to `canonical_applied`. On a FIRST materialization every refusal
/// `apply_markdown_block` makes today still fires, unchanged — removing a block that was never
/// there is still an error, adding one that exists with different content is still an error. Only
/// once materialization has demonstrably run for this change does an already-reflected item
/// become a no-op instead, because only then is "absent" a state this change itself produced.
fn prepare_pending_delta_application(
    root: &Path,
    record: &ChangeRecord,
) -> Result<PreparedMaterialization, String> {
    let mut prepared = PreparedMaterialization::default();
    if record.no_spec_change {
        return Ok(prepared);
    }
    let specs_dir = crate::config::load_config(root).specs_dir;
    for module in &record.affected_specs {
        let delta =
            read_bounded_change_text(&delta_path_checked(root, record, module)?, "semantic delta")?;
        prepare_module_application(root, record, &specs_dir, module, &delta, &mut prepared)
            .map_err(|error| {
                let frozen = record.canonical_applied && load_approvals(root, record)
                    .is_ok_and(|ledger| !ledger.reopenings.is_empty());
                if frozen {
                    let detail = error.split(';').next().unwrap_or(&error);
                    format!("frozen audited-reopened change {} cannot replay its {module} delta: {detail}; preserve the approved definition and later canonical text; use a separately approved current-intent replacement and audited nonacceptance retirement, or hold while authenticated custody is unavailable", record.id)
                } else { error }
            })?;
    }
    Ok(prepared)
}

/// One module's step of `prepare_pending_delta_application`, on a delta body the caller read.
///
/// Materialization passes the body it just read; `approved_delta_summaries` passes the body the
/// approval binding digested, so `materialized` is answered for the same bytes as `delta_intact`.
/// Writes nothing: it only pushes the prepared files and, when the module has outstanding work,
/// the module onto `prepared.pending`.
fn prepare_module_application(
    root: &Path,
    record: &ChangeRecord,
    specs_dir: &str,
    module: &str,
    delta: &str,
    prepared: &mut PreparedMaterialization,
) -> Result<(), String> {
    // A change that was sent back may already carry its earlier scope's text in the canonical
    // tree, so it converges even though the send-back cleared `canonical_applied`.
    let ledger = load_approvals(root, record).unwrap_or_default();
    let converging = record.canonical_applied || !ledger.send_backs.is_empty();
    let items = parse_delta(delta)?;
    let (spec_path, requirements_path) = canonical_module_paths(root, specs_dir, module)?;
    let mut spec = fs::read_to_string(&spec_path)
        .map_err(|error| format!("failed to read {}: {error}", spec_path.display()))?;
    let mut requirements = fs::read_to_string(&requirements_path)
        .unwrap_or_else(|_| format!("---\nspec: {module}.spec.md\n---\n\n# Requirements\n"));
    // Read BEFORE applying. A delta may legitimately target `## Change Log`, and the question
    // is whether the row was already there, never whether this run has just written one.
    let recorded = changelog_records_change(&spec, &record.id);
    if converging
        && recorded
        && items
            .iter()
            .all(|item| delta_item_is_applied(&spec, &requirements, item))
    {
        return Ok(());
    }
    for item in &items {
        if converging && delta_item_is_applied(&spec, &requirements, item) {
            continue;
        }
        let operation =
            if sent_back_added_block_is_own(&ledger, module, item, &spec, &requirements) {
                DeltaOperation::Modified
            } else {
                item.operation
            };
        (spec, requirements) = apply_delta_item(&spec, &requirements, item, operation)?;
    }
    if !recorded {
        spec = bump_spec_version(&spec)?;
        spec = append_changelog(&spec, &record.id, &record.title);
    }
    prepared.files.push((spec_path, spec));
    prepared.files.push((requirements_path, requirements));
    prepared.pending.push(module.to_string());
    Ok(())
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
    let (header_line, body) = existing.split_once('\n').unwrap_or((&existing, ""));
    if !requirement_heading_line_matches(header_line, &normalize(heading)) {
        return false;
    }
    normalize(body) == normalize(content)
}

/// Whether `line` names the `heading` block: exactly the heading, or — for `### ` requirement
/// targets only — any header form the shared requirement parser accepts for the same id
/// (`### REQ-cmd-change-012 — Title`, `### REQ-x-001: Title`, up to three spaces of indent;
/// #1024, t384's review). The parser owns the id rule (`REQ-<module>-<number>`, multi-segment
/// module allowed), so the materializer never drifts from what every reader accepts. A `## `
/// section key matches only exactly, so `Change Log` never matches `## Change Log Archive`.
fn requirement_heading_line_matches(line: &str, heading: &str) -> bool {
    if line.trim_end() == heading {
        return true;
    }
    let Some(key) = heading.strip_prefix("### ") else {
        return false;
    };
    crate::validator::requirement_heading_id(line) == Some(key)
}

/// Byte range of the `heading` block in `source` — the heading line through to the next heading at
/// `level` or shallower — or `None` when `source` carries no such heading.
///
/// Extracted so that applying an item and asking whether an item is ALREADY applied read the same
/// block out of the same scan. Two answers derived from two scans is how a tree and the record of
/// it drift apart, which is the whole subject of #741.
fn markdown_block_range(source: &str, heading: &str, level: usize) -> Option<Range<usize>> {
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
        .position(|(_, line)| requirement_heading_line_matches(line, heading))?;
    let end = lines
        .iter()
        .skip(start + 1)
        .find(|(_, line)| markdown_heading_level(line).is_some_and(|found| found <= level))
        .map_or(source.len(), |(next_offset, _)| *next_offset);
    Some(lines[start].0..end)
}

/// Why a `### REQUIREMENTS SECTION` item cannot be approved, or `None` (#866, REQ-change-162): its
/// content holds a requirement heading outside fenced code, and a requirement changes only
/// through a `### REQUIREMENT` item, with its tombstones.
fn requirements_section_item_problem(item: &DeltaItem) -> Option<String> {
    if item.target != DeltaTarget::RequirementsSection {
        return None;
    }
    let scanned = crate::parser::blank_fenced_code(&item.content);
    let id = scanned
        .lines()
        .find_map(crate::validator::requirement_heading_id)?;
    Some(format!(
        "`### REQUIREMENTS SECTION {}` holds the requirement heading `{id}`; a requirement changes only through a `### REQUIREMENT {id}` item, so move it into one",
        item.key
    ))
}

/// The text an item targets: `requirements.md` for a requirement or a requirements section, the
/// canonical spec for a spec section.
fn delta_target_source<'a>(target: DeltaTarget, spec: &'a str, requirements: &'a str) -> &'a str {
    match target {
        DeltaTarget::SpecSection => spec,
        DeltaTarget::Requirement | DeltaTarget::RequirementsSection => requirements,
    }
}

/// The heading prefix an item's block is written under.
fn delta_target_prefix(target: DeltaTarget) -> &'static str {
    match target {
        DeltaTarget::Requirement => "### ",
        DeltaTarget::SpecSection | DeltaTarget::RequirementsSection => "## ",
    }
}

/// How a refusal names an item's target: a requirement by its ID, a section by its name.
fn delta_target_label(item: &DeltaItem) -> String {
    match item.target {
        DeltaTarget::Requirement => item.key.clone(),
        DeltaTarget::SpecSection => format!("section `{}`", item.key),
        DeltaTarget::RequirementsSection => format!("requirements section `{}`", item.key),
    }
}

/// The block `key` names in `source` for `target`, read the same way by application, the
/// already-applied check, the approved-base digest and send-back.
fn delta_block_range(source: &str, target: DeltaTarget, key: &str) -> Option<Range<usize>> {
    match target {
        DeltaTarget::RequirementsSection => requirements_section_range(source, key),
        DeltaTarget::Requirement | DeltaTarget::SpecSection => {
            let prefix = delta_target_prefix(target);
            markdown_block_range(source, &format!("{prefix}{key}"), markdown_prefix_level(prefix))
        }
    }
}

/// The `## <key>` section of `requirements.md` (#866, REQ-change-162): from its heading to the next
/// heading of level 2 or shallower or the next requirement heading, whichever comes first, so a
/// requirement block is never part of a section. A heading inside fenced code is quoted text.
fn requirements_section_range(source: &str, key: &str) -> Option<Range<usize>> {
    let range = markdown_block_range(source, &format!("## {key}"), 2)?;
    let scanned = crate::parser::blank_fenced_code(&source[range.clone()]);
    let mut offset = 0;
    for (index, line) in scanned.split_inclusive('\n').enumerate() {
        let content = line.trim_end_matches(['\n', '\r']);
        if index > 0 && crate::validator::requirement_heading_id(content).is_some() {
            return Some(range.start..range.start + offset);
        }
        offset += line.len();
    }
    Some(range)
}

/// Apply one delta item, with `operation`, to a module's spec and requirements text; returns
/// both texts, one of them unchanged.
fn apply_delta_item(
    spec: &str,
    requirements: &str,
    item: &DeltaItem,
    operation: DeltaOperation,
) -> Result<(String, String), String> {
    match item.target {
        DeltaTarget::Requirement => Ok((
            spec.to_string(),
            apply_markdown_block(requirements, "### ", &item.key, &item.content, operation)?,
        )),
        DeltaTarget::SpecSection => Ok((
            apply_markdown_block(spec, "## ", &item.key, &item.content, operation)?,
            requirements.to_string(),
        )),
        DeltaTarget::RequirementsSection => Ok((
            spec.to_string(),
            apply_markdown_block_at(
                requirements,
                "## ",
                &item.key,
                &item.content,
                operation,
                requirements_section_range(requirements, &item.key),
            )?,
        )),
    }
}

/// Heading depth a delta target writes at: `"## "` is 2, `"### "` is 3.
fn markdown_prefix_level(prefix: &str) -> usize {
    prefix
        .chars()
        .take_while(|character| *character == '#')
        .count()
}

fn apply_markdown_block(
    source: &str,
    prefix: &str,
    key: &str,
    content: &str,
    operation: DeltaOperation,
) -> Result<String, String> {
    let heading = format!("{prefix}{key}");
    let block = markdown_block_range(source, &heading, markdown_prefix_level(prefix));
    apply_markdown_block_at(source, prefix, key, content, operation, block)
}

/// `apply_markdown_block` with the targeted block's range already read, so a requirements section
/// applies with its own boundary (#866).
fn apply_markdown_block_at(
    source: &str,
    prefix: &str,
    key: &str,
    content: &str,
    operation: DeltaOperation,
    block: Option<Range<usize>>,
) -> Result<String, String> {
    let heading = format!("{prefix}{key}");
    let line_ending = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    match operation {
        // An ADDED block that is already present with exactly the declared content
        // is an applied delta, not a conflict. Re-deriving the canonical tree must
        // converge, otherwise the delta can never be reconciled against the tree
        // and a partially-applied run leaves the change permanently unverifiable.
        DeltaOperation::Added if block.is_some() => {
            if let Some(range) = block.clone()
                && markdown_block_matches(&source[range], &heading, content)
            {
                return Ok(source.to_string());
            }
            return Err(format!(
                "cannot add existing block `{key}` with different content; use ## MODIFIED to \
                 change a block already present in the living tree"
            ));
        }
        DeltaOperation::Modified | DeltaOperation::Removed if block.is_none() => {
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
        // A matched block keeps its own header line byte-for-byte (only the line ending's `\r`
        // comes off): a title suffix the parser accepted is the author's annotation, not drift
        // to normalize away (#1024, t384's review).
        let header = block
            .as_ref()
            .and_then(|range| source[range.clone()].lines().next())
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .filter(|line| !line.is_empty())
            .unwrap_or(&heading);
        format!("{header}{line_ending}{line_ending}{normalized_content}{line_ending}{line_ending}")
    };
    if let Some(range) = block {
        let mut output =
            String::with_capacity(source.len() - (range.end - range.start) + replacement.len());
        output.push_str(&source[..range.start]);
        output.push_str(&replacement);
        output.push_str(&source[range.end..]);
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
    // ATX headings may be indented by up to three spaces — four is indented code. The block-end
    // scan must see the same headings the matcher sees (t384's round-2 review of #1039): an
    // indented `### ` header ends the previous block exactly as an unindented one does.
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let line = &line[indent..];
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
        changes
            .push("public-contract, architecture-risk, or constraints declaration changed".into());
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
    // The relay gate is outside every digest: a relay-gated change hashes as the workflow-v2
    // change it is, so raising it stales nothing already recorded (REQ-change-158).
    canonical_record.workflow_version = digest_workflow_version(canonical_record.workflow_version);
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
        // Hashed as both correction digests hash it: without the relays its embedded approvals
        // carry, so a relay changes no gate (#927, REQ-change-relay-compare-001).
        let ledger = CorrectionLedger {
            schema_version: 1,
            corrections: corrections.iter().map(correction_digest_projection).collect(),
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
    // Line endings are not content for definition artifacts: Git blob capture is
    // LF, but post-move archive paths are untracked and `core.autocrlf=true`
    // feeds CRLF working-tree bytes into the same digest. Fold `\r\n` → `\n`
    // the same way `canonical_delta_body` does; a lone `\r` stays.
    let mut canonical = fold_crlf_to_lf(payload);
    if !relative.ends_with("/tasks.md") {
        return canonical;
    }

    for offset in markdown_task_checkbox_offsets(&canonical) {
        canonical[offset] = b' ';
    }
    canonical
}

fn fold_crlf_to_lf(payload: &[u8]) -> Vec<u8> {
    if !payload.contains(&b'\r') {
        return payload.to_vec();
    }
    let mut out = Vec::with_capacity(payload.len());
    let mut index = 0;
    while index < payload.len() {
        if payload[index] == b'\r' && payload.get(index + 1) == Some(&b'\n') {
            out.push(b'\n');
            index += 2;
        } else {
            out.push(payload[index]);
            index += 1;
        }
    }
    out
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
        // Without the embedded approvals' relayed decisions (#633), like `corrected_view_digest`.
        let bytes = serde_json::to_vec(&correction_digest_projection(correction))
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
    // An invalid `verification_ignored_paths` list is an invalid policy, and every caller that
    // propagates this error refuses (REQ-change-115). With the list absent or empty, discovery,
    // frames and domain are exactly as before, so the digest is byte-identical (REQ-change-114).
    let ignored = validated_verification_ignored_paths(root)?;
    // Name clean Git files by object id. The digest still changes when the
    // blob changes, and the payload bound applies only to bytes actually read
    // (dirty files and repositories that are not Git).
    let (paths, evidence) = if ignored.is_empty() {
        stable_discovered_evidence(root, None, &BTreeSet::new(), false, true)?
    } else {
        stable_discovered_evidence_excluding(root, &ignored)?
    };
    let mut digest = FramedDigest::new(PROJECT_DIGEST_DOMAIN);
    frame_verification_ignored_paths(&mut digest, &ignored);
    for relative in paths {
        if project_input_is_volatile(&relative) {
            continue;
        }
        let entry = evidence.entry(&relative)?;
        if entry.object_identity {
            digest.object_entry(
                &relative,
                acceptance_kind_bytes(&entry.kind),
                entry.mode,
                &entry.payload,
            );
        } else {
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

#[cfg(test)]
fn git_project_paths(root: &Path) -> Result<Option<Vec<String>>, String> {
    git_project_paths_excluding(root, &[])
}

/// `git_project_paths` without the paths under `excluded` validated prefixes. They are dropped
/// before the inventory bound is counted, so another tool's records never count toward it.
fn git_project_paths_excluding(
    root: &Path,
    excluded: &[String],
) -> Result<Option<Vec<String>>, String> {
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
        if git_path_bytes_are_volatile(path) || git_path_bytes_are_verification_ignored(path, excluded)
        {
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

#[cfg(test)]
fn strict_walk_project_paths(root: &Path) -> Result<Vec<String>, String> {
    strict_walk_project_paths_excluding(root, &[])
}

fn strict_walk_project_paths_excluding(
    root: &Path,
    excluded: &[String],
) -> Result<Vec<String>, String> {
    let mut prior = None;
    for attempt in 0..2 {
        let mut paths = Vec::new();
        let mut total = 0_usize;
        let entries = walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let relative = portable_project_path(root, entry.path());
                // A listed prefix is never descended into, so a tool writing there cannot trip
                // the stability retry below. A file named like an entry without its slash still
                // matches nothing, because only a directory's descendants carry the slash.
                relative.is_empty()
                    || !(project_input_is_volatile(&relative)
                        || (entry.file_type().is_dir()
                            && path_is_verification_ignored(&format!("{relative}/"), excluded)))
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
    identify_clean_blobs: bool,
) -> Result<(Vec<String>, GitEvidence), String> {
    stable_discovered_evidence_cached(
        root,
        scopes,
        extra_candidates,
        regular_files_only,
        identify_clean_blobs,
        &[],
    )
}

/// Whole-project discovery for the project-input digest with a validated, non-empty
/// `verification_ignored_paths` list: paths under a listed prefix are left out before capture,
/// so they are never read and never count toward the payload bound.
fn stable_discovered_evidence_excluding(
    root: &Path,
    excluded: &[String],
) -> Result<(Vec<String>, GitEvidence), String> {
    stable_discovered_evidence_cached(root, None, &BTreeSet::new(), false, true, excluded)
}

fn stable_discovered_evidence_cached(
    root: &Path,
    scopes: Option<&BTreeSet<String>>,
    extra_candidates: &BTreeSet<String>,
    regular_files_only: bool,
    identify_clean_blobs: bool,
    excluded: &[String],
) -> Result<(Vec<String>, GitEvidence), String> {
    let key = DiscoveredEvidenceCacheKey {
        scopes: scopes.map(|values| values.iter().cloned().collect()),
        extra_candidates: extra_candidates.iter().cloned().collect(),
        regular_files_only,
        identify_clean_blobs,
        excluded_prefixes: excluded.to_vec(),
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
        identify_clean_blobs,
        excluded,
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
    identify_clean_blobs: bool,
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
        identify_clean_blobs,
        &[],
        hook,
    )
}

fn stable_discovered_evidence_with_hook_internal<Hook>(
    root: &Path,
    scopes: Option<&BTreeSet<String>>,
    extra_candidates: &BTreeSet<String>,
    regular_files_only: bool,
    identify_clean_blobs: bool,
    excluded: &[String],
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
                    let inspected = inspect_git_candidates(
                        root,
                        candidates,
                        regular_files_only,
                        identify_clean_blobs,
                    )?;
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
                None => match git_project_paths_excluding(root, excluded)? {
                    Some(paths) => Ok(paths),
                    None => strict_walk_project_paths_excluding(root, excluded),
                },
            }
        };
        let mut before_paths = discover()?;
        before_paths.retain(|path| {
            (!project_input_is_volatile(path) || explicitly_scoped_path(scopes, path))
                && !path_is_verification_ignored(path, excluded)
        });
        before_paths.extend(extra_candidates.iter().cloned());
        before_paths.sort();
        before_paths.dedup();
        let before_candidates = before_paths.iter().cloned().collect();
        let (before_index, before) = capture(&before_candidates)?;
        hook(attempt, root);

        let mut after_paths = discover()?;
        after_paths.retain(|path| {
            (!project_input_is_volatile(path) || explicitly_scoped_path(scopes, path))
                && !path_is_verification_ignored(path, excluded)
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
        AcceptanceDiscovery::Full,
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
        AcceptanceDiscovery::Full,
    )
}

/// The production-source delivery inputs finalize would refuse for want of a declared owner,
/// read-only: the acceptance manifest finalize builds, over the deltas it would apply, with an
/// unowned production input falling to the exact delivery owner instead of failing (#986, #956).
fn unowned_production_delivery_inputs(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Vec<String>, String> {
    let prepared = prepare_pending_delta_application(root, record)?.files;
    let manifest = acceptance_manifest_legacy(root, record, &prepared)?;
    Ok(manifest
        .entries
        .iter()
        .filter(|entry| {
            // A reserved delivery path is signed with the exact delivery owner by design, and
            // finalize accepts it; only the fallback assignment marks an unowned input.
            path_is_production_source(root, &entry.path)
                && !path_is_recognized_delivery_metadata(&entry.path)
                && entry.owners.iter().all(|owner| owner == EXACT_DELIVERY_OWNER)
        })
        .map(|entry| entry.path.clone())
        .collect())
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
        AcceptanceDiscovery::Full,
    )
}

/// Recursive validation compares the authenticated manifest's entries only. Later additions
/// are not signed inputs; fresh acceptance and reopening still use full discovery.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AcceptanceDiscovery {
    Full,
    SignedInputs,
}

fn acceptance_manifest_for_signed_inputs(
    root: &Path,
    record: &ChangeRecord,
    signed: &AcceptanceManifestV1,
) -> Result<AcceptanceManifestV1, String> {
    acceptance_manifest_internal(
        root,
        record,
        &[],
        Some(signed),
        UnownedProductionSource::Reject,
        AcceptanceDiscovery::SignedInputs,
    )
}

fn acceptance_manifest_internal(
    root: &Path,
    record: &ChangeRecord,
    overrides: &[(PathBuf, String)],
    signed: Option<&AcceptanceManifestV1>,
    unowned_source: UnownedProductionSource,
    discovery: AcceptanceDiscovery,
) -> Result<AcceptanceManifestV1, String> {
    let discovery_scopes = match discovery {
        AcceptanceDiscovery::Full => acceptance_discovery_scopes(root, record)?,
        AcceptanceDiscovery::SignedInputs => BTreeSet::new(),
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
    let mut extra_candidates = override_content.keys().cloned().collect::<BTreeSet<_>>();
    if discovery == AcceptanceDiscovery::Full {
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
    }
    if let Some(signed) = signed {
        extra_candidates.extend(signed.entries.iter().map(|entry| entry.path.clone()));
    }
    // Clean files are named by object id and streamed one blob at a time below, so the scope's
    // clean bytes are never read in one batch (REQ-change-144).
    let (mut paths, mut evidence) = stable_discovered_evidence(
        root,
        Some(&discovery_scopes),
        &extra_candidates,
        false,
        true,
    )?;
    paths.retain(|path| {
        (!project_input_is_volatile(path) || path == LEGACY_BASELINE_PATH)
            && record_covers_project_path(root, record, path)
    });
    let historical_sequence_ledger = if record_covers_project_path(root, record, SEQUENCE_PATH) {
        historical_sequence_ledger_acceptance_content(root, record)?
    } else {
        None
    };
    let mut payloads = DeliveryPayloads::new(root);
    if signed.is_none() {
        payloads.materialize_affected_specs(record, &mut evidence)?;
    }
    let mut entries = Vec::new();
    for relative in paths {
        let (kind, mode, payload_digest) = if let Some(content) = override_content.get(&relative)
        {
            let mode = evidence.generated_file_mode(&relative)?;
            let kind = acceptance_kind_for_mode(mode);
            (kind, mode, sha256_hex(content))
        } else if relative == SEQUENCE_PATH
            && let Some(content) = &historical_sequence_ledger
        {
            (
                AcceptanceInputKind::File,
                evidence.generated_file_mode(&relative)?,
                sha256_hex(content),
            )
        } else {
            let captured = evidence.entry(&relative)?;
            (
                captured.kind.clone(),
                captured.mode,
                sha256_hex(&payloads.of(&relative, captured)?),
            )
        };
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
            historical_reconciliation: None,
        });
    }
    let base = record
        .base_commit
        .as_deref()
        .filter(|base| is_canonical_commit_id(base))
        .ok_or_else(|| {
            "semantic succession requires a definition-signed base commit (a full commit id)"
                .to_string()
        })?;
    let ancestor = crate::git_utils::status_without_inherited_output(
        Command::new("git")
            .args(["merge-base", "--is-ancestor", base, "HEAD"])
            .current_dir(root),
    )
    .map_err(|error| format!("failed to validate succession base ancestry: {error}"))?;
    if !ancestor.success() {
        return Err(format!(
            "semantic succession base commit `{base}` is not an ancestor of HEAD"
        ));
    }
    let historical = build_historical_reconciliation_evidence(root, record)?;
    let mut historical_manifests = BTreeMap::new();
    if historical.is_some() {
        for edge in &record.supersedes {
            let (manifest, _) = historical_predecessor_receipt(root, base, &edge.predecessor_id)?;
            historical_manifests.insert(edge.predecessor_id.clone(), manifest);
        }
    }
    let mut tuples = Vec::new();
    for edge in &record.supersedes {
        for obligation in &edge.obligations {
            let base_digest = if let Some(manifest) = historical_manifests.get(&edge.predecessor_id) {
                historical_supported_path(root, &obligation.path)?;
                let entry = manifest.entries.iter().find(|entry| entry.path == obligation.path)
                    .ok_or_else(|| format!("historical receipt has no input `{}`", obligation.path))?;
                historical_supported_entry(entry)?;
                entry.entry_digest.clone()
            } else {
                acceptance_entry_digest_at_commit(root, base, &edge.predecessor_id, &obligation.path)?
            };
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
            if historical.is_some() {
                historical_supported_entry(successor)?;
            }
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
        schema_version: if historical.is_some() { 2 } else { 1 },
        tuples,
        historical_reconciliation: historical,
    };
    validate_semantic_succession(record, &evidence)?;
    Ok(evidence)
}

fn validate_semantic_succession(
    record: &ChangeRecord,
    evidence: &SemanticSuccessionEvidenceV1,
) -> Result<(), String> {
    validate_historical_reconciliation_evidence(record, evidence)?;
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
    if let Some(historical) = &evidence.historical_reconciliation {
        let bytes = serde_json::to_vec(historical).map_err(|error| error.to_string())?;
        digest.frame(b"historical-reconciliation-unproven-v1", &bytes);
    }
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
