## ADDED

### REQUIREMENT REQ-change-041

Canonical module path resolution SHALL fall back to default `specs/<module>/<module>.spec.md` paths when the local registry file is missing or an inert stub, without weakening fail-closed behavior for invalid non-inert registries.

Acceptance Criteria

- An inert 5.0.1-era empty registry stub does not block `canonical_module_paths` resolution.
- Conventional `specs/<module>/<module>.spec.md` paths remain the fallback when the registry is missing or inert and no mapping applies.
- A non-inert unparsable local registry still fails closed with the exact pre-fix diagnostic `failed to parse local registry {path} while resolving `{module}``.
- Named registries with safe mappings continue to win over the conventional fallback.

## MODIFIED

### SPEC SECTION Invariants


1. Change IDs are monotonically assigned as `CHG-NNNN-slug` across active and archived workspaces.
2. No emergency or force transition bypass exists.
3. Approval digests exclude volatile lifecycle state but include every selected artifact and semantic delta.
4. Any approved definition change invalidates approval until the new digest is approved.
5. Acceptance rejects stale commits, stale contracts, incomplete tasks, failed tests, and missing requirement evidence.
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
22. Local and hosted verification freshness inspect every intervening commit against every parent, permit only the three supported persistence files below canonical active-change IDs, and never infer safety from a net diff or broad volatile-path exclusion.
23. Exact-owner corrections are additive, restricted to an original affected path and a current canonical source owner, and cannot mutate semantic definition fields or prior evidence.
24. A fully valid later accepted sequence owner covers only historical sequence-ledger drift, while the current owner and every non-ledger input remain exact.
25. A structurally valid audited delivery reopen preserves immutable sequence-collision history while fresh verification and closing approval remain mandatory.
26. Accepted-change archival trusts an in-history commit recording the change as accepted with byte-identical evidence when no first-acceptance transition anchor matches, so squash-merged evidence remains archivable while the exactly-one-eligible rule stays fail-closed.
27. Legacy acceptance-manifest reconstruction assigns the exact delivery owner to production-source inputs with no deterministic canonical owner, so adoption-era archived ledgers validate without remediation while newly signed evidence stays fail-closed.
28. Batch exact-owner correction validates every proposed path/module pair independently and fails closed with zero persisted mutations when any entry is invalid.
29. The 5.0 ledger migration backfills reopening digest fields idempotently from recorded evidence only, verifies each repair before writing, and never mutates ledgers it cannot repair deterministically.
30. Canonical module path resolution treats missing and inert local registries as absent fallbacks while non-inert unparsable registries still fail closed with the established parse diagnostic.


### SPEC SECTION Behavioral Examples


**Scenario: Verified feature delivery**

- **Given** an approved feature with `REQ-auth-001`, completed artifacts, and configured tests
- **When** implementation verifies and a human accepts it
- **Then** canonical requirements/specs update, the spec version increments, and the change becomes accepted

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


### SPEC SECTION Error Cases


| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
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

