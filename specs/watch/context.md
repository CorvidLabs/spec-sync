---
spec: watch.spec.md
---

## Key Decisions

- **500ms debounce + 300ms minimum gap**: The debounce prevents rapid re-runs from burst saves (e.g., IDE auto-save). The extra 300ms gap ensures the previous check finishes before starting the next one.
- **Subprocess isolation**: Each check runs as a forked child process rather than an in-process function call. This isolates any `exit()` calls in the check path from killing the watcher.
- **Event filtering**: Only Create, Modify, and Remove events trigger re-runs. Access and metadata changes are ignored to reduce noise.
- **Screen clear between runs**: The terminal is cleared before each re-run for a clean output, with the changed file path displayed in a separator header so the user knows what triggered it.
- **Exit on no directories**: If neither spec nor source directories exist, the watcher exits immediately with an error rather than watching nothing silently.

## Files to Read First

- `src/watch.rs` — Single-file module with the watch loop, event filtering, and subprocess management.

## Current Status

Fully implemented. The watcher uses `notify-debouncer-full` for cross-platform file system events. Works on macOS (FSEvents), Linux (inotify), and Windows (ReadDirectoryChanges).

## Notes

- The watch command is one of two "long-running" modes alongside MCP. Both take over the process and run indefinitely.
- An initial check runs immediately on startup before entering the watch loop, giving the user immediate feedback.
