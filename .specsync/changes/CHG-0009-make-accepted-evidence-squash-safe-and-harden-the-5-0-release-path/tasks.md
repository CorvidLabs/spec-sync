---
change: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
artifact: tasks
---

# Tasks

- [x] Add fail-closed squash-merge evidence validation.
- [x] Add positive and adversarial Git topology regressions.
- [x] Archive the accepted 5.0 workspaces from the merged tree.
- [x] Restrict and validate release tags.
- [x] Pin and document Action version behavior.
- [x] Run lifecycle, unit, integration, package, site, and Action consumer gates.
- [x] Isolate squash topology fixtures from Windows `core.autocrlf`.
- [x] Add versioned, collision-safe digest framing and file-kind/mode evidence.
- [x] Add binary, NUL-boundary, executable-mode, symlink, and line-ending regressions.
- [x] Update the repository lifecycle stamp to 5.0.0.
- [x] Restrict the published crate to executable sources and required user-facing metadata.
- [x] Make squash integration evidence repository-relative for nested SpecSync projects.
- [x] Add a nested-project squash integration regression.
- [x] Replace duplicated README manuals with a concise product entry point and documentation map.
- [x] Correct linked architecture, configuration, quick-start, and companion-file documentation.
- [x] Re-run and record every locally available gate after the failed acceptance was invalidated.

## Release Gates

Closing acceptance remains blocked on the corrected Linux, macOS, and Windows PR matrix. Release remains blocked on
the post-acceptance matrix and post-merge `main`.
