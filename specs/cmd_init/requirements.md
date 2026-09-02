---
spec: cmd_init.spec.md
---

## User Stories

- As a developer adopting spec-sync, I want `specsync init` to create the 5.0 project layout, enable verified SDD, and detect source/test commands so I can start with one guided workflow.
- As a developer, I want init to be safe to re-run so that it never clobbers an existing config.
- As a developer, I want the local-only hash cache to be gitignored automatically so that I don't accidentally commit `.specsync/hashes.json`.

## Acceptance Criteria

- `cmd_init` writes `.specsync/config.toml`, a `5.0.0` version stamp, `.specsync/sdd.json`, lifecycle/change/archive directories, and auto-detected source directories.
- On a terminal, init offers native agent installation and creation of the first verified change; non-interactive/CI initialization never blocks for input.
- If either `specsync.json` or `.specsync.toml` already exists, init prints a message and returns without writing.
- On success, init prints the created v5 layout and detected source directories.
- After writing the config, `ensure_hashes_gitignored` adds `.specsync/hashes.json` to the root `.gitignore` (with a comment header) unless it is already present; the result is reported as a success line, a no-op, or a warning.
- `ensure_hashes_gitignored` is idempotent: re-running never duplicates the entry.

## Constraints

- `ensure_hashes_gitignored(root) -> Result<bool, String>`: `Ok(true)` when the entry was added, `Ok(false)` when already present, `Err(String)` on write failure (the io error is mapped to a "Failed to update .gitignore: …" message).
- A failure to write `.gitignore` is non-fatal; failure to create config, policy, or versioned layout is fatal.
- Must not panic on expected error conditions.

## Out of Scope

- Editing a `.specsync/.gitignore` file (only the repository-root `.gitignore` is touched).
- Creating the `specs/` directory or any spec files.
- GUI or web output.

### REQ-cmd-init-001

The `cmd_init` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-init-002

The system SHALL describe and create the same current versioned project layout.

Acceptance Criteria
- Canonical initialization documentation identifies the 5.0 layout and TOML configuration.
- Tests and examples do not describe the removed root JSON initialization path as current behavior.

### REQ-cmd-init-003

Fresh initialization SHALL make detected project source directories and committed SDD policy files meaningful by default.

Acceptance Criteria
- Detected source directories are merged into the generated policy.
- Policy/configuration paths cannot disable or weaken SDD coverage without lifecycle coverage.

### REQ-cmd-init-004

Initialization SHALL enable Git-dependent SDD coverage only when the project can provide Git comparison evidence.

Acceptance Criteria
- Git repositories receive normal strict SDD defaults.
- Non-Git directories initialize successfully without an immediately impossible changed-path gate.

### REQ-cmd-init-005

Initialization SHALL leave a repository that passes the very next lifecycle check.

Acceptance Criteria
- The protected SDD paths initialization creates are recorded in `.specsync/bootstrap.json`.
- The first check after initialization reports no uncovered meaningful delivery for files
  initialization itself wrote.
- Failure to write the record is reported as a warning and does not fail initialization.
- Fresh `init` writes SDD off (`enabled: false`, `require_change_for_meaningful_files: false`)
  so the first `specsync check` is drift-only.

### REQ-cmd-init-006

CSV field quoting SHALL have a single implementation.

Acceptance Criteria
- No command carries its own copy of the quoting rule.
