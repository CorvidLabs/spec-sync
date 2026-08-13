---
change: CHG-0114-a-semantic-delta-section-body-may-contain-subheadings-so-scaffolded-specs-can-be
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry, stating that the tool generated specs its own lifecycle refused, and that
the parser was fixed rather than the generator so existing projects are repaired too.

## Behaviour change

| delta shape | before | after |
|---|---|---|
| item body containing subheadings | rejected | accepted as content |
| subheading before any item | rejected | unchanged — still rejected |
| every previously valid delta | — | unchanged |

Projects that already converted their spec subheadings to bold labels to work around this
need do nothing; both forms are accepted.

## No new public API
