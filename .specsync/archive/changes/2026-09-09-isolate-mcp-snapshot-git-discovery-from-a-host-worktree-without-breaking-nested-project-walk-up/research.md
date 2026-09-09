---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: research
---

# Research

Git 2.39.5 `GIT_CEILING_DIRECTORIES` (colon-separated absolute paths; git will not chdir up past a listed directory):

| cwd | ceiling | `rev-parse --is-inside-work-tree` |
|---|---|---|
| `repo/packages/foo` | unset | true |
| `repo/packages/foo` | `parent(repo)` | true |
| `repo/packages/foo` | `repo` (this was overnight `parent(project_root)`) | fatal |
| `repo/tmp/specsync-mcp-abc` | unset | true (host leak) |
| `repo/tmp/specsync-mcp-abc` | `repo/tmp` = parent(snapshot) | fatal (isolated) |
| `repo/tmp/specsync-mcp-abc` | the snapshot path itself | true (still walks up) |

Default `git_cmd` cannot pin `parent(root)` without breaking nested projects. MCP snapshots can pin `parent(snapshot)` without affecting CLI callers.
