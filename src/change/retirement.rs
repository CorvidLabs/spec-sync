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
    let mut digest = FramedDigest::new(b"specsync.retirement.v1");
    digest.frame(b"record", &serde_json::to_vec(&preimage).map_err(|e| e.to_string())?);
    Ok(digest.finish())
}

fn retirement_text(value: &str, flag: &str, max: usize) -> Result<(), String> {
    if value.trim().is_empty() || value.trim() != value || value.len() > max {
        return Err(format!("retirement requires bounded non-empty {flag}"));
    }
    Ok(())
}

// Authenticate receipt custody, without rehabilitating the original inconsistent review.
fn retirement_source(root: &Path, original: &ChangeRecord)
    -> Result<(AcceptanceManifestV1, String, String, bool), String>
{
    if original.state != ChangeState::Verifying || !original.canonical_applied {
        return Err("retirement requires an audited-reopened verifying change".into());
    }
    ensure_definition_approval_valid(root, original)?;
    ensure_reopened_definition_unchanged(root, original)?;
    let ledger = load_approvals(root, original)?;
    let reopening = ledger.reopenings.last()
        .ok_or_else(|| "retirement requires an audited reopening".to_string())?;
    if reopening.schema_version != 1 || reopening.change_id != original.id ||
        reopening.actor.trim().is_empty() || reopening.reason.trim().is_empty() ||
        reopening.from_state != ChangeState::Accepted || reopening.to_state != ChangeState::Verifying ||
        !reopening.prior_verification.passed ||
        reopening.superseded_approval.digest != closing_digest(original, &reopening.prior_verification) ||
        !is_terminal_approval(&reopening.superseded_approval) ||
        reopening.prior_verification.acceptance_input_digest.as_deref() != Some(reopening.stale_acceptance_input_digest.as_str()) {
        return Err("retirement requires intact audited reopening evidence and prior definition".into());
    }
    validate_verification_execution_digest(root, original, &reopening.prior_verification)?;
    let manifest = reopening.prior_verification.acceptance_manifest.clone()
        .ok_or_else(|| "retirement requires a frozen prior manifest; unknown history remains blocked".to_string())?;
    validate_acceptance_manifest(&manifest)?;
    let expected = serde_json::to_vec(&reopening.prior_verification).map_err(|e| e.to_string())?;
    let anchors = if let Some(anchor) = &reopening.recorded_review_mismatch_anchor {
        let default = remote_default_ref(root)
            .ok_or_else(|| "cannot authenticate inconsistent review without remote default".to_string())?;
        ensure_git_ancestor(root, anchor, &default, "retirement inconsistent review")?;
        vec![anchor.clone()]
    } else {
        accepted_recording_anchors(root, original)?
    };
    for anchor in anchors {
        ensure_git_ancestor(root, &anchor, "HEAD", "retirement source receipt")?;
        for directory in historical_change_directories(root, &anchor, &original.id)? {
            let snapshot = git_tree_snapshot(root, &anchor, &directory)?;
            let Some((_, state_bytes)) = snapshot.get("state.json") else { continue };
            let Ok(state) = serde_json::from_slice::<ChangeRecord>(state_bytes) else { continue };
            if state.id != original.id || state.state != ChangeState::Accepted { continue; }
            validate_loaded_change(&state, &original.id, Path::new(&directory))?;
            if !definition_digest_matches(root, &state, &reopening.prior_verification.contract_digest)? {
                continue;
            }
            let Some((_, verification_bytes)) = snapshot.get("verification.json") else { continue };
            let Ok(verification) = serde_json::from_slice::<VerificationRecord>(verification_bytes) else { continue };
            if serde_json::to_vec(&verification).map_err(|e| e.to_string())? != expected { continue; }
            let Some((_, approval_bytes)) = snapshot.get("approvals.json") else { continue };
            let approvals: ApprovalLedger = serde_json::from_slice(approval_bytes).map_err(|e| e.to_string())?;
            if !approvals.approvals.iter().any(|a|
                a.gate == reopening.superseded_approval.gate &&
                a.actor == reopening.superseded_approval.actor &&
                a.timestamp == reopening.superseded_approval.timestamp &&
                a.digest == reopening.superseded_approval.digest &&
                a.note == reopening.superseded_approval.note) { continue; }
            return Ok((manifest, anchor, historical_package_digest(&snapshot)?,
                reopening.recorded_review_mismatch_anchor.is_some()));
        }
    }
    Err("retirement cannot authenticate the original accepted receipt at an intact ancestor anchor".into())
}

fn retirement_custody(
    root: &Path, original: &ChangeRecord, replacement_id: &str,
    manifest: &AcceptanceManifestV1, records: &BTreeMap<String, ChangeRecord>,
) -> Result<(Vec<RetirementCustodyEntry>, String, String), String> {
    let replacement = records.get(replacement_id)
        .ok_or_else(|| format!("replacement change {replacement_id} not found"))?;
    if replacement.id == original.id ||
        !matches!(replacement.state, ChangeState::Accepted | ChangeState::Archived) {
        return Err("retirement replacement must be independently accepted or archived".into());
    }
    if dependency_reaches(root, &replacement.id, &original.id, &mut BTreeSet::new()) {
        return Err("retirement replacement cannot depend on the obligation it retires".into());
    }
    let mut memo = AcceptedInputValidationMemo::default();
    validate_accepted_inputs_recursive(root, replacement, records, &mut BTreeSet::new(), &mut memo, None)
        .map_err(|e| format!("retirement replacement has stale or unauthenticated evidence: {e}"))?;
    let verification = memo.authenticate(root, replacement, None)?;
    let current = verification.acceptance_manifest.as_ref()
        .ok_or_else(|| "retirement replacement requires a signed current manifest".to_string())?;
    let mut custody = Vec::new();
    for prior in &manifest.entries {
        let next = current.entries.iter().find(|entry| entry.path == prior.path)
            .ok_or_else(|| format!("retirement replacement has no custody for {}", prior.path))?;
        if next.owners != prior.owners {
            return Err(format!("retirement cannot transfer signed owners of {}", prior.path));
        }
        custody.push(RetirementCustodyEntry {
            path: prior.path.clone(), predecessor_entry_digest: prior.entry_digest.clone(),
            predecessor_owners: prior.owners.clone(), replacement_entry_digest: next.entry_digest.clone(),
            replacement_owners: next.owners.clone(),
        });
    }
    Ok((custody, closing_digest(replacement, &verification), acceptance_manifest_digest(current)?))
}

fn validate_retirement_with_records(
    root: &Path, record: &ChangeRecord, records: &BTreeMap<String, ChangeRecord>,
) -> Result<(), String> {
    if record.state != ChangeState::Retired { return Err("change is not retired".into()); }
    let directory = find_change_dir(root, &record.id)?;
    let receipt: RetirementRecord = serde_json::from_str(&read_bounded_change_text(
        &directory.join(RETIREMENT_FILE), "retirement receipt")?)
        .map_err(|e| format!("invalid retirement receipt: {e}"))?;
    for (text, flag, limit) in [
        (receipt.actor.as_str(), "--actor", 256), (receipt.reason.as_str(), "--reason", 4096),
        (receipt.authority_source.as_str(), "--authority-source", 4096),
    ] { retirement_text(text, flag, limit)?; }
    if receipt.schema_version != 1 || receipt.domain != "specsync.retirement.v1" ||
        receipt.change_id != record.id || receipt.implementation_accepted ||
        receipt.historical_transition_status != "unproven" || receipt.digest != retirement_digest(&receipt)? {
        return Err("retirement receipt identity, classification or digest is invalid".into());
    }
    let original: ChangeRecord = serde_json::from_str(&receipt.original_state)
        .map_err(|e| format!("invalid retirement original state: {e}"))?;
    let mut expected = original.clone();
    expected.state = ChangeState::Retired;
    expected.updated_at = receipt.timestamp;
    if serde_json::to_value(&expected).map_err(|e| e.to_string())? !=
        serde_json::to_value(record).map_err(|e| e.to_string())? {
        return Err("retired state differs from its preserved original definition".into());
    }
    let relative = portable_project_path(root, &directory);
    let mut snapshot = archive_workspace_snapshot(root, &directory, &relative)?;
    snapshot.remove(RETIREMENT_FILE);
    snapshot.insert("state.json".into(), (receipt.original_state_mode, receipt.original_state.as_bytes().to_vec()));
    if historical_package_digest(&snapshot)? != receipt.original_package_digest {
        return Err("retirement no longer preserves the exact original reopened package".into());
    }
    let (manifest, anchor, source_digest, untrusted) = retirement_source(root, &original)?;
    if receipt.source_anchor != anchor || receipt.source_package_digest != source_digest ||
        receipt.source_review_untrusted != untrusted ||
        receipt.prior_manifest_digest != acceptance_manifest_digest(&manifest)? {
        return Err("retirement historical receipt custody does not authenticate".into());
    }
    let (custody, closing, digest) =
        retirement_custody(root, &original, &receipt.replacement_id, &manifest, records)?;
    if custody != receipt.custody || closing != receipt.replacement_closing_digest ||
        digest != receipt.replacement_manifest_digest {
        return Err("retirement replacement custody no longer matches authenticated closing evidence".into());
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
pub fn retire_change(
    root: &Path, id: &str, actor: String, reason: String,
    replacement: String, authority_source: String,
) -> Result<ChangeRecord, String> {
    let _lock = acquire_project_lock(root)?;
    for (text, flag, limit) in [
        (actor.as_str(), "--actor", 256), (reason.as_str(), "--reason", 4096),
        (authority_source.as_str(), "--authority-source", 4096),
    ] { retirement_text(text, flag, limit)?; }
    let mut record = load_change(root, id)?;
    if record.state == ChangeState::Retired {
        validate_retirement(root, &record)?;
        let receipt: RetirementRecord = serde_json::from_str(&read_bounded_change_text(
            &find_change_dir(root, id)?.join(RETIREMENT_FILE), "retirement receipt")?)
            .map_err(|e| e.to_string())?;
        if receipt.actor == actor && receipt.reason == reason &&
            receipt.authority_source == authority_source && receipt.replacement_id == replacement {
            return Ok(record);
        }
        return Err("retirement already records a different immutable disposition".into());
    }
    let source = change_dir(root, id);
    if find_change_dir(root, id)? != source {
        return Err("retirement requires an active audited-reopened workspace".into());
    }
    let (manifest, source_anchor, source_package_digest, source_review_untrusted) = retirement_source(root, &record)?;
    let records = list_all_changes_structural(root)?;
    let (custody, replacement_closing_digest, replacement_manifest_digest) =
        retirement_custody(root, &record, &replacement, &manifest, &records)?;
    let snapshot = archive_workspace_snapshot(root, &source, &portable_project_path(root, &source))?;
    if snapshot.contains_key(RETIREMENT_FILE) {
        return Err("active workspace already contains a retirement receipt".into());
    }
    let (original_state_mode, original_state_bytes) = snapshot.get("state.json")
        .ok_or_else(|| "retirement original state is missing".to_string())?;
    let mut receipt = RetirementRecord {
        schema_version: 1, domain: "specsync.retirement.v1".into(), change_id: id.into(),
        actor, reason, authority_source, timestamp: now(),
        original_state: String::from_utf8(original_state_bytes.clone()).map_err(|e| e.to_string())?,
        original_state_mode: *original_state_mode, original_package_digest: historical_package_digest(&snapshot)?,
        source_anchor, source_package_digest, source_review_untrusted,
        prior_manifest_digest: acceptance_manifest_digest(&manifest)?,
        replacement_id: replacement, replacement_closing_digest, replacement_manifest_digest,
        custody, implementation_accepted: false, historical_transition_status: "unproven".into(),
        digest: String::new(),
    };
    receipt.digest = retirement_digest(&receipt)?;
    let destination = root.join(ARCHIVE_PATH).join(format!("{}-{id}", today()));
    validate_late_close_destination(root, &destination)?;
    if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    validate_late_close_destination(root, &destination)?;
    record.state = ChangeState::Retired;
    record.updated_at = receipt.timestamp;
    write_prepared_files(root, &[
        (source.join("state.json"), json_content(&record)?),
        (source.join(RETIREMENT_FILE), json_content(&receipt)?),
    ])?;
    if let Err(error) = rename_durable(&source, &destination) {
        let restore = fs::write(source.join("state.json"), original_state_bytes);
        let remove = fs::remove_file(source.join(RETIREMENT_FILE));
        if restore.is_err() || remove.is_err() {
            return Err(format!("retirement move failed: {error}; original package rollback failed"));
        }
        return Err(format!("retirement move failed; original package restored: {error}"));
    }
    Ok(record)
}


/// Explain terminal nonacceptance and any broken retained-custody obligation.
pub fn retirement_next_action(root: &Path, record: &ChangeRecord) -> String {
    match validate_retirement(root, record) {
        Ok(()) => "retired without implementation acceptance; historical transitions remain unproven; the independent replacement owns current delivery".into(),
        Err(error) => format!("retirement custody is invalid: {error}; restore the retained receipt from trusted history, then run specsync change status {}", record.id),
    }
}
