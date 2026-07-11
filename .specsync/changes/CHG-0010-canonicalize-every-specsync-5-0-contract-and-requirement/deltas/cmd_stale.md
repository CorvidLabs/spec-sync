## ADDED

### REQUIREMENT REQ-cmd-stale-001

The stale command SHALL report Git-distance staleness deterministically with threshold and maturity-status filtering.

Acceptance Criteria
- `specsync stale` lists specs whose source files have changed since the spec was last committed, sorted most-stale-first
- A spec is stale when any of its source files has `>= threshold` commits since the spec's last commit (default threshold: 5)
- Output reports per-spec commit count and the list of drifted source files (each with its own commit count)
- Honors the global `--exclude-status` / `--only-status` filters when selecting which specs to scan
- Supports `text`/`table`/`csv` (human), `json` (machine), and `markdown`/`github` output formats
- Exit code is 1 when any stale specs are detected, 0 when all are fresh (for CI usage)
- Requires a git repository: errors and exits 1 when `is_git_repo` returns false (JSON mode emits an error object instead of stderr text)

## MODIFIED

### SPEC SECTION Public API

**Exported Functions**

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `cmd_stale` | `root: &Path, format: types::OutputFormat, threshold: usize, exclude_status: &[String], only_status: &[String]` | `()` | Detect and report stale specs based on git commit distance |
