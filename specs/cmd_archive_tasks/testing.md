---
spec: cmd_archive_tasks.spec.md
---

## Automated Coverage

| Area | Command | Assertions To Watch |
|------|---------|---------------------|
| `tests/integration/commands.rs` | `fledge run test -- archive_tasks_` | Text dry-run, JSON shorthand equivalence, Markdown/GitHub structure, exit-1 failure schema, and no-write guarantees |
| `src/commands/archive_tasks.rs` | `fledge run test -- commands::archive_tasks::tests` | Pluralization, platform-aware separators, one-element code rendering, backslash/pipe composition, text/Markdown control and bidi sanitization, and JSON escaping |
| `src/archive.rs` (delegate logic) | `fledge run test -- archive::tests` | Parsing plus plan/stage/publish/rollback transaction boundaries |

## Coverage Gaps

(none for issue #417)

## Behavioral Verification

| Flow | Fixture / Setup | Action | Expected Result |
|------|-----------------|--------|-----------------|
| Tasks archived successfully | a companion `tasks.md` has checked items (`- [x]`) | `cmd_archive_tasks(root, false, Text)` is called | checked items move to `## Archive`; per-file line + summary printed |
| Text dry run | `tasks.md` has completed items | run with `--dry-run` | prints "Dry run" banner and "would archive" lines, modifies no files |
| JSON dry run | `tasks.md` has completed items | run with `--format json` and `--json` | both emit the same valid ANSI-free document; `would_change` is true and `applied` is false |
| Markdown/GitHub dry run | `tasks.md` has completed items | run with `--format markdown` and `--format github` | both emit a heading, notice, table, and truthful summary; neither modifies files |
| Incomplete apply | one candidate is valid and another is non-UTF-8 | run JSON apply | exits 1 with `complete: false`, no succeeded operations, a read failure, and zero file changes |
| Platform-aware structured paths | Windows separators or a Unix literal backslash | render JSON or Markdown | Windows uses `/`; Unix preserves the literal backslash |
| Adversarial Markdown path | path contains pipes, backtick runs, line/control, or bidi characters | render Markdown/GitHub | one intact row with one code element and visible safe escapes |

## Regression Matrix

| Case | Required Behavior | Test Obligation |
|------|-------------------|-----------------|
| No `tasks.md` files / no completed tasks (empty result) | Prints "No completed tasks to archive." and returns, no summary | Keep or add a focused assertion before changing this behavior |
| Multiple affected files | Summary sums `archived_count` across all results and reports the file count | Keep or add a focused assertion before changing this behavior |
| Structured stdout contamination | JSON must be exactly one parseable document with no banner or ANSI bytes | `archive_tasks_json_formats_are_clean_truthful_and_equivalent` |
| Incomplete mutation | Failure must exit 1 after rendering; `applied` false; no success on preflight/stage failure | `archive_tasks_apply_failure_exits_one_and_reports_zero_writes` and archive transaction unit tests |
| Host-native path separators | Windows separators normalize without corrupting Unix identities | cfg-specific `structured_output_*` unit tests and structured CLI assertions |
| Markdown path injection | Pipes, backticks, controls, and bidi controls cannot create rows or malformed code elements; every Unix backslash parity, including backslash-before-pipe, remains literal | `markdown_paths_use_safe_dynamic_code_spans`, `markdown_paths_preserve_literal_unix_backslashes`, `markdown_paths_preserve_backslash_before_pipe_in_one_table_cell` |
| Terminal diagnostic injection | Text paths/errors visibly encode controls and bidi characters | `text_paths_and_errors_visibly_escape_control_and_bidi_characters` |

## Reviewer Checklist

- Run `cargo run -- archive-tasks --help` and confirm the help text still names the documented flags and behavior.
- Run `fledge run test -- archive::tests` when changing the delegate; run `fledge run test -- commands::archive_tasks::tests` when changing the wrapper.
- Reproduce one Behavioral Verification row with a temporary project fixture before changing user-visible output.
- If an output string changes, update the matching Regression Matrix row and test assertion in the same commit.
- Run the release checks for this module: `fledge run fmt`, `fledge run lint`, `fledge run test`, `fledge spec check --strict`, `fledge run build`.
