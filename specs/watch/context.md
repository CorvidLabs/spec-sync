---
spec: watch.spec.md
---

## Key Decisions

- **500ms debounce + 300ms minimum gap**: The debounce prevents rapid re-runs from burst saves (e.g., IDE auto-save). The extra 300ms gap ensures the previous check finishes before starting the next one.
- **Subprocess isolation**: Each check runs as a forked child process rather than an in-process function call. This isolates any `exit()` calls in the check path from killing the watcher.
- **Event filtering**: Only Create, Modify, and Remove events trigger re-runs. Access and metadata changes are ignored to reduce noise.
- **Screen clear between runs**: The terminal is cleared before each re-run for a clean output, with the changed file path displayed in a separator header so the user knows what triggered it.
- **Exit on no paths**: If no valid spec directory, source directory or exact regular source file exists, the watcher exits immediately with an error rather than watching nothing silently.

- **Exact file selections**: Confined regular source files use `NonRecursive`; directories retain `Recursive`. A non-recursive immediate-parent lifecycle observation detects atomic replacement/recreation, but filters to exact selected names and rearms only regular confined files. No recursive parent/root watch is substituted for a file.
- **Path diagnostics**: Missing paths keep `nonexistent_watch_directory` for compatibility. Links, escapes, invalid types and failed registrations use `invalid_watch_path` and disclose the actual reason. Component inspection uses a retained root capability and identity is rechecked around path-based registration; this is not a claim that the watcher API uses retained descriptors.

- **Coarse parent events**: Compare bounded SHA-256 snapshots of the selected exact files only. This detects same-size edits even if modification time is restored, without inspecting siblings. Reads use the same 8 MiB per-source cap as validation, no-follow acquisition and before/after identity checks. Precise source events refresh snapshots so a later sibling notification cannot repeat an already-observed edit. Fingerprint failure is disclosed separately from precise native monitoring.

## Files to Read First

- `src/watch.rs` — Single-file module with the watch loop, event filtering, and subprocess management.

## Current Status

The watcher uses `notify-debouncer-full` with the platform recommended backend: macOS (FSEvents), Linux (inotify), and Windows (ReadDirectoryChanges). Runtime event delivery must be demonstrated with a positive spec-edit control before claiming a source-edit rerun passed on a particular platform.

## Notes

- The watch command is one of two "long-running" modes alongside MCP. Both take over the process and run indefinitely.
- An initial check runs immediately on startup before entering the watch loop, giving the user immediate feedback.

- Native Mac qualification limitation: the same causal invalid-spec/error followed by restoration/PASS fixture stopped at restoration with both the frozen pre-followup binary and the corrected candidate. The full Mac source/save sequence is not qualified. Preserve that inherited/local backend result separately from focused unit proof; require the real Linux hosted sequence before merge/release.
