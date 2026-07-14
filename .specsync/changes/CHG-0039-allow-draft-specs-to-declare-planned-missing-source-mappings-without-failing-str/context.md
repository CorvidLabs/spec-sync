---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: context
---

# Context

SpecSync already treats draft specifications as pre-implementation contracts by skipping required-section and export drift checks. File existence is still enforced before that status-sensitive behavior, so a draft cannot record a future source mapping without creating a premature stub file or failing strict validation.

The change distinguishes a safe normalized missing draft mapping from a current source file. Missing draft mappings become explicit non-failing notices and remain absent from coverage. Existing files and unsafe paths continue through normal structural, ownership, readability, and coverage validation.

The prior conflicted PR review identified additional contract and correctness requirements: config, command, comment, and output specs must document the new interfaces; incremental validation must compare changed specs with cached unchanged owners; and redundant dot segments must not break the transition from a planned path to ordinary coverage. Those findings are part of this fresh definition.
