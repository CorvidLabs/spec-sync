---
id: classify-the-archive-commit-that-finalize-records
state: implementing
type: bug_fix
base_commit: efa8046bc3bb3d70f006304b7ceef3fa285725cb
---

# Classify the archive commit that finalize records

## Intent

Classify the archive commit that finalize records

## Affected Canonical Specs

- None

## Acceptance Criteria

- The integration test classifies the archive commit that change finalize records, and that commit is the child of the implementation commit.

## No-spec Rationale

The test follows the command. Finalize already commits the archive, so the spec text is unchanged.
