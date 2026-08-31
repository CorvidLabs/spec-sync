---
module: change
version: 116
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
5. `change check` compares specs to source in-process, records that pass as `specsync check` evidence, and does not spawn `sdd.json` `verification_commands`, `cargo test`, or any other project test or build command.
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
| `check_change` | `root, optional id` | `Result<Option<VerificationRecord>, String>` | Select one approved/implementing change, materialize its canonical deltas, and compare this change's specs to code |
| `check_change_with_strict` | `root, optional id, strict` | `Result<Option<VerificationRecord>, String>` | Run `check_change` with warnings failing as they do under `specsync check --strict` |
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
| `recorded_scoped_review_currency` | `root, record` | `Option<ScopedReviewCurrency>` | Classify a change's recorded scoped review against the tree on disk; `None` means no usable review record exists, which is a different question from currency |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, correction health, and next action using the shared verification-freshness predicate |
| `summarize_change_with_strict` | `root, record, explicit_strict` | `ChangeSummary` | Project the same status plus the exact scoped command the next pass records as evidence; `--strict` only when requested |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Compare this change's specs to code in-process and record commit/contract evidence |
| `verify_change_with_strict` | `root, id, strict` | `Result<VerificationRecord, String>` | The same spec↔code pass with warnings failing as they do under `specsync check --strict` |
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
34. `finalize_change` assembles `lesson-bundle.md` into the archive on a best-effort basis: a bundle failure never undoes a completed archival, and the material is read entirely from disk so finalize keeps working offline and in CI. SpecSync assembles and never authors the lessons. Folding them into `context.md` is a convention, not a `next_action` gate.
35. This module defines no frontmatter reader of its own. Lesson counting, archived lesson bundles, and artifact completeness all read through `parser::strip_frontmatter`, the single canonical implementation, which ends frontmatter at its CLOSING delimiter LINE in either LF or CRLF encoding and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, a CRLF-authored companion is stripped exactly as an LF one is, a leading BOM never hides the opening delimiter, and a delimiter line padded with trailing whitespace still ends the block at BOTH ends. Four failure modes of the strippers this module used to own are gone with them: a written CRLF artifact is no longer refused as incomplete; an artifact that is only frontmatter is no longer accepted as written when it is closed at end of file, prefixed with a BOM, or opened with a delimiter carrying a trailing space; and prose above the first horizontal rule in a body is no longer deleted when the CLOSING delimiter carries one. One residual is stated rather than guessed at: an artifact opened with `----`, a Markdown thematic break and not a delimiter, still reads as written even when it holds nothing but frontmatter — accepting it as a delimiter would cut real bodies at their first rule, which is the worse failure, and deriving the gate from the generated scaffold instead would not close it either, because a file with a mangled opener no longer equals that scaffold.
36. A `###` heading inside an open semantic-delta item is section CONTENT and does not end that item. Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` start a new item, and classification happens before the previous item is flushed — otherwise one section carrying subheadings becomes several items under one key and application keeps only the last, silently discarding documented behaviour the change never touched.
37. A semantic delta declaring the same operation, target and key more than once is REFUSED. Applying it would keep the last body and discard the earlier ones with no diagnostic.
38. Semantic delta bodies are bound to the definition approval that signed them: approval records a per-module digest over each delta file's body, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. The body is hashed with `\r\n` folded to `\n` and NOTHING ELSE folded, so the binding asks the question delta application already asks: `markdown_block_matches` compares ignoring line-ending style and `parse_delta` reads through `lines()`, so a CRLF and an LF delta materialize byte-identical canonical specs and a checkout that rewrote the line endings cannot invalidate an approval. The equality stays STRICTLY NARROWER than the applier's: trailing whitespace, blank lines and a lone carriage return are wording a reviewer signed and still move the digest, and Git rewrites none of them on its own. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.
39. Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the JSON already pinned there and for the reason that file already states: change artifacts and semantic delta bodies are read as lifecycle evidence, so a working tree that rewrites them into CRLF makes honest, unmodified work arrive in non-canonical form. The pin governs this repository's own working trees; it is not a substitute for readers that tolerate CRLF, because an adopter's repository, a tarball, or an archive extracted without Git is never covered by it.
40. A recorded delta binding is MONOTONE within one approval ledger. Every writer of a `definition` gate records the per-module delta digest it approved — ordinary approval, the normalizing approval inside explicit acceptance, and both members of the portable SpecSync 5.0.1 pair alike — so within a ledger the binding only ever goes from absent to present. An effective definition approval that records no delta wording while another definition approval in the same ledger records it is a claim being withdrawn, not evidence predating the binding, and materialization and acceptance refuse it and name the re-approval remedy. Absence across a whole ledger still reads as unknown, because that is the only shape recorded history has: a change is either from before the binding existed or from after it, never both.
41. `canonical_applied` records that materialization RAN, never that it ran for the delta bodies on disk now, so `change check` and acceptance decide from the canonical artefacts instead of from the flag alone. Materialization produces three outputs per module — the delta applied to the canonical files, the spec's `version:` bump, and the spec's Change Log row — and the flag's short-circuit skipped ALL THREE, so a delta corrected after review and re-approved satisfied the delta binding (a new approval signs the new body) and then left changed contract text with no bump and no row while `change check`, `change audit --strict` and `specsync check --strict` all passed. A module is materialized again when its delta is not fully reflected in the canonical files or when its Change Log carries no row for the change, and is left untouched when both hold: a byte-identical re-approval still writes nothing, and one change bumps one module's version exactly once. Convergence is scoped to an already-applied change — on a first materialization every application refusal still fires, and only afterwards does an already-reflected item, such as a `## REMOVED` block that is already absent, read as done rather than as an error.

## Behavioral Examples

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
| Spec documents an export that is not in source | `change check` fails with the spec finding; configured test commands are not run |
| HEAD changes after verification | Acceptance requires re-verification |
| Any intervening commit changes a disallowed path, even if later reverted | Status and strict checking require re-verification in every environment |
| Accepted delivery evidence is still current AND its verification commit is still anchored | Reopen is rejected without changing lifecycle or audit state |
| Accepted verification commit is unreachable and no reachable history records the acceptance | Reopen is admitted and records `VerificationCommitUnanchored`, even when delivery inputs are byte-identical |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |
| Ownership correction is not exact, additive, in-scope, and canonically provable | Correction is rejected transactionally |
| Covered delivery input of an accepted change changes with no covering accepted successor | Unified check names the input path, its owner, and the `change reopen` remediation |
| Covered delivery input changes while every covering successor is itself stale | Unified check names the input, the sorted covering successor IDs, and their stale evidence state |
| Covered delivery input disappears from the current inventory | Unified check names the missing path and the restore-or-reopen remediation |
| Non-inert local registry cannot be parsed while resolving a module | Canonical path resolution fails closed with `failed to parse local registry {path} while resolving `{module}`` |
| A repeated stage-zero path has a different mode or object ID | Git candidate inspection fails closed without replacing the first observation |
| Correction ledger is invalid when a definition mutation acquires the project lock | Mutation emits the safe integrity diagnostic and persists no lifecycle update |
| Effective definition approval records no semantic delta wording while an earlier definition approval in the same ledger recorded it | Materialization and acceptance refuse the withdrawn claim and name `specsync change approve <id>` as the remedy |

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

| 2026-07-30 | Add `audit_project`; scope CLI `change check` to one change; archives are history |

| Date | Change |
|------|--------|
| 2026-08-30 | Fresh `init` writes SDD off; `require_change_for_meaningful_files` is false on new policies. `check` does not call `audit_project`. `change check` is in-process spec↔code sync and does not spawn `verification_commands`. |
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
| 2026-08-30 | make-check-the-product-and-stop-change-check-from-spawning-project-tests: Make check the product and stop change check from spawning project tests |
