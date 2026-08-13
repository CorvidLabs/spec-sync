---
change: CHG-0111-report-the-checks-that-frontmatter-parse-failure-prevented-instead-of-printing-a
artifact: docs
---

# Docs

## CHANGELOG

One `Fixed` entry covering all four lines, and stating plainly that the section claim was
**false** rather than merely vacuous, with the control that demonstrates it.

## Behaviour change

Only the report changes. Exit status is unaffected — invalid frontmatter was an error before
and remains one, so this never let a bad spec through a gate. A project whose specs all
parse sees no difference at all.

## No new public API
