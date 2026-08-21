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
- Only implementing, verifying, or terminal changes cover their own meaningful delivery paths; archived packages present in the current delivery (same-PR finalize tips) cover their affected_paths for path coverage even when no active change remains; only closing-valid accepted or authenticated archived changes can satisfy successor evidence.
- Local coverage includes committed, staged, unstaged, and untracked meaningful paths.
- Active accepted workspaces require successful verification, matching closing approval, and recursive exact-or-successor-covered current-input validity; archives require authenticated historical integrity and enter current-input recursion only when selected as successors.
- Delta modules, operation headings, tombstones at acceptance, and transitive dependency order are validated deterministically.
- Integer and semantic spec versions advance without losing their format.

### REQ-change-013

The lifecycle SHALL reject untrusted or corrupt persisted workspace identity, scope, approval, and
verification evidence before using it, with one environment-independent verification-freshness
decision.

Acceptance Criteria

- Loaded change IDs match their requested workspace and remain a single validated component.
- Persisted affected spec names are validated before delta paths are constructed.
- Unreadable or malformed historical tombstone deltas and approval ledgers fail closed.
- Verifying workspaces require passed evidence, a matching effective contract digest, and a
  matching project-input digest in local and hosted checks.
- Freshness is decided by content equality alone; no commit ancestry, intervening-commit
  inspection, or path allowlist participates in the decision.

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

The lifecycle SHALL preserve accepted closing evidence across repository-integrated commits without
accepting unintegrated or altered evidence, while verifying evidence SHALL be judged on content
alone.

Acceptance Criteria

- Verification currency does not depend on commit ancestry, on inspecting intervening commits, or
  on restricting which paths may change after verification. Provenance of that kind is recorded by
  `attest`, keyed to commit SHAs, and is outside this tool.
- `verification.commit` is retained as an informational correlation key and is never a gate; a
  squash merge that discards the recorded commit does not invalidate the evidence.
- Matching effective contract and project-input digests plus consistent state, verification, and
  latest-attempt evidence remain mandatory.
- A squash fallback for accepted closing evidence still requires matching scoped inputs and an
  unchanged accepted workspace integrated on the remote default branch.
- Changed scoped inputs, stale contracts, and mismatched closing approvals fail closed.
- Digest fields remain versioned, domain-separated, and length-framed; binary bytes, topology, and
  executable modes remain exact.

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

The change lifecycle SHALL allow accepted evidence to be reopened when delivery inputs are stale, even if verification.json tip no longer matches the closing approval, by binding reopen to the historical verification attempt that authenticates the closing digest.

Acceptance Criteria
- Reopen succeeds when attempt history contains the acceptance-bound verification the closing approval signed.
- After reopen, re-verify and re-accept (or finalize on workflow v2) restore a matching closing approval.
- Definition approval can be refreshed while accepted when the definition digest is stale.

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
  when available; `change adopt` activates that baseline without rewriting an existing version-1
  policy, refuses before mutation when any existing workflow-v1 record is absent from the proposed
  cutoff, and atomically publishes policy, imports, report, and baseline so interruption or failure
  cannot partially activate workflow v2; every transaction target has a lossless UTF-8,
  platform-separator-safe journal identity and is confined beneath the project with symlink
  components rejected before and during publication. The lifecycle lock is likewise opened through
  a no-follow project capability before any metadata write. All subsequent changes use workflow
  v2, its introduction remains valid after squash/rebase, and a workflow-v1 record remains eligible
  only when that exact ID/version with omitted or explicit version-1 origin existed at the trusted
  cutoff.
- Every bounded workflow-v2 baseline-touching commit and readable parent retains the exact
  introduction bytes, so rewrite→restore history cannot conceal a changed cutoff and deleting a
  committed baseline—including one introduced only on a merged parent—cannot silently reactivate
  workflow v1.
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
- Native review recording and finalization run the same every-parent verification-freshness
  validator as project checking, and every persisted review attempt revalidates that its reviewer
  is distinct from the scope approver bound to that attempt's contract digest.
- Every intervening commit is inspected against every parent; any implementation change, including
  change-then-revert history, stales the review, while the metadata/archive-only finalization commit
  does not rerun or stale it. Native and hosted validators load the same committed descendant,
  parent, output, and timeout limits.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured
  scoped-review check runs.

### REQ-change-audit-project-001

The change module SHALL expose `audit_project` that validates active change workspaces and living SDD policy/spec coherence without rewalking archived terminal evidence by default.

Acceptance Criteria
- `audit_project` does not load or re-authenticate every archived change's terminal evidence.
- `check_project` remains available for full integrity including archives (tests / rare callers).
- CLI project-health surface uses the active-only path.

### REQ-change-check-scoped-002

`check_change` SHALL continue to materialize approved deltas and run verification for one selected change only; project-wide archive integrity is not part of that function.

Acceptance Criteria
- Selecting zero, one, or many open changes behaves as before (nothing / that id / error listing ids).
- Archive terminal evidence is not required for a successful scoped check.


### REQ-change-047

The change lifecycle SHALL prefer completing incomplete selected artifacts over definition approval for draft changes once the interview is complete and artifact completeness validation fails.

Acceptance Criteria

- When selected artifacts contain incomplete HTML TODO comment stubs or are empty, summarize_change sets artifacts_complete to false and next_action does not recommend change approve.
- After selected artifacts are complete, draft next action may recommend definition approval.

### REQ-change-048

The change lifecycle SHALL refuse definition approval when a semantic delta uses ADDED for a requirement ID whose requirement heading already exists in the living module requirements file, and the diagnostic SHALL steer agents to MODIFIED.

Acceptance Criteria

- validate_delta_files and approve_definition fail with cannot add existing block for living requirement IDs under ADDED.
- The error text mentions MODIFIED.
- MODIFIED of an existing living requirement ID validates successfully.

### REQ-change-049

Lifecycle verification SHALL resolve evidence completeness before running any
verification command, SHALL name the artifact and section an author must edit to close an
evidence gap, and SHALL name the failing command when a command fails. Delta application
SHALL converge when an `## ADDED` block is already present with byte-identical content, and
SHALL reject a duplicate `CHG-NNNN` ordinal claimed by two distinct changes from the same
base commit.

Acceptance Criteria

- Incomplete acceptance or requirement evidence fails before any verification command runs.
- The evidence-gap message names the change `testing.md` and its `## Requirement evidence`
  table; the command-failure message names the failing command and its exit code.
- An `## ADDED` block already present with byte-identical content applies as a no-op, so
  re-deriving the canonical tree converges.
- An `## ADDED` block present with different content fails and directs the author to
  `## MODIFIED`.
- Two distinct changes claiming one ordinal from the same base commit are rejected at
  definition approval and by `change audit`; differing or unknown base commits are accepted.

### REQ-change-050

SpecSync SHALL leave a newly initialised project able to complete its own lifecycle, and
SHALL treat an active-change directory that contains no `state.json` as not an active change
in this working tree rather than as corruption.

Acceptance Criteria

- `init` detects a verification command for Cargo, bun, Swift, fledge, Go, Python and npm
  projects, and when none is detected warns at init time naming `.specsync/sdd.json` and an
  example command.
- A change directory with no `state.json` is skipped by active-change discovery, so
  `change new` succeeds on a branch that does not contain an earlier change.
- Every other read error, including an unreadable or malformed `state.json`, still fails closed.
- Verification exposes a lock-free body so a caller already holding the project lock can
  re-run it without deadlocking on the non-reentrant lock.

### REQ-change-051

Git candidate scope guards SHALL admit the tracked files that a directory
candidate expands to, treating a returned path as in scope when it equals a
candidate or is a descendant of one.

Acceptance Criteria

- A `:(top,literal)` pathspec naming a directory expands to every tracked file
  beneath it; those files are in scope because the directory requested them.
- Descendant matching compares at the path separator, so an unrelated sibling
  such as `a/bc` is never admitted by the candidate `a/b`.
- The index, modified, visibility and fsmonitor guards apply identical scope
  semantics; no guard admits a path the others would reject.
- A path sharing no candidate ancestor remains rejected, preserving the guard
  against Git returning genuinely out-of-scope paths.
- Evidence collection succeeds in a repository containing archived changes,
  which is the state of every project past its first archival.

### REQ-change-052

The change module SHALL hold canonical ownership of its own logic, leaving each
command wiring module the sole canonical owner of its own file.

Acceptance Criteria

- `specs/change/change.spec.md` lists `src/change.rs` and does not list
  `src/commands/change.rs`.
- `specs/cmd_change/cmd_change.spec.md` remains the sole claimant of
  `src/commands/change.rs`.
- No source file is claimed by two specs.

### REQ-change-053

Canonical ownership of declared paths SHALL be resolved when the definition is
approved, and a change that has never closed SHALL be able to correct an
acceptance input owner without an audited reopen event.

Acceptance Criteria

- Approving a definition rejects declared paths that no declared module
  canonically owns, naming every offending path in one error.
- Paths that do not yet exist are not rejected at approve, since the owning spec
  may claim them in the same change; they remain enforced at finalize.
- A change with justified `no_spec_change` (empty declared specs) is not
  ownership-rejected at approve, because there is no owner set to resolve
  against; finalize still enforces production ownership. Empty specs without
  that justification fail closed at definition validation and at ownership
  validation.
- A change at verifying that has never closed may correct an acceptance input
  owner under a currently valid definition approval. That substitute is for
  guided-path reachability, not audit-equivalent provenance to an
  Accepted→reopen cycle.
- A change that did close continues to require an audited reopen, unchanged.

### REQ-change-054

Change artifact completeness SHALL treat HTML TODO comments, bare TODO lines, and markdown headings whose title is only TODO (optionally with a trailing description after a colon) as incomplete placeholder content.

Acceptance Criteria

- `change approve` rejects when any selected artifact body is empty or only placeholder TODO content after YAML frontmatter.
- `change status` / next-action guidance list those incomplete artifact paths and do not recommend approve.
- Artifacts with real prose or completed checklist items remain complete even when a section heading is present.
- HTML TODO comments continue to mark an artifact incomplete.

### REQ-change-055

Change sequence allocation SHALL floor on the highest sequence observed locally (active, archive, local ledger) and, when available, the remote default-branch `.specsync/change-sequence.json` high-water. Concurrent multi-clone fleets MAY set `SPECSYNC_SEQUENCE_BASE` to disjoint ranges so agents that cannot see each other do not mint the same numeric CHG prefix.

Acceptance Criteria

- When `origin/HEAD` (or `origin/main` / `origin/master`) contains a schema-v1 sequence ledger, `change new` allocates above that sequence even if the local ledger file is missing or lower.
- `SPECSYNC_SEQUENCE_BASE=N` makes the next allocated sequence at least `N`.
- Simultaneous clones without BASE or a fetched remote high-water may still collide; post-merge sequence validation continues to fail closed on unacknowledged duplicates.

### REQ-change-056

The change domain SHALL expose correction-ledger health to text lifecycle inspection without
returning correction values, ledger bytes, or digest material to a human output path.

Acceptance Criteria

- Malformed, unauthenticated, or otherwise invalid correction history produces a deterministic
  invalid-health result.
- The text-facing diagnostic is generic, names the correction ledger, and directs restoration
  from trusted history.
- The diagnostic contains no correction value, ledger fragment, or digest.
- Valid correction history continues to permit normal text lifecycle inspection.

### REQ-change-057

Existing-change definition mutations SHALL validate correction-ledger integrity inside the same
project-lock transaction that persists their state.

Acceptance Criteria

- `answer_question`, `add_dependency`, and `add_supersedes_obligation` acquire the project lock
  before loading and validating the current correction ledger.
- A ledger corrupted while a mutation waits for the lock causes a deterministic safe failure and
  leaves every lifecycle file other than the external corruption byte-for-byte unchanged.
- The safe diagnostic contains no correction value, ledger fragment, or digest.
- A successful mutation returns the effective definition, correction history, and normal/strict
  machine summaries validated by its transaction, so command rendering does not reread the ledger
  or emit a contradictory result after persistence.
- The documented `answer_question`, `add_dependency`, and `add_supersedes_obligation` wrappers
  remain compiled in production while command-only snapshot variants carry the richer response.
- Valid mutations retain their established state and output behavior.

### REQ-change-058

The lifecycle check SHALL expose exactly one configured-command output behavior, and the
quiet-output variant used solely to keep lifecycle findings out of a machine-consumed
report stream SHALL NOT exist.

Acceptance Criteria

- No lifecycle entry point suppresses configured verification command output; every
  invocation inherits the parent streams.
- The quiet-output check path and its selector type are absent rather than retained
  unused, so no caller can reintroduce the suppressed-output behavior.
- Verification command execution, failure reporting, and recursion refusal are otherwise
  unchanged.


### REQ-change-060

A bootstrap record SHALL exempt a path from lifecycle path coverage only when that exemption cannot
be used to hide product delivery or later policy edits.

Acceptance Criteria

- A recorded path is honored only when it is a protected SDD path, is absent at the delivery
  comparison base, its recorded base commit is an ancestor of `HEAD`, and its content still matches
  the recorded digest.
- A bootstrap record never exempts a path that is not a protected SDD path.
- Editing a bootstrapped file revokes its own exemption and the normal change workflow applies from
  that point on.
- Bootstrap records written in the earlier single-path shape continue to be honored.

### REQ-change-061

The digest recorded for a bootstrapped policy SHALL pin the enforcement surface rather than the
file's bytes.

Acceptance Criteria

- Every field that determines whether the coverage gate applies is covered by the digest.
- Verification commands are excluded, so populating them as initialization instructs does not revoke
  the bootstrap.
- A policy file that cannot be parsed falls back to a digest of its bytes.

### REQ-change-062

Resolution of the delivery comparison base SHALL succeed in a repository containing a single commit.

Acceptance Criteria

- Both a range form and a bare commit reduce to a single commit through its merge base with `HEAD`.
- No resolution path depends on a parent commit existing.

### REQ-change-063

An unfinished spec section SHALL gate on whether a change authored it, not on its content shape.

Acceptance Criteria

- A generated section no active change authored produces no fatal effective-contract finding.
- A section an active change authored and then emptied remains fatal, through both the pending and
  the applied delta paths.
- Unknown authorship fails closed and exempts nothing.
- Ignore configuration is applied through the project's ignore rules rather than re-derived.
- Suppressions are reported as warnings rather than dropped silently.

### REQ-change-064

The uncovered-paths remediation SHALL stay readable regardless of how many paths are
reported.

Acceptance Criteria
- At most a fixed number of paths are named explicitly.
- Any remainder is summarized with a count and a covering-prefix suggestion.

### REQ-change-065

A semantic delta SHALL accept subheadings within an item's body, and SHALL identify its own
items by keyword rather than by heading depth.

Acceptance Criteria
- A subheading met while an item is open is treated as that item's content.
- The spec sections a scaffold generates are accepted verbatim as delta content, without editing the spec first.
- A subheading appearing before any item is opened remains an error, because it cannot be attached to anything.
- That error names both valid item forms so the required shape is discoverable from the message.

### REQ-change-066

The change module's tests SHALL live in their own file while remaining inline for name resolution.

Acceptance Criteria
- Tests are declared with `#[cfg(test)] #[path]` so `use super::*` continues to reach every private item; a sibling module would force visibility changes across hundreds of items and turn a move into an edit.
- Test-only helpers and fault-injection hooks that production code paths reference remain in the production file, because they instrument production code rather than merely living beside it.
- A future split of this module is verified by counting test functions and passing tests before and after, not by reading the diff: a move that loses a test still compiles and still passes.

### REQ-change-067

A refused reopen SHALL leave the archive as finalize wrote it.

Acceptance Criteria
- The dated archive package remains at its original path, with no orphan in the active workspace and the record still archived.
- The refusal states that the archive was restored, so a user whose reopen failed knows the package survived; if the restore itself fails, the message names the path to move back by hand.
- Retrying reproduces the same refusal rather than a different one, because the first attempt consumed nothing.
- A reopen that legitimately succeeds still un-archives, so the restore cannot be satisfied by never moving anything.

### REQ-change-068

Enumerating active changes SHALL return what could be read and what could not, as separate facts, so that no caller can mistake an unreadable workspace for an absent one.

Acceptance Criteria
- The roster reports readable records and unreadable workspaces separately, and each unreadable entry carries the workspace identity and a reason naming the offending path.
- A workspace that cannot be read does not abort enumeration: its healthy siblings are still returned.
- A failure that leaves no partial truth to report — the changes directory itself being unreadable — remains a hard error rather than an empty roster.
- A directory with no state file is still skipped rather than reported unreadable, because a husk left by a branch switch is not an active change here.
- The plain record list used by digest, ledger and successor computations continues to fail closed on any unreadable workspace, since a silently short roster is worse there than a hard error.
- A project with no active changes still yields an empty roster with nothing unreadable.

### REQ-change-069

Declaring an additional affected module SHALL never remove a verification command from what a change receives.

Acceptance Criteria
- The command set selected for a scope is a superset of the set selected for any subset of that scope, so widening declared scope can only add verification.
- A declared module with no component routing entry contributes the project-wide verification commands, because a module nobody routed is not a module that needs no verification.
- A change scoped entirely to routed modules still receives only its component commands, so targeted verification remains available.
- A change declaring no affected module still receives the project-wide verification commands.
- Strict escalation continues to append its own commands without removing any already selected.

### REQ-change-070

A lifecycle commit SHALL NOT record a change sequence ledger below the highest sequence already committed, and SHALL disclose any raise it performs.

Acceptance Criteria
- Before staging, a working-tree ledger lower than the committed high-water mark is raised to it, so no lifecycle commit can lower the recorded mark.
- A working-tree ledger at or above the committed mark is left exactly as the author wrote it, because a newer claim is the ordinary result of allocating a change and must not be overwritten.
- The raise is reported on a stream that survives quiet output and does not contaminate a machine-readable payload, naming both the previous and the adopted value.
- Acknowledged collisions recorded on either side are preserved across the raise rather than replaced by one side's copy.
- Every staging site in the lifecycle applies the rule, so a commit path added later cannot reintroduce the regression by bypassing one of them.

### REQ-change-071

Validating change sequences SHALL refuse a ledger below the high-water mark the default branch has already published, whether or not the higher-numbered workspaces are present on disk.

Acceptance Criteria
- A local ledger below the default branch's recorded sequence is refused even when no higher-numbered workspace directory exists locally, which is the ordinary state of a fresh clone or an unfetched branch.
- The refusal names both the claimed and the published sequence, and states the command that restores the ledger.
- A local ledger at or above the published mark is accepted, so ordinary allocation is unaffected.
- The published mark is read from the same source the allocation floor already consults, rather than from a second implementation of the same lookup.

### REQ-change-072

The change sequence ledger gate SHALL judge a ledger against the highest mark the current branch has itself recorded, and SHALL NOT refuse a branch for trailing the default branch.

Acceptance Criteria
- A branch whose ledger is older than the default branch's, but consistent with its own history, is accepted, and allocation on it continues to floor against the remote mark so it cannot remint an ordinal the default branch already used.
- A ledger below the highest mark the branch itself recorded is refused, including when the branch raised the ledger and then rewrote it downwards to a value still above the point at which it diverged.
- The gate consults no remote, so a repository without an origin is judged by the same rule rather than having the gate silently disabled.
- The refusal names the mark that was lost and a recovery command that applies to the branch's own history.

### REQ-change-073

Scoped review evidence SHALL be permitted to move between a change's active workspace and its archive in either direction, and SHALL be refused anywhere else.

Acceptance Criteria
- A change that was finalized, reopened, re-checked and re-reviewed can be finalized again, leaving exactly one archive package and no active workspace.
- The move performed by reopen is accepted on the same terms as the move performed by finalize, since both relocate the same evidence between the only two locations a change occupies.
- Relocation to any other path is still refused, so the check continues to detect evidence moved outside the lifecycle.

### REQ-change-074

An archived change package SHALL NOT retain a directory that holds no regular file at any depth, and enumeration SHALL treat such a directory under the archive as an absent change rather than a damaged one.

Acceptance Criteria
- Shipping a change whose `deltas/` is empty leaves no untrackable directory in the dated archive package, so a checkout of a commit that predates the package removes the package entirely instead of stranding a husk that `git status` reports as clean.
- A directory under the archive that holds no regular file at any depth is skipped by `change new`, `change audit`, `change adopt` and `check`, since git cannot represent it and its presence records the absence of a change rather than a corrupt one.
- A directory under the archive that holds at least one regular file but no `state.json` is still refused, so the allowance cannot be satisfied by ignoring corruption.
- Directories in an archived package that do hold files are preserved, so pruning removes only what git could never have committed.

### REQ-change-075

A semantic delta parser SHALL distinguish a file that is empty from a file that has content
but no recognized operation heading, SHALL name the allowed operation headings in that
second case, SHALL accept item headings case-insensitively, and SHALL apply the same empty
versus unrecognized wording on the historical delta path.

Acceptance Criteria
- A file whose only content is prose or unrecognized text reports that it contains no recognized operation headings and names `## Added`, `## Modified`, and `## Removed`, instead of reporting that the file is empty.
- A file that is empty or whitespace-only still reports that it is empty.
- `### requirement` and `### spec section` parse as `### REQUIREMENT` and `### SPEC SECTION`.
- An unrecognized `##` heading is still refused and names the allowed operation values.
- An unrecognized `###` heading before any item is still refused and still names both valid item forms.
- A `###` line that is not an item keyword, met while an item is open, remains that item's content.
- A valid uppercase delta still parses to the same items.
- The historical delta walk uses the same empty-versus-unrecognized distinction and does not report a populated unrecognized file as empty.

### REQ-change-076

The effective checkout overrides SHALL be read from Git in a single configuration query rather than one query per key, and SHALL derive the same values that separate per-key queries produced.

Acceptance Criteria
- The four `core` keys that determine the checkout overrides are obtained in one `git config` invocation instead of four.
- A key set more than once resolves to its last value, matching what a single-key query returns.
- A key present with no value normalizes exactly as the empty value does.
- A key written under a mixed-case section, or with surrounding whitespace, normalizes identically.
- No matching key is treated as unset rather than as a failure.
- A malformed configuration file still fails loudly and is never read as unset, so a broken repository cannot be mistaken for a default one.
- No value is cached: every read still queries Git, so a configuration change between reads is still observed.

### REQ-change-077

A bounded Git read SHALL be bounded for the response it can actually receive, not for the response the call it replaced received.

Acceptance Criteria
- Reading the effective checkout overrides succeeds when the four core keys are set in more than one configuration scope, the ordinary layout of a global file plus a repository-local override.
- The values derived equal what a separate per-key query returns for each key, compared against that query rather than against an assumption about which scope takes precedence.
- A genuinely unbounded response is still refused, so the deterministic-output guard is retained rather than removed.

### REQ-change-078

The rule governing where committed scoped review evidence may move SHALL be pinned by tests that fail when either the permitted directions or the refusal itself is removed.

Acceptance Criteria
- Removing the archive-to-active direction fails a test, so the defect where a reopened change could never be closed again cannot return silently.
- Deleting the guard entirely fails a test, and fails a different one than the direction removal does, so a fix and the refusal it lives inside are pinned independently.
- A move to any location other than a change's active workspace and its archive is refused, asserted in both directions.
- Deleting committed review evidence is refused.

### REQ-change-079

Evidence persisted to disk SHALL be readable by a reader that does not recognise every field it carries, and a change recording an unrecognised workflow version SHALL be reported as written by a newer SpecSync rather than as an invalid change state.

Acceptance Criteria
- Evidence carrying a field this reader does not know is parsed rather than rejected, so an evidence shape can be extended within a major version without breaking installations already deployed.
- A change whose workflow version this reader does not support names both the cause, that a newer SpecSync wrote it, and the remedy, that the reader should be upgraded, and does not describe the record as invalid.
- The spec and source hash cache continues to reject a shape it cannot understand, because it is untracked and rebuilt from scratch on any parse failure, so discarding one costs nothing and cannot lose evidence. A file that is committed and shared is evidence regardless of what it holds, and is tolerated.
- A file read through a canonical-bytes round trip gains nothing from tolerance, because the unknown field is dropped on parse and the re-serialized bytes then differ from the bytes on disk. This limit is deliberate for the files that anchor history, and is pinned by a test rather than left to be discovered.
- Every digest is unchanged, because tolerance at read time was never part of any preimage.

### REQ-change-088

A later generation of a change's terminal evidence SHALL be trusted only when it extends the generation already committed, and closing evidence that history has not seen SHALL be presentable only by the process writing that package out of the active workspace.

Acceptance Criteria
- A generation is accepted as later only when it contains, unrewritten, every approval and reopen event the committed generation already holds, because a count of reopen events is written by whoever writes the file and so distinguishes nothing.
- Rewriting any earlier entry while appending a new one is refused, so a forged reopen cannot launder a tampered approval by appearing to advance the ledger.
- A change that has been genuinely reopened can be closed again, because the evidence for a new generation necessarily does not yet exist in history at the moment it is being written.
- Evidence that history has not seen is accepted only from the process writing the package out of the active workspace, so a working tree cannot speak for a package that history already holds.

### REQ-change-086

A change identity SHALL be minted from its description alone, and identity uniqueness SHALL be enforced directly rather than as a side effect of allocating a number.

Acceptance Criteria
- A newly created change is identified by its description alone, with no allocated number, so two people working from the same base need not coordinate to avoid claiming the same identity.
- A description that would produce an identity already in use is refused by naming the existing change, its location and its state, rather than by exhausting an allocation retry.
- Two workspaces claiming one identity are refused directly, because an allocated number is no longer providing that guarantee as a side effect and an identity that names two packages is ambiguous.
- An identity that carries no number takes part in no number-based accounting, while an identity that carries a malformed number is still refused, because tolerating an absent number must not become tolerating a corrupt one.
- Identities already allocated keep working unchanged, including the historical ones that share a number by prior acknowledgement.

### REQ-change-085

Terminal evidence SHALL be trusted only against the commit where that evidence entered history, and no later commit that re-introduces the same package SHALL be usable as its anchor.

Acceptance Criteria
- A commit that re-introduces a package cannot authenticate the evidence it carries, because the check compares committed bytes against the working tree and would otherwise be satisfied by any commit of the current state, whatever that state has become.
- The rule applies wherever a package can be re-introduced, not only where it is archived: a package moved back to an active workspace and archived again is re-introduced at a path SpecSync itself writes, and is covered.
- A package is identified for this purpose by the identity recorded inside its evidence, not by the name of the directory holding it, because the directory name is not part of a package's identity anywhere else.
- Relocating a package without altering it continues to authenticate, so history can be reorganised and the earlier evidence still stands.
- Every archive that authenticates before this rule is applied continues to authenticate after it.

### REQ-change-084

A change identity SHALL be accepted or refused on the properties that make a string a safe path component, and SHALL NOT be required to begin with any particular prefix.

Acceptance Criteria
- An identity carrying no ordinal is accepted, because a prefix is text any caller can type and is therefore evidence neither that an identity is well-formed nor that SpecSync minted it.
- An identity is refused when it is empty, is not a single path component, contains a path separator or a control character, exceeds the longest name a path component may hold, or is a name a supported platform reserves.
- Every identity shape SpecSync has previously minted remains acceptable, so relaxing what is required does not orphan history.

### REQ-change-083

A minted change slug SHALL be a legal directory component on every platform SpecSync ships a binary for, and SHALL remain readable when the description is too long to keep.

Acceptance Criteria
- The length limit bounds the bytes of the name that reaches the filesystem rather than the characters of the description it came from, and is sized so the deepest path a change produces stays within the shortest maximum path length of any supported platform.
- A name that must be shortened is cut at a word boundary rather than mid-word whenever a boundary is near enough for the result to stay legible, because the description is stored in full elsewhere and the directory name exists to be read.
- A description that would reduce to a reserved directory name does not become one, including the name substituted when a description reduces to nothing.
- A description that needs none of this produces exactly the name it produced before.

### REQ-change-082

Succession SHALL be ordered by when a change was created rather than by how it is named, and every ordering applied to a change's succession edges SHALL agree with the ordering that is signed.

Acceptance Criteria
- A superseded change that was created after its successor is refused whatever the two are called, because succession is a claim about what happened first and a name is not evidence of that.
- Succession ordering does not read a number out of an identifier, so an identifier that carries no number cannot silently reduce the relation to alphabetical order.
- Every sort applied to a change's succession edges produces the same order as the sort whose result is signed, so a canonical form cannot be rejected by the gate that validates it.
- Changes created in the same second remain strictly ordered, because the surrounding gates enforce strict sorts and a tie would make a valid record unrepresentable.

### REQ-change-081

A gate SHALL determine a change's identity from its persisted state rather than from the shape of a directory or file name, and a gate that cannot determine identity SHALL withhold the permission it grants rather than granting it.

Acceptance Criteria
- An archived package that has lost its lifecycle state is refused as damaged whatever it is named, because a naming convention is not evidence that a package is real and skipping a damaged package hides corruption.
- A genuine pre-lifecycle record, holding deltas and nothing else, continues to be skipped, so refusing damage is not achieved by refusing everything.
- Continuous integration determines which changes require an independent review by reading persisted state, so no identity shape can reduce the set of changes needing review to zero and let a pull request merge unreviewed while reporting success.
- A gate that cannot read identity withholds what it grants: an archive fast lane is not taken when the archived state is unreadable, so the full verification runs instead.

### REQ-change-080

A persisted policy SHALL load even when it omits a field this SpecSync knows, and each omitted field SHALL take a value that enforces rather than relaxes.

Acceptance Criteria
- A policy file written before a field existed still loads, so adding a field within a major version does not make every policy written before it unreadable by the SpecSync that added it.
- An absent enablement flag reads as enabled and an absent change requirement reads as required, so a truncated or partial policy cannot silently disable enforcement.

