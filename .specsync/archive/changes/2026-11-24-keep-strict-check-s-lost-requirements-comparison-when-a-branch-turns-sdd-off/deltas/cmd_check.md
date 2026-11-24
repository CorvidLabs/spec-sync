## MODIFIED

### REQUIREMENT REQ-cmd-check-004

The primary check command SHALL provide bidirectional spec-to-code drift validation. With explicit `--strict` on a Git project that has SDD enabled in its working tree or at its trusted comparison base, it SHALL also run the shared branch comparison in REQ-change-archived-requirement-loss-001 and account for a loss using approved removals. It SHALL report the historical findings through its supported text and structured validation results. Lifecycle policy compliance, general workspace health and unrelated archive integrity SHALL remain with the lifecycle commands.

Acceptance Criteria

- Ordinary `specsync check` reports drift without emitting lifecycle workspace or archive findings.
- After the last change has been archived, `check --strict` exits unsuccessfully if the branch dropped a canonical requirement without an approved removal. It names the lost ID, the original canonical file, and how to display the original block.
- A separate managed fixture with no requirement loss passes. A managed project can use only strict check in CI and still catch an unexplained loss.
- A branch that sets `enabled` to false or deletes `.specsync/sdd.json` while its comparison base has SDD enabled is still compared. `check --strict` and `change audit --strict` both refuse an unexplained loss there.
- A Git project that never enabled SDD, or whose comparison base has it disabled, retains the ordinary spec validation result, including when a requirement was edited without a removal delta. The same applies outside Git.
