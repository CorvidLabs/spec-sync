---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: docs
---

# Docs

CHANGELOG entry under Unreleased → Fixed, stating the consumer-visible
consequence: JSON, CSV, Markdown and MCP reported 100% where text reported
nothing measured, and a `--require-coverage` gate could pass on a tree it had
never measured.

`site/src/content/docs/cli.md` carried a Coverage JSON example showing a numeric
`file_coverage` for an empty tree; it now shows `null`.

`vscode-extension/src/extension.ts` consumed the percentage directly and is
updated to render the unmeasured state rather than a number.
