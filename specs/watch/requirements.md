---
spec: watch.spec.md
---

## User Stories

- As a developer editing specs, I want live validation feedback on file save so that I can see errors immediately without manually re-running commands
- As a developer, I want the screen cleared and the changed file shown before each re-run so that I can quickly identify what triggered the re-validation
- As a developer making rapid edits, I want changes debounced so that spec-sync doesn't run dozens of times while I'm typing

## Acceptance Criteria

- Debounce interval of 500ms prevents rapid-fire re-runs
- Extra 300ms minimum between consecutive check runs
- Only reacts to Create, Modify, and Remove events (not metadata-only changes)
- Queued events are drained after each check completes
- Exits immediately if there are no directories to watch
- Check runs as a child process to isolate exit calls from the watcher
- Screen is cleared before each re-run
- Changed file path is displayed in the separator header
- Watches both spec and source directories

## Constraints

- Must use the `notify` crate for cross-platform file watching
- Watcher runs in the foreground — no daemon mode
- Must handle watcher errors gracefully (log and continue, don't crash)

## Out of Scope

- Watching for config file changes (requires restart)
- Triggering actions other than `specsync check` (no custom commands)
- Network-based file watching (only local filesystem)
- Integration with editor save events or LSP
