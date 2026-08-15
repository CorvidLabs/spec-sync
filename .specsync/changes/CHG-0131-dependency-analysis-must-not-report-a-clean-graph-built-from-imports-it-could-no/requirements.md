---
change: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
artifact: requirements
---

# Requirements

`REQ-deps-00N` — dependency analysis SHALL NOT report a clean graph built from
imports it could not read or could not resolve.

`REQ-cmd-deps-00N` — every output format SHALL disclose what the analysis could
not read and could not attribute, without affecting the exit code.

Out of scope: gating `--strict` on either disclosure.
