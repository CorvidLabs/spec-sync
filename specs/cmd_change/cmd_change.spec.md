---
module: cmd_change
version: 34
status: active
files:
  - src/commands/change.rs
db_tables: []
tracks: []
depends_on:
  - specs/change/change.spec.md
  - specs/cli_args/cli_args.spec.md
  - specs/types/types.spec.md
---

# Cmd Change

## Purpose

Exposes the single one-approval SpecSync lifecycle through equivalent human-readable and structured JSON commands under `specsync change`.

## Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Every status projection provides exactly one concrete next action on the `new → approve → check → review → finalize → GitHub merge` path.
4. Supersede records an explicit digest-bound predecessor/path/module obligation before definition approval.
5. Reopen renders the exact persisted versioned supersession event in deterministic JSON.
6. Correct-owner renders one persisted exact canonical-owner correction and directs the user to definition reapproval.
7. Batch correct-owner resolves repeated paths, a manifest, or `--all-missing` into domain entries, renders the persisted record, and directs the user to definition reapproval without partial mutation on failure.
8. Strict is an additive validator selection on the same workflow and evidence, never a second lifecycle mode.
9. Finalize reports readiness for GitHub merge and never invokes an external merge API.
10. Review renders the same explicit pass/block verdict and stable reviewer claim in text and JSON
    while domain policy preserves append-only attempts and hosted policy authenticates provenance.
11. Existing-change mutations delegate to domain transactions that validate correction-ledger
    integrity after acquiring the persistence lock and render from the returned validated snapshot,
    so neither rendering failure nor a lock-wait race can conceal a mutation that already took effect.

## Public API

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `cmd_change` | `root: &Path, action: ChangeAction, format: OutputFormat, strict: bool` | `()` | Dispatch every change lifecycle command, additive strict validators, and equivalent text/JSON output |

## Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` runs scoped verification for one change only: evidence completeness then in-process spec↔code sync. It does not spawn project tests or rewalk archived terminal evidence.
4. `change audit` reports active-workspace and living-spec integrity only and exits non-zero on report errors.
5. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.
6. `change ship-status` decides readiness from evidence CURRENCY — the recorded plan and tree still match what was verified — never from whether the recorded commit is reachable from HEAD. A squash-merge rewrites that commit, so reachability would make a squash-merged change permanently unfinalizable while its evidence is intact. The rule covers the scoped review as well as the verification: readiness asks whether the recorded review is current, reports that answer as `current`, `stale`, or `unavailable`, and treats only `current` as satisfied. An unavailable guarantee reported as a satisfied one is worse than the refusal it conceals, and readiness that never asks receives no negative answer and reads its own silence as a pass.
7. The lessons loop surfaces at each of the three moments a lesson exists: `change new` names every affected module's `specs/<module>/context.md` that holds substantive prose, a FAILED `change check` names where to record what the failure taught, and BOTH `change finalize` and `change ship` name folding the archived bundle into those specs before their remaining guidance. Every surface is a pointer, never a dump, and none can fail a lifecycle command. A passing `change check` says nothing, and a change owning no affected specs receives the same guidance it received before the fold-back existed. Both verbs also emit a `lesson_bundle` path in `--json`.

## Behavioral Examples

### Scenario: Agent creates a change

- **Given** `specsync --json change new "Add passkeys"`
- **When** creation succeeds
- **Then** JSON includes the record, gate summary, and deterministic questions

### Scenario: Agent reopens stale accepted evidence

- **Given** current governed inputs no longer match an accepted change's closing evidence
- **When** `specsync --json change reopen <id> --actor <human> --reason <text>` succeeds
- **Then** JSON contains the verifying change and versioned audit record with the superseded approval and prior verification

### Scenario: Finalize an implementation PR

- **Given** verification and the configured scoped-review check are current
- **When** `specsync change finalize <id>` succeeds
- **Then** output names the dated archive and says the PR is ready for GitHub merge without merging it

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Unknown change type | Descriptive error and exit 1 |
| Invalid transition | Current and expected states plus exit 1 |
| Missing actor or reason | Clap or domain validation exits non-zero without lifecycle mutation |
| Current or successor-covered accepted evidence | Reopen reports the shared non-stale reason and exits 1 |
| Missing or mismatched supersede obligation | Command reports the exact predecessor/path/module/digest mismatch and exits 1 without definition mutation |
| Invalid exact owner correction | Command reports the domain rejection and exits 1 without lifecycle mutation |
| Invalid batch owner correction or empty discovery | Command reports the domain rejection and exits 1 without lifecycle mutation |
| Scope approver records the scoped review, or the current verdict is blocking | Command reports the independent-review rejection and finalization remains blocked |
| Invalid correction ledger before answer, depend, or supersede | Command emits the safe integrity diagnostic and leaves lifecycle files unchanged |
| Correction ledger changes after a successful mutation | Command renders the transaction's validated snapshot and does not report a false failure after persistence |
| Affected module has no `context.md`, or it holds only scaffold prompts | Surfacing is skipped for that module and change creation is unaffected |

## Dependencies

| Module | What is used |
|--------|-------------|
| change | Lifecycle operations and projections |
| types | Output format |

**Frontmatter Synchronization**

Implementation SHALL add `specs/cli_args/cli_args.spec.md` to `depends_on`. Rust source-module ownership maps
`crate::cli::ChangeAction` to the `cli_args` contract rather than the top-level `cli` executable contract.

## Change Log

| 2026-07-30 | Scoped `change check`; add `change audit` two-verb UX |

| Date | Change |
|------|--------|
| 2026-08-30 | `change check` is spec↔code sync; it does not spawn `verification_commands`. |
| 2026-08-30 | `ship` / archived `next_action` no longer gate merge on writing lessons. |
| 2026-08-01 | Draft next-action prefers complete artifacts over approve. |
| 2026-07-10 | Initial 5.0 change command |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-13 | Add text and deterministic JSON dispatch for audited stale-accepted reopen |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
| 2026-07-30 | CHG-0068: Render explicit pass/block scoped-review results while keeping independence policy in the change domain |
| 2026-07-30 | CHG-0068: Render stable reviewer claims while preserving append-only attempts and externally authenticated check provenance |
| 2026-07-31 | CHG-0069-scoped-change-check-change-audit-and-agent-pack-for-the-two-verb-lifecycle: Scoped change check, change audit, and agent pack for the two-verb lifecycle |
| 2026-08-01 | CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa: Approve rejects living ADDED REQs and draft next_action waits on complete artifacts |
| 2026-08-07 | CHG-0089-add-change-check-commit-to-perform-the-sequence-it-requires: Add change check --commit to perform the sequence it requires |
| 2026-08-07 | CHG-0091-add-change-ship-status-for-local-ship-readiness-and-merge-before-finalize-warning: Add change ship-status for local ship readiness and merge-before-finalize warning |
| 2026-08-07 | CHG-0092-complete-buttery-ship-status-tip-class-and-ship-preflight-for-agents: Complete buttery ship status tip class and ship preflight for agents |
| 2026-08-07 | CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path: Encode multi-active ship ordering warnings and Agents.md happy path |
| 2026-08-07 | CHG-0093-encode-ship-multi-active-ordering-rules-and-agents-happy-path: Encode ship multi-active ordering rules and AGENTS happy path |
| 2026-08-08 | CHG-0099-ship-status-live-github-check-run-trust-for-product-parent-sha: Ship-status live GitHub check-run trust for product parent SHA |
| 2026-08-08 | CHG-0100-ship-push-wait-archive-tip-orchestration-for-buttery-multi-tip-ship: Ship --push --wait archive tip orchestration for buttery multi-tip ship |
| 2026-08-08 | CHG-0100-fail-closed-in-text-lifecycle-views-when-a-correction-ledger-is-invalid: Fail closed in text lifecycle views when a correction ledger is invalid |
| 2026-08-10 | CHG-0103: Validate correction-ledger integrity before existing-change mutations and increment the command contract version |
| 2026-08-10 | CHG-0103-address-pr-531-review-by-validating-correction-ledger-health-before-mutating-lif: Address PR 531 review by validating correction-ledger health before mutating lifecycle commands and incrementing the cmd_change contract version |
| 2026-08-10 | CHG-0103: Delegate correction-ledger validation to locked change-domain mutations and render successful mutations from their validated transaction snapshots |
| 2026-08-10 | CHG-0103: Select the normal/strict mutation summary captured under the domain transaction lock |
| 2026-08-17 | CHG-0136-an-unreadable-change-workspace-must-be-reported-not-counted-as-absent: An unreadable change workspace must be reported, not counted as absent |
| 2026-08-17 | CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards: A stale sequence ledger must not be committed backwards |
| 2026-08-18 | CHG-0145-the-sequence-ledger-floor-must-be-wired-not-merely-present: The sequence ledger floor must be wired, not merely present |
| 2026-08-19 | CHG-0153-ship-status-must-name-the-action-the-lifecycle-state-requires-and-resolve-an-ar: Ship-status must name the action the lifecycle state requires, and resolve an archived change's evidence |
| 2026-08-23 | ship-readiness-is-a-content-question-not-a-history-one: Ship readiness is a content question, not a history one |
| 2026-08-24 | close-the-lessons-loop-surface-what-a-module-already-learned-at-proposal-name-where-a-lesson-goes-when-a-build-fails: Close the lessons loop: surface what a module already learned at proposal, name where a lesson goes when a build fails, and assemble the archived bundle at finalize |
| 2026-08-24 | ship-must-name-the-lesson-fold-back-too-the-archive-bundle-is-written-and-only-finalize-says-so: Ship must name the lesson fold-back too: the archive bundle is written and only finalize says so |
| 2026-08-29 | ship-status-readiness-must-ask-whether-the-scoped-review-is-current-and-say-so-when-it-cannot-tell: Ship-status readiness must ask whether the scoped review is current, and say so when it cannot tell |
| 2026-08-30 | make-check-the-product-and-stop-change-check-from-spawning-project-tests: Make check the product and stop change check from spawning project tests |
