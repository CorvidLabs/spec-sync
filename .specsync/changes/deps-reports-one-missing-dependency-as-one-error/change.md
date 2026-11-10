---
id: deps-reports-one-missing-dependency-as-one-error
state: implementing
type: bug_fix
base_commit: 5d20c7fb871e450e133e9b6c93422a0cfb978c61
---

# Deps reports one missing dependency as one error

## Intent

deps reports one missing dependency as one error

## Affected Canonical Specs

- `deps`

## Acceptance Criteria

- a spec whose depends_on names a module with no spec yields exactly one deps error for it, the 'Dependency spec not found' verdict check gives, for bare and path entries; missing_deps still names the edge; the regression_w1 pin now expects one occurrence

## No-spec Rationale

The deps acceptance criterion already says validate_deps reports an error for each declared dependency whose target has no spec; one missing dependency produced two errors. This makes the count match the existing requirement
