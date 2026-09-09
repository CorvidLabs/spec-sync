---
change: isolate-mcp-snapshot-git-discovery-from-a-host-worktree-without-breaking-nested-project-walk-up
artifact: design
---

# Design

- `git_cmd` stays ceiling-free by default. `env_clear` still drops host `GIT_CEILING_DIRECTORIES`, `GITHUB_TOKEN`, `GIT_DIR`. Nested `is_git_repo(repo/sub)` remains true.
- New `with_discovery_ceiling(ceiling, f)` is a crate-visible, panic-safe thread-local. Git children spawned inside `f` get `GIT_CEILING_DIRECTORIES` set to that absolute path after the clear; the previous slot is restored on return or unwind. The ceiling is not on the inherit allowlist.
- Empirically the isolating value is `parent(snapshot)`: ceiling equal to the snapshot path itself still walks up; ceiling equal to the snapshot's parent stops before the host `.git`.
- MCP tool and resource dispatch wrap every snapshot git probe with `with_discovery_ceiling(parent(snapshot.root()))`. CLI callers never enter that wrap.
- `cmd_new` companions (`requirements.md`, `context.md`, `tasks.md`) drop the obsolete `chrono_lite_today` / `get_exported_symbols` invariants so they match the already-landed spec body. No `src/commands/new.rs` change.
