## MODIFIED

### REQUIREMENT REQ-change-012

The lifecycle SHALL fail closed across coverage, canonical persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or terminal changes cover their own meaningful delivery paths; only closing-valid accepted or authenticated archived changes can satisfy successor evidence.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Active accepted workspaces require successful verification, matching closing approval, and recursive exact-or-successor-covered current-input validity; archives require authenticated historical integrity and enter current-input recursion only when selected as successors.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.

### REQUIREMENT REQ-change-014

The lifecycle SHALL preserve evidence, canonical truth, project-root isolation, bootstrap usability, and import safety through acceptance and archival.

Acceptance Criteria
- Accepted changes remain valid while every signed input is exact or every changed path/module obligation is governed by explicit closing-valid semantic succession evidence.
- Archive eligibility is attributable to the specific accepted change and its authenticated accepted snapshot rather than overlapping path coverage.
- Active and dated-archive workspaces are resolved by authenticated location-aware reads; duplicates and ambiguous locations fail closed.
- Archive preflights target historical integrity plus every active accepted root and dependent candidate before mutation, ignores unrelated authenticated archive drift, and keeps immediate uncommitted check/status consistent.
- Trusted policy lookup and meaningful changed paths are relative to the requested project root.
- Canonical specs require lifecycle coverage and adoption covers its protected policy bootstrap.
- A no-spec declaration cannot accompany a declared public-contract change.
- OpenSpec and Spec Kit imports reject symlinked files and directories.
- Rejected foreign imports leave no partial adoption policy, report, or imported content.
- The exact schema-v1 self-adoption record is the sole migration exception to the no-spec/public-contract rule.

### REQUIREMENT REQ-change-017

The lifecycle SHALL provide an audited recovery transition when accepted verification is genuinely stale after exact and semantic-successor validation.

Acceptance Criteria
- Reopen requires an explicit non-empty human actor and reason and rejects exact or successor-covered accepted evidence using the shared validity reason.
- Reopen moves stale accepted evidence to verifying so strict checks remain red until a fresh verification run succeeds.
- Prior definition approval, verification, closing approval, manifests, successor evidence, and accepted snapshot remain inspectable in append-only audit history.
- Reacceptance requires a new closing approval and does not reapply canonical deltas already accepted.
- Reacceptance rejects a definition digest that differs from the latest pre-reopen verification contract and directs further spec work to a new change workspace.
- A verifying already-applied change without audited reopen history fails closed.

### REQUIREMENT REQ-change-018

Audited reopening SHALL recognize only provable canonical acceptance and deterministic semantic succession recorded in trusted Git history.

Acceptance Criteria
- Definition digest, passed evidence, closing approval, stale delivery inputs, actor, and reason remain mandatory.
- An unreachable verification commit is allowed only when the exact accepted-transition anchor or explicit predecessor/path/module/digest successor evidence is provable from trusted history.
- ID order, timestamps, lexicographic ordering, and independent path/spec scope overlap are never succession evidence.
- Repeated trusted commits yielding identical canonical reconstructed evidence are deduplicated; distinct reconstructions fail as ambiguous.
- A descendant feature branch preserves squash-accepted evidence only when the remote default branch records the same accepted state, definition, delivery inputs, and closing approval.
- Arbitrary off-history evidence remains rejected.

### REQUIREMENT REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, explicit semantic succession, and validation of every current canonical contract it reapproves.

Acceptance Criteria
- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy successor governance even when its paths and specs overlap.
- A supported pre-approval supersede transition records a durable definition-bound predecessor edge with explicit path/module/predecessor-digest obligations.
- Closing evidence binds each adopted obligation only when the same successor has the module's semantic delta and an exact old/new transition from its trusted definition-signed base tree to its descendant unique accepted-transition tree; the acceptance commit's immediate parent is not the before tree.
- Every owner of a changed input requires its own same-successor path/module obligation; owner intersection and cross-record path/spec unions fail closed.
- A reopened canonical-applied change validates its current canonical modules without replaying its already-applied semantic delta.
- Strict project checks reject a reopened definition that reacceptance would reject.
- Definition reapproval keeps a canonical-applied reopened record in verifying so fresh evidence remains mandatory.
- Nested project history lookup anchors repository-relative workspace state paths at the Git repository top.
- Reopen rejects a request when the shared validator reports exact or successor-covered evidence.

### REQUIREMENT REQ-change-024

Strict lifecycle checking SHALL permit only explicit closing-valid terminal semantic successors to govern changed inputs of an accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria
- Draft, approved, implementing, verifying, failed, stale, tampered, no-spec, semantically empty, and partial successors never suppress predecessor errors.
- Accepted or authenticated archived successors selected as candidates require valid definition, verification, closing approval, history integration, and recursive exact-or-successor-covered current inputs; standalone archives require historical integrity without equality to today's inputs.
- Every changed input expands to one obligation per signed canonical owner and every obligation matches one exact predecessor/path/module/old-digest/new-digest tuple from the same successor.
- Multiple terminal successors may cover disjoint obligations, while cycles fail closed and completed validity results are memoized.

## ADDED

### REQUIREMENT REQ-change-032

Acceptance SHALL persist bounded canonical per-input manifests and explicit semantic succession evidence without changing legacy closing-approval bytes.

Acceptance Criteria
- Manifest schema, strictly sorted unique portable paths, sorted unique owners, supported kind/mode pairs, lowercase SHA-256 digests, and fixed entry/path/owner bounds validate fail closed.
- Candidate-scoped Git evidence is bounded to 100,000 paths, 4,096 bytes per path, and 64 MiB aggregate path bytes before payload/owner work; NUL-safe attribute batches reject active regular-file `filter`, `working-tree-encoding`, and `ident` conversion without blocking unrelated, symlink, or gitlink paths.
- Project freshness removes volatile paths and acceptance evidence removes noncovered paths before Git inspection while preserving every record-covered override, canonical-spec, tracked, and untracked input.
- Streaming discovery, index/split-index reads, and attribute output are bounded before full buffering; one candidate-filtered index parse and bounded retry return captured candidate topology/content for caller consumption.
- Positive Git detection makes every later command/parse failure fatal with bounded diagnostics; concurrent capped drains kill/reap overflow, effective-index fingerprinting honors `GIT_INDEX_FILE` plus split dependencies, and unrelated unmerged stages do not block selected candidates.
- Conversion attributes apply only to clean materialized tracked regular substitution; all false Git booleans disable fsmonitor; first authority binding requires its baseline ledger; definition regular files allow modes `100644` and `100755`.
- Canonical substitution rejects governed assume-unchanged, materialized fsmonitor-valid, materialized skip-worktree, and unmerged paths, preserves absent sparse index topology, and retries or fails closed when the index or split-index generation changes during inspection.
- Selected lifecycle definition artifacts are regular files; clean tracked, dirty, and untracked symlinks fail closed before any referent payload or size read.
- Source owners come only from the immutable post-delta canonical snapshot; unmapped production source paths fail and recognized governed test/fixture paths plus delivery metadata are exact-only.
- Symlinks hash exact portable target bytes and reject non-portable targets; gitlinks hash the exact index/tree object ID instead of checked-out directory topology.
- Full-entry topology digests use `specsync.acceptance-entry.v1` over path, kind, mode, and payload digest so same-payload mode/kind transitions remain distinct.
- New v1 aggregates use `specsync.acceptance-manifest.v1`, reproduce solely from canonical manifest entries, and equal the persisted acceptance input digest; legacy raw-content aggregates retain `specsync.acceptance-input-digest.v2`.
- Empty supersedes and absent manifest/succession fields are defaulted and omitted so old state JSON, verification JSON, definition digests, and closing digests remain byte-identical.
- Succession evidence uses `specsync.semantic-succession.v1` with bounded unique tuples strictly sorted by numeric sequence then full predecessor ID then path then module, portable paths, canonical modules, lowercase full-entry digests, conflict rejection, and exact one-to-one approved-obligation derivation.
- Stale legacy reconstruction uses one trusted accepted-transition anchor and deduplicates identical evidence content before deciding ambiguity.
- Enumerated standalone pre-CHG43 archives may authenticate only through the strictly sorted `.specsync/archive/legacy-baseline.json` ledger bound by CHG43's manifest-backed closing and trusted acceptance/history anchor; each entry binds a trusted cutoff, unique pre-cutoff introduction, canonical path, and exact domain-separated subtree digest, and never supplies accepted-transition, current-input, candidate, preflight, or semantic-succession validity.
- Before authority acceptance, the exact ledger digest is definition-bound and requires valid definition approval plus a canonical cutoff exactly equal to the authority base commit and ancestral to current history; accepted or archived authority upgrades to mandatory manifest-backed closing/history proof and cannot downgrade to bootstrap.
- Archive authenticates accepted-state bytes from the unique trusted accepted transition and prior closing evidence, never mints approval, restores byte-identical source artifacts after post-move failure, and fails closed for unverifiable legacy archives.
- Legacy archive and baseline snapshots union tracked index entries with present working-tree entries so sparse-absent inputs remain signed, dirty tracked symlinks use current topology, and a dirty or untracked missing authority baseline fails closed without preserving a stale binding.
- Active accepted check/status/reopen/archive eligibility consume one recursive cycle-safe current-input validator; archive integrity uses a separate history authenticator and separately keyed cache.
- Active accepted status reports exact, successor-covered, or stale; archived status reports authenticated-history or corrupt-history.


## MODIFIED

### SPEC SECTION Contract

1. Every meaningful SDD change moves through draft, approved, implementing, verifying, accepted, and archived states without bypasses.
2. Definition and closing approvals are portable records bound to deterministic SHA-256 digests.
3. Approved semantic deltas form the effective future contract without mutating canonical specs before acceptance.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell and rejects direct or indirect entry into every lifecycle command surface.
6. Verification evidence is bound to the tested commit and working-tree inputs, and registry-resolved effective contracts must validate before acceptance.
7. Invalid policy, unavailable coverage comparison, failed evidence, stale ordering gates, and protected sequence-ledger edits without lifecycle coverage fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, corrupt state fails closed, and archival failures remain retryable.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
11. Concurrent effective-contract validations use isolated temporary workspaces.
12. Stale accepted delivery evidence can return only to verifying through an explicit human actor and reason, while prior verification and closing evidence remain inspectable.
13. Historical collision acknowledgements are exact immutable accepted-or-archived evidence and numeric sequence width has no four-digit upper bound.
14. A fully valid later sequence claim supersedes only the sequence-ledger bytes in historical acceptance inputs; the current owner and every other covered input remain exact evidence.
15. Changed active accepted inputs are governed only by bounded signed manifests and explicit definition-bound terminal semantic succession tuples; archives are globally checked for authenticated historical integrity and are current-input validated only when selected as successor candidates.

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
| `ChangeRecord` | Durable machine state for one change workspace, including omitted-when-empty supersedes edges |
| `LegacyArchiveBaselineV1` | Definition- and closing-bound authority, cutoff, and sorted legacy archive subtree entries |
| `LegacyArchiveBaselineEntryV1` | Archive ID, canonical dated path, unique introduction commit, and exact subtree digest |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `ApprovalRecord` | Actor, timestamp, gate, digest, and optional note for one approval |
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, and stale/current input digests |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
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
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record mandatory definition approval |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract/manifest evidence |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `correct_interview_metadata` | `root, id, field, value, actor, reason` | `Result<CorrectionResult, String>` | Append a supported accepted-metadata correction and return the effective audited view |
| `effective_change_definition` | `root, record` | `Result<EffectiveChangeDefinition, String>` | Validate and project original metadata through its ordered correction history |
| `correction_history` | `root, record` | `Result<Vec<CorrectionRecord>, String>` | Load validated append-only correction records for inspection clients |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval, signed succession evidence, and atomically apply semantic deltas |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Authenticate and move an accepted workspace into the dated archive after graph preflight |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health, next action, and shared terminal-evidence conclusion |
| `check_project` | `root: &Path` | `SddCheckReport` | Validate active accepted current inputs, archive historical integrity, and report each state-appropriate evidence conclusion |
| `check_project_quiet` | `root: &Path` | `SddCheckReport` | Run the same fail-closed lifecycle check while suppressing configured command output for machine-consumable report protocols |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or enable SDD and import OpenSpec or Spec Kit artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |

**Exported Methods**

| Method | Description |
|--------|-------------|
| `as_str` | Return the stable serialized name for a change state, kind, or terminal-evidence validity |
| `parse` | Parse user-facing change-kind or artifact names into typed values |
| `file_name` | Resolve an adaptive artifact to its safe Markdown filename |
