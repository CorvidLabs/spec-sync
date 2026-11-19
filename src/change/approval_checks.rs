// The checks of a definition approval, as one ordered list (#1091, REQ-change-163).
//
// `change approve` and `change approve --dry-run` both walk `ApprovalStep::ALL` through one
// `ApprovalRun`, so each check exists once and the two cannot drift. The approval stops at the
// first refusal and then writes. The dry run collects the first refusal of every step that can
// run, skips the steps that cannot run without a refused one, and writes nothing: no lock file,
// no recovered transaction, no ledger, no state, no cache.

/// One check `change approve` makes before it writes, in the order it makes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalStep {
    /// The relayed-decision flags come together, with `--actor`, and name that actor.
    Relay,
    /// The lifecycle lock. The approval acquires it, which also recovers an interrupted write.
    LifecycleLock,
    /// The change loads.
    Change,
    /// No other change claims this change's sequence.
    Sequence,
    /// The change's state admits a definition approval.
    State,
    /// Every change record in the project reads.
    Roster,
    /// The legacy archive baseline authority binds, on the record in memory.
    Baseline,
    /// The definition is complete.
    Definition,
    /// The semantic deltas are valid.
    Deltas,
    /// A declared module owns every declared path.
    Ownership,
    /// `verification_ignored_paths` is valid and hides none of the change's paths.
    VerificationIgnoredPaths,
    /// The approval record can be built: digest, ledger, scope, delta digests and bases.
    ApprovalRecord,
    /// The actor resolves to a name the ledger can hold.
    Actor,
}

impl ApprovalStep {
    /// Every step, in the order the approval takes them.
    const ALL: [ApprovalStep; 13] = [
        ApprovalStep::Relay,
        ApprovalStep::LifecycleLock,
        ApprovalStep::Change,
        ApprovalStep::Sequence,
        ApprovalStep::State,
        ApprovalStep::Roster,
        ApprovalStep::Baseline,
        ApprovalStep::Definition,
        ApprovalStep::Deltas,
        ApprovalStep::Ownership,
        ApprovalStep::VerificationIgnoredPaths,
        ApprovalStep::ApprovalRecord,
        ApprovalStep::Actor,
    ];

    /// The name a dry-run report gives this check. Tools read these, so they do not change.
    fn name(self) -> &'static str {
        match self {
            ApprovalStep::Relay => "relay",
            ApprovalStep::LifecycleLock => "lifecycle_lock",
            ApprovalStep::Change => "change",
            ApprovalStep::Sequence => "sequence",
            ApprovalStep::State => "state",
            ApprovalStep::Roster => "roster",
            ApprovalStep::Baseline => "baseline",
            ApprovalStep::Definition => "definition",
            ApprovalStep::Deltas => "deltas",
            ApprovalStep::Ownership => "ownership",
            ApprovalStep::VerificationIgnoredPaths => "verification_ignored_paths",
            ApprovalStep::ApprovalRecord => "approval_record",
            ApprovalStep::Actor => "actor",
        }
    }

    /// The steps this one cannot run without, in order. When one of them is refused, the dry
    /// run skips this step and names the first refused one.
    ///
    /// Three rules, and no more: nothing runs after the lock step refuses, because the tree may
    /// hold a half-written change; nothing runs for a change that did not load or whose state
    /// does not admit approval; and the approval record is not built over a delta the delta
    /// check refused, because it digests those same bodies. Every other step runs on its own.
    fn waits_on(self) -> &'static [ApprovalStep] {
        match self {
            ApprovalStep::Relay | ApprovalStep::LifecycleLock => &[],
            ApprovalStep::Change => &[ApprovalStep::LifecycleLock],
            ApprovalStep::Sequence | ApprovalStep::State => {
                &[ApprovalStep::LifecycleLock, ApprovalStep::Change]
            }
            ApprovalStep::ApprovalRecord => &[
                ApprovalStep::LifecycleLock,
                ApprovalStep::Change,
                ApprovalStep::State,
                ApprovalStep::Deltas,
            ],
            ApprovalStep::Roster
            | ApprovalStep::Baseline
            | ApprovalStep::Definition
            | ApprovalStep::Deltas
            | ApprovalStep::Ownership
            | ApprovalStep::VerificationIgnoredPaths
            | ApprovalStep::Actor => &[
                ApprovalStep::LifecycleLock,
                ApprovalStep::Change,
                ApprovalStep::State,
            ],
        }
    }
}

/// How one check of an approval dry run ended (REQ-change-163).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalCheckResult {
    /// The check ran and found nothing.
    Passed,
    /// The check ran and refused; `message` is what `change approve` says.
    Refused,
    /// The check did not run, because `skipped_because` was refused.
    Skipped,
}

/// One check of an approval dry run: its name, how it ended, and why (REQ-change-163).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApprovalCheckReport {
    /// The check's stable name, such as `definition` or `deltas`.
    pub check: &'static str,
    /// Passed, refused or skipped.
    pub result: ApprovalCheckResult,
    /// For a refused check, the first problem it found, in `change approve`'s own words.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// For a skipped check, the refused check it waits on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_because: Option<&'static str>,
}

/// What `change approve --dry-run` found: every check in approval order, and whether the
/// approval would be accepted. Nothing was written to produce it (REQ-change-163).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApprovalDryRun {
    /// The change id that was asked about.
    pub id: String,
    /// Always true: this is a dry run's report, not an approval.
    pub dry_run: bool,
    /// True only when no check refused.
    pub would_approve: bool,
    /// One entry per check, in the order the approval makes them.
    pub checks: Vec<ApprovalCheckReport>,
}

/// What the approval record is built from, read before anything is written.
enum PreparedApproval {
    Plain {
        digest: String,
        parts: Box<ApprovalRecordParts>,
    },
    Portable(PortableApprovalParts),
}

/// One approval being checked: its flags, and what the steps so far have loaded and built.
struct ApprovalRun<'a> {
    root: &'a Path,
    id: &'a str,
    actor: Option<String>,
    note: Option<String>,
    portable_v501: bool,
    /// The relayed decision as the command line gave it, or the refusal of its flags.
    relay: Result<Option<RelayedDecision>, String>,
    /// The relayed decision once the relay step has accepted it.
    relayed_by: Option<RelayedDecision>,
    record: Option<ChangeRecord>,
    prepared: Option<PreparedApproval>,
    resolved_actor: Option<String>,
}

impl<'a> ApprovalRun<'a> {
    fn new(
        root: &'a Path,
        id: &'a str,
        actor: Option<String>,
        note: Option<String>,
        portable_v501: bool,
        relay: Result<Option<RelayedDecision>, String>,
    ) -> Self {
        Self {
            root,
            id,
            actor,
            note,
            portable_v501,
            relay,
            relayed_by: None,
            record: None,
            prepared: None,
            resolved_actor: None,
        }
    }

    /// The record the change step loaded. A later step never runs without it.
    fn loaded(&self) -> Result<&ChangeRecord, String> {
        self.record
            .as_ref()
            .ok_or_else(|| "the change was not loaded before this approval check".to_string())
    }

    /// Run one step. It reads, and it may change the record in memory; it writes nothing.
    ///
    /// The lock step is the one exception in kind: here it only predicts whether the lock could
    /// be taken. The approval itself acquires the lock in that step instead of calling this.
    fn check(&mut self, step: ApprovalStep) -> Result<(), String> {
        match step {
            ApprovalStep::Relay => {
                // The recorded actor is trimmed (#426), so the relay's decider is trimmed the
                // same way: otherwise `actor` and `relayed_by.decided_by` differ by padding and
                // every later read of the ledger refuses the mismatch (#1026 review).
                let relayed_by =
                    std::mem::replace(&mut self.relay, Ok(None))?.map(|mut relay| {
                        relay.decided_by = relay.decided_by.trim().to_string();
                        relay
                    });
                if relayed_by.is_some() {
                    let Some(decider) = self.actor.as_deref() else {
                        return Err(
                            "a relayed decision names who decided with --actor <who decided>; without it the \
                             actor comes from Git or $USER, which names the recording agent"
                                .into(),
                        );
                    };
                    ensure_relay_names_the_decider(relayed_by.as_ref(), decider.trim(), "--actor")?;
                }
                self.relayed_by = relayed_by;
                Ok(())
            }
            ApprovalStep::LifecycleLock => lifecycle_lock_could_be_taken(self.root),
            ApprovalStep::Change => {
                self.record = Some(load_change(self.root, self.id)?);
                Ok(())
            }
            ApprovalStep::Sequence => ensure_no_sequence_collision(self.root, self.loaded()?),
            ApprovalStep::State => {
                let record = self.loaded()?;
                // Accepted records may re-approve the definition when the artifact digest
                // drifted (reopen recovery). Refuse no-op re-approvals that would only rewrite an
                // already-valid definition approval while the change is accepted.
                let allow_accepted_definition_refresh = record.state == ChangeState::Accepted
                    && ensure_definition_approval_valid(self.root, record).is_err();
                if allow_accepted_definition_refresh {
                    return Ok(());
                }
                require_state(
                    record,
                    &[
                        ChangeState::Draft,
                        ChangeState::Approved,
                        ChangeState::Implementing,
                        ChangeState::Verifying,
                    ],
                    "approve the definition",
                )
            }
            ApprovalStep::Roster => list_changes_checked(self.root).map(|_| ()),
            ApprovalStep::Baseline => {
                let root = self.root;
                let record = self.record.as_mut().ok_or_else(|| {
                    "the change was not loaded before this approval check".to_string()
                })?;
                bind_legacy_archive_baseline_authority(root, record)
            }
            ApprovalStep::Definition => validate_definition(self.root, self.loaded()?),
            ApprovalStep::Deltas => validate_delta_files(self.root, self.loaded()?),
            ApprovalStep::Ownership => {
                validate_declared_path_ownership(self.root, self.loaded()?)
            }
            ApprovalStep::VerificationIgnoredPaths => {
                ensure_change_clear_of_verification_ignored(self.root, self.loaded()?).map(|_| ())
            }
            ApprovalStep::ApprovalRecord => {
                let record = self.loaded()?;
                let prepared = if self.portable_v501 {
                    PreparedApproval::Portable(portable_definition_approval_parts(
                        self.root, record,
                    )?)
                } else {
                    let digest = definition_digest(self.root, record)?;
                    PreparedApproval::Plain {
                        digest,
                        parts: Box::new(approval_record_parts(self.root, record, "definition")?),
                    }
                };
                self.prepared = Some(prepared);
                Ok(())
            }
            ApprovalStep::Actor => {
                self.resolved_actor = Some(resolve_actor(self.root, self.actor.clone())?);
                Ok(())
            }
        }
    }

    /// Raise a workflow-v2 record to the relay-gated version when the approval carries a
    /// relayed decision (REQ-change-158). In memory only; returns whether the record changed.
    fn raise_for_relay(&mut self) -> bool {
        match (&self.relayed_by, self.record.as_mut()) {
            (Some(_), Some(record)) => raise_for_relayed_decision(record),
            _ => false,
        }
    }

    /// Write the approval every step has prepared: the ledger, then `state.json` and
    /// `change.md`. Only `change approve` calls this, after every step passed.
    fn write(mut self) -> Result<ChangeRecord, String> {
        const UNPREPARED: &str = "the approval was not fully checked before it was written";
        let mut record = self.record.take().ok_or(UNPREPARED)?;
        let actor = self.resolved_actor.take().ok_or(UNPREPARED)?;
        let prior_state = record.state;
        let relayed_on_workflow_v1 = self.relayed_by.is_some() && record.workflow_version < 2;
        match self.prepared.take().ok_or(UNPREPARED)? {
            PreparedApproval::Portable(parts) => write_portable_definition_approval(
                self.root,
                &record,
                parts,
                actor,
                self.note.take(),
                self.relayed_by.take(),
            )?,
            PreparedApproval::Plain { digest, parts } => write_approval_record(
                self.root,
                &record,
                "definition",
                *parts,
                actor,
                digest,
                self.note.take(),
                self.relayed_by.take(),
            )?,
        }
        record.state = match prior_state {
            ChangeState::Draft | ChangeState::Approved => ChangeState::Approved,
            ChangeState::Verifying if record.canonical_applied => ChangeState::Verifying,
            ChangeState::Implementing | ChangeState::Verifying => ChangeState::Implementing,
            ChangeState::Accepted | ChangeState::Archived => prior_state,
            ChangeState::Retired => return Err("retirement is immutable nonacceptance".into()),
        };
        record.updated_at = now();
        save_change(self.root, &record)?;
        write_change_markdown(self.root, &record)?;
        if relayed_on_workflow_v1 {
            eprintln!("{}", ungated_relay_note(&record.id));
        }
        Ok(record)
    }
}

/// Whether the lifecycle lock could be taken, without taking it (REQ-change-163).
///
/// Acquiring the lock creates `.specsync/change.lock` and recovers an interrupted lifecycle
/// write, and a dry run writes nothing. So it asks what the lock asks, read-only. The lock's own
/// no-symlink walk on the lock path predicts that refusal. A journal recovery cannot replay,
/// because it does not read or names a path outside the project, is a refusal the real approval
/// gives too, so it is reported in recovery's own words. A journal that reads is the one case
/// with no counterpart: a real approval recovers it first, and what the tree holds after that is
/// not known here, so the dry run says a write is waiting and checks nothing else.
fn lifecycle_lock_could_be_taken(root: &Path) -> Result<(), String> {
    reject_symlink_components_for(root, &root.join(LOCK_PATH), "lifecycle lock path")?;
    let Some(entries) = read_pending_transaction(root)? else {
        return Ok(());
    };
    for entry in &entries {
        pending_transaction_entry_path(root, entry)?;
    }
    Err(format!(
        "an interrupted lifecycle write is waiting to be recovered (`{TRANSACTION_PATH}`); the next          lifecycle command that writes recovers it before it checks anything, and a dry run writes          nothing, so it cannot say what `change approve` would find. Run the dry run again once that          write is recovered"
    ))
}

/// Record a definition approval, or the portable 5.0.1 pair, carrying an optional relayed
/// decision (#633). The relay must name the approval's explicit actor as the one who decided,
/// and is refused before anything is written otherwise.
///
/// It walks `ApprovalStep::ALL`, the list `approve_definition_dry_run` walks too, and returns
/// the first refusal. Then it writes.
pub fn approve_definition_with_relay(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
    portable_v501: bool,
    relayed_by: Option<RelayedDecision>,
) -> Result<ChangeRecord, String> {
    let mut run = ApprovalRun::new(root, id, actor, note, portable_v501, Ok(relayed_by));
    let mut _lock = None;
    for step in ApprovalStep::ALL {
        match step {
            // The approval takes the lock here, which also recovers an interrupted write.
            ApprovalStep::LifecycleLock => _lock = Some(acquire_project_lock(root)?),
            ApprovalStep::ApprovalRecord => {
                // A workflow-v2 change is raised to the relay-gated version, and `state.json`
                // written, before the relay is, so an interrupted approve never leaves a relay
                // that SpecSync 6.2.0 could drop (REQ-change-158). A workflow-v1 change keeps
                // its version and says so once the approval is written.
                if run.raise_for_relay() {
                    save_change(root, run.loaded()?)?;
                }
                run.check(step)?;
            }
            _ => run.check(step)?,
        }
    }
    run.write()
}

/// Say whether `change approve` would accept the change as it stands, and write nothing
/// (REQ-change-163).
///
/// `relay` is the relayed decision as the command line gave it, or the refusal of its flags, so
/// the relay check reports that refusal too. Every step that can run does, and reports its first
/// problem in the words `change approve` uses; a step that waits on a refused one is skipped and
/// names it. The legacy-baseline binding and a relayed decision's workflow-version raise happen
/// on the record in memory and are dropped with it.
pub fn approve_definition_dry_run(
    root: &Path,
    id: &str,
    actor: Option<String>,
    note: Option<String>,
    portable_v501: bool,
    relay: Result<Option<RelayedDecision>, String>,
) -> ApprovalDryRun {
    let mut run = ApprovalRun::new(root, id, actor, note, portable_v501, relay);
    let mut refused: Vec<ApprovalStep> = Vec::new();
    let mut checks = Vec::with_capacity(ApprovalStep::ALL.len());
    for step in ApprovalStep::ALL {
        if let Some(waits_on) = step
            .waits_on()
            .iter()
            .find(|needed| refused.contains(needed))
        {
            checks.push(ApprovalCheckReport {
                check: step.name(),
                result: ApprovalCheckResult::Skipped,
                message: None,
                skipped_because: Some(waits_on.name()),
            });
            continue;
        }
        if step == ApprovalStep::ApprovalRecord {
            // The approval saves this raise before it builds the record. Here it stays in memory.
            run.raise_for_relay();
        }
        match run.check(step) {
            Ok(()) => checks.push(ApprovalCheckReport {
                check: step.name(),
                result: ApprovalCheckResult::Passed,
                message: None,
                skipped_because: None,
            }),
            Err(message) => {
                refused.push(step);
                checks.push(ApprovalCheckReport {
                    check: step.name(),
                    result: ApprovalCheckResult::Refused,
                    message: Some(message),
                    skipped_because: None,
                });
            }
        }
    }
    ApprovalDryRun {
        id: id.to_string(),
        dry_run: true,
        would_approve: refused.is_empty(),
        checks,
    }
}
