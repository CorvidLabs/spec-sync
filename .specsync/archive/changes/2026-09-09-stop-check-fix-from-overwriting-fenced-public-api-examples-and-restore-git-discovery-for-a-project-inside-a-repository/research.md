---
change: stop-check-fix-from-overwriting-fenced-public-api-examples-and-restore-git-discovery-for-a-project-inside-a-repository
artifact: research
---

# Research

- `blank_fenced_code` is space-padded to the original byte length so offsets in the blanked copy index the original. The insertion-point scanner at `auto_fix` already uses that correctly (`api_section_for_headers` for `find_iter`, `api_section` for the header line). `fix_near_miss_headers` cloned the blanked copy as `new_section` and wrote it back — the only writer that treated blanking as a replacement buffer.
- Git 2.39.5: `GIT_CEILING_DIRECTORIES=parent(root)` → `git -C repo/sub rev-parse --is-inside-work-tree` fails. Ceiling set to `root` itself still walks up (git looks in the current directory even when it is the ceiling, then chdirs into parents that are not listed). Ceiling of parent is what blocks nested discovery. Nested-project and "snapshot sitting inside a host worktree" are the same git topology; they cannot both be satisfied by a parent ceiling. `is_git_repo` is specified as "inside a git work tree", so walk-up wins. Env sanitization (`env_clear`) is the MCP token-leak fix and is independent of ceiling.
