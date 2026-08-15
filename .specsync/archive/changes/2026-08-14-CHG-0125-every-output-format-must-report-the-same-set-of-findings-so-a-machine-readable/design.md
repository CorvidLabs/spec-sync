---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: design
---

# Design

One list, rendered many ways. `output.rs` gains the finding list and the
renderers; every format draws from the same list rather than deciding for itself
what to show.

The coverage payload collapses from three hand-built copies — CLI, MCP tool, MCP
resource — to one constructor. That is the point rather than a tidiness: the
previous attempt at this issue fixed `tool_coverage` and missed
`resource_coverage`, because there was no single place to fix. The two MCP
surfaces are now byte-identical on the same tree, verified.

`csv_field` existed twice, in `init.rs` and `init_registry.rs`. Both deleted in
favour of one.

**Staleness is the subtle half.** Staleness findings drive `effective_warnings`
and therefore the exit code, but live in `stale_entries`, not `all_warnings`.
`warnings_with_staleness` merges them, and it must be applied at every non-text
arm. It was initially applied at two — table and csv — leaving markdown and
github exiting 1 while naming no finding: RANK 2 verbatim, surviving inside its
own fix, in the two formats a human actually reads. `--format github` is the
PR-comment renderer.

The test that missed it looped over exactly `["table", "csv"]` — the two arms
that had been fixed. A test written from the fix can only confirm the fix. It
now loops over all four.

Deliberately unchanged: exit codes, which were already format-stable.
