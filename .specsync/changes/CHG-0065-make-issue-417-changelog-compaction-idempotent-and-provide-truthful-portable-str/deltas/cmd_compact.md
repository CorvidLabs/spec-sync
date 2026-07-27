## MODIFIED

### REQUIREMENT REQ-cmd-compact-001

The compact command SHALL delegate deterministic changelog compaction and SHALL preserve dry-run
and summary behavior.

Acceptance Criteria

- `cmd_compact(root, keep, dry_run, format)` loads config, resolves `config.specs_dir`, and
  delegates to `compact::compact_changelogs(root, &specs_dir, keep, dry_run)`.
- `--keep N` controls how many changelog entries to retain per spec.
- Dry-run prints preview output and makes no writes; apply mode reports completed work.
- Empty results remain successful and do not print a misleading aggregate summary.
- Per-spec and aggregate counts report removed and retained ordinary rows truthfully, with correct
  singular/plural labels.
- JSON is one ANSI-free document, and `--json` is byte-equivalent to `--format json`.
- Markdown and GitHub render a heading, optional dry-run notice, result table, and truthful summary.
- Windows separators normalize to `/`; Unix literal backslashes retain identity.
- Markdown/GitHub paths use sanitized variable-length code spans.
- JSON exposes complete/partial state, planned/succeeded/failed counts, and structured errors.
- Any incomplete operation exits 1 after rendering and never claims `applied: true`.

### SPEC SECTION Invariants

4. Per-spec and aggregate output use correct singular/plural labels and exclude the generated
   summary from the kept count.
5. JSON is one parseable, ANSI-free document; `--json` and `--format json` are equivalent.
6. Markdown and GitHub formats render a heading, dry-run notice, result table, and truthful summary.
7. Structured dry-run output distinguishes `would_change: true` from `applied: false`.
8. Structured paths normalize only actual host separators.
9. Markdown/GitHub paths cannot inject table rows or break code spans.
10. Incomplete work renders truthful operation evidence and exits 1.
