---
change: CHG-0110-stop-printing-green-result-lines-for-checks-that-could-not-run-when-frontmatter
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry: when a spec's frontmatter cannot be parsed, `specsync check` no longer
prints `✓ All source files exist`, `✓ All required sections present`, or
`✓ All dependency specs exist` for checks that never ran. Each is reported as skipped.

Worth stating plainly in the entry that the section line was **false**, not merely vacuous:
the same body with valid frontmatter reports five missing sections.

## Behaviour change

Only the report changes. Exit status is unaffected — invalid frontmatter was an error
before and remains one. A project whose specs all parse sees no difference at all.

## No new public API
