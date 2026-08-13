---
change: CHG-0115-a-fix-request-that-could-not-be-applied-must-be-reported-not-reported-as-succes
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, naming the discarded `if let Ok(())` as the cause and stating that
`--fix --dry-run` is deliberately unaffected.

## Behaviour change

| invocation | before | after |
|---|---|---|
| `--fix`, spec not writable | exit 0, silent | exit 1, path and OS error on stderr |
| `--fix`, spec not readable | skipped silently | reported, exit 1 |
| `--fix`, writable spec | repaired, exit 0 | unchanged |
| `--fix --dry-run`, not writable | exit 0 | unchanged |

A project whose specs are writable sees no difference.

## No new public API
