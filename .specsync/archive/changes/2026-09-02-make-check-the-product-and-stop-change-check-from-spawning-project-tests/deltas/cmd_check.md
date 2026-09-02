## MODIFIED

### SPEC SECTION Purpose

Implements the primary deterministic validation entry point, including one fallible schema
snapshot, visible ignore suppression, caching, local Markdown auto-fix, structured formats, and
optional drift issues. SDD / change / archive history is not part of this command.

### SPEC SECTION Invariants

9. Coverage uses checked manifest discovery. Malformed Gradle settings make the result
   inconclusive and exit 1 instead of producing partial or vacuous coverage WHEN `source_dirs` was
   not configured — the source list would otherwise be the output of the discovery that failed.
   When `source_dirs` IS configured, the same failure does not abort the command: coverage runs
   over the stated list and the JSON payload carries the degradation in `manifest_notices`
   alongside `skipped_links`, so a machine consumer acting on `passed` can see it.
10. Text, JSON, Markdown, and GitHub output distinguish emitted warnings from deterministic
    suppressed-warning details, while strict exit behavior counts only unsuppressed findings.
11. A warm hash-cache skip skips re-validation, never the previous findings: unchanged specs
    replay their stored snapshot, `specs_checked` counts them, and a hash-only cache with no
    snapshot is re-validated rather than reported clean.
12. `check` does not consult SDD policy, active change workspaces, or archive history. Those
    surfaces belong to `specsync change`.

### REQUIREMENT REQ-cmd-check-004

The primary check command SHALL be the bidirectional spec-to-code drift check
and SHALL NOT consult SDD policy, active change workspaces, or archive history.

Acceptance Criteria

- `specsync check` does not print an active-change count and does not emit
  SDD workspace or archive findings.
- Exit status derives solely from spec validation results, the effective enforcement mode,
  `--strict`, and `--require-coverage`.
- Lifecycle gating remains reachable through the `change` verbs and `specsync change audit`,
  whose behavior is unchanged. A project that never enables SDD can use `check` as the
  whole product.
