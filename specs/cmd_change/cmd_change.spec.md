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

Exposes the verified SDD lifecycle through equivalent human-readable and structured JSON commands under `specsync change`.

## Contract

1. Every operation delegates domain policy to the change module.
2. Errors render consistently and exit non-zero.
3. Status and interviews provide a concrete next action and a state-appropriate active-current or archived-history validity reason.
4. Supersede records an explicit digest-bound predecessor/path/module obligation before definition approval.
5. Reopen renders the exact persisted versioned supersession event in deterministic JSON.
6. Correct-owner renders one persisted exact canonical-owner correction and directs the user to definition reapproval.
7. Batch correct-owner resolves repeated paths, a manifest, or `--all-missing` into domain entries, renders the persisted record, and directs the user to definition reapproval without partial mutation on failure.
8. Listing preserves healthy changes when another workspace is corrupt; JSON marks degraded output invalid and every corrupt status exits non-zero.
9. New-change affected specs and optional-digest supersede inputs are validated before lifecycle mutation.

## Public API

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `cmd_change` | `root: &Path, action: ChangeAction, format: OutputFormat` | `()` | Dispatch every change lifecycle command and render text or JSON |

## Invariants

1. JSON output contains no terminal coloring.
2. Domain errors always produce exit code 1.
3. `change check` fails on any lifecycle error.
4. Healthy list JSON remains the stable array; degraded JSON is an object with `valid: false`, healthy `changes`, and path-aware `errors`.

## Behavioral Examples

### Scenario: Agent creates a change

- **Given** `specsync --json change new "Add passkeys"`
- **When** creation succeeds
- **Then** JSON includes the record, gate summary, and deterministic questions

### Scenario: Agent reopens stale accepted evidence

- **Given** current governed inputs no longer match an accepted change's closing evidence
- **When** `specsync --json change reopen <id> --actor <human> --reason <text>` succeeds
- **Then** JSON contains the verifying change and versioned audit record with the superseded approval and prior verification

### Scenario: Agent lists a partially corrupt project

- **Given** one healthy active change and one corrupt workspace
- **When** `specsync --json change list` runs
- **Then** it emits the healthy change with `valid: false` and corruption diagnostics, then exits 1

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
| `change new --spec` names a missing canonical spec | Command exits 1 before allocating a sequence or writing state |
| `change supersede` omits `--digest` | Command resolves the signed predecessor digest or reports a contextual evidence error |
| Reapproval occurs during ordinary verification | Command emits a warning before returning the change to implementation |
| Any listed or selected workspace is corrupt | Healthy records remain visible, corruption is explicit, and the command exits 1 |

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
| 2026-07-26 | v9: add degraded corruption reporting, optional supersede digest resolution, pre-mutation spec validation, and explicit verification-state reapproval warnings |
| 2026-07-10 | Initial 5.0 change command |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-07-13 | Add text and deterministic JSON dispatch for audited stale-accepted reopen |
| 2026-07-13 | CHG-0015-add-audited-stale-accepted-change-reopening: Add audited stale accepted change reopening |
| 2026-07-15 | CHG-0040-support-audited-append-only-correction-of-accepted-interview-metadata-without-re: Support audited append-only correction of accepted interview metadata without replaying canonical deltas |
| 2026-07-15 | CHG-0043-make-accepted-change-validity-successor-aware-with-exact-per-input-evidence-rec: Make accepted-change validity successor-aware with exact per-input evidence, recursive cycle-safe validation, fail-closed legacy compatibility, and safe archived successors |
| 2026-07-15 | CHG-0047-permit-audited-deterministic-ownership-corrections-for-reopened-already-applied: Permit audited deterministic ownership corrections for reopened already-applied changes |
| 2026-07-19 | CHG-0055-batch-mode-for-change-correct-owner-so-multiple-omitted-exact-canonical-owners-c: Batch mode for change correct-owner so multiple omitted exact canonical owners can be audited and appended in one transactional correction before a single reapprove-verify-accept cycle |
