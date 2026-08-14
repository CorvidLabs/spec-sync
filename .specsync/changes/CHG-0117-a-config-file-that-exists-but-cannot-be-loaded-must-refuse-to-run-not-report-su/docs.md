---
change: CHG-0117-a-config-file-that-exists-but-cannot-be-loaded-must-refuse-to-run-not-report-su
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, stating that the fallback sites already carried comments requiring this
and did not deliver it.

## Behaviour change

| project | before | after |
|---|---|---|
| config exists, cannot be loaded | built-in defaults, stderr warning, **success** | **refused**, exit 1, file named |
| config valid | enforced | unchanged |
| no config file | built-in defaults | unchanged |

A project with a malformed config will now fail where it previously passed. That is the
point: it was passing against rules it had not loaded.

## No new public API

`SpecSyncConfig` gains a runtime-only field, not serialized and not part of the config file
schema.
