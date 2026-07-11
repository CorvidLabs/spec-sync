## ADDED

### REQUIREMENT REQ-watch-001

Watch mode SHALL debounce supported filesystem changes and rerun isolated validation without crashing on recoverable watcher errors.

Acceptance Criteria
- Debounce interval of 500ms prevents rapid-fire re-runs
- Extra 300ms minimum between consecutive check runs
- Only reacts to Create, Modify, and Remove events (not metadata-only changes)
- Queued events are drained after each check completes
- Exits immediately if there are no directories to watch
- Check runs as a child process to isolate exit calls from the watcher
- Screen is cleared before each re-run
- Changed file path is displayed in the separator header
- Watches both spec and source directories
