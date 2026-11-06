# Retire the init contract clause that promises strict SDD defaults to Git repositories

## MODIFIED

### REQUIREMENT REQ-cmd-init-004

Initialization SHALL write the same SDD-off enforcement values in a Git repository as in any other directory, so no changed-path gate applies until the project adopts SDD.

Acceptance Criteria
- A Git repository receives `enabled: false` and `require_change_for_meaningful_files: false`, the same values a non-Git directory receives; `specsync change adopt` is the on-switch.
- Non-Git directories initialize successfully without an immediately impossible changed-path gate.
- The strict `true` values of `SddPolicy::default()` only fill fields missing from an existing `sdd.json`, so an older file never reads as disabled; fresh initialization never writes them.
