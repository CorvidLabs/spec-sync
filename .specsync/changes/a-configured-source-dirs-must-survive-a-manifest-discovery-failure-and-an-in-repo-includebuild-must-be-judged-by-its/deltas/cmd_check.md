## MODIFIED

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
