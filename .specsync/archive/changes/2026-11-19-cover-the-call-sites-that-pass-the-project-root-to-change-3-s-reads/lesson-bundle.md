# Lesson bundle: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Cover the call sites that pass the project root to change 3's reads
- **Kind**: BugFix
- **Specs**: cmd_init_registry, cmd_issues, cmd_score, cmd_view, mcp
- **Paths**: src/commands/init_registry.rs, src/commands/issues.rs, src/commands/score.rs, src/commands/view.rs, src/mcp/confinement.rs, src/mcp/tools.rs
- **Acceptance**: specsync change audit reports no meaningful changed path that no change covers, with the six call-site files declared here; the files are unchanged by this change; the full test suite still passes

## Evidence

- Verification commit: `ac7effc65541e706d01147b172cc10da5c386f73`
- Base commit: `6d524c6c2f5c4579b81bd4acca85651834be45da`
- Verified by: `specsync check --spec cmd_init_registry --spec cmd_issues --spec cmd_score --spec cmd_view --spec mcp`

## From the change's context.md

# Context

After `never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound` was archived, `specsync change audit` reported six meaningful changed paths that no change covers. Change 3's commit had updated their call sites, but its declared paths did not list them. Each is a one-line edit that passes the project root to a function change 3 gave a root parameter. Finalize did not catch it; the archive-first CI audit would.

Human decision, relayed by the planning session (2026-11-19, about 15:50Z): "yes", covering both this change's scope approval and its implementation review. The code was committed and reviewed with change 3.

## From the change's design.md

# Design

No new code. This change declares the six files so the audit finds a change covering them:

| File | The edit change 3 made |
|---|---|
| `src/commands/init_registry.rs` | `validate_config_file(root, …)` |
| `src/commands/issues.rs` | `schema::schema_read_errors(root, …)` |
| `src/commands/score.rs` | `filter_by_status(root, …)` |
| `src/commands/view.rs` | `view::view_spec(root, …)` |
| `src/mcp/confinement.rs` | `crate::config::read_config_file(root, …)` |
| `src/mcp/tools.rs` | `crate::schema::build_schema(root, …)` |

No canonical spec changes: each call keeps its meaning, and change 3's specs already describe the functions.

## From the change's testing.md

# Testing

- `specsync change audit` passes once this change covers the six files; before it, it named them.
- The full verify lane and the Linux container run on change 3's tip already cover these lines, and this change adds no code.

## Where these lessons go

- `specs/cmd_init_registry/cmd_init_registry.3md`
- `specs/cmd_issues/cmd_issues.3md`
- `specs/cmd_score/cmd_score.3md`
- `specs/cmd_view/cmd_view.3md`
- `specs/mcp/mcp.3md`
