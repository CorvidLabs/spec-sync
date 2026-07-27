---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: testing
---

# Testing

## Characterization and regressions

- repeated compaction is byte-identical without new excess rows;
- later excess rows fold the old summary count/range;
- exact-shape unmarked rows are preserved and duplicate marked summaries fail;
- 0–3 backslashes, code-span pipes, malformed widths, and secondary tables are characterized;
- LF, CRLF, mixed endings, no-final-newline state, keep-zero, and retained counts are exact;
- fixed-width count overflow fails without panic or wrap;
- dry runs do not write and report truthful totals;
- `--json` and `--format json` parse to equivalent documents;
- Markdown/GitHub safely contain pipes, backtick runs, controls, bidi marks, and hostile paths;
- Windows separators render with `/` while Unix literal backslashes remain unchanged; and
- unreadable/malformed targets and staged/published failures produce truthful nonzero outcomes with
  zero preflight writes and explicit complete/partial/error fields.

## Commands

- `fledge run fmt`
- `fledge run lint`
- `fledge run test -- compact_`
- `fledge run test -- archive_tasks_`
- `fledge run test -- structured_output_paths_`
- `fledge run test -- markdown_code_span_`
- `fledge lanes run verify`
- `fledge spec check --strict`
- `specsync coverage --require 100`
- `specsync score --all --min-score 80`
- `fledge trust verify --range origin/main..HEAD`

## External and independent evidence

Run compact and archive-tasks dry-run/apply/idempotence scenarios in the private
`CorvidLabs/spec-sync-sandbox` repository with the exact candidate binary. Require a separate agent
to map every issue-body facet to code and tests, and another to perform adversarial compatibility
and regression review. Resolve every high/medium finding before recording Attest provenance or
requesting closing approval.
