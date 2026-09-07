Warning: truncated output (original token count: 22020)
Total output lines: 1282

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
- A failed configured command, missing semantic acceptance evidence, and missing requirement evidence produce distinct diagnostics.

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
- The archive preflight forward…12020 tokens truncated… archive-to-active direction fails a test, so the defect where a reopened change could never be closed again cannot return silently.
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
  ledger, and stale legacy terminal evidence are `not-yet`, and the step named is the repair the
  lifecycle already requires (clear the freeze, re-approve, restore the ledger, reopen).
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
- An accepted workflow-v2 change and an archived change are `safe`.
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

