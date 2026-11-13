// Explicitly unproven custody, never acceptance or semantic succession.
const RETIREMENT_FILE: &str = "retirement.json";

/// A formerly accepted input retained under a current accepted custodian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetirementCustodyEntry {
    pub path: String,
    pub predecessor_entry_digest: String,
    pub predecessor_owners: Vec<String>,
    pub replacement_entry_digest: String,
    pub replacement_owners: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<RetirementCustodyClassification>,
}

/// Nonacceptance disposition. Actor/source are recorded claims, not human authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetirementRecord {
    pub schema_version: u32,
    pub domain: String,
    pub change_id: String,
    pub actor: String,
    pub reason: String,
    pub authority_source: String,
    pub timestamp: u64,
    pub original_state: String,
    pub original_state_mode: u32,
    pub original_package_digest: String,
    pub source_anchor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<RetirementSourceKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_directory: Option<String>,
    pub source_package_digest: String,
    pub source_review_untrusted: bool,
    pub prior_manifest_digest: String,
    pub replacement_id: String,
    pub replacement_closing_digest: String,
    pub replacement_manifest_digest: String,
    pub custody: Vec<RetirementCustodyEntry>,
    pub implementation_accepted: bool,
    pub historical_transition_status: String,
    pub digest: String,
}

fn retirement_digest(record: &RetirementRecord) -> Result<String, String> {
    let mut preimage = record.clone();
    preimage.digest.clear();
    let domain: &[u8] = match record.schema_version {
        1 => b"specsync.retirement.v1",
        2 => b"specsync.retirement.v2",
        _ => return Err("unknown retirement digest schema; upgrade reader".into()),
    };
    let mut digest = FramedDigest::new(domain);
    digest.frame(
        b"record",
        &serde_json::to_vec(&preimage).map_err(|e| e.to_string())?,
    );
    Ok(digest.finish())
}

fn retirement_text(value: &str, flag: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.trim() != value || value.len() > max {
        return Err(format!("retirement requires bounded non-empty {flag}"));
    }
    Ok(())
}

// Authenticate receipt custody, without rehabilitating the original inconsistent review.
fn retirement_source_header(
    root: &Path,
    original: &ChangeRecord,
) -> Result<(AcceptanceManifestV1, ReopenRecord), String> {
    if original.state != ChangeState::Verifying || !original.canonical_applied {
        return Err("retirement requires an audited-reopened verifying change".into());
    }
    // The retired change's own definition must be the one the reopening froze. This guards
    // retirement's retained-original claim, not the custody judgment, so it stays retire-side.
    ensure_definition_approval_valid(root, original)?;
    ensure_reopened_definition_unchanged(root, original)?;
    reopened_prior_acceptance(root, original)
}

/// The last accepted manifest of a change in an audited post-acceptance reopen, authenticated
/// by the same reopening evidence retirement requires. A reopened custodian is between its
/// last accepted evidence and its next; custody judged against the accepted manifest keeps
/// the roster loading while the prescribed recovery runs (#1021).
fn reopened_prior_acceptance(
    root: &Path,
    record: &ChangeRecord,
) -> Result<(AcceptanceManifestV1, ReopenRecord), String> {
    let ledger = load_approvals(root, record)?;
    let reopening = ledger
        .reopenings
        .last()
        .ok_or_else(|| "retirement requires an audited reopening".to_string())?;
    if reopening.schema_version != 1
        || reopening.change_id != record.id
        || reopening.actor.trim().is_empty()
        || reopening.reason.trim().is_empty()
        // `finalize` accepts and archives in one command, so a change needing
        // owner correction is Archived by the time anyone reopens it, and its
        // reopen legitimately records `from_state: archived` — the same
        // rationale the owner-correction read path in sdd_version.rs gives.
        || !matches!(
            reopening.from_state,
            ChangeState::Accepted | ChangeState::Archived
        )
        || reopening.to_state != ChangeState::Verifying
        || !reopening.prior_verification.passed
        // A reopening `reopen_change` wrote proves the accepted evidence went
        // stale: the acceptance-input digest moved, or the verification commit
        // left current history, or an authenticated committed review-mismatch
        // anchor drives the recovery (that anchor is itself verified against
        // the remote default and the committed package downstream). Without
        // this check a hand-edited workspace could append a syntactically
        // plausible reopening over still-current accepted evidence and retire
        // it into terminal nonacceptance. The manifest requirement below
        // already excludes the legacy unreconstructible cause (it is defined
        // by the manifest being absent), so the unanchored cause is the only
        // equal-digest reopening admissible here without a mismatch anchor.
        || (reopening.stale_acceptance_input_digest == reopening.current_acceptance_input_digest
            && reopening.recorded_review_mismatch_anchor.is_none()
            && reopening.stale_evidence_cause != Some(ReopenCauseV1::VerificationCommitUnanchored))
        || reopening.superseded_approval.digest
            != closing_digest(record, &reopening.prior_verification)
        || !is_terminal_approval(&reopening.superseded_approval)
        || reopening
            .prior_verification
            .acceptance_input_digest
            .as_deref()
            != Some(reopening.stale_acceptance_input_digest.as_str())
    {
        return Err(
            "retirement requires intact audited reopening evidence and prior definition".into(),
        );
    }
    validate_verification_execution_digest(root, record, &reopening.prior_verification)?;
    let manifest = reopening
        .prior_verification
        .acceptance_manifest
        .clone()
        .ok_or_else(|| {
            "retirement requires a frozen prior manifest; unknown history remains blocked"
                .to_string()
        })?;
    validate_acceptance_manifest(&manifest)?;
    if acceptance_manifest_digest(&manifest)? != reopening.stale_acceptance_input_digest {
        return Err(
            "reopening prior manifest does not reproduce its recorded acceptance-input digest"
                .into(),
        );
    }
    let closing = serde_json::to_vec(&reopening.superseded_approval).map_err(|e| e.to_string())?;
    if !ledger
        .approvals
        .iter()
        .any(|approval| serde_json::to_vec(approval).is_ok_and(|bytes| bytes == closing))
    {
        return Err("reopening no longer retains its superseded closing approval".into());
    }
    Ok((manifest, reopening.clone()))
}

fn retirement_source(
    root: &Path,
    original: &ChangeRecord,
) -> Result<(AcceptanceManifestV1, String, String, bool, String), String> {
    let (manifest, reopening) = retirement_source_header(root, original)?;
    let expected = serde_json::to_vec(&reopening.prior_verification).map_err(|e| e.to_string())?;
    let anchors = if let Some(anchor) = &reopening.recorded_review_mismatch_anchor {
        let default = remote_default_ref(root).ok_or_else(|| {
            "cannot authenticate inconsistent review without remote default".to_string()
        })?;
        if !is_canonical_commit_id(anchor) {
            return Err("retirement inconsistent review anchor is not a full commit id".into());
        }
        ensure_git_ancestor(root, anchor, &default, "retirement inconsistent review")?;
        vec![anchor.clone()]
    } else {
        accepted_recording_anchors(root, original)?
    };
    for anchor in anchors {
        ensure_git_ancestor(root, &anchor, "HEAD", "retirement source receipt")?;
        for directory in historical_change_directories(root, &anchor, &original.id)? {
            let snapshot = git_tree_snapshot(root, &anchor, &directory)?;
            let Some((_, state_bytes)) = snapshot.get("state.json") else {
                continue;
            };
            let Ok(state) = serde_json::from_slice::<ChangeRecord>(state_bytes) else {
                continue;
            };
            if state.id != original.id || state.state != ChangeState::Accepted {
                continue;
            }
            validate_loaded_change(&state, &original.id, Path::new(&directory))?;
            if !definition_digest_matches(
                root,
                &state,
                &reopening.prior_verification.contract_digest,
            )? {
                continue;
            }
            let Some((_, verification_bytes)) = snapshot.get("verification.json") else {
                continue;
            };
            let Ok(verification) = serde_json::from_slice::<VerificationRecord>(verification_bytes)
            else {
                continue;
            };
            if serde_json::to_vec(&verification).map_err(|e| e.to_string())? != expected {
                continue;
            }
            let Some((_, approval_bytes)) = snapshot.get("approvals.json") else {
                continue;
            };
            let approvals: ApprovalLedger =
                serde_json::from_slice(approval_bytes).map_err(|e| e.to_string())?;
            if !approvals.approvals.iter().any(|a| {
                a.gate == reopening.superseded_approval.gate
                    && a.actor == reopening.superseded_approval.actor
                    && a.timestamp == reopening.superseded_approval.timestamp
                    && a.digest == reopening.superseded_approval.digest
                    && a.note == reopening.superseded_approval.note
            }) {
                continue;
            }
            return Ok((
                manifest,
                anchor,
                historical_package_digest(&snapshot)?,
                reopening.recorded_review_mismatch_anchor.is_some(),
                directory
                    .strip_prefix(&git_repo_prefix(root)?)
                    .ok_or_else(|| {
                        "accepted source directory escaped selected project".to_string()
                    })?
                    .to_string(),
            ));
        }
    }
    Err(
        "retirement cannot authenticate the original accepted receipt at an intact ancestor anchor"
            .into(),
    )
}

fn retirement_custody(
    root: &Path,
    original: &ChangeRecord,
    replacement_id: &str,
    manifest: &AcceptanceManifestV1,
    records: &BTreeMap<String, ChangeRecord>,
    schema_version: u32,
) -> Result<(Vec<RetirementCustodyEntry>, String, String), String> {
    let replacement = records
        .get(replacement_id)
        .ok_or_else(|| format!("replacement change {replacement_id} not found"))?;
    if replacement.id == original.id
        || !matches!(
            replacement.state,
            ChangeState::Accepted | ChangeState::Archived
        )
    {
        return Err("retirement replacement must be independently accepted or archived".into());
    }
    if dependency_reaches(root, &replacement.id, &original.id, &mut BTreeSet::new()) {
        return Err("retirement replacement cannot depend on the obligation it retires".into());
    }
    let mut memo = AcceptedInputValidationMemo::default();
    let validity = validate_accepted_inputs_recursive(
        root,
        replacement,
        records,
        &mut BTreeSet::new(),
        &mut memo,
        None,
    )
    .map_err(|e| format!("retirement replacement has stale or unauthenticated evidence: {e}"))?;
    if validity == AcceptedInputValidity::ProvisionallyCovered {
        return Err(
            "retirement replacement's delivery inputs are covered only for now, by in-flight changes; accept those first"
                .into(),
        );
    }
    let verification = memo.authenticate(root, replacement, None)?;
    let current = verification
        .acceptance_manifest
        .as_ref()
        .ok_or_else(|| "retirement replacement requires a signed current manifest".to_string())?;
    let mut custody = Vec::new();
    for prior in &manifest.entries {
        let next = current
            .entries
            .iter()
            .find(|entry| entry.path == prior.path)
            .ok_or_else(|| format!("retirement replacement has no custody for {}", prior.path))?;
        let classification = if schema_version == 1 {
            if next.owners != prior.owners {
                return Err(format!(
                    "legacy retirement cannot transfer signed owners of {}",
                    prior.path
                ));
            }
            None
        } else {
            Some(retirement_entry_classification(prior, next)?)
        };
        custody.push(RetirementCustodyEntry {
            path: prior.path.clone(),
            predecessor_entry_digest: prior.entry_digest.clone(),
            predecessor_owners: prior.owners.clone(),
            replacement_entry_digest: next.entry_digest.clone(),
            replacement_owners: next.owners.clone(),
            classification,
        });
    }
    Ok((
        custody,
        closing_digest(replacement, &verification),
        acceptance_manifest_digest(current)?,
    ))
}

fn validate_retirement_with_records(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
) -> Result<(), String> {
    if record.state != ChangeState::Retired {
        return Err("change is not retired".into());
    }
    let directory = find_change_dir(root, &record.id)?;
    let receipt: RetirementRecord = serde_json::from_str(&read_bounded_change_text(
        &directory.join(RETIREMENT_FILE),
        "retirement receipt",
    )?)
    .map_err(|e| format!("invalid retirement receipt: {e}"))?;
    for (text, flag, limit) in [
        (receipt.actor.as_str(), "--actor", 256),
        (receipt.reason.as_str(), "--reason", 4096),
        (
            receipt.authority_source.as_str(),
            "--authority-source",
            4096,
        ),
    ] {
        retirement_text(text, flag, limit)?;
    }
    let format_valid = match receipt.schema_version {
        1 => {
            receipt.domain == "specsync.retirement.v1"
                && receipt.source_kind.is_none()
                && receipt.source_directory.is_none()
                && receipt
                    .custody
                    .iter()
                    .all(|entry| entry.classification.is_none())
        }
        2 => {
            receipt.domain == "specsync.retirement.v2"
                && receipt.source_kind.is_some()
                && receipt.source_directory.is_some()
                && receipt
                    .custody
                    .iter()
                    .all(|entry| entry.classification.is_some())
        }
        _ => false,
    };
    if !format_valid
        || receipt.change_id != record.id
        || receipt.implementation_accepted
        || receipt.historical_transition_status != "unproven"
        || receipt.digest != retirement_digest(&receipt)?
    {
        return Err("retirement receipt identity, classification or digest is invalid".into());
    }
    let expected_location = root.join(ARCHIVE_PATH).join(format!(
        "{}-{}",
        utc_date(receipt.timestamp),
        receipt.change_id
    ));
    if directory != change_dir(root, &record.id) && directory != expected_location {
        return Err(
            "retirement package location differs from its immutable disposition date".into(),
        );
    }
    let original: ChangeRecord = serde_json::from_str(&receipt.original_state)
        .map_err(|e| format!("invalid retirement original state: {e}"))?;
    let mut expected = original.clone();
    expected.state = ChangeState::Retired;
    expected.updated_at = receipt.timestamp;
    if serde_json::to_value(&expected).map_err(|e| e.to_string())?
        != serde_json::to_value(record).map_err(|e| e.to_string())?
    {
        return Err("retired state differs from its preserved original definition".into());
    }
    let relative = portable_project_path(root, &directory);
    let mut snapshot = archive_workspace_snapshot(root, &directory, &relative)?;
    snapshot.remove(RETIREMENT_FILE);
    if snapshot
        .get("state.json")
        .is_none_or(|entry| entry.0 != receipt.original_state_mode)
    {
        return Err("retirement state mode differs from its preserved original".into());
    }
    snapshot.insert(
        "state.json".into(),
        (
            receipt.original_state_mode,
            receipt.original_state.as_bytes().to_vec(),
        ),
    );
    if historical_package_digest(&snapshot)? != receipt.original_package_digest {
        return Err("retirement no longer preserves the exact original reopened package".into());
    }
    let source_kind = receipt
        .source_kind
        .unwrap_or(RetirementSourceKind::AcceptedReceipt);
    let expected = receipt
        .source_directory
        .as_deref()
        .map(|directory| (receipt.source_anchor.as_str(), directory));
    let proof = retirement_source_proof(root, &original, &snapshot, source_kind, expected)?;
    if receipt.source_anchor != proof.anchor
        || receipt.source_package_digest != proof.package_digest
        || receipt.source_review_untrusted != proof.review_untrusted
        || receipt.prior_manifest_digest != acceptance_manifest_digest(&proof.manifest)?
    {
        return Err("retirement historical receipt custody does not authenticate".into());
    }
    retirement_custody_holds(root, &original, &receipt, &proof.manifest, records)
}

/// Judge a recorded receipt against the custodian's current authenticated evidence.
///
/// Validation deliberately differs from retire time in two ways. First, it does not re-run
/// the custodian's live delivery-input freshness: staleness is the custodian's own reported
/// lifecycle state, and refusing the roster here deadlocked the `change reopen` remedy the
/// staleness report names (#1021). Second, it never re-derives a receipt's recorded
/// classification under today's rules: the receipt is pinned to committed history, so its
/// classification is a historical fact checked for consistency with the receipt's own
/// fields. Custody then holds while the custodian's authenticated manifest still carries
/// every retained path under the recorded signed owners; byte and digest evolution from a
/// genuine audited re-acceptance is not a custody break.
fn retirement_custody_holds(
    root: &Path,
    original: &ChangeRecord,
    receipt: &RetirementRecord,
    prior_manifest: &AcceptanceManifestV1,
    records: &BTreeMap<String, ChangeRecord>,
) -> Result<(), String> {
    let replacement = records
        .get(&receipt.replacement_id)
        .ok_or_else(|| format!("replacement change {} not found", receipt.replacement_id))?;
    if replacement.id == original.id {
        return Err("retirement replacement must be independently accepted or archived".into());
    }
    if dependency_reaches(root, &replacement.id, &original.id, &mut BTreeSet::new()) {
        return Err("retirement replacement cannot depend on the obligation it retires".into());
    }
    // A custodian in an audited post-acceptance reopen is between its last accepted evidence
    // and its next: custody is judged against the authenticated last accepted manifest so the
    // roster keeps loading while the prescribed recovery runs (#1021). The lineage checks bind
    // the manifest to the recorded reopening, and the prior evidence must still anchor in
    // committed history. Every other state has no accepted evidence to hold custody with.
    let current = match replacement.state {
        ChangeState::Accepted | ChangeState::Archived => {
            let mut memo = AcceptedInputValidationMemo::default();
            let verification = memo.authenticate(root, replacement, None).map_err(|e| {
                format!("retirement replacement evidence does not authenticate: {e}")
            })?;
            verification.acceptance_manifest.clone().ok_or_else(|| {
                "retirement replacement requires a signed current manifest".to_string()
            })?
        },
        ChangeState::Draft
        | ChangeState::Approved
        | ChangeState::Implementing
        | ChangeState::Verifying => {
            let (manifest, reopening) = reopened_prior_acceptance(root, replacement)
                .map_err(|e| {
                    format!(
                        "retirement replacement's last accepted evidence does not authenticate during its audited reopen: {e}"
                    )
                })?;
            if !accepted_evidence_is_anchored(root, replacement, &reopening.prior_verification) {
                return Err(
                    "retirement replacement's last accepted evidence is not anchored in current history during its audited reopen"
                        .into(),
                );
            }
            manifest
        },
        _ => {
            return Err(
                "retirement replacement must be independently accepted or archived".into(),
            );
        },
    };
    if receipt.custody.len() != prior_manifest.entries.len() {
        return Err("retirement custody does not cover exactly the retained signed inputs".into());
    }
    for prior in &prior_manifest.entries {
        let entry = receipt
            .custody
            .iter()
            .find(|candidate| candidate.path == prior.path)
            .ok_or_else(|| {
                format!(
                    "retirement receipt has no custody for retained input {}",
                    prior.path
                )
            })?;
        if prior.entry_digest != entry.predecessor_entry_digest
            || prior.owners != entry.predecessor_owners
        {
            return Err(format!(
                "retirement custody for {} does not match the retained original manifest",
                entry.path
            ));
        }
        let consistent = match entry.classification {
            None => {
                receipt.schema_version == 1
                    && entry.predecessor_owners == entry.replacement_owners
            },
            Some(RetirementCustodyClassification::SameSignedOwners) => {
                receipt.schema_version == 2
                    && entry.predecessor_owners == entry.replacement_owners
            },
            Some(RetirementCustodyClassification::UnchangedExactFileCurrentOwner) => {
                receipt.schema_version == 2
                    && entry.predecessor_owners == ["@exact:delivery".to_string()]
                    && entry.predecessor_entry_digest == entry.replacement_entry_digest
                    && !entry.replacement_owners.is_empty()
                    && entry
                        .replacement_owners
                        .iter()
                        .all(|owner| !owner.starts_with("@exact:"))
            },
        };
        if !consistent {
            return Err(format!(
                "retirement custody classification for {} is inconsistent with its recorded owners and digests",
                entry.path
            ));
        }
        let next = current
            .entries
            .iter()
            .find(|candidate| candidate.path == entry.path)
            .ok_or_else(|| format!("retirement replacement has no custody for {}", entry.path))?;
        if next.owners != entry.replacement_owners {
            return Err(format!(
                "retirement replacement no longer holds {} under its recorded signed owners",
                entry.path
            ));
        }
    }
    Ok(())
}

/// Validate retained original obligations and the current accepted custodian.
pub fn validate_retirement(root: &Path, record: &ChangeRecord) -> Result<(), String> {
    let records = list_all_changes_structural(root)?;
    validate_retirement_with_records(root, record, &records)
}

/// Terminal nonacceptance. Prior artifacts and failures remain unchanged, with original state
/// bytes retained verbatim. A retirement is never an ordinary accepted successor.
pub fn retire_change_with_source(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
    replacement: String,
    authority_source: String,
    source_kind: RetirementSourceKind,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    // A retirement already recorded resumes with exactly the actor its receipt holds, even one an
    // older SpecSync accepted and `normalize_actor` would now refuse: recorded history is never
    // re-judged (#1026 review). A new retirement takes a normalized actor.
    let recorded_actor = load_change(root, id)
        .ok()
        .filter(|record| {
            record.state == ChangeState::Retired
                || find_change_dir(root, id).is_ok_and(|dir| {
                    fs::symlink_metadata(dir.join(RETIREMENT_FILE)).is_ok()
                })
        })
        .and_then(|record| retirement_record(root, &record).ok())
        .map(|receipt| receipt.actor);
    let actor = match recorded_actor {
        Some(recorded) if recorded == actor => actor,
        _ => normalize_actor(&actor, "retirement")?,
    };
    for (text, flag, limit) in [
        (actor.as_str(), "--actor", 256),
        (reason.as_str(), "--reason", 4096),
        (authority_source.as_str(), "--authority-source", 4096),
    ] {
        retirement_text(text, flag, limit)?;
    }
    let mut record = load_change(root, id)?;
    let current = find_change_dir(root, id)?;
    let has_receipt = fs::symlink_metadata(current.join(RETIREMENT_FILE)).is_ok();
    if record.state == ChangeState::Retired || has_receipt {
        let receipt = retirement_record(root, &record)?;
        if receipt
            .source_kind
            .unwrap_or(RetirementSourceKind::AcceptedReceipt)
            != source_kind
        {
            return Err("retirement already records a different immutable source kind".into());
        }
        if receipt.actor != actor
            || receipt.reason != reason
            || receipt.authority_source != authority_source
            || receipt.replacement_id != replacement
        {
            return Err("retirement already records a different immutable disposition".into());
        }
        // Recover the old failed-rollback layout only when its state is EXACTLY the preserved
        // original. All immutable receipt/history and replacement checks still run before writes.
        if record.state != ChangeState::Retired {
            if current != change_dir(root, id)
                || fs::read(current.join("state.json")).map_err(|e| e.to_string())?
                    != receipt.original_state.as_bytes()
            {
                return Err(
                    "incomplete retirement does not retain its exact original state".into(),
                );
            }
            record.state = ChangeState::Retired;
            record.updated_at = receipt.timestamp;
        }
        let mut proposed = list_all_change_metadata(root, false)?;
        proposed.insert(id.into(), record.clone());
        validate_retired_records(root, &proposed)?;
        if current == change_dir(root, id) {
            write_prepared_files(
                root,
                &[(json_content_keeping_unknown_members(&current.join("state.json"), relay_gated_record(&current, &record).as_ref())
                    .map(|content| (current.join("state.json"), content))?)],
            )?;
            finish_retirement_move(root, &current, &receipt)?;
        }
        return Ok(record);
    }
    let source = change_dir(root, id);
    reject_symlink_components_for(root, &source, "retirement source")?;
    if find_change_dir(root, id)? != source {
        return Err("retirement requires an active audited-reopened workspace".into());
    }
    let snapshot =
        archive_workspace_snapshot(root, &source, &portable_project_path(root, &source))?;
    let proof = retirement_source_proof(root, &record, &snapshot, source_kind, None)?;
    let records = list_all_changes_structural(root)?;
    let (custody, replacement_closing_digest, replacement_manifest_digest) =
        retirement_custody(root, &record, &replacement, &proof.manifest, &records, 2)?;
    if snapshot.contains_key(RETIREMENT_FILE) {
        return Err("active workspace already contains a retirement receipt".into());
    }
    let (original_state_mode, original_state_bytes) = snapshot
        .get("state.json")
        .ok_or_else(|| "retirement original state is missing".to_string())?;
    let mut receipt = RetirementRecord {
        schema_version: 2,
        domain: "specsync.retirement.v2".into(),
        change_id: id.into(),
        actor,
        reason,
        authority_source,
        timestamp: now(),
        original_state: String::from_utf8(original_state_bytes.clone())
            .map_err(|e| e.to_string())?,
        original_state_mode: *original_state_mode,
        original_package_digest: historical_package_digest(&snapshot)?,
        source_anchor: proof.anchor,
        source_kind: Some(source_kind),
        source_directory: Some(proof.directory),
        source_package_digest: proof.package_digest,
        source_review_untrusted: proof.review_untrusted,
        prior_manifest_digest: acceptance_manifest_digest(&proof.manifest)?,
        replacement_id: replacement,
        replacement_closing_digest,
        replacement_manifest_digest,
        custody,
        implementation_accepted: false,
        historical_transition_status: "unproven".into(),
        digest: String::new(),
    };
    receipt.digest = retirement_digest(&receipt)?;
    let encoded_receipt = json_content(&receipt)?;
    if encoded_receipt.len() as u64 > MAX_CHANGE_ARTIFACT_BYTES {
        return Err(
            "retirement receipt exceeds the bounded readable artifact size; nothing was changed"
                .into(),
        );
    }
    let destination = root
        .join(ARCHIVE_PATH)
        .join(format!("{}-{id}", utc_date(receipt.timestamp)));
    validate_late_close_destination(root, &destination)?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    validate_late_close_destination(root, &destination)?;
    record.state = ChangeState::Retired;
    record.updated_at = receipt.timestamp;
    write_prepared_files(
        root,
        &[
            (json_content_keeping_unknown_members(&source.join("state.json"), relay_gated_record(&source, &record).as_ref()).map(|content| (source.join("state.json"), content))?),
            (source.join(RETIREMENT_FILE), encoded_receipt),
        ],
    )?;
    // Keep the journaled state and receipt together if rename fails. This is a recorded
    // nonacceptance whose archive move is pending, never a claim that nothing changed.
    finish_retirement_move(root, &source, &receipt)?;
    Ok(record)
}

/// Explain terminal nonacceptance and any broken retained-custody obligation.
pub fn retirement_next_action(root: &Path, record: &ChangeRecord) -> String {
    match validate_retirement(root, record) {
        Ok(()) if change_dir(root, &record.id).is_dir() =>
            "retirement custody is recorded but its archive move is pending; rerun the identical change retire command; no implementation acceptance".into(),
        Ok(()) => "retired without implementation acceptance; historical transitions remain unproven; the independent replacement owns current delivery".into(),
        Err(error) => format!("retirement custody is invalid: {error}; restore the retained receipt from trusted history, then run specsync change status {}", record.id),
    }
}

fn retirement_history(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    // Literal glob components prevent a project subdirectory from changing the pathspec.
    let literal = |value: &str| {
        value
            .chars()
            .flat_map(|c| {
                if matches!(c, '*' | '?' | '[' | '\\') {
                    vec!['\\', c]
                } else {
                    vec![c]
                }
            })
            .collect::<String>()
    };
    if !git_repository_present(root)? {
        return Ok(BTreeMap::new());
    }
    let prefix = git_repo_prefix(root)?;
    let active = format!(
        ":(top,glob){}/*/retirement.json",
        literal(&format!("{prefix}{CHANGES_PATH}"))
    );
    let archived = format!(
        ":(top,glob){}/*/retirement.json",
        literal(&format!("{prefix}{ARCHIVE_PATH}"))
    );
    let limit = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
    let output = run_git_bounded(
        root,
        &[
            "log",
            "--format=%H",
            &limit,
            "-G",
            "specsync[.]retirement[.]v",
            "HEAD",
            "--",
            &active,
            &archived,
        ],
        None,
        MAX_GIT_COMMAND_OUTPUT_BYTES,
    )?;
    if !output.status.success() {
        // A valid unborn branch has no committed history. Probe only after the
        // failed log, preserving all bounded-command errors and corrupt-HEAD refusals.
        if head_is_unborn(root)? {
            return Ok(BTreeMap::new());
        }
        return Err("cannot read committed retirement history".into());
    }
    let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let commits = text.lines().filter(|s| !s.is_empty()).collect::<Vec<_>>();
    if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
        return Err("committed retirement history exceeds its deterministic bound".into());
    }
    let mut receipts = BTreeMap::new();
    for commit in commits {
        let active_root = format!(":(top,literal){prefix}{CHANGES_PATH}");
        let archive_root = format!(":(top,literal){prefix}{ARCHIVE_PATH}");
        let tree = run_git_bounded(
            root,
            &[
                "ls-tree",
                "-r",
                "-z",
                "--full-name",
                "--name-only",
                commit,
                "--",
                &active_root,
                &archive_root,
            ],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )?;
        if !tree.status.success() {
            return Err("cannot enumerate committed retirement receipts".into());
        }
        for path in nul_terminated_git_paths(&tree.stdout, "retirement history")? {
            if !path.ends_with("/retirement.json") {
                continue;
            }
            let bytes = git_file_at_commit(root, commit, &path)?
                .ok_or_else(|| "committed retirement receipt is unavailable".to_string())?;
            let receipt: RetirementRecord = serde_json::from_slice(&bytes)
                .map_err(|e| format!("invalid committed retirement receipt: {e}"))?;
            validate_change_id(&receipt.change_id)?;
            if !matches!(
                (receipt.schema_version, receipt.domain.as_str()),
                (1, "specsync.retirement.v1") | (2, "specsync.retirement.v2")
            ) {
                return Err("unknown committed retirement format; upgrade the reader".into());
            }
            if let Some(prior) = receipts.insert(receipt.change_id.clone(), bytes.clone())
                && prior != bytes
            {
                return Err(format!(
                    "committed retirement history diverges for {}",
                    receipt.change_id
                ));
            }
        }
    }
    Ok(receipts)
}

fn validate_retired_records(
    root: &Path,
    records: &BTreeMap<String, ChangeRecord>,
) -> Result<(), String> {
    for (id, committed) in retirement_history(root)? {
        let record = records
            .get(&id)
            .ok_or_else(|| format!("committed retirement history for {id} was removed"))?;
        if record.state != ChangeState::Retired {
            return Err(format!(
                "committed retirement history for {id} was rolled back"
            ));
        }
        let bytes = fs::read(find_change_dir(root, &id)?.join(RETIREMENT_FILE))
            .map_err(|e| format!("committed retirement receipt is missing: {e}"))?;
        if bytes != committed {
            return Err(format!(
                "committed retirement history for {id} was edited; restore its exact anchored bytes"
            ));
        }
    }
    for record in records.values().filter(|r| r.state == ChangeState::Retired) {
        validate_retirement_with_records(root, record, records)?;
    }
    Ok(())
}

/// Read disposition details after the caller validates retirement custody.
pub fn retirement_record(root: &Path, record: &ChangeRecord) -> Result<RetirementRecord, String> {
    serde_json::from_str(&read_bounded_change_text(
        &find_change_dir(root, &record.id)?.join(RETIREMENT_FILE),
        "retirement receipt",
    )?)
    .map_err(|error| format!("invalid retirement record: {error}"))
}

#[cfg(test)]
std::thread_local! {
    static RETIREMENT_TEST_FAIL_MOVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn finish_retirement_move(
    root: &Path,
    source: &Path,
    receipt: &RetirementRecord,
) -> Result<(), String> {
    let destination = root.join(ARCHIVE_PATH).join(format!(
        "{}-{}",
        utc_date(receipt.timestamp),
        receipt.change_id
    ));
    let moved = (|| {
        reject_symlink_components_for(root, source, "retirement source")?;
        validate_late_close_destination(root, &destination)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        validate_late_close_destination(root, &destination)?;
        #[cfg(test)]
        if RETIREMENT_TEST_FAIL_MOVE.with(|value| value.get()) {
            return Err("injected retirement move failure".to_string());
        }
        rename_durable(source, &destination).map_err(|e| e.to_string())
    })();
    moved.map_err(|error| format!(
        "retirement custody is recorded but archive move is pending: {error}; rerun the identical change retire command to finish; no implementation acceptance was recorded"))
}

/// Explicit retained-record source; never implies implementation acceptance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetirementSourceKind {
    AcceptedReceipt,
    CommittedAuditedReopeningUnproven,
}

/// How a retained input is bound to the independently accepted custodian.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RetirementCustodyClassification {
    SameSignedOwners,
    UnchangedExactFileCurrentOwner,
}

fn retirement_entry_classification(
    prior: &AcceptanceInputEntryV1,
    next: &AcceptanceInputEntryV1,
) -> Result<RetirementCustodyClassification, String> {
    if prior.owners == next.owners {
        // A reserved `@exact:*` label means no module owns the input, so "same
        // owners" alone is not custody: an independently accepted but unrelated
        // replacement could rewrite the bytes and still retire the predecessor
        // as fully held. Exact-labelled custody transfers only when every
        // byte-level attribute is unchanged.
        if prior.owners.iter().any(|owner| owner.starts_with("@exact:")) {
            let unchanged = prior.kind == AcceptanceInputKind::File
                && next.kind == AcceptanceInputKind::File
                && matches!(prior.mode, 0o100644 | 0o100755)
                && prior.mode == next.mode
                && prior.payload_digest == next.payload_digest
                && prior.entry_digest == next.entry_digest;
            return if unchanged {
                Ok(RetirementCustodyClassification::SameSignedOwners)
            } else {
                Err(format!(
                    "retirement cannot transfer signed owners of {}",
                    prior.path
                ))
            };
        }
        return Ok(RetirementCustodyClassification::SameSignedOwners);
    }
    let unchanged_exact_file = prior.owners == vec!["@exact:delivery".to_string()]
        && prior.kind == AcceptanceInputKind::File
        && next.kind == AcceptanceInputKind::File
        && matches!(prior.mode, 0o100644 | 0o100755)
        && prior.path == next.path
        && prior.mode == next.mode
        && prior.payload_digest == next.payload_digest
        && prior.entry_digest == next.entry_digest
        && !next.owners.is_empty()
        && next.owners.iter().all(|owner| {
            !owner.starts_with("@exact:") && crate::commands::validate_module_name(owner).is_ok()
        });
    if unchanged_exact_file {
        Ok(RetirementCustodyClassification::UnchangedExactFileCurrentOwner)
    } else {
        Err(format!(
            "retirement cannot transfer signed owners of {}",
            prior.path
        ))
    }
}

/// Default retirement uses the unchanged accepted-receipt authentication branch.
pub fn retire_change(
    root: &Path,
    id: &str,
    actor: String,
    reason: String,
    replacement: String,
    authority_source: String,
) -> Result<ChangeRecord, String> {
    retire_change_with_source(
        root,
        id,
        actor,
        reason,
        replacement,
        authority_source,
        RetirementSourceKind::AcceptedReceipt,
    )
}

type RetirementPackageSnapshot = BTreeMap<String, (u32, Vec<u8>)>;

struct RetirementSourceProof {
    manifest: AcceptanceManifestV1,
    anchor: String,
    directory: String,
    package_digest: String,
    review_untrusted: bool,
}

fn retirement_source_proof(
    root: &Path,
    original: &ChangeRecord,
    snapshot: &RetirementPackageSnapshot,
    kind: RetirementSourceKind,
    expected: Option<(&str, &str)>,
) -> Result<RetirementSourceProof, String> {
    if kind == RetirementSourceKind::AcceptedReceipt {
        let (manifest, anchor, package_digest, review_untrusted, directory) =
            retirement_source(root, original)?;
        if let Some((expected_anchor, expected_directory)) = expected
            && (anchor != expected_anchor || directory != expected_directory)
        {
            return Err(
                "accepted retirement source anchor/directory differs from its immutable receipt"
                    .into(),
            );
        }
        return Ok(RetirementSourceProof {
            manifest,
            anchor,
            directory,
            package_digest,
            review_untrusted,
        });
    }
    let (manifest, opening) = retirement_source_header(root, original)?;
    // This is not a recovery from an accepted receipt that failed authentication.
    if opening.recorded_review_mismatch_anchor.is_some()
        || !accepted_recording_anchors(root, original)?.is_empty()
    {
        return Err("accepted-state or recorded mismatch-anchor history exists; do not downgrade it to committed-reopening custody".into());
    }
    if git_output(root, &["rev-parse", "--is-shallow-repository"]).as_deref() != Some("false") {
        return Err(
            "committed-reopening custody requires complete available nonshallow history".into(),
        );
    }
    let directory = format!("{CHANGES_PATH}/{}", original.id);
    let git_directory = git_repo_relative_path(root, &directory)?;
    let candidates = if let Some((anchor, source_directory)) = expected {
        if source_directory != directory || !is_canonical_commit_id(anchor) {
            return Err("committed-reopening source anchor/directory is invalid".into());
        }
        vec![anchor.to_string()]
    } else {
        let pathspec = format!(":(top,literal){git_directory}");
        let limit = format!("--max-count={}", MAX_TRUSTED_HISTORY_COMMITS + 1);
        let output = run_git_bounded(
            root,
            &["log", "--format=%H", &limit, "HEAD", "--", &pathspec],
            None,
            MAX_GIT_COMMAND_OUTPUT_BYTES,
        )?;
        if !output.status.success() {
            return Err("cannot read committed reopening package history".into());
        }
        let text = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        let commits = text
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if commits.len() > MAX_TRUSTED_HISTORY_COMMITS {
            return Err(
                "committed reopening package history exceeds its deterministic bound".into(),
            );
        }
        commits
    };
    for anchor in candidates {
        ensure_git_ancestor(root, &anchor, "HEAD", "committed audited-reopening custody")?;
        let committed = git_tree_snapshot(root, &anchor, &git_directory)?;
        if &committed != snapshot {
            continue;
        }
        let (_, state_bytes) = committed
            .get("state.json")
            .ok_or_else(|| "committed reopening state is missing".to_string())?;
        let state: ChangeRecord = serde_json::from_slice(state_bytes).map_err(|e| e.to_string())?;
        validate_loaded_change(&state, &original.id, Path::new(&directory))?;
        if state.state != ChangeState::Verifying || !state.canonical_applied {
            return Err(
                "committed custody source is not Verifying with canonical application recorded"
                    .into(),
            );
        }
        let implementation = opening
            .prior_verification
            .commit
            .as_deref()
            .ok_or_else(|| {
                "committed reopening has no available prior verification commit".to_string()
            })?;
        if !is_canonical_commit_id(implementation) {
            return Err("committed reopening prior verification commit is invalid".into());
        }
        ensure_git_ancestor(
            root,
            implementation,
            &anchor,
            "reopening prior verification",
        )?;
        // These are internal content-consistency checks, NOT authentication of an Accepted
        // state transition or a human. The exact full ledger/review/finalization bytes are kept.
        for file in [
            "approvals.json",
            "verification.json",
            "finalization.json",
            "review.json",
        ] {
            if !committed.contains_key(file) {
                return Err(format!("committed reopening source is missing {file}"));
            }
        }
        validate_finalization_evidence(root, original, &opening.prior_verification)?;
        return Ok(RetirementSourceProof {
            manifest,
            anchor,
            directory,
            package_digest: historical_package_digest(&committed)?,
            review_untrusted: false,
        });
    }
    Err("no exact pre-existing committed audited-reopening package authenticates this source; retain HOLD".into())
}
