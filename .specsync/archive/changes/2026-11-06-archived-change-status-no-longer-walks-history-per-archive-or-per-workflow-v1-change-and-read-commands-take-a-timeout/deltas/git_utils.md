## ADDED

### REQUIREMENT REQ-git-utils-011

A worker thread that only reads the repository on behalf of a command SHALL be able to open a repository-fact scope for the same root, seeded with the facts the command's open scopes hold, so it does not ask git again for a fact the command already knows. A seeded scope SHALL memoize the worker's own reads under the same fact rules, SHALL NOT start a `git cat-file --batch` reader (so a command keeps one batch child per root, REQ-git-utils-010), and SHALL NOT be offered while reads are live or a discovery ceiling is pinned.

#### Acceptance Criteria

- A seeded worker scope answers a fact the parent scope already holds without spawning git.
- A batched read inside a seeded scope returns `None`, so the worker reads per file.
- With live reads or a discovery ceiling on the parent thread, no seed is produced.
- `cold_archived_status_prefetch_workers_reuse_repository_facts`: four more archives add at most 150 Git processes to one cold archived status read (250 before seeding).
