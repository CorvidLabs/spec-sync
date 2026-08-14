---
change: CHG-0123-staleness-that-cannot-be-measured-must-be-refused-not-reported-as-zero-drift-i
artifact: requirements
---

# Requirements

`REQ-git-utils-00N` — the absence of git history SHALL be representable as a
value distinguishing "no repository" from "no commits".

`REQ-cmd-report-00N`, `REQ-cmd-check-00N` — a staleness reader SHALL refuse when
history cannot answer, rather than reporting zero drift. JSON uses `null`, never
`0` or `[]`.

`REQ-cmd-stale-00N` — `stale` SHALL derive its precondition from the shared
helper, with its existing messages and JSON unchanged.

`REQ-cmd-lifecycle-00N` — the `no_stale` guard SHALL NOT pass when staleness is
unverifiable.

`REQ-scoring-00N` — the freshness dimension SHALL withhold rather than award
points it could not measure, and SHALL record that it did.

`REQ-mcp-00N` — MCP staleness surfaces SHALL report unmeasurable history as
unmeasurable.

Out of scope: what counts as stale when history IS available, and plain `check`
without `--stale`.
