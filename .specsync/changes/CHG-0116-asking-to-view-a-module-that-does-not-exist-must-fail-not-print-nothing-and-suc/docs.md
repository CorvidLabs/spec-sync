---
change: CHG-0116-asking-to-view-a-module-that-does-not-exist-must-fail-not-print-nothing-and-suc
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry covering both directions — the unknown-module filter and the ignored
render failure — with the suggestion output shown.

## Behaviour change

| invocation | before | after |
|---|---|---|
| `--spec <unknown>` | 0 bytes, exit 0 | named error + suggestion, exit 1 |
| `--spec <existing>` | renders | unchanged |
| no filter | renders all | unchanged |
| a spec that fails to render | error printed, exit 0 | error printed, exit 1 |

Scripts that relied on `view --spec` exiting 0 for a name that does not exist will now see a
failure. That is the point: the previous behaviour was indistinguishable from success.

## No new public API
