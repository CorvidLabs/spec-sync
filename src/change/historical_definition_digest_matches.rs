
/// Whether `module` owns `relative` today by the rule that signs acceptance entries for
/// `record`: its spec's `files:`, its canonical companions, or a configured `owns` path.
/// Production source that no declared module owns is not owned, not an error.
fn module_currently_owns_path(
    root: &Path,
    record: &ChangeRecord,
    relative: &str,
    module: &str,
    evidence: &GitEvidence,
) -> Result<bool, String> {
    match acceptance_input_owners(
        root,
        record,
        relative,
        &[],
        evidence,
        UnownedProductionSource::Reject,
    ) {
        Ok(owners) => Ok(owners.iter().any(|owner| owner == module)),
        Err(error) if error.contains("without deterministic canonical ownership") => Ok(false),
        Err(error) => Err(error),
    }
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
        definition_approver_for_review_contract(&approvals, attempt)?;
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

/// Why a recorded scoped review is, or is not, current — split three ways.
///
/// `scoped_review_is_current` answers with a bare `bool`, and every caller that only needs
/// a gate keeps using it. But `false` collapses two different answers, and `ship-status`
/// needs them apart (#743): a review whose CONTENT no longer matches the tree is a decided
/// negative the reader can act on, while a review whose descendant walk could not run at all
/// is an *unavailable* guarantee. #694 states the standard — "an unavailable guarantee
/// reported as a satisfied one is worse than the current failure" — and no predicate that
/// cannot tell the two apart can honour it.
///
/// This type deliberately does NOT decide what a caller should do about `Unavailable`. That
/// is #694's open question. It only makes the distinction sayable, so that the one caller
/// which reported an unavailable guarantee as a satisfied one can stop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopedReviewCurrency {
    /// Every check the review carries was evaluated, and every one agreed with this tree.
    Current,
    /// A decided negative: a recorded digest no longer matches, or the descendant walk ran
    /// and caught a change it forbids. The payload names what moved.
    Stale(String),
    /// The question could not be answered at all. A squash rewrites
    /// `review.implementation_commit`, so `implementation_commit..HEAD` is not a range and
    /// the descendant walk never runs — the guarantee is unobtainable, not violated.
    Unavailable(String),
}

impl ScopedReviewCurrency {
    /// Why, for the callers that render a blocker or a warning.
    ///
    /// Reader-facing *labels* deliberately live with the reporter rather than here: the one
    /// reporter that needs them also has to name "no review at all" and "a review that could
    /// not be read", which are questions about the artifact rather than about its currency.
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Current => None,
            Self::Stale(reason) | Self::Unavailable(reason) => Some(reason.as_str()),
        }
    }
}

/// Classifies a recorded scoped review against the current tree.
///
/// The ordering is deliberate and follows #689: CONTENT first, history last. A review whose
/// digests already disagree is stale for a reason the reader can act on today, and answering
/// "the walk was unavailable" about it would be true but useless. Only once the content
/// agrees does the walk's own availability become the interesting question.
fn scoped_review_currency(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> ScopedReviewCurrency {
    if review.schema_version != 2 {
        return ScopedReviewCurrency::Stale(
            "the recorded review uses an unsupported schema version".into(),
        );
    }
    if review.change_id != record.id {
        return ScopedReviewCurrency::Stale(format!(
            "the recorded review belongs to change `{}`",
            review.change_id
        ));
    }
    if let Err(error) = validate_scoped_reviewer_claim(&review.reviewer) {
        return ScopedReviewCurrency::Stale(format!(
            "the recorded reviewer claim is not usable ({error})"
        ));
    }
    if !scoped_review_provenance_valid(review) {
        return ScopedReviewCurrency::Stale(
            "the recorded review provenance is not the required scoped-review check".into(),
        );
    }
    if review.verdict != ScopedReviewVerdict::Pass {
        return ScopedReviewCurrency::Stale("the recorded review verdict is not a pass".into());
    }
    match definition_digest_matches(root, record, &review.contract_digest) {
        Ok(true) => {}
        Ok(false) => {
            return ScopedReviewCurrency::Stale(
                "the change contract changed after the review was recorded".into(),
            );
        }
        Err(error) => {
            return ScopedReviewCurrency::Unavailable(format!(
                "the change contract digest could not be computed ({error})"
            ));
        }
    }
    if record.workflow_version >= 2 {
        match execution_digest(root, record) {
            Ok(expected) if review.execution_digest.as_deref() == Some(expected.as_str()) => {}
            Ok(_) => {
                return ScopedReviewCurrency::Stale(
                    "the verification evidence changed after the review was recorded".into(),
                );
            }
            Err(error) => {
                return ScopedReviewCurrency::Unavailable(format!(
                    "the verification execution digest could not be computed ({error})"
                ));
            }
        }
    }
    match project_input_digest(root) {
        Ok(digest) if digest == review.workspace_digest => {}
        Ok(_) => {
            return ScopedReviewCurrency::Stale(
                "the reviewed project files changed after the review was recorded".into(),
            );
        }
        Err(error) => {
            return ScopedReviewCurrency::Unavailable(format!(
                "the project input digest could not be computed ({error})"
            ));
        }
    }
    review_commit_currency(root, record, review)
}

fn scoped_review_is_current(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> bool {
    scoped_review_currency(root, record, review) == ScopedReviewCurrency::Current
}

/// The scoped-review twin of `recorded_verification_is_current`, for callers holding only a
/// record. `None` means no usable review record was found at all — a different question from
/// currency, and one the caller has to keep asking separately.
pub fn recorded_scoped_review_currency(
    root: &Path,
    record: &ChangeRecord,
) -> Option<ScopedReviewCurrency> {
    load_scoped_review(root, record)
        .ok()
        .map(|review| scoped_review_currency(root, record, &review))
}

fn review_commit_is_current(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> bool {
    review_commit_currency(root, record, review) == ScopedReviewCurrency::Current
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

/// Classifies the history half of scoped-review currency.
///
/// This used to be `review_commit_is_current_checked`, a `Result<(), String>` whose two
/// callers both discarded the reason with `.is_ok()`. Every failure therefore read as the
/// same thing, and #743 is the bill for that: a squash-rewritten anchor and a descendant
/// that genuinely touched a forbidden path became one indistinguishable `false`.
fn review_commit_currency(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
) -> ScopedReviewCurrency {
    let head = match scoped_review_git_text(root, &["rev-parse", "--verify", "HEAD^{commit}"]) {
        Ok(head) => head,
        Err(_) => {
            return ScopedReviewCurrency::Unavailable("HEAD does not resolve to a commit".into());
        }
    };
    if review.implementation_commit == head {
        return ScopedReviewCurrency::Current;
    }
    // Availability is settled BEFORE violation, and this is the line #694 turns on. A squash
    // rewrites `review.implementation_commit`, so `implementation_commit..HEAD` is not a range
    // and the walk below never runs at all. Reporting that as a pass is what #743 fixes;
    // reporting it as a violation would be the same lie with the sign flipped.
    let ancestor = match run_scoped_review_git(
        root,
        &[
            "merge-base",
            "--is-ancestor",
            &review.implementation_commit,
            &head,
        ],
    ) {
        Ok(output) => output.status.success(),
        Err(error) => {
            return ScopedReviewCurrency::Unavailable(format!(
                "the reviewed commit could not be compared with HEAD ({error})"
            ));
        }
    };
    if !ancestor {
        let short = review
            .implementation_commit
            .get(..8)
            .unwrap_or(review.implementation_commit.as_str());
        return ScopedReviewCurrency::Unavailable(format!(
            "the reviewed commit {short} is no longer reachable from HEAD (squash merge, rebase, amend, or force-push rewrote it), so the scoped-review descendant check could not run"
        ));
    }
    match review_commit_walk(root, record, review, &head) {
        Ok(None) => ScopedReviewCurrency::Current,
        Ok(Some(violation)) => ScopedReviewCurrency::Stale(violation),
        Err(reason) => ScopedReviewCurrency::Unavailable(reason),
    }
}

/// Walks every commit between the review and HEAD, proving that nothing changed except this
/// change's own lifecycle records.
///
/// The three-way return is the whole point. `Ok(None)`: the walk ran and found nothing
/// forbidden. `Ok(Some(reason))`: the walk ran and caught a forbidden change — a decided
/// violation. `Err(reason)`: the walk could not run, so it proved nothing either way.
fn review_commit_walk(
    root: &Path,
    record: &ChangeRecord,
    review: &ScopedReviewRecord,
    head: &str,
) -> Result<Option<String>, String> {
    let range = format!("{}..{head}", review.implementation_commit);
    let limits = lifecycle_validation_limits();
    let max_count = format!("--max-count={}", limits.scoped_review_max_descendants + 1);
    let commits = scoped_review_git_text(root, &["rev-list", "--reverse", &max_count, &range])
        .map_err(|_| "the scoped-review descendants could not be enumerated".to_string())?;
    if commits.lines().filter(|line| !line.is_empty()).count()
        > limits.scoped_review_max_descendants
    {
        return Err(format!(
            "the scoped-review descendant history exceeds the deterministic {}-commit bound, so the walk was not run",
            limits.scoped_review_max_descendants
        ));
    }
    let allowed_prefix = git_repo_relative_path(root, &format!("{CHANGES_PATH}/{}/", record.id))?;
    for descendant in commits.lines().filter(|line| !line.is_empty()) {
        let parents =
            scoped_review_git_text(root, &["rev-list", "--parents", "-n", "1", descendant])
                .map_err(|_| {
                    format!("the parents of review descendant {descendant} could not be loaded")
                })?;
        let fields: Vec<&str> = parents.split_whitespace().collect();
        if fields.first().copied() != Some(descendant) || fields.len() < 2 {
            return Err(format!(
                "review descendant {descendant} has ambiguous parent history, so the walk could not continue"
            ));
        }
        if fields.len() - 1 > limits.scoped_review_max_parents {
            return Err(format!(
                "review descendant {descendant} exceeds the deterministic {}-parent bound, so the walk was not run",
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
                    "review descendant {descendant} could not be inspected"
                ));
            }
            for raw_path in output
                .stdout
                .split(|byte| *byte == 0)
                .filter(|path| !path.is_empty())
            {
                let path = std::str::from_utf8(raw_path).map_err(|_| {
                    "a review descendant contains a non-UTF-8 path, so the walk could not continue"
                        .to_string()
                })?;
                let Some(relative) = path.strip_prefix(&allowed_prefix) else {
                    return Ok(Some(format!(
                        "commit {descendant} changed `{path}`, which is outside this change's own lifecycle records"
                    )));
                };
                if !matches!(
                    relative,
                    SCOPED_REVIEW_FILE
                        | SCOPED_REVIEW_ATTEMPTS_FILE
                        | "state.json"
                        | "verification.json"
                        | "verification-attempts.json"
                ) {
                    return Ok(Some(format!(
                        "commit {descendant} changed `{path}`, which is outside this change's own lifecycle records"
                    )));
                }
            }
        }
    }
    Ok(None)
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
                && review_commit_is_current(root, &record, previous)
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
    // Same question as `materialize_change_deltas`, for the same reason (#741): the flag says
    // materialization ran, not that it ran for this delta content. An empty result is exactly
    // what an already-current tree produces, so acceptance still writes nothing when there is
    // nothing outstanding.
    let mut prepared = prepare_pending_delta_application(root, &record)?.files;
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
            history
        } else {
            VerificationAttemptLedger {
                schema_version: 1,
                attempts: Vec::new(),
            }
        };
        // Missing and empty are the same evidence: nothing recorded the
        // implementation commit. Deleting the ledger used to skip the guard
        // that emptying it tripped (#656).
        if verification_adopted && attempts.attempts.is_empty() {
            return Err(
                "verification attempt history cannot adopt the implementation commit".into(),
            );
        }
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
pub(crate) fn module_context_path(root: &Path, module: &str) -> String {
    let specs_dir = crate::config::load_config(root).specs_dir;
    if let Ok((spec, _)) = canonical_module_paths(root, &specs_dir, module)
        && let Some(directory) = spec.parent()
    {
        return portable_project_path(root, &directory.join("context.md"));
    }
    let specs_dir = specs_dir.trim_matches('/');
    format!("{specs_dir}/{module}/context.md")
}

/// The configured principles file, or the policy error when `.specsync/sdd.json` cannot be read.
///
/// `load_policy` turns a broken file into "nothing configured". An inspection command has to
/// tell those apart: a person cannot honor principles the tool has hidden.
pub(crate) fn configured_principles_file(root: &Path) -> Result<Option<String>, String> {
    Ok(load_policy_checked(root)?.and_then(|policy| policy.principles_file))
}

/// Whether the named principles file is a regular file inside the project.
///
/// `Path::join` would treat an absolute path as the whole location and would
/// follow a symlink out of the project. Approval rejects those paths, so a
/// command that reports principles must reject them too.
pub(crate) fn principles_file_is_present(root: &Path, path: &str) -> Result<bool, String> {
    Ok(safe_project_path(root, path)?.is_file())
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
        let Some((relative, substantive)) = module_lesson_lines(root, module) else {
            continue;
        };
        if !substantive.is_empty() {
            found.push((relative, substantive.len()));
        }
    }
    found
}

/// `specs/<module>/<module>.3md` when the spec directory is known.
///
/// The spec itself stays `<module>.spec.md`. This file is the optional place
/// for context and lessons, and it is plain text.
fn module_layers_path(root: &Path, module: &str) -> String {
    let specs_dir = crate::config::load_config(root).specs_dir;
    if let Ok((spec, _)) = canonical_module_paths(root, &specs_dir, module)
        && let Some(directory) = spec.parent()
    {
        return portable_project_path(root, &directory.join(format!("{module}.3md")));
    }
    let specs_dir = specs_dir.trim_matches('/');
    format!("{specs_dir}/{module}/{module}.3md")
}

/// Lesson lines for one module, and the file they came from.
///
/// A `<module>.3md` file means the module has moved. The first plane in the
/// file is context. Every plane after it is a lesson, in the order written,
/// whatever `z` says. Until that file exists, or when it cannot be parsed,
/// `context.md` is the lesson source. This must not fail a lifecycle command.
fn module_lesson_lines(root: &Path, module: &str) -> Option<(String, Vec<String>)> {
    let layers = module_layers_path(root, module);
    if root.join(&layers).is_file()
        && let Ok(text) = fs::read_to_string(root.join(&layers))
        && let Ok(document) = threemd::parse(&text)
    {
        let mut lines = Vec::new();
        for plane in document.planes.into_iter().skip(1) {
            lines.extend(context_substantive_lines(module, &plane.body));
        }
        return Some((layers, lines));
    }
    let relative = module_context_path(root, module);
    let text = fs::read_to_string(root.join(&relative)).ok()?;
    Some((relative, context_substantive_lines(module, &text)))
}

/// The file a module's lessons are folded into.
fn lesson_file_for_module(root: &Path, module: &str) -> String {
    let layers = module_layers_path(root, module);
    if root.join(&layers).is_file() {
        layers
    } else {
        module_context_path(root, module)
    }
}

/// How much of a lesson or principles file the open-change briefing quotes.
///
/// A wall of text at creation time gets scrolled past. Twenty lines, or 1,200 characters,
/// whichever comes first, is enough to see the lesson. The path names the rest.
const BRIEFING_MAX_LINES: usize = 20;
const BRIEFING_MAX_CHARS: usize = 1_200;

/// One module context that already holds substantive lessons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LessonBriefing {
    pub path: String,
    pub lines: usize,
    pub excerpt: String,
    pub truncated: bool,
}

/// The project's principles file, when `.specsync/sdd.json` names one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct PrinciplesBriefing {
    pub path: String,
    pub excerpt: String,
    pub truncated: bool,
    pub missing: bool,
}

/// Lessons and principles computed when a change is opened. Not stored on the record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OpenChangeBriefing {
    pub lessons: Vec<LessonBriefing>,
    pub principles: Option<PrinciplesBriefing>,
}

/// What a change should be shown before anyone writes more of it.
///
/// An unreadable context or principles file is omitted or marked missing. This never fails
/// the caller: it is an authoring affordance, same as [`accumulated_lessons`].
pub(crate) fn open_change_briefing(root: &Path, record: &ChangeRecord) -> OpenChangeBriefing {
    let mut lessons = Vec::new();
    for module in &record.affected_specs {
        let Some((relative, substantive)) = module_lesson_lines(root, module) else {
            continue;
        };
        if substantive.is_empty() {
            continue;
        }
        let (excerpt, truncated) = bounded_excerpt(&substantive);
        lessons.push(LessonBriefing {
            path: relative,
            lines: substantive.len(),
            excerpt,
            truncated,
        });
    }
    let principles = load_policy(root)
        .and_then(|policy| policy.principles_file)
        .map(|path| match read_principles(root, &path) {
            Some(text) => {
                let lines = principles_excerpt_lines(&text);
                let (excerpt, truncated) = bounded_excerpt(&lines);
                PrinciplesBriefing {
                    path,
                    excerpt,
                    truncated,
                    missing: false,
                }
            }
            None => PrinciplesBriefing {
                path,
                excerpt: String::new(),
                truncated: false,
                missing: true,
            },
        });
    OpenChangeBriefing {
        lessons,
        principles,
    }
}

fn context_substantive_lines(module: &str, text: &str) -> Vec<String> {
    let scaffold = crate::generator::generated_context_scaffold(module);
    let scaffold_lines: std::collections::HashSet<&str> = scaffold.lines().map(str::trim).collect();
    let text = strip_html_comments(text);
    strip_frontmatter(&text)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty()
                || line.starts_with("<!--")
                || line.starts_with('#')
                || scaffold_lines.contains(line)
            {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect()
}

fn principles_excerpt_lines(text: &str) -> Vec<String> {
    let text = strip_html_comments(text);
    strip_frontmatter(&text)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("<!--") {
                None
            } else {
                Some(line.to_string())
            }
        })
        .collect()
}

fn read_principles(root: &Path, path: &str) -> Option<String> {
    let absolute = safe_project_path(root, path).ok()?;
    fs::read_to_string(absolute).ok()
}

fn bounded_excerpt(lines: &[String]) -> (String, bool) {
    let mut kept = Vec::new();
    let mut chars = 0usize;
    for line in lines {
        if kept.len() >= BRIEFING_MAX_LINES {
            return (kept.join("\n"), true);
        }
        let extra = if kept.is_empty() { 0 } else { 1 };
        if chars + extra + line.chars().count() > BRIEFING_MAX_CHARS {
            if kept.is_empty() {
                let excerpt = line.chars().take(BRIEFING_MAX_CHARS).collect::<String>();
                return (excerpt, true);
            }
            return (kept.join("\n"), true);
        }
        chars += extra + line.chars().count();
        kept.push(line.clone());
    }
    (kept.join("\n"), false)
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
                .map(|module| lesson_file_for_module(root, module))
                .collect()
        })
        .unwrap_or_default()
}

/// The hash cache `specsync check` rewrites beside the lifecycle ledgers. `init` ignores it; it
/// is named as lifecycle-owned so a project that does not ignore it still commits its own cache.
const HASH_CACHE_PATH: &str = ".specsync/hashes.json";

/// What a lifecycle commit for one change may stage, and what it must neither stage nor report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LifecycleCommitScope {
    /// Untracked paths the lifecycle wrote or the change owns. A lifecycle commit stages these.
    pub(crate) owned: Vec<String>,
    /// Lifecycle runtime files: the project lock and an in-flight transaction journal. They are
    /// never committed, and never reported as left out either, because they are nobody's delivery
    /// and the project-input digest already excludes them.
    pub(crate) runtime: Vec<String>,
}

/// Which untracked paths a lifecycle commit for this change may stage, and which runtime files it
/// must leave alone.
///
/// `change check --commit` and `change ship --push` staged with `git add -A`, which swept every
/// untracked, non-ignored file in the project into a pushed commit: a private debug archive, an
/// agent's scratch directory, an experiment script. Tracked edits are the delivery, and the
/// command layer stages all of them. An untracked file is the lifecycle's to commit only when the
/// lifecycle wrote it or the change owns it, and this is the one answer to which files those are:
///
/// - the active workspace. `finalize` empties it by moving it, so staging the removal of its
///   tracked files needs this path as well;
/// - the archive package, once `finalize` has moved the workspace there;
/// - each affected spec's canonical spec file and its companions, resolved through the same
///   registry-aware resolver materialization writes through, so the two cannot name different
///   files. Only those files, never the whole directory: a stray file there is not a spec;
/// - the lifecycle ledgers: the sequence ledger the staging path floors, the workflow-v2 and
///   legacy-archive baselines, the bootstrap record, and the hash cache. `change new` writes the
///   workflow-v2 baseline in a freshly adopted project, and it has to reach history with the first
///   change or every later reader of that change fails its origin check.
///
/// The paths are project-relative with forward slashes, sorted and deduplicated. The change's
/// `affected_paths` are deliberately absent. They are prefixes as broad as `src/` or `.`, and
/// owning every untracked file under them is the sweep this exists to stop. A new source file
/// joins the delivery when its author stages it.
pub(crate) fn lifecycle_commit_scope(
    root: &Path,
    id: &str,
) -> Result<LifecycleCommitScope, String> {
    let record = load_change(root, id)?;
    let mut owned = BTreeSet::new();
    owned.insert(portable_project_path(root, &change_dir(root, &record.id)));
    owned.insert(portable_project_path(
        root,
        &find_change_dir(root, &record.id)?,
    ));
    let specs_dir = crate::config::load_config(root).specs_dir;
    for module in &record.affected_specs {
        let (spec, _) = canonical_module_paths(root, &specs_dir, module)?;
        owned.insert(portable_project_path(root, &spec));
        if let Some(directory) = spec.parent() {
            for companion in canonical_companion_names(module) {
                owned.insert(portable_project_path(root, &directory.join(companion)));
            }
        }
    }
    for ledger in [
        SEQUENCE_PATH,
        WORKFLOW_V2_BASELINE_PATH,
        LEGACY_BASELINE_PATH,
        BOOTSTRAP_RECORD_PATH,
        HASH_CACHE_PATH,
    ] {
        owned.insert(ledger.to_string());
    }
    Ok(LifecycleCommitScope {
        owned: owned.into_iter().collect(),
        runtime: vec![LOCK_PATH.to_string(), TRANSACTION_PATH.to_string()],
    })
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
        "Material for folding this change's lessons into each affected module's lesson file.\n\
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
            out.push_str(&format!("- `{}`\n", lesson_file_for_module(root, module)));
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
    // The projection is also the one successor that can cover a change this package supersedes,
    // and the token is what lets it speak for itself here: without it the walk judged the package
    // by a history whose closing commit has not been made, refused it silently, and reported the
    // superseded change as uncovered during the very finalize that was covering it.
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
            pending_close.as_ref(),
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

/// The handoff verdict for one change, from the same signals `summarize_change` reads — but
/// gathered only as far as the state needs. Text-mode `status` prints this on every change,
/// and an archived record's terminal-evidence walk over the whole archive would turn that line
/// into a minutes-long wait for a verdict that is `safe` regardless.
pub fn handoff_summary(root: &Path, record: &ChangeRecord) -> HandoffSummary {
    let sequence_frozen = sequence_ledger_freeze_next_action(root).is_some();
    let mut signals = HandoffSignals {
        state: record.state,
        workflow_version: record.workflow_version,
        sequence_frozen,
        open_questions: false,
        artifacts_complete: true,
        approval_valid: true,
        correction_valid: true,
        scoped_edits_uncommitted: None,
        verification_current: true,
        scoped_review_current: true,
        terminal_evidence_stale: false,
    };
    if sequence_frozen {
        return classify_handoff(&record.id, &signals);
    }
    match record.state {
        ChangeState::Archived => {}
        ChangeState::Draft => {
            signals.open_questions = !next_questions_for(root, record).is_empty();
            signals.artifacts_complete =
                signals.open_questions || validate_artifacts(root, record).is_ok();
        }
        ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying => {
            signals.approval_valid = ensure_definition_approval_valid(root, record).is_ok();
            if signals.approval_valid {
                signals.scoped_edits_uncommitted = scoped_edits_uncommitted(root, record);
            }
            if record.state == ChangeState::Verifying
                && signals.approval_valid
                && signals.scoped_edits_uncommitted != Some(true)
            {
                signals.verification_current = load_verification(root, record)
                    .is_ok_and(|verification| verification_is_current(root, record, &verification));
                signals.scoped_review_current = signals.verification_current
                    && load_scoped_review(root, record)
                        .is_ok_and(|review| scoped_review_is_current(root, record, &review));
            }
        }
        ChangeState::Accepted => {
            signals.correction_valid = effective_change_definition(root, record).is_ok();
            if signals.correction_valid && record.workflow_version < 2 {
                signals.terminal_evidence_stale = ensure_closing_approval_valid(root, record)
                    .is_err()
                    || terminal_evidence_summary(root, record).validity
                        == TerminalEvidenceValidity::Stale;
            }
        }
    }
    classify_handoff(&record.id, &signals)
}

/// Decides handoff readiness from gathered lifecycle signals. Pure: the strings are built from
/// literals and `id` only, so nothing digest-bearing can reach a text line through it.
///
/// The order is the order a fresh session would trip over things. A project-wide freeze comes
/// first because it blocks every verb. A Draft is never `Safe` — approval is the first boundary
/// `change status` can resume from, so until then the intent lives in this session. Once
/// approved, a stale definition digest means the definition was edited after approval and only
/// this session knows whether that was meant. Then uncommitted edits under the change's paths:
/// the work is on disk, but nothing records why it was made. Then evidence currency: a stale
/// check is redone, not lost, so that is `Conditional`. Evidence under `.specsync/` is never a
/// signal — `review` then `finalize` runs with it uncommitted by design.
pub fn classify_handoff(id: &str, signals: &HandoffSignals) -> HandoffSummary {
    let resume = format!("specsync change status {id}");
    let approve = format!("run `specsync change approve {id} --actor <name>`");
    let change_md = format!("`.specsync/changes/{id}/change.md`");
    let (readiness, reason, before_clearing): (HandoffReadiness, &str, Vec<String>) = if signals
        .sequence_frozen
    {
        (
            HandoffReadiness::NotYet,
            "the change-sequence ledger is frozen project-wide, and a fresh session inherits the block without knowing what caused it",
            vec!["clear the freeze named under Next".into()],
        )
    } else {
        match signals.state {
            ChangeState::Archived => (
                HandoffReadiness::Safe,
                "the change is archived; nothing this session knows is still needed",
                Vec::new(),
            ),
            ChangeState::Draft if signals.open_questions => (
                HandoffReadiness::Conditional,
                "the interview is unfinished, so what this session has decided is not recorded yet",
                vec![
                    format!(
                        "run `specsync change answer {id} <question> <answer>` for each remaining question"
                    ),
                    format!("or write the undecided points into {change_md}"),
                ],
            ),
            ChangeState::Draft if !signals.artifacts_complete => (
                HandoffReadiness::Conditional,
                "selected artifacts still carry stubs and the definition is unapproved, so the intent behind them lives only here",
                vec![
                    format!(
                        "finish the selected artifacts, then {approve} — approval is the first boundary a fresh session resumes from"
                    ),
                    format!("or write the open decisions into {change_md}"),
                ],
            ),
            ChangeState::Draft => (
                HandoffReadiness::Conditional,
                "the definition is complete but unapproved; approval pins what this session decided",
                vec![approve.clone()],
            ),
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
                if !signals.approval_valid =>
            {
                (
                    HandoffReadiness::NotYet,
                    "the definition changed after it was approved, and only this session knows whether that edit was intended",
                    vec![format!(
                        "review the definition, then {approve} to re-pin it — or revert the edit"
                    )],
                )
            }
            ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
                if signals.scoped_edits_uncommitted == Some(true) =>
            {
                (
                    HandoffReadiness::Conditional,
                    "uncommitted edits sit under this change's paths, and nothing on disk records why they were made",
                    vec![
                        "commit the work in progress".into(),
                        format!("or write its intent and open ends into {change_md}"),
                    ],
                )
            }
            ChangeState::Verifying if signals.workflow_version < 2 => (
                HandoffReadiness::Conditional,
                "a workflow-v1 change still closes through verify then accept then archive",
                vec![format!(
                    "run `specsync change verify {id}`, then `specsync change accept {id}`, then `specsync change archive {id}`"
                )],
            ),
            ChangeState::Verifying if !signals.verification_current => (
                HandoffReadiness::Conditional,
                "the tree moved after the last check, so a fresh session would resume from evidence that no longer matches",
                vec![format!("run `specsync change check {id} --commit`")],
            ),
            ChangeState::Verifying if !signals.scoped_review_current => (
                HandoffReadiness::Safe,
                // Currency is this working tree. `change check` without `--commit`
                // leaves the change directory untracked, and an empty path list
                // counts as clean, so this branch has not seen a commit.
                "verification is current for this working tree; a fresh session here resumes at the independent review",
                Vec::new(),
            ),
            ChangeState::Verifying => (
                HandoffReadiness::Safe,
                "verification and review are current; a fresh session resumes at finalize — do not commit before it runs",
                Vec::new(),
            ),
            ChangeState::Approved | ChangeState::Implementing => (
                HandoffReadiness::Safe,
                "the approval pins the definition; a fresh session in this tree resumes at `change check`",
                Vec::new(),
            ),
            ChangeState::Accepted if !signals.correction_valid => (
                HandoffReadiness::NotYet,
                "the correction ledger is invalid, so a fresh session cannot trust the accepted definition",
                vec![format!(
                    "restore `.specsync/changes/{id}/corrections.json` from trusted history"
                )],
            ),
            ChangeState::Accepted if signals.workflow_version >= 2 => (
                HandoffReadiness::Safe,
                "acceptance is recorded and finalize needs nothing from this session — do not commit before it runs",
                Vec::new(),
            ),
            ChangeState::Accepted if signals.terminal_evidence_stale => (
                HandoffReadiness::NotYet,
                "the accepted evidence went stale, and the reason is known only here",
                vec![format!(
                    "run `specsync change reopen {id} --actor <name> --reason <reason>` while the reason is still known"
                )],
            ),
            ChangeState::Accepted => (
                HandoffReadiness::Safe,
                "the accepted evidence is current; a fresh session archives from the record on disk",
                Vec::new(),
            ),
        }
    };
    HandoffSummary {
        readiness,
        reason: reason.into(),
        resume,
        before_clearing,
    }
}

/// True when `git status` reports uncommitted changes under the change's `affected_paths`,
/// ignoring `.specsync/`: lifecycle evidence is uncommitted by design between `change review`
/// and `change finalize`, and counting it would report `conditional` at exactly the moment the
/// lifecycle wants the tree left alone. `None` outside a Git repository.
fn scoped_edits_uncommitted(root: &Path, record: &ChangeRecord) -> Option<bool> {
    let paths: Vec<&str> = record
        .affected_paths
        .iter()
        .map(String::as_str)
        .filter(|path| !path.starts_with(".specsync/"))
        .collect();
    if paths.is_empty() {
        return Some(false);
    }
    let mut args = vec!["status", "--porcelain", "--"];
    args.extend(paths);
    let output = Command::new("git")
        .args(&args)
        .current_dir(root)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let listing = String::from_utf8_lossy(&output.stdout);
    Some(listing.lines().any(|line| {
        // `XY path` or `XY old -> new`; a quoted path is treated as an edit rather than parsed.
        let path = line.get(3..).unwrap_or("");
        let path = path.rsplit(" -> ").next().unwrap_or(path);
        !path.starts_with(".specsync/")
    }))
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
                        Ok(content) if !artifact_is_incomplete(artifact, &content) => None,
                        _ => Some(relative),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // Only a verifying change has verification to be current; everywhere else the load is
    // skipped, as it was when this was a closure the match called lazily.
    let verification_current = record.state == ChangeState::Verifying
        && load_verification(root, record)
            .is_ok_and(|verification| verification_is_current(root, record, &verification));
    let scoped_review_current = load_scoped_review(root, record)
        .is_ok_and(|review| scoped_review_is_current(root, record, &review));
    let routing = load_verification_routing(root).unwrap_or_default();
    let strict_validation_required =
        explicit_strict || change_requires_strict_validation(record, &routing);
    // Advertise the command that will actually run. `strict_validation_required`
    // still reports that this change touches a high-risk path and should be
    // checked with `--strict`, but only `--strict` on THIS invocation makes the
    // pass strict — naming it otherwise printed a command `verify_change` never
    // records as evidence, so status and `verification.json` disagreed.
    let verification_commands = vec![scoped_check_command(
        &scoped_spec_files(root, record, &crate::config::load_config(root)).filters,
        explicit_strict,
    )];
    let terminal_evidence = matches!(record.state, ChangeState::Accepted | ChangeState::Archived)
        .then(|| terminal_evidence_summary(root, record));
    // Sequence-ledger freezes must outrank state-local next actions (including reopen /
    // finalize). Premature multi-id acknowledgements block change new/adopt project-wide.
    let sequence_freeze = sequence_ledger_freeze_next_action(root);
    let next_action = if let Some(remediation) = sequence_freeze.clone() {
        remediation
    } else {
        match record.state {
            ChangeState::Draft if !next_questions_for(root, record).is_empty() => {
                let question = next_questions_for(root, record)
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
            ChangeState::Verifying if record.workflow_version < 2 => {
                format!(
                    "run `specsync change verify {0}`, then `specsync change accept {0}`, then `specsync change archive {0}`",
                    record.id
                )
            }
            ChangeState::Verifying if !verification_current => {
                format!("run `specsync change check {}`", record.id)
            }
            ChangeState::Verifying if !scoped_review_current => {
                format!(
                    "run `specsync change review {} --reviewer <human>` after the PR's scoped review passes",
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
    let handoff = classify_handoff(
        &record.id,
        &HandoffSignals {
            state: record.state,
            workflow_version: record.workflow_version,
            sequence_frozen: sequence_freeze.is_some(),
            open_questions: !next_questions_for(root, record).is_empty(),
            artifacts_complete,
            approval_valid,
            correction_valid,
            scoped_edits_uncommitted: matches!(
                record.state,
                ChangeState::Approved | ChangeState::Implementing | ChangeState::Verifying
            )
            .then(|| scoped_edits_uncommitted(root, record))
            .flatten(),
            verification_current,
            scoped_review_current,
            terminal_evidence_stale: terminal_evidence
                .as_ref()
                .is_some_and(|evidence| evidence.validity == TerminalEvidenceValidity::Stale)
                || (record.state == ChangeState::Accepted
                    && record.workflow_version < 2
                    && ensure_closing_approval_valid(root, record).is_err()),
        },
    );
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
        handoff,
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
        terminal_evidence_results_with_records(root, &all_records, &all_records)
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
            // An active terminal record is judged against every change that could cover its
            // inputs, and a finalized successor is an archive. Archives are loaded here only as
            // candidates -- none of them is evaluated, and only one declaring a matching
            // obligation is ever authenticated -- and only when there is an active terminal
            // record to judge, so the empty common case still loads none.
            match list_all_changes_checked(root) {
                Ok(successors) => {
                    terminal_evidence_results_with_records(root, &active_terminal, &successors)
                }
                Err(error) => {
                    return SddCheckReport {
                        enabled: true,
                        errors: vec![error],
                        ..SddCheckReport::default()
                    };
                }
            }
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
    // Read outside the lock only to phrase the action line; the authoritative
    // read that decides what is written happens under the lock below. A policy
    // that does not parse is reported as work to do rather than as "already on",
    // and the locked read then fails closed on it instead of overwriting it.
    actions.push(match load_policy_checked(root) {
        Ok(Some(policy)) if policy.enabled => {
            format!("SDD policy is already enabled at {POLICY_PATH}")
        }
        _ => format!("enable SDD policy at {POLICY_PATH}"),
    });
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
    let existing_bootstrap = fs::read_to_string(root.join(".specsync/adoption-report.json"))
        .ok()
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|value| value.get("bootstrap_policy").cloned());
    // `adopt` is the on-switch. `init` writes the policy OFF, so deciding from
    // mere file existence made adoption a no-op on every project that had run
    // `init` — which is every project the hint is printed to.
    //
    // Read through `load_policy_checked` rather than `Path::exists`: a policy
    // that does not parse must fail closed here, not be silently replaced by a
    // fresh default that discards whatever the author had written.
    //
    // Only `enabled` moves. `require_change_for_meaningful_files` is left where
    // the author left it: turning path coverage on here would revoke this file's
    // own bootstrap exemption (its digest pins `enabled`) and demand an active
    // change covering `.specsync/sdd.json` on the very next check.
    let policy_content = match load_policy_checked(root)? {
        None => Some(json_content(&SddPolicy {
            enabled: true,
            ..default_policy(root, detect_verification_commands(root))
        })?),
        Some(policy) if !policy.enabled => Some(json_content(&SddPolicy {
            enabled: true,
            ..policy
        })?),
        // Already adopted: idempotent, and the author's policy is not rewritten.
        Some(_) => None,
    };
    let bootstrap_policy = match policy_content.as_deref() {
        // Re-pin over the bytes actually being written. `bootstrap_digest`
        // covers `enabled`, so a flip that kept the old digest would silently
        // revoke the exemption `init` recorded.
        Some(content) => adoption_bootstrap_record_for_content(root, content.as_bytes())?,
        None => existing_bootstrap,
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
    // Only REFRESH a record that already exists — never create one here.
    // `adopt` on a project that never ran `init` is exempted by the adoption
    // report alone, and that report is deliberately the single authority for a
    // policy adoption created (`adoption_bootstrap_covers_only_the_original_policy`).
    // Writing a second, independent exemption would mean tampering with the
    // report no longer revokes anything.
    let refresh_bootstrap = root.join(BOOTSTRAP_RECORD_PATH).is_file()
        && prepared
            .iter()
            .any(|(path, _)| path == &root.join(POLICY_PATH));
    if let Some(candidate) = baseline_candidate {
        write_prepared_files_checked(root, &prepared, || {
            validate_workflow_v2_adoption_git_snapshot(root, &candidate.git_snapshot)
        })?;
    } else {
        write_prepared_files(root, &prepared)?;
    }
    // `init` recorded a digest over the policy it wrote with SDD off. Flipping
    // `enabled` invalidates it, and a stale bootstrap record does not fail — it
    // silently stops exempting the file it was written to exempt. Re-pin here so
    // the exemption survives adoption. A repository with no HEAD has no coverage
    // gate to exempt from and returns early.
    if refresh_bootstrap {
        record_bootstrap_paths(root)?;
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
