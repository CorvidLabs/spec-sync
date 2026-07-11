## ADDED

### REQUIREMENT REQ-cmd-deps-001

The dependency command SHALL validate missing dependencies, cycles, and undeclared imports and SHALL provide deterministic graph formats.

Acceptance Criteria
- `cmd_deps(root, strict, format, mermaid, dot)` loads config and, when `--mermaid` or `--dot` is set, builds the graph and prints the corresponding diagram, then returns (no validation, no exit code change)
- In diagram mode, if there are no `depends_on` edges but the graph is non-empty, a hint about adding `depends_on:` is printed to stderr
- Mermaid renders missing targets as dashed `❌` nodes; DOT renders them as dashed red nodes; both sort modules/deps for deterministic output
- Without a diagram flag, validation runs via `deps::validate_deps`, producing module/edge counts, errors, warnings, cycles, missing deps, and undeclared imports
- Output is rendered per `OutputFormat`: Json (full report object), Markdown/Github (headed sections), and Text/Table/Csv (decorated lines plus a topological build order when there are no cycles)
- Exits 1 if `report.errors` is non-empty (after printing the report in any format)
- Under `--strict`, exits 1 when `report.warnings` is non-empty even if there are no errors; a "warnings treated as errors" note is printed to stderr for non-JSON formats and suppressed for JSON (the exit code and the `warnings` array are the JSON contract)
