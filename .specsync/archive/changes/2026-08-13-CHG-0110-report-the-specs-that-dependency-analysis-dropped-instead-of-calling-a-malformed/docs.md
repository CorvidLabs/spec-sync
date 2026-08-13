---
change: CHG-0110-report-the-specs-that-dependency-analysis-dropped-instead-of-calling-a-malformed
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry: `deps` no longer reports a malformed dependency graph as valid. It now
surfaces the parser's frontmatter errors with the same wording `check` uses, and reports
specs dropped from the analysis rather than skipping them silently.

## Behaviour change

A project whose specs all parse sees no difference. A project carrying a malformed spec sees
`deps` change from exit 0 with a green line to exit 1 naming the file and the defect — which
is the answer `check` was already giving for the same input.

## No new public API
