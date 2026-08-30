## MODIFIED

### SPEC SECTION Purpose

Implements `specsync init`. Creates the 5.0 `.specsync/` layout with detected source directories, canonical TOML configuration, SDD policy written **disabled**, version stamp, local-state ignore rules, and lifecycle/change/archive directories. Does not start an SDD interview. Enable the change workflow later with `specsync change adopt`.

### SPEC SECTION Invariants

1. Auto-detects source directories via `config::detect_source_dirs()`.
2. Never overwrites an existing current or legacy configuration; legacy configurations receive a migration hint.
3. Writes the 5.0 policy, version, and layout deterministically without blocking in non-interactive environments. The written policy has `enabled: false` and `require_change_for_meaningful_files: false`.
4. Local hash cache, lifecycle lock, and transaction journal files are ignored and never treated as portable project state.
5. Re-running initialization is idempotent.

### REQUIREMENT REQ-cmd-init-005

Initialization SHALL leave a repository that passes the very next lifecycle check.

Acceptance Criteria
- The protected SDD paths initialization creates are recorded in `.specsync/bootstrap.json`.
- The first check after initialization reports no uncovered meaningful delivery for files
  initialization itself wrote.
- Failure to write the record is reported as a warning and does not fail initialization.
- Fresh `init` writes SDD off (`enabled: false`, `require_change_for_meaningful_files: false`)
  so the first `specsync check` is drift-only.
