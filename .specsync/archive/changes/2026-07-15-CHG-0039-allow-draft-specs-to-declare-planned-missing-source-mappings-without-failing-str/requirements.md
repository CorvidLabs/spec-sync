---
change: CHG-0039-allow-draft-specs-to-declare-planned-missing-source-mappings-without-failing-str
artifact: requirements
---

# Requirements

## REQ-CHG-0039-001 — Planned draft mappings

SpecSync SHALL treat a safe normalized missing path in a draft spec as a planned mapping by default.

Acceptance criteria:

- Strict validation emits a clear planned-mapping notice and succeeds when no other gate fails.
- Planned missing paths do not enter file or LOC coverage denominators.
- Active, review, stable, deprecated, and unknown-status specs retain missing-file errors.
- Creating the planned file transitions it to normal validation and coverage without editing the spec.
- Redundant dot segments cannot create a mismatch between spec mappings and discovered coverage paths.

## REQ-CHG-0039-002 — Strict opt-in and structural safety

Configuration SHALL provide a default-false `require_draft_files` option (`requireDraftFiles` in legacy JSON) that restores missing-file errors for drafts.

Acceptance criteria:

- Canonical TOML reads and round-trips `require_draft_files = true`; legacy JSON reads `requireDraftFiles`.
- Existing draft-mapped files retain UTF-8, containment, duplicate-ownership, and other structural validation.
- Incremental validation compares changed specs against unchanged cached spec owners.
- Absolute, traversal, and escaping symlink paths never become planned mappings.
- Text, JSON, Markdown, and GitHub output expose planned mappings without counting them as warnings.
- Canonical config, commands, comment, output, types, and validator specs document the implemented interfaces.
