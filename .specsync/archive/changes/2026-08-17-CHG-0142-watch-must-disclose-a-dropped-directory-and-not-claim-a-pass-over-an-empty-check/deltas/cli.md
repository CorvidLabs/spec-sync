## ADDED

### REQUIREMENT REQ-cli-009

The `watch` command SHALL receive the selected output format, so its diagnostics can honour the format the caller asked for.

Acceptance Criteria
- The dispatcher passes the resolved output format to the watch entry point rather than dropping it.
- Watch's own diagnostics render as JSON under `--format json` and as human text otherwise.
- No other command's dispatch changes.
