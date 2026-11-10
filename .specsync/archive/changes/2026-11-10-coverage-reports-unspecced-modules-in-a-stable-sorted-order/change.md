---
id: coverage-reports-unspecced-modules-in-a-stable-sorted-order
state: archived
type: bug_fix
base_commit: 4fe1e1ed93e00102d1eda73d3cf3e19d5cd53c7f
---

# Coverage reports unspecced modules in a stable sorted order

## Intent

coverage reports unspecced modules in a stable sorted order

## Affected Canonical Specs

- `validator`

## Acceptance Criteria

- compute_coverage_checked returns unspecced_modules sorted, so coverage text and JSON, PR comments and generate's spec order are identical across runs of the same command on the same tree, whatever HashMap order config.modules and manifest.modules iterate in

## No-spec Rationale

No requirement changes: coverage already reports unspecced modules; this only makes their order reproducible, as find_spec_files and the uncovered-file list already are
