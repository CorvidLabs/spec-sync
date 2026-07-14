---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: context
---

# Context

SpecSync already treats draft specifications as pre-implementation contracts by skipping required-section and export drift checks. File existence is still enforced before that status-sensitive behavior, so a draft cannot record a future source mapping without creating a placeholder file or failing strict validation.

The change distinguishes a safe, missing draft mapping from a current source file. Missing draft mappings become explicit non-failing notices and remain absent from coverage. Existing files and unsafe paths continue through normal structural, ownership, readability, and coverage validation.
