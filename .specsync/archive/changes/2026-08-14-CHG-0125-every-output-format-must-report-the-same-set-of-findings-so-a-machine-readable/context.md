---
change: CHG-0125-every-output-format-must-report-the-same-set-of-findings-so-a-machine-readable
artifact: context
---

# Context

One broken tree, every format, measured:

    text              mul + sub, exit 1
    json              same two identities
    markdown/github   same two identities
    table / csv       SUMMARY ONLY — identities absent, still exit 1
    coverage --json   NO findings at all — {file_coverage:100, modules:[], uncovered_files:[]}

Two defects. `check --format table` and `--format csv` exit 1 while naming
nothing, so a consumer parsing CSV sees zero rows and concludes the tree is
clean while the exit code says otherwise. And `coverage --format json`, the
payload an agent reads, carried no findings at all.

Table and Csv were never implemented for `check`. They shared the Text arm,
which printed only the summary and the coverage line, while per-finding output
sat behind `matches!(format, Text)`. They were aliases for "text minus the
findings".

Explicitly NOT a defect, and not addressed here: `coverage`'s format flag being
a no-op for markdown/github/table/csv. That is a missing feature. Conflating it
would have widened this change past what the evidence supports.
