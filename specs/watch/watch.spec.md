---
module: watch
version: 7
status: stable
files:
  - src/watch.rs
db_tables: []
tracks: [4]
depends_on:
  - specs/config/config.spec.md
---

# Watch

## Purpose

File watcher that re-runs `specsync check` on file changes. Watches the spec directory and configured source directories or exact regular source files with 500ms debounce, providing live validation feedback during development.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `run_watch` | `root, strict, require_coverage, format` | `()` | Run check in watch mode — initial run, then re-run on file changes |

## Invariants

1. Debounce interval is 500ms — prevents rapid-fire re-runs during batch saves
2. Additional 300ms minimum between runs as extra debounce
3. Only reacts to Create, Modify, and Remove events — ignores access and metadata changes
4. Queued events are drained after each check run to prevent cascading re-runs
5. Watch exits if no valid configured paths can be registered (a spec directory, source directory, or exact regular source file). Missing-path diagnostics retain their legacy machine code.
6. Check is run as a child process (fork of current executable) to isolate exit calls
7. Screen is cleared before each re-run for clean output
8. Changed file path is displayed in the separator header
9. Directories register recursively; confined exact regular source files register non-recursively. Non-recursive immediate-parent lifecycle observation supports atomic saves and recreation, but exact-path filtering rejects sibling events and never adds a recursive project-root watch.
10. Configured paths reject absolute/parent escapes, symlink/reparse components and unsupported file kinds; registration and rearming recheck target identity; a file selection cannot become a recursive directory selection. Missing paths and invalid paths have distinct text/JSON disclosures.
11. The startup banner lists only successfully registered paths; if all registrations fail, watch exits 1.
12. Coarse immediate-parent events compare SHA-256 snapshots of declared exact files only, through confined no-follow opens with the validator's existing 8 MiB per-source cap. Precise source events refresh the remembered snapshot. Unchanged siblings do not trigger validation; fingerprint failures are disclosed without claiming precise monitoring was dropped.
13. The pass/fail footer reflects the actual check results — the child's stdout is streamed through and its summary line is parsed, because `enforcement = warn` (the default) exits 0 even when specs failed

## Behavioral Examples

### Scenario: Initial run

- **Given** a project with specs and source directories
- **When** `run_watch` is called
- **Then** runs `specsync check` immediately, then watches for changes

### Scenario: File modification triggers re-check

- **Given** watch mode is running
- **When** a `.spec.md` file is modified
- **Then** re-runs check after 500ms debounce, showing the changed file path

### Scenario: Exact Cargo source save

- **Given** initialization selected a root `lib.rs` or `main.rs` in `source_dirs`
- **When** the existing regular file is registered and edited on a working native backend
- **Then** it is watched non-recursively and a source edit re-runs validation; it is never reported as nonexistent or replaced by a recursive project-root watch

### Scenario: Rapid saves

- **Given** watch mode is running
- **When** multiple files are saved within 500ms
- **Then** only one check run is triggered (debounced)

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No valid directories or exact files to watch | Prints error retaining `No directories to watch`, exits with code 1 |
| Watcher creation fails | Panics with "Failed to create file watcher" |
| Missing configured path | Emits legacy `nonexistent_watch_directory` JSON or a human missing-path warning; continues if other valid paths register |
| Invalid path, link/reparse component, escape, unsupported kind or registration failure | Emits `invalid_watch_path` JSON or a human invalid-path warning; never labels an existing regular source file nonexistent |
| Coarse exact-source fingerprint exceeds 8 MiB, cannot be read, or changes during acquisition | Discloses unavailable coarse fingerprinting; never reads unselected siblings or claims unsupported backend proof |
| Individual path watch fails or changes identity during registration | Discloses and omits the path from the banner; continues only if another valid path registers |
| Check command fails or any spec failed | Prints "Some checks failed", continues watching |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| config | `load_config` |
| notify | File system event watching with debounce |
| cap-std / cap-primitives | Confined component inspection, no-follow exact-file opens and filesystem identity checks |
| sha2 | Bounded selected-source fingerprints for coarse parent notifications |

### Consumed By

| Module | What is used |
|--------|-------------|
| main | `run_watch` (via `watch` subcommand) |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-14 | v6: Complete exact Cargo source handling: non-recursive exact regular files, recursive directories, confined non-link selection, truthful missing/invalid diagnostics and registration identity checks. |
| 2026-10-10 | Add `watch.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-03-25 | Initial spec |
| 2026-06-11 | v2: Footer parses the check summary line so it never prints "All checks passed!" under a failing report |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-17 | CHG-0142-watch-must-disclose-a-dropped-directory-and-not-claim-a-pass-over-an-empty-check: Watch must disclose a dropped directory and not claim a pass over an empty check |
| 2026-10-15 | complete-exact-cargo-source-handling-and-scoped-batch-recognition-for-specsync-6-1-1: Complete exact Cargo source handling and scoped batch recognition for SpecSync 6.1.1 |
