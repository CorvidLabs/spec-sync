---
module: change
version: 21
status: active
files:
  - src/change.rs
db_tables: []
tracks: []
depends_on:
  - specs/hash_cache/hash_cache.spec.md
---

# Change

## Purpose

Provides the spec-sync 5.0 verified spec-driven development lifecycle, including audited recovery and re-verification when governed delivery inputs make accepted evidence stale.

## Contract

1. Every meaningful SDD change moves through draft, approved, implementing, verifying, accepted, and archived states without bypasses.
2. Definition and closing approvals are portable records bound to deterministic SHA-256 digests.
3. Approved semantic deltas form the effective future contract without mutating canonical specs before acceptance.
4. Requirements use stable `REQ-<module>-<number>` IDs, normative SHALL statements, and acceptance criteria.
5. Verification executes only project-configured commands without a shell and rejects unsafe shell syntax.
6. Verification evidence is bound to the tested commit and working-tree inputs, and effective contracts must validate before acceptance.
7. Invalid policy, unavailable coverage comparison, failed evidence, and stale ordering gates fail closed.
8. Concurrent deltas follow declared dependency order and canonical Markdown application preserves unrelated sections.
9. Approval validates complete module-scoped deltas, corrupt state fails closed, and archival failures remain retryable.
10. Permanent requirement tombstones come only from accepted history, and default path coverage includes root delivery metadata.
11. Concurrent effective-contract validations use isolated temporary workspaces.
12. Stale accepted delivery evidence can return only to verifying through an explicit human actor and reason, while prior verification and closing evidence remain inspectable.

## Public API

### Exported Constants

| Name | Description |
|------|-------------|
| `SDD_VERSION` | Current SDD project-layout version written by initialization |

### Exported Types

| Type | Description |
|------|-------------|
| `ChangeState` | Six-state delivery lifecycle: draft, approved, implementing, verifying, accepted, archived |
| `ChangeKind` | Deterministic policy classification for feature, bug fix, refactor, migration, documentation, and operations work |
| `ArtifactKind` | Built-in or custom adaptive companion artifact selection |
| `SddPolicy` | Versioned enforcement, path, verification-command, template, and principles configuration |
| `ChangeRecord` | Durable machine state for one change workspace |
| `CreateChangeRequest` | Validated creation inputs grouped for CLI, imports, and agent clients |
| `ApprovalRecord` | Actor, timestamp, gate, digest, and optional note for one approval |
| `ReopenRecord` | Immutable audit event preserving superseded closing approval, prior verification, actor, reason, transition, and stale/current input digests |
| `ReopenResult` | Deterministic change-plus-audit result returned by the reopen transition |
| `ApprovalLedger` | Ordered portable approval and reopen history |
| `CommandEvidence` | Exit evidence for one configured verification command |
| `VerificationRecord` | Commit-bound verification result, contract digest, command results, and requirement coverage |
| `InterviewQuestion` | Stable deterministic question with choices and recommendation |
| `ChangeSummary` | Human/agent status projection with gate health and next action |
| `SddCheckReport` | Unified lifecycle validation errors, warnings, and checked-change count |

### Exported Functions

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
| `approve_definition` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Validate and record mandatory definition approval |
| `start_implementation` | `root, id` | `Result<ChangeRecord, String>` | Enter implementation after approval and conflict validation |
| `verify_change` | `root, id` | `Result<VerificationRecord, String>` | Run configured tests and record commit/contract evidence |
| `reopen_change` | `root, id, actor, reason` | `Result<ReopenResult, String>` | Move stale accepted evidence to verifying and append an immutable supersession audit event |
| `accept_change` | `root, id, actor, note` | `Result<ChangeRecord, String>` | Record closing approval and atomically apply semantic deltas |
| `archive_change` | `root, id` | `Result<PathBuf, String>` | Move an accepted workspace into the dated archive |
| `summarize_change` | `root, record` | `ChangeSummary` | Project gate health and next action for clients |
| `check_project` | `root: &Path` | `SddCheckReport` | Validate lifecycle state, approvals, conflicts, deltas, and path coverage |
| `check_project_quiet` | `root: &Path` | `SddCheckReport` | Run the same fail-closed lifecycle check while suppressing configured command output for machine-consumable report protocols |
| `adopt` | `root, dry_run, source` | `Result<Vec<String>, String>` | Preview or enable SDD and import OpenSpec or Spec Kit artifacts |
| `detect_verification_commands` | `root: &Path` | `Vec<String>` | Detect explicit fledge, Cargo, Bun, or Swift test commands |

### Exported Methods

| Method | Description |
|--------|-------------|
| `as_str` | Return the stable serialized name for a change state or kind |
| `parse` | Parse user-facing change-kind or artifact names into typed values |
| `file_name` | Resolve an adaptive artifact to its safe Markdown filename |

## Invariants

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
16. Reacceptance of an already-applied change requires the definition digest captured by the latest audited reopen event; further definition changes require a new change workspace.
17. False default lifecycle fields remain absent from new persisted state, while definition validation recognizes both omitted and transitional explicit-false encodings so upgrades preserve existing approvals and verification; explicit acceptance appends stable definition evidence when the latest compatible approval uses the transitional encoding.
18. Audited reopen accepts unreachable verification commits only when canonical acceptance is recorded in current history or later recorded canonical changes govern every affected contract surface.
19. Acceptance appends a Change Log row matching the canonical table's existing column schema and uses the post-bump version when the schema includes `Version`.

## Behavioral Examples

### Scenario: Verified feature delivery

- **Given** an approved feature with `REQ-auth-001`, completed artifacts, and configured tests
- **When** implementation verifies and a human accepts it
- **Then** canonical requirements/specs update, the spec version increments, and the change becomes accepted

### Scenario: Approved intent changes

- **Given** a valid definition approval
- **When** a selected design, requirement, or delta is edited
- **Then** progress is blocked until the new digest is approved

### Scenario: Feature branch rebases onto upstream

- **Given** a change workspace created before new commits landed on the remote default branch
- **When** the feature branch rebases and unified checking computes meaningful changed paths
- **Then** upstream-only paths are excluded and only the feature branch diff requires change coverage

### Scenario: Review fixes stale accepted evidence

- **Given** an accepted change whose governed delivery inputs changed after closing approval
- **When** a human reopens it with an actor and reason
- **Then** the prior verification and closing approval remain in audit history, strict checking stays red until fresh verification, and reacceptance records a new closing approval without reapplying canonical deltas

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Missing acceptance criteria or affected scope | Definition approval fails |
| Missing or invalid semantic delta | Approval, verification, and unified check fail |
| Verification command contains shell operators | Command is rejected without execution |
| HEAD changes after verification | Acceptance requires re-verification |
| Accepted delivery evidence is still current | Reopen is rejected without changing lifecycle or audit state |
| Reopen actor or reason is empty | Reopen is rejected before any mutation |
| Concurrent changes edit the same semantic key | Progress requires dependency ordering or rebase |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| hash_cache | Project SHA-256 dependency for content identity |

### Consumed By

| Module | What is used |
|--------|-------------|
| cmd_change | Complete lifecycle command surface |
| cmd_check | Unified SDD gate before canonical validation |
| cmd_init | New-project policy and version initialization |

## Change Log

| Date | Change |
|------|--------|
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
