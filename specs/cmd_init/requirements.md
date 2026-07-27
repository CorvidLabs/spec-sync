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
- Empty projects report the `src` value as a fallback rather than a detected directory.
- `init --repair` restores missing support artifacts without rewriting config, specs, or existing ignore content.
- An initialized ancestor prevents nested metadata creation.
- Structured output reports created/repaired/unchanged state and failures truthfully.

## Constraints

- `ensure_hashes_gitignored(root) -> Result<bool, String>`: `Ok(true)` when the entry was added, `Ok(false)` when already present, `Err(String)` on write failure (the io error is mapped to a "Failed to update .gitignore: …" message).
- A failure to write `.gitignore` is non-fatal; failure to create config, policy, or versioned layout is fatal.
- Must not panic on expected error conditions.

## Out of Scope

- Editing a `.specsync/.gitignore` file (only the repository-root `.gitignore` is touched).
- Creating the `specs/` directory or any spec files.
- GUI or web output.

### REQ-cmd-init-001

The system SHALL initialize local lifecycle coordination files as ignored, recoverable implementation details.

Acceptance Criteria
- New projects ignore the lifecycle lock and transaction journal.
- Initialization remains idempotent and does not weaken SDD enforcement.

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

Initialization SHALL be truthful, additive, and non-destructive across creation, inspection, and repair.

Acceptance Criteria

- Empty or source-free directories use `source_dirs = ["src"]` while reporting `source_dirs_detected = false`.
- Plain re-init is byte-identical and reports an unchanged outcome.
- `--repair` validates the selected config before writes and restores only missing support files/directories.
- Existing config, specs, and `.gitignore` content are never clobbered.
- Running below an initialized ancestor fails without creating nested metadata.
- Predictable blocking file/symlink topology fails before partial layout creation.
- JSON/Markdown/GitHub/table/CSV outputs reflect success, creation, repair, unchanged state, and errors.
