---
change: CHG-0065-make-issue-417-changelog-compaction-idempotent-and-provide-truthful-portable-str
artifact: context
---

# Context

GitHub issue #417 reports correctness and output-contract defects in `specsync compact`, with
related structured-output gaps in `archive-tasks`. The prior implementation treated its own
generated summary row as ordinary history on later runs, could produce malformed wide-table
summaries, lost trailing-newline state, misreported retained counts, and did not consistently honor
JSON or Markdown output selection.

The existing PR implementation addresses those behaviors across the `compact`, `cmd_compact`,
`cmd_archive_tasks`, and root `cli` owners. After rebasing onto the MCP-security delivery, hosted
Windows tests exposed one remaining compatibility defect: repo-relative result paths retained
Windows `\` separators in JSON and Markdown, making structured output host-dependent.

The change must preserve the established text experience and task/changelog mutation semantics.
It must not broaden file ownership, silently rewrite unrelated rows, or treat user-authored
`Compacted:` prose as tool-owned state.

Independent acceptance and adversarial reviews found that shape-only ownership could consume exact
user lookalikes, multiple summaries could corrupt counts, line reconstruction normalized CRLF,
backslash parity/code spans were parsed incorrectly, unchecked counts could overflow, Unix
backslashes were aliased, Markdown paths were injectable, and write failures could leave partial
state while exiting successfully. These findings are release blockers and are now in scope.

The rebuilt PR #447 branch now contains eight dependency-ordered commits based directly on current
`main`. Candidate `d6c12fdbf4f2a1d96c15408946392074c02244d5` passed the full local lane, private
sandbox replay, two independent reviews, signed 0.95-confidence Attest verification, hosted Trust,
all cross-platform GitHub checks, and Corvin approval. CHG-0065 is ready for a refreshed definition
approval, native lifecycle verification, and explicit closing approval.
