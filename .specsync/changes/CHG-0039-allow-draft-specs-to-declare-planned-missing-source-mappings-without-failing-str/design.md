---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: design
---

# Design

Add `require_draft_files: bool` to `SpecSyncConfig`, defaulting to false. A missing mapping is planned only when the parsed status is exactly `draft`, the opt-in is false, and the path is a safe normalized project-relative path. Review and later states remain enforcement states.

Add validation notices as a separate result channel. Notices render with informational/planned language in text and their own structured collection in JSON, Markdown, and GitHub output; they never increment warning counts, so `--strict` remains truthful.

Coverage continues to enumerate real files from configured source directories. Planned missing paths are never synthesized into numerator or denominator data. Existing mappings continue through containment and readability checks.

Ownership indexing receives the complete discovered spec inventory independently from the subset selected for incremental validation. It records each real normalized file once per spec and performs direct lookups only for files owned by the currently validated spec, retaining linear behavior without missing cached owners.
