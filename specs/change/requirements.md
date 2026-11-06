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

### REQ-change-016

The lifecycle SHALL preserve accepted closing evidence across repository-integrated commits without
accepting unintegrated or altered evidence, while verifying evidence SHALL be judged on content
alone.

Acceptance Criteria

- Verification currency does not depend on commit ancestry, on inspecting intervening commits, or
  on restricting which paths may change after verification. Provenance of that kind is recorded by
  `attest`, keyed to commit SHAs, and is outside this tool.
- `verification.commit` is never a gate on verification currency or ship readiness; a squash merge
  that discards the recorded commit does not invalidate the evidence or block delivery. Archival
  authentication of accepted evidence is a separate question — whether the acceptance is anchored
  in history a reader can reach — and MAY consult commit ancestry there, as one basis among the
  integrated accepted workspace and the acceptance recorded on the remote default branch. Ancestry
  MUST NOT be the only basis on which anchoring can be established.
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
- Reopen requires an explicit non-empty human actor and reason and rejects accepted evidence that is exact-or-successor-covered, still anchored in current history, and historically reconstructible when it is manifest-less legacy acceptance.
- Reopen moves stale accepted evidence to verifying so strict checks remain red until a fresh verification run succeeds.
- Prior definition approval, verification, closing approval, manifests, successor evidence, and accepted snapshot remain inspectable in append-only audit history.
- Reacceptance requires a new closing approval and does not reapply canonical deltas already accepted.
- Reacceptance rejects a definition digest that differs from the latest pre-reopen verification contract and directs further spec work to a new change workspace.
- A verifying already-applied change without audited reopen history fails closed.

### REQ-change-018

Audited reopening SHALL recognize only provable canonical acceptance and deterministic semantic succession recorded in trusted Git history.

Acceptance Criteria
- Definition digest, passed evidence, closing approval, a supported staleness cause, actor, and reason remain mandatory.
- An unreachable verification commit or failed manifest-less legacy acceptance reconstruction is an admissible staleness axis for reopen, recorded as an explicit cause in the reopen ledger. Neither substitutes for fresh verification or closing approval; existing definition, verification, and closing authentication checks remain fatal.
- ID order, timestamps, lexicographic ordering, and independent path/spec scope overlap are never succession evidence.
- Repeated trusted commits yielding identical canonical reconstructed evidence are deduplicated; distinct reconstructions fail as ambiguous.
- A descendant feature branch preserves squash-accepted evidence only when the remote default branch records the same accepted state, definition, delivery inputs, and closing approval.
- Arbitrary off-history evidence remains rejected.

### REQ-change-019

Verification SHALL recognize a non-removed requirement or spec-section delta item as semantic acceptance evidence when observable acceptance criteria are present.

Acceptance Criteria

- A section-only modified delta can pass with an empty requirement-ID list.
- Requirement evidence mapping remains mandatory for every collected requirement ID.
- Missing semantic acceptance evidence and missing requirement evidence produce distinct diagnostics, and neither is reported as a configured-command failure, because verification executes no configured command (REQ-change-023).

### REQ-change-020

Audited reacceptance SHALL preserve compatible legacy definition evidence while enforcing immutable reopened definitions, fresh evidence, explicit semantic succession, and validation of every current canonical contract it reapproves.

Acceptance Criteria

- A prior verification digest using the transitional explicit-false lifecycle encoding remains compatible with the stable omitted-false encoding during reopened reacceptance.
- An accepted no-spec change cannot satisfy successor governance even when its paths and specs overlap.
- A supported pre-approval supersede transition records a durable definition-bound predecessor edge with explicit path/module/predecessor-digest obligations.
- Closing evidence binds each adopted obligation only when the same successor has the module's semantic delta and an exact old/new transition from its trusted definition-signed base tree to its descendant unique accepted-transition tree — or, while that transition is not yet in history, to the working tree its closing evidence was signed against; the acceptance commit's immediate parent is not the before tree.
- Every owner of a changed input requires its own same-successor path/module obligation; owner intersection and cross-record path/spec unions fail closed.
- A changed input whose signed owners are all reserved exact labels requires one obligation from a successor whose module owns the path under the current configuration; a `supersede` declaration for such an entry is admitted only for a module that owns the path now, and refused otherwise with the frozen label and the `[modules."<name>"] owns` remedy named, while a module that is not a signed owner of an entry a module signed is still refused.
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
- Nothing claims a next ID, so the duplicates this gate finds are historical ordinals brought together by a merge rather than two branches minting the same number.
- Existing accepted collisions can be baselined exactly without rewriting accepted state or evidence.

### REQ-change-023

Verification SHALL compare THIS CHANGE's specs to code in-process, SHALL NOT spawn project
test or build commands, and SHALL preserve retryable attempt history without weakening
unrelated gates.

Acceptance Criteria

- Scope is the union of the modules in `affected_specs` and every spec whose `files:` mapping
  falls inside a declared `affected_paths` scope. Drift outside that scope does not fail this
  change; project-wide validation is `specsync check`.
- A declared module that resolves to no spec file on disk FAILS verification and is named in the
  error, and the attempt is recorded like any other spec↔code failure so the retry after writing
  the spec stays append-only. It is never dropped from scope, even when other specs are in path
  scope and would otherwise make the pass look real.
- An empty scope is a PASS only for a change that declared no module and whose declared paths map
  no spec — a change that claimed no contract.
- Evidence is the scoped command the verdict was reached under, `specsync check --spec <name> …`,
  each name being what `filter_specs` matches (the file stem with `.spec` removed) rather than a
  frontmatter `module:` that would select nothing, with `--strict` only when requested. It
  reproduces the verdict when every named spec resolves or when none does; a mixed scope fails
  the check but can rerun green, because an unmatched filter is demoted to a warning once any
  filter matches.
- When the effective-contract gate refuses a module the change declares, the failed attempt keeps
  that scoped entry and adds `specsync change check <id> (effective contract: <module>, …)` with
  `success: false`, the command that reproduces the gate's refusal, rather than claiming a
  `--strict` run that did not happen.
- `change check` does not execute `.specsync/sdd.json` `verification_commands`.
- Direct re-entry into SpecSync through `SPECSYNC_VERIFICATION_CONTEXT` still fails once.
- Failed spec↔code attempts remain inspectable and a corrected retry can record passed latest evidence.
- Other failed or stale changes continue failing closed.

### REQ-change-024

Strict lifecycle checking SHALL permit only explicit closing-valid terminal semantic successors to govern changed inputs of an accepted predecessor without hiding unrelated stale evidence.

Acceptance Criteria

- Draft, approved, implementing, verifying, failed, stale, tampered, no-spec, semantically empty, and partial successors never suppress predecessor errors.
- Accepted or authenticated archived successors selected as candidates require valid definition, verification, closing approval, history integration, and recursive exact-or-successor-covered current inputs; standalone archives require historical integrity without equality to today's inputs.
- Every changed input expands to one obligation per signed canonical owner and every obligation matches one exact predecessor/path/module/old-digest/new-digest tuple from the same successor.
- Multiple terminal successors may cover disjoint obligations, while cycles fail closed and completed validity results are memoized.
- The archive preflight forwards its closing token into the successor walk, so the package being closed authenticates as a successor of the changes it supersedes exactly as its own historical-integrity preflight authenticates it; every reading path passes no token and is judged by history alone.
- A successor whose acceptance transition is not in history — the package being closed, or an archive whose commit has not been made — is admitted only through its working-tree closing evidence, and its succession tuple is then checked against the working tree that evidence signed, with base ancestry checked against HEAD; once the archive commit exists, history is again the sole anchor.
- A changed input whose signed owners are all reserved exact labels names no module of its own: its claimants are the modules under which later accepted or archived successors declared it, each claimant is judged by every check a signed owner's successor passes, one authenticated claimant covers the input, and otherwise every refused claimant is named with its reason and the frozen label as the input's owner.

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
- A change that edits the ledger covers it as a delivery input; no change generates a claim to cover, because nothing allocates a sequence.
- An acknowledgement matches the exact currently located ID set and remains valid only when every member is accepted or archived.
- Removed IDs, added IDs, single surviving records, and draft, approved, implementing, or verifying collision members fail closed.

### REQ-change-027

A SpecSync lifecycle command SHALL refuse to run under an inherited verification context, at every lifecycle command surface.

Acceptance Criteria

- `check`, `change`, and `lifecycle` commands that inherit `SPECSYNC_VERIFICATION_CONTEXT` fail once, before performing validation or mutation.
- The diagnostic names the parent command recorded in `SPECSYNC_VERIFICATION_CONTEXT`.
- SpecSync sets that marker on no child of its own: verification executes no configured command (REQ-change-023), so the refusal applies only to a context a caller outside SpecSync supplies.

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

Lifecycle enforcement SHALL preserve explicit user scope, precise canonical companion coverage, registry authority, and policy opt-out boundaries while retaining fail-closed SpecSync recursion protection.

Acceptance Criteria

- Generated sequence bookkeeping does not satisfy or suppress the interview question for source, test, documentation, or configuration scope.
- Registry-resolved modules cover only their exact canonical spec and the standard `requirements.md`, `tasks.md`, `context.md`, `testing.md`, and `design.md` companions; unrelated siblings and the containing directory are not implicitly covered.
- Both registry files remain protected lifecycle inputs because they control canonical writes.
- An explicitly disabled SDD policy returns without sequence-ledger validation.
- `change check` does not execute configured commands (REQ-change-023), so a policy listing `cargo run --bin specsync -- check` cannot recurse through verification.
- Direct SpecSync lifecycle commands remain rejected when invoked under `SPECSYNC_VERIFICATION_CONTEXT`.
- The retained Cargo identity classifier is consulted by no lifecycle command, because no configured command is executed. As a helper it still classifies `cargo run -- check` as non-SpecSync unless Cargo is actually selecting SpecSync by manifest identity, `default-run`, binary, or package.
- In that classifier, both `--manifest-path <path>` and `--manifest-path=<path>` participate in Cargo identity detection, unsafe explicit manifest paths fail closed, and argument parsing tokenizes ordinary whitespace, quoted values, and trailing comments without shell execution.

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
- Commit reachability is a second staleness axis alongside delivery-input drift. Reopen admits exactly the axes for which no restore exists: content that drifted can be put back, but an orphaned commit can only be superseded.

### REQ-change-035

Acknowledged immutable sequence collisions SHALL remain valid while an accepted member completes an
audited delivery-only reopen, provided its already-applied definition, prior passing verification,
superseded closing approval, and exact accepted-to-verifying transition remain valid.

Acceptance Criteria

- A structurally valid audited reopen keeps the collision acknowledgement usable during verification.
- A reopen caused by an unreachable verification commit records identical stale and current delivery-input digests and still preserves immutable sequence history; the recorded cause, not digest inequality, is what proves the evidence was stale.
- Missing, tampered, definition-stale, unapplied, or non-verifying reopen evidence remains mutable.
- The reopened member still requires fresh verification and a new closing approval before acceptance.
- Collision IDs are never renumbered, deleted, or silently rewritten.

### REQ-change-036

Stale accepted-change verification diagnostics SHALL name the offending delivery input and state the concrete remediation, without changing the underlying freshness model.

Acceptance Criteria

- A changed covered input that no accepted or archived successor claims reports the input path, its owner module, and the `specsync change reopen <id>` remediation.
- A changed covered input that one or more successors claimed and were refused for reports the input path, its owner module, and each refused successor with the reason it was refused — its evidence did not authenticate, its manifest could not be resolved, its evidence carries no tuple for the input, its manifest does not carry the tuple's successor entry, its delta has no semantic item, its tuple does not hold or could not be evaluated, or its own delivery-input evidence is stale — sorted by successor ID; a refusal is never reported as the absence of a successor.
- When the stale change is workflow v1 and a refused successor is workflow v2, the diagnostic directs the operator to finish that successor (`specsync change status <successor>` names its next step) and does not offer `specsync change reopen` of the legacy change, whose replayed canonical delta would overwrite the successor's materialization; every other combination directs the operator to verify and accept a covering successor or reopen the accepted change.
- A covered input that disappeared from the current inventory reports the missing path and the restore-or-reopen remediation; a changed exact-only input that no successor claims reports the path and the audited-reopen remediation, and names the supersede alternative under `[modules."<name>"] owns` wherever the configuration can grant the path; a changed exact-only input whose claimants were all refused is reported like a signed owner's input, with the frozen label as its owner; missing delivery-input evidence keeps its established phrase and gains the reopen remediation.
- Every stale reason remains deterministic: sorted successor IDs, no timestamps, and no environment-dependent content.
- The `accepted change verification is stale for current delivery inputs` check prefix, the terminal-evidence validity values, and every freshness predicate remain unchanged.

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
- That history is read with repository-root pathspecs (`:(top,glob)` to discover dated archive
  paths, `:(top,literal)` to walk them). A project rooted in a subdirectory of its Git repository
  still sees every dated path and still refuses a workflow-version downgrade that a later commit
  restored.
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

Agent-authored changes SHALL receive one scoped review of implementation evidence before finalization. The reviewer claim MAY be the same actor as the definition approver. SpecSync SHALL NOT invent a second-person requirement beyond GitHub's merge rules.

Acceptance Criteria

- Review input contains only the change package, implementation diff, canonical semantic delta, and targeted evidence.
- The result binds the implementation parent commit, those input digests, an explicit pass/block verdict, a stable reviewer claim, and the stored provenance-provider declaration. Local validation checks that declaration and content freshness; it does not authenticate reviewer identity or fetch a live GitHub check verdict.
- Authenticated identity or provenance enforcement requires separately configured hosted checks and policy verification. A reviewer label or stored provider declaration alone is not that enforcement.
- The reviewer claim MAY equal the definition approver (comparison is still case-insensitive for identity, not for refusal).
- Every review attempt is append-only; `review.json` is only the latest projection and cannot erase a prior blocking result.
- Native review recording and finalization run the same every-parent verification-freshness validator as project checking.
- Every intervening commit is inspected against every parent; any implementation change, including change-then-revert history, stales the review, while the metadata/archive-only finalization commit does not rerun or stale it.
- Scoped-review currency remains the three-valued answer `current` / `stale` / `unavailable` as already specified.
- Content is decided before history: a review whose recorded contract, execution, or workspace digest no longer matches the tree is `stale` for that reason.
- Finalization fails when a required scoped review is missing or blocking.
- Status states when review is needed and directs the user to open or update the PR so the configured scoped-review check runs.
- SpecSync does not refuse a ship solely because the reviewer is the definition approver.

### REQ-change-audit-project-001

The change module SHALL expose `audit_project` that validates active change workspaces and living SDD policy/spec coherence without rewalking archived terminal evidence by default.

Acceptance Criteria

- `audit_project` does not load or re-authenticate every archived change's terminal evidence.
- An active terminal record is judged against every accepted or archived change as a successor candidate: archived records are loaded only when such a record exists, only as candidates and never for evaluation, and only one that declares a matching obligation is authenticated. A legacy accepted change superseded by a finalized successor is successor-covered on the audit path exactly as on the full walk, and a refused archived successor is named with its reason.
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

Lifecycle verification SHALL resolve evidence completeness before comparing specs
to code, SHALL name the artifact and section an author must edit to close an
evidence gap, and SHALL NOT spawn the project's test or build commands. Spec↔code
sync is the verifier. Delta application
SHALL converge when an `## ADDED` block is already present with byte-identical content, and
SHALL reject a duplicate `CHG-NNNN` ordinal claimed by two distinct changes from the same
base commit.

Acceptance Criteria

- Incomplete acceptance or requirement evidence fails before spec↔code sync runs.
- The evidence-gap message names the change `testing.md` and its `## Requirement evidence`
  table. Drift failure names the spec finding, not a test-suite exit code.
- Configured `verification_commands` in `.specsync/sdd.json` are not executed.
- An `## ADDED` block already present with byte-identical content applies as a no-op, so
  re-deriving the canonical tree converges.
- An `## ADDED` block present with different content fails and directs the author to
  `## MODIFIED`.
- Two distinct changes claiming one ordinal from the same base commit are rejected at
  definition approval and by `change audit`; differing or unknown base commits are accepted.

### REQ-change-050

SpecSync SHALL leave a newly initialised project able to complete its own lifecycle, and
SHALL treat an active-change directory that contains no `state.json` and no regular file at
any depth as not an active change in this working tree rather than as corruption. A directory
that holds a file or symlink but no `state.json` is damaged and still open.

Acceptance Criteria

- Fresh `init` writes SDD off with an empty `verification_commands` list; `specsync check` is
  the next step and does not need a project test command.
- A change directory with no regular file at any depth and no `state.json` is a husk and is
  skipped by active-change discovery, so `change new` succeeds on a branch that does not
  contain an earlier change.
- A change directory that holds a file or symlink but no `state.json` is reported unreadable,
  with a reason that says the state file is missing, and does not abort enumeration of its
  siblings. Sequence enumeration refuses that directory rather than skipping it.
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

The lifecycle check SHALL NOT spawn configured verification commands, and the
quiet-output variant used solely to keep lifecycle findings out of a machine-consumed
report stream SHALL NOT exist.

Acceptance Criteria

- `change check` and `change audit` do not execute `.specsync/sdd.json` `verification_commands`.
- The quiet-output check path and its selector type are absent rather than retained
  unused, so no caller can reintroduce suppressed-output command execution.
- Failed spec↔code evidence remains inspectable in `verification.json`.

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
- Verification commands are excluded, so editing the list does not revoke the bootstrap. Nothing
  executes the list (REQ-change-023), and fresh initialization writes it empty.
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
- A directory with no state file and no regular file at any depth is still skipped rather than reported unreadable, because a husk left by a branch switch is not an active change here. A directory that holds a file or symlink but no state file is reported unreadable, with a reason that says the state file is missing, and does not abort the roster.
- The plain record list used by digest, ledger and successor computations continues to fail closed on any unreadable workspace, since a silently short roster is worse there than a hard error.
- A project with no active changes still yields an empty roster with nothing unreadable.

### REQ-change-069

Declaring an additional affected module SHALL never remove a verification command from the set the retained routing helper selects for a change. Routing selects commands and executes none of them (REQ-change-023).

Acceptance Criteria
- The command set selected for a scope is a superset of the set selected for any subset of that scope, so widening declared scope can only add commands.
- A declared module with no component routing entry contributes the project-wide verification commands, because a module nobody routed is not a module that needs no verification.
- A change scoped entirely to routed modules is still routed only its component commands, so targeted routing remains available.
- A change declaring no affected module is still routed the project-wide verification commands.
- Strict escalation continues to append its own commands without removing any already selected.
- No lifecycle command executes a routed command; `change check` records only its in-process spec↔code evidence.

### REQ-change-070

A lifecycle commit SHALL NOT record a change sequence ledger below the highest sequence already committed, and SHALL disclose any raise it performs.

Acceptance Criteria
- Before staging, a working-tree ledger lower than the committed high-water mark is raised to it, so no lifecycle commit can lower the recorded mark.
- A working-tree ledger at or above the committed mark is left exactly as the author wrote it, because raising is the only direction this rule may move a ledger and a mark that is already higher is not a regression to repair.
- The raise is reported on a stream that survives quiet output and does not contaminate a machine-readable payload, naming both the previous and the adopted value.
- Acknowledged collisions recorded on either side are preserved across the raise rather than replaced by one side's copy.
- Every staging site in the lifecycle applies the rule, so a commit path added later cannot reintroduce the regression by bypassing one of them.

### REQ-change-072

The change sequence ledger gate SHALL judge a ledger against the highest mark the current branch has itself recorded, and SHALL NOT refuse a branch for trailing the default branch.

Acceptance Criteria
- A branch whose ledger is older than the default branch's, but consistent with its own history, is accepted. Nothing mints an ordinal any more, so trailing the default branch cannot lead to reminting one.
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
- A directory under the archive that holds at least one file or symlink but no `state.json` is still refused, so the allowance cannot be satisfied by ignoring corruption.
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
- An identity is refused when it is empty, is not a single path component, contains a path separator or a control character, exceeds the longest name a path component may hold, or is a name a host platform reserves, Windows device names included.
- Every identity shape SpecSync has previously minted remains acceptable, so relaxing what is required does not orphan history.

### REQ-change-083

A minted change slug SHALL be a legal directory component on every platform a SpecSync repository may be checked out on, Windows included, whether or not SpecSync publishes a binary for that platform, and SHALL remain readable when the description is too long to keep.

Acceptance Criteria
- The platforms this rule covers are the platforms a repository may be checked out on, not the platforms SpecSync publishes binaries for. Narrowing the published set does not narrow this rule, because the directory a slug becomes is created in someone else's clone.
- The length limit bounds the bytes of the name that reaches the filesystem rather than the characters of the description it came from, and is sized so the deepest path a change produces stays within the shortest maximum path length of any host platform, which is Windows `MAX_PATH` at 260.
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

### REQ-change-089

A semantic delta body SHALL be bound to the definition approval that signed it, and an approval that recorded no such binding SHALL read as unknown rather than as a violation.

Acceptance Criteria
- Approval records a digest over every semantic delta file the change owns, keyed by module, so the wording that rewrites a canonical spec is part of what a human signed.
- The body is hashed with `\r\n` folded to `\n`, because delta application already treats line-ending style as not part of the content, so a checkout that rewrote a delta's line endings without editing a character SHALL NOT invalidate the approval that signed it.
- No other difference is folded. Trailing whitespace, blank lines, and a lone carriage return keep changing the digest, so the binding's equality stays strictly narrower than the applier's, which also trims surrounding blank lines and horizontal whitespace.
- Folding line endings moves no digest recorded before it, because a body containing no `\r\n` hashes exactly as it did.
- Materialization and acceptance verify that binding before any delta is applied, and refuse by naming every module whose body changed after approval together with the remedy.
- The refusal is evaluated before the already-materialized short-circuit, so a body that drifts after the first application is still caught while it remains this change's evidence.
- An approval carrying no delta binding proceeds unchanged, because every change approved before the binding existed made no claim about wording and absent evidence is not a violation.
- The binding is omitted from persisted JSON when it is absent, so no existing approval digest moves and ledgers written by earlier binaries stay readable and byte-identical.

### REQ-change-090

A delta binding once recorded in an approval ledger SHALL NOT be withdrawn by a later definition
approval, and absence of a binding SHALL keep meaning that the approval predates it.

Acceptance Criteria
- A portable SpecSync 5.0.1 definition approval records the per-module delta digest it approves on
  both members of its marked pair, because it is a definition approval and a definition approval
  records the wording it signed.
- Recording that binding leaves the portable projection untouched: the pair's current and legacy
  digests, its metadata and its resolution are exactly what they were, because the binding is an
  input to none of them and persisted approval evidence tolerates fields an older reader does not
  know.
- An effective definition approval that records no delta wording while another definition approval
  in the same ledger records it is refused at materialization and acceptance, and the refusal names
  the re-approval that restores a truthful ledger.
- A ledger in which no definition approval ever recorded delta wording still materializes, however
  many such approvals it holds, because it withdrew nothing and every archived change is in that
  position.
- A portable SpecSync 5.0.1 definition approval remains available on a workflow-v1 change with no
  prior definition approval, because refusing it there would remove the only route an adopter has
  to a 5.0.1-verifiable approval.

### REQ-change-091

Lifecycle verification SHALL NOT spawn project test or build commands, so it SHALL NOT wait on a
Cargo build-directory lock and SHALL NOT create a verification child process.

Acceptance Criteria
- `change check` records in-process spec↔code evidence naming its scope, as
  `specsync check --spec <name> …`.
- A configured `verification_commands` sentinel is not executed.
- A held Cargo `.cargo-lock` is not named on stderr during `change check`.
- A configured reporter script is not started.

### REQ-change-092

Canonical materialization SHALL be decided from the canonical artefacts rather than from the
`canonical_applied` flag alone, so that every output materialization produces is present for the
delta bodies the change has currently approved.

Acceptance Criteria
- A semantic delta corrected after review and re-approved is materialized into the canonical spec
  by the next `change check`, and the superseded wording does not survive beside the correction.
- A module whose canonical spec carries the change's contract text but carries neither the
  `version:` bump nor a Change Log row naming the change receives both on the next `change check`.
  Neither is derivable from a delta digest, so re-applying the delta alone would not close this.
- A re-approval whose delta body is byte-identical writes nothing at all: the canonical spec and
  requirements stay byte for byte as they were, the version stands at one bump, and the Change Log
  carries exactly one row for the change. Re-materializing unconditionally is refused as a fix,
  because it would rewrite every canonical spec on every check.
- Re-materialization does not refuse the work its own earlier run performed: a `## REMOVED` item
  whose block is already absent reads as applied. That reading is available ONLY to a change that
  has already materialized once; on a first materialization every application refusal still fires
  unchanged, including removing a block that was never present.
- The refusal for a semantic delta that changed after its approval names `specsync change check`
  after `specsync change approve`. Approval binds the wording and only `check` puts it in the
  canonical spec, so a remedy naming approval alone walked the author into the silent skip.

### REQ-change-093

The lifecycle SHALL compute a handoff readiness for every change so an agent can tell whether
clearing its context — or handing the change to a fresh session — loses anything the lifecycle
still needs, and SHALL say what to do first when it does.

Acceptance Criteria
- `handoff_summary` returns `safe`, `conditional`, or `not-yet` with a plain-language reason, the
  resume command `specsync change status <id>`, and, when readiness is not `safe`, at least one
  concrete step to take before clearing. No reason or step contains a digest.
- `classify_handoff` is a pure function of `HandoffSignals`; every branch has a unit test that
  needs no repository.
- A frozen sequence ledger, a definition changed after its approval, an invalid correction
  ledger, stale legacy terminal evidence, and an archive whose terminal evidence is
  `corrupt-history` are `not-yet`, and the step named is the repair the lifecycle already requires
  (clear the freeze, re-approve, restore the ledger, reopen, restore the archive from trusted
  history).
- A Draft is never `safe`: open questions, stub artifacts, and a complete-but-unapproved
  definition are each `conditional`, and the steps name answering, finishing the artifacts, or
  approving — approval is the first boundary a fresh session can resume from.
- Uncommitted edits under the change's `affected_paths` make an approved, implementing, or
  verifying change `conditional` and name committing or writing the intent into `change.md`;
  uncommitted files under `.specsync/` alone never do, because `change review` then
  `change finalize` runs with that evidence uncommitted by design.
- A verifying change whose recorded verification is stale is `conditional` and names
  `specsync change check <id> --commit`; one whose verification is current is `safe` whether the
  scoped review has been recorded yet or not, and the reason says which step a fresh session
  resumes at.
- An accepted workflow-v2 change and an archived change whose terminal evidence authenticates
  (including `historical-transition-unproven`) are `safe`.
- For an archived change whose terminal evidence is `corrupt-history`, the step before clearing
  and the summary's next action are one string ending with `specsync change status <id>`:
  `git restore --source=HEAD --staged --worktree -- <package>` when the package's tracked files
  differ from `HEAD`; the same restore from the commit that archived the package (named by
  `git log -1 --diff-filter=A --format=%H -- <package>/state.json`) followed by a commit when the
  package matches `HEAD` but not that commit; and, when the package still matches the commit
  that archived it or no package or repository can answer, `specsync change status <id> --json`
  to read what `terminal_evidence.reason` names and restore that from trusted Git history, since
  a package restore would change nothing. Running the named step through a shell on a tampered
  archive returns `authenticated-history` and `safe`.
- Every change ID and path interpolated into a printed command is single-quoted as one shell
  word, so a change ID or archive directory carrying shell metacharacters cannot turn the
  copied step into a different command.
- `handoff_summary` judges an archived record from the same terminal-evidence verdict
  `ChangeSummary` carries, so the text line cannot call an archive `safe` that JSON reports as
  `corrupt-history`.
- `ChangeSummary` carries the decision as `handoff`, so JSON consumers read the same verdict the
  text line prints.

### REQ-change-094

The lifecycle SHALL allow audited reopening of manifest-less legacy accepted evidence when historical acceptance reconstruction fails, even if current delivery inputs match and the verification commit is anchored.

Acceptance Criteria
- Record an explicit legacy reconstruction failure cause and preserve prior closing and verification evidence.
- Reconstructible legacy evidence and current manifest-backed evidence remain non-reopenable.
- Authentication, explicit actor and reason, fresh verification, and new closing approval remain mandatory.
- Reverification and acceptance produce a modern manifest that can be archived.

### REQ-change-095

The change lifecycle SHALL let a module own delivery paths beyond its spec's `files:` through `[modules."<name>"] owns` in the project configuration, and SHALL judge a successor's eligibility to supersede a predecessor entry signed under a reserved exact owner by the module that owns the path now rather than by the frozen label.

Acceptance Criteria

- An `owns` entry is a project-relative file, or a directory that owns everything beneath it, matched the way `affected_paths` scopes are; an acceptance manifest signs a matching path under every declared module that owns it, ahead of the reserved `@exact:test` and `@exact:delivery` classes, and a directory entry takes ownership like a file.
- Configured ownership reaches acceptance manifests and semantic succession only: an owned path is not a source mapping, `specsync check` demands no spec coverage for it, a spec's `files:` list still does not lift a mapped test out of `@exact:test`, and no path under `.specsync/` or among the protected SDD paths is configurable.
- `change supersede` accepts a module for a predecessor entry whose signed owners are all reserved exact labels when the module owns the path under the current configuration, refuses it otherwise naming the frozen label and the `owns` remedy without persisting anything, and keeps refusing a module that is not a signed owner of an entry a module signed.
- The succession tuple is unchanged — the successor's module, the predecessor entry digest, and the successor entry digest, with the digest-matches-base-tree rule intact — and no owner correction, reopen, or additional audit record is required to supersede an exact-only entry.
- A workflow-v2 successor that edits, deletes, and re-signs exact-only inputs of an archived bootstrap change finalizes, and the bootstrap is successor-covered on the full walk and on the active-only audit before and after the archive commit.

### REQ-change-096

Bounded Git timeout regression coverage SHALL observe child identity independently of child startup scheduling while checking deadline failure, child termination/reaping, and completion with blocked stdin.

Acceptance Criteria
- Obtain the spawned PID in the parent through test-only instrumentation.
- Retain the short deadline and a payload larger than pipe capacity.
- A delayed-start control does not require a child-written file.
- A cleanup defect is still detected by a negative control.
- Production deadlines, public API, and lifecycle policies remain unchanged.

### REQ-change-097

Accepting or finalizing a change SHALL refuse to adopt the implementation commit when verification-attempt history is missing or empty. A Verifying change SHALL refuse to recreate a missing attempts ledger.

Acceptance Criteria
- Deleting `verification-attempts.json` after a successful `change check` is the same evidence as emptying it: adopted finalization fails with `verification attempt history cannot adopt the implementation commit`.
- `record_verification_attempt` on a Verifying change whose ledger file is gone fails with `verification attempt history is missing` instead of writing a one-entry ledger.
- A present non-empty ledger is unchanged: the current verification is appended as before.

### REQ-change-098

Canonical definition-artifact payloads SHALL fold CRLF to LF before any other rewrite, so a `core.autocrlf=true` checkout hashes the same as the LF blob Git stored.

Acceptance Criteria
- `canonical_definition_artifact_payload` for a non-tasks file with `\r\n` equals the LF form of the same bytes.
- A lone `\r` is content and is preserved.
- For `tasks.md`, the CRLF fold runs before checkbox-space rewriting, so post-move archive hashing cannot disagree with the pre-move digest on line endings alone.

### REQ-change-099

Lifecycle validation limits SHALL be bundled under `src/` so the crates.io include set can compile the binary.

Acceptance Criteria
- `include_str!("lifecycle-validation-limits.json")` is the source of `lifecycle_validation_limits()`.
- `src/lifecycle-validation-limits.json` is byte-identical to `.github/scripts/lifecycle-validation-limits.json`.
- `cargo package --list` includes the bundled JSON.

### REQ-change-100

A workflow-v1 change in `Verifying` SHALL advertise `specsync change verify` then `specsync change accept` then `specsync change archive` as `next_action`, never the workflow-v2 `check` / `review` / `finalize` verbs.

Acceptance Criteria
- `summarize_change` for `workflow_version < 2` and `ChangeState::Verifying` (artifacts complete, approval valid) names `verify`, `accept`, and `archive` in that order.
- The `Accepted` arm is unchanged: v2 still names `finalize`; v1 still names `archive` and does not skip `accept`.
- A workflow-v2 Verifying change still names `check` / `review` / `finalize`.
- Tests that assert this contract SHALL NOT interpolate the full `next_action` string into panic messages.

### REQ-change-101

The uncovered-paths remediation SHALL spell `specsync change new` with `--kind bug-fix`, which is the clap value `ChangeKind` accepts.

Acceptance Criteria
- The message contains `--kind bug-fix` and does not contain `--kind fix`.
- `--no-spec-change` and `--path` guidance is unchanged.

### REQ-change-102

The change domain SHALL be the single answer to which untracked paths a lifecycle commit for one change may stage, so the command layer that stages them never decides lifecycle ownership itself.

Acceptance Criteria
- `lifecycle_commit_scope` names the change's active workspace, its archive package once finalization has moved it there, each affected spec's canonical spec file and its canonical companions, and the lifecycle ledgers: the sequence ledger, the workflow-v2 and legacy-archive baselines, the bootstrap record, and the hash cache.
- It never names the change's `affected_paths` prefixes or an affected spec's whole directory, so an untracked file under `src/` or beside a spec is not owned by being there.
- It names the project lock and the transaction journal separately as runtime files, which a lifecycle commit neither stages nor reports.
- Canonical spec paths resolve through the same registry-aware resolver materialization writes through.
- Paths are project-relative with forward slashes, sorted and deduplicated, and a change that cannot be loaded is an error rather than an empty answer.

### REQ-change-103

The workspace digest SHALL name each clean tracked file by its Git object id, in a frame that file bytes cannot match. It SHALL read file bytes only for dirty or untracked paths, and those reads SHALL stay inside the existing payload bound. `filter=lfs` SHALL be accepted as that object id only when the configured clean and smudge commands are Git LFS and the blob is an LFS pointer. Every other content filter SHALL still fail closed. Acceptance evidence for a change SHALL still hash the bytes of the paths that change covers.

#### Acceptance Criteria
- A clean `filter=lfs` file with the Git LFS driver and a pointer blob does not fail the workspace digest, and the recorded identity is the Git object id rather than the working-tree bytes.
- Replacing that file with the object id text changes the workspace digest.
- A `filter=lfs` path whose clean command is not Git LFS fails closed.
- A `filter=lfs` blob that is not an LFS pointer fails closed.
- `filter=demo`, `working-tree-encoding`, and `ident` still fail closed.
- Scoped acceptance of a covered path still hashes that path's bytes.

### REQ-change-104

The system SHALL ask a draft which existing lessons or principles constrain it, and only when that draft's affected modules have substantive lessons or the project names a principles file. The question SHALL be asked after the rest of the interview is answered. The answer "none" SHALL be complete. The answer SHALL be part of the stable scope, so changing it SHALL require a fresh approval. Re-answering the affected specs SHALL clear the answer. A draft with neither lessons nor a principles file SHALL NOT be asked. A change that has left draft without the answer SHALL NOT be sent back to the interview.

Acceptance Criteria
- A draft with substantive module lessons, or a configured principles file, is asked `constraints` only after the other interview questions are answered.
- The answer "none" completes the question.
- Changing the answer changes the scope digest.
- Re-answering affected specs clears the answer and asks the question again when it still applies.
- A scaffold-only context and no principles file produces no question.
- An approved, implementing, verifying, or archived change that never recorded the answer stays a complete interview.

### REQ-change-105

The system SHALL refuse approval of a selected plan until that plan says what will be done, what will not be done, the steps, the risks, and which existing constraints are honored. The system SHALL NOT add a plan to a change the interview did not select one for.

Acceptance Criteria
- A selected plan is incomplete while Approach, Out of scope, Steps, Risks, or Constraints consulted is missing or has no substantive text.
- A required section ends at the next level-two heading. Prose under a later heading does not fill the empty section above it.
- Headings, blank lines, HTML comments including those that span lines, and placeholder TODO lines are not substantive text.
- Headings and text inside a fenced code block are not part of the plan. A plan whose required headings appear only inside a fence is incomplete.
- A closing fence is a marker run followed only by whitespace. An info string does not close a fence. A marker indented more than three spaces is not a fence.
- A plan with substantive text under each of those headings is complete, once the generic artifact TODO check also passes.
- A bug fix and a documentation change that did not select a plan are not given one by this rule.
- Status names the plan file while it is incomplete, using the same predicate as approval.

### REQ-change-106

The system SHALL show, when a change is opened, shown, or asked its status, the lessons already written for the modules it touches and the project's principles when a principles file is configured. The text result and the structured result SHALL carry the same briefing. A module context that holds only its generated scaffold SHALL NOT be presented as knowledge. The briefing SHALL be bounded and SHALL name the file for anything left unread. An unreadable context or principles file SHALL NOT fail the command.

Acceptance Criteria
- The briefing lists each affected module context that has substantive lines, with the path, the substantive line count, a bounded excerpt, and whether the excerpt was cut.
- When `specs/<module>/<module>.3md` exists and parses, lessons come from every plane after the first, in the order written, and the briefing names that file. The first plane is context even when its `z` is higher than a later plane. `context.md` is not the lesson source for that module.
- When that file does not exist, or it cannot be parsed, `context.md` remains the lesson source.
- An unreadable or invalid 3md file does not fail the command.
- The lesson bundle names the same file the briefing reads.
- The context path is the directory of the module's canonical spec, including a non-default specs directory or a registry mapping.
- A multiline HTML comment is not a lesson.
- A scaffold-only context is absent.
- A configured principles file is named, with a bounded excerpt, or marked missing when it cannot be read.
- No principles file produces no principles entry.
- The briefing is computed and is not stored on the change record.

### REQ-change-107

The approval tests SHALL keep passing after a selected plan must name its sections. A test that sets a principles file SHALL record which constraints apply before approval, and changing that file SHALL still invalidate the approval.

Acceptance Criteria
- `stale_definition_approval_is_rejected` approves a plan that names its sections.
- `project_principles_are_part_of_the_approval_digest` records a constraints answer and still fails approval after the principles file changes.

### REQ-change-108

The system SHALL refuse to open a change when `.specsync/sdd.json` exists and cannot be parsed. The error SHALL say the SDD policy is invalid. A missing policy file SHALL still open a workflow-v2 change. An existing workflow-v2 baseline SHALL NOT skip this check.

Acceptance Criteria
- `create_change` fails with `invalid SDD policy` when the policy file is `{`.
- The same failure happens when `.specsync/workflow-v2-baseline.json` is present.
- No change directory is created for that failure.
- `configured_principles_file` returns that same error for the same file.
- A missing policy file is not reported as an invalid policy.

### REQ-change-109

The change module SHALL keep the same behavior when its source is split across `include!` files, and none of those files SHALL exceed 5,000 lines.

Acceptance Criteria
- `src/change.rs` includes its parts in the original order.
- Every part is a whole-item slice of the previous file.
- No change source file exceeds 5,000 lines.

### REQ-change-110

The system SHALL recognize an authenticated finalized policy migration from the current delivery during active-only audit when its signed policy entry matches the current policy, without treating an unrelated historical archive or unauthenticated claim as coverage.

Acceptance Criteria
- A finalized same-delivery policy migration passes after the active workspace is removed.
- No-record, wrong-scope, fabricated archive, altered approval/review/finalization evidence, and changed policy after acceptance remain failures.
- A policy archive from a different delivery does not authorize a new policy edit.
- Existing active coverage and full-history integrity checks retain their behavior.
- Only relevant delivery archive candidates are authenticated; unrelated historical damage is not a new active-audit failure.
- The Algonest #48 reproduction passes with the candidate, and original6.1.1 still fails it. Missing history objects remain explicit verification limitations.

### REQ-change-111

The system SHALL report a definition approval as invalid in change status, in the handoff verdict, and in the active-workspace audit whenever the semantic delta bodies that approval recorded no longer match the change's delta files, using the same binding that materialization and acceptance enforce, and SHALL NOT reinterpret an approval that recorded no delta binding.

Acceptance Criteria
- For an approved, implementing, or verifying change whose effective definition approval recorded delta digests, a delta body edited after approval makes `summary.approval_valid` false, makes `summary.next_action` name `specsync change approve <id> --actor <name>`, and makes the handoff readiness `not-yet`.
- `summary.changed_approved_deltas` lists the edited modules in sorted order and is omitted from JSON when empty.
- `change audit` reports, for that change, the same refusal `change check` gives.
- A body that differs only in CRLF line endings, a restored body, and a re-approved body keep or return `approval_valid` to true.
- A later definition approval that withdraws a recorded binding reads as invalid, as materialization already refuses it.
- A ledger in which no definition approval recorded a delta digest keeps its current status verdict.
- Draft, accepted, and archived summaries, accepted and archived authentication, persisted ledgers, and every digest are unchanged.
- An edit to `plan.md` or another artifact outside the approved scope keeps `approval_valid` true and stales only execution evidence.

### REQ-change-112

The system SHALL apply the effective-contract gate to the change being verified on every `change check`, including the first one after materialization, and SHALL record that gate's refusal of a module the change declares as a failed verification attempt, so recorded verification never reports a pass the lifecycle's own gates refuse for the same tree.

Acceptance Criteria
- The first `change check` of an approved change whose declared module carries an unsuppressed validator warning fails and names the warning, as later checks, finalization, and audit already do, whatever `enforcement` is configured.
- The refusal is appended to `verification-attempts.json` and projected into `verification.json` with `passed: false`; `commands[]` keeps the scoped `specsync check --spec <name> …` entry and adds `specsync change check <id> (effective contract: <module>, …)` with `success: false`.
- A failed attempt supersedes an earlier recorded pass, so status names `specsync change check <id>` rather than review, and `change check --commit` commits nothing.
- A change whose declared modules carry no unsuppressed warning records exactly the single scoped command it recorded before.
- Gate failures not attributable to a module the change declares are refused before any attempt is recorded, as before.
- `specsync check`, configured `enforcement`, explicit `--strict`, the gate's promotion rule and suppressions, path-only scope, `--no-spec-change` changes, acceptance, audit, and accepted or archived evidence are unchanged.

### REQ-change-114

The system SHALL leave every file under a directory prefix listed in `.specsync/sdd.json` `verification_ignored_paths` out of the verification project-input digest, whether the file is tracked or untracked, SHALL leave the scoped-review descendant walk unchanged, and with the list absent or empty the digest and every persisted byte SHALL be unchanged.

Acceptance Criteria
- With `[".chough/"]` committed and the change verified, adding, editing, deleting or committing a file under `.chough/` before the review keeps verification current and status names `change review`.
- `change review` records a passing review and `change finalize` accepts it; an uncommitted edit under a listed prefix after the review leaves the review current.
- Any commit after the review, including one that touches only listed paths, still stales the review through the descendant walk.
- Edits or commits outside every listed prefix, and a path equal to an entry without its trailing slash, still stale verification and review.
- Adding, editing or removing an entry stales verification and review.
- No persisted record gains a field; with the list absent or empty, the digest, `verification.json`, `review.json` and status JSON are byte-identical to before.
- `ignored_paths`, `meaningful_paths`, coverage, `config.toml` `exclude_dirs` and `exclude_patterns`, acceptance-input digests and manifests, and lifecycle commit staging are unchanged.

### REQ-change-115

The system SHALL accept a `verification_ignored_paths` entry only when it is a bounded, portable, project-relative directory prefix that overlaps no specs, sources, SDD policy inputs or `meaningful_paths` scope, SHALL refuse an invalid list in every lifecycle verb that loads SDD policy, and SHALL refuse, for a change, an entry that overlaps that change's declared `affected_paths`.

Acceptance Criteria
- An entry is refused, naming it and the rule, unless it ends in `/`, has no glob or pattern character, is relative with no empty, `.` or `..` component, backslash or control character, is not and does not lie under a symlink on disk, is at most 128 bytes, and neither duplicates nor nests in another entry; the list holds at most 8 entries.
- An entry is refused when it overlaps, ignoring ASCII case, `.specsync/`, the configured specs tree, a `source_dirs` entry, the configured `schema_dir`, any `meaningful_paths` scope, an SDD policy file, the `principles_file`, a `custom_artifacts` template, a file listed in a spec's `files:`, a path-form spec `depends_on` entry, or a file named in a spec's `Consumed By` table; an `ignored_paths` scope covering the entry does not relax this.
- In a project rooted in a subdirectory of its Git repository, an entry excludes only paths under that project's own copy of the prefix.
- `change new`, `change answer`, `change approve`, `change check`, `change review`, `change finalize`, `change ship` and `change audit` refuse an invalid list as an invalid SDD policy, `change status` makes fixing the entry its one next action, `change ship-status` reports it as a blocker naming the entry, and no evidence is recorded against it.
- A change whose `affected_paths` overlap an entry is refused at create, at the paths answer, and at approve, check, review and finalize; other changes are unaffected.
- `specsync check` does not read the list.

### REQ-change-116

The lesson bundle that finalization writes SHALL add no U+2013 (en dash) or U+2014 (em dash) of its own; user-supplied record text it copies SHALL be copied unchanged.

Acceptance Criteria
- The bundle's first line is `# Lesson bundle: <id>`.
- The bundle's fixed text (heading, section headings, labels and the guidance paragraph) contains neither character.
- User-supplied record text is copied unchanged, including any dash it holds: the change title, every acceptance criterion, affected specs and paths, recorded verification command strings, and the bodies of the change's `context.md`, `design.md` and `testing.md`.
- A bundle assembled from a record and artifacts whose user-supplied text holds neither character contains neither character.
- Archived bundles are not rewritten, and nothing reads the heading back.

### REQ-change-113

The system SHALL report, for an approved, implementing, or verifying change whose scope approval is current, each affected module's declared requirement IDs and spec sections from the semantic delta body the definition approval recorded, with a per-module integrity flag computed by the same delta binding that materialization enforces, and SHALL NOT report any requirement as approved from a delta body that no longer matches its approval.

Acceptance Criteria
- After approval, `approved_delta_summaries` for a change whose `shipping` delta declares `### REQUIREMENT REQ-shipping-002` under `## ADDED` returns a `shipping` entry with `delta_intact: true`, `requirements.added == ["REQ-shipping-002"]`, the modified spec section names under `sections.modified`, and `materialized: false`.
- Editing that delta body after approval makes the entry's `delta_intact` false and omits its `requirements`, `sections` and `materialized`; the module is the one `ChangeSummary.changed_approved_deltas` names, and `approval_valid` is false.
- A body that differs from the approved one only in CRLF line endings keeps `delta_intact: true`.
- A later definition approval that withdraws a recorded delta binding reports every declared module with `delta_intact: false`.
- A ledger in which no definition approval recorded a delta digest reports each module with `delta_intact: null` and no requirements, never `false`.
- After a passing `change check` materializes the delta, `materialized` is true. After a corrected delta is re-approved following materialization, `materialized` is false until the next `change check` applies it.
- A requirement ID that appears only in prose inside a delta body is not reported; only `### REQUIREMENT <id>` item headings are.
- A body whose digest matches the approval but that the current delta grammar refuses reports `delta_intact: true` with a `declarations_error` and no `requirements`, `sections` or `materialized`; `approval_valid` stays true and `changed_approved_deltas` stays empty for it.
- Within one read scope or one definition-mutation result, `ChangeSummary.changed_approved_deltas`, `approval_valid`, `delta_intact`, the reported requirement IDs and sections, and `materialized` all derive from one read of each delta file; the materialization and acceptance refusals still read the delta files live.
- Draft, accepted and archived changes, and an approved change whose scope approval is stale, return an empty list. `ChangeSummary`, `changed_approved_deltas`, persisted ledgers, digests and materialization are unchanged.

### REQ-change-117

The system SHALL permit an audited reopen of an active workflow-v2 accepted record with a historical scoped-review workspace mismatch only when its entire raw package exactly matches the current remote-default Git tree, that commit is an ancestor of HEAD, and all other definition, execution, identity, closing and finalization digest checks pass.

#### Acceptance Criteria

- The mismatched old review remains invalid for ordinary acceptance and archive.
- Recovery preserves every original byte through a recorded immutable Git commit and returns only to verifying.
- Uncommitted changes, extra or missing files, mode or content tampering, missing remote-default evidence, and other invalid closing fields refuse without mutation.
- Fresh verification and a newly current scoped review are mandatory before closing again.
- Existing bounds and legacy serialized cause values are unchanged; old records serialize identically when the optional anchor is absent.
- Within the exact recorded workspace-mismatch recovery, the inconsistent old review hash is retained as untrusted evidence rather than authenticated. The finalization self-digest and all non-review digests remain strict.
- A review-hash mismatch without a review-workspace mismatch remains refused without mutation.

### REQ-change-118

The system SHALL offer explicitly approved historical receipt reconciliation that preserves and authenticates an immutable ancestral Git package and its frozen signed manifest, binds each exact adopted predecessor path/module/digest to fresh current acceptance evidence, and labels the historical file transition unproven rather than treating it as normal before/after succession.

#### Acceptance Criteria

- The opt-in request is definition-bound, carries an actor/reason and the immutable definition base, and invalidates old scope approval. Accepted or archived successors cannot be silently converted.
- Every predecessor is an already accepted normal manifest-backed receipt whose complete regular-file package matches its committed anchor; after native archival only the authenticated accepted-state preimage and exact native-generated archived change.md may be normalized back to their immutable original bytes. All definition, execution, identity, manifest, closing and finalization checks remain strict.
- Versioned closing evidence binds sorted exact package/manifest witnesses and one-to-one current path/module/digest tuples, with transition status unproven. A forged witness, missing anchor, altered receipt, non-ancestor, unsupported or protected path, stale current successor or a claim of proven history refuses.
- Fresh verification and current scoped review are mandatory after new scope approval. The new mode does not fabricate old dates, approvals, source bytes, product acceptance or historical transitions.
- Ordinary succession retains its actual base-file check and unchanged serialized schema/digests. All resource bounds remain unchanged. Historical mode supports regular-file, missing and directory entries, not symlinks or gitlinks.

### REQ-change-119

The system SHALL answer read-only lifecycle reads (`change status`, `change show`, `change list`, `change ship-status`, `change audit`) without re-deriving work whose inputs have not changed: inside one read scope an archived change's terminal evidence is computed for that change alone, scoped-review Git reads and archive lookups are memoized, and passing archived workflow-version history verdicts and archive introductions are reused across invocations from a cache under the Git directory that is keyed by content digest and invalidated by Git history.

#### Acceptance Criteria

- Output of every read is identical to a live read: same JSON, same refusals, same first error.
- A cached workflow-version history verdict is reused only when the SHA-256 of the archived `state.json` bytes, its archive path, the build identity and the repository prefix all match, the commit it was computed at is an ancestor of `HEAD`, and no commit in between touched that change's active or archived state paths. Anything else recomputes it live, and only passing verdicts are stored.
- That invalidation walk uses `:(top,literal)` pathspecs. A project rooted in a subdirectory of its Git repository does not reuse a cached pass across a commit that touched the change's state, including a workflow-version downgrade that a later commit restored.
- Archive introductions are cached per commit, which Git makes immutable, and only when every Git read behind them answered.
- The cache lives under the Git directory, is never a project input, is disabled in a shallow repository, and a failure to read or write it falls back to live reads. Mutating commands keep reading live.
- The Git processes one archived status read spawns do not grow with the number of other archived changes once the cache is warm, and a benchmark against a Raven-sized fixture (124 archived changes, 120 specs) times the read against the 2 second budget.

### REQ-change-120

When `change approve` refuses affected paths that no declared module canonically owns, the error SHALL say which spec to edit for each refused path. Ownership itself is still decided exactly as REQ-change-053 requires.

#### Acceptance Criteria

- A refused path that some spec under the specs directory already lists in its `files:` names that spec file and its module, and says to declare it with `--spec`. Every such spec is named, sorted by spec path, whatever order the file system returns them in.
- A listing spec that cannot own the path says why: it has no `module:`, or it is not the canonical spec of the declared module it names.
- Otherwise the refused path names the declared spec whose `files:` entries share the longest directory with the path, comparing whole path components, and says to add the path to that spec's `files:` list. When no declared spec shares any directory with the path (depth 0), the refusal gives the generic message instead of naming a spec.
- The same paths are refused as before; only the wording of the refusal changes.

### REQ-change-121

The system SHALL let `change finalize` and `change archive` reuse, for the duration of one command, Git answers that cannot have changed while it runs, so that one finalize does not re-derive every other archived change's history: the persistent workflow-version history verdicts and archive introductions of REQ-change-119 under the same digest and history rule, Git queries that name only full object ids, the repository prefix, and a passing workflow-v2 baseline history verdict keyed by the resolved `HEAD` id, the baseline path, its cutoff and its exact bytes.

#### Acceptance Criteria

- Every finalize and archive verdict, refusal and first error is identical to a live run.
- Only queries naming full object ids are memoized; queries naming `HEAD`, a branch or the working tree run live, `HEAD` is resolved live on every baseline check, failed Git runs are not stored, and refusals are always re-derived.
- The memo lives only inside the command's own scope and takes no working-tree snapshot, so the command sees its own writes.
- The Git processes one finalize spawns grow by at most 12 per other archived change.

### REQ-change-122

A definition approval SHALL record, for every `## MODIFIED` and `## REMOVED` delta item, a digest of the canonical block it was approved against. When such an item is not already applied and its canonical block no longer matches that recorded base, `change check` SHALL refuse before writing any canonical file and `change audit` SHALL report the same refusal. The refusal SHALL name the module and block and SHALL name rewriting the delta body onto the current canonical text, `change approve` and `change check` as the remedy. An approval that recorded no bases is unknown, not violated, and keeps materializing as before.

#### Acceptance Criteria

- A stale MODIFIED item whose block main rewrote after approval is refused by `change check`, both before and after the first materialization, and the canonical file keeps main's bytes.
- `change audit` reports the same refusal for that change.
- The refusal names the module, the requirement or section, and `specsync change approve <id>`.
- A rebased and re-approved delta materializes, keeping the other change's text it now carries.
- A block main changed that this change does not touch is not refused.
- An approval without recorded bases keeps materializing as before.

### REQ-change-123

`change send-back` SHALL move an approved, implementing or verifying change back to draft so its scope questions can be answered again. It SHALL require a non-empty human actor and reason, SHALL append both, with the prior state, to `approvals.json` under `send_backs`, and SHALL cancel the definition approval recorded before it without removing any recorded approval, verification or other evidence. A definition approval recorded before the latest send-back SHALL NOT satisfy the definition gate or bind a scoped review; only a fresh `change approve` does. A scoped review recorded before the send-back SHALL stay in the append-only review history and SHALL still validate against the approval it bound; it SHALL NOT stop a fresh `change review` after re-approval, and `change status` SHALL report it stale rather than unreadable. A send-back SHALL record how many review attempts preceded it, and a scoped review recorded before the latest send-back SHALL NOT be current, even when the re-approved definition has the same digest; for a send-back recorded without that count, a review SHALL count only when its timestamp is later than the send-back's. A send-back the committed ledger already holds SHALL NOT be removable or editable to bring a cancelled approval back. A send-back committed in any revision of the ledger SHALL NOT be removable or editable later, and the project audit SHALL report one that was. The send-back SHALL list the spec items the change had already applied, including REMOVED items, with each block's text from before the change, and the modules whose Change Log carries the change's row. After a send-back, materialization SHALL converge on what the canonical tree already carries, so a kept REMOVED, ADDED or MODIFIED item is not applied twice and a kept ADDED item may change its body. `change check` and `change accept` SHALL refuse, naming the spec file and the item and saying what to restore, while an item the new scope no longer declares is still applied, or a module it no longer declares still has the change's Change Log row.

#### Acceptance Criteria

- A verifying change sent back can have `affected_paths` answered again, is approved, checked, verified and accepted under its own ID, and an already-applied section is not applied twice.
- A missing actor or reason is refused, naming `--actor` or `--reason`, and nothing is written.
- The send-back record keeps the actor, reason and prior state; earlier approvals and `verification.json` are unchanged; a second send-back appends.
- Until re-approval the definition gate refuses, says the change was sent back, and names `specsync change approve <id>`.
- A draft, accepted or archived change cannot be sent back.
- A sent-back change that owns a spec can be finalized and its archive passes the project audit.
- A re-scope that drops an item the change already applied is refused by check, naming the spec file and the item, until that text is taken out of the spec.
- The definition approval event resolver refuses an approval that a send-back cancelled.
- Removing a committed send-back from `approvals.json` is refused and does not bring the cancelled approval back.
- After a send-back, a change with a REMOVED item materializes and checks again (0xLeif's repro), and a kept ADDED item with a new body replaces the change's earlier body.
- A dropped MODIFIED item is refused with advice to restore its pre-change text, quoting it; a dropped REMOVED item is listed and refused until the block is restored.
- A module dropped from `affected_specs` is refused while its Change Log keeps the change's row.
- A send-back deleted in a later commit is refused by check and reported by the project audit.
- A change reviewed before its send-back is re-approved, re-checked and reviewed again, and finalize archives it with both review attempts in its history, the first unchanged (#914).
- After a send-back that changes the scope, `change status` reports the earlier review stale with the re-review next action, and `change ship` refuses it.
- A review bound only to an approval a send-back cancelled is never current.
- After a send-back with an unchanged scope and a `change check` without `--commit`, `change status` names `change review`, `change ship-status` reports the earlier review stale, and `change ship` refuses until a fresh review is recorded (#960).
- A send-back records the number of review attempts before it; a review recorded before the latest send-back is never current, and a send-back without that count compares timestamps.

### REQ-change-124

The system SHALL let `change finalize` and `change archive` validate in full only the change they archive, reading every other archived change structurally from its own `state.json` bytes without walking its Git workflow-version history, and SHALL keep that per-archive history walk in the opt-in `change audit --archives`, so `change audit --strict` keeps its cost, which names each archived change whose workflow-version history fails.

#### Acceptance Criteria

- The Git processes one finalize spawns do not grow with the number of other archived changes once the persistent read cache is warm. With a cold cache, each other archive adds at most the three reads of the immutable commit that introduced it, and no workflow-version history walk.
- Every refusal finalize and archive issue for the change being archived is unchanged: its own workflow-version history, evidence, review, deltas, ownership, closing approval and archive integrity are still validated in full.
- An archived change that finalize or archive relies on, as a dependency, a semantic successor or a sequence owner, still has its own workflow-version history validated before it is relied on.
- `change audit --archives` refuses a workflow-version downgrade committed into another archive's history and hidden again by a later commit, naming that change, with or without a warm read cache. `change audit` without `--strict` does not take on that archive-history walk.

### REQ-change-125

`change audit` SHALL refuse a living requirements file that contains the same requirement ID more than once, and the error SHALL name that ID.

Acceptance Criteria
- Two `### REQ-<module>-<number>` headings with the same ID in one `requirements.md` fail `specsync change audit`.
- The error text contains that ID.
- The refusal is reported even when SDD policy is disabled.
- A fenced copy of the heading is not a second requirement.
- No other audit refusal is weakened.

### REQ-change-126

A bounded Git read SHALL return when the direct child exits and SHALL NOT wait on a descendant's open pipe after that child exits. `change status` is one such read (SPEED-1, SPEED-1.a, SPEED-1.e). The wall-clock deadline, the output cap, and the stdin-writer join SHALL stay as they are.

#### Acceptance Criteria

- `bounded_git_read_returns_while_a_descendant_holds_the_output_pipe` returns the child's stdout while the descendant is still alive.
- `bounded_git_runner_times_out_a_silent_process`, `timed_out_git_runner_reaps_child_and_joins_blocked_writer`, `timed_out_git_runner_reaps_child_before_child_can_make_progress`, and `capped_git_runner_drains_kills_and_reaps_large_output` still pass. No deadline, cap, or join assertion is loosened.

### REQ-change-127

Lifecycle commands SHALL stay within a tested ceiling of git process spawns per command, and the ceiling SHALL NOT grow with unrelated archived changes. The integration test `eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci` SHALL spawn at most 40% of its 15,563-process baseline measured at 67aa7dc2.

#### Acceptance Criteria

- A test counts git spawns for `change new`, `answer`, `approve`, `check`, `status`, `review` and `finalize` and asserts a ceiling for each.
- The slowest lifecycle integration test spawns at most 6,225 git processes.
- No assertion of an existing lifecycle test is removed or loosened to meet the ceiling.

### REQ-change-128

Working-tree evidence capture SHALL read Git's modified-path output relative to the project root, so a project rooted in a subdirectory of its repository can complete `change check --commit`, and SHALL still refuse any path Git reports outside the project or outside the change's candidates.

Acceptance Criteria
- In a project at `app/`, `change check <id> --commit` commits the materialized spec and the verification evidence.
- A reported path without the repository prefix, or a prefixed path that is not a candidate, is refused as out of scope and named.
- A project at the repository root behaves as before.

### REQ-change-129

The system SHALL refuse `change review` and `change finalize`, and SHALL NOT report readiness `safe` from `change status` or `change ship-status`, for a change whose record is not archived while the remote default branch holds an archive of the same change id that `HEAD` does not contain. The refusal SHALL name the reference and commit of that archive, the unarchived copy, and the recovery: rebase onto that reference dropping the commit that added the copy (or reset to it when the branch has nothing else). Merging, or deleting the copy in a new commit, is not a recovery: the resulting history fails integrity. An archive that `HEAD` already contains SHALL NOT be refused: that is the reopen-and-rearchive path.

Acceptance Criteria
- With the remote default holding an archive of the change, `change review` refuses before recording acceptance and the record stays `verifying`.
- `change finalize` refuses with the same named cause and writes no archive.
- Without an upstream archive, a squash before finalize still reviews and finalizes.
- An unarchived record whose archive `HEAD` already contains is not refused.

### REQ-change-131

A lifecycle command that verifies a change, records a passing scoped review of it, or finalizes it SHALL refuse while the change's selected `tasks.md` has an open task, a `[ ]` checkbox outside fenced code. The refusal SHALL come before the command writes any record. This SHALL hold for a task reopened after a passing check, although checkbox state is outside the approval and verification digests. The refusal SHALL name:

- the task file;
- each open task by line and text, naming at most five and counting the rest;
- the exact command to re-run once they are done.

`change check` SHALL name `specsync change check <id>` with the `--strict`, `--commit` and `--push` flags it was given. `change verify` SHALL name `specsync change verify <id>`. `change review` SHALL name `specsync change review <id> --reviewer <reviewer>`. `change finalize` SHALL name `specsync change finalize <id>`. A scoped review recorded with a verdict other than `pass` SHALL NOT be refused.

#### Acceptance Criteria

- Through the binary, `change check <id> --commit` on an approved workflow-v2 change with one open task exits 1 without recording verification. Its stderr names `tasks.md`, the task's line and text, and `specsync change check <id> --commit`. Ticking the box and running that exact command passes.
- After a passing check, reopening one task makes `change review` and `change finalize` exit 1 without writing a review, acceptance or archive. Each names the open task and its own command. Ticking the box and running the named command succeeds.
- The gate-level test fails when the incomplete-task guard is bypassed (audit mutation SS-M05).

### REQ-change-132

`change status` SHALL name the open tasks and how to close them before the command that would otherwise be next, instead of naming only that command. This SHALL apply to an approved, implementing or verifying change whose approval is current, whose artifacts are complete and whose `tasks.md` has an open task. It SHALL apply in the text `Next:` line and in JSON `summary.next_action`.

#### Acceptance Criteria

- With one open task on an approved change, text and JSON status name the task file, the task's line and text, and then `specsync change check <id>`.
- After a passing check with one task reopened, text and JSON status name the open task before the review step.

### REQ-change-133

When `change approve` refuses a draft because its definition is unfinished (no acceptance criterion, no affected path, no affected spec without a justified no-spec change, or interview questions still open), the refusal SHALL name the `specsync change answer` command that supplies the missing answer. A record past draft SHALL NOT be told to run `change answer`, which refuses outside draft. The approval guard SHALL refuse exactly the definitions it refused before.

Acceptance Criteria
- With no acceptance criterion, a draft's refusal names `specsync change answer <id> acceptance_criteria "<observable outcome>"`.
- With no affected path, a draft's refusal names `specsync change answer <id> affected_paths "<path>"`; with no affected spec and no justified no-spec change, `specsync change answer <id> affected_specs "<module>"`.
- With interview questions still open, the refusal names every remaining question and the `specsync change answer <id> <question> <answer>` command for the first, giving the question's choices as the placeholder (`<yes|no>`, `<none>`), or `"<answer>"` when it has none.
- Running the named command with its placeholder filled moves approval on to the next refusal or to the selected-artifact refusal.
- An approved, implementing or verifying record that fails the same checks is refused with the same condition text and no `change answer` command.
- A definition that approval refused before is still refused.

### REQ-change-134

The persistent read cache SHALL be reused only by the build that wrote it. Its build identity SHALL be the package version plus the build fingerprint of REQ-hash-cache-005. A cache written by another build SHALL NOT be loaded, so its passing workflow-history verdicts are recomputed live. That includes another build of the same version that shares the Git common directory, such as a sibling worktree, and a build from before this requirement.

#### Acceptance Criteria

- A persistent read cache stamped with the pre-#878 identity is not reused. The next read recomputes the verdict and rewrites the cache under this build's identity.
- Removing the fingerprint from the read-cache identity fails that test.

### REQ-change-135

The archive-introduction index SHALL be built from one `git log` walk of the archive root that names the paths each commit added, and SHALL read package bytes through the per-command object batch reader, so the Git processes a cold archived `change status` spawns SHALL NOT grow with the number of commits that introduced archives. The index SHALL hold the same introductions, and every refusal it backs SHALL be unchanged (SPEED-1.a).

Acceptance Criteria
- `cold_archive_introduction_index_does_not_grow_with_archive_introductions`: a cold index build spawns fewer than 3 more Git processes after 4 more archive-introducing commits.
- The index built from one walk equals the per-commit index on the same history, including merge commits and packages added in more than one commit.
- The deterministic history bound still refuses a history over it.

### REQ-change-136

Whether a workflow-v1 change existed at the trusted pre-v2 cutoff SHALL be decided from the cutoff commit's own tree, read once per command for all workflow-v1 changes, and the verdict SHALL be kept in the persistent read cache keyed by the cutoff commit, which never changes. A warm archived `change status` SHALL NOT spawn more Git processes as the project holds more workflow-v1 changes (SPEED-1.a, SPEED-1.c).

Acceptance Criteria
- `warm_archived_status_git_processes_do_not_grow_with_legacy_changes`: six more workflow-v1 changes add fewer than 3 Git processes.
- A workflow-v1 change absent at the cutoff, added after it, or present at it only under another id is still refused with the existing message.
- A workflow-v1 change archived or moved after the cutoff is still accepted.
- A hidden workflow downgrade is still refused, cached or not.

### REQ-change-137

A read-only lifecycle read given a deadline SHALL stop at the next phase boundary once the deadline passes, SHALL write nothing to the change, SHALL flush every verdict it completed to the persistent read cache, and SHALL report the phase it was in, its progress within that phase when known, and the exact command that resumes it.

Acceptance Criteria
- A read stopped by its deadline leaves every change file byte-identical.
- Verdicts completed before the deadline are in the read cache afterwards, so the next run does not redo them.
- The report names the phase and progress, for example `archive-introduction index` and `140/214`.

### REQ-change-139

The upstream-archive lookup SHALL identify the remote default's archive of a change by the `id` recorded in each archived package's `state.json`, as `find_change_dir` does, not by the package directory name, and SHALL do so with a bounded number of Git processes that does not grow with the number of archived packages.

Acceptance Criteria
- A renamed or undated upstream package of the change is found.
- The lookup uses one tree listing and one batched blob read of the archive directory, plus per-match work only for a matching package.

### REQ-change-140

Inside a change read scope, the workflow-version history of every archived change SHALL be answered from one lifecycle-state history index per command, built from a fixed number of Git walks from `HEAD` over the lifecycle state paths of the changes root and the archive root. Each archive's candidate state paths, its oldest-first state-path history and its history commits' parents SHALL be derived from that index, and every archive SHALL still be fully validated. A cold archived `change status` SHALL NOT spawn more Git processes as the project holds more archives (SPEED-1.a.1). The index SHALL never change a verdict: each archive SHALL be accepted or refused exactly as the per-archive check would, with the same first refusal and message, the same non-canonical archive path refusal, and the same per-archive history and parent bounds. When the index cannot be built (over its commit bound or output limit, or a Git call fails), each archive SHALL fall back to the per-archive check.

Acceptance Criteria
- `cold_archived_status_git_processes_do_not_grow_with_archives`: four more archives add fewer than 3 Git processes to a cold archived status.
- Index-derived candidate paths, history and parents equal the per-archive Git answers on histories with merges, moved or re-archived packages, root commits, a project in a subdirectory, a non-canonical archive path, a deleted creation anchor, a changed identity and a history over the per-archive bound.
- With the index unavailable, every verdict equals today's.

### REQ-change-141

When the lifecycle-state history index answers, the state-file reads and path-exists checks behind archived workflow-version validation SHALL run on the command's own thread through its one object batch reader (REQ-git-utils-010), with a read deadline check between archives (REQ-change-137). A state blob the commit's tree lists but Git cannot read SHALL still refuse, and SHALL never be treated or cached as absent. Prefetch workers (REQ-git-utils-011) SHALL remain the path when the index does not answer.

Acceptance Criteria
- A listed state blob that cannot be read refuses the archive and leaves no cached absence.
- A cold archived status past its `--timeout` still exits 124 with the phase and progress.

### REQ-change-142

When a remote default exists and its archived state records cannot be listed or read, the upstream-archive lookup SHALL report that it cannot tell whether the change is already archived there, rather than reporting no upstream archive. `change review` and `change finalize` SHALL refuse before acceptance and write no archive, and `change status` and `change ship-status` SHALL report the same refusal. The refusal SHALL name the remote default reference and how to find out (SHIP-3.c).

Acceptance Criteria
- A failed listing, a failed or truncated batched read, or a failed addition or ancestry lookup for a matching package is reported as cannot-tell.
- No remote default, or a readable archive with no copy of the change, reports nothing.

