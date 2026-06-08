---
spec: cmd_compact.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `src/commands/compact.rs` | cargo test commands::compact | Command wrapper has no inline tests (output formatting only); cover `cmd_compact` end-to-end before risky changes |
| `src/compact.rs` (delegate logic) | cargo test compact | `test_compact_changelog`, `test_compact_no_change_needed`, `test_compact_three_column_table` |

## Coverage Gaps

- No end-to-end CLI test asserts the wrapper's stdout (per-spec lines, "would compact" vs "compacted", the summary, or the "No changelogs need compaction…" path). Add one before changing user-visible output.

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Compact changelogs | a spec's `## Change Log` has 25 entries, `--keep 10` | `cmd_compact(root, 10, false)` | 15 oldest entries removed, 10 newest kept; per-spec line + summary printed |
| Dry run | a spec exceeds the keep limit | `cmd_compact(root, 10, true)` | prints "Dry run" banner + "would compact" lines, modifies no files |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs need compaction (all within limit) | Prints "No changelogs need compaction (all within limit)." and returns, no summary | Keep or add a focused assertion before changing this behavior |
| Fewer entries than `--keep` | Spec unchanged, not reported | Keep or add a focused assertion before changing this behavior |
| Multiple affected specs | Summary sums `removed` across results and reports the spec count | Keep or add a focused assertion before changing this behavior |

## Reviewer Checklist

- Run `cargo run -- compact --help` and confirm the help text still names the documented flags (`--keep`, `--dry-run`).
- Run `cargo test compact` when changing the delegate; run `cargo test commands::compact` when changing the wrapper.
- Reproduce one Behavioral Verification row with a temporary spec fixture before changing user-visible output.
- If an output string changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
