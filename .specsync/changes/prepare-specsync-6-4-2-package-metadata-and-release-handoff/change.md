---
id: prepare-specsync-6-4-2-package-metadata-and-release-handoff
state: approved
type: operations
base_commit: 71f1867d7ddcf2b075b49c421d2f87ea0b760be1
---

# Prepare SpecSync 6.4.2 package metadata and release handoff

## Intent

Prepare SpecSync 6.4.2 package metadata and release handoff

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo metadata and the self-built CI and Trust pins select 6.4.2. Action examples name the explicit candidate version. The installed binary default and current-install guidance select published 6.4.1 until publication. The changelog and release handoff account for user-visible fixes actually merged since v6.4.1. The version validator, workflow-pin validator, release-candidate contract tests and project gates pass on the prepared revision.

## No-spec Rationale

The patch release packages merged bug fixes using the existing release contract. Its changes cover version metadata, examples, verification fixtures and operator documentation.
