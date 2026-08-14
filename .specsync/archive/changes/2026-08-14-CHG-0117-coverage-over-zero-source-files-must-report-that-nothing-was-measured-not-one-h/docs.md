---
change: CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, quoting the `compute_exit_code` comment that already described this
hazard — the gate was defended and the display was not.

## Behaviour change

| tree | before | after |
|---|---|---|
| no source files found | `0/0 (100%)` + two affirmative lines | `0/0 (no source files to measure)` + the likely cause |
| has source files | percentages + affirmative lines | unchanged |
| `--require-coverage` gate | fails on zero source files | unchanged |

Anything scraping `File coverage: N/M (P%)` will no longer find a percentage in the
zero-source case. That is the point: there was never a percentage to report, and 100% was
the wrong answer rather than an unhelpful one.

## No new public API
