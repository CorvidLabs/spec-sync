## MODIFIED

### REQUIREMENT REQ-hooks-002

Hook installation SHALL resolve the repository's effective hook directory and manage only
project-keyed SpecSync blocks.

Acceptance Criteria

- Installation honors normal repositories, worktrees, submodules, and `core.hooksPath`.
- Managed blocks carry a stable project key so multiple projects sharing a hook path do not collide.
- Install is idempotent and the generated pre-commit hook runs `specsync check` (honoring `.specsync/config.toml` `enforcement`; default is strict on errors, warnings pass unless `--strict` is given) rather than hardcoding `--strict`.
- Uninstall removes only the matching SpecSync block and preserves user and other-project content.
- Symlink escapes, ambiguous hook roots, and unsafe paths fail before mutation.

## ADDED

### REQUIREMENT REQ-hooks-003

A first-run `init` → source file → `add-spec` → `check` (errors 0, warnings present) → `hooks install` → `git commit` sequence SHALL succeed without `--no-verify`, because the generated hook does not treat warnings as errors.

Acceptance Criteria
- The generated hook body contains `specsync check` or `specsync --root … check` and does not contain `check --strict`.
- The hook comment names `.specsync/config.toml`, not `specsync.json`, and does not call `warn` the default.
- `init_add_spec_hooks_install_then_commit_succeeds_without_strict` ends in a successful commit.
