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

The system SHALL initialize local lifecycle coordination files as ignored, recoverable implementation details.

Acceptance Criteria
- New projects ignore the lifecycle lock and transaction journal.
- Initialization remains idempotent and does not weaken SDD enforcement.
