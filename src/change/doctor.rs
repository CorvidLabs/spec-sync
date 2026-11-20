// ─── `specsync doctor` and the late close (REQ-cmd-change-035, REQ-change-143) ───

/// Where a late close puts a package: under `.specsync/archive/`, which every SpecSync 6.x treats
/// as a non-input, and outside `.specsync/archive/changes/`, the only archive directory 6.x
/// enumerates. SpecSync 6.0 to 6.2 therefore never read a closed-late change, so they neither trip
/// on it nor count it as checked (SHIP-12.g).
const CLOSED_LATE_PATH: &str = ".specsync/archive/closed-late";
const LATE_CLOSE_RECORD: &str = "late-close.json";

/// The acknowledged sequence collision, as it stood, that a late close took this change out of in
/// `.specsync/change-sequence.json`, because the change left the directories SpecSync enumerates (D5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateCloseCollision {
    pub sequence: u64,
    pub ids: Vec<String>,
}

/// What a late close records beside the package it moved (REQ-change-143).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateCloseRecord {
    pub schema_version: u32,
    pub change_id: String,
    pub actor: String,
    pub reason: String,
    pub timestamp: u64,
    /// The state the change's own `state.json` still holds, byte for byte.
    pub from_state: ChangeState,
    pub workflow_version: u32,
    /// The project path the package was moved from.
    pub from_path: String,
    pub default_ref: String,
    pub default_commit: String,
    /// The commit on `default_ref` that added the package at `from_path`, and its committer date.
    pub merged_commit: String,
    pub merged_at: String,
    /// Always false: a late close re-checks nothing (SHIP-12.b).
    pub verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropped_collision_acknowledgement: Option<LateCloseCollision>,
}

/// Whether `name` is a closed-late package directory for `id`: `YYYY-MM-DD-<id>`, the only form a
/// late close writes.
fn is_closed_late_name_for(name: &str, id: &str) -> bool {
    let Some(date) = name.strip_suffix(id).and_then(|rest| rest.strip_suffix('-')) else {
        return false;
    };
    let bytes = date.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 4 | 7) {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}

/// The closed-late package for `id`, if there is one.
fn closed_late_package(root: &Path, id: &str) -> Option<PathBuf> {
    fs::read_dir(root.join(CLOSED_LATE_PATH))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| is_closed_late_name_for(name, id))
        })
}

/// Whether `id` is held only by a closed-late package: no active workspace and no archive package
/// records it. Status reports the closed-late record only then, so it never contradicts the
/// records the lifecycle verbs act on (Codex review of #994).
pub fn is_only_closed_late(root: &Path, id: &str) -> bool {
    !change_dir(root, id).is_dir()
        && archived_package_for(root, id).is_none()
        && closed_late_package(root, id).is_some()
}

/// The closed-late package for `id` and its record, read from `late-close.json`.
pub fn closed_late_record(
    root: &Path,
    id: &str,
) -> Option<(PathBuf, Result<LateCloseRecord, String>)> {
    let package = closed_late_package(root, id)?;
    let path = package.join(LATE_CLOSE_RECORD);
    let record = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|error| format!("invalid {}: {error}", path.display()))
        });
    Some((package, record))
}

/// The one sentence every verb gives for a closed-late change: who closed it, when and why, the
/// commit that merged it, and that it was never re-checked and is not verified (SHIP-12.a).
pub fn closed_late_sentence(root: &Path, id: &str) -> Option<String> {
    let (package, record) = closed_late_record(root, id)?;
    let location = portable_project_path(root, &package);
    Some(match record {
        Ok(record) => format!(
            "change `{id}` was closed late on {} by {} ({}); it merged in {} on {} and was never re-checked, so it is not verified{}; its package is `{location}`",
            utc_date(record.timestamp),
            printable_record_text(&record.actor),
            printable_record_text(&record.reason),
            short_commit(&printable_record_text(&record.merged_commit)),
            printable_record_text(&record.merged_at),
            if record.from_state == ChangeState::Draft {
                "; it was never approved"
            } else {
                ""
            },
        ),
        Err(error) => format!(
            "change `{id}` was closed late, but its record cannot be read ({}); it is not verified; its package is `{location}`",
            printable_record_text(&error)
        ),
    })
}

fn short_commit(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

/// The IDs of every closed-late package, from the dated directory names.
fn closed_late_ids(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = fs::read_dir(root.join(CLOSED_LATE_PATH)) else {
        return Vec::new();
    };
    let mut ids: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let name = entry.file_name().to_str()?.to_string();
            let id = name.get(11..)?.to_string();
            is_closed_late_name_for(&name, &id).then(|| (id, entry.path()))
        })
        .collect();
    ids.sort();
    ids
}

/// How a doctor finding's fix is applied (RECOVER-3.c).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorFixClass {
    /// `doctor --fix` applies it: it completes or restores recorded state and claims nothing new.
    Safe,
    /// A person decides it: `--close-late` with `--actor` and `--reason`.
    Decision,
    /// Doctor names the command and applies nothing.
    ReportOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorFix {
    pub class: DoctorFixClass,
    /// The exact command that applies it, or what to edit when no command can.
    pub command: String,
    /// Another fix a person may choose instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternative: Option<String>,
}

/// One problem `specsync doctor` found (REQ-cmd-change-035).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorFinding {
    pub kind: &'static str,
    /// The change it is about; `None` for a project-wide finding.
    pub change: Option<String>,
    /// `active`, `archive`, `closed-late` or `project`.
    pub location: &'static str,
    pub reason: String,
    pub fix: DoctorFix,
}

/// What `doctor --fix` did, or with `--dry-run` would do.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorAction {
    pub kind: &'static str,
    pub change: Option<String>,
    /// `applied`, `would-apply`, `after-revalidation` (a dry run's later lifecycle moves), `skipped`
    /// or `failed`.
    pub outcome: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub findings: Vec<DoctorFinding>,
    pub dry_run: bool,
    pub actions: Vec<DoctorAction>,
}

impl DoctorReport {
    /// Whether every action it attempted succeeded.
    pub fn is_clean(&self) -> bool {
        self.actions.iter().all(|action| action.outcome != "failed")
    }
}

/// What `specsync doctor` was asked to do.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DoctorRequest {
    pub fix: bool,
    pub dry_run: bool,
    /// Change IDs to close late, or `all`.
    pub close_late: Vec<String>,
    pub actor: Option<String>,
    pub reason: Option<String>,
}

/// `id` as one shell word: unchanged when it holds only characters a shell leaves alone, single-quoted
/// otherwise, so a printed fix command can be copied whatever the ID holds (Codex review of #994).
fn shell_word(id: &str) -> String {
    if !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
    {
        id.to_string()
    } else {
        shell_quote(id)
    }
}

fn close_late_command(id: &str) -> String {
    format!(
        "specsync doctor --fix --close-late {} --actor <name> --reason <text>",
        shell_word(id)
    )
}

/// Recorded text is printed, not obeyed: control and bidirectional formatting characters are shown
/// as escapes, so an actor or reason cannot move the cursor, recolour the terminal or reorder the
/// line it is printed on (Codex review of #994).
fn printable_record_text(text: &str) -> String {
    let mut printable = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_control() {
            printable.extend(character.escape_debug());
        } else if matches!(
            character,
            '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
        ) {
            printable.extend(character.escape_unicode());
        } else {
            printable.push(character);
        }
    }
    printable
}

fn finding(
    kind: &'static str,
    change: Option<&str>,
    location: &'static str,
    reason: String,
    class: DoctorFixClass,
    command: String,
) -> DoctorFinding {
    DoctorFinding {
        kind,
        change: change.map(str::to_string),
        location,
        reason,
        fix: DoctorFix {
            class,
            command,
            alternative: None,
        },
    }
}

/// Every problem doctor can find in the repository's change records, each with its fix. It writes
/// nothing. A check that cannot run becomes a finding that names its error, and the scan goes on
/// (RECOVER-3, RECOVER-3.a, RECOVER-3.d).
fn doctor_scan(root: &Path, closing: &BTreeSet<String>) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    let roster = match list_changes(root) {
        Ok(roster) => roster,
        Err(error) => {
            findings.push(finding(
                "could-not-check",
                None,
                "project",
                format!("the active changes could not be listed: {error}"),
                DoctorFixClass::ReportOnly,
                "repair the change directory named in the reason, then run `specsync doctor` again".into(),
            ));
            ChangeRoster::default()
        }
    };
    for unreadable in &roster.unreadable {
        findings.push(finding(
            "could-not-check",
            Some(&unreadable.id),
            "active",
            unreadable.reason.clone(),
            DoctorFixClass::ReportOnly,
            "restore the record from Git (`git checkout HEAD -- <path>`) or remove the stray directory".into(),
        ));
    }
    let reference = remote_default_ref(root);
    if reference.is_none() && !roster.records.is_empty() {
        findings.push(finding(
            "could-not-check",
            None,
            "project",
            "no remote default branch (`origin/HEAD`, `origin/main` or `origin/master`) resolves, so doctor cannot tell which changes merged but were never archived".into(),
            DoctorFixClass::ReportOnly,
            "git fetch origin && git remote set-head origin --auto".into(),
        ));
    }
    for record in &roster.records {
        if let Some(reference) = &reference
            && package_is_on_reference(root, &format!("{CHANGES_PATH}/{}/state.json", record.id), reference)
        {
            findings.push(finding(
                "merged-never-archived",
                Some(&record.id),
                "active",
                format!(
                    "it is on `{reference}` but was never archived; it is still {}",
                    record.state.as_str()
                ),
                DoctorFixClass::Decision,
                close_late_command(&record.id),
            ));
        }
    }

    // Each change is judged on its own, against every record that could be read, so one record
    // that cannot be read never hides the others (RECOVER-3.d).
    let mut records: BTreeMap<String, ChangeRecord> = roster
        .records
        .iter()
        .map(|record| (record.id.clone(), record.clone()))
        .collect();
    for record in archived_records(root) {
        records.entry(record.id.clone()).or_insert(record);
    }
    // A change named for a late close is not read for evidence: the late close re-checks nothing,
    // and on the changes it exists for the read is what fails (Codex review of #994).
    let accepted: Vec<(&ChangeRecord, TerminalEvidenceSummary)> = roster
        .records
        .iter()
        .filter(|record| record.state == ChangeState::Accepted && !closing.contains(&record.id))
        .map(|record| {
            let evidence = terminal_evidence_summary_with_records(root, record, &records);
            (record, evidence)
        })
        .collect();
    // Every accepted record whose evidence is not current stops the archive preflight a finalize
    // or archive runs, so resuming one is not safe while another is still there (Codex review of
    // #994). An accepted package left in the archive stops it too.
    let mut blockers: BTreeSet<String> = accepted
        .iter()
        .filter(|(_, evidence)| {
            !matches!(
                evidence.validity,
                TerminalEvidenceValidity::Exact
                    | TerminalEvidenceValidity::SuccessorCovered
                    | TerminalEvidenceValidity::ProvisionallyCovered
            )
        })
        .map(|(record, _)| record.id.clone())
        .collect();
    blockers.extend(
        archived_accepted_packages(root)
            .into_iter()
            .map(|(_, record)| record.id)
            .filter(|id| !closing.contains(id)),
    );
    for (record, evidence) in &accepted {
        if evidence.validity == TerminalEvidenceValidity::Stale {
            let reopen = format!(
                "specsync change reopen {} --actor <name> --reason <text>",
                shell_word(&record.id)
            );
            let merged = reference.as_ref().is_some_and(|reference| {
                package_is_on_reference(
                    root,
                    &format!("{CHANGES_PATH}/{}/state.json", record.id),
                    reference,
                )
            });
            // A late close applies only to a merged change, so an unmerged one is offered the
            // reopen that can actually run (Codex review of #994).
            let mut drifted = finding(
                "accepted-inputs-drifted",
                Some(&record.id),
                "active",
                evidence
                    .reason
                    .clone()
                    .unwrap_or_else(|| "its accepted evidence is no longer current".into()),
                DoctorFixClass::Decision,
                if merged {
                    close_late_command(&record.id)
                } else {
                    reopen.clone()
                },
            );
            if merged {
                drifted.fix.alternative = Some(reopen);
            }
            findings.push(drifted);
            continue;
        }
        if !matches!(
            evidence.validity,
            TerminalEvidenceValidity::Exact
                    | TerminalEvidenceValidity::SuccessorCovered
                    | TerminalEvidenceValidity::ProvisionallyCovered
        ) {
            findings.push(finding(
                "could-not-check",
                Some(&record.id),
                "active",
                evidence.reason.clone().unwrap_or_else(|| {
                    format!("its accepted evidence reads as {}", evidence.validity.as_str())
                }),
                DoctorFixClass::ReportOnly,
                format!("specsync change status {}", shell_word(&record.id)),
            ));
            continue;
        }
        let (stopped, command) = if record.workflow_version >= 2 {
            if !change_dir(root, &record.id).join("finalization.json").is_file() {
                continue;
            }
            (
                "its finalize stopped after acceptance; the package was never archived",
                format!("specsync change finalize {}", shell_word(&record.id)),
            )
        } else {
            (
                "it was accepted under workflow v1 and never archived",
                format!("specsync change archive {}", shell_word(&record.id)),
            )
        };
        let others: Vec<String> = blockers
            .iter()
            .filter(|id| *id != &record.id)
            .map(|id| format!("`{id}`"))
            .collect();
        findings.push(if others.is_empty() {
            finding(
                "stopped-finalize",
                Some(&record.id),
                "active",
                stopped.into(),
                DoctorFixClass::Safe,
                command,
            )
        } else {
            finding(
                "stopped-finalize",
                Some(&record.id),
                "active",
                format!(
                    "{stopped}; the archive preflight would stop at {} first, so close those late or reopen them, then `doctor --fix` resumes it",
                    others.join(", ")
                ),
                DoctorFixClass::ReportOnly,
                command,
            )
        });
    }
    let verifying: Vec<&ChangeRecord> = roster
        .records
        .iter()
        .filter(|record| {
            record.state == ChangeState::Verifying
                || (record.state == ChangeState::Implementing
                    && change_dir(root, &record.id).join("verification.json").is_file())
        })
        .collect();
    if !verifying.is_empty() {
        match project_input_digest(root) {
            Ok(project_digest) => {
                for record in verifying {
                    let current = load_verification(root, record).and_then(|evidence| {
                        verification_is_current_checked_with_project_digest(
                            root,
                            record,
                            &evidence,
                            &project_digest,
                        )
                    });
                    if let Err(reason) = current {
                        findings.push(finding(
                            "stale-verification",
                            Some(&record.id),
                            "active",
                            format!("its verification is not current: {reason}"),
                            DoctorFixClass::ReportOnly,
                            format!("specsync change check {} --commit", shell_word(&record.id)),
                        ));
                    }
                }
            }
            Err(error) => findings.push(finding(
                "could-not-check",
                None,
                "project",
                format!("the project inputs verification is judged against could not be read: {error}"),
                DoctorFixClass::ReportOnly,
                "repair the input the reason names, then run `specsync doctor` again".into(),
            )),
        }
    }
    // The audit reports the remaining record problems; the ones judged above, and records that
    // could not be read, are not reported twice.
    let roster_ids: BTreeSet<&str> = roster
        .records
        .iter()
        .map(|record| record.id.as_str())
        .chain(roster.unreadable.iter().map(|unreadable| unreadable.id.as_str()))
        .collect();
    let audit = audit_project(root);
    for error in &audit.errors {
        if roster
            .unreadable
            .iter()
            .any(|unreadable| error.contains(&unreadable.reason) || unreadable.reason.contains(error.as_str()))
        {
            continue;
        }
        let (change, message) = match error.split_once(": ") {
            Some((id, message)) if roster_ids.contains(id) => (Some(id), message),
            _ => (None, error.as_str()),
        };
        if change.is_some_and(|id| roster.unreadable.iter().any(|unreadable| unreadable.id == id))
            || message.starts_with("accepted change verification is stale for current delivery inputs")
            || message.contains("verification evidence is stale for the current commit or contract")
            || message.contains("latest verification evidence failed")
        {
            continue;
        }
        if message.contains("artifact is incomplete") {
            findings.push(finding(
                "other-version-record",
                change,
                if change.is_some() { "active" } else { "project" },
                message.to_string(),
                DoctorFixClass::ReportOnly,
                "complete the artifact the reason names (a workflow-v1 plan needs the 6.2 sections)".into(),
            ));
            continue;
        }
        findings.push(finding(
            "audit-error",
            change,
            if change.is_some() { "active" } else { "project" },
            message.to_string(),
            DoctorFixClass::ReportOnly,
            "follow the remediation in the reason (`specsync change audit` reports the same)".into(),
        ));
    }
    match backfill_reopen_digests(root, true) {
        Ok(backfill) => {
            for id in &backfill.repaired {
                findings.push(finding(
                    "other-version-record",
                    Some(id),
                    "active",
                    "its reopen ledger was written by SpecSync 5.0.1 and lacks the 5.1 digest fields".into(),
                    DoctorFixClass::Safe,
                    "specsync migrate 5.0".into(),
                ));
            }
            for (id, reason) in &backfill.failed {
                findings.push(finding(
                    "could-not-check",
                    Some(id),
                    "active",
                    format!("its reopen ledger cannot be backfilled: {reason}"),
                    DoctorFixClass::ReportOnly,
                    "repair the reopen ledger the reason names".into(),
                ));
            }
        }
        Err(error) => findings.push(finding(
            "could-not-check",
            None,
            "project",
            format!("reopen ledgers could not be inspected: {error}"),
            DoctorFixClass::ReportOnly,
            "repair the record the reason names".into(),
        )),
    }
    for (package, error) in unreadable_archive_records(root) {
        findings.push(finding(
            "could-not-check",
            None,
            "archive",
            format!(
                "`{}` cannot be read: {error}",
                portable_project_path(root, &package)
            ),
            DoctorFixClass::ReportOnly,
            "restore the archived record from Git (`git checkout HEAD -- <path>`)".into(),
        ));
    }
    for (package, record) in archived_accepted_packages(root) {
        findings.push(finding(
            "accepted-in-archive",
            Some(&record.id),
            "archive",
            format!(
                "`{}` is in the archive but its state still says accepted",
                portable_project_path(root, &package)
            ),
            DoctorFixClass::Decision,
            close_late_command(&record.id),
        ));
    }
    for (id, package) in closed_late_ids(root) {
        if let Some((_, Err(error))) = closed_late_record(root, &id) {
            findings.push(finding(
                "could-not-check",
                Some(&id),
                "closed-late",
                format!(
                    "`{}` was closed late, but its record cannot be read: {error}",
                    portable_project_path(root, &package)
                ),
                DoctorFixClass::ReportOnly,
                "restore late-close.json from Git (`git checkout HEAD -- <path>`)".into(),
            ));
        }
        if find_change_dir(root, &id).is_ok() {
            findings.push(finding(
                "duplicate-id",
                Some(&id),
                "closed-late",
                format!(
                    "`{}` was closed late, and another package holds the same ID; an older SpecSync, which cannot see closed-late packages, can mint it again",
                    portable_project_path(root, &package)
                ),
                DoctorFixClass::ReportOnly,
                "recreate the newer change from a distinct description".into(),
            ));
        }
    }
    findings.sort_by(|left, right| {
        (&left.change, left.kind).cmp(&(&right.change, right.kind))
    });
    findings
}

/// The changes `--close-late all` can close: active packages and accepted packages left in the
/// archive whose `state.json` is on the remote default. Decided from merge presence alone, so no
/// delivery evidence is read for them.
fn merged_late_close_candidates(root: &Path) -> BTreeSet<String> {
    let Some(reference) = remote_default_ref(root) else {
        return BTreeSet::new();
    };
    let mut candidates = BTreeSet::new();
    if let Ok(roster) = list_changes(root) {
        for record in roster.records {
            if package_is_on_reference(
                root,
                &format!("{CHANGES_PATH}/{}/state.json", record.id),
                &reference,
            ) {
                candidates.insert(record.id);
            }
        }
    }
    for (package, record) in archived_accepted_packages(root) {
        if package_is_on_reference(
            root,
            &format!("{}/state.json", portable_project_path(root, &package)),
            &reference,
        ) {
            candidates.insert(record.id);
        }
    }
    candidates
}

/// Every package under `.specsync/archive/changes/` whose `state.json` exists but cannot be read
/// or parsed. A directory with no `state.json` is a legacy tombstone or a Git husk, as the roster
/// treats it, and is not reported.
fn unreadable_archive_records(root: &Path) -> Vec<(PathBuf, String)> {
    let Ok(entries) = fs::read_dir(root.join(ARCHIVE_PATH)) else {
        return Vec::new();
    };
    let mut unreadable: Vec<(PathBuf, String)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.join("state.json").is_file())
        .filter_map(|path| {
            let error = match fs::read(path.join("state.json")) {
                Err(error) => error.to_string(),
                Ok(content) => serde_json::from_slice::<ChangeRecord>(&content).err()?.to_string(),
            };
            Some((path, error))
        })
        .collect();
    unreadable.sort();
    unreadable
}

/// Every record under `.specsync/archive/changes/` that parses; a package that does not is
/// reported by `unreadable_archive_records`.
fn archived_records(root: &Path) -> Vec<ChangeRecord> {
    let Ok(entries) = fs::read_dir(root.join(ARCHIVE_PATH)) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| serde_json::from_slice(&fs::read(path.join("state.json")).ok()?).ok())
        .collect()
}

/// Packages under `.specsync/archive/changes/` whose `state.json` still says `accepted`: the
/// hand-archived shape #833 found in Corvidinho.
fn archived_accepted_packages(root: &Path) -> Vec<(PathBuf, ChangeRecord)> {
    let Ok(entries) = fs::read_dir(root.join(ARCHIVE_PATH)) else {
        return Vec::new();
    };
    let mut packages: Vec<(PathBuf, ChangeRecord)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            let record: ChangeRecord =
                serde_json::from_slice(&fs::read(path.join("state.json")).ok()?).ok()?;
            (record.state == ChangeState::Accepted).then_some((path, record))
        })
        .collect();
    packages.sort_by(|left, right| left.0.cmp(&right.0));
    packages
}

/// Whether the project path `project_path` exists in the tree of `reference`.
fn package_is_on_reference(root: &Path, project_path: &str, reference: &str) -> bool {
    let Ok(repo_path) = git_repo_relative_path(root, project_path) else {
        return false;
    };
    git_output_allow_empty(root, &["cat-file", "-e", &format!("{reference}:{repo_path}")]).is_some()
}

/// One validated late close, ready to write.
struct LateClosePlan {
    source: PathBuf,
    destination: PathBuf,
    record: LateCloseRecord,
}

/// Refuse existing links (including dangling links) before creating or moving anything. The
/// lifecycle lock uses the same component walk; checking only `exists()` follows the link.
fn validate_late_close_destination(root: &Path, destination: &Path) -> Result<(), String> {
    reject_symlink_components_for(root, destination, "late-close destination").map_err(|error| {
        if error.starts_with("refusing symlinked ") {
            format!("{error}; replace the symlinked parent with an in-project directory, or remove the link at the dated destination; leave the dated destination absent, then retry")
        } else {
            error
        }
    })?;
    match fs::symlink_metadata(destination) {
        Ok(_) => Err(format!("{} already exists", portable_project_path(root, destination))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("failed to inspect {}: {error}", destination.display())),
    }
}

/// Check one change for a late close and plan it; nothing is written (SHIP-12.e).
fn plan_late_close(root: &Path, id: &str, actor: &str, reason: &str, date: &str) -> Result<LateClosePlan, String> {
    validate_change_id(id)?;
    let destination = root.join(CLOSED_LATE_PATH).join(format!("{date}-{id}"));
    if closed_late_package(root, id).is_some() {
        return Err(format!("change `{id}` was already closed late"));
    }
    validate_late_close_destination(root, &destination)?;
    let active = change_dir(root, id);
    if active.is_dir()
        && let Some(archived) = archived_package_for(root, id)
    {
        return Err(format!(
            "change `{id}` is ambiguous: both `{}` and `{}` hold it; decide which package is the change before closing it late",
            portable_project_path(root, &active),
            portable_project_path(root, &archived)
        ));
    }
    let source = if active.is_dir() {
        active
    } else {
        match archived_package_for(root, id) {
            Some(package) => package,
            None => return Err(format!("change `{id}` was not found")),
        }
    };
    let state_path = source.join("state.json");
    let record: ChangeRecord = fs::read(&state_path)
        .map_err(|error| format!("failed to read {}: {error}", state_path.display()))
        .and_then(|content| {
            serde_json::from_slice(&content)
                .map_err(|error| format!("invalid change state {}: {error}", state_path.display()))
        })?;
    if record.id != id {
        return Err(format!(
            "{} names change `{}`, not `{id}`",
            state_path.display(),
            record.id
        ));
    }
    // A record this binary cannot validate is not moved: its lifecycle format is unknown here
    // (Codex review of #994).
    validate_loaded_change(&record, id, &state_path)
        .map_err(|error| format!("change `{id}` cannot be closed late: {error}"))?;
    // The package keeps its bytes, so a late-close record it already holds would be overwritten.
    let existing_record = source.join(LATE_CLOSE_RECORD);
    if fs::symlink_metadata(&existing_record).is_ok() {
        return Err(format!(
            "change `{id}` cannot be closed late: its package already holds `{}`",
            portable_project_path(root, &existing_record)
        ));
    }
    let from_path = portable_project_path(root, &source);
    if from_path.starts_with(&format!("{ARCHIVE_PATH}/")) && record.state != ChangeState::Accepted {
        return Err(format!(
            "change `{id}` is already archived; a late close applies only to a change that was never archived"
        ));
    }
    let default_ref = remote_default_ref(root).ok_or_else(|| {
        format!(
            "change `{id}` cannot be closed late: no remote default branch (`origin/HEAD`, `origin/main` or `origin/master`) resolves, so SpecSync cannot tell whether its pull request merged; fetch the remote, then retry"
        )
    })?;
    let default_commit = git_output(root, &["rev-parse", "--verify", &format!("{default_ref}^{{commit}}")])
        .ok_or_else(|| format!("`{default_ref}` does not name a commit"))?;
    let state_project_path = format!("{from_path}/state.json");
    if !package_is_on_reference(root, &state_project_path, &default_ref) {
        return Err(format!(
            "change `{id}` has not merged: `{state_project_path}` is not on `{default_ref}`, so its pull request has not merged; finish it through review and finalize instead"
        ));
    }
    // The package must be what merged, not a later version an unmerged branch advanced.
    let package_repo_path = git_repo_relative_path(root, &from_path)?;
    let package_pathspec = format!(":(top,literal){package_repo_path}");
    let unchanged = git_output_allow_empty(
        root,
        &["diff", "--quiet", &default_ref, "--", &package_pathspec],
    )
    .is_some()
        && git_output_allow_empty(
            root,
            &["ls-files", "--others", "--exclude-standard", "--", &package_pathspec],
        )
        .is_some_and(|untracked| untracked.trim().is_empty());
    if !unchanged {
        return Err(format!(
            "change `{id}`'s package `{from_path}` differs from `{default_ref}`: the version here has not merged, so its pull request has not merged; finish it through review and finalize instead"
        ));
    }
    let repo_path = git_repo_relative_path(root, &state_project_path)?;
    let added = git_output(
        root,
        &[
            "log",
            "-1",
            "--format=%H %cI",
            "--diff-filter=A",
            &default_ref,
            "--",
            &format!(":(top,literal){repo_path}"),
        ],
    )
    .ok_or_else(|| format!("no commit on `{default_ref}` adds `{state_project_path}`"))?;
    let (merged_commit, merged_at) = added
        .split_once(' ')
        .ok_or_else(|| format!("unexpected `git log` answer `{added}`"))?;
    Ok(LateClosePlan {
        source,
        destination,
        record: LateCloseRecord {
            schema_version: 1,
            change_id: id.to_string(),
            actor: actor.to_string(),
            reason: reason.to_string(),
            timestamp: now(),
            from_state: record.state,
            workflow_version: record.workflow_version,
            from_path,
            default_ref,
            default_commit,
            merged_commit: merged_commit.to_string(),
            merged_at: merged_at.to_string(),
            verified: false,
            dropped_collision_acknowledgement: None,
        },
    })
}

/// The package under `.specsync/archive/changes/` whose `state.json` records `id`. An archive is
/// identified by what it records, not by its directory name, which a hand archive need not
/// follow (#907).
fn archived_package_for(root: &Path, id: &str) -> Option<PathBuf> {
    fs::read_dir(root.join(ARCHIVE_PATH))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .find(|path| {
            fs::read(path.join("state.json"))
                .ok()
                .and_then(|content| serde_json::from_slice::<ChangeRecord>(&content).ok())
                .is_some_and(|record| record.id == id)
        })
}

/// Take `id` out of the acknowledged collision that names it, and return that acknowledgement as it
/// stood (D5). The acknowledgement stays for the members SpecSync still enumerates while at least
/// two remain, and goes once fewer than two do. The ledger's sequence and claim are untouched.
fn drop_collision_acknowledgement(
    ledger: &mut ChangeSequenceLedger,
    id: &str,
) -> Option<LateCloseCollision> {
    let index = ledger
        .acknowledged_collisions
        .iter()
        .position(|collision| collision.ids.iter().any(|member| member == id))?;
    let as_it_stood = LateCloseCollision {
        sequence: ledger.acknowledged_collisions[index].sequence,
        ids: ledger.acknowledged_collisions[index].ids.clone(),
    };
    let collision = &mut ledger.acknowledged_collisions[index];
    collision.ids.retain(|member| member != id);
    if collision.ids.len() < 2 {
        ledger.acknowledged_collisions.remove(index);
    }
    Some(as_it_stood)
}

/// Close every planned change late, or none: each package moves byte for byte into the
/// closed-late folder and gets its record, and a failed write restores what was moved.
fn write_late_closes(root: &Path, plans: &mut [LateClosePlan]) -> Result<(), String> {
    if plans.is_empty() {
        return Ok(());
    }
    let _lock = acquire_project_lock(root)?;
    // Planning (including dry-run) is read-only and happens before this lock. Recheck the whole
    // batch before any write so a link planted since planning cannot move even its first member.
    for plan in plans.iter() {
        validate_late_close_destination(root, &plan.destination)
            .map_err(|error| format!("{error}; nothing was closed late"))?;
    }
    let mut ledger = load_change_sequence_ledger(root)?;
    let mut ledger_changed = false;
    if let Some(ledger) = ledger.as_mut() {
        for plan in plans.iter_mut() {
            if let Some(collision) = drop_collision_acknowledgement(ledger, &plan.record.change_id) {
                plan.record.dropped_collision_acknowledgement = Some(collision);
                ledger_changed = true;
            }
        }
    }
    let ledger_path = root.join(SEQUENCE_PATH);
    let original_ledger = if ledger_changed {
        Some(fs::read(&ledger_path).map_err(|error| {
            format!("failed to read {}: {error}", ledger_path.display())
        })?)
    } else {
        None
    };
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut write = || -> Result<(), String> {
        fs::create_dir_all(root.join(CLOSED_LATE_PATH))
            .map_err(|error| format!("failed to create {CLOSED_LATE_PATH}: {error}"))?;
        for plan in plans.iter() {
            // Like lifecycle-lock acquisition, check again after creating the parent and at
            // each move boundary. The existing rollback still protects earlier batch members.
            validate_late_close_destination(root, &plan.destination)?;
            rename_durable(&plan.source, &plan.destination)?;
            moved.push((plan.source.clone(), plan.destination.clone()));
            write_json(&plan.destination.join(LATE_CLOSE_RECORD), &plan.record)?;
        }
        if ledger_changed && let Some(ledger) = &ledger {
            write_json_keeping_unknown_members(&ledger_path, ledger)?;
        }
        Ok(())
    };
    let outcome = write();
    if let Err(error) = outcome {
        let mut failures = restore_late_close_moves(&moved);
        if let Some(bytes) = original_ledger
            && let Err(restore) = fs::write(&ledger_path, bytes)
        {
            failures.push(format!(
                "`{SEQUENCE_PATH}` could not be restored: {restore}"
            ));
        }
        if failures.is_empty() {
            return Err(format!("{error}; nothing was closed late"));
        }
        return Err(format!(
            "{error}; the rollback did not finish, so restore these by hand: {}",
            failures.join("; ")
        ));
    }
    Ok(())
}

/// Move every package in `moved` back, newest first, then remove the record a late close wrote
/// into it. Returns one sentence per package that could not be put back (Codex review of #994).
fn restore_late_close_moves(moved: &[(PathBuf, PathBuf)]) -> Vec<String> {
    let mut failures = Vec::new();
    for (source, destination) in moved.iter().rev() {
        // The record stays with a package that cannot be moved back, so it is never stranded
        // under closed-late without its actor, reason and merge proof (Codex review of #994).
        if let Err(error) = rename_durable(destination, source) {
            failures.push(format!(
                "`{}` could not be moved back to `{}`, and keeps its late-close record: {error}",
                destination.display(),
                source.display()
            ));
            continue;
        }
        match fs::remove_file(source.join(LATE_CLOSE_RECORD)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => failures.push(format!(
                "`{}` could not be removed: {error}",
                source.join(LATE_CLOSE_RECORD).display()
            )),
        }
    }
    failures
}

/// Run `specsync doctor`: scan, then with `--fix` apply the decisions named with `--close-late`
/// and every safe fix, or with `--dry-run` report what that would do (REQ-cmd-change-035). A
/// refused late close writes nothing and is an error.
pub fn run_doctor(root: &Path, request: &DoctorRequest) -> Result<DoctorReport, String> {
    // A change named for a late close takes no evidence read. `all` is resolved from merge
    // presence alone, before the scan, so its candidates take none either (Codex review of #994).
    let select_all = request.fix && request.close_late.iter().any(|id| id == "all");
    if select_all && (find_change_dir(root, "all").is_ok() || closed_late_package(root, "all").is_some()) {
        return Err(
            "`--close-late all` is ambiguous here: a change is named `all`; name the changes to close late one by one".into(),
        );
    }
    let named: BTreeSet<String> = if select_all {
        merged_late_close_candidates(root)
    } else if request.fix {
        request.close_late.iter().cloned().collect()
    } else {
        BTreeSet::new()
    };
    let findings = doctor_scan(root, &named);
    let mut report = DoctorReport {
        findings,
        dry_run: request.dry_run,
        actions: Vec::new(),
    };
    if !request.fix {
        return Ok(report);
    }
    // The changes this call closes late, or with --dry-run would: no safe fix applies to them.
    let mut closing: BTreeSet<String> = BTreeSet::new();
    if !request.close_late.is_empty() {
        let actor = request.actor.as_deref().map(str::trim).unwrap_or_default();
        if actor.is_empty() {
            return Err("a late close needs a non-empty human actor passed with --actor".into());
        }
        let actor = normalize_actor(actor, "late close")?;
        let actor = actor.as_str();
        let reason = request.reason.as_deref().map(str::trim).unwrap_or_default();
        if reason.is_empty() {
            return Err("a late close needs a non-empty reason passed with --reason".into());
        }
        let ids: Vec<String> = if select_all {
            let mut eligible: Vec<String> = report
                .findings
                .iter()
                .filter(|finding| {
                    finding.fix.class == DoctorFixClass::Decision
                        && finding.fix.command.starts_with("specsync doctor --fix --close-late ")
                })
                .filter_map(|finding| finding.change.clone())
                .collect();
            eligible.sort();
            eligible.dedup();
            eligible
        } else {
            let mut seen = BTreeSet::new();
            request
                .close_late
                .iter()
                .filter(|id| seen.insert(id.as_str()))
                .cloned()
                .collect()
        };
        let date = today();
        let mut plans = Vec::new();
        let mut refusals = Vec::new();
        for id in &ids {
            match plan_late_close(root, id, actor, reason, &date) {
                Ok(plan) => plans.push(plan),
                // `all` means every eligible change: one that cannot be closed late is named,
                // not refused. A change named on its own is all or nothing.
                Err(error) if select_all => report.actions.push(DoctorAction {
                    kind: "close-late",
                    change: Some(id.clone()),
                    outcome: "skipped",
                    detail: error,
                }),
                Err(error) => refusals.push(error),
            }
        }
        if !refusals.is_empty() {
            return Err(format!(
                "nothing was closed late: {}",
                refusals.join("; ")
            ));
        }
        closing.extend(plans.iter().map(|plan| plan.record.change_id.clone()));
        if request.dry_run {
            for plan in &plans {
                report.actions.push(DoctorAction {
                    kind: "close-late",
                    change: Some(plan.record.change_id.clone()),
                    outcome: "would-apply",
                    detail: format!(
                        "would move `{}` to `{}`",
                        plan.record.from_path,
                        portable_project_path(root, &plan.destination)
                    ),
                });
            }
        } else {
            write_late_closes(root, &mut plans)?;
            for plan in &plans {
                let mut detail = format!(
                    "moved `{}` to `{}`; it is recorded as closed late and not verified",
                    plan.record.from_path,
                    portable_project_path(root, &plan.destination)
                );
                if let Some(collision) = &plan.record.dropped_collision_acknowledgement {
                    detail.push_str(&format!(
                        "; it was taken out of the CHG-{:04} collision acknowledgement in `{SEQUENCE_PATH}` (the acknowledgement as it stood is kept in its record), so an open change on this pull request must declare `{SEQUENCE_PATH}` where meaningful files require one",
                        collision.sequence
                    ));
                }
                report.actions.push(DoctorAction {
                    kind: "close-late",
                    change: Some(plan.record.change_id.clone()),
                    outcome: "applied",
                    detail,
                });
            }
        }
    }
    // Each safe fix is revalidated against a fresh scan after every move: a late close or an applied
    // fix can make the next one unsafe or moot (Codex review of #994). A dry run moves nothing, so
    // it plans from the first scan less the changes it would close late.
    // With late closes in this call, the plan comes from a scan that treats them as gone: they are
    // no longer blockers, and get no safe fix. A dry run moves nothing, so this is its preview.
    let mut findings_now = if closing.is_empty() {
        report.findings.clone()
    } else {
        doctor_scan(root, &closing)
    };
    let mut previewed_move = false;
    let mut attempted: BTreeSet<(&'static str, Option<String>, String)> = BTreeSet::new();
    let mut migrated = false;
    loop {
        let next = findings_now
            .iter()
            .filter(|finding| finding.fix.class == DoctorFixClass::Safe)
            .filter(|finding| {
                finding
                    .change
                    .as_ref()
                    .is_none_or(|change| !closing.contains(change))
            })
            .find(|finding| {
                !attempted.contains(&(finding.kind, finding.change.clone(), finding.fix.command.clone()))
            })
            .cloned();
        let Some(finding) = next else {
            break;
        };
        attempted.insert((finding.kind, finding.change.clone(), finding.fix.command.clone()));
        let change = finding.change.clone();
        if request.dry_run {
            // A real run rescans after every lifecycle move, which can make the next one unsafe;
            // the preview promises only the first (Codex review of #994).
            let lifecycle_move = finding.fix.command.starts_with("specsync change finalize ")
                || finding.fix.command.starts_with("specsync change archive ");
            let (outcome, detail) = if lifecycle_move && previewed_move {
                (
                    "after-revalidation",
                    format!(
                        "would run `{}` only if a fresh scan after the previous move still finds it safe",
                        finding.fix.command
                    ),
                )
            } else {
                ("would-apply", format!("would run `{}`", finding.fix.command))
            };
            previewed_move |= lifecycle_move;
            report.actions.push(DoctorAction {
                kind: finding.kind,
                change,
                outcome,
                detail,
            });
            continue;
        }
        let result = if finding.fix.command == "specsync migrate 5.0" {
            if migrated {
                continue;
            }
            migrated = true;
            backfill_reopen_digests(root, false).and_then(|backfill| {
                if backfill.failed.is_empty() {
                    Ok(format!("backfilled {}", backfill.repaired.join(", ")))
                } else {
                    Err(backfill
                        .failed
                        .iter()
                        .map(|(id, reason)| format!("{id}: {reason}"))
                        .collect::<Vec<_>>()
                        .join("; "))
                }
            })
        } else if let Some(id) = &change {
            if finding.fix.command.starts_with("specsync change finalize ") {
                finalize_change(root, id).map(|archive| {
                    format!("finalized into `{}`", portable_project_path(root, &archive))
                })
            } else {
                archive_change(root, id).map(|archive| {
                    format!("archived into `{}`", portable_project_path(root, &archive))
                })
            }
        } else {
            continue;
        };
        report.actions.push(match result {
            Ok(detail) => DoctorAction {
                kind: finding.kind,
                change,
                outcome: "applied",
                detail,
            },
            Err(error) => DoctorAction {
                kind: finding.kind,
                change,
                outcome: "failed",
                detail: format!("`{}` refused: {error}", finding.fix.command),
            },
        });
        findings_now = doctor_scan(root, &closing);
    }
    Ok(report)
}
