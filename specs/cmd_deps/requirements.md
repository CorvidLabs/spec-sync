---
spec: cmd_deps.spec.md
---

## User Stories

- As a developer, I want cross-module `depends_on` declarations validated so missing dependency specs, cycles, and undeclared imports are caught
- As an architect, I want to render the dependency graph as Mermaid or Graphviz DOT so I can visualize module relationships
- As a CI operator, I want `deps` to exit non-zero when there are dependency errors so broken graphs fail the build
- As a CI operator, I want `deps --strict` to also fail on undeclared-import warnings so drift is gated, not just cycles/missing specs
- As a tooling author, I want `deps --format json` to stay fully machine-readable (no human diagnostics, even on stderr) so I can parse it reliably

## Acceptance Criteria

- `cmd_deps(root, strict, format, mermaid, dot)` loads config and, when `--mermaid` or `--dot` is set, builds the graph and prints the corresponding diagram, then returns (no validation, no exit code change)
- In diagram mode, if there are no `depends_on` edges but the graph is non-empty, a hint about adding `depends_on:` is printed to stderr
- Mermaid renders missing targets as dashed `❌` nodes; DOT renders them as dashed red nodes; both sort modules/deps for deterministic output
- Without a diagram flag, validation runs via `deps::validate_deps`, producing module/edge counts, errors, warnings, cycles, missing deps, and undeclared imports
- Output is rendered per `OutputFormat`: Json (full report object), Markdown/Github (headed sections), and Text/Table/Csv (decorated lines plus a topological build order when there are no cycles)
- Exits 1 if `report.errors` is non-empty (after printing the report in any format)
- Under `--strict`, exits 1 when `report.warnings` is non-empty even if there are no errors; a "warnings treated as errors" note is printed to stderr for non-JSON formats and suppressed for JSON (the exit code and the `warnings` array are the JSON contract)

## Constraints

- Graph building, cycle detection, import analysis, and topological sort all live in the `deps` module; this wrapper formats and decides the exit code
- `render_mermaid` and `render_dot` are private helpers in this module
- The empty-graph hint only fires in diagram mode, not in the validation path

## Out of Scope

- The dependency analysis algorithms (owned by the `deps` module)
- Writing diagrams/reports to a file (output goes to stdout)
- Interactive prompts or GUI

### REQ-cmd-deps-001

The `cmd_deps` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.


### REQ-cmd-deps-002

`deps` SHALL NOT print a coverage percentage that was not measured.

Acceptance Criteria
- A zero-source tree renders the unmeasured state.
