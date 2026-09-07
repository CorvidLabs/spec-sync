Warning: truncated output (original token count: 18302)
Total output lines: 482

---
module: change
version: 120
status: active
files:
  - src/change.rs
  - src/change_tests.rs
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
---

# Change

## Purpose

Provides the SpecSync verified spec-driven development lifecycle: one scope approval, targeted verification, one independent scoped review, same-PR finalization, and compatible audited recovery for historical evidence.

## Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied, and no later definition approval may withdraw a delta binding an earlier one recorded.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. `change check` compares THIS change's specs to source in-process and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command. Scope is the modules in `affected_specs` union the specs whose `files:` fall inside `affected_paths`, so a `--no-spec-change` delivery still verifies against the contracts its source can break; drift outside that scope belongs to `specsync check`. A declared module that resolves to no spec file on disk FAILS verification and is named — never silently dropped, even when other specs are in path scope and would make the pass look real. An empty scope is a PASS only for a change that declared no module and maps no spec. Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`, where each name is what `filter_specs` matches (the file stem with `.spec` removed) and `--strict` appears only when it was requested.
6. Verification and scoped-review evidence bind the implementation commit and governed inputs; a scoped review records an explicit pass/block verdict, may be recorded by the same actor as the definition approver, and stays fresh only when every descendant/parent edge changes supported lifecycle persistence. GitHub remains the merge authority for required reviewers.
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

## Public API

**Exported Constants**

| Name | Description |
|------|-------------|
| `LESSON_BUNDLE_FILE` | Filename of the lesson bundle written into an archive, shared with the command layer so the two cannot name different files |
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
| `ApprovalRecord` | Actor, timestamp, gate, digest, optional note, optional backward-readable portable-pair metadata, and the optional per-module semantic delta body digests a definition gate approved |
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
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, stale/current input digests, and the staleness cause when the digests are equal |
| `ReopenCauseV1` | Why accepted evidence was stale despite matching inputs: unanchored verification or unreconstructible legacy acceptance; absent means inputs drifted |
| `recorded_verification_is_current` | Whether a change's recorded verification still matches the tree and plan on disk, as a content question; missing or unreadable evidence is not current |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `ReopenBackfillReport` | Per-change repair, skip, and failure detail for a `migrate 5.0` ledger backfill |
| `CorrectionField` | Closed supported accepted-metadata field set: public contract and architecture risk |
| `CorrectionRecord` | Immutable sequenced metadata correction with original/effective values, actor, reason, artifacts, prior evidence, and portable digest chain |
| `EffectiveChangeDefinition` | Validated projection of original answers/artifacts plus ordered corrections |
| `CorrectionResult` | Deterministic corrected change, event, effective definition, history, and gate-summary projection |
| `DefinitionMutationResult` | Crate-private successful definition mutation plus the effective definition, correction history, and normal/strict summaries validated inside its persistence transaction |
| `ApprovalLedger` | Ordered portable approval, allowlisted scope-adoption, and reopen history |
| `CommandEvidence` | Evidence for one verification step (in-process spec↔code sync) |
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
| `ScopedReviewCurrency` | Three-valued answer to whether a recorded scoped review still holds: current, stale carrying what moved, or unavailable when the guarantee could not be evaluated at all |
| `reason` | Why a scoped review is not current, for the callers that render a blocker or a warning |
| `FinalizationRecord` | Automated non-approval evidence binding implementation commit/tree, contract/workspace/closing/review digests, archive identity, and a domain-separated finalization digest |
| `ChangeReadScope` | Crate-private invocation guard that owns one bounded read-only lifecycle snapshot |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `TerminalEvidenceValidity` | State-aware exact, successor-covered, stale, authenticated-history, or corrupt-history evidence conclusion |
| `TerminalEvidenceSummary` | Shared terminal validity plus optional fail-closed reason |
| `TerminalEvidenceResult` | Change ID paired with its shared terminal-evidence summary |
| `HandoffReadiness` | `Safe`, `Conditional`, or `NotYet`: whether clearing context now loses anything the lifecycle has not recorded; serialized kebab-case, printed as `safe` / `conditional` / `not yet` |
| `HandoffSummary` | Readiness, a plain-language reason, the `specsync change status <id>` resume command, and the steps to take before clearing; carries no digest |
| `HandoffSignals` | The complete, digest-free input to `classify_handoff`: state, workflow version, sequence-ledger freeze, open questions, artifact completeness, approval/correction validity, uncommitted scoped edits, verification/review currency, and stale legacy terminal evidence |
| `ChangeSummary` | Human/agent status projection with approval health/current scope digest, plain-language material expansion, validator plan, scoped-review freshness, exactly one next action, a handoff verdict, and optional terminal evidence |
| `SddCheckReport` | Unified lifecycle errors, warnings, checked-change count, and terminal-evidence results |
| `UnreadableChange` | One active-change workspace that exists on disk but could not be read, carrying its directory identity and a reason naming the offending path |
| `ChangeRoster` | The active-change roster as two separate facts: the records that were read and the workspaces that could not be, so absence and unreadability cannot share a value |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas only when not already canonical |
| `acceptance_entries` | `root: &Path, record: &ChangeRecord` | `Vec<AcceptanceInputEntryV1>` | Accepted acceptance-input entries, so `change show --json` can surface the `specsync.acceptance-entry.v1` digests `change supersede --digest` requires; empty when evidence is absent |
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
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record an ordinary mandatory definition approval |
| `approve_definition_portable_v501` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Atomically record the marked current/5.0.1 portable definition pair |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `artifacts_complete_for_guidance` | `root, record` | `bool` | Lightweight selected-artifact completeness for human next-action guidance without digest loaders |
| `audit_project` | `root: &Path` | `SddCheckReport` | Active workspaces + living policy/spec coherence only — does not rewalk archived terminal evidence |
| `backfill_reopen_digests` | `root: &Path, dry_run: bool` | `Result<ReopenBackfillReport, String>` | Backfill 5.1 reopening digest fields on 5.0.1-era ledgers with verified, idempotent, dry-run-aware writes |
| `begin_change_read_scope` | `root: &Path` | `ChangeReadScope` | Install one invocation-scoped read snapshot for list/show/status and project reports |
| `check_change` | `root, optional id` | `Result<Option<VerificationRecord>, String>` | S…8302 tokens truncated…pr-366: Address all remaining review feedback from PR 366 |
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
