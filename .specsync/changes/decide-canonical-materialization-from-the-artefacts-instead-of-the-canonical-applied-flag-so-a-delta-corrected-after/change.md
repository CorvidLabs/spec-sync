---
id: decide-canonical-materialization-from-the-artefacts-instead-of-the-canonical-applied-flag-so-a-delta-corrected-after
state: implementing
type: bug_fix
base_commit: 7df407728de3ac6458ef8807e79bbadb51da3324
---

# Decide canonical materialization from the artefacts instead of the canonical_applied flag so a delta corrected after review and re-approved reaches the canonical spec with its version bump and Change Log row

## Intent

Decide canonical materialization from the artefacts instead of the canonical_applied flag so a delta corrected after review and re-approved reaches the canonical spec with its version bump and Change Log row

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A delta corrected after review and re-approved is materialized by the next change check, and the superseded wording does not survive; a canonical spec carrying the change's contract text without the version bump or a Change Log row naming the change receives both; a byte-identical re-approval writes nothing at all, leaving the spec byte for byte, one version bump and one Change Log row; re-materialization does not refuse a removal its own earlier run performed, while every first-run application refusal still fires; the refusal for a drifted delta names change check after change approve.

## No-spec Rationale

Not applicable
