---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: requirements
---

# Requirements

`REQ-output-00N` — one finding list SHALL back every format, and every format
SHALL render all of it. The set a consumer sees does not depend on `--format`.

`REQ-cmd-check-00N` — table and csv SHALL name each finding; csv one row per
finding with stable columns and correct quoting.

`REQ-cmd-coverage-00N` — the JSON payload SHALL carry the same findings the text
renderer reports.

`REQ-mcp-00N` — both coverage surfaces SHALL emit payloads from one shared
constructor.

Out of scope: making `coverage`'s format flag honour non-JSON formats, and exit
code behaviour, which is unchanged.
