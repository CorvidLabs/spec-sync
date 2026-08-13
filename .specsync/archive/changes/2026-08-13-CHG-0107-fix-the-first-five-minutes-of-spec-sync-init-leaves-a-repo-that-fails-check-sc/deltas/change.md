## ADDED

### REQUIREMENT REQ-change-060

A bootstrap record SHALL exempt a path from lifecycle path coverage only when that exemption cannot
be used to hide product delivery or later policy edits.

Acceptance Criteria

- A recorded path is honored only when it is a protected SDD path, is absent at the delivery
  comparison base, its recorded base commit is an ancestor of `HEAD`, and its content still matches
  the recorded digest.
- A bootstrap record never exempts a path that is not a protected SDD path.
- Editing a bootstrapped file revokes its own exemption and the normal change workflow applies from
  that point on.
- Bootstrap records written in the earlier single-path shape continue to be honored.

### REQUIREMENT REQ-change-061

The digest recorded for a bootstrapped policy SHALL pin the enforcement surface rather than the
file's bytes.

Acceptance Criteria

- Every field that determines whether the coverage gate applies is covered by the digest.
- Verification commands are excluded, so populating them as initialization instructs does not revoke
  the bootstrap.
- A policy file that cannot be parsed falls back to a digest of its bytes.

### REQUIREMENT REQ-change-062

Resolution of the delivery comparison base SHALL succeed in a repository containing a single commit.

Acceptance Criteria

- Both a range form and a bare commit reduce to a single commit through its merge base with `HEAD`.
- No resolution path depends on a parent commit existing.

### REQUIREMENT REQ-change-063

An unfinished spec section SHALL gate on whether a change authored it, not on its content shape.

Acceptance Criteria

- A generated section no active change authored produces no fatal effective-contract finding.
- A section an active change authored and then emptied remains fatal, through both the pending and
  the applied delta paths.
- Unknown authorship fails closed and exempts nothing.
- Ignore configuration is applied through the project's ignore rules rather than re-derived.
- Suppressions are reported as warnings rather than dropped silently.
## MODIFIED

### SPEC SECTION Public API

**Exported Constants**

| Name | Description |
|------|-------------|
| `SDD_VERSION` | Current SDD project-layout version written by initialization |

**Exported Types**

| Type | Description |
|------|-------------|
| `ChangeState` | Six-state delivery lifecycle: draft, approved, implementing, verifying, accepted, archived |
| `ChangeKind` | Deterministic policy classification for feature, bug fix, refactor, migration, documentation, and operations work |
| `ArtifactKind` | Built-in or custom adaptive companion artifact selection |
| `SddPolicy` | Versioned enforcement, path, verification-command, template, and principles configuration |
| `SuccessionObligation` | Definition-bound predecessor path, canonical owner module, and full predecessor entry digest |
| `SupersedesEdge` | Durable predecessor ID and its sorted semantic succession obligations |
| `AcceptanceOwnerCorrection` | Sequenced human-authored exact path/module ownership correction for acceptance evidence |
| `ChangeRecord` | Durable machine state for one change workspace, including an explicit legacy-or-single-workflow version and omitted-when-empty supersedes/correction evidence |
| `LegacyArchiveBaselineV1` | Definition- and closing-bound authority, cutoff, and sorted legacy archive subtree entries |
| `LegacyArchiveBaselineEntryV1` | Archive ID, canonical dated path, unique introduction commit, and exact subtree digest |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `ApprovalRecord` | Actor, timestamp, gate, digest, optional note, and optional backward-readable portable-pair metadata for one approval |
| `ApprovedScopeV1` | Canonical stable intent, acceptance contract, risk declarations, and affected scope bound by one human approval |
| `NonMaterialScopeChangeCategory` | Closed implementation, test/evidence, canonical-materialization, and lifecycle-metadata classification set |
| `NonMaterialScopeChangeV1` | Path and concise evidence-backed classification for one approval-preserving migration change |
| `ScopeApprovalMigrationV1` | Historical embedded migration shape retained only to authenticate the allowlisted CHG-0068 anchor blob; it is not accepted as a general live projection bridge |
| `ScopeAdoptionSourcePreimageStatus` | Explicit declaration that the one allowlisted legacy approval preimage is unavailable |
| `ScopeAdoptionEquivalenceClaim` | Explicit declaration that the allowlisted adoption makes no cryptographic equivalence claim |
| `ScopeAdoptionAnchorV1` | Exact historical commit, approval index, and approval-ledger blob digest |
| `ScopeAdoptionAuthorizationV1` | Actor, recording time, and truthful reason for the one allowlisted adoption exception |
| `ScopeAdoptionV1` | Frozen adopted stable scope, anchor, authorization, and non-material classification evidence |
| `DefinitionApprovalPairRole` | Current/full or legacy/projected role for one marked portable definition member |
| `DefinitionApprovalPairV1` | Versioned pair identity, projection, role, change/correction coordinates, event index, and both digests |
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, and stale/current input digests |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `ReopenBackfillReport` | Per-change repair, skip, and failure detail for a `migrate 5.0` ledger backfill |
| `CorrectionField` | Closed supported accepted-metadata field set: public contract and architecture risk |
| `CorrectionRecord` | Immutable sequenced metadata correction with original/effective values, actor, reason, artifacts, prior evidence, and portable digest chain |
| `EffectiveChangeDefinition` | Validated projection of original answers/artifacts plus ordered corrections |
| `CorrectionResult` | Deterministic corrected change, event, effective definition, history, and gate-summary projection |
| `DefinitionMutationResult` | Crate-private successful definition mutation plus the effective definition, correction history, and normal/strict summaries validated inside its persistence transaction |
| `ApprovalLedger` | Ordered portable approval, allowlisted scope-adoption, and reopen history |
| `CommandEvidence` | Exit evidence for one configured verification command |
| `AcceptanceInputKind` | Canonical file, symlink, gitlink, missing, or non-file topology kind |
| `AcceptanceInputEntryV1` | Bounded path, kind, mode, payload digest, full-entry digest, and sorted owners for one accepted input |
| `AcceptanceManifestV1` | Versioned sorted per-input acceptance manifest |
| `SemanticSuccessionTupleV1` | Exact predecessor, path, module, old-entry digest, and new-entry digest transition |
| `SemanticSuccessionEvidenceV1` | Versioned sorted one-to-one closing evidence for approved supersedes obligations |
| `VerificationRecord` | Commit-bound verification result with separate stable-scope and volatile-execution digests, commands, requirement coverage, and optional acceptance manifest/succession evidence |
| `ScopedReviewVerdict` | Explicit passing or blocking conclusion for one independent scoped review |
| `ScopedReviewProvenanceProvider` | External provider class that authenticates the required scoped-review result |
| `ScopedReviewProvenanceV1` | Versioned required GitHub Actions check binding carried by review evidence |
| `ScopedReviewRecord` | Stable reviewer claim, required-check provenance, explicit verdict, implementation commit, scope/execution/workspace digests, and review timestamp bound before finalization |
| `FinalizationRecord` | Automated non-approval evidence binding implementation commit/tree, contract/workspace/closing/review digests, archive identity, and a domain-separated finalization digest |
| `ChangeReadScope` | Crate-private invocation guard that owns one bounded read-only lifecycle snapshot |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `TerminalEvidenceValidity` | State-aware exact, successor-covered, stale, authenticated-history, or corrupt-history evidence conclusion |
| `TerminalEvidenceSummary` | Shared terminal validity plus optional fail-closed reason |
| `TerminalEvidenceResult` | Change ID paired with its shared terminal-evidence summary |
| `ChangeSummary` | Human/agent status projection with approval health/current scope digest, plain-language material expansion, validator plan, scoped-review freshness, exactly one next action, and optional terminal evidence |
| `SddCheckReport` | Unified lifecycle errors, warnings, checked-change count, and terminal-evidence results |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `record_bootstrap_paths` | `root: &Path` | `Result<(), String>` | Record the protected SDD paths this bootstrap created in `.specsync/bootstrap.json`, so initialization's own output is not reported as uncovered meaningful delivery; editing a recorded file revokes its exemption |
| `write_default_policy` | `root: &Path, verification_commands: Vec<String>` | `Result<(), String>` | Write new-project/adoption policy without overwriting existing policy |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `acceptance_entries` | `root: &Path, record: &ChangeRecord` | `Vec<AcceptanceInputEntryV1>` | Accepted acceptance-input entries, so `change show --json` can surface the `specsync.acceptance-entry.v1` digests `change supersede --digest` requires; empty when evidence is absent |
| `list_changes` | `root: &Path` | `Vec<ChangeRecord>` | List active changes in stable ID order |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Return deterministic unanswered interview questions |
| `answer_question` | `root, id, question, answer` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then persists an interview answer and updates adaptive artifacts |
| `answer_question_with_snapshot` | `root, id, question, answer` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the answer mutation with its full in-transaction machine snapshot |
| `add_dependency` | `root, id, dependency` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, declares ordering between active changes, and invalidates stale approval digests |
| `add_dependency_with_snapshot` | `root, id, dependency` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the dependency mutation with its full in-transaction machine snapshot |
| `add_supersedes_obligation` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then adds one definition-bound semantic succession obligation to a draft |
| `add_supersedes_obligation_with_snapshot` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the supersession mutation with its full in-transaction machine snapshot |
| `add_acceptance_owner_correction` | `root, id, path, module, actor, reason` | `Result<ChangeRecord, String>` | Append one audited exact canonical owner correction to a reopened already-applied change |
| `add_acceptance_owner_corrections` | `root, id, entries, actor, reason` | `Result<ChangeRecord, String>` | Validate every exact path/module owner correction, then append all as sequenced audit entries in one transactional write |
| `add_missing_acceptance_owner_corrections` | `root, id, module, actor, reason` | `Result<ChangeRecord, String>` | Discover production-source affected paths lacking canonical ownership for a module and append them as one transactional batch |
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record an ordinary mandatory definition approval |
| `approve_definition_portable_v501` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Atomically record the marked current/5.0.1 portable definition pair |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract evidence |
| `verify_change_with_strict` | `root, id, strict` | `Result<VerificationRecord, String>` | Run targeted validators plus additive strict policy/classification validators on the same evidence path |
| `check_change` | `root, optional id` | `Result<Option<VerificationRecord>, String>` | Select one approved/implementing change, materialize its canonical deltas, and verify it |
| `check_change_with_strict` | `root, optional id, strict` | `Result<Option<VerificationRecord>, String>` | Run `check_change` with additive strict validators |
| `record_scoped_review` | `root, id, reviewer` | `Result<ScopedReviewRecord, String>` | Record one independent implementation-scoped review bound to current governed inputs |
| `record_scoped_review_with_verdict` | `root, id, reviewer, verdict` | `Result<ScopedReviewRecord, String>` | Record an explicit passing or blocking independent review; only a current passing verdict permits finalization |
| `finalize_change` | `root, id` | `Result<PathBuf, String>` | Validate current verification/review evidence and transactionally produce the dated same-PR archive |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas only when not already canonical |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `backfill_reopen_digests` | `root: &Path, dry_run: bool` | `Result<ReopenBackfillReport, String>` | Backfill 5.1 reopening digest fields on 5.0.1-era ledgers with verified, idempotent, dry-run-aware writes |
| `begin_change_read_scope` | `root: &Path` | `ChangeReadScope` | Install one invocation-scoped read snapshot for list/show/status and project reports |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, correction health, and next action using the shared verification-freshness predicate |
| `summarize_change_with_strict` | `root, record, explicit_strict` | `ChangeSummary` | Project the same status plus exact targeted/additive-strict validator commands |
| `artifacts_complete_for_guidance` | `root, record` | `bool` | Lightweight selected-artifact completeness for human next-action guidance without digest loaders |
| `check_project` | `root: &Path` | `SddCheckReport` | Full lifecycle integrity including archive terminal evidence (tests and rare callers; not the default CLI path) |
| `audit_project` | `root: &Path` | `SddCheckReport` | Active workspaces + living policy/spec coherence only — does not rewalk archived terminal evidence |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or atomically enable SDD, activate workflow v2 without stranding cutoff-ineligible legacy records or rewriting legacy policy, and import OpenSpec or Spec Kit artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |

**Exported Methods**

| Method | Description |
|--------|-------------|
| `as_str` | Return the stable serialized name for a change state, kind, or correction field |
| `parse` | Parse user-facing change-kind, artifact, or supported correction-field names into typed values |
| `file_name` | Resolve an adaptive artifact to its safe Markdown filename |
| `is_clean` | Return true when a ledger backfill recorded no per-change failures |

Acceptance Criteria

- Nested lifecycle commands still fail once with the established deterministic contextual error.
- The process marker and diagnostic helper remain private binary implementation details.
- Correction inspection exposes typed portable records without exposing mutable ledger internals.
- Acceptance-owner corrections expose only immutable audit fields and never mutable internal ledgers.

