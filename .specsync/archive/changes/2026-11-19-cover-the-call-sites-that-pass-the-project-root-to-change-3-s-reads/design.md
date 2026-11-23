---
change: cover-the-call-sites-that-pass-the-project-root-to-change-3-s-reads
artifact: design
---

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
