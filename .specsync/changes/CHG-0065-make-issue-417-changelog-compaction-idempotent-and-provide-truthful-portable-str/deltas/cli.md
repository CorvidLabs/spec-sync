## ADDED

### REQUIREMENT REQ-cli-007

The root CLI SHALL forward the resolved global output format to compact and archive-tasks handlers.

Acceptance Criteria

- `--json` and `--format json` dispatch the same `OutputFormat::Json` value.
- `--format markdown` dispatches `OutputFormat::Markdown`.
- No human banner is emitted before the structured renderer.

## MODIFIED

### SPEC SECTION Invariants

11. `compact` and `archive-tasks` receive the resolved global output format instead of silently
    falling back to text.
