## MODIFIED

### REQUIREMENT REQ-cmd-archive-tasks-001

The archive-tasks command SHALL delegate task archival safely and SHALL distinguish dry-run,
no-op, per-file, and summary output.

Acceptance Criteria

- `cmd_archive_tasks(root, dry_run, format)` loads config, resolves `config.specs_dir`, and
  delegates to `archive::archive_tasks(root, &specs_dir, dry_run)`.
- Dry-run prints preview output and makes no writes; apply mode reports completed work.
- Empty results remain successful and do not print a misleading aggregate summary.
- Per-file and aggregate counts report affected tasks and files truthfully.
- JSON is one ANSI-free document, and `--json` is byte-equivalent to `--format json`.
- Markdown and GitHub render a heading, optional dry-run notice, result table, and truthful summary.
- Windows separators normalize to `/`; Unix literal backslashes retain identity.
- Markdown/GitHub paths use sanitized variable-length code spans.
- JSON exposes complete/partial state, planned/succeeded/failed counts, and structured errors.
- Any incomplete operation exits 1 after rendering and never claims `applied: true`.

### SPEC SECTION Invariants

4. JSON is one parseable, ANSI-free document; `--json` and `--format json` are equivalent.
5. Markdown and GitHub formats render a heading, dry-run notice, result table, and truthful summary.
6. Structured dry-run output distinguishes `would_change: true` from `applied: false`.
7. Structured paths normalize only actual host separators.
8. Markdown/GitHub paths cannot inject table rows or break code spans.
9. Incomplete work renders truthful operation evidence and exits 1.
