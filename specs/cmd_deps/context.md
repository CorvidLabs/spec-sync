---
spec: cmd_deps.spec.md
---

## Key Decisions

- Two modes: diagram mode (`--mermaid`/`--dot`) builds the graph, prints the diagram, and returns early without validation or an exit-code change; validation mode runs `deps::validate_deps` and renders per `OutputFormat`.
- `render_mermaid` and `render_dot` are private helpers kept in this wrapper (not the `deps` module) because they are presentation-only; both sort modules and deps for deterministic output and mark missing targets specially (dashed `❌` / dashed red).
- The text path additionally prints a topological build order via `deps::topological_sort` when there are no cycles and the graph is non-empty.
- Exit code: the command exits 1 whenever `report.errors` is non-empty, after the report has been printed in whatever format was requested. Under `--strict` it also exits 1 when `report.warnings` is non-empty (undeclared imports), so drift is gated in CI (#304).
- The `--strict` "warnings treated as errors" note is a human diagnostic. It goes to stderr for non-JSON formats and is suppressed entirely in JSON mode — a JSON consumer relies only on the machine-readable body (the `warnings` array) plus the exit code, so no ANSI/human text leaks into its pipeline even via stderr.

## Files to Read First

- `src/commands/deps.rs` — the command wrapper (this module), including `render_mermaid`/`render_dot`
- `src/deps.rs` — `build_dep_graph`, `validate_deps`, `topological_sort`, `DepNode`, and `DepsReport { module_count, edge_count, errors, warnings, cycles, missing_deps, undeclared_imports }`
- `src/types.rs` — `OutputFormat`

## Current Status

Implemented and stable. The `deps` delegate is heavily unit-tested; `tests/integration.rs::dependency_spec_not_found_errors` exercises the missing-dependency path end-to-end. The diagram renderers in this wrapper have no direct test.

## Notes

- The empty-graph hint ("No depends_on relationships found…") is stderr-only and diagram-mode-only.
