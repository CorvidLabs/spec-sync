---
module: change
version: 234
status: active
files:
  - src/change.rs
  - src/change/collect_requirement_ids.rs
  - src/change/consider_accepted_evidence_anchor.rs
  - src/change/retirement.rs
  - src/change/doctor.rs
  - src/change/historical_definition_digest_matches.rs
  - src/change/historical_reconciliation.rs
  - src/change/read_cache.rs
  - src/change/read_deadline.rs
  - src/change/relayed_decision.rs
  - src/change/sdd_version.rs
  - src/change/send_back.rs
  - src/change/unknown_members.rs
  - src/change/validate_loaded_change.rs
  - src/change/verification_ignored_paths.rs
  - src/change_tests.rs
  - src/change_tests/approve_ownership_names_the_spec.rs
  - src/change_tests/approve_refusal_hints.rs
  - src/change_tests/approve_rejects_a_declared_path_owned_by_an_undeclared_module.rs
  - src/change_tests/approved_delta_requirements.rs
  - src/change_tests/archive_post_move_failure_restores_exact_source_bytes_without_residue.rs
  - src/change_tests/bounded_failure_output.rs
  - src/change_tests/canonical_resolution_tolerates_inert_legacy_registry_stub.rs
  - src/change_tests/commit_identity_guards.rs
  - src/change_tests/doctor_collision.rs
  - src/change_tests/fast_archived_reads.rs
  - src/change_tests/fast_finalize.rs
  - src/change_tests/historical_reconciliation.rs
  - src/change_tests/open_task_gate.rs
  - src/change_tests/recorded_review_recovery.rs
  - src/change_tests/recovery_retirement.rs
  - src/change_tests/relayed_decision.rs
  - src/change_tests/semantic_application_resolves_registry_backed_canonical_paths.rs
  - src/change_tests/send_back_for_rescope.rs
  - src/change_tests/stale_modified_delta.rs
  - src/change_tests/transaction_journal_error.rs
  - src/change_tests/titled_requirement_header_delta.rs
  - src/change_tests/inert_ignored_paths.rs
  - src/change_tests/substantive_plan.rs
  - src/change_tests/unknown_members.rs
  - src/change_tests/upstream_archive.rs
  - src/change_tests/verification_ignored_paths.rs
  - src/lifecycle-validation-limits.json
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
---

# Change

## Purpose

Provides the SpecSync verified spec-driven development lifecycle: one scope approval, targeted verification, one scoped human review, same-PR finalization, and compatible audited recovery for historical evidence.

## Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. `change check` compares THIS change's specs to source in-process and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command. Scope is the modules in `affected_specs` union the specs whose `files:` fall inside `affected_paths`, so a `--no-spec-change` delivery still verifies against the contracts its source can break; drift outside that scope belongs to `specsync check`. A declared module that resolves to no spec file on disk FAILS verification and is named — never silently dropped, even when other specs are in path scope and would make the pass look real. An empty scope is a PASS only for a change that declared no module and maps no spec. Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`, where each name is what `filter_specs` matches (the file stem with `.spec` removed) and `--strict` appears only when it was requested. The effective-contract gate judges the change being verified on every check, the first one after materialization included, and a refusal of a module the change declares is recorded as a failed attempt that also names `specsync change check <id> (effective contract: <module>, …)`.
6. Verification and scoped-review evidence bind the implementation commit and governed inputs (every project input except files under a `verification_ignored_paths` prefix, item 24); a scoped review records an explicit pass/block verdict, may be recorded by the same actor as the definition approver, and stays fresh only when every descendant/parent edge changes supported lifecycle persistence. GitHub remains the merge authority for required reviewers.
7. Invalid policy, unavailable coverage comparison, failed evidence, stale ordering gates, and protected sequence-ledger edits without lifecycle coverage fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, refuses `## ADDED` for requirement IDs already present in the living tree (agents must use `## MODIFIED`), corrupt state fails closed, and transactional same-PR finalization remains retryable before or after the archive-directory move.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
11. Concurrent effective-contract validations use isolated temporary workspaces.
12. Stale accepted delivery evidence can return only to verifying through an explicit human actor and reason, while prior verification and closing evidence remain inspectable.
13. Historical collision acknowledgements are exact immutable accepted-or-archived evidence and numeric sequence width has no four-digit upper bound.
14. A fully valid later sequence claim supersedes only the sequence-ledger bytes in historical acceptance inputs; the current owner and every other covered input remain exact evidence.
15. Supported accepted interview metadata changes only through a portable append-only correction ledger whose effective definition requires fresh gates and never replays canonical deltas.
16. Audited exact acceptance-owner corrections can repair omitted canonical ownership on an already-scoped input without changing semantic scope or replaying canonical deltas.
17. A transactional batch of audited exact acceptance-owner corrections validates every entry independently and persists all or none as sequenced ledger entries.
18. Bounded Git candidate inspection deduplicates repeated stage-zero paths only when their normalized mode and object identity match exactly; conflicting observations fail closed.
19. Only projects outside a Git repository may persist verification with no commit identity; an unborn Git repository with no `HEAD` still fails closed.
20. Workflow-v2 adoption atomically freezes a comparison-base cutoff that precedes its unique introduction, opens its lifecycle lock without following symlinks, journals only lossless UTF-8 publication paths whose filename components cannot be confused with platform separators, confines them beneath the project without symlink traversal, leaves an existing enabled version-1 policy byte-identical (adoption rewrites only `enabled`, and only when it is off), refuses to strand v1 records absent from that cutoff, routes every subsequent change through workflow v2, and fails closed if any reachable parent introduced a subsequently absent baseline.
21. Existing-change definition mutations validate correction-ledger integrity while holding the same project lock that guards persistence and return the validated effective-definition snapshot used by command output.
22. Every change carries a handoff readiness — `safe`, `conditional`, or `not-yet` — computed as a pure function of its lifecycle signals: a project-wide sequence freeze, open interview questions, artifact completeness, definition-approval currency, correction-ledger validity, uncommitted edits under `affected_paths` (never `.specsync/` evidence, which the review → finalize pair leaves uncommitted by design), verification currency, scoped-review currency, and terminal-evidence staleness. A Draft is never `safe` because approval is the first boundary a fresh session can resume from; the summary names one plain-language reason without digests, the resume command `specsync change status <id>`, and the steps to take before clearing when readiness is not `safe`.
23. The workspace digest names each clean tracked file by its Git object id (`specsync.project-input-digest.v4`), in an `object-id` frame that file bytes cannot match. It reads file bytes only for dirty or untracked paths, and those reads stay inside the existing payload bound. `filter=lfs` is that object id only when the configured clean and smudge commands are Git LFS and the blob is an LFS pointer. Every other content filter still fails closed. A change's own acceptance evidence still hashes the bytes of the paths it covers.
24. `.specsync/sdd.json` `verification_ignored_paths` is an opt-in list, empty by default, of at most 8 portable project-relative directory prefixes ending in `/`. Files under them, tracked or untracked, are left out of the project-input digest and nowhere else: the scoped-review descendant walk still stales a review for any later commit, including one under a listed prefix. A non-empty list adds one length-framed `verification-ignored` frame, a tag no path or file-bytes frame can produce, to the same v4 domain. An entry is refused by every lifecycle verb that loads SDD policy when it is a pattern, absolute, symlinked, oversized, duplicated or nested, or when it overlaps `.specsync/`, the specs tree, `source_dirs`, the configured `schema_dir`, a `meaningful_paths` scope, spec `files:`, path-form spec `depends_on` entries, files named in a spec's `Consumed By` table, SDD policy, principles or custom-artifact files; `ignored_paths` never relaxes this. A change whose `affected_paths` overlap an entry is refused. No persisted record gains a field, and with the list absent or empty the digest is byte-identical.

## Public API

**Exported Constants**

| Name | Description |
|------|-------------|
| `RECORDED_CLAIM_BOUNDARY` | The words `change status` prints beside an approval's actor and a review's reviewer: a recorded claim, not authenticated |
| `RELAYED_CLAIM_BOUNDARY` | The boundary sentence printed with every relayed decision, shared word for word with Chough |
| `LESSON_BUNDLE_FILE` | Filename of the lesson bundle written into an archive, shared with the command layer so the two cannot name different files |
| `READ_TIMEOUT_EXIT_CODE` | Exit status (124) of a read-only lifecycle command stopped by its `--timeout` deadline (REQ-change-137) |
| `SDD_VERSION` | Current SDD project-layout version written by initialization |

**Exported Types**

| Type | Description |
|------|-------------|
| `ReadDeadline` | When a read must stop and the limit it was given; copied to worker threads |
| `ReadDeadlineGuard` | Clears this thread's read deadline when dropped (REQ-change-137) |
| `ChangeState` | Delivery lifecycle including retired terminal nonacceptance; retired grants no dependency or succession credit |
| `ChangeKind` | Deterministic policy classification for feature, bug fix, refactor, migration, documentation, and operations work |
| `ArtifactKind` | Built-in or custom adaptive companion artifact selection |
| `UpstreamArchive` | What the remote default says about an archive of a change this branch holds unarchived: `Archived` with the reference and the commit on it that archived the change (#694), or `CannotTell` with the reference and selected project root when its archived state records could not be read (REQ-change-142, #946) |
| `SddPolicy` | Versioned enforcement, path, verification-command, template, and principles configuration, plus the opt-in `verification_ignored_paths` directory prefixes left out of the verification project-input digest (empty by default and omitted when empty) |
| `SuccessionObligation` | Definition-bound predecessor path, canonical owner module, and full predecessor entry digest |
| `SupersedesEdge` | Durable predecessor ID and its sorted semantic succession obligations |
| `AcceptanceOwnerCorrection` | Sequenced human-authored exact path/module ownership correction for acceptance evidence |
| `ChangeRecord` | Durable machine state for one change workspace, including an explicit legacy-or-single-workflow version and omitted-when-empty supersedes/correction evidence |
| `LegacyArchiveBaselineV1` | Definition- and closing-bound authority, cutoff, and sorted legacy archive subtree entries |
| `LegacyArchiveBaselineEntryV1` | Archive ID, canonical dated path, unique introduction commit, and exact subtree digest |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `CheckInvocation` | The `--strict`, `--commit` and `--push` flags of one `change check`, so a refusal names that exact command to re-run |
| `ApprovalRecord` | Actor claim, timestamp, gate, digest, optional note, optional backward-readable portable-pair metadata, the optional per-module semantic delta body digests a definition gate approved, and the optional `relayed_by` decision an agent relayed; the digest binds content, never the actor |
| `RelayedDecision` | Shared relayed-decision claim: `decided_by`, verbatim `words`, `where` (including any relay chain) and `recorded_by`; all four required, non-blank and at most 4096 bytes, with `null` and unknown keys refused on read; covered by no digest |
| `RecordedClaim` | Status projection of the effective definition approval or the latest scoped review: gate, named actor or reviewer, review verdict, optional relayed decision; carries no digest |
| `RelayRepair` | One relayed decision `repair_relayed_decisions` put back or removed: `file`, the record's 1-based position in it, `action` (`put_back` or `removed`), the copy it came `from` for a put-back, and the damaged value it `removed` for a removal |
| `RelayRepairReport` | What `repair_relayed_decisions` did to one change: the change id and its repairs; empty repairs means nothing was damaged and nothing was written |
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
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, stale/current input digests, the staleness cause when the digests are equal, and an optional exact remote-default Git anchor when reopening a recorded review-workspace mismatch |
| `ReopenCauseV1` | Why accepted evidence was stale despite matching inputs: unanchored verification or unreconstructible legacy acceptance; absent means inputs drifted |
| `recorded_verification_is_current` | Whether a change's recorded verification still matches the tree and plan on disk, as a content question; missing or unreadable evidence is not current |
| `recorded_verification_commit_contains_change_work` | Whether the recorded verification commit contains the change's current work — the contents of its `affected_paths` and any canonical specs it materialized; uncommitted `.specsync/` evidence alone is not a failure; a missing commit is not compared (#535, REQ-change-157) |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `AppliedDeltaItem` | One block a change's delta put into a canonical spec or requirements file: module, target, key and body, listed by a send-back |
| `SendBackRecord` | Immutable audit event for one send-back to draft: actor, reason, timestamp, prior and new state, the number of approvals before it, the number of scoped review attempts before it (absent on a send-back recorded before #960), whether canonical text had been applied, and the applied items |
| `SendBackResult` | Deterministic change-plus-audit result returned by the send-back transition |
| `ReopenBackfillReport` | Per-change repair, skip, and failure detail for a `migrate 5.0` ledger backfill |
| `CorrectionField` | Closed supported accepted-metadata field set: public contract and architecture risk |
| `CorrectionRecord` | Immutable sequenced metadata correction with original/effective values, actor, reason, artifacts, prior evidence, and portable digest chain |
| `EffectiveChangeDefinition` | Validated projection of original answers/artifacts plus ordered corrections |
| `CorrectionResult` | Deterministic corrected change, event, effective definition, history, and gate-summary projection |
| `DefinitionMutationResult` | Crate-private successful definition mutation plus the effective definition, correction history, normal/strict summaries and approved delta summaries built inside its persistence transaction |
| `ApprovalLedger` | Ordered portable approval, allowlisted scope-adoption, and reopen and send-back history |
| `CommandEvidence` | Evidence for one verification step (in-process spec↔code sync); a failed step carries a bounded `output` with the findings that failed it, omitted on success (#640) |
| `AcceptanceInputKind` | Canonical file, symlink, gitlink, missing, or non-file topology kind |
| `AcceptanceInputEntryV1` | Bounded path, kind, mode, payload digest, full-entry digest, and sorted owners for one accepted input |
| `AcceptanceManifestV1` | Versioned sorted per-input acceptance manifest |
| `SemanticSuccessionTupleV1` | Exact predecessor, path, module, old-entry digest, and new-entry digest transition |
| `SemanticSuccessionEvidenceV1` | Versioned sorted one-to-one closing evidence for approved supersedes obligations |
| `VerificationRecord` | Commit-bound verification result with separate stable-scope and volatile-execution digests, commands, requirement coverage, and optional acceptance manifest/succession evidence |
| `ScopedReviewVerdict` | Explicit passing or blocking conclusion for one scoped human review |
| `ScopedReviewProvenanceProvider` | Stored provenance-provider declaration; identity authentication requires separately enforced external policy |
| `ScopedReviewProvenanceV1` | Versioned required GitHub Actions check binding carried by review evidence |
| `ScopedReviewRecord` | Stable reviewer claim, required-check provenance, explicit verdict, implementation commit, scope/execution/workspace digests, review timestamp bound before finalization, and the optional `relayed_by` decision, which the finalization review digest leaves out |
| `ScopedReviewCurrency` | Three-valued answer to whether a recorded scoped review still holds: current, stale carrying what moved, or unavailable when the guarantee could not be evaluated at all |
| `reason` | Why a scoped review is not current, for the callers that render a blocker or a warning |
| `FinalizationRecord` | Automated non-approval evidence binding implementation commit/tree, contract/workspace/closing/review digests, archive identity, and a domain-separated finalization digest |
| `ChangeReadScope` | Crate-private invocation guard that owns one bounded read-only lifecycle snapshot |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `LessonBriefing` | One module context that already holds substantive lessons: path, line count, bounded excerpt, and whether the excerpt was cut |
| `OpenChangeBriefing` | Lessons for the change's modules plus the configured principles file, computed when a change is opened and never stored on the record |
| `PrinciplesBriefing` | The project's principles file: path, bounded excerpt, whether the excerpt was cut, and whether the file could not be read |
| `TerminalEvidenceValidity` | State-aware exact, successor-covered, stale, authenticated-history, or corrupt-history evidence conclusion |
| `TerminalEvidenceSummary` | Shared terminal validity plus optional fail-closed reason |
| `TerminalEvidenceResult` | Change ID paired with its shared terminal-evidence summary |
| `HandoffReadiness` | `Safe`, `Conditional`, or `NotYet`: whether clearing context now loses anything the lifecycle has not recorded; serialized kebab-case, printed as `safe` / `conditional` / `not yet` |
| `HandoffSummary` | Readiness, a plain-language reason, the `specsync change status <id>` resume command, and the steps to take before clearing; carries no digest |
| `HandoffSignals` | The complete, digest-free input to `classify_handoff`: state, workflow version, sequence-ledger freeze, open questions, artifact completeness, approval/correction validity, uncommitted scoped edits, verification/review currency, stale legacy terminal evidence, and a corrupt archive (`corrupt_archive`) |
| `CorruptArchive` | Where an archived record whose terminal evidence is `corrupt-history` can be restored from: `DiffersFromHead` (the package differs from `HEAD`), `DamageCommitted` (it matches `HEAD` but not the commit that archived it), or `OutsidePackage` (it still matches that commit, or no package or repository can answer, so the failing evidence lies elsewhere); holds a project-relative path at most, never a digest (#880) |
| `ChangeSummary` | Human/agent status projection with approval health (the approved scope and the semantic delta bodies the approval recorded)/current scope digest, plain-language material expansion, the modules whose approved delta body changed after approval, validator plan, scoped-review freshness, the validated `verification_ignored_paths` list (omitted when empty or invalid), exactly one next action, a handoff verdict, and optional terminal evidence |
| `ApprovedDeltaSummary` | One module's semantic delta as the current definition approval covers it: module, a three-valued `delta_intact` (true, false, or null when no definition approval in the ledger recorded a delta binding), and, only when `delta_intact` is true, the declared requirement IDs, declared spec sections and whether the canonical spec already carries that body, or a `declarations_error` when the body is intact but the current delta grammar refuses it |
| `DeltaDeclarations` | Sorted `added`, `modified` and `removed` keys parsed from `### REQUIREMENT` or `### SPEC SECTION` delta item headings, never from prose |
| `SddCheckReport` | Unified lifecycle errors, warnings, checked-change count, and terminal-evidence results |
| `UnreadableChange` | One active-change workspace that exists on disk but could not be read, carrying its directory identity and a reason naming the offending path |
| `ChangeRoster` | The active-change roster as two separate facts: the records that were read and the workspaces that could not be, so absence and unreadability cannot share a value |
| `LifecycleCommitScope` | Crate-private answer to what one change's lifecycle commit may stage: the untracked paths the lifecycle wrote or the change owns, and the runtime files (lock, transaction journal) it must neither stage nor report |
| `LateCloseRecord` | What a late close writes beside the package it moved: actor, reason, timestamp, the state the package still holds, workflow version, the path it came from, the remote default reference and commit, the commit that merged it and its date, `verified: false`, and any dropped collision acknowledgement (REQ-change-143) |
| `LateCloseCollision` | The acknowledged sequence collision a late close removed from `.specsync/change-sequence.json` because its member left the enumerated directories (REQ-change-143) |
| `DoctorRequest` | What `specsync doctor` was asked to do: `fix`, `dry_run`, the IDs (or `all`) to close late, and the actor and reason a late close needs (REQ-cmd-change-035) |
| `DoctorReport` | Every finding, whether it was a dry run, and the actions `--fix` applied, would apply, skipped or failed; `is_clean` is false when an action failed |
| `DoctorFinding` | One problem doctor found: its kind, change (or none for the project), location, reason and fix |
| `DoctorFix` | A finding's fix: its class, the exact command or what to edit, and an optional alternative a person may choose |
| `DoctorFixClass` | `safe` (`doctor --fix` applies it), `decision` (`--close-late` with `--actor` and `--reason`) or `report-only` |
| `DoctorAction` | One thing `doctor --fix` did, or with `--dry-run` would do: kind, change, outcome and detail |

| `RetirementRecord` | Explicit unproven nonacceptance event binding original raw state/package, historical receipt anchor, actor/source claims and complete current replacement custody |
| `RetirementCustodyEntry` | Exact historical path/digest/owners and current replacement signed path/digest/owners, preserving every prior input obligation |
| `RetirementSourceKind` | Explicit accepted-receipt or committed-audited-reopening-unproven source; neither authenticates human authority |
| `RetirementCustodyClassification` | Same signed owners or the retirement-only unchanged exact ordinary-file/current named-owner classification |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `retire_change` | root, id, actor, reason, replacement, authority_source | Result<ChangeRecord, String> | Retire only intact audited-reopened intent under explicit nonacceptance with a genuinely accepted fresh custodian for every prior input; atomic/recoverable package move, idempotent same disposition, committed-history tamper refusal |
| `retire_change_with_source` | root, id, actor, reason, replacement, authority_source, source_kind | Result<ChangeRecord, String> | Explicit schema-v2 source selection; committed reopening proves only retained ancestor package custody, never Accepted transition validity |
| `validate_retirement` | root, record | Result<(), String> | Authenticate retained original package, committed disposition and receipt integrity, and prove custody still holds: every retained input is bound to the frozen prior manifest and still carried under its recorded signed owners in the custodian's authenticated evidence — the current acceptance, or the last accepted manifest during an audited post-acceptance reopen — with receipt classifications read as historical fact. The custodian's own freshness stays terminal-evidence data on its record, so the reopen remedy stays reachable (#1021); supplies no acceptance credit |
| `retirement_record` | root, record | Result<RetirementRecord, String> | Read disposition details after caller validates retirement custody |
| `retirement_next_action` | root, record | String | Explain explicitly unproven nonacceptance and any invalid custody hold |
| `run_doctor` | `root: &Path, request: &DoctorRequest` | `Result<DoctorReport, String>` | Scan the change records; with `--fix`, apply the late closes named with `--close-late` (all or nothing, or every eligible one for `all`) and then every safe fix; with `--dry-run`, report them and write nothing. Planning and the locked write boundary reject destination symlink components, including dangling links, and name recovery. A refused late close is an error and writes nothing (REQ-cmd-change-035, REQ-change-143) |
| `is_only_closed_late` | `root: &Path, id: &str` | `bool` | Whether only a closed-late package holds `id` (no active workspace, no archive package records it); status and ship-status report the closed-late record only then (REQ-change-143) |
| `closed_late_record` | `root: &Path, id: &str` | `Option<(PathBuf, Result<LateCloseRecord, String>)>` | The closed-late package for `id` under `.specsync/archive/closed-late/` and its `late-close.json` (REQ-change-143) |
| `is_canonical_commit_id` | `value: &str` | `bool` | Crate-private: whether `value` is a full commit id as SpecSync records one (40 lowercase hex digits, SHA-1). A commit identity read from a change record reaches Git as a revision only when it passes (REQ-change-013) |
| `archived_review_is_settled_by_finalize` | `root: &Path, record: &ChangeRecord` | `bool` | Crate-private: whether an archived change's scoped review is settled by the finalize that archived it, so ship-status reports the review stage as done by finalize and suggests no review (#1038) |
| `closed_late_sentence` | `root: &Path, id: &str` | `Option<String>` | The one sentence every verb gives for a closed-late change: who closed it, when and why, the commit that merged it, never re-checked and not verified, and never approved for a draft (REQ-change-143) |
| `begin_read_deadline` | `limit_secs, command, resume, json` | `ReadDeadlineGuard` | Give this thread's read-only command a deadline; past it, `check_read_deadline` reports the phase and the resume command and exits 124 (REQ-change-137) |
| `begin_worker_read_deadline` | `deadline: Option<ReadDeadline>` | `ReadDeadlineGuard` | Give a prefetch worker the command's deadline; a worker stops taking work instead of reporting |
| `check_read_deadline` | `phase, done, total` | `Result<(), String>` | Call between items of a long read phase; past the deadline the command's thread flushes its read caches, reports and exits 124, and a worker gets `Err` |
| `current_read_deadline` | — | `Option<ReadDeadline>` | This thread's deadline, to hand to a worker |
| `read_deadline_passed` | — | `bool` | Whether this thread's deadline has passed |
| `read_timeout_secs` | `flag: Option<u64>` | `Option<u64>` | `--timeout` when given, else `SPECSYNC_TIMEOUT` when it is a whole number of seconds, else none (REQ-cmd-change-032) |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas only when not already canonical |
| `acceptance_entries` | `root: &Path, record: &ChangeRecord` | `Vec<AcceptanceInputEntryV1>` | Accepted acceptance-input entries, so `change show --json` can surface the `specsync.acceptance-entry.v1` digests `change supersede --digest` requires; empty when evidence is absent |
| `record_refused_attempt` | `root: &Path, record: &ChangeRecord, reason: &str` | `Result<(), String>` | Append a `passed: false` attempt carrying the refusal reason to the change's `verification-attempts.json` (#847); never writes `verification.json` or moves the recorded state |
| `accumulated_lessons` | `root, modules` | `Vec<(String, usize)>` | Substantive-prose line count for each module context that holds any, so a new change can be pointed at what its modules already learned |
| `active_change_id` | `root` | `Option<String>` | The change a bare lifecycle command acts on: the single active approved/implementing/verifying record — the same states `check_change` selects — or none |
| `add_acceptance_owner_correction` | `root, id, path, module, actor, reason` | `Result<ChangeRecord, String>` | Append one audited exact canonical owner correction to a reopened already-applied change |
| `add_acceptance_owner_corrections` | `root, id, entries, actor, reason` | `Result<ChangeRecord, String>` | Validate every exact path/module owner correction, then append all as sequenced audit entries in one transactional write |
| `add_dependency` | `root, id, dependency` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, declares ordering between active changes, and invalidates stale approval digests |
| `add_dependency_with_snapshot` | `root, id, dependency` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the dependency mutation with its full in-transaction machine snapshot |
| `add_missing_acceptance_owner_corrections` | `root, id, module, actor, reason` | `Result<ChangeRecord, String>` | Discover production-source affected paths lacking canonical ownership for a module and append them as one transactional batch |
| `add_supersedes_obligation` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then adds one definition-bound semantic succession obligation to a draft |
| `add_supersedes_obligation_with_snapshot` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the supersession mutation with its full in-transaction machine snapshot |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or atomically enable SDD — writing an enabled policy when none exists and flipping `enabled` on one written off by `init`, failing closed on a policy it cannot parse — activate workflow v2 without stranding cutoff-ineligible legacy records or rewriting legacy policy, and import OpenSpec or Spec Kit artifacts |
| `answer_question` | `root, id, question, answer` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then persists an interview answer and updates adaptive artifacts |
| `answer_question_with_snapshot` | `root, id, question, answer` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the answer mutation with its full in-transaction machine snapshot |
| `approval_current_for_guidance` | `root, record` | `bool` | Whether an approved, implementing, or verifying change's definition approval still covers its approved scope and the semantic delta bodies it recorded — the predicate behind `ChangeSummary.approval_valid` — for human next-action guidance without printing a digest |
| `approved_delta_summaries` | `root, record` | `Vec<ApprovedDeltaSummary>` | Per-module approved delta declarations for an approved, implementing or verifying change whose scope approval is current, read from the same bytes the delta-binding comparison digests, shared with `ChangeSummary` within one read scope; empty in every other state |
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record an ordinary mandatory definition approval |
| `approve_definition_portable_v501` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Atomically record the marked current/5.0.1 portable definition pair |
| `approve_definition_with_relay` | `root, id, actor, note, portable_v501, relayed_by` | `Result<ChangeRecord, String>` | Record a definition approval (or the portable pair) carrying an optional relayed decision; a relay whose `decided_by` is not the explicit `--actor` is refused before anything is written |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `artifacts_complete_for_guidance` | `root, record` | `bool` | Lightweight selected-artifact completeness for human next-action guidance without digest loaders |
| `audit_project` | `root: &Path` | `SddCheckReport` | Active workspaces + living policy/spec coherence only — does not rewalk archived terminal evidence. A duplicated requirement ID in a living `requirements.md` is an error and is named |
| `audit_project_with_archives` | `root: &Path, archives: bool` | `SddCheckReport` | `audit_project`, and with `archives` also every archived change's own workflow-version history, each failure named by change ID; the walk finalize no longer repeats (REQ-change-124). Duplicate requirement IDs are reported whether or not `--archives` is set |
| `backfill_reopen_digests` | `root: &Path, dry_run: bool` | `Result<ReopenBackfillReport, String>` | Backfill 5.1 reopening digest fields on 5.0.1-era ledgers with verified, idempotent, dry-run-aware writes |
| `begin_change_read_scope` | `root: &Path` | `ChangeReadScope` | Install one invocation-scoped read snapshot for list/show/status and project reports |
| `check_change` | `root, optional id` | `Result<Option<VerificationRecord>, String>` | Select one approved/implementing change, materialize its canonical deltas, and compare this change's specs to code |
| `check_change_with_strict` | `root, optional id, strict` | `Result<Option<VerificationRecord>, String>` | Run `check_change` with warnings failing as they do under `specsync check --strict` |
| `check_change_as` | `root, optional id, invocation: CheckInvocation` | `Result<Option<VerificationRecord>, String>` | Run `check_change_with_strict` for one known invocation, so an open-task refusal names that invocation's own command line, flags included, as the command to re-run (REQ-change-131) |
| `check_project` | `root: &Path` | `SddCheckReport` | Full lifecycle integrity including archive terminal evidence (tests and rare callers; not the default CLI path) |
| `corrupt_archive_recovery_for_guidance` | `root, record` | `Option<String>` | For an archived record whose terminal evidence is `corrupt-history`, the restore step `ChangeSummary.next_action` and the handoff carry, for the text `Next:` line: `git restore --source=HEAD --staged --worktree -- <package>` when the package differs from `HEAD`, the same restore from the commit that archived it when the damage is committed, otherwise `specsync change status <id> --json` to find the failing evidence; each ends with `specsync change status <id>`, and every interpolated ID and path is shell-quoted. `None` otherwise. No digest (#880) |
| `classify_handoff` | `id, signals` | `HandoffSummary` | Pure classification of handoff readiness from `HandoffSignals`; the only source of the verdict, so text and JSON cannot disagree |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `finalize_change` | `root, id` | `Result<PathBuf, String>` | Validate current verification/review evidence and transactionally produce the dated same-PR archive |
| `find_change_dir` | Resolves a change's workspace wherever it lives, active or archived — the single answer to where a change's artifacts are |
| `floor_sequence_ledger_to_committed` | `root: &Path` | `Result<Option<(u64, u64)>, String>` | Raise a working-tree sequence ledger to the committed high-water mark before staging, returning the previous and adopted values so the caller can disclose the raise, or `None` when the ledger is already at or above it |
| `sequence_ledger_freeze_next_action` | `root: &Path` | `Option<String>` | The remediation for a frozen change-sequence ledger, or `None`. Every status surface names it before any other next step, the upstream archive included (#908) |
| `handoff_summary` | `root, record` | `HandoffSummary` | Gather only the signals the record's state needs and classify them; an Archived record reads the same terminal-evidence verdict `ChangeSummary` carries, so text cannot call `safe` an archive JSON reports as `corrupt-history` (#880); the same verdict `ChangeSummary.handoff` carries |
| `lesson_fold_targets` | `root, id` | `Vec<String>` | Module context paths this change's lessons are folded into at archival; empty when the change is unreadable or owns no specs |
| `lifecycle_commit_scope` | `root, id` | `Result<LifecycleCommitScope, String>` | The one answer to which untracked paths a lifecycle commit for this change may stage: its workspace, its archive package, each affected spec's canonical file and companions, and the lifecycle ledgers — never its `affected_paths` prefixes; `Err` when the change cannot be loaded |
| `list_changes` | `root: &Path` | `Result<ChangeRoster, String>` | List active changes in stable ID order alongside the workspaces that could not be read, including a directory that holds a regular file but no `state.json`; a husk with no regular file is not a change; `Err` only when the changes directory itself is unreadable |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `module_context_path` | `root, module` | `String` | Where a module's accumulated lessons live, from the registry-aware canonical spec directory, shared by surfacing and folding so they cannot disagree |
| `configured_principles_file` | `root` | `Result<Option<String>, String>` | The configured principles file, or the error when `.specsync/sdd.json` cannot be read |
| `principles_file_is_present` | `root`, `path` | `Result<bool, String>` | Whether that path is a file inside the project. An absolute path or `..` is an error |
| `answer_command_hint` | `id: &str, question_id: &str, choices: &[String]` | `String` | The backticked `change answer` command shared by draft refusals and text/JSON status: multiple choices form a placeholder, one choice is literal, and scope questions retain their specific free-text placeholders (REQ-change-133) |
| `normalize_actor` | `value: &str, verb: &str` | `Result<String, String>` | Crate-private shared actor identity rule: trim padding, refuse empty-after-trim, control characters, U+2028/U+2029, and every format (Cf) or default-ignorable code point except ZWNJ and ZWJ, before any ledger write (#426) |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Record-only unanswered interview questions. Does not ask about lessons or principles |
| `next_questions_for` | `root: &Path, record: &ChangeRecord` | `Vec<InterviewQuestion>` | Unanswered interview questions, including the draft-only constraints question when the rest of the interview is done and lessons or a principles file exist |
| `open_change_briefing` | `root: &Path, record: &ChangeRecord` | `OpenChangeBriefing` | Bounded lessons and principles for this change. Omits a scaffold-only context and never fails the caller |
| `open_tasks_block` | `record: &ChangeRecord` | `bool` | Whether an open task blocks the next verb: approved, implementing or verifying, or a workflow-v2 finalization stopped after acceptance (its workspace still active) |
| `open_tasks_next_step` | `root: &Path, record: &ChangeRecord` | `Option<String>` | The open tasks of the change's selected `tasks.md` by line and text, with how to close them, or `None` when every task is done. The one sentence status, ship-status and every refusal use (REQ-change-132) |
| `open_tasks_refusal` | `root, record, rerun: &str` | `Option<String>` | The refusal a verb gives while tasks are open: `tasks.md contains incomplete tasks: <open tasks>, then re-run `<rerun>``, or `None` when nothing is open (REQ-change-131) |
| `record_bootstrap_paths` | `root: &Path` | `Result<(), String>` | Record the protected SDD paths this bootstrap created in `.specsync/bootstrap.json`, so initialization's own output is not reported as uncovered meaningful delivery; editing a recorded file revokes its exemption |
| `record_scoped_review` | `root, id, reviewer` | `Result<ScopedReviewRecord, String>` | Record one implementation-scoped review bound to current governed inputs; the reviewer may be the definition approver |
| `record_scoped_review_with_verdict` | `root, id, reviewer, verdict` | `Result<ScopedReviewRecord, String>` | Record an explicit passing or blocking review; only a current passing verdict permits finalization |
| `record_scoped_review_with_relay` | `root, id, reviewer, verdict, relayed_by` | `Result<ScopedReviewRecord, String>` | Record a scoped review carrying an optional relayed decision whose `decided_by` must be the validated reviewer claim |
| `recorded_claims` | `root, record` | `Result<Vec<RecordedClaim>, String>` | Read-only projection of the effective definition approval (never one a send-back cancelled) and the latest scoped review, with any relayed decision; empty when neither exists, and, when a relay record cannot be read, an error naming `specsync change repair-relays <id>` for an active change or the `git checkout HEAD` recovery for an archived one |
| `relayed_decision` | `rerun, decided_by, words, where_, recorded_by` | `Result<Option<RelayedDecision>, String>` | Build the optional record from command flags: none given is `None`; some given is refused naming every missing flag and the full `rerun` command; a blank or over-4096-byte field is refused naming its flag |
| `repair_relayed_decisions` | `root, id` | `Result<RelayRepairReport, String>` | Repair every damaged `relayed_by` in an active change's `approvals.json`, `review-attempts.json` and `review.json`: put each back from the same record at `HEAD`, then from its twin (`review.json` and the last attempt), and otherwise remove only that relay; never removes an approval or attempt or changes a verdict, writes every file in one step, and writes nothing when nothing is damaged; refuses an archived change or a file that is not JSON, naming the `git checkout HEAD` recovery when one exists |
| `recorded_scoped_review_currency` | `root, record` | `Option<ScopedReviewCurrency>` | Classify a change's recorded scoped review against the tree on disk; `None` means no usable review record exists, which is a different question from currency |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `ownership_preflight_blocker` | `root, record` | `Result<Option<(String, String)>, String>` | The ship-preflight blocker, and the recovery to name next, for production source in the change's delivery that no declared module owns; builds finalize's acceptance manifest read-only, so it names every input finalize would refuse (REQ-cmd-change-008, #986, #956) |
| `send_back_change` | `root, id, actor, reason` | `Result<SendBackResult, String>` | Move an approved, implementing or verifying change back to draft for re-scoping, append an audited send-back, and cancel the earlier definition approval without removing it |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, correction health, and next action using the shared verification-freshness predicate |
| `summarize_change_with_strict` | `root, record, explicit_strict` | `ChangeSummary` | Project the same status plus the exact scoped command the next pass records as evidence; `--strict` only when requested |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Compare this change's specs to code in-process and record commit/contract evidence |
| `verify_change_with_strict` | `root, id, strict` | `Result<VerificationRecord, String>` | The same spec↔code pass with warnings failing as they do under `specsync check --strict` |
| `verification_ignored_paths_for_display` | `root: &Path` | `Vec<String>` | The validated, sorted `.specsync/sdd.json` `verification_ignored_paths` list for disclosure; empty when it is absent, empty or invalid |
| `verification_ignored_paths_refusal` | `root: &Path, record: Option<&ChangeRecord>` | `Option<String>` | The refusal an invalid list, or an entry overlapping the given change's declared `affected_paths`, causes in every lifecycle verb; `None` when nothing refuses |
| `upstream_archive_of` | `root: &Path, record: &ChangeRecord` | `Option<UpstreamArchive>` | The remote default's archive of an unarchived record when `HEAD` does not contain it; review and finalize refuse it and status never reports it `safe` (REQ-change-129). Checks every unarchived state with a targeted lookup of the remote default's archive directory, never the full archive index. `None` with no remote default, or for an archive already in `HEAD` (reopen); once a remote default resolves, a failed or truncated listing or batched read, an archived `state.json` with no readable `id`, or a failed addition or ancestry lookup for a matching package, is `CannotTell`, never `None` (REQ-change-142). The archive root is passed as `:(top,literal)` (#962) |
| `upstream_archive_message` | `id: &str, upstream: &UpstreamArchive` | `String` | The single sentence review, finalize, status and ship-status print. `Archived`: the reference, the archive commit, the unarchived copy and the recovery: rebase onto that reference dropping the commit that added the copy. `CannotTell`: the reference, that the tool cannot tell, and how to find out: `git fetch --refetch origin` (which repairs a missing object even when the reference has not moved), `git remote set-head origin --auto` (which repairs a dangling remote default), then `specsync --root '<project-root>' change status -- '<id>'`, which retains the selected project and reruns this lookup and either clears the refusal or names the archive and the rebase; the root and id are separate shell-quoted words |
| `write_default_policy` | `root: &Path, verification_commands: Vec<String>` | `Result<(), String>` | Write new-project/adoption policy without overwriting existing policy |
| `bounded_failure_output` | `findings: &[String]` | `Option<String>` | Crate-private bound of failed-command findings to the last 4 KiB for `CommandEvidence.output`; `None` when there is nothing to record (#640) |

**Exported Methods**

| Method | Description |
|--------|-------------|
| `as_str` | Return the stable serialized name for a change state, kind, or correction field |
| `parse` | Parse user-facing change-kind, artifact, or supported correction-field names into typed values |
| `file_name` | Resolve an adaptive artifact to its safe Markdown filename |
| `is_clean` | Return true when a ledger backfill recorded no per-change failures |
| `is_degraded` | Return true when at least one workspace could not be read, so no caller may draw a conclusion from a missing record |

Acceptance Criteria

- Nested lifecycle commands still fail once with the established deterministic contextual error.
- The process marker and diagnostic helper remain private binary implementation details.
- Correction inspection exposes typed portable records without exposing mutable ledger internals.
- Acceptance-owner corrections expose only immutable audit fields and never mutable internal ledgers.

| `enable_historical_reconciliation` | Request definition-bound unproven historical receipt reconciliation; require fresh approval and closing gates |
| `HistoricalReceiptWitnessV1` | Immutable predecessor identity, package digest and frozen manifest digest |
| `HistoricalReconciliationEvidenceV1` | Explicit unproven status, immutable base anchor and sorted exact receipt witnesses |

## Invariants

1. Change IDs are minted from the change description as a slug and are unique across active and archived workspaces; the historical `CHG-NNNN-slug` ordinals are read for collision accounting and never allocated.
2. No emergency or force transition bypass exists.
3. Approval digests exclude volatile lifecycle state. The workflow-v1 definition digest hashes every selected artifact and semantic delta body; the workflow-v2 stable scope digest hashes intent and boundary only, and delta bodies are bound instead by a per-module content digest recorded on the definition approval event.
4. Any addition, removal, or replacement in approved stable scope invalidates approval until the new digest is approved.
5. Finalization rejects stale commits, contracts, reviews, incomplete tasks, failed tests, and missing requirement evidence.
6. Overlapping active semantic keys are blocked unless changes declare ordering dependencies.
7. Canonical spec versions increment and changelogs reference the accepted change ID.
8. A failed multi-file write restores all prior canonical content.
9. Change dependencies are acyclic and must be accepted or archived before dependent implementation begins.
10. Meaningful-path coverage compares the branch with the current GitHub/remote default base after a rebase, falling back to the recorded creation commit only when no remote base is available.
11. Approval digests hash repository-relative artifact paths so identical Git content validates across checkout locations and operating systems.
12. Verification-command detection remains for adoption-era policy files; `change check` does not execute the list.
13. Persisted and hashed project paths use forward slashes on every operating system.
14. `change check` does not spawn configured commands, so there is no child stdout to suppress.
15. Reopening accepted evidence is rejected as current only when its delivery inputs are current, its verification commit is anchored, and any manifest-less legacy acceptance is historically reconstructible; reopening stale evidence never reapplies an already canonical semantic delta.
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event unless every difference is a validated additive exact-owner correction.
17. False default lifecycle fields remain absent from new persisted state, while definition validation recognizes both omitted and transitional explicit-false encodings so upgrades preserve existing approvals and verification; explicit acceptance appends stable definition evidence when the latest compatible approval uses the transitional encoding.
18. An unreachable verification commit or failed manifest-less legacy acceptance reconstruction is an admissible staleness axis for audited reopen, recorded as an explicit cause in the reopen ledger. Definition, verification, and closing authentication checks remain fatal; reopening grants no archival authority to the old evidence and fresh verification and closing remain mandatory.
19. Acceptance appends a Change Log row matching the canonical table's existing column schema and uses the post-bump version when the schema includes `Version`.
20. Generated bookkeeping never replaces explicit delivery scope; registry authority, policy enablement, and native command identity are evaluated consistently before lifecycle enforcement.
21. Trusted correction-history discovery ignores unresolved remote-default references and parses Git tree paths without quoting ambiguity; regression fixtures preserve quoted-path coverage where supported while remaining valid on Windows.
22. Local and hosted verification freshness inspect every intervening commit against every parent, permit only `state.json`, `verification.json`, `verification-attempts.json`, `review.json`, and `review-attempts.json` below canonical active-change IDs, and never infer safety from a net diff or broad volatile-path exclusion.
23. Exact-owner corrections are additive, restricted to an original affected path and a current canonical source owner, and cannot mutate semantic definition fields or prior evidence.
24. A fully valid later accepted sequence owner covers only historical sequence-ledger drift; reconstruction reuses exact committed collision-owner ledger bytes when available, while the current owner and every non-ledger input remain exact.
25. A structurally valid audited delivery reopen preserves immutable sequence-collision history while fresh verification and closing approval remain mandatory.
26. Accepted-change archival trusts an in-history commit recording the change as accepted with byte-identical evidence when no first-acceptance transition anchor matches, so squash-merged evidence remains archivable while the exactly-one-eligible rule stays fail-closed.
27. Legacy acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers validate without remediation while newly signed evidence stays fail-closed.
28. Batch exact-owner correction validates every proposed path/module pair independently and fails closed with zero persisted mutations when any entry is invalid.
29. The 5.0 ledger migration backfills reopening digest fields idempotently from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.
30. Canonical module path resolution treats missing and inert local registries as absent fallbacks while non-inert unparsable registries still fail closed with the established parse diagnostic.
31. Immutable workflow-origin validation follows every bounded reachable canonical dated archive path for the exact change ID, preserving identity across archive, reopen, and cross-date rearchive moves, including when the project root is a subdirectory of the Git repository. The discovery glob, the history walk of those paths, and the read-cache invalidation walk anchor repository-relative pathspecs at the repository root with `:(top)`, so a non-empty repository prefix is not applied twice.
32. The workflow-v2 baseline retains its exact introduction bytes at every bounded touching commit and readable parent, rejecting rewrite-then-restore history.
33. Answer, dependency, and supersession mutations load and validate correction history only after acquiring the lifecycle project lock.
34. `finalize_change` assembles `lesson-bundle.md` into the archive on a best-effort basis: a bundle failure never undoes a completed archival, and the material is read entirely from disk so finalize keeps working offline and in CI. SpecSync assembles and never authors the lessons. Folding them into `context.md` is a convention, not a `next_action` gate. Finalize and ship name the bundle path in text and JSON without making that fold the next action.
35. This module defines no frontmatter reader of its own. Lesson counting, archived lesson bundles, and artifact completeness all read through `parser::strip_frontmatter`, the single canonical implementation, which ends frontmatter at its CLOSING delimiter LINE in either LF or CRLF encoding and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, a CRLF-authored companion is stripped exactly as an LF one is, a leading BOM never hides the opening delimiter, and a delimiter line padded with trailing whitespace still ends the block at BOTH ends. Four failure modes of the strippers this module used to own are gone with them: a written CRLF artifact is no longer refused as incomplete; an artifact that is only frontmatter is no longer accepted as written when it is closed at end of file, prefixed with a BOM, or opened with a delimiter carrying a trailing space; and prose above the first horizontal rule in a body is no longer deleted when the CLOSING delimiter carries one. One residual is stated rather than guessed at: an artifact opened with `----`, a Markdown thematic break and not a delimiter, still reads as written even when it holds nothing but frontmatter — accepting it as a delimiter would cut real bodies at their first rule, which is the worse failure, and deriving the gate from the generated scaffold instead would not close it either, because a file with a mangled opener no longer equals that scaffold.
36. A `###` heading inside an open semantic-delta item is section CONTENT and does not end that item. Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` start a new item, and classification happens before the previous item is flushed — otherwise one section carrying subheadings becomes several items under one key and application keeps only the last, silently discarding documented behaviour the change never touched.
37. A semantic delta declaring the same operation, target and key more than once is REFUSED. Applying it would keep the last body and discard the earlier ones with no diagnostic.
38. Semantic delta bodies are bound to the definition approval that signed them: approval records a per-module digest over each delta file's body, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. The body is hashed with `\r\n` folded to `\n` and NOTHING ELSE folded, so the binding asks the question delta application already asks: `markdown_block_matches` compares ignoring line-ending style and `parse_delta` reads through `lines()`, so a CRLF and an LF delta materialize byte-identical canonical specs and a checkout that rewrote the line endings cannot invalidate an approval. The equality stays STRICTLY NARROWER than the applier's: trailing whitespace, blank lines and a lone carriage return are wording a reviewer signed and still move the digest, and Git rewrites none of them on its own. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.
39. Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the JSON already pinned there and for the reason that file already states: change artifacts and semantic delta bodies are read as lifecycle evidence, so a working tree that rewrites them into CRLF makes honest, unmodified work arrive in non-canonical form. The pin governs this repository's own working trees; it is not a substitute for readers that tolerate CRLF, because an adopter's repository, a tarball, or an archive extracted without Git is never covered by it.
40. A recorded delta binding is MONOTONE within one approval ledger. Every writer of a `definition` gate records the per-module delta digest it approved — ordinary approval, the normalizing approval inside explicit acceptance, and both members of the portable SpecSync 5.0.1 pair alike — so within a ledger the binding only ever goes from absent to present. An effective definition approval that records no delta wording while another definition approval in the same ledger records it is a claim being withdrawn, not evidence predating the binding, and materialization and acceptance refuse it and name the re-approval remedy. Absence across a whole ledger still reads as unknown, because that is the only shape recorded history has: a change is either from before the binding existed or from after it, never both.
41. `canonical_applied` records that materialization RAN, never that it ran for the delta bodies on disk now, so `change check` and acceptance decide from the canonical artefacts instead of from the flag alone. Materialization produces three outputs per module — the delta applied to the canonical files, the spec's `version:` bump, and the spec's Change Log row — and the flag's short-circuit skipped ALL THREE, so a delta corrected after review and re-approved satisfied the delta binding (a new approval signs the new body) and then left changed contract text with no bump and no row while `change check`, `change audit --archives` and `specsync check --strict` all passed. A module is materialized again when its delta is not fully reflected in the canonical files or when its Change Log carries no row for the change, and is left untouched when both hold: a byte-identical re-approval still writes nothing, and one change bumps one module's version exactly once. Convergence is scoped to an already-applied change — on a first materialization every application refusal still fires, and only afterwards does an already-reflected item, such as a `## REMOVED` block that is already absent, read as done rather than as an error.
42. Handoff readiness is a PURE function of `HandoffSignals`, and the verdict a session sees in text is the same object JSON carries under `summary.handoff`. A frozen sequence ledger, a stale approval digest, an invalid correction ledger, stale legacy terminal evidence, and an archive whose terminal evidence is `corrupt-history` are `not yet` and name their repair, because clearing context there strands the next session on a gate it cannot see the cause of. A Draft is never `safe` — the interview and artifacts live only in the session's head until approval records them — so it is `conditional` and names approval as the first clean boundary. Uncommitted edits under `affected_paths` are `conditional` and name committing or writing intent into `change.md`; evidence under `.specsync/` alone never counts, because `review.json` is uncommitted between `review` and `finalize` by design. A current approval on a clean tree, a Verifying change with current verification, an Accepted workflow-v2 change, and an Archived change whose terminal evidence authenticates are `safe`, and the reason says where the next session resumes. A corrupt archive's one step is the restore that recovers it — from `HEAD` when the package differs from it, from the commit that archived it when the damage is committed, and from wherever `terminal_evidence.reason` points when the package is intact — then `specsync change status <id>`, with every interpolated ID and path shell-quoted; it is also the summary's next action; text reads the same terminal-evidence verdict as JSON, so the two cannot disagree (#880). The reason, the resume command, and the steps carry the change ID and literal prose only; no digest reaches them.

43. A draft whose affected modules have substantive lessons, or whose project names a principles file, is asked which of those constrain it once the rest of the interview is answered, and "none" is a complete answer stored in the stable scope. A draft with neither is not asked. A change that has left draft without that answer is not sent back to the interview. Re-answering affected specs clears the answer. A selected plan is incomplete until Approach, Out of scope, Steps, Risks, and Constraints consulted each contain substantive text. A change that did not select a plan does not gain one. Opening, showing, or asking status computes a briefing of those lessons and of that principles file. The excerpt is bounded, a scaffold-only context is omitted, and an unreadable file does not fail the command.

44. Opening a change fails when `.specsync/sdd.json` exists and cannot be parsed, including when a workflow-v2 baseline is already present. The error says the SDD policy is invalid. A missing policy file still opens a workflow-v2 change. A parsed policy is not treated as absent.

45. A bounded Git read returns when the direct Git child exits. On Unix and on Windows, once that exit is observed the reader drains one finite snapshot of the bytes already queued and stops, so a descendant that keeps writing cannot hold the join and the child's last queued bytes are not dropped. `change status` is one such read (SPEED-1, SPEED-1.a, SPEED-1.e). The wall-clock deadline, the output cap, and the stdin-writer join are unchanged.
46. `change audit` refuses a living `requirements.md` that contains the same `### REQ-<module>-<number>` heading more than once, and the error names that ID. The scan runs even when SDD policy is disabled. A fenced copy of the heading is not a second requirement. This does not weaken any other audit refusal.
47. Checkbox state is outside the approval and verification digests, so ticking a task needs no reapproval or re-verification. Every verb that verifies a change, records a passing review of it or finalizes it therefore asks the task gate itself, before writing anything. A task reopened after a passing check is refused at review and finalize, and the refusal names each open task and the exact command to re-run (REQ-change-131).
48. A scoped review recorded before a send-back stays valid review history. It validates against the definition approval it bound, including one the send-back cancelled, so it never blocks a fresh `change review` after re-approval, and it is never removed or rewritten. Whether a review counts is a separate question, answered as before: it must match the current contract and evidence and bind a definition approval the latest send-back did not cancel. A review bound only to a cancelled approval is reported stale, never unreadable, and never lets `change ship` or `change finalize` through (#914, REQ-change-123, CHANGE-12.b, CHANGE-12.d).
49. A passing proof of a legacy archive baseline's introductions is kept in the persistent read cache under a key that digests the baseline ledger digest, the full cutoff commit id and every enumerated archive's current subtree digest, so an archived read of one legacy archive does not walk every introduction commit and every legacy archive's trees again. The proof's other inputs are commits named by full id, which are immutable while the persistent cache is enabled. A refusal is never cached, a changed archive subtree or baseline misses the cache and is judged live, and with the persistent cache disabled (shallow, grafted, replaced or partial clone) the proof always runs live (#901).
50. A send-back records how many scoped review attempts preceded it, and a scoped review recorded before the latest send-back is never current, even when the change is re-approved with an unchanged scope and the re-approval carries the same digest the review followed; it is reported stale until a fresh `change review`, and `change ship` and `change finalize` refuse it. A send-back recorded before that count existed answers by time: only a review with a strictly later timestamp counts. Review history validity is unchanged (#960, REQ-change-123, CHANGE-12.e).
51. Canonical ownership is checked before a human is asked to act. Approve refuses an existing production-source path that no declared module owns, including for a no-spec-change change with no declared specs, and for a draft names `specsync change answer <id> affected_specs <modules>`. Ship preflight, `change ship --dry-run` and `change ship-status` build finalize's acceptance manifest read-only and report every production-source delivery input no declared module owns as a blocker, naming the send-back and answer recovery as the next action, so a change finalize would refuse is never reported ready to finalize. Who owns a path is decided exactly as before (#986, #956, REQ-change-053).
52. A corrupt or unreadable transaction journal is reported with its path and a quoted set-aside command; every printed `change reopen <id>` remediation includes `--actor <name> --reason <text>` (#433, REQ-change-151).
53. The delta materializer matches a requirement block through the shared requirement-heading parser (`crate::validator::requirement_heading_id`), so every header form the parser accepts — a title suffix (`### REQ-<module>-<number> — Title`), punctuation after the id (`### REQ-x-001: Title`), a multi-segment module id, up to three spaces of indent — matches, and the id token boundary holds (`REQ-cli-006` never matches `REQ-cli-0060`). The tolerance applies only to `### ` requirement blocks — a `## ` section key still requires an exact heading, so `Change Log` never matches `## Change Log Archive`. A materialized block keeps its matched header line unchanged, title included; convergence comes from the tolerant matcher, not normalization (#1024, REQ-change-152, REQ-validator-047).
54. An `ignored_paths` entry whose coverage is entirely always-meaningful — the configured specs tree, or an SDD policy file named directly — is inert, and the policy loader says so on stderr at load, once per process, naming the entry and the file to edit; an entry that also covers other files (`.specsync/` covers the change workspaces) stays silent, and a newly written default policy carries no inert entry (#676, REQ-change-155).
55. A `change check` refused on grounds in the change's own workspace — open tasks or missing requirement evidence — records the refusal as a `passed: false` entry carrying its reason in the change's `verification-attempts.json`, so a reader can see the check was tried and why it failed; a gate failure about another change's module records nothing. A refusal never writes `verification.json` and never moves the recorded state (#847, REQ-change-154).
56. Interview answers and identity strings are validated before they are persisted: `change answer` refuses a question key outside the accepted interview set naming the accepted ids, refuses an empty answer or one that parses to no value (`none` clears `affected_specs` and `dependencies`), refuses a yes/no answer that is not yes or no (recognised spellings are stored as the choice they name) and an `affected_paths` answer of `[]`, and refuses an `affected_specs` entry whose module has no spec at its canonical path, its own directory and file spelled exactly and reached without a symlink (unless the project has no spec files and no registry entries; `change new --spec` likewise), naming the path and how to create the spec; an exact `none` or `[]` clears the optional lists (`none` is refused as ambiguous when something is named `none`); a spec declaring the module off its canonical path is named with move-or-register advice; owner recoveries print no `affected_specs` answer naming a module without its spec (not even in a project with no specs), say to create a missing spec first, and name each declared module whose spec is missing; a case variant of a module name is refused naming the module. Actor identity strings are trimmed and refused when empty or holding control characters, the separators U+2028/U+2029, or any format (Cf) or default-ignorable code point as of Unicode 16.0 (ZWNJ and ZWJ are allowed), before any ledger write: through one shared `normalize_actor` on approve, reopen, send-back, correct, correct-owner, reconcile-history, retire and the doctor late close, and through the stricter reviewer-claim check on review. A relayed approval's decider is trimmed like its actor (#426, REQ-change-150).
57. A lifecycle record or ledger that SpecSync rewrites in place (`state.json`, `approvals.json`, `verification-attempts.json`, `review-attempts.json`, `corrections.json`, a relay repair's `review.json`, the archived `state.json` and the change-sequence ledger) keeps every member it does not know, with its exact key and value text: members of the root, and members of each array entry, matched by known content wherever the rewrite moves it, otherwise by position in an array of unchanged length, which counts as the same record edited in place (no lifecycle rewrite replaces an entry in place); a moved entry never takes another entry's members. The kept members change nothing this SpecSync reads and every digest is unchanged; a rewrite that cannot keep them, including one that removes an entry holding them, is refused instead of dropping them (#1043, #434, REQ-change-079).
58. Failed verification command evidence carries a bounded `output` of the findings that failed it (errors, and warnings only under `--strict`), so a recorded failure is classifiable without re-running; passing records omit the field and serialize byte-identically to before (#640, REQ-change-153).
59. After definition approval, the guided next action for an approved or implementing change names `specsync change check <id> --commit`. `change review` and `change ship` refuse when the recorded verification commit does not contain the change's own work — the current contents of its `affected_paths` and any canonical specs it materialized — and they do not record a review bound to such a commit. Uncommitted files under `.specsync/` alone do not trigger that refusal (#535, REQ-change-156, REQ-change-157).
60. A change that holds a relayed decision is recorded at workflow version 3, the gated form of workflow v2, so SpecSync 6.2.0 and older refuse it with "was written by a newer SpecSync" rather than rewrite it and drop `relayed_by` (REQ-change-158). Approve and review write version 3 before the relay, and every other `state.json` write raises a version-2 change whose ledgers hold a relay and never writes 2 over a file already at 3. The origin anchor stays 2; the workflow-version history accepts only the 2-to-3 raise and refuses 3 back to 2; every digest reads 3 as 2, so the raise stales nothing. A workflow-v1 change, which a `--portable-5-0-1` pair requires, keeps version 1, and approve and review print a note that its relay is not gated.

## Behavioral Examples

**Scenario: A broken policy does not open a change**

- **Given** `.specsync/sdd.json` exists and is not valid JSON
- **When** a change is opened
- **Then** opening fails with an invalid SDD policy error and no change is created
- **And** a project with no policy file still opens a workflow-v2 change

**Scenario: Archival compounds knowledge into the spec**

- **Given** a change is finalized
- **When** `finalize_change` archives it
- **Then** the archive contains `lesson-bundle.md` naming the change, its specs, its paths, and the material to fold
- **And** an unwritable bundle leaves the archive intact and the finalize successful

**Scenario: Verified feature delivery**

- **Given** an approved feature with `REQ-auth-001`, completed artifacts, and matching specs and code
- **When** implementation verifies, receives its scoped PR review, and runs `change finalize`
- **Then** canonical requirements/specs update and the package moves to the dated archive in the same PR, ready for GitHub merge

**Scenario: Approved intent changes**

- **Given** a valid definition approval
- **When** a selected design, requirement, or delta is edited
- **Then** progress is blocked until the new digest is approved

**Scenario: Feature branch rebases onto upstream**

- **Given** a change workspace created before new commits landed on the remote default branch
- **When** the feature branch rebases and unified checking computes meaningful changed paths
- **Then** upstream-only paths are excluded and only the feature branch diff requires change coverage

**Scenario: Review fixes stale accepted evidence**

- **Given** an accepted change whose governed delivery inputs changed after closing approval
- **When** a human reopens it with an actor and reason
- **Then** the prior verification and closing approval remain in audit history, strict checking stays red until fresh verification, and reacceptance records a new closing approval without reapplying canonical deltas

**Scenario: Persisted verification evidence**

- **Given** a supported verification run passed on the current commit
- **When** one or more descendant commits persist only its canonical state, verification, and attempt-ledger files
- **Then** local status, local strict checking, and hosted checking all keep the evidence current while matching contract and project-input digests remain mandatory

**Scenario: Inert registry stub falls back to conventional paths**

- **Given** a project with an inert 5.0.1-era `.specsync/registry.toml` stub and a conventional `specs/auth/auth.spec.md`
- **When** semantic preparation resolves module `auth`
- **Then** resolution succeeds via the conventional path without requiring a registry name

**Scenario: Overlapping Git candidate batches repeat an index entry**

- **Given** a delivery scope containing a tracked parent directory and enough exact tracked children to cross the pathspec batch boundary
- **When** Git returns one child through both the parent pathspec and its later exact pathspec
- **Then** identical mode/object pairs are represented once, while either a mode or object mismatch fails closed

**Scenario: Handoff verdict follows the lifecycle, not the evidence files**

- **Given** an approved change whose implementation is committed and whose scoped human review just wrote an uncommitted `review.json`
- **When** the session asks `handoff_summary` before and after editing one file under `affected_paths`
- **Then** the review file alone leaves the verdict `safe` resuming at finalize, the edit turns it `conditional` naming a commit or a `change.md` note, and neither reason carries a digest

**Scenario: Correction history changes while a mutation waits**

- **Given** an existing-change mutation blocked on the lifecycle project lock
- **When** the correction ledger becomes invalid before that mutation acquires the lock
- **Then** the mutation reloads and validates the ledger under lock, fails safely, and persists no lifecycle update

**Scenario: A draft is shown what it should honor**

- **Given** a draft whose affected module context holds substantive lessons, or a project that names a principles file
- **When** the rest of the interview is answered
- **Then** the remaining question is `constraints`, the answer "none" completes it, and the briefing names the lessons and the principles file
- **And** a selected plan whose required headings have no substantive text is incomplete, while a change that did not select a plan is not given one
- **And** a change that has left draft without a constraints answer is not returned to the interview

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails. A draft is told which answer finishes it: `specsync change answer <id> acceptance_criteria "<observable outcome>"`, `affected_paths "<path>"` or `affected_specs "<module>"`. A record past draft keeps the bare condition, because `change answer` refuses outside draft (REQ-change-133) |
| Interview questions still open at approval | Approval fails. A draft is told every open question and the `specsync change answer <id> <question> <answer>` command for the first. Two or more choices are the placeholder (`<yes|no>`), one choice is the answer itself (`constraints none`, with the option to name constraining lessons or principles instead), and no choice takes `"<answer>"`; a record past draft keeps the bare condition (REQ-change-133) |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Populated semantic delta with no recognized operation heading | Approval and historical validation name the allowed `## Added`, `## Modified`, and `## Removed` headings instead of reporting the file empty |
| Spec documents an export that is not in source | `change check` fails with the spec finding; configured test commands are not run |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current AND its verification commit is still anchored | Reopen is rejected without changing lifecycle or audit state |
| Accepted verification commit is unreachable and no reachable history records the acceptance | Reopen is admitted and records `VerificationCommitUnanchored`, even when delivery inputs are byte-identical |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes and no successor claims it | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every successor that claimed it was refused | Unified check names the input and each refused successor with its reason, sorted by ID; a workflow-v1 predecessor with a refused workflow-v2 successor is directed to finish that successor, never to `change reopen` |
| Archive preflight finds that the package being closed is the only successor covering a legacy change it supersedes | The preflight authenticates that package with its closing token and checks its succession tuples against the working tree its closing evidence signed; finalize succeeds and the predecessor is successor-covered before and after the archive commit |
| `change audit` judges an active legacy accepted change whose changed inputs a finalized (archived) successor covers | Archives are offered as successor candidates without being evaluated; the predecessor is reported successor-covered, and a refused archived successor is named with its reason |
| `change supersede` names a module for a predecessor entry signed under reserved exact owners that the module does not own now | The obligation is refused, naming the frozen label and the `[modules."<name>"] owns` remedy, and nothing is persisted |
| Changed exact-only delivery input that no successor claims | Unified check names the input, the audited-reopen remediation, and — wherever the configuration can grant the path — the supersede alternative |
| Changed exact-only delivery input whose claimants were all refused | Unified check names the input with its frozen label as owner and each refused claimant with its reason, exactly as for a signed owner's input |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |
| Effective definition approval records no semantic delta wording while an earlier definition approval in the same ledger recorded it | Materialization and acceptance refuse the withdrawn claim and name `specsync change approve <id>` as the remedy |
| Later files expand a previously accepted directory scope beyond the evidence payload bound | Recursive accepted-input validation captures only authenticated signed manifest entries; fresh acceptance and reopen keep full discovery and unchanged resource bounds, and signed input drift remains fail-closed |
| An active v2 finalization has a mismatched historical review workspace but its entire raw package matches the current remote-default tree | Reopen may record that exact Git anchor and downgrade to verifying after all other authentication checks; ordinary acceptance/archive remain strict and fresh verification plus scoped review are required |
| The mismatched package is modified, unanchored, not active v2, or has any other invalid finalization digest | Recovery refuses without changing lifecycle or evidence bytes |
| The exactly recorded mismatched old review also has an inconsistent review hash | Reopen-only recovery preserves both as untrusted at the same anchor; finalization self-digest and all non-review checks remain strict, and a pure review-hash mismatch remains refused |
| A workflow-version downgrade is committed and then restored, including when the project root is a subdirectory of the Git repository | Load, status, and a warm read cache refuse with `workflow-version history changed immutable identity`, and a downgrade that exists only on an earlier dated archive path is still seen after reopen and rearchive |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| hash_cache | Project SHA-256 dependency for content identity |
| parser | `strip_frontmatter` — the canonical frontmatter reader for lesson counting, archived lesson bundles, and artifact completeness |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_change | Complete lifecycle command surface |
| cmd_check | Unified SDD gate before canonical validation |
| cmd_init | New-project policy and version initialization |

## Change Log

| 2026-11-10 | v223: Ownership recovery follow-ups (#1011, #999 review): recognized reserved delivery metadata is no longer an approve-time ownership refusal (finalize already signs it `@exact:delivery`); owner recovery never prints a `change answer` command that cannot run as printed (comma/newline-bearing module names) or one built on a truncated spec listing, says when the 10,000-entry walk bound was hit, and renews the constraints answer that declaring owners clears; registry-registered canonical specs outside `specs_dir` join owner discovery; a declared directory scope is not file-level ownership-checked at approve (its descendants are checked at finalize). |

| 2026-11-10 | v221: Retirement custody validation stops re-running the custodian's live freshness and re-deriving historical receipt classifications on every load — the deadlock that made the prescribed `change reopen` remedy unreachable once a custodian drifted (#1021). Custody holds while every retained path stays under its recorded signed owners in the custodian's authenticated evidence — its current acceptance, or its last accepted manifest while the custodian is in an audited post-acceptance reopen, so the roster keeps loading through the recovery window. Owner or path loss still refuses, and custodian staleness stays terminal-evidence data on the custodian's own record. |

| 2026-11-09 | v217: Deadline-boundary review fixes (#1005): recheck the deadline after the final archive-introduction commit before caching, after the post-history HEAD reads, and count completed workspaces (including unreadable ones) in the final active-loading boundary (REQ-change-149). |

| 2026-11-09 | v216: Clippy `--all-targets` cleanup in the attributes reader and batch-output terminator check (`as_chunks::<3>()`, byte-string comparison); no behavior change (CI-8, #608, #1007 merge train). |

| 2026-11-09 | v213: Retirement review fixes (#1018): accept an archived-origin audited reopening (`from_state: archived`, the ordinary finalize-then-reopen path) and require the reopening to prove staleness; pin that a shallow clone cannot launder retirement custody (custody validation fails closed beyond the boundary and a locally deleted retired archive is still found through the boundary commit's receipt); require unchanged bytes when custody owners are reserved `@exact:*` labels; classify Retired records as historical in sequence accounting; a send-back at or after the latest reopening supersedes it in the frozen-conflict test; the default active audit validates pending retired records' custody. |

| 2026-11-09 | v210: Declare the existing recovery_retirement regression file in the change module coverage list; no runtime code, fixture, assertion or configuration change. |

| 2026-11-08 | v208: Explicit committed audited-reopening custody source and unchanged exact ordinary-file owner classification in schema v2, retaining v1 digest compatibility and all nonacceptance guards. |

| 2026-11-09 | v207: Explicit audited-reopened retirement with complete retained custody, committed-event immutability and old-reader fail-closed state; preserve frozen conflict refusal and allow fresh draft dependency answers. |

| 2026-07-30 | Add `audit_project`; scope CLI `change check` to one change; archives are history |

| Date | Change |
|------|--------|
| 2026-11-06 | v195: approve refuses a no-spec-change draft over owned production source and names the `change answer` command; ship preflight and ship-status run finalize's ownership check (#986, #956). |
| 2026-11-07 | v200: carry the selected root in `CannotTell` so command adapters remain unchanged within the original change-only scope. v199: keep the selected project root as a quoted `--root` argument in cannot-tell recovery; execute the printed status command from the repository parent (#967, REQ-change-142). |

| 2026-11-08 | v201: PR1014 succession compares lossless normalized approval/reopening prefixes and the first newly superseded approval; three committed decimal-edit regressions fail on the previous source and pass unchanged after the private-helper fix, with prior numeric and relay controls preserved (REQ-change-relay-compare-001). |
| 2026-11-06 | #878: the persistent read cache is reused only by the build that wrote it; its identity is the package version plus the build fingerprint, so a sibling worktree on other code at the same version recomputes its history verdicts (REQ-change-134). |
| 2026-11-06 | v200: Reject symlink components in late-close destinations during planning/dry-run, before any batch write under the lifecycle lock, and again after directory creation before each rename; include dangling links and name how to retry safely (REQ-change-143, #994 P1) |
| 2026-11-06 | v187: `specsync doctor`'s late close moves a merged but never-archived package byte for byte into `.specsync/archive/closed-late/<date>-<id>/` with `late-close.json` (`verified: false`); it reads no delivery evidence and adds no change state, so SpecSync 6.0 to 6.2 never read it. A collision member's acknowledgement is dropped and recorded. 6.3 reports a closed-late change in status and ship-status, refuses it in every lifecycle verb, and reserves its ID in `change new` (REQ-change-143, #833) |
| 2026-11-06 | v179: a send-back records how many review attempts preceded it, and a review recorded before the latest send-back never counts, even after an identical re-approval (#960, CHANGE-12.e). |
| 2026-11-06 | v190: the acceptance manifest and the acceptance input digest name clean delivery files by object id and hash each blob from one `git cat-file --batch` child, one blob at a time, under the per-object bound, so reopen, finalize and accepted-input validation no longer refuse a change whose clean affected files exceed one whole-scope read; recorded manifests and digests are byte-identical (REQ-change-144, #976). |
| 2026-11-06 | v198: the upstream-archive lookup reports `CannotTell` instead of no archive when the remote default's archived state records cannot be read, so review and finalize refuse and status and ship-status say how to find out (REQ-change-142, #946); the archive root is read as a literal pathspec from the top (#962). Codex review of #967: a record with no readable `id` is `CannotTell`; the recovery refetches, repairs `origin/HEAD`, and reruns `change status` with the id shell-quoted instead of grepping from the caller's directory. |
| 2026-11-06 | Codex review of #903: a workflow-v2 finalization stopped after acceptance, with its workspace still active, is gated too; status, ship-status and ship name its open task (`open_tasks_block`), and a review refusal names `--reviewer=<claim>` (REQ-change-131, REQ-cmd-change-031, #892). |
| 2026-11-05 | v174: check, verify, a passing review and finalize refuse a change with an open task directly, even one reopened after a passing check, and name each open task and the command to re-run; status names the open tasks before the next step (REQ-change-131, REQ-change-132, #892). |
| 2026-11-06 | v175: a passing legacy-baseline introduction proof is kept in the persistent read cache, so an archived read of one legacy archive no longer re-walks every introduction commit and legacy archive tree; a refusal is always re-derived (#901). |
| 2026-11-05 | v162: archived workflow-version history and the read-cache invalidation walk use `:(top)` pathspecs, so a project in a repository subdirectory still sees that history and still refuses a hidden workflow-version downgrade (#851). |
| 2026-11-05 | v162: lifecycle commands stay within a tested git spawn ceiling per command (REQ-change-127, #861). |
| 2026-11-01 | v150: the lesson bundle heading is `# Lesson bundle: <id>` and the bundle's fixed text adds no en or em dash; user record text is copied unchanged (REQ-change-116, #827). |
| 2026-10-10 | Add `change.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-10-10 | v141: lesson planes follow the order written, the lesson bundle names the 3md file, and a new `<module>.3md` is staged with the other spec companions. An unreadable 3md file falls back to `context.md`. |
| 2026-10-10 | v140: materialize the optional module.3md lesson requirement. |
| 2026-10-10 | v139: an optional `<module>.3md` file holds context on its first plane and lessons on later planes. `context.md` remains the lesson source until that file exists. |
| 2026-10-10 | v138: materialize the fence-closer and symlink requirements. |
| 2026-10-10 | v137: a closing fence is only whitespace after the marker, a marker indented more than three spaces is not a fence, and a symlink keeps a state-less directory open. |
| 2026-10-10 | v136: materialize the fenced-plan and damaged-directory requirements. |
| 2026-10-10 | v135: a fenced code block cannot complete a selected plan, and a change directory with files but no state.json stays open. |
| 2026-10-10 | v134: a change directory that holds a regular file but no `state.json` is still open; a husk with no regular file is not a change. |
| 2026-10-10 | v132: lesson paths follow the registry-aware spec directory; a plan section ends at the next heading and ignores HTML comments. |
| 2026-10-09 | v129: group private Git capture options to satisfy Clippy's argument limit without changing evidence behavior. |
| 2026-10-09 | v128: a clean object id is framed apart from file bytes. `filter=lfs` requires the Git LFS clean and smudge commands and a pointer blob. |
| 2026-10-08 | name-clean-git-files-by-object-id-so-change-check-accepts-an-lfs-pointer: Name clean Git files by object id so change check accepts an LFS pointer |
| 2026-09-01 | Scoped review may be recorded by the definition approver; GitHub remains merge authority. |
| 2026-08-30 | Fresh `init` writes SDD off; `require_change_for_meaningful_files` is false on new policies. `check` does not call `audit_project`. `change check` is in-process spec↔code sync and does not spawn `verification_commands`. |
| 2026-08-30 | v117: `change check` is scoped to the change. A declared module with no spec on disk fails by name instead of shrinking the scope to nothing; an empty scope stays a pass only for a change that claimed no contract. Evidence `--spec` names are the stems `filter_specs` matches. |
| 2026-08-30 | `change adopt` is the on-switch `init` points at: it flips `enabled` on a policy written off, fails closed on one it cannot parse, and re-pins the bootstrap record it invalidates. `change check` is scoped to the change; evidence names `--spec` and only advertises `--strict` when requested. |
| 2026-08-01 | Approve rejects ADDED existing living REQs; draft next_action prefers complete artifacts over approve when stubs remain. |
| 2026-07-10 | v4: normalize imported, evidence, and digest paths across Windows and Unix |
| 2026-07-10 | v3: make approval digests and detected verification commands portable across CI checkouts |
| 2026-07-10 | v2: compare meaningful path coverage with the current remote base after rebases |
| 2026-07-10 | Initial 5.0 verified SDD lifecycle |
| 2026-07-11 | CHG-0002-harden-specsync-5-0-lifecycle-safety-and-release-validation: Harden SpecSync 5.0 lifecycle safety and release validation |
| 2026-07-11 | CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation: Finalize SpecSync 5.0 release consistency and parallel validation |
| 2026-07-11 | CHG-0004-close-final-pr-review-gaps-in-5-0-lifecycle-enforcement: Close final PR review gaps in 5.0 lifecycle enforcement |
| 2026-07-11 | CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting: Close final fail-closed review gaps in 5.0 lifecycle evidence and PR reporting |
| 2026-07-11 | CHG-0006-close-final-specsync-5-0-evidence-monorepo-bootstrap-reporting-and-import-re: Close final SpecSync 5.0 evidence, monorepo, bootstrap, reporting, and import review gaps |
| 2026-07-11 | CHG-0007-harden-specsync-5-0-as-an-agent-native-secret-free-sdd-core-and-close-release-r: Harden SpecSync 5.0 as an agent-native, secret-free SDD core and close release regressions |
| 2026-07-11 | CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path: Make accepted evidence squash-safe and harden the 5.0 release path |
| 2026-07-13 | Add audited reopen and re-verification for stale accepted delivery evidence |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-13 | Preserve legacy and transitional definition evidence when canonical application state is false |
| 2026-07-13 | CHG-0016-reject-modified-definitions-when-reaccepting-an-already-applied-change: Reject modified definitions when reaccepting an already-applied change |
| 2026-07-13 | Normalize compatible transitional definition evidence during explicit acceptance for older contract checkers |
| 2026-07-13 | CHG-0017-allow-audited-reopen-after-squash-and-canonical-successors: Allow audited reopen after squash and canonical successors |
| 2026-07-13 | CHG-0018-allow-section-only-semantic-deltas-to-satisfy-verification-evidence: Allow section-only semantic deltas to satisfy verification evidence |
| 2026-07-13 | CHG-0020-harden-reopened-acceptance-compatibility-and-canonical-governance: Harden reopened acceptance compatibility and canonical governance |
| 2026-07-13 | CHG-0021-close-reopened-lifecycle-review-gaps: Close reopened lifecycle review gaps |
| 2026-07-13 | CHG-0022-preserve-canonical-change-log-table-schemas-when-accepting-semantic-deltas: Preserve canonical Change Log table schemas when accepting semantic deltas |
| 2026-07-13 | CHG-0023-allow-squash-accepted-evidence-on-descendant-branches: Allow squash-accepted evidence on descendant branches |
| 2026-07-14 | CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2: Stabilize SpecSync 5 lifecycle integrity and strict validation for 5.0.2 |
| 2026-07-14 | CHG-0025-address-all-unresolved-review-feedback-on-pr-366: Address all unresolved review feedback on PR 366 |
| 2026-07-14 | CHG-0026-keep-lifecycle-recursion-detection-private-while-preserving-deterministic-nested: Keep lifecycle recursion detection private while preserving deterministic nested-command failures |
| 2026-07-14 | CHG-0027-preserve-accepted-evidence-across-valid-later-sequence-claims: Preserve accepted evidence across valid later sequence claims |
| 2026-07-14 | CHG-0029-address-all-remaining-review-feedback-from-pr-366: Address all remaining review feedback from PR 366 |
| 2026-07-14 | CHG-0032-address-all-actionable-review-findings-on-pr-370-with-regression-coverage: Address all actionable review findings on PR 370 with regression coverage |
| 2026-07-14 | CHG-0033-close-final-5-0-2-lifecycle-review-and-intent-preservation-gaps: Close final 5.0.2 lifecycle review and intent-preservation gaps |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | Harden CHG-0040 trusted-history reference resolution and NUL-delimited Git path parsing during PR review |
| 2026-07-15 | Keep the CHG-0040 Unicode-path regression valid on Windows without dropping quoted-path coverage on Unix |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-16 | CHG-0044-harden-canonical-numeric-change-ordering-across-chg-9999-to-chg-10000-and-correc: Harden canonical numeric change ordering across CHG-9999 to CHG-10000 and correct 5.1 release documentation |
| 2026-07-16 | CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re: Unify local and CI verification freshness so descendant evidence-only commits remain current while source, test, configuration, contract, or nonancestor changes fail closed |
| 2026-07-17 | CHG-0049-make-stale-accepted-change-verification-diagnostics-actionable-with-named-delive: Make stale accepted-change verification diagnostics actionable with named delivery inputs and remediation |
| 2026-07-17 | CHG-0051-govern-the-deterministic-reconciliation-of-concurrent-accepted-chg-0048-sequence: Govern the deterministic reconciliation of concurrent accepted CHG-0048 sequence claims while preserving both immutable histories and the 5.1.1 release gate |
| 2026-07-17 | CHG-0052-allow-a-fully-valid-later-sequence-owner-to-preserve-historical-exact-ledger-evi: Allow a fully valid later sequence owner to preserve historical exact ledger evidence after an accepted collision reconciliation |
| 2026-07-17 | CHG-0053-permit-audited-reopened-collision-members-to-retain-immutable-sequence-history-s: Permit audited reopened collision members to retain immutable sequence-history status during re-verification |
| 2026-07-18 | CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged: Trust accepted-change evidence that is recorded in main history by squash-merged commits so accepted and archived changes whose verification and closing approval bytes match an in-history accepted record can be archived even when the original acceptance-transition commit was discarded by a squash merge |
| 2026-07-19 | CHG-0056-repair-archived-legacy-change-ledgers-whose-acceptance-inputs-include-production: Repair archived legacy change ledgers whose acceptance inputs include production source with no canonical owner by resolving unowned production source to the exact delivery owner during legacy acceptance-manifest reconstruction, so adoption-era archived records validate under current rules without per-repo remediation |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
| 2026-07-19 | CHG-0057-add-a-native-migration-path-for-5-0-1-era-change-ledgers-that-backfills-the-5-1: Add a native migration path for 5.0.1-era change ledgers that backfills the 5.1 reopening stale and current acceptance-input digest fields idempotently with a closing-digest verification pass, and surfaces an actionable migrate hint when check encounters the 5.0.1 reopening schema |
| 2026-07-19 | CHG-0059-tolerate-inert-5-0-1-registry-toml-stubs-so-module-resolution-falls-back-to-defa: Tolerate inert 5.0.1 registry.toml stubs so module resolution falls back to default specs layout without failing closed on empty legacy stubs |
| 2026-07-27 | CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi: Fix issue #467 by deduplicating identical stage-zero entries from overlapping Git pathspec batches while rejecting conflicting mode or object observations |
| 2026-07-29 | CHG-0068: Bind one human approval to stable scope while automated evidence tracks implementation and materialization changes |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-30 | CHG-0068: Freeze the truthful legacy scope adoption, enforce independent pass/block review evidence, commit-by-commit freshness, symmetric scope changes, and retryable post-move finalization |
| 2026-07-30 | CHG-0068: Fail closed on missing adoption anchors, bind append-only review attempts to authenticated hosted checks, recover partial archive transactions, share freshness bounds, and authenticate squash-surviving v2 archives |
| 2026-07-30 | CHG-0068: Bind legacy workflow eligibility to an immutable pre-v2 project cutoff so first-reachable records cannot downgrade by omitting both version fields |
| 2026-07-30 | CHG-0068 review hardening: Make the pre-v2 cutoff squash-stable and preserve explicitly anchored workflow-v1 records |
| 2026-07-30 | CHG-0068 sandbox hardening: Suppress raw Git diagnostics from expected missing-history status probes |
| 2026-07-30 | CHG-0068 sandbox hardening: Preserve exact committed collision-owner sequence ledgers when later workflow-v2 claims advance the current ledger |
| 2026-07-30 | CHG-0068 adversarial hardening: Follow immutable workflow-origin history across cross-date rearchives |
| 2026-07-30 | CHG-0068 review hardening: Reject workflow-v2 baseline rewrite-then-restore history |
| 2026-07-30 | CHG-0068 review hardening: Preserve exact review-only children while enforcing every-parent verification freshness and persisted reviewer independence at native review/finalization mutations |
| 2026-07-30 | CHG-0068 sandbox hardening: Atomically and path-safely activate workflow v2 without rewriting or stranding existing workflow-v1 evidence, and fail closed on every-parent baseline deletion |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-08-01 | CHG-0072-heal-reopen-closing-approval-recovery-for-stale-accepted-evidence: Heal reopen closing-approval recovery for stale accepted evidence |
| 2026-08-01 | CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa: Approve rejects living ADDED REQs and draft next_action waits on complete artifacts |
| 2026-08-03 | CHG-0080-fail-lifecycle-verification-before-running-the-suite-when-evidence-is-incomplete: Fail lifecycle verification before running the suite when evidence is incomplete, make already-applied ADDED deltas converge, and reject duplicate change ordinals from one base |
| 2026-08-04 | CHG-0081-make-a-fresh-project-usable-out-of-the-box-stop-a-leftover-directory-from-block: Make a fresh project usable out of the box, stop a leftover directory from blocking change new, and extract a lock-free verification body |
| 2026-08-05 | CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change: Let finalize work in a repository that has archived a change |
| 2026-08-05 | CHG-0084-give-the-change-module-canonical-ownership-of-its-cli-wiring: Give the change module canonical ownership of its CLI wiring |
| 2026-08-05 | CHG-0085-resolve-canonical-ownership-at-approve-and-free-never-closed-changes: Resolve canonical ownership at approve and free never-closed changes |
| 2026-08-05 | CHG-0086-return-src-commands-change-rs-to-its-sole-canonical-owner: Return src/commands/change.rs to its sole canonical owner |
| 2026-08-07 | CHG-0090-harden-approve-ownership-skips-and-correct-owner-provenance-comments: Harden approve ownership skips and correct-owner provenance comments |
| 2026-08-07 | CHG-0094-count-same-pr-archived-changes-toward-path-coverage-after-finalize: Count same-PR archived packages toward path coverage after finalize |
| 2026-08-07 | CHG-0095-reject-hash-todo-artifact-headings-at-approve: Reject hash TODO artifact headings at approve |
| 2026-08-07 | CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base: Floor change sequences from remote ledger and document multi-clone BASE |
| 2026-08-07 | CHG-0094-count-same-pr-archived-changes-toward-path-coverage-after-finalize: Count same-PR archived changes toward path coverage after finalize |
| 2026-08-07 | CHG-0095-reject-hash-todo-artifact-headings-at-approve: Reject hash TODO artifact headings at approve |
| 2026-08-07 | CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base: Floor change sequences from remote ledger and document multi-clone BASE |
| 2026-08-07 | CHG-0096-floor-change-sequences-from-remote-ledger-and-document-multi-clone-base: Floor change sequences from remote ledger and document multi-clone BASE |
| 2026-08-08 | CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid: Fail closed in text lifecycle views when a correction ledger is invalid |
| 2026-08-10 | CHG-0103: Validate correction-ledger health inside locked existing-change definition mutations |
| 2026-08-10 | CHG-0103: Keep documented mutation wrappers in production and capture normal/strict machine summaries inside the locked transaction |
| 2026-08-12 | CHG-0104-sever-specsync-check-and-comment-from-the-trust-layer-lifecycle-state-becomes-i: Sever specsync check and comment from the trust layer: lifecycle state becomes informational and never affects exit status |
| 2026-08-12 | CHG-0106-make-verification-currency-a-content-question-delete-the-git-ancestry-walk-the: Make verification currency a content question: delete the git-ancestry walk, the REQ-change-016 persistence allowlist, and the verification-commit ancestry binding |
| 2026-08-13 | CHG-0107-fix-the-first-five-minutes-of-spec-sync-init-leaves-a-repo-that-fails-check-sc: Fix the first five minutes of spec-sync: init leaves a repo that fails check, scaffold writes prose that check rejects, and a directory in files: makes check silently green |
| 2026-08-13 | CHG-0108-stop-reporting-success-for-checks-that-did-not-happen-gate-drafts-that-document: Stop reporting success for checks that did not happen: gate drafts that document a contract over present source, drop cold-cache drift noise, and stop taking quoted frontmatter paths literally |
| 2026-08-13 | CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be: A semantic delta section body may contain subheadings so scaffolded specs can be changed |
| 2026-08-16 | CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu: Extract the change module's tests into their own file so the file that manufactures the sibling-site defect can be read, without altering a single test |
| 2026-08-16 | CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive: A refused reopen must restore the archive it un-archived, because the un-archive move happens before the preconditions are checked and a correct refusal was destroying the package |
| 2026-08-17 | CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent: An unreadable change workspace must be reported, not counted as absent |
| 2026-08-17 | CHG-0139-declaring-a-module-must-never-reduce-the-verification-a-change-receives: Declaring a module must never reduce the verification a change receives |
| 2026-08-17 | CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards: A stale sequence ledger must not be committed backwards |
| 2026-08-18 | CHG-0143-the-sequence-ledger-gate-must-judge-a-branch-by-its-own-history-not-by-origin: The sequence ledger gate must judge a branch by its own history, not by origin |
| 2026-08-18 | CHG-0148-a-reopened-change-must-be-closeable-again: A reopened change must be closeable again |
| 2026-08-18 | CHG-0149-an-archived-change-package-must-not-leave-an-untrackable-husk: An archived change package must not leave an untrackable husk |
| 2026-08-18 | CHG-0152-a-populated-semantic-delta-must-not-report-as-empty: A populated semantic delta must not report as empty |
| 2026-08-19 | CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar: Ship-status must name the action the lifecycle state requires, and resolve an archived change's evidence |
| 2026-08-19 | CHG-0154-one-git-config-read-instead-of-four-for-effective-checkout-overrides: One git config read instead of four for effective checkout overrides |
| 2026-08-19 | CHG-0155-the-batched-config-read-must-not-overflow-the-bound-sized-for-a-single-key: The batched config read must not overflow the bound sized for a single key |
| 2026-08-19 | CHG-0156-the-reopen-then-close-guard-must-be-pinned-by-tests-not-only-by-a-drill: The reopen-then-close guard must be pinned by tests, not only by a drill |
| 2026-08-19 | CHG-0157-a-newer-six-must-be-readable-by-an-older-six: A newer six must be readable by an older six |
| 2026-08-19 | CHG-0158-the-forward-compatibility-valve-must-be-true-everywhere-it-is-claimed: The forward-compatibility valve must be true everywhere it is claimed |
| 2026-08-19 | CHG-0159-identity-must-come-from-state-json-never-from-the-shape-of-a-name: Identity must come from state.json, never from the shape of a name |
| 2026-08-20 | CHG-0160-succession-must-be-ordered-by-when-a-change-happened-not-by-how-it-is-named: Succession must be ordered by when a change happened, not by how it is named |
| 2026-08-20 | CHG-0161-a-slug-must-be-a-legal-directory-name-on-every-platform-we-ship: A slug must be a legal directory name on every platform we ship |
| 2026-08-20 | CHG-0162-a-change-identity-must-be-validated-for-what-it-is-not-for-how-it-starts: A change identity must be validated for what it is, not for how it starts |
| 2026-08-20 | CHG-0163-a-trust-anchor-must-be-where-evidence-entered-history-not-any-commit-that-re-introduces-it: A trust anchor must be where evidence entered history, not any commit that re-introduces it |
| 2026-08-20 | retire-the-ordinal-and-keep-the-ledger-readable-forever: Retire the ordinal and keep the ledger readable forever |
| 2026-08-21 | a-reopen-must-extend-the-committed-ledger-not-merely-count-itself: A reopen must extend the committed ledger, not merely count itself |
| 2026-08-22 | an-orphaned-verification-commit-must-be-reopenable: An orphaned verification commit must be reopenable |
| 2026-08-23 | ship-readiness-is-a-content-question-not-a-history-one: Ship readiness is a content question, not a history one |
| 2026-08-24 | close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails: Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize |
| 2026-08-24 | frontmatter-stripping-and-scaffold-detection-must-survive-crlf-and-an-unexpanded-module-placeholder: Frontmatter stripping and scaffold detection must survive CRLF and an unexpanded module placeholder |
| 2026-08-24 | a-subheading-inside-a-delta-item-must-not-flush-the-item-content-headings-split-one-section-into-fragments-and-the-last: A subheading inside a delta item must not flush the item: content headings split one section into fragments and the last one wins |
| 2026-08-24 | bind-semantic-delta-bodies-to-the-approval-that-signed-them: Bind semantic delta bodies to the approval that signed them |
| 2026-08-25 | one-canonical-frontmatter-reader-for-crlf-checkouts: One canonical frontmatter reader for CRLF checkouts |
| 2026-08-25 | req-change-016-must-describe-ancestry-as-it-is-used-never-a-gate-on-currency-or-ship-readiness-admissible-as-one-basis: REQ-change-016 must describe ancestry as it is used: never a gate on currency or ship readiness, admissible as one basis for archival anchoring |
| 2026-08-27 | drop-the-windows-binary-from-the-6-0-release-matrix-and-stop-claiming-windows-support-while-keeping-every-windows: Drop the Windows binary from the 6.0 release matrix and stop claiming Windows support, while keeping every Windows-content correctness guarantee |
| 2026-08-27 | a-definition-approval-may-not-withdraw-the-delta-binding-an-earlier-approval-recorded: A definition approval may not withdraw the delta binding an earlier approval recorded |
| 2026-08-27 | retire-the-spec-text-describing-the-deleted-change-sequence-allocation-including-specsync-sequence-base: Retire the spec text describing the deleted change-sequence allocation, including SPECSYNC_SEQUENCE_BASE |
| 2026-08-27 | one-delimiter-rule-for-every-frontmatter-reader-at-both-ends-of-the-block: One delimiter rule for every frontmatter reader, at both ends of the block |
| 2026-08-27 | hash-the-semantic-delta-binding-over-line-ending-canonical-bytes-so-a-crlf-checkout-of-an-unedited-delta-stops-failing: Hash the semantic delta binding over line-ending-canonical bytes so a CRLF checkout of an unedited delta stops failing the approval gate, and fold nothing else |
| 2026-08-27 | name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group: Name a held Cargo build-directory lock before verification blocks on it, and reap the verification child's process group |
| 2026-08-29 | decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after: Decide canonical materialization from the artefacts instead of the canonical_applied flag so a delta corrected after review and re-approved reaches the canonical spec with its version bump and Change Log row |
| 2026-08-29 | ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell: Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell |
| 2026-09-02 | allow-the-scope-approver-to-record-scoped-review-so-solo-projects-can-ship-when-github-does-not-require-a-second: Allow the scope approver to record scoped review so solo projects can ship when GitHub does not require a second reviewer. |
| 2026-08-30 | make-check-the-product-and-stop-change-check-from-spawning-project-tests: Make check the product and stop change check from spawning project tests |
| 2026-09-02 | tell-agents-when-it-is-safe-to-clear-context: Tell agents when it is safe to clear context |
| 2026-09-05 | allow-audited-reopening-when-legacy-acceptance-cannot-be-reconstructed: Allow audited reopening when legacy acceptance cannot be reconstructed |
| 2026-09-05 | archive-preflight-lets-the-package-being-closed-cover-the-legacy-change-it-supersedes-and-stale-input-diagnostics-name: Archive preflight lets the package being closed cover the legacy change it supersedes, and stale-input diagnostics name the refused successor |
| 2026-09-07 | let-a-module-own-paths-beyond-its-spec-files-so-a-later-change-can-supersede-the-exact-only-inputs-of-an-archived: Let a module own paths beyond its spec files so a later change can supersede the exact-only inputs of an archived bootstrap change |
| 2026-09-07 | make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling: Make bounded Git timeout cleanup tests independent of child startup scheduling |
| 2026-09-08 | complete-specsync-6-promotion-and-public-release-contracts: Clarify scoped reviewer claims and the separate authentication policy boundary |
| 2026-09-09 | close-the-specsync-6-0-0-p1-release-defects-found-in-overnight-proving: Close the SpecSync 6.0.0 P1 release defects found in overnight proving |
| 2026-09-09 | close-remaining-specsync-6-0-0-first-user-p1s-pre-commit-honors-config-toml-config-fail-closed-merge-git-sanitization: Close remaining SpecSync 6.0.0 first-user P1s: pre-commit honors config, TOML config fail-closed, merge git sanitization, and 5.x upgrade docs |
| 2026-09-09 | drop-interpolated-next-action-from-v1-verifying-assert-messages-so-codeql-cleartext-logging-is-not-a-required-check: Drop interpolated next_action from v1 verifying assert messages so CodeQL cleartext-logging is not a required-check failure |
| 2026-09-26 | lifecycle-commits-stage-only-what-the-change-owns-never-every-untracked-file: Lifecycle commits stage only what the change owns, never every untracked file |
| 2026-10-10 | show-an-opened-change-the-lessons-and-principles-it-should-honor-and-require-its-plan-to-say-how: Show an opened change the lessons and principles it should honor, and require its plan to say how |
| 2026-10-10 | keep-approval-tests-valid-after-a-plan-must-name-its-sections: Keep approval tests valid after a plan must name its sections |
| 2026-10-10 | address-every-review-comment-on-the-archive-and-pull-request-commands: Address every review comment on the archive and pull request commands |
| 2026-10-10 | a change directory with files and no state is still open |
| 2026-10-10 | refuse-a-fenced-plan-and-keep-a-damaged-change-directory-open: Refuse a fenced plan and keep a damaged change directory open |
| 2026-10-10 | close-a-fence-only-on-whitespace-and-count-a-symlink-as-a-file: Close a fence only on whitespace and count a symlink as a file |
| 2026-10-10 | show-lessons-from-an-optional-module-3md-file: Show lessons from an optional module 3md file |
| 2026-10-10 | read-3md-lessons-in-file-order-and-stage-the-file: Read 3md lessons in file order and stage the file |
| 2026-10-10 | expect-a-module-3md-file-in-the-lifecycle-commit-scope: Expect a module 3md file in the lifecycle commit scope |
| 2026-10-10 | refuse-a-broken-policy-and-commit-the-archive-move: Refuse a broken policy and commit the archive move |
| 2026-10-10 | parse-a-broken-policy-beside-a-baseline-and-retry-the-archive-commit: Parse a broken policy beside a baseline and retry the archive commit |
| 2026-10-11 | split-source-files-so-none-is-longer-than-5-000-lines: Split source files so none is longer than 5,000 lines |
| 2026-10-23 | authenticate-finalized-policy-migrations-during-active-only-audit-without-weakening-policy-change-coverage: Authenticate finalized policy migrations during active-only audit without weakening policy-change coverage |
| 2026-10-31 | report-an-approval-as-invalid-in-change-status-when-an-approved-semantic-delta-body-is-edited: Report an approval as invalid in change status when an approved semantic delta body is edited |
| 2026-10-31 | run-the-effective-contract-gate-on-the-first-change-check-and-record-its-refusal-instead-of-leaving-a-stale-pass: Run the effective contract gate on the first change check and record its refusal instead of leaving a stale pass current |
| 2026-11-01 | write-generated-markdown-and-hook-files-without-em-or-en-dashes-while-still-recognizing-the-managed-markers-earlier: Write generated Markdown and hook files without em or en dashes while still recognizing the managed markers earlier versions wrote |
| 2026-11-01 | report-the-requirement-ids-an-approved-semantic-delta-declares-per-module-in-change-status-and-show-json: Report the requirement IDs an approved semantic delta declares, per module, in change status and show JSON |
| 2026-11-01 | let-an-opt-in-bounded-verification-ignored-paths-policy-list-keep-another-tool-s-record-directory-out-of-the: Let an opt-in bounded verification_ignored_paths policy list keep another tool's record directory out of the verification project-input digest |
| 2026-11-01 | keep-receipt-reads-bounded-and-recover-exactly-recorded-review-mismatches: Keep receipt reads bounded and recover exactly recorded review mismatches |
| 2026-11-01 | retain-compound-historical-review-inconsistencies-during-anchored-reopen: Retain compound historical review inconsistencies during anchored reopen |
| 2026-11-01 | acknowledge-anchored-historical-receipts-explicitly-without-weakening-normal-transition-proof: Acknowledge anchored historical receipts explicitly without weakening normal transition proof |
| 2026-11-04 | answer-archived-change-status-change-list-and-change-status-within-2-seconds-on-raven: Answer archived change status, change list and change status within 2 seconds on Raven |
| 2026-11-04 | finalize-a-change-in-seconds-on-raven-however-many-archives-it-holds: Finalize a change in seconds on Raven however many archives it holds |
| 2026-11-04 | name-the-spec-to-edit-when-change-approve-refuses-an-unowned-affected-path: Name the spec to edit when change approve refuses an unowned affected path |
| 2026-11-04 | make-the-unowned-path-ownership-hint-deterministic-and-fall-back-at-depth-zero: Make the unowned-path ownership hint deterministic and fall back at depth zero |
| 2026-11-04 | refuse-a-stale-modified-delta-instead-of-overwriting-newer-canonical-text: Refuse a stale modified delta instead of overwriting newer canonical text |
| 2026-11-04 | send-an-approved-change-back-to-draft-for-re-scoping: Send an approved change back to draft for re-scoping |
| 2026-11-05 | finalize-validates-only-the-change-it-archives: Finalize validates only the change it archives |
| 2026-11-05 | Never wait on a descendant's open pipe after a Git child exits, so `change status` cannot be stalled by that pipe (#854, #855, SPEED-1, SPEED-1.a) |
| 2026-11-05 | After the child-exit signal, drain one finite snapshot of queued Git bytes, including on the non-Unix bounded reader |
| 2026-11-05 | stop-waiting-on-a-descendant-pipe-after-the-git-child-exits: Stop waiting on a descendant pipe after the git child exits |
| 2026-11-05 | restore-archived-history-reads-for-a-project-in-a-repository-subdirectory: Restore archived history reads for a project in a repository subdirectory |
| 2026-11-05 | v162 / #862: `change audit` refuses a duplicated requirement ID and names it |
| 2026-11-05 | merge-two-finalized-changes-to-one-module-without-a-spec-conflict: Merge two finalized changes to one module without a spec conflict |
| 2026-11-05 | cut-git-process-spawns-per-specsync-command-and-run-the-release-candidate-suite-with-cargo-nextest-so-macos: Cut git process spawns per SpecSync command and run the release-candidate suite with cargo nextest so macOS qualification fits 45 minutes |
| 2026-11-06 | #861's spawn-ceiling requirement is REQ-change-127. Its archived package names it REQ-change-125 because #862 took 125 first; the archive is left as reviewed |
| 2026-11-06 | let-change-check-commit-a-project-rooted-in-a-repository-subdirectory: Let change check commit a project rooted in a repository subdirectory |
| 2026-11-06 | refuse-review-status-and-finalize-of-a-change-already-archived-on-the-remote-default-and-name-it: Refuse review status and finalize of a change already archived on the remote default and name it |
| 2026-11-06 | #694's upstream-archive requirements are REQ-change-129 and REQ-cmd-change-029. Its archived package names them REQ-change-127 and REQ-cmd-change-028 because #861 took those first; the archive is left as reviewed |
| 2026-11-06 | find-an-upstream-archive-by-the-id-in-its-state-record-rather-than-its-directory-name: Find an upstream archive by the id in its state record rather than its directory name |
| 2026-11-06 | reuse-a-cached-validation-snapshot-or-read-cache-verdict-only-in-the-build-that-wrote-it-so-a-stale-pass-cannot-hide-a: Reuse a cached validation snapshot or read-cache verdict only in the build that wrote it, so a stale pass cannot hide a new refusal (#878) |
| 2026-11-06 | refuse-check-review-finalize-and-ship-while-a-change-has-an-open-task-naming-each-open-task-and-the-command-to-re-run: Refuse check, review, finalize and ship while a change has an open task, naming each open task and the command to re-run (#892) |
| 2026-11-06 | report-a-not-yet-handoff-for-an-archive-that-fails-its-integrity-check-and-name-the-git-restore-that-recovers-it: Report a not-yet handoff for an archive that fails its integrity check and name the git restore that recovers it |
| 2026-11-06 | retire-the-contract-clauses-that-promise-change-check-runs-configured-verification-commands: Retire the contract clauses that promise change check runs configured verification commands |
| 2026-11-06 | v172 / #883: approve's missing-acceptance-criterion and unfinished-interview refusals name the `specsync change answer` command that finishes the definition; the guard refuses exactly what it refused before (REQ-change-133) |
| 2026-11-06 | show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve: Show the steps between change new and approve on the landing page and name the change answer command in approve refusals (#883, D11) |
| 2026-11-06 | let-a-change-reviewed-before-its-send-back-be-reviewed-again-after-re-approval: Let a change reviewed before its send-back be reviewed again after re-approval |
| 2026-11-06 | v177 / #917: approve's refusal hints are draft-only (a record past draft keeps the bare condition, since `change answer` refuses outside draft); the affected-path and affected-spec refusals name their `change answer` commands; a question with any choices shows them as the placeholder (REQ-change-133) |
| 2026-11-06 | keep-approve-refusal-hints-to-drafts-cover-the-scope-questions-show-single-choice-placeholders-and-commit-the-quick: Keep approve refusal hints to drafts, cover the scope questions, show single-choice placeholders, and commit the Quick Start source (#917) |
| 2026-11-06 | S9: one `diff-tree --stdin` for the archive-introduction index (REQ-change-135); the workflow-v1 cutoff check comes from the cutoff tree and is cached by the cutoff commit (REQ-change-136); read-only commands take a deadline that reports the phase and resume command (REQ-change-137) |
| 2026-11-06 | archived-change-status-no-longer-walks-history-per-archive-or-per-workflow-v1-change-and-read-commands-take-a-timeout: Archived change status no longer walks history per archive or per workflow-v1 change, and read commands take a timeout that names the phase and how to resume |
| 2026-11-06 | require-a-fresh-review-after-a-send-back-even-when-the-scope-comes-back-unchanged: Require a fresh review after a send-back even when the scope comes back unchanged |
| 2026-11-06 | a-cold-archived-change-status-answers-every-archive-s-workflow-version-history-from-one-history-walk-and-the-command-s: A cold archived change status answers every archive's workflow-version history from one history walk and the command's batch reader |
| 2026-11-06 | record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication: Record relayed human decisions on approvals and reviews as claims, not authentication |
| 2026-11-06 | hash-a-change-s-clean-delivery-files-one-blob-at-a-time-so-reopen-does-not-overflow-a-single-git-read-on-large-affected: Hash a change's clean delivery files one blob at a time so reopen does not overflow a single Git read on large affected paths |
| 2026-11-06 | the-workflow-v1-cutoff-check-reports-an-unreadable-archive-tree-as-a-read-failure-not-as-an-absent-change: The workflow-v1 cutoff check reports an unreadable archive tree as a read failure, not as an absent change |
| 2026-11-07 | report-production-source-no-declared-module-owns-before-approval-and-review-and-never-call-such-a-change-ready-to: Report production source no declared module owns before approval and review, and never call such a change ready to finalize |
| 2026-11-06 | specsync-s-own-git-history-reads-ignore-git-replace-ref-base-and-git-no-replace-objects-as-they-already-ignore-git: SpecSync's own Git history reads ignore GIT_REPLACE_REF_BASE and GIT_NO_REPLACE_OBJECTS as they already ignore GIT_GRAFT_FILE |
| 2026-11-06 | a-warm-status-of-an-archived-change-does-not-digest-the-whole-working-tree-to-report-its-review-currency: A warm status of an archived change does not digest the whole working tree to report its review currency |
| 2026-11-06 | repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed: Repair a damaged relayed decision with one command, whether or not it is committed |
| 2026-11-06 | stop-review-and-finalize-when-the-upstream-archive-cannot-be-read-instead-of-reading-it-as-absent: Stop review and finalize when the upstream archive cannot be read instead of reading it as absent |
| 2026-11-06 | add-specsync-doctor-to-report-every-change-problem-with-its-fix-and-close-merged-changes-late-where-older-specsync: Add specsync doctor to report every change problem with its fix and close merged changes late where older SpecSync never reads |
| 2026-11-06 | keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze: Keep the upstream-archive status guidance behind a sequence-ledger freeze |
| 2026-11-07 | print-a-single-choice-interview-answer-literally-in-the-approval-refusal-so-the-named-change-answer-command-runs-as: Print a single-choice interview answer literally in the approval refusal, so the named change answer command runs as printed (#958) |
| 2026-11-07 | leave-relayed-decisions-out-of-approval-succession-the-definition-digest-and-the-trusted-correction-history: Leave relayed decisions out of approval succession, the definition digest and the trusted correction history |
| 2026-11-07 | run-clippy-on-every-target-in-ci-and-the-verify-lane-and-clear-the-test-code-lints-it-finds: Run clippy on every target in CI and the verify lane, and clear the test-code lints it finds |
| 2026-11-07 | read-deadlines-never-panic-on-a-huge-timeout-and-stop-at-every-long-read-phase-including-active-change-loading-and: Read deadlines never panic on a huge timeout and stop at every long read phase, including active-change loading and before the bulk archive listing |
| 2026-11-08 | v199 / PR1014 review: preserve a reopening's own unknown fields, compare and deduplicate accepted approval ledgers without approval relays, and validate both embedded correction deciders before relay projection; existing relay requirements and public API unchanged |
| 2026-11-08 | v200 / PR1014 numeric review: project validated raw JSON spans without rounding unknown numbers or dropping duplicate unknown members; retain scalar types, harmless wire equivalence and the existing approval-only relay positions |
| 2026-11-09 | retire-obsolete-reopened-obligations-without-acceptance-and-diagnose-frozen-canonical-conflicts: Retire obsolete reopened obligations without acceptance and diagnose frozen canonical conflicts |
| 2026-11-09 | the-corrupt-archive-recovery-step-names-summary-terminal-evidence-reason-the-field-change-status-json-actually-prints: The corrupt-archive recovery step names summary.terminal_evidence.reason, the field change status --json actually prints (#966) |
| 2026-11-09 | the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id: The workflow-v1 cutoff check finds an archive by the record it holds, not only by a directory name ending in its id |
| 2026-11-10 | keep-retired-custody-valid-across-audited-custodian-drift-so-the-recorded-reopen-remedy-stays-reachable: Keep retired custody valid across audited custodian drift so the recorded reopen remedy stays reachable |
| 2026-11-10 | fix-the-ownership-recovery-follow-ups-from-the-999-review-comma-module-names-reserved-delivery-paths-at-no-spec: Fix the ownership recovery follow-ups from the #999 review: comma module names, reserved delivery paths at no-spec approval, listing-bound silence, constraints renewal, registry specs, and directory scopes |
| 2026-11-11 | journal-error-names-its-file-and-recovery: Journal error names its file and recovery |
| 2026-11-11 | journal-read-errors-name-the-file-and-a-quoted-recovery: Journal read errors name the file and a quoted recovery |
| 2026-11-11 | delta-materializer-matches-and-keeps-titled-requirement-headers: Delta materializer matches and keeps titled requirement headers |
| 2026-11-11 | a-failed-verification-records-the-findings-that-failed-it: A failed verification records the findings that failed it |
| 2026-11-11 | inert-ignored-paths-disclosed-at-policy-load: Inert ignored paths disclosed at policy load |
| 2026-11-11 | a-refused-check-records-its-refusal: A refused check records its refusal |
| 2026-11-11 | the-ownership-recovery-narrows-a-scope-over-files-the-change-never-touched: The ownership recovery narrows a scope over files the change never touched |
| 2026-11-10 | change-answer-validates-question-keys-answers-and-actor-identity: reject unknown interview question keys, empty answers, unregistered affected specs, and malformed actor identities (#426) |
| 2026-11-10 | every-actor-accepting-verb-normalizes-identity-the-same-way: Every actor-accepting verb normalizes identity the same way |
| 2026-11-11 | identity-and-answer-validation-keeps-relays-readable-covers-every-actor-verb-and-refuses-empty-lists: Identity and answer validation keeps relays readable, covers every actor verb and refuses empty lists |
| 2026-11-11 | answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested: Answer and actor validation round five: recoveries run as printed, exact none, every actor verb tested |
| 2026-11-11 | a-lifecycle-rewrite-keeps-the-fields-a-newer-specsync-wrote: A lifecycle rewrite keeps the fields a newer specsync wrote |
| 2026-11-11 | after-approve-next-action-names-check-commit-and-review-and-ship-refuse-a-verification-commit-that-lacks-the-change-s: After approve, next_action names check --commit, and review and ship refuse a verification commit that lacks the change's work |
| 2026-11-12 | record-a-change-that-holds-a-relayed-decision-at-workflow-version-3-so-released-specsync-6-2-0-refuses-it-instead-of: Record a change that holds a relayed decision at workflow version 3 so released SpecSync 6.2.0 refuses it instead of dropping relayed_by |
