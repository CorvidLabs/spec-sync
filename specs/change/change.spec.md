---
module: change
version: 90
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
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell and rejects direct or indirect entry into every lifecycle command surface.
6. Verification and scoped-review evidence bind the implementation commit and governed inputs; a scoped review records an explicit pass/block verdict, must be independent from the scope approver, and stays fresh only when every descendant/parent edge changes supported lifecycle persistence.
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
20. Workflow-v2 adoption atomically freezes a comparison-base cutoff that precedes its unique introduction, opens its lifecycle lock without following symlinks, journals only lossless UTF-8 publication paths whose filename components cannot be confused with platform separators, confines them beneath the project without symlink traversal, leaves an existing version-1 policy byte-identical, refuses to strand v1 records absent from that cutoff, routes every subsequent change through workflow v2, and fails closed if any reachable parent introduced a subsequently absent baseline.
21. Existing-change definition mutations validate correction-ledger integrity while holding the same project lock that guards persistence and return the validated effective-definition snapshot used by command output.

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
| `UnreadableChange` | One active-change workspace that exists on disk but could not be read, carrying its directory identity and a reason naming the offending path |
| `ChangeRoster` | The active-change roster as two separate facts: the records that were read and the workspaces that could not be, so absence and unreadability cannot share a value |

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas only when not already canonical |
| `acceptance_entries` | `root: &Path, record: &ChangeRecord` | `Vec<AcceptanceInputEntryV1>` | Accepted acceptance-input entries, so `change show --json` can surface the `specsync.acceptance-entry.v1` digests `change supersede --digest` requires; empty when evidence is absent |
| `add_acceptance_owner_correction` | `root, id, path, module, actor, reason` | `Result<ChangeRecord, String>` | Append one audited exact canonical owner correction to a reopened already-applied change |
| `add_acceptance_owner_corrections` | `root, id, entries, actor, reason` | `Result<ChangeRecord, String>` | Validate every exact path/module owner correction, then append all as sequenced audit entries in one transactional write |
| `add_dependency` | `root, id, dependency` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, declares ordering between active changes, and invalidates stale approval digests |
| `add_dependency_with_snapshot` | `root, id, dependency` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the dependency mutation with its full in-transaction machine snapshot |
| `add_missing_acceptance_owner_corrections` | `root, id, module, actor, reason` | `Result<ChangeRecord, String>` | Discover production-source affected paths lacking canonical ownership for a module and append them as one transactional batch |
| `add_supersedes_obligation` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then adds one definition-bound semantic succession obligation to a draft |
| `add_supersedes_obligation_with_snapshot` | `root, id, predecessor, path, module, predecessor_entry_digest` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the supersession mutation with its full in-transaction machine snapshot |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or atomically enable SDD, activate workflow v2 without stranding cutoff-ineligible legacy records or rewriting legacy policy, and import OpenSpec or Spec Kit artifacts |
| `answer_question` | `root, id, question, answer` | `Result<ChangeRecord, String>` | Production domain API that validates ledger health under lock, then persists an interview answer and updates adaptive artifacts |
| `answer_question_with_snapshot` | `root, id, question, answer` | `Result<DefinitionMutationResult, String>` | Crate-private command path that returns the answer mutation with its full in-transaction machine snapshot |
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record an ordinary mandatory definition approval |
| `approve_definition_portable_v501` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Atomically record the marked current/5.0.1 portable definition pair |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `artifacts_complete_for_guidance` | `root, record` | `bool` | Lightweight selected-artifact completeness for human next-action guidance without digest loaders |
| `audit_project` | `root: &Path` | `SddCheckReport` | Active workspaces + living policy/spec coherence only — does not rewalk archived terminal evidence |
| `backfill_reopen_digests` | `root: &Path, dry_run: bool` | `Result<ReopenBackfillReport, String>` | Backfill 5.1 reopening digest fields on 5.0.1-era ledgers with verified, idempotent, dry-run-aware writes |
| `begin_change_read_scope` | `root: &Path` | `ChangeReadScope` | Install one invocation-scoped read snapshot for list/show/status and project reports |
| `check_change` | `root, optional id` | `Result<Option<VerificationRecord>, String>` | Select one approved/implementing change, materialize its canonical deltas, and verify it |
| `check_change_with_strict` | `root, optional id, strict` | `Result<Option<VerificationRecord>, String>` | Run `check_change` with additive strict validators |
| `check_project` | `root: &Path` | `SddCheckReport` | Full lifecycle integrity including archive terminal evidence (tests and rare callers; not the default CLI path) |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `finalize_change` | `root, id` | `Result<PathBuf, String>` | Validate current verification/review evidence and transactionally produce the dated same-PR archive |
| `find_change_dir` | Resolves a change's workspace wherever it lives, active or archived — the single answer to where a change's artifacts are |
| `floor_sequence_ledger_to_committed` | `root: &Path` | `Result<Option<(u64, u64)>, String>` | Raise a working-tree sequence ledger to the committed high-water mark before staging, returning the previous and adopted values so the caller can disclose the raise, or `None` when the ledger is already at or above it |
| `list_changes` | `root: &Path` | `Result<ChangeRoster, String>` | List active changes in stable ID order alongside the workspaces that could not be read; `Err` only when the changes directory itself is unreadable |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Return deterministic unanswered interview questions |
| `record_bootstrap_paths` | `root: &Path` | `Result<(), String>` | Record the protected SDD paths this bootstrap created in `.specsync/bootstrap.json`, so initialization's own output is not reported as uncovered meaningful delivery; editing a recorded file revokes its exemption |
| `record_scoped_review` | `root, id, reviewer` | `Result<ScopedReviewRecord, String>` | Record one independent implementation-scoped review bound to current governed inputs |
| `record_scoped_review_with_verdict` | `root, id, reviewer, verdict` | `Result<ScopedReviewRecord, String>` | Record an explicit passing or blocking independent review; only a current passing verdict permits finalization |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, correction health, and next action using the shared verification-freshness predicate |
| `summarize_change_with_strict` | `root, record, explicit_strict` | `ChangeSummary` | Project the same status plus exact targeted/additive-strict validator commands |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract evidence |
| `verify_change_with_strict` | `root, id, strict` | `Result<VerificationRecord, String>` | Run targeted validators plus additive strict policy/classification validators on the same evidence path |
| `write_default_policy` | `root: &Path, verification_commands: Vec<String>` | `Result<(), String>` | Write new-project/adoption policy without overwriting existing policy |

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

## Invariants

1. Change IDs are monotonically assigned as `CHG-NNNN-slug` across active and archived workspaces.
2. No emergency or force transition bypass exists.
3. Approval digests exclude volatile lifecycle state but include every selected artifact and semantic delta.
4. Any addition, removal, or replacement in approved stable scope invalidates approval until the new digest is approved.
5. Finalization rejects stale commits, contracts, reviews, incomplete tasks, failed tests, and missing requirement evidence.
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
22. Local and hosted verification freshness inspect every intervening commit against every parent, permit only `state.json`, `verification.json`, `verification-attempts.json`, `review.json`, and `review-attempts.json` below canonical active-change IDs, and never infer safety from a net diff or broad volatile-path exclusion.
23. Exact-owner corrections are additive, restricted to an original affected path and a current canonical source owner, and cannot mutate semantic definition fields or prior evidence.
24. A fully valid later accepted sequence owner covers only historical sequence-ledger drift; reconstruction reuses exact committed collision-owner ledger bytes when available, while the current owner and every non-ledger input remain exact.
25. A structurally valid audited delivery reopen preserves immutable sequence-collision history while fresh verification and closing approval remain mandatory.
26. Accepted-change archival trusts an in-history commit recording the change as accepted with byte-identical evidence when no first-acceptance transition anchor matches, so squash-merged evidence remains archivable while the exactly-one-eligible rule stays fail-closed.
27. Legacy acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers validate without remediation while newly signed evidence stays fail-closed.
28. Batch exact-owner correction validates every proposed path/module pair independently and fails closed with zero persisted mutations when any entry is invalid.
29. The 5.0 ledger migration backfills reopening digest fields idempotently from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.
30. Canonical module path resolution treats missing and inert local registries as absent fallbacks while non-inert unparsable registries still fail closed with the established parse diagnostic.
31. Immutable workflow-origin validation follows every bounded reachable canonical dated archive path for the exact change ID, preserving identity across archive, reopen, and cross-date rearchive moves.
32. The workflow-v2 baseline retains its exact introduction bytes at every bounded touching commit and readable parent, rejecting rewrite-then-restore history.
33. Answer, dependency, and supersession mutations load and validate correction history only after acquiring the lifecycle project lock.

## Behavioral Examples

**Scenario: Verified feature delivery**

- **Given** an approved feature with `REQ-auth-001`, completed artifacts, and configured targeted tests
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

**Scenario: Correction history changes while a mutation waits**

- **Given** an existing-change mutation blocked on the lifecycle project lock
- **When** the correction ledger becomes invalid before that mutation acquires the lock
- **Then** the mutation reloads and validates the ledger under lock, fails safely, and persists no lifecycle update

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Populated semantic delta with no recognized operation heading | Approval and historical validation name the allowed `## Added`, `## Modified`, and `## Removed` headings instead of reporting the file empty |
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
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |

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

| 2026-07-30 | Add `audit_project`; scope CLI `change check` to one change; archives are history |

| Date | Change |
|------|--------|
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
