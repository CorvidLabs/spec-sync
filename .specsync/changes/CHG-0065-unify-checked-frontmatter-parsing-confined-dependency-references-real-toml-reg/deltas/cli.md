## ADDED

### REQUIREMENT REQ-cli-006

CLI dispatch SHALL carry dependency coverage, output format, remote depth, and strictness into the
selected command without accepting an inert flag.

Acceptance Criteria

- The global `--require-coverage` value reaches `deps` in text, structured, Mermaid, and DOT modes.
- The selected output format reaches `resolve`; `--json` cannot be parsed and then ignored.
- `--verify` continues to imply `--remote`, while local mode remains network-free.
- Help and public documentation state that runtime findings or inconclusive work exit 1 and CLI
  usage errors, including an out-of-range coverage percentage, exit 2.

## MODIFIED

### SPEC SECTION Invariants

1. When no subcommand is given, `check` runs by default.
2. `--root` defaults to the current working directory; it must resolve to an existing canonical
   directory or dispatch exits 2.
3. `--strict` promotes advisory warnings only where the selected command defines them; runtime
   findings remain failures without it.
4. The global `--require-coverage` value reaches `deps` and every other coverage-gated command;
   requested thresholds outside 0 through 100 are usage errors with exit 2.
5. The selected output format reaches `resolve` and every structured-capable command; JSON is one
   machine-readable stdout document without ANSI or human preamble.
6. `resolve --verify` implies `--remote`, and resolve performs no network access in local mode.
7. Trustworthy or advisory completion exits 0, findings and inconclusive requested work exit 1,
   and CLI grammar or usage errors exit 2.
8. Initialization and registry creation remain idempotent and preserve existing configuration.
9. Generation, scoring, resolution, dependency, hooks, lifecycle, and reporting commands delegate
   domain policy to their focused modules.
10. Inherited verification context rejects recursive check, change, or lifecycle dispatch before
    handler execution.
11. MCP dispatch forwards the parsed write capability unchanged; read-only remains the default.
12. MCP server-root initialization failures are printed to stderr and exit 2 before request input
    is processed.
