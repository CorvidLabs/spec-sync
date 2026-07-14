---
change: CHG-0036-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: design
---

# Design

Add `require_draft_files: bool` to `SpecSyncConfig`, defaulting to false. A missing mapping is planned only when the parsed status is exactly `draft`, the opt-in is false, and the path is lexically project-relative. Review and later states remain enforcement states.

Add validation notices as a separate result channel. Notices render with informational/planned language in text and their own structured collection in JSON, Markdown, and GitHub output; they never increment warning counts, so `--strict` remains truthful.

Coverage continues to enumerate real files from configured source directories. Planned missing paths are never synthesized into numerator or denominator data. Existing mappings continue through containment and readability checks. A project-wide ownership index reports real files mapped by multiple specs, including drafts, without treating two nonexistent plans as current ownership.
