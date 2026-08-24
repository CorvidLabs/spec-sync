## MODIFIED

### SPEC SECTION Contract

1. Every new meaningful change follows one guided path: draft, one scope approval, implementation, verification, scoped review, same-PR finalization/archive, and GitHub merge.
2. The scope approval is bound to a deterministic SHA-256 projection of stable intent, contract, and affected scope; volatile implementation, test/evidence, semantic-delta materialization, canonical materialization, and lifecycle metadata bind a separate execution digest. The one CHG-0068 legacy adoption declares its missing source preimage and lack of equivalence proof, and a compile-time allowlist freezes its exact commit/blob anchor, source approval, adopted scope, authorization, and classifications.
3. Approved semantic deltas form the effective future contract, and `change check` materializes them into canonical specs before scoped review and finalization; a delta body that changed after its approval is refused rather than applied.
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

### SPEC SECTION Public API

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
| `ReopenCauseV1` | Why accepted evidence was stale when delivery-input digests alone cannot show it; absent means the inputs drifted |
| `recorded_verification_is_current` | Whether a change's recorded verification still matches the tree and plan on disk, as a content question; missing or unreadable evidence is not current |
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
| `accumulated_lessons` | `root, modules` | `Vec<(String, usize)>` | Substantive-prose line count for each module context that holds any, so a new change can be pointed at what its modules already learned |
| `active_change_id` | `root` | `Option<String>` | The change a bare lifecycle command acts on: the single active approved/implementing/verifying record — the same states `check_change` selects — or none |
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
| `lesson_fold_targets` | `root, id` | `Vec<String>` | Module context paths this change's lessons are folded into at archival; empty when the change is unreadable or owns no specs |
| `list_changes` | `root: &Path` | `Result<ChangeRoster, String>` | List active changes in stable ID order alongside the workspaces that could not be read; `Err` only when the changes directory itself is unreadable |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `module_context_path` | `module` | `String` | The single definition of where a module's accumulated lessons live, shared by surfacing and folding so they cannot disagree |
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

### SPEC SECTION Invariants

1. Change IDs are monotonically assigned as `CHG-NNNN-slug` across active and archived workspaces.
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
12. Verification command detection prefers portable project-manifest commands and uses Fledge only when no native manifest is available.
13. Persisted and hashed project paths use forward slashes on every operating system.
14. Quiet reporting executes every configured command and preserves failures while suppressing only child stdout and stderr; normal checking and verification continue streaming diagnostics.
15. Reopening accepted evidence is rejected only when its delivery inputs are current AND its verification commit is still anchored in current history; reopening stale evidence never reapplies an already canonical semantic delta.
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event unless every difference is a validated additive exact-owner correction.
17. False default lifecycle fields remain absent from new persisted state, while definition validation recognizes both omitted and transitional explicit-false encodings so upgrades preserve existing approvals and verification; explicit acceptance appends stable definition evidence when the latest compatible approval uses the transitional encoding.
18. An unreachable verification commit is itself an admissible staleness axis for audited reopen, recorded as an explicit cause in the reopen ledger; every other authentication check stays fatal, and reacceptance still requires canonical acceptance or deterministic semantic succession provable from trusted history.
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
34. `finalize_change` assembles `lesson-bundle.md` into the archive on a best-effort basis: a bundle failure never undoes a completed archival, and the material is read entirely from disk so finalize keeps working offline and in CI. SpecSync assembles and never authors the lessons; the agent that ran `finalize` writes them, guided by `next_action`.
35. Frontmatter is stripped at its closing delimiter LINE, in either LF or CRLF encoding, and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, and a CRLF-authored companion is stripped exactly as an LF one is — otherwise its frontmatter survives into content counting and an untouched scaffold reports itself as recorded knowledge.
36. Semantic delta bodies are bound to the definition approval that signed them: approval records a digest over each delta file's exact bytes keyed by module, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.

## ADDED

### REQUIREMENT REQ-change-089

A semantic delta body SHALL be bound to the definition approval that signed it, and an approval that recorded no such binding SHALL read as unknown rather than as a violation.

Acceptance Criteria
- Approval records a digest over the exact bytes of every semantic delta file the change owns, keyed by module, so the wording that rewrites a canonical spec is part of what a human signed.
- Materialization and acceptance verify that binding before any delta is applied, and refuse by naming every module whose body changed after approval together with the remedy.
- The refusal is evaluated before the already-materialized short-circuit, so a body that drifts after the first application is still caught while it remains this change's evidence.
- An approval carrying no delta binding proceeds unchanged, because every change approved before the binding existed made no claim about wording and absent evidence is not a violation.
- The binding is omitted from persisted JSON when it is absent, so no existing approval digest moves and ledgers written by earlier binaries stay readable and byte-identical.
