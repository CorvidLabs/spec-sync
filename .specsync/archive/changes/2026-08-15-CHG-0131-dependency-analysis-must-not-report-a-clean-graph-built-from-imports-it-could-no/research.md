---
change: CHG-0131-dependency-analysis-must-not-report-a-clean-graph-built-from-imports-it-could-no
artifact: research
---

# Research

This is the seventh instance in this campaign of a fix containing the defect it
fixes, and the second where an adversarial reviewer caught it before merge
rather than a later sweep. The pattern is specific enough to name: a fix that
adds a COLLECTION step without adding the corresponding FAILURE state will drop
what it cannot handle, because the absent value is indistinguishable from the
empty one.

`Option` is the usual vehicle. `filter_map` over `Option` is the usual verb.

Two residuals recorded rather than left to be found:

- Bash is classified as having no import concept, though `source` names another
  file. A judgment call, one match arm to flip.
- Unspecced project code now shows up as unresolved. That is correct — it IS an
  import we cannot map — but it is new output on repos with partial coverage.

Pre-existing drift noticed while working, not introduced here:
`specs/deps/requirements.md:26` says unreadable source files are "skipped
silently, not treated as errors", which contradicts invariant 7 added in July.
