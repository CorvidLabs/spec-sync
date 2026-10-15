---
spec: watch.spec.md
---

## Tasks

## Post-5.0 Roadmap

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

- [x] Accept confined exact regular sources non-recursively while preserving recursive directory watches
- [x] Distinguish missing-path and invalid-path text/JSON diagnostics
- [x] Add selection/mode, escape, symlink, unsupported-kind and replacement identity controls
- [x] Add bounded selected-content fingerprints for coarse parent events, including same-size/restored-mtime and sibling-negative controls
- [ ] Complete real backend spec/source, atomic-save, recreation and sibling-negative qualification on the immutable release candidate

## Gaps

- No CLI selection override; configured spec/source paths determine monitoring scope
- Native event delivery proof must include successful spec and source controls; a timeout is not a passing event test
- No integration with editor/IDE for inline diagnostics
- Re-runs full check on every change — no incremental validation

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
