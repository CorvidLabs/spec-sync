---
spec: watch.spec.md
---

## Tasks

- [ ] Add `--filter` flag to watch only specific spec or source directories
- [ ] Add desktop notification support on check failure (via `notify-rust` or similar)
- [ ] Support incremental re-validation (only check specs affected by changed files)
- [ ] Add `--once` flag to run a single check and exit (useful for scripting)

## Done

- [x] File watcher with 500ms debounce
- [x] Event filtering (create/modify/remove only)
- [x] Subprocess isolation for check runs
- [x] Screen clear with change notification header
- [x] Initial check on startup
- [x] Cross-platform support (macOS, Linux, Windows)

## Gaps

- No way to filter which directories/files are watched (watches everything)
- No integration with editor/IDE for inline diagnostics
- Re-runs full check on every change — no incremental validation

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
