---
spec: compact.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/compact.rs` | `fledge run test -- compact::tests` | Core trimming plus marked ownership, duplicate rejection, fixed-width overflow, backslash parity/code spans, contiguous tables, exact CRLF preservation, keep-zero, staging all-or-none, and idempotence |
| `tests/integration/commands.rs` | `fledge run test -- compact_` | CLI dry-run no-write behavior, output counts, newline preservation, and byte-identical second run |

## Coverage Gaps

(none for issue #417; command rendering is covered by the `cmd_compact`, `cmd_archive_tasks`, and CLI module test matrices)

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Compact a long changelog | a spec with 20 changelog entries and `keep = 5` | `compact_changelogs` is called | the first 15 entries are replaced with a single summary row of the form `\| <first_date> — <last_date> \| Compacted: 15 entries \|` |
| Short changelog (no compaction needed) | a spec with 3 changelog entries and `keep = 5` | `compact_changelogs` is called | the spec is skipped (not included in results) |
| Dry run | specs with long changelogs | `compact_changelogs(root, specs_dir, 5, true)` is called | returns `CompactResult` entries but does not modify any files |
| Repeat compaction | a generated summary plus exactly `keep` ordinary rows | run compaction again | file bytes and summary metadata remain unchanged |
| New rows after compaction | a generated summary plus more than `keep` ordinary rows | run compaction again | old and new removed counts are folded and the original range start survives |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| Read/parse/stage failure | Typed failure, nonzero command outcome, and zero writes before publication | `compact_preflight_failure_prevents_all_writes` plus command JSON failure fixture |
| No changelog section found | Spec is silently skipped | Keep or add a focused assertion before changing this behavior |
| Unmarked row exactly resembles a summary | Preserve it as ordinary history; never fold it as generated state | `compact_preserves_exact_shape_user_row_without_marker` |
| Duplicate generated summaries | Refuse ambiguous folding | `compact_rejects_multiple_marked_summaries` |
| Escaped/code-span pipe | Honor backslash parity and code delimiters | `split_cells_honors_backslash_parity_and_code_spans`, `compact_handles_escaped_pipes_in_cells` |
| CRLF or mixed endings | Preserve every untouched terminator | `compact_preserves_crlf_and_mixed_line_endings` |
| Count overflow | Return an error rather than panic or wrap | `compact_rejects_summary_count_overflow` |

## Reviewer Checklist

- Run the narrow source command above before the full suite when changing `src/compact.rs`.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an error message changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`.
