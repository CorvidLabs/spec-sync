---
spec: change.spec.md
---

# Requirements

### REQ-change-001

The system SHALL require definition and closing human approval gates for every meaningful change.

#### Acceptance Criteria

- Neither gate exposes a force or emergency bypass.
- Artifact changes invalidate the definition approval digest.

### REQ-change-002

The system SHALL validate implementation against canonical contracts plus approved active semantic deltas.

#### Acceptance Criteria

- Canonical files remain unchanged before acceptance.
- Overlapping active deltas are detected before implementation.

### REQ-change-003

The system SHALL connect durable requirement IDs to technical specs, tests, and verification evidence.

#### Acceptance Criteria

- New or modified requirements use SHALL statements and acceptance criteria.
- Acceptance fails when spec-changing work has no requirement evidence.

### REQ-change-004

The system SHALL support equivalent human CLI and structured agent workflows.

#### Acceptance Criteria

- Every change operation has machine-readable JSON output.
- The same deterministic interview drives terminal and agent integrations.

### REQ-change-005

The system SHALL preserve unrelated canonical Markdown when applying semantic blocks.

#### Acceptance Criteria
- Modifying or removing the final requirement before a higher-level heading preserves that heading and all following content.
- Failed preparation leaves canonical files byte-for-byte unchanged.
- An interrupted multi-file acceptance is recovered from its transaction journal before the next lifecycle mutation.

### REQ-change-006

The system SHALL bind verification evidence to every tested working-tree input.

#### Acceptance Criteria
- Source, test, configuration, or contract edits after verification invalidate acceptance even when HEAD is unchanged.
- Failed verification remains an error until fresh successful evidence is recorded.

### REQ-change-007

The system SHALL fail closed when lifecycle enforcement cannot be evaluated.

#### Acceptance Criteria
- Malformed policy and unavailable changed-path comparison fail unified checking.
- A successful changed-path comparison with no output is valid empty coverage evidence.
- Effective-contract validation runs during verification and acceptance.
- Oversized lifecycle artifacts and unsafe, traversing, or symlink-escaping project paths are rejected.

### REQ-change-008

The system SHALL apply concurrent change semantics in declared dependency order.

#### Acceptance Criteria
- Effective deltas are topologically ordered regardless of change ID.
- Dependency and conflict gates are rechecked immediately before acceptance.
- Path coverage matches complete path components rather than arbitrary prefixes.
- Lifecycle mutations serialize through an operating-system lock so concurrent creation cannot duplicate IDs.

### REQ-change-009

The system SHALL keep definitions and persisted lifecycle state trustworthy through approval, adoption, and archival.

#### Acceptance Criteria
- Definition approval rejects missing, malformed, or cross-module semantic requirements before recording evidence.
- Corrupt active state fails unified checking instead of disappearing from enforcement.
- Failed archive moves preserve the accepted active workspace so archival can be retried.
- Only accepted or archived requirement removals become permanent tombstones.
- Spec Kit adoption does not classify native companion-only spec directories as feature workspaces.

### REQ-change-010

The system SHALL require lifecycle coverage for common root action, manifest, and dependency lock files by default.

#### Acceptance Criteria
- Root Action configuration and supported ecosystem manifest or lockfile changes are meaningful paths.
- Component-boundary matching continues to exclude similarly prefixed unrelated files.

### REQ-change-011

The system SHALL isolate temporary effective-contract state across concurrent validations.

Acceptance Criteria
- Parallel validations in one process allocate distinct scratch paths.
- Each validation removes only its own scratch workspace.

### REQ-change-012

The lifecycle SHALL fail closed across coverage, persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or accepted changes cover meaningful delivery paths.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Accepted workspaces require fresh successful verification and matching closing approval evidence.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.

### REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval, history, and verification evidence before using it, with one environment-independent verification-freshness decision.

Acceptance Criteria
- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed evidence, a matching effective contract digest, and a matching project-input digest in local and hosted checks.
- A descendant verification commit remains current only when every intervening commit and every parent edge changes exactly `state.json`, `verification.json`, or `verification-attempts.json` under a canonical active-change ID and the persisted state/evidence remains consistent.
- Source-change-then-revert history, ambiguous merges, nonancestor history, malformed paths, and any broader volatile or lifecycle path fail closed.

### REQ-change-014

The lifecycle SHALL preserve evidence, canonical truth, project-root isolation, bootstrap usability, and import safety through acceptance and archival.

Acceptance Criteria
- Accepted changes remain valid only while verification matches current delivery inputs, and archive revalidates the same evidence.
- Archive eligibility is attributable to the specific accepted change rather than overlapping path coverage from another change.
- Trusted policy lookup and meaningful changed paths are relative to the requested project root.
- Canonical specs require lifecycle coverage and adoption covers its protected policy bootstrap.
- A no-spec declaration cannot accompany a declared public-contract change.
- OpenSpec and Spec Kit imports reject symlinked files and directories.
- Rejected foreign imports leave no partial adoption policy, report, or imported content.
- The exact schema-v1 self-adoption record is the sole migration exception to the no-spec/public-contract rule.

### REQ-change-015

Unified lifecycle checking SHALL support a protocol-clean reporting mode without weakening verification.

Acceptance Criteria
- Reporting mode still executes every configured verification command and records failures.
- Reporting mode suppresses child command stdout and stderr so the caller can emit one machine-consumable document.
- Normal check and explicit change verification retain their diagnostic output.

### REQ-change-016

The lifecycle SHALL preserve accepted closing evidence and supported verifying evidence across repository-integrated commits without accepting unintegrated, altered, or historically tainted evidence.

Acceptance Criteria
- Normal verification-commit ancestry remains mandatory proof and uses identical local and CI semantics.
- Every intervening commit is inspected against every parent with NUL-delimited portable paths; a net tree diff cannot hide a governed change and later revert.
- Only supported verification persistence beneath canonical active-change IDs may follow verification without invalidating it; archive, approvals, tasks, definitions, sequence, hashes, locks, configuration, policy, specs, source, tests, build, and cache paths are rejected.
- Matching effective contract and project-input digests plus consistent state, verification, and latest-attempt evidence remain mandatory.
- A squash fallback requires matching scoped inputs and an unchanged accepted workspace already integrated on the
  remote default branch.
- Unintegrated heads, changed scoped inputs, stale contracts, mismatched closing approvals, nonancestor evidence, and ambiguous merges fail closed.
- Squash-integrated accepted workspaces remain archivable.
- Digest fields are versioned, domain-separated, and length-framed so file boundaries cannot be forged with embedded
  NUL bytes.
- Binary bytes remain exact, while stable file kind and executable-mode evidence invalidate relevant delivery changes.
- Cross-platform topology verification is independent of a runner's global line-ending checkout policy.

### REQ-change-017

The lifecycle SHALL provide an audited recovery transition when accepted verification becomes stale because governed delivery inputs changed.

Acceptance Criteria
- Reopen requires an explicit non-empty human actor and reason and rejects non-stale accepted evidence.
- Reopen moves accepted evidence to verifying so strict checks remain red until a fresh verification run succeeds.
- Prior definition approval, verification, and closing approval evidence remain inspectable in append-only audit history.
- Reacceptance requires a new closing approval and does not reapply canonical deltas already accepted.
- Reacceptance rejects a definition digest that differs from the latest pre-reopen verification contract and directs further spec work to a new change workspace.
- A verifying already-applied change without audited reopen history fails closed.

### REQ-change-018

Audited reopening SHALL recognize canonical acceptance recorded in current Git history after squash integration or complete later canonical governance.

Acceptance Criteria

- Definition digest, passed evidence, closing approval, stale delivery inputs, actor, and reason remain mandatory.
- An unreachable verification commit is allowed only when current history records acceptance or later recorded canonical changes govern every affected spec and path.
- A descendant feature branch preserves squash-accepted evidence when the remote default branch records the accepted state and the definition, delivery inputs, and closing approval remain current.
- Arbitrary off-history evidence remains rejected.

### REQ-change-019

Verification SHALL recognize a non-removed requirement or spec-section delta item as semantic acceptance evidence when observable acceptance criteria are present.

Acceptance Criteria

- A section-only modified delta can pass with an empty requirement-ID list.
- Requirement evidence mapping remains mandatory for every collected requirement ID.
- A failed configured command, missing semantic acceptance evidence, and missing requirement evidence produce distinct diagnostics.

### REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, semantic successor governance, and validation of every current canonical contract it reapproves.

Acceptance Criteria
- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy the canonical-successor fallback, even when its affected paths and specs overlap.
- A later recorded semantic canonical change can satisfy successor governance for every overlapping affected spec and path.
- A reopened canonical-applied change validates its current canonical modules without replaying its already-applied semantic delta.
- Strict project checks reject a reopened definition that reacceptance would reject.
- Definition reapproval keeps a canonical-applied reopened record in the verifying state so fresh evidence remains mandatory.
- Nested project history lookup anchors repository-relative workspace state paths at the Git repository top.
- Reopen rejects a request when current delivery inputs match accepted evidence, regardless of another closing-validity failure.

### REQ-change-021

The lifecycle SHALL preserve the existing canonical Change Log table schema when acceptance appends its audit row.

Acceptance Criteria

- A `Version | Date | Changes` table receives the post-bump canonical version, current date, and accepted change description in that order.
- A `Date | Author | Change` table receives the current date, `SpecSync`, and accepted change description in that order.
- Existing two-column `Date | Change` tables retain their current output.
- The appended row has the same number and order of cells as every recognized existing header.

### REQ-change-022

The lifecycle SHALL prevent parallel branches from silently merging duplicate numeric change sequences while preserving exact historical collision evidence.

Acceptance Criteria

- Active and archived records are scanned together by numeric `CHG-NNNN` sequence.
- Unacknowledged duplicate sequences fail with every conflicting full ID and path.
- Repository-backed sequence claims make independent next-ID claims conflict during Git integration.
- Existing accepted collisions can be baselined exactly without rewriting accepted state or evidence.

### REQ-change-023

Verification SHALL reject recursive lifecycle checks and preserve retryable attempt history without weakening unrelated gates.

Acceptance Criteria

- Direct and indirect re-entry fails once before repeated child execution.
- Native-only verification executes once.
- Failed attempts remain inspectable and a corrected retry can record passed latest evidence.
- Other failed or stale changes continue failing closed.

### REQ-change-024

Strict lifecycle checking SHALL permit an exact current canonical successor to replace a stale accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria

- An implementing successor with current approved definition evidence can govern every affected module and path while reaching verification.
- A verifying successor requires current passed evidence.
- Draft, no-spec, partial, failed, abandoned, and stale-definition successors never suppress predecessor errors.
- Accepted successors leave strict validation clean while preserving predecessor history.

### REQ-change-025

Semantic-delta preparation and application SHALL resolve canonical spec and companion paths through the committed registry.

Acceptance Criteria

- Registry-backed non-conventional module paths receive semantic spec and requirements updates.
- Conventional paths remain the fallback when no mapping exists.
- Unsafe registry paths fail closed.

### REQ-change-026

The lifecycle SHALL treat sequence claims and historical collision acknowledgements as protected exact repository evidence across arbitrarily wide numeric sequences.

Acceptance Criteria

- Numeric change sequences contain at least four ASCII digits and support values beyond 9999.
- The committed sequence ledger always requires lifecycle coverage even when `.specsync/` is ignored.
- Every newly allocated change automatically includes its generated sequence-ledger claim in its affected path scope.
- An acknowledgement matches the exact currently located ID set and remains valid only when every member is accepted or archived.
- Removed IDs, added IDs, single surviving records, and draft, approved, implementing, or verifying collision members fail closed.

### REQ-change-027

Configured verification SHALL reject direct and indirect entry into every SpecSync lifecycle command surface.

Acceptance Criteria

- Nested `check`, `change`, and `lifecycle` commands fail before performing validation or mutation.
- Native verification commands remain unaffected and execute once.
- The diagnostic names the configured parent command.

### REQ-change-028

Effective contract and canonical-successor validation SHALL use canonical repository resolution without redundant full-project hashing.

Acceptance Criteria

- Effective validation reads registry-backed canonical specs through the safe project-path resolver.
- Conventional canonical paths remain the fallback when no registry mapping exists.
- Unsafe registry mappings fail closed before effective validation.
- The current project digest is computed at most once per canonical-successor candidate scan.

### REQ-change-029

Acceptance evidence SHALL preserve historical validity across valid later sequence claims without weakening current sequence-ledger integrity.

Acceptance Criteria

- Creating a later valid lifecycle record does not stale an earlier accepted record solely because the sequence ledger advanced.
- Historical reconstruction uses the earlier owner and includes only collision acknowledgements whose sequence is not later than that owner.
- The current sequence owner remains bound to the exact current ledger content.
- Malformed claims, claims without a workspace, non-maximum claims, duplicate sequences, and invalid collision acknowledgements fail closed.
- Every covered path other than a valid later-owned sequence ledger remains acceptance-digest input.

### REQ-change-030

Lifecycle enforcement SHALL preserve explicit user scope, precise canonical companion coverage, registry authority, policy opt-out boundaries, and native verification commands while retaining fail-closed SpecSync recursion protection across Cargo manifest selection.

Acceptance Criteria

- Generated sequence bookkeeping does not satisfy or suppress the interview question for source, test, documentation, or configuration scope.
- Registry-resolved modules cover only their exact canonical spec and the standard `requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md` companions; unrelated siblings and the containing directory are not implicitly covered.
- Both registry files remain protected lifecycle inputs because they control canonical writes.
- An explicitly disabled SDD policy returns without sequence-ledger validation.
- Native `cargo run -- check` commands remain allowed unless Cargo is actually selecting SpecSync by manifest identity, `default-run`, binary, or package.
- Both `--manifest-path <path>` and `--manifest-path=<path>` participate in Cargo identity detection, and unsafe explicit manifest paths fail closed.
- Recursive lifecycle verification is rejected before verification-attempt history or lifecycle state mutates.
- Direct SpecSync lifecycle commands remain rejected.
- Cargo argument parsing tolerates ordinary whitespace, quoted values, and trailing comments without shell execution.

### REQ-change-031

The deterministic change interview SHALL preserve free-text user intent exactly while parsing multi-value scope answers only through explicit, question-appropriate list semantics.

Acceptance Criteria

- A scalar acceptance criterion containing commas or line breaks remains one criterion with its punctuation and internal text preserved.
- A JSON array of strings explicitly represents multiple acceptance criteria.
- Affected-spec and affected-path questions retain comma/newline list parsing.
- Boolean and scalar interview answers retain their existing semantics.
- Persisted state and rendered change documents preserve the parsed criterion text without silent fragments.

### REQ-change-032

The verified lifecycle SHALL support human-authorized, append-only correction of explicitly
supported accepted interview metadata without rewriting history or replaying canonical deltas.

Acceptance Criteria

- Only `public_contract` and `architecture_risk` accept normalized `yes` or `no` corrections.
- Every event preserves the original value and records the prior effective value, corrected value,
  actor, non-empty reason, timestamp, added artifacts, prior gate evidence, and portable
  domain-separated prior/corrected metadata-view digests.
- Effective answers and selected artifacts are derived from a validated ordered correction ledger;
  artifacts are monotonic and malformed, truncated, reordered, unsupported, or tampered history
  fails closed.
- A correction moves an accepted canonically applied change to verifying and requires fresh
  definition approval, verification, and closing approval.
- Corrected acceptance prepares no canonical semantic-delta application, and repeated corrections
  preserve all earlier evidence across portable checkouts and squash integration.
