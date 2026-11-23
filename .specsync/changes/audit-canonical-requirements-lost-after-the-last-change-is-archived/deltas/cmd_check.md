## MODIFIED

### REQUIREMENT REQ-cmd-check-004

The primary check command SHALL provide bidirectional spec-to-code drift validation. With explicit `--strict` on a Git project that has enabled SDD, it SHALL also run the shared branch comparison in REQ-change-archived-requirement-loss-001 and account for a loss using approved removals. It SHALL report the historical findings through its supported text and structured validation results. Lifecycle policy compliance, general workspace health and unrelated archive integrity SHALL remain with the lifecycle commands.

Acceptance Criteria

- Ordinary `specsync check` reports drift without emitting lifecycle workspace or archive findings.
- After the last change has been archived, `check --strict` exits unsuccessfully if the branch dropped a canonical requirement without an approved removal. It names the lost ID, the original canonical file, and how to display the original block.
- A separate managed fixture with no requirement loss passes. A managed project can use only strict check in CI and still catch an unexplained loss.
- A Git project that never enabled SDD retains the ordinary spec validation result, including when a requirement was edited without a removal delta. The same applies outside Git.

### SPEC SECTION Purpose

Implements deterministic validation of living specifications and code, including a fallible schema
snapshot, visible ignore suppression, caching, local Markdown auto-fix, structured formats, and
optional drift issues. For projects using the change lifecycle, explicit strict validation includes a
shared historical guard for unexplained canonical requirement loss.

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
12. Ordinary `check` validates living specs and code. With `--strict` on an SDD-enabled Git
    project, it also invokes the shared canonical requirement-loss comparison
    (REQ-change-archived-requirement-loss-001). Approved removal evidence is considered only to
    account for a lost ID. Lifecycle policy compliance, workspace health and general archive
    validation stay in `specsync change`.
13. When the hash cache location is refused, because no local state folder can be found or
    what is at `hashes.json` there is a link or not a regular file, `check` prints one
    `warning: hash cache not used: <reason>` on stderr before anything can end the run, neither
    reads nor saves the cache, and keeps the stdout output, verdict and exit code of a run with no
    cache (REQ-cmd-check-018).
14. Every read of a project file here, the spec files it reads again (8 MiB), goes through the bounded project read of `util`. A link that leaves the project, a dangling link, a file that is not regular and a file over its bound are refused and handled as that read's other read errors are, so nothing here waits on a FIFO or a device or reads past its bound (REQ-cmd-check-019).
15. `--fix --backup` reads each spec through the bounded project read and writes its backup to `.specsync/backup-fix/<spec path>` through the confined project write; a `backup-fix` directory or backup file that leads out of the repository is refused with the path, and `--fix` aborts before it changes a spec. `check` uses only the cache in the project's local state folder: a `.specsync/hashes.json` in the work tree, tracked or not and in any letter case, is never read, changes nothing and is not mentioned, and the "Cache:" line names the real path (REQ-cmd-check-020, REQ-cmd-check-021).
16. The schema folder's files enter the cache inputs only through `util::read_project_dir` (REQ-cmd-check-022).
