---
module: cmd_resolve
version: 5
status: stable
files:
  - src/commands/resolve.rs
db_tables: []
tracks: [159]
depends_on:
  - specs/commands/commands.spec.md
  - specs/parser/parser.spec.md
  - specs/registry/registry.spec.md
  - specs/validator/validator.spec.md
  - specs/util/util.spec.md
---

# Cmd Resolve

## Purpose

Implements the `specsync resolve` command. Resolves dependency references — local by file existence, cross-project via registry lookups. `--remote` fetches remote registries from GitHub. `--verify` performs deep content verification: fetches actual remote spec files, checks exports, validates bidirectional dependencies, and detects drift.

## Public API

### Exported Functions

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_resolve` | `root: &Path, remote: bool, verify: bool, cache_ttl: u64` | `()` | Resolve all dependency references; optionally verify remote content |

## Invariants

1. Classifies deps as local vs cross-project
2. Local verified by file existence
3. No network calls without `--remote` or `--verify`
4. `--verify` implies `--remote` — registry check always runs first
5. Warnings for unresolvable refs (does not exit non-zero)
6. `--verify` exits 1 on breaking drift (deprecated status, missing exports)
7. Bidirectional dependency mismatches are warnings, not errors
8. Remote spec content is cached with configurable TTL (default 1 hour)
9. Cache stored in `.specsync-cache/remote-specs/` under project root
10. The remote spec cache is never read or written through a symbolic link, inside the project or not: a cache never needs one, and the entry's file name comes from the repository's own `depends_on`. A cache file or directory that is a link, or not a regular file and a directory, skips the cache for that entry with one `warning: remote spec cache not used` on stderr per run, and the fetched content is used as with no cache (REQ-cmd-resolve-002).

## Behavioral Examples

### Scenario: All local deps resolve

- **Given** all `depends_on` point to existing files
- **When** `cmd_resolve(root, false, false, 3600)` runs
- **Then** prints green checkmarks for each resolved dependency

### Scenario: Remote registry check

- **Given** a spec with `depends_on: ["corvid-labs/algochat@auth"]`
- **When** `cmd_resolve(root, true, false, 3600)` runs
- **Then** fetches `specsync-registry.toml` from `corvid-labs/algochat`
- **And** prints checkmark if `auth` module exists in the registry

### Scenario: Verify detects deprecated remote spec

- **Given** a cross-project ref to `remote-repo@parser`
- **When** `cmd_resolve(root, true, true, 3600)` runs
- **And** the remote spec's status is `deprecated`
- **Then** prints `DRIFT remote-repo@parser: remote spec status is "deprecated"`
- **And** exits with code 1

### Scenario: Verify detects missing export

- **Given** local spec consumes `parse_ast` from `remote-repo@parser`
- **When** `cmd_resolve(root, true, true, 3600)` runs
- **And** `parse_ast` no longer exists in remote spec's Public API table
- **Then** prints `DRIFT ... but export 'parse_ast' no longer exists in remote spec`
- **And** exits with code 1

### Scenario: Verify warns on non-bidirectional dependency

- **Given** local spec depends on `remote-repo@parser`
- **When** `cmd_resolve(root, true, true, 3600)` runs
- **And** remote spec does not reference our repo in its `depends_on`
- **Then** prints `WARN ... but remote spec does not reference <our-repo>`
- **And** does NOT exit non-zero (warning only)

### Scenario: Cache avoids redundant fetches

- **Given** a prior `--verify` run cached remote spec content
- **When** `cmd_resolve(root, true, true, 3600)` runs within the TTL window
- **Then** reads from `.specsync-cache/remote-specs/` instead of fetching

## Error Cases

| Condition | Behavior |
|-----------|----------|
| Local dep missing | Warning printed |
| Remote registry fetch fails | Warning, continues |
| Remote spec fetch fails | Warning, continues |
| Remote spec unparseable | Warning, continues |
| Remote spec deprecated/removed | DRIFT error, exit 1 |
| Consumed export missing from remote | DRIFT error, exit 1 |
| Non-bidirectional dependency | Warning only |
| A cache entry, `.specsync-cache` or `remote-specs` is a symbolic link or not a regular file and a directory | The cache is skipped for that entry with one stderr warning naming the path; nothing is read or written through it; the fetched content is still used |

## Dependencies

### Consumes

| Module | What is used |
|--------|-------------|
| commands | `load_and_discover` |
| parser | `parse_frontmatter`, `get_spec_symbols` |
| registry | `fetch_remote_registry`, `fetch_remote_spec`, `parse_remote_spec`, `RemoteSpec` |
| validator | `is_cross_project_ref`, `parse_cross_project_ref` |
| github | `detect_repo` |

### Consumed By

| Module | What is used |
|--------|-------------|
| cli (main.rs) | Entry point for `specsync resolve` |

## Change Log

| Date | Change |
|------|--------|
| 2026-10-10 | Add `cmd_resolve.3md` beside the spec. The first plane is context and later planes are lessons. `context.md` stays. |
| 2026-04-10 | v2: Added `--verify` for deep content verification, `--cache-ttl`, drift detection, exit codes |
| 2026-04-09 | Initial spec |
| 2026-07-11 | CHG-0010-canonicalize-every-specsync-5-0-contract-and-requirement: Canonicalize every SpecSync 5.0 contract and requirement |
| 2026-08-01 | CHG-0071-land-pre-6-0-product-fixes-for-hooks-init-coverage-naming-and-exit-codes-scoped: Land pre-6.0 product fixes for hooks init coverage naming and exit codes (scoped paths) |
