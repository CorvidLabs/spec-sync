---
spec: cmd_compact.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `tests/integration/commands.rs` | `fledge run test -- compact_` | Text dry-run/idempotence, JSON shorthand equivalence and portable paths, Markdown structure, newline preservation |
| `src/commands/compact.rs` | `fledge run test -- commands::compact::tests` | Unix backslash identity, hostile dynamic spans, and truthful late-publish partial JSON/Markdown |
| `src/compact.rs` (delegate logic) | `fledge run test -- compact::tests` | Core and #417 regression matrix |

## Coverage Gaps

(none for issue #417)

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Compact changelogs | a spec's `## Change Log` has 25 entries, `--keep 10` | `cmd_compact(root, 10, false)` | 15 oldest entries removed, 10 newest kept; per-spec line + summary printed |
| Dry run | a spec exceeds the keep limit | `cmd_compact(root, 10, true)` | prints "Dry run" banner + "would compact" lines, modifies no files |
| JSON dry run | a spec exceeds the keep limit | run with `--format json` and `--json` | both emit the same valid ANSI-free document; `would_change` is true and `applied` is false |
| Markdown dry run | a spec exceeds the keep limit | run with `--format markdown` | emits a heading, notice, table, and truthful summary; modifies no files |
| Windows structured paths | the delegate returns a Windows path | render JSON or Markdown | output uses portable `/` separators |
| Unix literal backslash | a legal Unix filename contains `\` | render structured output | path identity is preserved |
| Hostile Markdown path | a path contains pipes, backtick runs, controls, or bidi marks | render Markdown/GitHub | one sanitized code element remains inside one table cell |
| Incomplete apply | any read/parse/stage/publish operation fails | render JSON | exit 1; `complete: false`, truthful counts/errors, and no false `applied` |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No specs need compaction (all within limit) | Prints "No changelogs need compaction (all within limit)." and returns, no summary | Keep or add a focused assertion before changing this behavior |
| Fewer entries than `--keep` | Spec unchanged, not reported | Keep or add a focused assertion before changing this behavior |
| Multiple affected specs | Summary sums `removed` across results and reports the spec count | Keep or add a focused assertion before changing this behavior |
| Structured stdout contamination | JSON must be exactly one parseable document with no banner or ANSI bytes | `compact_json_formats_are_clean_truthful_and_equivalent` |
| Host-native path separators | Normalize actual Windows separators without aliasing Unix filename bytes | cfg-specific portable-path tests and structured CLI assertions |
| Markdown path injection | Pipes/backticks/controls cannot create rows or break code elements; every Unix backslash parity, including backslash-before-pipe, remains literal | `markdown_code_span_sanitizes_paths_and_uses_safe_delimiters`, `markdown_code_span_preserves_literal_unix_backslashes`, `markdown_code_span_preserves_backslash_before_pipe_in_one_table_cell` |
| Incomplete operations | Render valid structured output, exit 1, and never claim complete application | focused failure integration fixture, `partial_publish_reports_are_truthful_in_json_and_markdown` |

## Reviewer Checklist

- Run `cargo run -- compact --help` and confirm the help text still names the documented flags (`--keep`, `--dry-run`).
- Run `cargo test compact` when changing the delegate; run `cargo test commands::compact` when changing the wrapper.
- Run `fledge run test -- compact_` for the unit and end-to-end issue #417 matrix.
- Reproduce one Behavioral Verification row with a temporary spec fixture before changing user-visible output.
- If an output string changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
