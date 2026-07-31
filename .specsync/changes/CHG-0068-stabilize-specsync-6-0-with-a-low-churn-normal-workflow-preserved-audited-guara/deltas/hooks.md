## ADDED

### REQUIREMENT REQ-hooks-002

Hook installation SHALL resolve the repository's effective hook directory and manage only
project-keyed SpecSync blocks.

Acceptance Criteria

- Installation honors normal repositories, worktrees, submodules, and `core.hooksPath`.
- Managed blocks carry a stable project key so multiple projects sharing a hook path do not collide.
- Install is idempotent and generated pre-commit checks are strict and blocking.
- Uninstall removes only the matching SpecSync block and preserves user and other-project content.
- Symlink escapes, ambiguous hook roots, and unsafe paths fail before mutation.
