---
module: change
version: 42
status: active
files:
  - src/change.rs
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
---

# Change

## Purpose

Provides the spec-sync 5.0 verified spec-driven development lifecycle, including audited recovery and re-verification when governed delivery inputs make accepted evidence stale.

## Contract

1. Every meaningful SDD change moves through draft, approved, implementing, verifying, accepted, and archived states without bypasses.
2. Definition and closing approvals are portable records bound to deterministic SHA-256 digests; an explicitly requested 5.1 authority approval uses one atomically appended marked current/5.0.1-compatible definition pair whose effective full member is resolved centrally without historical approval search.
3. Approved semantic deltas form the effective future contract without mutating canonical specs before acceptance.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell and rejects direct or indirect entry into every lifecycle command surface.
6. Verification evidence is bound to the tested commit and working-tree inputs; descendant freshness is environment-independent and permits only internally consistent supported verification-persistence commits after inspecting every commit and parent edge.
7. Invalid policy, unavailable coverage comparison, failed evidence, stale ordering gates, and protected sequence-ledger edits without lifecycle coverage fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, corrupt state fails closed, and archival failures remain retryable.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
11. Concurrent effective-contract validations use isolated temporary workspaces.
12. Stale accepted delivery evidence can return only to verifying through an explicit human actor and reason, while prior verification and closing evidence remain inspectable.
13. Historical collision acknowledgements are exact immutable accepted-or-archived evidence and numeric sequence width has no four-digit upper bound.
14. A fully valid later sequence claim supersedes only the sequence-ledger bytes in historical acceptance inputs; the current owner and every other covered input remain exact evidence.
15. Supported accepted interview metadata changes only through a portable append-only correction ledger whose effective definition requires fresh gates and never replays canonical deltas.
16. Audited exact acceptance-owner corrections can repair omitted canonical ownership on an already-scoped input without changing semantic scope or replaying canonical deltas.
17. A transactional batch of audited exact acceptance-owner corrections validates every entry independently and persists all or none as sequenced ledger entries.

## Public API

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
| `ChangeRecord` | Durable machine state for one change workspace, including omitted-when-empty supersedes edges and acceptance-owner corrections |
| `LegacyArchiveBaselineV1` | Definition- and closing-bound authority, cutoff, and sorted legacy archive subtree entries |
| `LegacyArchiveBaselineEntryV1` | Archive ID, canonical dated path, unique introduction commit, and exact subtree digest |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `ApprovalRecord` | Actor, timestamp, gate, digest, optional note, and optional backward-readable portable-pair metadata for one approval |
| `DefinitionApprovalPairRole` | Current/full or legacy/projected role for one marked portable definition member |
| `DefinitionApprovalPairV1` | Versioned pair identity, projection, role, change/correction coordinates, event index, and both digests |
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, and stale/current input digests |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `ReopenBackfillReport` | Per-change repair, skip, and failure detail for a `migrate 5.0` ledger backfill |
| `CorrectionField` | Closed supported accepted-metadata field set: public contract and architecture risk |
| `CorrectionRecord` | Immutable sequenced metadata correction with original/effective values, actor, reason, artifacts, prior evidence, and portable digest chain |
| `EffectiveChangeDefinition` | Validated projection of original answers/artifacts plus ordered corrections |
| `CorrectionResult` | Deterministic corrected change, event, effective definition, history, and gate-summary projection |
| `ApprovalLedger` | Ordered portable approval and reopen history |
| `CommandEvidence` | Exit evidence for one configured verification command |
| `AcceptanceInputKind` | Canonical file, symlink, gitlink, missing, or non-file topology kind |
| `AcceptanceInputEntryV1` | Bounded path, kind, mode, payload digest, full-entry digest, and sorted owners for one accepted input |
| `AcceptanceManifestV1` | Versioned sorted per-input acceptance manifest |
| `SemanticSuccessionTupleV1` | Exact predecessor, path, module, old-entry digest, and new-entry digest transition |
| `SemanticSuccessionEvidenceV1` | Versioned sorted one-to-one closing evidence for approved supersedes obligations |
| `VerificationRecord` | Commit-bound verification result, contract digest, commands, requirement coverage, and optional acceptance manifest/succession evidence |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `TerminalEvidenceValidity` | State-aware exact, successor-covered, stale, authenticated-history, or corrupt-history evidence conclusion |
| `TerminalEvidenceSummary` | Shared terminal validity plus optional fail-closed reason |
| `TerminalEvidenceResult` | Change ID paired with its shared terminal-evidence summary |
| `ChangeSummary` | Human/agent status projection with gate health, next action, and optional terminal-evidence summary |
| `SddCheckReport` | Unified lifecycle errors, warnings, checked-change count, and terminal-evidence results |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `write_default_policy` | `root: &Path, verification_commands: Vec<String>` | `Result<(), String>` | Write new-project/adoption policy without overwriting existing policy |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `list_changes` | `root: &Path` | `Vec<ChangeRecord>` | List active changes in stable ID order |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Return deterministic unanswered interview questions |
| `answer_question` | `root, id, question, answer` | `Result<ChangeRecord, String>` | Persist an interview answer and update adaptive artifacts |
| `add_dependency` | `root, id, dependency` | `Result<ChangeRecord, String>` | Declare ordering between active changes and invalidate stale approval digests |
| `add_supersedes_obligation` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<ChangeRecord, String>` | Add one validated definition-bound semantic succession obligation to a draft |
| `add_acceptance_owner_correction` | `root, id, path, module, actor, reason` | `Result<ChangeRecord, String>` | Append one audited exact canonical owner correction to a reopened already-applied change |
| `add_acceptance_owner_corrections` | `root, id, entries, actor, reason` | `Result<ChangeRecord, String>` | Validate every exact path/module owner correction, then append all as sequenced audit entries in one transactional write |
| `add_missing_acceptance_owner_corrections` | `root, id, module, actor, reason` | `Result<ChangeRecord, String>` | Discover production-source affected paths lacking canonical ownership for a module and append them as one transactional batch |
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record an ordinary mandatory definition approval |
| `approve_definition_portable_v501` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Atomically record the marked current/5.0.1 portable definition pair |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract evidence |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas only when not already canonical |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `backfill_reopen_digests` | `root: &Path, dry_run: bool` | `Result<ReopenBackfillReport, String>` | Backfill 5.1 reopening digest fields on 5.0.1-era ledgers with verified, idempotent, dry-run-aware writes |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, correction health, and next action using the shared verification-freshness predicate |
| `check_project` | `root: &Path` | `SddCheckReport` | Validate lifecycle state, approvals, corrections, conflicts, deltas, path coverage, and shared verification freshness |
| `check_project_quiet` | `root: &Path` | `SddCheckReport` | Run the same fail-closed lifecycle check while suppressing configured command output for machine-consumable report protocols |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or enable SDD and import OpenSpec or Spec Kit artifacts |
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

## Invariants

1. Change IDs are monotonically assigned as `CHG-NNNN-slug` across active and archived workspaces.
2. No emergency or force transition bypass exists.
3. Approval digests exclude volatile lifecycle state but include every selected artifact and semantic delta.
4. Any approved definition change invalidates approval until the new digest is approved.
5. Acceptance rejects stale commits, stale contracts, incomplete tasks, failed tests, and missing requirement evidence.
6. Overlapping active semantic keys are blocked unless changes declare ordering dependencies.
7. Canonical spec versions increment and changelogs reference the accepted change ID.
8. A failed multi-file write restores all prior canonical content.
9. Change dependencies are acyclic and must be accepted or archived before dependent implementation begins.
10. Meaningful-path coverage compares the branch with the current GitHub/remote default base after a rebase, falling back to the recorded creation commit only when no remote base is available.
11. Approval digests hash repository-relative artifact paths so identical Git content validates across checkout locations and operating systems.
12. Verification command detection prefers portable project-manifest commands and uses Fledge only when no native manifest is available.
13. Persisted and hashed project paths use forward slashes on every operating system.
14. Quiet reporting executes every configured command and preserves failures while suppressing only child stdout and stderr; normal checking and verification continue streaming diagnostics.
15. Reopening current accepted evidence is rejected, and reopening stale evidence never reapplies an already canonical semantic delta.
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event unless every difference is a validated additive exact-owner correction.
17. False default lifecycle fields remain absent from new persisted state, while definition validation recognizes both omitted and transitional explicit-false encodings so upgrades preserve existing approvals and verification; explicit acceptance appends stable definition evidence when the latest compatible approval uses the transitional encoding.
18. Audited reopen accepts unreachable verification commits only when canonical acceptance is recorded in current history or later recorded canonical changes govern every affected contract surface.
19. Acceptance appends a Change Log row matching the canonical table's existing column schema and uses the post-bump version when the schema includes `Version`.
20. Generated bookkeeping never replaces explicit delivery scope; registry authority, policy enablement, and native command identity are evaluated consistently before lifecycle enforcement.
21. Trusted correction-history discovery ignores unresolved remote-default references and parses Git tree paths without quoting ambiguity; regression fixtures preserve quoted-path coverage where supported while remaining valid on Windows.
22. Local and hosted verification freshness inspect every intervening commit against every parent, permit only the three supported persistence files below canonical active-change IDs, and never infer safety from a net diff or broad volatile-path exclusion.
23. Exact-owner corrections are additive, restricted to an original affected path and a current canonical source owner, and cannot mutate semantic definition fields or prior evidence.
24. A fully valid later accepted sequence owner covers only historical sequence-ledger drift, while the current owner and every non-ledger input remain exact.
25. A structurally valid audited delivery reopen preserves immutable sequence-collision history while fresh verification and closing approval remain mandatory.
26. Accepted-change archival trusts an in-history commit recording the change as accepted with byte-identical evidence when no first-acceptance transition anchor matches, so squash-merged evidence remains archivable while the exactly-one-eligible rule stays fail-closed.
27. Legacy acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers validate without remediation while newly signed evidence stays fail-closed.
28. Batch exact-owner correction validates every proposed path/module pair independently and fails closed with zero persisted mutations when any entry is invalid.
29. The 5.0 ledger migration backfills reopening digest fields idempotently from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.

## Behavioral Examples

### Scenario: Verified feature delivery

- **Given** an approved feature with `REQ-auth-001`, completed artifacts, and configured tests
- **When** implementation verifies and a human accepts it
- **Then** canonical requirements/specs update, the spec version increments, and the change becomes accepted

### Scenario: Approved intent changes

- **Given** a valid definition approval
- **When** a selected design, requirement, or delta is edited
- **Then** progress is blocked until the new digest is approved

### Scenario: Feature branch rebases onto upstream

- **Given** a change workspace created before new commits landed on the remote default branch
- **When** the feature branch rebases and unified checking computes meaningful changed paths
- **Then** upstream-only paths are excluded and only the feature branch diff requires change coverage

### Scenario: Review fixes stale accepted evidence

- **Given** an accepted change whose governed delivery inputs changed after closing approval
- **When** a human reopens it with an actor and reason
- **Then** the prior verification and closing approval remain in audit history, strict checking stays red until fresh verification, and reacceptance records a new closing approval without reapplying canonical deltas

### Scenario: Persisted verification evidence

- **Given** a supported verification run passed on the current commit
- **When** one or more descendant commits persist only its canonical state, verification, and attempt-ledger files
- **Then** local status, local strict checking, and hosted checking all keep the evidence current while matching contract and project-input digests remain mandatory

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Verification command contains shell operators | Command is rejected without execution |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current | Reopen is rejected without changing lifecycle or audit state |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes with no covering accepted successor | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every covering successor is itself stale | Unified check names the input, the sorted covering successor IDs, and their stale evidence state |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| hash_cache | Project SHA-256 dependency for content identity |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_change | Complete lifecycle command surface |
| cmd_check | Unified SDD gate before canonical validation |
| cmd_init | New-project policy and version initialization |

## Change Log

| Date | Change |
|------|--------|
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
