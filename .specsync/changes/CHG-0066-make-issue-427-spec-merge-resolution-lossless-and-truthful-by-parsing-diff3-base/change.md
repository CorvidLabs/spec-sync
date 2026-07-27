---
id: CHG-0066-make-issue-427-spec-merge-resolution-lossless-and-truthful-by-parsing-diff3-base
state: accepted
type: bug_fix
base_commit: a55a744a97b46f5c01068d6fe4433990d4fef942
---

# Make issue 427 spec merge resolution lossless and truthful by parsing diff3 bases, preserving both side labels, selecting the maximum numeric version, unioning list fields, leaving conflicting table rows and scalar fields unresolved, and preserving all-or-nothing writes

## Intent

Make issue 427 spec merge resolution lossless and truthful by parsing diff3 bases, preserving both side labels, selecting the maximum numeric version, unioning list fields, leaving conflicting table rows and scalar fields unresolved, and preserving all-or-nothing writes

## Affected Canonical Specs

- `merge`

## Acceptance Criteria

- Diff3 base regions are excluded from resolved output and diagnostics report both ours and incoming labels.
- Frontmatter list values are unioned and sorted while version uses the maximum numeric value.
- Conflicting same-key table rows and divergent or non-numeric scalar fields remain unresolved.
- One-sided scalar fields, YAML null-versus-list disagreements, table headers or separators inside or immediately after a hunk, and nested frontmatter mappings remain unresolved.
- Only exact standard conflict markers are accepted; malformed, orphan, duplicate, nested, incomplete, and lookalike markers remain manual.
- Reconstructed output must contain valid YAML frontmatter without duplicate keys and must retain every required field, a valid status, and non-empty files.
- Unreadable all-files candidates remain explicit Manual findings and failed Git discovery is a safe no-op.
- CRLF line endings and final-newline form are preserved.
- Any unresolved region prevents every write to the file; diagnostics never claim HEAD won when incoming content was selected and distinguish dry-run candidates from persisted resolutions.

## No-spec Rationale

Not applicable
