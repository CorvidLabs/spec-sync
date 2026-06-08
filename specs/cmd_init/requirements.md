---
spec: cmd_init.spec.md
---

## User Stories

- As a developer adopting spec-sync, I want `specsync init` to create a ready-to-use `specsync.json` with my source directories already detected so that I can start validating specs immediately.
- As a developer, I want init to be safe to re-run so that it never clobbers an existing config.
- As a developer, I want the local-only hash cache to be gitignored automatically so that I don't accidentally commit `.specsync/hashes.json`.

## Acceptance Criteria

- `cmd_init` writes `specsync.json` with `specsDir: "specs"`, auto-detected `sourceDirs` (via `config::detect_source_dirs`), a standard `requiredSections` list, and default `excludeDirs`/`excludePatterns`.
- If either `specsync.json` or `.specsync.toml` already exists, init prints a message and returns without writing.
- On success, init prints "Created specsync.json" and the detected source directories.
- After writing the config, `ensure_hashes_gitignored` adds `.specsync/hashes.json` to the root `.gitignore` (with a comment header) unless it is already present; the result is reported as a success line, a no-op, or a warning.
- `ensure_hashes_gitignored` is idempotent: re-running never duplicates the entry.

## Constraints

- `ensure_hashes_gitignored(root) -> Result<bool, String>`: `Ok(true)` when the entry was added, `Ok(false)` when already present, `Err(String)` on write failure (the io error is mapped to a "Failed to update .gitignore: …" message).
- A failure to write `.gitignore` is non-fatal — it is printed as a `warning:` and init still succeeds; a failure to write `specsync.json` is fatal (exit 1).
- Must not panic on expected error conditions.

## Out of Scope

- Editing a `.specsync/.gitignore` file (only the repository-root `.gitignore` is touched).
- Creating the `specs/` directory or any spec files.
- Interactive prompts, GUI, or web output.
