---
spec: change.spec.md
---

# Requirements

### REQ-change-001

The system SHALL require exactly one human scope approval for each new meaningful change and SHALL
record finalization as automated terminal evidence rather than a second approval.

#### Acceptance Criteria

- The scope approval exposes no force or emergency bypass.
- Scope approval binds only stable intent, acceptance criteria, public-contract/risk declarations,
  and affected spec/path/dependency/supersession scope.
- Implementation notes, task progress, test/evidence plans, semantic-delta materialization,
  canonical materialization, and lifecycle metadata preserve scope approval but invalidate
  automated execution evidence and scoped review when their bound digest changes.
- An added, removed, or replaced affected spec/path, acceptance criterion, dependency,
  supersession obligation, or changed intent/classification requires renewed scope approval and
  status explains the exact change in plain language.
- Independent scoped review and current verification remain mandatory before same-PR finalization.
- The historical CHG-0068 definition preimage remains explicitly unavailable and no equivalence
  proof is claimed; one compile-time allowlist freezes its exact source approval, adoption
  commit/blob, stable-scope digest, authorization, and non-material classifications without
  appending a second approval.
- Historical closing approvals remain readable and verifiable without being required for new
  workflow-version-2 changes.

### REQ-change-002

The system SHALL validate implementation against canonical contracts plus approved active semantic deltas.

#### Acceptance Criteria

- Only `change check` materializes approved semantic deltas into canonical files, before scoped
  review and finalization.
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

The lifecycle SHALL fail closed across coverage, canonical persisted closing evidence, semantic-delta validation, dependency ordering, and supported canonical version formats.

Acceptance Criteria
- Only implementing, verifying, or terminal changes cover their own meaningful delivery paths; only closing-valid accepted or authenticated archived changes can satisfy successor evidence.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Active accepted workspaces require successful verification, matching closing approval, and recursive exact-or-successor-covered current-input validity; archives require authenticated historical integrity and enter current-input recursion only when selected as successors.
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

The lifecycle SHALL preserve evidence, canonical truth, project-root isolation, bootstrap usability,
and import safety through acceptance and archival.

Acceptance Criteria

- Accepted changes remain valid while every signed input is exact or every changed path/module
  obligation is governed by explicit closing-valid semantic succession evidence.
- Archive eligibility is attributable to the specific accepted change and its authenticated
  accepted snapshot rather than overlapping path coverage.
- Active and dated-archive workspaces are resolved by authenticated location-aware reads;
  duplicates and ambiguous locations fail closed.
- Archive preflights target historical integrity plus every active accepted root and dependent
  candidate before mutation, ignore unrelated authenticated archive drift, and keep immediate
  uncommitted check/status consistent.
- Trusted policy lookup and meaningful changed paths are relative to the requested project root.
- Canonical specs require lifecycle coverage and adoption covers its protected policy bootstrap.
- A no-spec declaration cannot accompany a declared public-contract change.
- OpenSpec and Spec Kit imports reject symlinked files and directories.
- Rejected foreign imports leave no partial adoption policy, report, or imported content.
- The exact schema-v1 self-adoption record is the sole migration exception to the
  no-spec/public-contract rule.
- A legacy archive baseline authority that covers the baseline ledger signs that exact ledger path
  in its acceptance manifest even though other dated archive paths remain volatile.

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
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an unchanged accepted workspace integrated on the remote default branch.
- Unintegrated heads, changed scoped inputs, stale contracts, mismatched closing approvals, nonancestor evidence, and ambiguous merges fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and executable modes remain exact.

### REQ-change-017

The lifecycle SHALL provide an audited recovery transition when accepted verification is genuinely stale after exact and semantic-successor validation.

Acceptance Criteria
- Reopen requires an explicit non-empty human actor and reason and rejects exact or successor-covered accepted evidence using the shared validity reason.
- Reopen moves stale accepted evidence to verifying so strict checks remain red until a fresh verification run succeeds.
- Prior definition approval, verification, closing approval, manifests, successor evidence, and accepted snapshot remain inspectable in append-only audit history.
- Reacceptance requires a new closing approval and does not reapply canonical deltas already accepted.
- Reacceptance rejects a definition digest that differs from the latest pre-reopen verification contract and directs further spec work to a new change workspace.
- A verifying already-applied change without audited reopen history fails closed.

### REQ-change-018

Audited reopening SHALL recognize only provable canonical acceptance and deterministic semantic succession recorded in trusted Git history.

Acceptance Criteria
- Definition digest, passed evidence, closing approval, stale delivery inputs, actor, and reason remain mandatory.
- An unreachable verification commit is allowed only when the exact accepted-transition anchor or explicit predecessor/path/module/digest successor evidence is provable from trusted history.
- ID order, timestamps, lexicographic ordering, and independent path/spec scope overlap are never succession evidence.
- Repeated trusted commits yielding identical canonical reconstructed evidence are deduplicated; distinct reconstructions fail as ambiguous.
- A descendant feature branch preserves squash-accepted evidence only when the remote default branch records the same accepted state, definition, delivery inputs, and closing approval.
- Arbitrary off-history evidence remains rejected.

### REQ-change-019

Verification SHALL recognize a non-removed requirement or spec-section delta item as semantic acceptance evidence when observable acceptance criteria are present.

Acceptance Criteria

- A section-only modified delta can pass with an empty requirement-ID list.
- Requirement evidence mapping remains mandatory for every collected requirement ID.
- A failed configured command, missing semantic acceptance evidence, and missing requirement evidence produce distinct diagnostics.

### REQ-change-020

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

Strict lifecycle checking SHALL permit only explicit closing-valid terminal semantic successors to govern changed inputs of an accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria
- Draft, approved, implementing, verifying, failed, stale, tampered, no-spec, semantically empty, and partial successors never suppress predecessor errors.
- Accepted or authenticated archived successors selected as candidates require valid definition, verification, closing approval, history integration, and recursive exact-or-successor-covered current inputs; standalone archives require historical integrity without equality to today's inputs.
- Every changed input expands to one obligation per signed canonical owner and every obligation matches one exact predecessor/path/module/old-digest/new-digest tuple from the same successor.
- Multiple terminal successors may cover disjoint obligations, while cycles fail closed and completed validity results are memoized.

### REQ-change-025

Semantic-delta preparation and application SHALL resolve canonical spec and companion paths through the committed registry.

Acceptance Criteria

- Registry-backed non-conventional module paths receive semantic spec and requirements updates.
- Conventional paths remain the fallback when no mapping exists.
- Unsafe registry paths fail closed.

### REQ-change-026

The lifecycle SHALL treat canonical numeric sequence claims and historical collision acknowledgements as protected exact repository evidence across arbitrarily wide numeric sequences.

Acceptance Criteria

- Numeric change sequences contain at least four ASCII digits, use exactly four zero-padded digits below 10000, and use unpadded decimal digits at or above 10000.
- Successor identity ordering compares parsed numeric sequence first and full canonical ID second, so `CHG-10000-*` follows `CHG-9999-*` while acknowledged same-sequence collisions remain deterministic.
- Malformed, noncanonical-width, and numerically unrepresentable IDs fail closed instead of participating in successor ordering.
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
- When acknowledged legacy collision members signed one canonical committed ledger for their shared sequence, reconstruction reuses those exact historical bytes instead of substituting each member's ID.
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

### REQ-change-033

The verified lifecycle SHALL support human-authorized, append-only correction of an exact
acceptance-input canonical owner for an audited reopened, already-applied change without changing
semantic scope or replaying canonical deltas.

Acceptance Criteria

- `change correct-owner` requires an exact portable path, canonical module, non-empty actor, and
  non-empty reason.
- The target is canonical-applied, verifying through an audited reopen, and unchanged from the
  reopened definition except for validated ownership-correction entries.
- The path is already covered by the original affected paths, and the named module's current
  canonical spec explicitly owns that exact source path.
- Corrections are immutable, sequenced, definition-bound records; duplicates, removals, malformed
  values, tampering, and ambiguous ownership fail before mutation.
- Original affected specs, semantic deltas, approvals, reopen evidence, and prior verification are
  preserved byte-for-byte.
- The corrected definition requires explicit reapproval, fresh verification, and closing approval.
- Acceptance adds the corrected module only to the exact manifest entry's sorted owner set and
  never reapplies canonical deltas.
- Records without ownership corrections preserve their existing serialized bytes and digests.

### REQ-change-034

Concurrent accepted sequence claims SHALL be reconciled without rewriting either immutable
accepted history, through an exact sorted collision acknowledgement and a later lifecycle-governed
sequence claim that owns the merged ledger transition.

Acceptance Criteria

- Every acknowledged collision member is an immutable accepted or archived record.
- The acknowledgement lists the complete exact sorted ID set for the duplicated numeric sequence.
- Neither accepted definition, approval history, verification record, nor canonical delta is
  renumbered, replayed, or rewritten to resolve the collision.
- A later approved and accepted canonical change advances the sequence ledger and governs only the
  reconciled ledger transition.
- Strict lifecycle validation passes without masking stale non-ledger delivery inputs.

### REQ-change-035

Acknowledged immutable sequence collisions SHALL remain valid while an accepted member completes an
audited delivery-only reopen, provided its already-applied definition, prior passing verification,
superseded closing approval, and exact accepted-to-verifying transition remain valid.

Acceptance Criteria

- A structurally valid audited reopen keeps the collision acknowledgement usable during verification.
- Missing, tampered, definition-stale, unapplied, or non-verifying reopen evidence remains mutable.
- The reopened member still requires fresh verification and a new closing approval before acceptance.
- Collision IDs are never renumbered, deleted, or silently rewritten.

### REQ-change-036

Stale accepted-change verification diagnostics SHALL name the offending delivery input and state
the concrete remediation, without changing the underlying freshness model.

Acceptance Criteria

- A changed covered input with no covering accepted or archived successor reports the input path,
  its owner module, and the `specsync change reopen <id>` remediation.
- A changed covered input whose only covering successors carry stale evidence of their own reports
  the input path, its owner module, and the sorted covering successor change IDs, and directs the
  operator to verify and accept a covering successor or reopen the accepted change.
- A covered input that disappeared from the current inventory reports the missing path and the
  restore-or-reopen remediation; a changed exact-only input reports the path and the audited-reopen
  remediation; missing delivery-input evidence keeps its established phrase and gains the reopen
  remediation.
- Every stale reason remains deterministic: sorted successor IDs, no timestamps, and no
  environment-dependent content.
- The `accepted change verification is stale for current delivery inputs` check prefix, the
  terminal-evidence validity values, and every freshness predicate remain unchanged.

### REQ-change-037

Accepted-change archival SHALL trust squash-merged evidence when an in-history commit records the
change as accepted with byte-identical state, verification, and approvals, so discarding the
original acceptance-transition commit in a squash merge never blocks archival.

Acceptance Criteria

- Only commits reachable from `HEAD` or the remote default qualify as recording anchors.
- Byte equality, accepted-state identity, and projection checks remain mandatory per anchor.
- The exactly-one-eligible rule still fails closed on missing or ambiguous evidence.
- First-acceptance transition anchors and the archived `accepted-state.json` scan keep priority;
  the recording-anchor fallback runs only when they find nothing.
- A change with no matching in-history accepted record remains unarchivable.

### REQ-change-038

Legacy acceptance-manifest reconstruction SHALL assign the exact delivery owner to
production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers
validate without per-repo remediation.

Acceptance Criteria

- Only pre-manifest (legacy) reconstruction is relaxed; current acceptance stays fail-closed.
- Historical aggregate reproduction, closing-approval authentication, and the exactly-one
  distinct reconstruction rule are unchanged.
- The exact delivery owner assignment appears only in reconstructed manifests, never in newly
  signed ones.
- No new command, state transition, or persisted evidence format is introduced.

### REQ-change-039

The verified lifecycle SHALL allow one transactional batch of audited exact acceptance-owner
corrections so rollout-era gaps with many omitted owners need only one reapprove → verify → accept
cycle, without weakening per-entry scope, ownership, or append-only sequencing rules.

Acceptance Criteria

- A batch may be supplied as repeated path/module pairs, a manifest file, or `--all-missing` with
  one canonical module.
- Every entry is validated independently against the same rules as a single `correct-owner`.
- Each accepted entry becomes its own sequenced `AcceptanceOwnerCorrection` record.
- If any entry is invalid, the command fails closed and persists no corrections from the batch.
- Single-path `correct-owner` remains supported and equivalent to a one-entry batch.

### REQ-change-040

SpecSync SHALL provide a native, idempotent migration that backfills 5.1 reopening digest
fields on 5.0.1-era change ledgers with a verification pass before any write.

Acceptance Criteria

- `stale` always reproduces the embedded prior verification's acceptance-input digest.
- `current` comes from the superseding verification's signed digest, else a live recomputation.
- Records already carrying both fields are never modified; re-running is a no-op.
- A reopening that cannot be repaired deterministically fails without mutating its ledger.
- Repaired ledgers re-parse and re-validate before the write lands.
- `check` on an un-migrated ledger prints the `specsync migrate 5.0` remediation, not a raw
  serde error.

### REQ-change-041

Canonical module path resolution SHALL fall back to default `specs/<module>/<module>.spec.md` paths when the local registry file is missing or an inert stub, without weakening fail-closed behavior for invalid non-inert registries.

Acceptance Criteria

- An inert 5.0.1-era empty registry stub does not block `canonical_module_paths` resolution.
- Conventional `specs/<module>/<module>.spec.md` paths remain the fallback when the registry is missing or inert and no mapping applies.
- A non-inert unparsable local registry still fails closed with the exact pre-fix diagnostic `failed to parse local registry {path} while resolving `{module}``.
- Named registries with safe mappings continue to win over the conventional fallback.

### REQ-change-042

Git candidate inspection SHALL deduplicate repeated stage-zero paths only when their normalized
mode and object identity are exact, while conflicting observations fail closed.

Acceptance Criteria

- A stage-zero path returned through overlapping bounded pathspec batches is represented once when
  every observed mode and normalized object ID is identical.
- A repeated path with a differing mode fails closed without replacing the first observation.
- A repeated path with a differing object ID fails closed without replacing the first observation.
- Parent-directory and exact-child candidate scopes remain valid across pathspec batch boundaries.
- Deterministic output bounds, unresolved-stage rejection, malformed metadata rejection, and
  out-of-scope path rejection remain unchanged.

### REQ-change-043

The verified lifecycle SHALL provide one discoverable workflow and one file layout with one human
scope approval for every new change.

Acceptance Criteria

- The path is `change new` → one `change approve` → implement → `change check` → ordinary PR
  review → `change finalize` → GitHub merge.
- There is no lifecycle-mode selection, second SpecSync approval, closing approval gate, alternate archive
  layout, or SpecSync merge command.
- Scope approval binds stable intent, acceptance criteria, public-contract/risk declarations, and
  affected spec/path/dependency/supersession scope—not implementation, test/evidence,
  semantic-delta materialization, canonical materialization, or lifecycle metadata.
- Non-material execution/evidence changes preserve scope approval, invalidate their separate
  execution digest, and require fresh automated validators plus the one scoped review.
- A demonstrable stable-scope change requires renewed approval, and status explains each added or
  removed criterion, affected spec/path, dependency, supersession obligation, or changed intent in
  plain language.
- Checking off an already-approved task records implementation progress without changing either
  scope or execution digests; changing task text preserves scope but stales execution evidence.
- Every status result prints exactly one explicit next action.
- Expected missing-history ancestry probes never leak raw Git fatal diagnostics into status output.
- Explicit `--strict`, project policy, or release/security classification adds full-history,
  full-suite, security, or release validators to the same verification evidence without changing
  the state machine, workflow, approval count, commands, finalization, archive, or layout.
- Existing two-approval records remain readable and verifiable without reinterpretation or resigning.
- Workflow-v2 adoption records one immutable project cutoff at the stable comparison-base ancestor
  when available; its introduction remains valid after squash/rebase, and a workflow-v1 record
  remains eligible only when that exact ID/version with omitted or explicit version-1 origin
  existed at the trusted cutoff, so omitting both version fields before first reachability cannot
  route a new change through legacy commands.
- Every bounded workflow-v2 baseline-touching commit and readable parent retains the exact
  introduction bytes, so rewrite→restore history cannot conceal a changed cutoff.
- Workflow-origin history boundedly follows every reachable canonical dated archive state path for
  the exact change ID, so archive→reopen→rearchive moves preserve the immutable creation anchor.
- The one CHG-0068 adoption fails closed and requests full trusted history when its immutable
  allowlisted commit/blob anchor is unavailable.

### REQ-change-044

The lifecycle SHALL finalize and archive a change on its implementation PR through one
metadata/archive-only commit without repeating implementation validation.

Acceptance Criteria

- `change finalize` requires the approved implementation parent to have every required green check.
- The finalization child may change only exact approved lifecycle/archive paths and must preserve
  code, canonical spec, requirements, tests, configuration, and delivery-tree relationships.
- Finalization applies semantic deltas, writes accepted state, validates bidirectional ownership,
  and moves the same package to `.specsync/archive/changes/YYYY-MM-DD-<id>/` transactionally.
- A process interruption between terminal archive-file writes is recovered from the transaction
  journal before retry, including after a calendar rollover.
- A fresh clone after squash or rebase merge authenticates the exact surviving archived subtree
  when the original implementation commit object is no longer reachable.
- The lightweight archive lane validates parent checks, diff classification, unchanged tree,
  archive integrity, ownership, and finalization digest and reports success to required CI.
- Product tests and independent scoped review are not rerun for a valid archive-only child.
- `change finalize` makes the PR ready; GitHub alone performs the merge.

### REQ-change-045

Lifecycle validation SHALL reuse a deterministic invocation-scoped snapshot and bounded evidence
queries without weakening fail-closed historical conclusions.

Acceptance Criteria

- Active/archive records, canonical owners, Git comparison state, candidate entries, and completed
  terminal evidence are loaded or computed at most once per invocation key.
- Git and evidence queries have deterministic bounds independent of overlapping path scopes.
- Dependency and successor graphs use stable ordering.
- Canonical owner batches are validated in one pass.
- Warm and cold validation return identical errors, warnings, path coverage, and evidence validity.

### REQ-change-046

Agent-authored changes SHALL receive one independent scoped review of implementation evidence before
finalization.

Acceptance Criteria

- Review input contains only the change package, implementation diff, canonical semantic delta, and
  targeted evidence.
- The result binds the implementation parent commit, those input digests, an explicit pass/block
  verdict, a stable reviewer claim distinct from the scope approver, and the exact required
  GitHub Actions check whose authenticated result is proven again by finalization.
- Every review attempt is append-only; `review.json` is only the latest projection and cannot erase
  a prior blocking result.
- Every intervening commit is inspected against every parent; any implementation change, including
  change-then-revert history, stales the review, while the metadata/archive-only finalization commit
  does not rerun or stale it. Native and hosted validators load the same committed descendant,
  parent, output, and timeout limits.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured
  scoped-review check runs.
