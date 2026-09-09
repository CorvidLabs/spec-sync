---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: docs
---

# Docs

- `site/src/content/docs/mcp-security.md`: git children still env-sanitize; drop the `GIT_CEILING_DIRECTORIES` parent pin. Nested-project walk-up is required.
- CHANGELOG `[6.0.0]` Fixed: `--fix` no longer blanks fenced Public API examples while renaming headers; git discovery works for a project inside a repository subdirectory.
