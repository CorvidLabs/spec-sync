---
module: cmd_merge
version: 7
status: stable
files:
  - src/commands/merge.rs
db_tables: []
tracks: []
depends_on:
  - specs/merge/merge.spec.md
  - specs/config/config.spec.md
  - specs/types/types.spec.md
---

# Cmd Merge

## Purpose

Implements the `specsync merge` command. Auto-resolves git merge conflicts in spec files, flagging files that need manual resolution.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_merge` | `root: &Path, dry_run: bool, all: bool, format: OutputFormat` | `()` | Resolve merge conflicts in spec files |
| `cmd_install_merge_driver` | `root: &Path, format: OutputFormat` | `()` | Register the local git merge driver and append `merge=specsync` attribute rules for the configured specs directory |
| `cmd_merge_file` | `ancestor: &Path, current: &Path, other: &Path, pathname: Option<&str>` | `()` | Git merge-driver entry. Writes the merge into `current` and exits 0 only when that merge is clean |

## Invariants

1. Delegates to `merge::merge_specs()`
2. `--all` scans all spec files for conflict markers
3. Exits 1 if any merge needs manual resolution
4. `specsync merge --install` writes the attribute rules into `.gitattributes` and sets `merge.specsync.driver` to this executable's `merge-file` command. A symlinked `.gitattributes` is refused. When the configured specs directory is `.` or `./`, the unprefixed `**/*.spec.md` and `**/requirements.md` rules match root and nested contracts (REQ-cmd-merge-004)
5. `specsync merge-file` is the driver git runs as `%O %A %B %P`. It exits 1 when the contract still contains a conflict, and it does not loosen `cmd_merge`'s marker scan

## Behavioral Examples

### Scenario: Auto-resolved

- **Given** 3 specs with simple conflicts
- **When** `cmd_merge` runs
- **Then** all auto-resolved

### Scenario: Manual needed

- **Given** complex conflict
- **When** `cmd_merge` runs
- **Then** flags file, exits 1

## Error Cases

| Condition | Behavior |
|-----------|----------|
| No conflicts | Prints "no conflicts" |
| Complex conflict | Exits 1 with file path |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| merge | `merge_specs`, `MergeStatus` |
| config | `load_config` |
| types | `OutputFormat` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync merge` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `cmd_merge.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-14 | CHG-0124-a-source-file-or-spec-body-carrying-an-unresolved-merge-conflict-must-be-refused: A source file or spec body carrying an unresolved merge conflict must be refused, because extracting declarations from both sides of a hunk describes source that does not exist |
| 2026-11-05 | v5 / #862: `merge --install` registers the git driver and `merge-file` merges one contract for it |
| 2026-11-05 | merge-two-finalized-changes-to-one-module-without-a-spec-conflict: Merge two finalized changes to one module without a spec conflict |
| 2026-11-06 | v6 / #871: `merge --install` matches root and nested contracts when the configured specs directory is `.` or `./` (REQ-cmd-merge-004) |
| 2026-11-07 | make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root: Make merge driver installation match contracts when the specs directory is the repository root |
