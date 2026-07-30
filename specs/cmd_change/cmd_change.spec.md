---
module: cmd_change
version: 9
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

## Public API

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `cmd_change` | `root: &Path, action: ChangeAction, format: OutputFormat, strict: bool` | `()` | Dispatch every change lifecycle command, additive strict validators, and equivalent text/JSON output |

## Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` fails on any lifecycle error.
4. `change finalize` requires current verification and scoped-review evidence and performs no provider merge.

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

## Dependencies

| Module | What is used |
|--------|-------------|
| change | Lifecycle operations and projections |
| types | Output format |

**Frontmatter Synchronization**

Implementation SHALL add `specs/cli_args/cli_args.spec.md` to `depends_on`. Rust source-module ownership maps
`crate::cli::ChangeAction` to the `cli_args` contract rather than the top-level `cli` executable contract.

## Change Log

| Date | Change |
|------|--------|
| 2026-07-10 | Initial 5.0 change command |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-13 | Add text and deterministic JSON dispatch for audited stale-accepted reopen |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
| 2026-07-30 | CHG-0068-stabilize-specsync-6-0-with-a-low-churn-normal-workflow-preserved-audited-guara: Stabilize SpecSync 6.0 with one scope approval, same-PR finalization, lightweight archive CI, scoped review, and selected UX fixes |
