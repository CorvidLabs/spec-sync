---
change: CHG-0112-a-tree-with-source-and-no-specs-must-show-its-coverage-number-and-must-not-pass
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, stating that the branch already carried a comment requiring `--strict` to
gate here and that only the other two gates delivered it.

## Behaviour change

| tree | before | after |
|---|---|---|
| has source, no specs, bare `check` | no coverage figure | coverage printed, exit 0 |
| has source, no specs, `--strict` | exit 0 | **exit 1** |
| no source, no specs | exit 0 | unchanged |
| any project with at least one spec | — | unchanged |

Projects adopting spec-sync will see `check --strict` fail until the first spec exists. That
is the intended signal: strict validation of a tree that was never measured should not
report clean. Bare `check` is unchanged and remains the right command during setup.

## No new public API
