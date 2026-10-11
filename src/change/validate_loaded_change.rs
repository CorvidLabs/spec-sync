
/// Whether some accepted or archived successor that claims `expected` of `record` for `owner`
/// authenticates as covering it. Every claimant that does not is recorded in
/// `rejected_successors` with the reason it was refused, keyed by successor ID.
#[allow(clippy::too_many_arguments)]
fn successor_covers_input(
    root: &Path,
    record: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
    pending: Option<&PendingArchiveClose<'_>>,
    expected: &AcceptanceInputEntryV1,
    owner: &str,
    rejected_successors: &mut BTreeMap<String, RejectedSuccessor>,
) -> bool {
    for candidate in records.values() {
        // Only a successor that declares this obligation can cover it: authenticated
        // succession tuples are one-to-one with the record's declared obligations,
        // and the legacy reconstruction requires the declaration too. Deciding that
        // first keeps every refusal recorded below about a successor that claimed
        // the input.
        if candidate.id == record.id
            || !matches!(
                candidate.state,
                ChangeState::Accepted | ChangeState::Archived
            )
            || candidate.no_spec_change
            || !happens_after(candidate, record)
            || !declares_succession_obligation(candidate, &record.id, expected, owner)
        {
            continue;
        }
        let mut reject = |reason: String| {
            rejected_successors.insert(
                candidate.id.clone(),
                RejectedSuccessor {
                    workflow_version: candidate.workflow_version,
                    reason,
                },
            );
        };
        let candidate_verification = match authenticate_accepted_evidence(root, candidate, pending)
        {
            Ok(verification) => verification,
            Err(error) => {
                reject(format!(
                    "its accepted evidence did not authenticate: {error}"
                ));
                continue;
            }
        };
        let candidate_manifest =
            if let Some(manifest) = candidate_verification.acceptance_manifest.as_ref() {
                manifest.clone()
            } else {
                match resolved_acceptance_manifest(root, candidate) {
                    Ok(manifest) => manifest,
                    Err(error) => {
                        reject(format!(
                            "its acceptance manifest could not be resolved: {error}"
                        ));
                        continue;
                    }
                }
            };
        let tuple = if let Some(evidence) = candidate_verification.semantic_succession.as_ref() {
            evidence
                .tuples
                .iter()
                .find(|tuple| semantic_tuple_matches_obligation(tuple, &record.id, expected, owner))
                .cloned()
        } else if candidate_verification.acceptance_manifest.is_none() {
            legacy_semantic_successor_tuple(root, record, expected, owner, candidate)
                .ok()
                .flatten()
        } else {
            None
        };
        let Some(tuple) = tuple else {
            reject(format!(
                "its acceptance evidence carries no succession tuple for `{}` (owner `{owner}`)",
                expected.path
            ));
            continue;
        };
        if !candidate_manifest.entries.iter().any(|entry| {
            entry.path == expected.path
                && entry.entry_digest == tuple.successor_entry_digest
                && entry.owners.iter().any(|entry_owner| entry_owner == owner)
        }) {
            reject(format!(
                "its acceptance manifest does not carry `{}` at the succession tuple's successor digest for owner `{owner}`",
                expected.path
            ));
            continue;
        }
        // A decided negative and a failure to evaluate are different answers (#743).
        match semantic_acceptance_item_exists_for_module(root, candidate, owner) {
            Ok(true) => {}
            Ok(false) => {
                reject(format!(
                    "its `{owner}` delta carries no non-removed semantic item"
                ));
                continue;
            }
            Err(error) => {
                reject(format!("its `{owner}` delta could not be read: {error}"));
                continue;
            }
        }
        match semantic_tuple_transition_is_valid(root, candidate, &tuple, pending) {
            Ok(true) => {}
            Ok(false) => {
                reject(format!(
                    "its succession tuple for `{}` does not hold between its base commit and its acceptance anchor",
                    expected.path
                ));
                continue;
            }
            Err(error) => {
                reject(format!(
                    "its succession tuple for `{}` could not be evaluated: {error}",
                    expected.path
                ));
                continue;
            }
        }
        match validate_accepted_inputs_recursive(root, candidate, records, visiting, memo, pending)
        {
            Ok(_) => return true,
            Err(error) => reject(format!("its own delivery-input evidence is stale: {error}")),
        }
    }
    false
}

/// The modules under which later accepted or archived changes claim `predecessor_entry` of
/// `predecessor`, sorted. An exact-only entry names no module of its own, so these are the
/// owners it is judged against.
fn succession_claimants(
    records: &BTreeMap<String, ChangeRecord>,
    predecessor: &ChangeRecord,
    predecessor_entry: &AcceptanceInputEntryV1,
) -> BTreeSet<String> {
    records
        .values()
        .filter(|candidate| {
            candidate.id != predecessor.id
                && matches!(
                    candidate.state,
                    ChangeState::Accepted | ChangeState::Archived
                )
                && !candidate.no_spec_change
                && happens_after(candidate, predecessor)
        })
        .flat_map(|candidate| candidate.supersedes.iter())
        .filter(|edge| edge.predecessor_id == predecessor.id)
        .flat_map(|edge| edge.obligations.iter())
        .filter(|obligation| {
            obligation.path == predecessor_entry.path
                && obligation.predecessor_entry_digest == predecessor_entry.entry_digest
        })
        .map(|obligation| obligation.module.clone())
        .collect()
}

/// A successor that claimed to cover a stale delivery input and was refused, with the reason.
///
/// The successor walk used to drop every refusal on the floor -- `Err(_) => continue` -- so the
/// diagnostic could only say that no successor covers the input, which was false whenever one
/// existed and was refused for a reason the tool knew at that moment (#685). The workflow version
/// travels with the reason because the remediation depends on it.
struct RejectedSuccessor {
    workflow_version: u32,
    reason: String,
}

fn stale_input_remediation_reason(
    record: &ChangeRecord,
    path: &str,
    owner: &str,
    rejected_successors: &BTreeMap<String, RejectedSuccessor>,
) -> String {
    if rejected_successors.is_empty() {
        return format!(
            "delivery input `{path}` (owner `{owner}`) changed after acceptance and no accepted or archived successor change covers it; run `specsync change reopen {}` to re-verify the accepted change",
            record.id
        );
    }
    let rejections = rejected_successors
        .iter()
        .map(|(id, rejected)| format!("successor `{id}` was rejected: {}", rejected.reason))
        .collect::<Vec<_>>()
        .join("; ");
    // Reopening a legacy change replays its canonical delta over whatever a workflow-v2 successor
    // materialized: its `## ADDED` section block conflicts with the successor's `## MODIFIED` of
    // the same section, and its `## MODIFIED` requirement blocks overwrite the amended wording.
    // Finishing the successor is the remedy, so the legacy reopen is not offered beside it.
    let current_successors = rejected_successors
        .iter()
        .filter(|(_, rejected)| rejected.workflow_version >= 2)
        .map(|(id, _)| format!("`specsync change status {id}`"))
        .collect::<Vec<_>>();
    if record.workflow_version < 2 && !current_successors.is_empty() {
        format!(
            "delivery input `{path}` (owner `{owner}`) changed after acceptance; {rejections}; bring a covering successor to `specsync change finalize` ({} names its next step) rather than reopening `{}`, because reopening a legacy change replays its canonical delta over the successor's materialization",
            current_successors.join(", "),
            record.id
        )
    } else {
        format!(
            "delivery input `{path}` (owner `{owner}`) changed after acceptance; {rejections}; verify and accept a covering successor, or run `specsync change reopen {}` to re-verify the accepted change",
            record.id
        )
    }
}

/// The remedy for a changed input whose only signed owners are reserved exact labels. Reopening
/// replays the change; a successor supersedes the input under a module that owns it now, which
/// is the remedy for a bootstrap change whose whole tree was signed exact-only, so it is named
/// wherever the configuration can grant it.
fn exact_only_input_remediation_reason(record: &ChangeRecord, path: &str) -> String {
    let mut reason = format!(
        "exact-only delivery input `{path}` changed after acceptance and requires an audited reopen; run `specsync change reopen {}` to re-verify the accepted change",
        record.id
    );
    if ownership_is_configurable(path) {
        reason.push_str(
            ", or supersede it from a later change under a module granted the path by `owns` in `.specsync/config.toml`",
        );
    }
    reason
}

fn later_sequence_owner_covers_historical_input(
    root: &Path,
    predecessor: &ChangeRecord,
    records: &BTreeMap<String, ChangeRecord>,
    visiting: &mut BTreeSet<String>,
    memo: &mut BTreeMap<String, Result<AcceptedInputValidity, String>>,
    pending: Option<&PendingArchiveClose<'_>>,
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
    let verification = match authenticate_accepted_evidence(root, owner, pending) {
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
    Ok(validate_accepted_inputs_recursive(root, owner, records, visiting, memo, pending).is_ok())
}

/// True when `candidate` declares the obligation that would let it cover `predecessor_entry`
/// for `owner` on behalf of `predecessor_id`.
fn declares_succession_obligation(
    candidate: &ChangeRecord,
    predecessor_id: &str,
    predecessor_entry: &AcceptanceInputEntryV1,
    owner: &str,
) -> bool {
    candidate.supersedes.iter().any(|edge| {
        edge.predecessor_id == predecessor_id
            && edge.obligations.iter().any(|obligation| {
                obligation.path == predecessor_entry.path
                    && obligation.module == owner
                    && obligation.predecessor_entry_digest == predecessor_entry.entry_digest
            })
    })
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
    if !declares_succession_obligation(candidate, &predecessor.id, predecessor_entry, owner)
        || !semantic_acceptance_item_exists_for_module(root, candidate, owner)?
    {
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
    // A portable approval is still a DEFINITION approval, and a definition approval records the
    // wording it approved (#711). Writing `None` here made `--portable-5-0-1` the one definition
    // gate that declined to say what it signed, and because `effective_definition_approval` reads
    // the LAST definition-gate event, that silence replaced a digest an earlier approval had
    // already recorded — so `ensure_approved_delta_bodies_unchanged` returned on absence and the
    // binding was disarmed for the rest of the change's life (#719).
    //
    // Carrying it forward rather than refusing the approve is what the rest of the module already
    // does: `append_approval` records this for every definition gate, and the normalizing approval
    // in `accept_change` carries it forward for the same stated reason. Refusing instead would
    // remove the one way an adopter can hand a 5.0.1 verifier a change the current binary approved.
    //
    // The bodies are read here, after `validate_delta_files`, so this is exactly the wording this
    // actor is approving now — not a claim inherited from an older event. It leaves the pair's own
    // digests alone: `approved_delta_digests` is an input to neither `definition_digest`, the
    // 5.0.1 projection bytes, nor `definition_approval_pair_id`, and `ApprovalLedger` tolerates
    // unknown fields by design, so a 5.0.1 reader still parses the record it came for.
    let approved_delta_digests = Some(delta_body_digests(root, record)?);
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
        approved_delta_digests: approved_delta_digests.clone(),
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
        approved_delta_digests,
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
    } else if record.state == ChangeState::Verifying {
        // A Verifying change already recorded at least one attempt. Recreating
        // an empty ledger here would drop that history the same way deleting
        // the file bypasses the empty-ledger adoption guard (#656).
        return Err("verification attempt history is missing".into());
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
        "\n  cover them: specsync change new \"<summary>\" --kind bug-fix --spec <module>{path_flags}"
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
                    canonical_companion_names(module)
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

/// Filename Cargo locks inside a build directory for the whole of a build.
const CARGO_BUILD_LOCK_FILE: &str = ".cargo-lock";

/// Cargo subcommands that take the build-directory lock.
///
/// Deliberately an allowlist rather than "anything but a known read-only
/// subcommand". Naming a lock a command will never wait on is the same defect
/// as staying silent about one it will: output that does not describe what is
/// happening. Third-party subcommands are excluded for the same reason —
/// `cargo nextest`'s `--profile` selects a NEXTEST profile and its
/// `--cargo-profile` selects the Cargo one, so reading its argv with these
/// rules would derive the wrong build directory.
const CARGO_BUILD_LOCK_SUBCOMMANDS: [&str; 15] = [
    "b", "bench", "build", "c", "check", "clippy", "d", "doc", "fix", "r", "run", "rustc",
    "rustdoc", "t", "test",
];

/// Cargo global options that consume the next argument before the subcommand.
const CARGO_GLOBAL_VALUE_OPTIONS: [&str; 4] = ["--color", "--config", "--target-dir", "-Z"];

/// Keys under a Cargo config's `[build]` table that move the build directory out
/// from under this derivation. A config setting one is not derivable here.
///
/// `build-dir` is unstable on the pinned toolchain, and included anyway: SpecSync
/// runs against whatever Cargo the operator's project has, and the cost of
/// bailing on a key that turns out not to matter is one missing notice.
const CARGO_LAYOUT_CONFIG_KEYS: [&str; 3] = ["target-dir", "target", "build-dir"];

/// Keys under a Cargo config's `[env]` table that set an environment variable
/// this derivation reads. Whether Cargo's `[env]` table feeds its own build
/// layout is not settled here, so the answer where one is present is silence.
const CARGO_LAYOUT_CONFIG_ENV_KEYS: [&str; 4] = [
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_BUILD_DIR",
];

/// Reports whether a Cargo configuration file in scope could move the build
/// directory.
///
/// Cargo merges `.cargo/config.toml` from the working directory upward and from
/// `CARGO_HOME`, and any of them can set `[build] target-dir`, `target` or
/// `build-dir`, or an `[env]` table entry for the variables this derivation
/// reads. It follows none of those, so the honest answer where one exists is no
/// notice at all: a stale `<root>/target/debug/.cargo-lock` left behind by an
/// earlier layout would otherwise be reported as the lock a command is waiting
/// on when it is waiting somewhere else entirely. A file that cannot be parsed
/// is treated the same way, because an unreadable config is not a config that
/// says nothing.
fn cargo_config_moves_build_directory(root: &Path, environment: &CargoBuildEnvironment) -> bool {
    let mut candidates: Vec<PathBuf> = Vec::new();
    for ancestor in root.ancestors() {
        candidates.push(ancestor.join(".cargo"));
    }
    if let Some(home) = environment.cargo_home.as_deref() {
        candidates.push(PathBuf::from(home));
    }
    for directory in candidates {
        for name in ["config.toml", "config"] {
            let path = directory.join(name);
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(value) = toml::from_str::<toml::Value>(&content) else {
                return true;
            };
            if let Some(build) = value.get("build").and_then(toml::Value::as_table)
                && CARGO_LAYOUT_CONFIG_KEYS
                    .iter()
                    .any(|key| build.contains_key(*key))
            {
                return true;
            }
            if let Some(env) = value.get("env").and_then(toml::Value::as_table)
                && CARGO_LAYOUT_CONFIG_ENV_KEYS
                    .iter()
                    .any(|key| env.contains_key(*key))
            {
                return true;
            }
        }
    }
    false
}

/// Cargo build-layout inputs that come from the environment rather than the argv.
///
/// Read once at the call site and passed in, so the derivation stays a pure
/// function of its inputs: process environment cannot be set from a test under
/// edition 2024 without `unsafe`, and a parallel suite must not share it.
#[derive(Debug, Default, Clone)]
struct CargoBuildEnvironment {
    target_dir: Option<String>,
    target_triple: Option<String>,
    cargo_home: Option<String>,
}

impl CargoBuildEnvironment {
    fn from_process() -> Self {
        let read = |name: &str| {
            std::env::var(name)
                .ok()
                .filter(|value| !value.trim().is_empty())
        };
        Self {
            target_dir: read("CARGO_TARGET_DIR").or_else(|| read("CARGO_BUILD_TARGET_DIR")),
            target_triple: read("CARGO_BUILD_TARGET"),
            cargo_home: read("CARGO_HOME").or_else(|| {
                read("HOME").map(|home| {
                    Path::new(&home)
                        .join(".cargo")
                        .to_string_lossy()
                        .into_owned()
                })
            }),
        }
    }
}

/// Resolves the `.cargo-lock` a configured Cargo command will contend on, or
/// `None` when that cannot be derived exactly from the argv and environment.
///
/// Silence is the honest answer for everything this cannot derive: a
/// `.cargo/config.toml` that sets `build.target-dir` or `build.target`, a
/// `--config` or `--manifest-path` argument, several `--target` triples, a
/// custom target JSON, a third-party subcommand whose flags mean something
/// else. A notice pointing at the wrong lock would name a file the command will
/// never wait on, and a stale `<root>/target` left behind by an earlier layout
/// makes that a live possibility rather than a theoretical one.
fn cargo_build_lock_path(
    root: &Path,
    program: &str,
    args: &[String],
    environment: &CargoBuildEnvironment,
) -> Option<PathBuf> {
    let program_name = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program)
        .trim_end_matches(".exe");
    if program_name != "cargo" {
        return None;
    }

    // `--config` can set the very keys this derivation cannot follow, and
    // `--manifest-path` moves the workspace whose `target/` is used. Either one
    // present anywhere in the command means the build directory is not derivable
    // from the rest of the argv.
    for argument in args {
        let argument = argument.as_str();
        if matches!(argument, "--manifest-path" | "--config")
            || argument.starts_with("--manifest-path=")
            || argument.starts_with("--config=")
        {
            return None;
        }
    }

    let mut target_dir = None;
    let mut index = 0;
    let subcommand = loop {
        let argument = args.get(index)?;
        if let Some(value) = argument.strip_prefix("--target-dir=") {
            target_dir = Some(value.to_string());
            index += 1;
            continue;
        }
        if CARGO_GLOBAL_VALUE_OPTIONS.contains(&argument.as_str()) {
            if argument == "--target-dir" {
                target_dir = Some(args.get(index + 1)?.clone());
            }
            index += 2;
            continue;
        }
        if argument.starts_with('-') {
            index += 1;
            continue;
        }
        break argument.as_str();
    };
    if !CARGO_BUILD_LOCK_SUBCOMMANDS.contains(&subcommand) {
        return None;
    }

    let mut profile = None;
    let mut release = false;
    let mut triples: Vec<String> = Vec::new();
    index += 1;
    while let Some(argument) = args.get(index) {
        if argument == "--" {
            break;
        }
        match argument.as_str() {
            "--release" | "-r" => {
                release = true;
                index += 1;
            }
            "--profile" => {
                profile = Some(args.get(index + 1)?.clone());
                index += 2;
            }
            "--target" => {
                triples.push(args.get(index + 1)?.clone());
                index += 2;
            }
            "--target-dir" => {
                target_dir = Some(args.get(index + 1)?.clone());
                index += 2;
            }
            other => {
                if let Some(value) = other.strip_prefix("--profile=") {
                    profile = Some(value.to_string());
                } else if let Some(value) = other.strip_prefix("--target=") {
                    triples.push(value.to_string());
                } else if let Some(value) = other.strip_prefix("--target-dir=") {
                    target_dir = Some(value.to_string());
                }
                index += 1;
            }
        }
    }

    // Cargo maps profile names onto directory names; only the four built-ins
    // are renamed, and a custom profile keeps its own name.
    let profile_directory = match profile.as_deref() {
        Some("dev" | "test") => "debug",
        Some("release" | "bench") => "release",
        Some(custom) => custom,
        None if release => "release",
        None if subcommand == "bench" => "release",
        None => "debug",
    };

    if triples.len() > 1 {
        return None;
    }
    let triple = triples
        .into_iter()
        .next()
        .or_else(|| environment.target_triple.clone());
    if triple
        .as_deref()
        .is_some_and(|triple| triple.ends_with(".json"))
    {
        // A custom target spec names its directory after the file stem; not
        // worth deriving, and a wrong path is worse than no notice.
        return None;
    }
    // A profile or triple is a single directory component. Anything carrying a
    // separator is not one, and joining it would probe somewhere else entirely.
    let is_single_component = |name: &str| {
        !name.is_empty()
            && name != "."
            && name != ".."
            && !name.contains(['/', '\\'])
            && !name.contains(':')
    };
    if !is_single_component(profile_directory) {
        return None;
    }
    if triple
        .as_deref()
        .is_some_and(|name| !is_single_component(name))
    {
        return None;
    }
    if cargo_config_moves_build_directory(root, environment) {
        return None;
    }

    let mut path = match target_dir.or_else(|| environment.target_dir.clone()) {
        Some(configured) => {
            let configured = PathBuf::from(configured);
            if configured.is_absolute() {
                configured
            } else {
                root.join(configured)
            }
        }
        None => root.join("target"),
    };
    if let Some(triple) = triple {
        path.push(triple);
    }
    path.push(profile_directory);
    path.push(CARGO_BUILD_LOCK_FILE);
    Some(path)
}

/// Reports whether the Cargo build-directory lock at `path` is held right now.
///
/// Read from the lock itself: a non-blocking exclusive acquisition either
/// succeeds (nothing held it) or reports contention. Nothing here infers from
/// elapsed time, so this cannot fire on a slow but healthy compile — which is
/// the whole point, because a "probably wedged" heuristic would be one more
/// signal that stopped carrying information while still looking like one.
fn cargo_build_lock_is_held(path: &Path) -> bool {
    let Ok(file) = OpenOptions::new().read(true).write(true).open(path) else {
        // The file is created by Cargo when it takes the lock; absent or
        // unreadable means there is nothing this run can prove.
        return false;
    };
    match file.try_lock_exclusive() {
        // Acquired, so nothing held it. Dropping `file` releases it again.
        Ok(()) => false,
        Err(error) => error.kind() == fs2::lock_contended_error().kind(),
    }
}

/// PIDs holding a `FLOCK` write lock on `major:minor:inode`, read from the text
/// of Linux `/proc/locks`.
///
/// Blocked waiters are printed with a `->` marker on the same lock id; they are
/// queued behind the holder, not holding it, so reporting one as the holder
/// would send the operator after the wrong process.
#[cfg(any(target_os = "linux", test))]
fn proc_locks_flock_holders(contents: &str, major: u32, minor: u32, inode: u64) -> Vec<u32> {
    let location = format!("{major:02x}:{minor:02x}:{inode}");
    let mut holders = Vec::new();
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        if fields.next().is_none() {
            continue;
        }
        let Some(kind) = fields.next() else {
            continue;
        };
        if kind != "FLOCK" {
            continue;
        }
        if fields.next().is_none() {
            continue;
        }
        if fields.next() != Some("WRITE") {
            continue;
        }
        let Some(Ok(pid)) = fields.next().map(str::parse::<i64>) else {
            continue;
        };
        if fields.next() != Some(location.as_str()) {
            continue;
        }
        // Open-file-description locks report `-1`; there is no process to name.
        if pid > 0 && u32::try_from(pid).is_ok_and(|pid| !holders.contains(&pid)) {
            holders.push(pid as u32);
        }
    }
    holders
}

/// PIDs currently holding the build-directory lock, where the platform can say.
///
/// Linux publishes lock ownership in `/proc/locks`. macOS publishes it nowhere
/// a process can read without either an external `lsof` — which verification's
/// own contract forbids it from spawning — or hand-written Darwin `libproc`
/// FFI. An empty result there is "not determinable", never "nobody holds it";
/// `cargo_build_lock_is_held` is the half that is always authoritative.
#[cfg(target_os = "linux")]
fn cargo_build_lock_holders(path: &Path) -> Vec<u32> {
    use std::os::unix::fs::MetadataExt;

    let Ok(metadata) = fs::metadata(path) else {
        return Vec::new();
    };
    let Ok(contents) = fs::read_to_string("/proc/locks") else {
        return Vec::new();
    };
    let device = metadata.dev();
    proc_locks_flock_holders(
        &contents,
        libc::major(device),
        libc::minor(device),
        metadata.ino(),
    )
}

#[cfg(not(target_os = "linux"))]
fn cargo_build_lock_holders(_path: &Path) -> Vec<u32> {
    Vec::new()
}

/// The line printed before a Cargo verification command that is about to block.
///
/// Cargo does say something of its own here — measured on 1.89 against a held
/// lock, `Blocking waiting for file lock on artifact directory` — so this notice
/// is ADDITIVE, not a replacement, and claiming otherwise would be its own false
/// premise. What Cargo's line does not say is which file, who holds it, or what
/// to do about it. This is emitted only when the lock is provably held, and it
/// names the holder's PID only where the platform actually reports ownership.
fn cargo_build_lock_wait_notice(
    root: &Path,
    program: &str,
    args: &[String],
    environment: &CargoBuildEnvironment,
) -> Option<String> {
    let path = cargo_build_lock_path(root, program, args, environment)?;
    if !cargo_build_lock_is_held(&path) {
        return None;
    }
    let display = portable_project_path(root, &path);
    let holders = cargo_build_lock_holders(&path);
    if holders.is_empty() {
        let mut notice = format!(
            "specsync: waiting on {display}: the Cargo build-directory lock is held by another \
             process, so this command is blocked rather than compiling"
        );
        // `lsof` lists the processes with the file OPEN, which is not the same
        // question — on macOS it does not report lock state at all. It is still
        // the right command, because holding a `flock` requires an open
        // descriptor, so the holder is always among what it lists. Saying that
        // plainly is the difference between a remedy and a wrong claim.
        if cfg!(unix) {
            notice.push_str(&format!(
                "\nspecsync: `lsof {display}` lists the processes holding it open, which is where \
                 a lock holder you can see will be — run it as the owning user, and end an orphan \
                 left by an interrupted check"
            ));
        }
        return Some(notice);
    }
    let named = holders
        .iter()
        .map(|pid| format!("PID {pid}"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "specsync: waiting on {display} held by {named}: the Cargo build-directory lock is taken, \
         so this command is blocked rather than compiling"
    ))
}

/// Process groups of live verification children, for teardown on a signal.
///
/// A fixed slot table rather than a lock: the reaper runs inside a signal
/// handler, where an atomic load and `kill` are async-signal-safe and taking a
/// mutex is not. The CLI runs exactly one verification child at a time — both
/// call sites are sequential loops under the project lock — so the extra slots
/// exist for the test binary, which verifies on several threads at once.
#[cfg(unix)]
const VERIFICATION_CHILD_GROUP_SLOTS: usize = 8;

#[cfg(unix)]
static VERIFICATION_CHILD_GROUPS: [std::sync::atomic::AtomicI32; VERIFICATION_CHILD_GROUP_SLOTS] =
    [const { std::sync::atomic::AtomicI32::new(0) }; VERIFICATION_CHILD_GROUP_SLOTS];

/// Signals whose default disposition ends this process, forwarded to the
/// verification child's own process group before this process dies of them.
#[cfg(unix)]
const VERIFICATION_REAPED_SIGNALS: [libc::c_int; 4] =
    [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];

#[cfg(unix)]
extern "C" fn reap_verification_children(signal: libc::c_int) {
    for slot in &VERIFICATION_CHILD_GROUPS {
        let group = slot.load(Ordering::SeqCst);
        // `> 1` and not `> 0`: `kill(0, ..)` signals this process's own group and
        // `kill(-1, ..)` signals every process the user can signal. Neither is
        // reachable from a `fork`ed child's PID, and neither may become
        // reachable through a future edit to how the slot is filled.
        if group > 1 {
            // SAFETY: `kill` is async-signal-safe. A group that already exited
            // fails with ESRCH, which is the expected no-op here.
            unsafe { libc::kill(-group, signal) };
        }
    }
    // SAFETY: restoring the default disposition and re-raising keeps the exit
    // status a wrapping shell or supervisor would otherwise have seen.
    unsafe {
        libc::signal(signal, libc::SIG_DFL);
        libc::raise(signal);
    }
}

#[cfg(unix)]
fn install_verification_reaper() {
    static INSTALL: OnceLock<()> = OnceLock::new();
    INSTALL.get_or_init(|| {
        let handler = reap_verification_children as extern "C" fn(libc::c_int);
        for signal in VERIFICATION_REAPED_SIGNALS {
            // SAFETY: installing a handler for a terminating signal. A signal
            // the parent inherited as ignored stays ignored — a backgrounded
            // job is started that way, and stealing SIGINT back from it would
            // change how the parent itself behaves.
            unsafe {
                let previous = libc::signal(signal, handler as libc::sighandler_t);
                if previous == libc::SIG_IGN {
                    libc::signal(signal, libc::SIG_IGN);
                }
            }
        }
    });
}

/// Registration for one verification child's process group.
///
/// Drop covers the non-signal exits (an unwind, a failed wait); the installed
/// handler covers the signal ones. Together they mean an interrupted check
/// cannot outlive itself holding the Cargo build-directory lock.
#[cfg(unix)]
struct VerificationChildGroup {
    slot: Option<usize>,
    group: i32,
    reaped: bool,
}

#[cfg(unix)]
impl VerificationChildGroup {
    fn arm(group: i32) -> Self {
        install_verification_reaper();
        let slot = VERIFICATION_CHILD_GROUPS.iter().position(|slot| {
            slot.compare_exchange(0, group, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        });
        Self {
            slot,
            group,
            reaped: false,
        }
    }

    /// Records that `wait` reaped the child, and unpublishes the group at the
    /// same moment.
    ///
    /// Clearing the slot here rather than in `Drop` matters: once the child is
    /// reaped its PID is free for reuse, so a signal arriving between `wait`
    /// returning and the guard dropping would have the handler signal a group
    /// this process never created.
    fn disarm(&mut self) {
        self.reaped = true;
        self.clear_slot();
    }

    fn clear_slot(&mut self) {
        if let Some(slot) = self.slot.take() {
            VERIFICATION_CHILD_GROUPS[slot].store(0, Ordering::SeqCst);
        }
    }
}

#[cfg(unix)]
impl Drop for VerificationChildGroup {
    fn drop(&mut self) {
        if !self.reaped && self.group > 1 {
            // SAFETY: signalling a process group this process created. The child
            // is unreaped here, so its PID cannot yet have been reused.
            unsafe { libc::kill(-self.group, libc::SIGTERM) };
        }
        self.clear_slot();
    }
}

/// Runs one verification child to completion.
///
/// On Unix the child leads its own process group, so ending that group ends
/// `cargo` and everything it spawned rather than only the process this one
/// waited on. A `SIGKILL`ed parent still orphans its child — no process runs
/// code after `SIGKILL` — which is exactly why the wait notice is not optional.
#[cfg(unix)]
#[allow(dead_code)]
fn run_verification_child(command: &mut Command) -> std::io::Result<std::process::ExitStatus> {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
    // Installed BEFORE the spawn. The child leaves the terminal's foreground
    // group the instant it exists, so between `spawn` returning and the group
    // being registered nothing would forward a signal to it — a window in which
    // this change would orphan a child that the unmodified code would not.
    // Installing first shrinks that window to the one instruction that publishes
    // the PID, which cannot be known any earlier.
    install_verification_reaper();
    let mut child = command.spawn()?;
    let mut group = VerificationChildGroup::arm(child.id() as i32);
    let status = child.wait()?;
    group.disarm();
    Ok(status)
}

#[cfg(not(unix))]
#[allow(dead_code)]
fn run_verification_child(command: &mut Command) -> std::io::Result<std::process::ExitStatus> {
    command.status()
}

#[allow(dead_code)]
fn run_configured_command(
    root: &Path,
    configured: &str,
) -> Result<std::process::ExitStatus, String> {
    let parts = shell_words(configured)?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| "empty verification command".to_string())?;
    if let Some(notice) =
        cargo_build_lock_wait_notice(root, program, args, &CargoBuildEnvironment::from_process())
    {
        eprintln!("{notice}");
    }
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(root)
        .env(crate::VERIFICATION_CONTEXT_ENV, configured);
    run_verification_child(&mut command).map_err(|error| {
        format!("failed to run configured verification command `{configured}`: {error}")
    })
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
            "../<!-- What led here: the problem, and how it was noticed. -->\n\n",
            "<!-- What a session picking this up mid-flight needs to know: constraints,\n",
            "     prior attempts, anything already ruled out. -->\n\n",
        ),
        ArtifactKind::Plan => concat!(
            "../## Approach\n\n<!-- What you will do. -->\n\n",
            "## Out of scope\n\n<!-- What you will not do. -->\n\n",
            "## Steps\n\n<!-- The steps, in order. -->\n\n",
            "## Risks\n\n<!-- What could go wrong, or none. -->\n\n",
            "## Constraints consulted\n\n<!-- Which existing lessons or principles this change honors, or none. -->\n\n",
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
#[path = "../change_tests.rs"]
mod tests;
