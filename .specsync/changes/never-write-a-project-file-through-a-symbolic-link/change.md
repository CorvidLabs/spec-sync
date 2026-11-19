---
id: never-write-a-project-file-through-a-symbolic-link
state: implementing
type: bug_fix
base_commit: 7b6e4779ba5b7d0fedd621c04a917fcb02e28c25
---

# Never write a project file through a symbolic link

## Intent

Never write a project file through a symbolic link

## Affected Canonical Specs

- `util`
- `hooks`
- `agents`
- `mcp`
- `registry`
- `cmd_resolve`
- `cmd_migrate`
- `generator`
- `cmd_scaffold`
- `cmd_new`
- `cmd_wizard`
- `cmd_import`

## Acceptance Criteria

- For each of hooks install and uninstall, agents install and uninstall, mcp lock --write, the scaffold registry append, migrate, new, scaffold, add-spec, wizard and import: with the file it writes, or a directory on the way to it, a symbolic link that leaves the project, a dangling link, or a non-regular file, the command refuses with an error naming the path, the link's target is unchanged and nothing is created outside the project; with a link to a regular file inside the project (CLAUDE.md to AGENTS.md) it writes through the link; the resolve --remote cache is never read or written through any link and says so once on stderr; each new test fails on 7b6e4779 and passes with the fix

## No-spec Rationale

Not applicable
