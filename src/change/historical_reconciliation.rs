const HISTORICAL_RECONCILIATION: &str = "historical_reconciliation";
const HISTORICAL_ANCHOR: &str = "historical_reconciliation_anchor";
const HISTORICAL_ACTOR: &str = "historical_reconciliation_actor";
const HISTORICAL_REASON: &str = "historical_reconciliation_reason";

/// Request a separately approved acknowledgement of immutable old receipts, never proof of
/// their historical file transitions. This changes scope only; no approval or closing gate runs.
pub fn enable_historical_reconciliation(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    let (mut record, _) = load_change_for_definition_mutation(root, id)?;
    require_state(
        &record,
        &[
            ChangeState::Draft,
            ChangeState::Approved,
            ChangeState::Implementing,
            ChangeState::Verifying,
        ],
        "request historical reconciliation",
    )?;
    let anchor = record.base_commit.clone().ok_or_else(|| {
        "historical reconciliation requires an immutable definition base".to_string()
    })?;
    record
        .answers
        .insert(HISTORICAL_RECONCILIATION.into(), "unproven".into());
    record.answers.insert(HISTORICAL_ANCHOR.into(), anchor);
    record
        .answers
        .insert(HISTORICAL_ACTOR.into(), normalize_actor(&actor, "reconcile-history")?);
    record
        .answers
        .insert(HISTORICAL_REASON.into(), reason.trim().into());
    // Validate every receipt before persisting anything. No live source bytes are substituted
    // for the frozen manifest, and normal acceptance authentication is not relaxed.
    build_historical_reconciliation_evidence(root, &record)?;
    record.updated_at = now();
    write_prepared_files(
        root,
        &[
            {
                let workspace = change_dir(root, id);
                let path = workspace.join("state.json");
                // A reconciled change that holds a relay stays gated (REQ-change-158).
                let content = json_content_keeping_unknown_members(
                    &path,
                    relay_gated_record(&workspace, &record).as_ref(),
                )?;
                (path, content)
            },
            (
                change_dir(root, id).join("change.md"),
                change_markdown_content(&record),
            ),
        ],
    )?;
    Ok(record)
}

fn historical_reconciliation_anchor(record: &ChangeRecord) -> Result<Option<&str>, String> {
    let Some(mode) = record.answers.get(HISTORICAL_RECONCILIATION) else {
        if [HISTORICAL_ANCHOR, HISTORICAL_ACTOR, HISTORICAL_REASON]
            .iter()
            .any(|key| record.answers.contains_key(*key))
        {
            return Err("historical reconciliation scope is incomplete".into());
        }
        return Ok(None);
    };
    if mode != "unproven"
        || record.workflow_version < 2
        || record.no_spec_change
        || record.supersedes.is_empty()
    {
        return Err("historical reconciliation requires an explicit unproven v2 successor scope with exact obligations".into());
    }
    let anchor = record
        .answers
        .get(HISTORICAL_ANCHOR)
        .ok_or_else(|| "historical reconciliation is missing its immutable anchor".to_string())?;
    if !is_canonical_commit_id(anchor) || record.base_commit.as_deref() != Some(anchor.as_str()) {
        return Err(
            "historical reconciliation anchor must equal the immutable definition base".into(),
        );
    }
    let actor = record
        .answers
        .get(HISTORICAL_ACTOR)
        .map(String::as_str)
        .unwrap_or("");
    let reason = record
        .answers
        .get(HISTORICAL_REASON)
        .map(String::as_str)
        .unwrap_or("");
    if actor.trim().is_empty()
        || actor.len() > 256
        || actor.trim() != actor
        || reason.trim().is_empty()
        || reason.len() > 4096
        || reason.trim() != reason
    {
        return Err(
            "historical reconciliation requires a bounded non-empty actor and reason".into(),
        );
    }
    Ok(Some(anchor))
}

fn historical_supported_path(root: &Path, path: &str) -> Result<(), String> {
    if strict_portable_relative_path(path)? != path {
        return Err("historical reconciliation path is not portable".into());
    }
    let protected = [
        ".git",
        ".specsync",
        ".github",
        ".fledge",
        ".codex",
        ".agents",
        "hi",
    ];
    let policy = load_policy_checked(root)?;
    if protected
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")))
        || is_protected_sdd_path(path)
        || [
            ".trust.toml",
            ".attest.json",
            ".augur.toml",
            "fledge.toml",
            "AGENTS.md",
            "INTENT.md",
        ]
        .contains(&path)
        || policy
            .as_ref()
            .and_then(|p| p.principles_file.as_deref())
            .is_some_and(|p| p == path)
        || policy
            .as_ref()
            .is_some_and(|p| p.custom_artifacts.values().any(|p| p == path))
    {
        return Err(format!(
            "historical reconciliation cannot cover protected policy input `{path}`"
        ));
    }
    Ok(())
}

fn historical_supported_entry(entry: &AcceptanceInputEntryV1) -> Result<(), String> {
    if !matches!(
        entry.kind,
        AcceptanceInputKind::File | AcceptanceInputKind::Missing | AcceptanceInputKind::NonFile
    ) {
        return Err(format!(
            "historical reconciliation does not support {:?} input `{}`",
            entry.kind, entry.path
        ));
    }
    Ok(())
}

fn historical_package_digest(
    snapshot: &BTreeMap<String, (u32, Vec<u8>)>,
) -> Result<String, String> {
    if snapshot.is_empty() || snapshot.len() > MAX_ACCEPTANCE_ENTRIES {
        return Err("historical receipt package is empty or exceeds its entry bound".into());
    }
    let mut digest = FramedDigest::new(b"specsync.historical-reconciliation.receipt.v1");
    for (path, (mode, bytes)) in snapshot {
        if !matches!(mode, 0o100644 | 0o100755) || bytes.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
            return Err("historical receipt must contain bounded regular files only".into());
        }
        digest.entry(path, b"file", *mode, bytes);
    }
    Ok(digest.finish())
}

/// Authenticate the current normal receipt, then require every original byte at the signed Git
/// anchor. Native archival may add files, but its authenticated accepted-state preimage must
/// reproduce the original state and all other original package bytes must still match.
fn historical_predecessor_receipt(
    root: &Path,
    anchor: &str,
    predecessor_id: &str,
) -> Result<HistoricalReceiptProof, String> {
    if !is_canonical_commit_id(anchor) {
        return Err("historical receipt anchor is not a canonical commit".into());
    }
    ensure_git_ancestor(root, anchor, "HEAD", "historical receipt scope")?;
    let predecessor = load_change(root, predecessor_id)?;
    if !matches!(
        predecessor.state,
        ChangeState::Accepted | ChangeState::Archived
    ) || historical_reconciliation_anchor(&predecessor)?.is_some()
    {
        return Err(
            "historical reconciliation requires an already accepted normal predecessor receipt"
                .into(),
        );
    }
    let verification = authenticate_accepted_evidence(root, &predecessor, None)?;
    let manifest = verification.acceptance_manifest.ok_or_else(|| {
        "historical reconciliation requires an existing frozen predecessor manifest".to_string()
    })?;
    let relative = format!("{CHANGES_PATH}/{predecessor_id}");
    let repo_relative = git_repo_relative_path(root, &relative)?;
    let snapshot = git_tree_snapshot(root, anchor, &repo_relative)?;
    let package_digest = historical_package_digest(&snapshot)?;
    let state = snapshot
        .get("state.json")
        .ok_or_else(|| "historical anchor has no accepted predecessor state".to_string())?;
    let anchored: ChangeRecord = serde_json::from_slice(&state.1).map_err(|e| e.to_string())?;
    if anchored.id != predecessor_id
        || anchored.state != ChangeState::Accepted
        || historical_reconciliation_anchor(&anchored)?.is_some()
    {
        return Err("historical anchor must contain the original active accepted receipt".into());
    }
    let workspace = find_change_dir(root, predecessor_id)?;
    let current_relative = strict_portable_project_path(root, &workspace)?;
    let mut current = archive_workspace_snapshot(root, &workspace, &current_relative)?;
    historical_package_digest(&current)?;
    if predecessor.state == ChangeState::Archived {
        let accepted = current.get("accepted-state.json").cloned().ok_or_else(|| {
            "archived historical receipt is missing its accepted-state preimage".to_string()
        })?;
        current.insert("state.json".into(), accepted);
        // Native archive regenerates change.md from the authenticated archived record.
        // Admit only that exact metadata projection; the immutable witness retains every
        // original markdown byte, and all other original files must still match below.
        if let Some(original_markdown) = snapshot.get("change.md") {
            let archived_markdown = current.get("change.md").ok_or_else(|| {
                "archived historical receipt is missing its generated change markdown".to_string()
            })?;
            if archived_markdown != original_markdown {
                if archived_markdown.0 != original_markdown.0
                    || archived_markdown.1 != change_markdown_content(&predecessor).as_bytes()
                {
                    return Err(
                        "archived historical receipt markdown is not the exact native projection"
                            .into(),
                    );
                }
                current.insert("change.md".into(), original_markdown.clone());
            }
        }
        if snapshot
            .iter()
            .any(|(path, entry)| current.get(path) != Some(entry))
        {
            return Err(
                "archived historical predecessor no longer preserves its anchored package".into(),
            );
        }
    } else if current != snapshot {
        return Err(
            "historical predecessor package does not exactly match its committed anchor".into(),
        );
    }
    let anchored_verification: VerificationRecord = serde_json::from_slice(
        &snapshot
            .get("verification.json")
            .ok_or_else(|| "historical anchor has no verification".to_string())?
            .1,
    )
    .map_err(|e| e.to_string())?;
    if anchored_verification.acceptance_manifest.as_ref() != Some(&manifest) {
        return Err("historical anchor does not contain the authenticated frozen manifest".into());
    }
    Ok((
        manifest.clone(),
        HistoricalReceiptWitnessV1 {
            predecessor_id: predecessor_id.into(),
            package_digest,
            manifest_digest: acceptance_manifest_digest(&manifest)?,
        },
    ))
}

fn build_historical_reconciliation_evidence(
    root: &Path,
    record: &ChangeRecord,
) -> Result<Option<HistoricalReconciliationEvidenceV1>, String> {
    let Some(anchor) = historical_reconciliation_anchor(record)? else {
        return Ok(None);
    };
    validate_supersedes_edges(record)?;
    validate_supersedes_semantics(root, record)?;
    let mut predecessors = Vec::new();
    for edge in &record.supersedes {
        let (manifest, witness) =
            historical_predecessor_receipt(root, anchor, &edge.predecessor_id)?;
        for obligation in &edge.obligations {
            historical_supported_path(root, &obligation.path)?;
            let entry = manifest
                .entries
                .iter()
                .find(|entry| entry.path == obligation.path)
                .ok_or_else(|| "historical receipt has no adopted input".to_string())?;
            historical_supported_entry(entry)?;
            if entry.entry_digest != obligation.predecessor_entry_digest {
                return Err("historical receipt entry does not match its exact obligation".into());
            }
        }
        predecessors.push(witness);
    }
    Ok(Some(HistoricalReconciliationEvidenceV1 {
        schema_version: 1,
        transition_status: "unproven".into(),
        anchor_commit: anchor.into(),
        predecessors,
    }))
}

fn validate_historical_reconciliation_evidence(
    record: &ChangeRecord,
    evidence: &SemanticSuccessionEvidenceV1,
) -> Result<(), String> {
    match (
        historical_reconciliation_anchor(record)?,
        evidence.historical_reconciliation.as_ref(),
    ) {
        (None, None) if evidence.schema_version == 1 => Ok(()),
        (Some(anchor), Some(historical)) if evidence.schema_version == 2 => {
            if historical.schema_version != 1
                || historical.transition_status != "unproven"
                || historical.anchor_commit != anchor
                || historical.predecessors.len() != record.supersedes.len()
            {
                return Err("historical reconciliation must retain its exact anchor and explicit unproven classification".into());
            }
            for (edge, witness) in record.supersedes.iter().zip(&historical.predecessors) {
                if witness.predecessor_id != edge.predecessor_id {
                    return Err(
                        "historical receipt witnesses must match the sorted approved predecessors"
                            .into(),
                    );
                }
                validate_sha256_digest(&witness.package_digest, "historical package digest")?;
                validate_sha256_digest(&witness.manifest_digest, "historical manifest digest")?;
            }
            Ok(())
        }
        _ => Err("historical and normal succession evidence cannot be interchanged".into()),
    }
}

/// Current successor bytes are still proven at its actual acceptance commit. Only ordinary
/// inputs are supported, so no special policy/ledger projection is bypassed by this direct read.
fn historical_entry_digest_at_commit(
    root: &Path,
    commit: &str,
    path: &str,
) -> Result<String, String> {
    historical_supported_path(root, path)?;
    let repo_path = git_repo_relative_path(root, path)?;
    let pathspec = format!(":(top,literal){repo_path}");
    let output = run_git_bounded(
        root,
        &["ls-tree", "-z", "--full-name", commit, "--", &pathspec],
        None,
        MAX_GIT_DIAGNOSTIC_BYTES,
    )?;
    if !output.status.success() {
        return Err("failed to inspect historical successor acceptance input".into());
    }
    let rows = output
        .stdout
        .split(|b| *b == 0)
        .filter(|row| !row.is_empty())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Ok(acceptance_entry_digest(
            path,
            &AcceptanceInputKind::Missing,
            0,
            &sha256_hex(b""),
        ));
    }
    if rows.len() != 1 {
        return Err("historical successor lookup was not exact".into());
    }
    let row = std::str::from_utf8(rows[0]).map_err(|e| e.to_string())?;
    let (metadata, actual_path) = row
        .split_once('\t')
        .ok_or_else(|| "invalid historical successor tree entry".to_string())?;
    if actual_path != repo_path {
        return Err("historical successor lookup escaped its exact path".into());
    }
    let fields = metadata.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 3 {
        return Err("invalid historical successor metadata".into());
    }
    let mode = u32::from_str_radix(fields[0], 8).map_err(|e| e.to_string())?;
    match (fields[1], mode) {
        ("tree", 0o040000) => Ok(acceptance_entry_digest(
            path,
            &AcceptanceInputKind::NonFile,
            0,
            &sha256_hex(b""),
        )),
        ("blob", 0o100644 | 0o100755) => {
            let bytes = git_blob_bytes_batch(root, &[fields[2]])?;
            Ok(acceptance_entry_digest(
                path,
                &AcceptanceInputKind::File,
                mode,
                &sha256_hex(&bytes[0]),
            ))
        }
        _ => Err(
            "historical reconciliation does not support symlink or gitlink successor inputs".into(),
        ),
    }
}

fn historical_terminal_summary() -> TerminalEvidenceSummary {
    TerminalEvidenceSummary {
        validity: TerminalEvidenceValidity::HistoricalTransitionUnproven,
        reason: Some("Original receipt bytes and current delivery are authenticated; historical file transitions remain unproven.".into()),
    }
}

type HistoricalReceiptProof = (AcceptanceManifestV1, HistoricalReceiptWitnessV1);
type AcceptedTransitionProof = (String, Vec<u8>, ChangeRecord);

/// Evidence reused only within one pure accepted-input graph traversal. Callers create a fresh
/// instance for each pass and discard it before any lifecycle write. This is the same lifetime
/// as the existing graph-validity memo, not the report snapshot or an invocation-wide cache.
#[derive(Default)]
struct AcceptedInputValidationMemo {
    validity: BTreeMap<String, Result<AcceptedInputValidity, String>>,
    /// Per accepted change, each changed input an in-flight later change covers for now, as
    /// `(path, covering change)` (#657).
    provisional: BTreeMap<String, Vec<(String, String)>>,
    authenticated: BTreeMap<String, Result<VerificationRecord, String>>,
    historical_receipts: BTreeMap<(String, String), Result<HistoricalReceiptProof, String>>,
    accepted_transitions: BTreeMap<String, Result<AcceptedTransitionProof, String>>,
}

impl AcceptedInputValidationMemo {
    fn authenticate(
        &mut self,
        root: &Path,
        record: &ChangeRecord,
        pending: Option<&PendingArchiveClose<'_>>,
    ) -> Result<VerificationRecord, String> {
        if let Some(value) = self.authenticated.get(&record.id) {
            return value.clone();
        }
        // An archive is relied on here, as a successor or a sequence owner, so its own
        // workflow-version history must hold. The full roster already proved that for every
        // archive (and memoized it); the structural roster finalize uses (REQ-change-124) did not,
        // so the archive this pass leans on is proven now, and only that one.
        let result = if record.state == ChangeState::Archived {
            validate_workflow_version_history(root, record)
                .and_then(|()| authenticate_accepted_evidence(root, record, pending))
        } else {
            authenticate_accepted_evidence(root, record, pending)
        };
        self.authenticated.insert(record.id.clone(), result.clone());
        result
    }

    fn historical_receipt(
        &mut self,
        root: &Path,
        anchor: &str,
        id: &str,
    ) -> Result<HistoricalReceiptProof, String> {
        let key = (anchor.to_string(), id.to_string());
        if let Some(value) = self.historical_receipts.get(&key) {
            return value.clone();
        }
        let result = historical_predecessor_receipt(root, anchor, id);
        self.historical_receipts.insert(key, result.clone());
        result
    }

    fn accepted_transition(
        &mut self,
        root: &Path,
        record: &ChangeRecord,
        pending: Option<&PendingArchiveClose<'_>>,
    ) -> Result<AcceptedTransitionProof, String> {
        if let Some(value) = self.accepted_transitions.get(&record.id) {
            return value.clone();
        }
        let result = authenticated_accepted_transition_for(root, record, pending);
        self.accepted_transitions
            .insert(record.id.clone(), result.clone());
        result
    }
}
