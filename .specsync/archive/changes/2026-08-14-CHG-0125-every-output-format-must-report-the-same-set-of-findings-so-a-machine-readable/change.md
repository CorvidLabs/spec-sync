---
id: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
state: archived
type: bug_fix
base_commit: d4a79069ec53da9bcc707be11acccfee77ceba0a
---

# Every output format must report the same set of findings, so a machine-readable consumer cannot see fewer problems than a human reading the text

## Intent

Every output format must report the same set of findings, so a machine-readable consumer cannot see fewer problems than a human reading the text

## Affected Canonical Specs

- `output`
- `cmd_check`
- `cmd_coverage`
- `mcp`
- `cmd_init`
- `cmd_init_registry`

## Acceptance Criteria

- One broken tree run through every format of check and of coverage yields the same set of finding identities. Presentation differs; the set does not. csv emits one row per finding with stable columns, table an aligned list, and coverage --format json carries the findings it previously omitted entirely. Both MCP coverage surfaces emit byte-identical payloads built by one shared constructor rather than three hand-built ones. Staleness findings, which drive the exit code but live outside the warning list, reach every non-text format and not only the tabular pair. A clean tree is all-clear in every format with exit 0, and a zero-source tree still reports nothing measured rather than a percentage.

## No-spec Rationale

Not applicable
