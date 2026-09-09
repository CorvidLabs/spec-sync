# Lesson bundle — isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Isolate MCP snapshot git discovery from a host worktree without breaking nested-project walk-up
- **Kind**: BugFix
- **Specs**: git_utils, mcp
- **Paths**: src/git_utils.rs, src/mcp.rs, site/src/content/docs/mcp-security.md, CHANGELOG.md
- **Acceptance**: MCP specsync_score on a snapshot whose tempfile sits inside a host git worktree reports git_freshness_available false, withholds git freshness once (no double penalty), and does not treat the host history as measurable; is_git_repo remains true for a nested project subdirectory when no snapshot isolation is in effect; git children still env_clear and drop GITHUB_TOKEN/GIT_DIR/GIT_CEILING_DIRECTORIES from the host environment.

## Evidence

- Verification commit: `9f64befece961b9412367b050e5bd5263ad3e10d`
- Base commit: `1f4f127f4741276a2e983b845be2d08a6a2641b4`
- Verified by: `specsync check --spec git_utils --spec mcp`

## From the change's context.md

# Context

Codex review of PR #771 re-raised REQ-mcp-008 after #770's follow-up dropped `GIT_CEILING_DIRECTORIES` from every `git_cmd`.

Empirically (Git 2.39.5): `GIT_CEILING_DIRECTORIES=parent(project_root)` makes `git -C repo/sub` fail, which is why the follow-up removed the global pin so nested CLI projects still discover the parent work tree. The same topology with no ceiling makes a tempfile sitting inside a host worktree (`TMPDIR` pointing at the repo) discover that host. `ProjectSnapshot` always copies without `.git` and sets `git_freshness_available: false`, but `score_spec` still probes git on the snapshot path. Walk-up then marks freshness `Measured` (untracked mtime fallback) and `score_spec_for_mcp` deducts another 5 points — double penalty — while claiming git history is unavailable.

REQ-mcp-008 still requires: an MCP snapshot sitting inside a host worktree cannot walk up into that worktree. Nested-project walk-up remains the git_utils default. Isolation is snapshot-scoped, not global.

Also in this package (Codex P2 on the same PR): `specs/cmd_new` companions still name `chrono_lite_today` / `get_exported_symbols` after the spec body was synchronized. Companion-only; `src/commands/new.rs` unchanged.

## From the change's design.md

# Design

- `git_cmd` stays ceiling-free by default. `env_clear` still drops host `GIT_CEILING_DIRECTORIES`, `GITHUB_TOKEN`, `GIT_DIR`. Nested `is_git_repo(repo/sub)` remains true.
- New `with_discovery_ceiling(ceiling, f)` is a crate-visible, panic-safe thread-local. Git children spawned inside `f` get `GIT_CEILING_DIRECTORIES` set to that absolute path after the clear; the previous slot is restored on return or unwind. The ceiling is not on the inherit allowlist.
- Empirically the isolating value is `parent(snapshot)`: ceiling equal to the snapshot path itself still walks up; ceiling equal to the snapshot's parent stops before the host `.git`.
- MCP tool and resource dispatch wrap every snapshot git probe with `with_discovery_ceiling(parent(snapshot.root()))`. CLI callers never enter that wrap.
- `cmd_new` companions (`requirements.md`, `context.md`, `tasks.md`) drop the obsolete `chrono_lite_today` / `get_exported_symbols` invariants so they match the already-landed spec body. No `src/commands/new.rs` change.

## From the change's testing.md

# Testing

| ID | Evidence |
|----|----------|
| REQ-git-utils-005 | `git_inherited_env_excludes_secrets_and_git_overrides` (deny list includes `GIT_CEILING_DIRECTORIES`), `is_git_repo_detects_project_inside_repository_subdirectory` |
| REQ-git-utils-006 | `discovery_ceiling_isolates_a_subdirectory_from_host_walk_up`, `discovery_ceiling_does_not_leak_after_the_closure` |
| REQ-mcp-008 | `mcp_snapshot_inside_host_worktree_does_not_discover_host_git_or_double_penalize` |

## Where these lessons go

- `specs/git_utils/context.md`
- `specs/mcp/context.md`
