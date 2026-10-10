## ADDED

### REQUIREMENT REQ-change-104

The system SHALL ask a draft which existing lessons or principles constrain it, and only when that draft's affected modules have substantive lessons or the project names a principles file. The question SHALL be asked after the rest of the interview is answered. The answer "none" SHALL be complete. The answer SHALL be part of the stable scope, so changing it SHALL require a fresh approval. Re-answering the affected specs SHALL clear the answer. A draft with neither lessons nor a principles file SHALL NOT be asked. A change that has left draft without the answer SHALL NOT be sent back to the interview.

Acceptance Criteria
- A draft with substantive module lessons, or a configured principles file, is asked `constraints` only after the other interview questions are answered.
- The answer "none" completes the question.
- Changing the answer changes the scope digest.
- Re-answering affected specs clears the answer and asks the question again when it still applies.
- A scaffold-only context and no principles file produces no question.
- An approved, implementing, verifying, or archived change that never recorded the answer stays a complete interview.

### REQUIREMENT REQ-change-105

The system SHALL refuse approval of a selected plan until that plan says what will be done, what will not be done, the steps, the risks, and which existing constraints are honored. The system SHALL NOT add a plan to a change the interview did not select one for.

Acceptance Criteria
- A selected plan is incomplete while Approach, Out of scope, Steps, Risks, or Constraints consulted is missing or has no substantive text.
- Headings, blank lines, HTML comments, and placeholder TODO lines are not substantive text.
- A plan with substantive text under each of those headings is complete, once the generic artifact TODO check also passes.
- A bug fix and a documentation change that did not select a plan are not given one by this rule.
- Status names the plan file while it is incomplete, using the same predicate as approval.

### REQUIREMENT REQ-change-106

The system SHALL show, when a change is opened, shown, or asked its status, the lessons already written for the modules it touches and the project's principles when a principles file is configured. The text result and the structured result SHALL carry the same briefing. A module context that holds only its generated scaffold SHALL NOT be presented as knowledge. The briefing SHALL be bounded and SHALL name the file for anything left unread. An unreadable context or principles file SHALL NOT fail the command.

Acceptance Criteria
- The briefing lists each affected module context that has substantive lines, with the path, the substantive line count, a bounded excerpt, and whether the excerpt was cut.
- A scaffold-only context is absent.
- A configured principles file is named, with a bounded excerpt, or marked missing when it cannot be read.
- No principles file produces no principles entry.
- The briefing is computed and is not stored on the change record.

## MODIFIED

### SPEC SECTION Invariants

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
31. Immutable workflow-origin validation follows every bounded reachable canonical dated archive path for the exact change ID, preserving identity across archive, reopen, and cross-date rearchive moves.
32. The workflow-v2 baseline retains its exact introduction bytes at every bounded touching commit and readable parent, rejecting rewrite-then-restore history.
33. Answer, dependency, and supersession mutations load and validate correction history only after acquiring the lifecycle project lock.
34. `finalize_change` assembles `lesson-bundle.md` into the archive on a best-effort basis: a bundle failure never undoes a completed archival, and the material is read entirely from disk so finalize keeps working offline and in CI. SpecSync assembles and never authors the lessons. Folding them into `context.md` is a convention, not a `next_action` gate. Finalize and ship name the bundle path in text and JSON without making that fold the next action.
35. This module defines no frontmatter reader of its own. Lesson counting, archived lesson bundles, and artifact completeness all read through `parser::strip_frontmatter`, the single canonical implementation, which ends frontmatter at its CLOSING delimiter LINE in either LF or CRLF encoding and never at the next `---` elsewhere in the document. A Markdown horizontal rule therefore never truncates a body to a fragment, a CRLF-authored companion is stripped exactly as an LF one is, a leading BOM never hides the opening delimiter, and a delimiter line padded with trailing whitespace still ends the block at BOTH ends. Four failure modes of the strippers this module used to own are gone with them: a written CRLF artifact is no longer refused as incomplete; an artifact that is only frontmatter is no longer accepted as written when it is closed at end of file, prefixed with a BOM, or opened with a delimiter carrying a trailing space; and prose above the first horizontal rule in a body is no longer deleted when the CLOSING delimiter carries one. One residual is stated rather than guessed at: an artifact opened with `----`, a Markdown thematic break and not a delimiter, still reads as written even when it holds nothing but frontmatter — accepting it as a delimiter would cut real bodies at their first rule, which is the worse failure, and deriving the gate from the generated scaffold instead would not close it either, because a file with a mangled opener no longer equals that scaffold.
36. A `###` heading inside an open semantic-delta item is section CONTENT and does not end that item. Only `### REQUIREMENT <id>` and `### SPEC SECTION <name>` start a new item, and classification happens before the previous item is flushed — otherwise one section carrying subheadings becomes several items under one key and application keeps only the last, silently discarding documented behaviour the change never touched.
37. A semantic delta declaring the same operation, target and key more than once is REFUSED. Applying it would keep the last body and discard the earlier ones with no diagnostic.
38. Semantic delta bodies are bound to the definition approval that signed them: approval records a per-module digest over each delta file's body, and materialization and acceptance refuse to rewrite a canonical spec when a body no longer matches, naming every module that drifted. The body is hashed with `\r\n` folded to `\n` and NOTHING ELSE folded, so the binding asks the question delta application already asks: `markdown_block_matches` compares ignoring line-ending style and `parse_delta` reads through `lines()`, so a CRLF and an LF delta materialize byte-identical canonical specs and a checkout that rewrote the line endings cannot invalidate an approval. The equality stays STRICTLY NARROWER than the applier's: trailing whitespace, blank lines and a lone carriage return are wording a reviewer signed and still move the digest, and Git rewrites none of them on its own. An approval recording no such digest predates the binding and reads as unknown, never as tampering, so every historical archive remains valid.
39. Markdown under `.specsync/` is pinned to `eol=lf` in `.gitattributes`, beside the JSON already pinned there and for the reason that file already states: change artifacts and semantic delta bodies are read as lifecycle evidence, so a working tree that rewrites them into CRLF makes honest, unmodified work arrive in non-canonical form. The pin governs this repository's own working trees; it is not a substitute for readers that tolerate CRLF, because an adopter's repository, a tarball, or an archive extracted without Git is never covered by it.
40. A recorded delta binding is MONOTONE within one approval ledger. Every writer of a `definition` gate records the per-module delta digest it approved — ordinary approval, the normalizing approval inside explicit acceptance, and both members of the portable SpecSync 5.0.1 pair alike — so within a ledger the binding only ever goes from absent to present. An effective definition approval that records no delta wording while another definition approval in the same ledger records it is a claim being withdrawn, not evidence predating the binding, and materialization and acceptance refuse it and name the re-approval remedy. Absence across a whole ledger still reads as unknown, because that is the only shape recorded history has: a change is either from before the binding existed or from after it, never both.
41. `canonical_applied` records that materialization RAN, never that it ran for the delta bodies on disk now, so `change check` and acceptance decide from the canonical artefacts instead of from the flag alone. Materialization produces three outputs per module — the delta applied to the canonical files, the spec's `version:` bump, and the spec's Change Log row — and the flag's short-circuit skipped ALL THREE, so a delta corrected after review and re-approved satisfied the delta binding (a new approval signs the new body) and then left changed contract text with no bump and no row while `change check`, `change audit --strict` and `specsync check --strict` all passed. A module is materialized again when its delta is not fully reflected in the canonical files or when its Change Log carries no row for the change, and is left untouched when both hold: a byte-identical re-approval still writes nothing, and one change bumps one module's version exactly once. Convergence is scoped to an already-applied change — on a first materialization every application refusal still fires, and only afterwards does an already-reflected item, such as a `## REMOVED` block that is already absent, read as done rather than as an error.
42. Handoff readiness is a PURE function of `HandoffSignals`, and the verdict a session sees in text is the same object JSON carries under `summary.handoff`. A frozen sequence ledger, a stale approval digest, an invalid correction ledger, and stale legacy terminal evidence are `not yet` and name their repair, because clearing context there strands the next session on a gate it cannot see the cause of. A Draft is never `safe` — the interview and artifacts live only in the session's head until approval records them — so it is `conditional` and names approval as the first clean boundary. Uncommitted edits under `affected_paths` are `conditional` and name committing or writing intent into `change.md`; evidence under `.specsync/` alone never counts, because `review.json` is uncommitted between `review` and `finalize` by design. A current approval on a clean tree, a Verifying change with current verification, an Accepted workflow-v2 change, and an Archived change are `safe`, and the reason says where the next session resumes. The reason, the resume command, and the steps carry the change ID and literal prose only; no digest reaches them.

43. A draft whose affected modules have substantive lessons, or whose project names a principles file, is asked which of those constrain it once the rest of the interview is answered, and "none" is a complete answer stored in the stable scope. A draft with neither is not asked. A change that has left draft without that answer is not sent back to the interview. Re-answering affected specs clears the answer. A selected plan is incomplete until Approach, Out of scope, Steps, Risks, and Constraints consulted each contain substantive text. A change that did not select a plan does not gain one. Opening, showing, or asking status computes a briefing of those lessons and of that principles file. The excerpt is bounded, a scaffold-only context is omitted, and an unreadable file does not fail the command.


### SPEC SECTION Behavioral Examples

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
| `ScopedReviewVerdict` | Explicit passing or blocking conclusion for one scoped human review |
| `ScopedReviewProvenanceProvider` | Stored provenance-provider declaration; identity authentication requires separately enforced external policy |
| `ScopedReviewProvenanceV1` | Versioned required GitHub Actions check binding carried by review evidence |
| `ScopedReviewRecord` | Stable reviewer claim, required-check provenance, explicit verdict, implementation commit, scope/execution/workspace digests, and review timestamp bound before finalization |
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
| `HandoffSignals` | The complete, digest-free input to `classify_handoff`: state, workflow version, sequence-ledger freeze, open questions, artifact completeness, approval/correction validity, uncommitted scoped edits, verification/review currency, and stale legacy terminal evidence |
| `ChangeSummary` | Human/agent status projection with approval health/current scope digest, plain-language material expansion, validator plan, scoped-review freshness, exactly one next action, a handoff verdict, and optional terminal evidence |
| `SddCheckReport` | Unified lifecycle errors, warnings, checked-change count, and terminal-evidence results |
| `UnreadableChange` | One active-change workspace that exists on disk but could not be read, carrying its directory identity and a reason naming the offending path |
| `ChangeRoster` | The active-change roster as two separate facts: the records that were read and the workspaces that could not be, so absence and unreadability cannot share a value |
| `LifecycleCommitScope` | Crate-private answer to what one change's lifecycle commit may stage: the untracked paths the lifecycle wrote or the change owns, and the runtime files (lock, transaction journal) it must neither stage nor report |

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
| `classify_handoff` | `id, signals` | `HandoffSummary` | Pure classification of handoff readiness from `HandoffSignals`; the only source of the verdict, so text and JSON cannot disagree |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `create_change` | `root: &Path, request: CreateChangeRequest` | `Result<ChangeRecord, String>` | Create a sequential draft workspace and adaptive artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `finalize_change` | `root, id` | `Result<PathBuf, String>` | Validate current verification/review evidence and transactionally produce the dated same-PR archive |
| `find_change_dir` | Resolves a change's workspace wherever it lives, active or archived — the single answer to where a change's artifacts are |
| `floor_sequence_ledger_to_committed` | `root: &Path` | `Result<Option<(u64, u64)>, String>` | Raise a working-tree sequence ledger to the committed high-water mark before staging, returning the previous and adopted values so the caller can disclose the raise, or `None` when the ledger is already at or above it |
| `handoff_summary` | `root, record` | `HandoffSummary` | Gather only the signals the record's state needs — never the archive-history walk for an Archived record — and classify them; the same verdict `ChangeSummary.handoff` carries |
| `lesson_fold_targets` | `root, id` | `Vec<String>` | Module context paths this change's lessons are folded into at archival; empty when the change is unreadable or owns no specs |
| `lifecycle_commit_scope` | `root, id` | `Result<LifecycleCommitScope, String>` | The one answer to which untracked paths a lifecycle commit for this change may stage: its workspace, its archive package, each affected spec's canonical file and companions, and the lifecycle ledgers — never its `affected_paths` prefixes; `Err` when the change cannot be loaded |
| `list_changes` | `root: &Path` | `Result<ChangeRoster, String>` | List active changes in stable ID order alongside the workspaces that could not be read; `Err` only when the changes directory itself is unreadable |
| `load_change` | `root: &Path, id: &str` | `Result<ChangeRecord, String>` | Load active or archived change state |
| `load_policy` | `root: &Path` | `Option<SddPolicy>` | Load `.specsync/sdd.json`; absence leaves existing projects unenforced |
| `module_context_path` | `module` | `String` | The single definition of where a module's accumulated lessons live, shared by surfacing and folding so they cannot disagree |
| `next_questions` | `record: &ChangeRecord` | `Vec<InterviewQuestion>` | Record-only unanswered interview questions. Does not ask about lessons or principles |
| `next_questions_for` | `root: &Path, record: &ChangeRecord` | `Vec<InterviewQuestion>` | Unanswered interview questions, including the draft-only constraints question when the rest of the interview is done and lessons or a principles file exist |
| `open_change_briefing` | `root: &Path, record: &ChangeRecord` | `OpenChangeBriefing` | Bounded lessons and principles for this change. Omits a scaffold-only context and never fails the caller |
| `record_bootstrap_paths` | `root: &Path` | `Result<(), String>` | Record the protected SDD paths this bootstrap created in `.specsync/bootstrap.json`, so initialization's own output is not reported as uncovered meaningful delivery; editing a recorded file revokes its exemption |
| `record_scoped_review` | `root, id, reviewer` | `Result<ScopedReviewRecord, String>` | Record one implementation-scoped review bound to current governed inputs; the reviewer may be the definition approver |
| `record_scoped_review_with_verdict` | `root, id, reviewer, verdict` | `Result<ScopedReviewRecord, String>` | Record an explicit passing or blocking review; only a current passing verdict permits finalization |
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
