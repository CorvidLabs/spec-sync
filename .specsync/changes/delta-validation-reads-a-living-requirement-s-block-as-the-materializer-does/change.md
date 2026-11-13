---
id: delta-validation-reads-a-living-requirement-s-block-as-the-materializer-does
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# Delta validation reads a living requirement's block as the materializer does

## Intent

Delta validation reads a living requirement's block as the materializer does

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- An ADDED requirement whose text is unchanged in the living requirements.md validates when a ## section follows it, and the applier reads the same block as applied; a different body under the same id is still refused as cannot add existing block (red then green)

## No-spec Rationale

Makes delta validation match REQ-change-048 and REQ-change-049: an ADDED requirement already present with the same content is applied, not a conflict. Validation now reads the living block the way the materializer does, ending at the next heading of level 3 or shallower. The change spec only lists the new test file.
