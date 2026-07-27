## ADDED

### REQUIREMENT REQ-cli-007

The root CLI SHALL forward the resolved global output format to compact and archive-tasks handlers.

Acceptance Criteria

- `--json` and `--format json` dispatch the same `OutputFormat::Json` value.
- `--format markdown` dispatches `OutputFormat::Markdown`.
- No human banner is emitted before the structured renderer.

## MODIFIED

### SPEC SECTION Invariants

1. When no subcommand is given, `check` runs by default.
2. `--root` defaults to the current working directory and is validated as an existing directory,
   otherwise the process exits 2. MCP plus check/coverage/generate/score/report/comment preserve
   the requested spelling so their retained-capability engines can detect public root replacement;
   other commands receive the canonicalized path.
3. `--strict` causes warnings to produce a non-zero exit code.
4. `--require-coverage N` causes exit 1 if file coverage percent is below N.
5. `--json` switches all output to machine-readable JSON without ANSI colors.
6. Initialization and registry creation remain idempotent and preserve existing configuration.
7. Generation, scoring, resolution, hooks, lifecycle, and reporting commands delegate policy to
   their focused modules.
8. Inherited verification context rejects recursive check/change/lifecycle dispatch before handler
   execution.
9. MCP dispatch forwards the parsed write capability unchanged; read-only remains the default.
10. MCP server-root initialization failures are printed to stderr and exit 2 before request input is
    processed.
11. `compact` and `archive-tasks` receive the resolved global output format instead of silently
    falling back to text.
