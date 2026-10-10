---
id: split-command-integration-regressions-into-cohesive-topical-test-modules
state: implementing
type: refactor
base_commit: 815690d1b01776915d93d11ff9f814efeced1a3d
---

# Split command integration regressions into cohesive topical test modules

## Intent

Split command integration regressions into cohesive topical test modules

## Affected Canonical Specs

- `cli`

## Acceptance Criteria

- Reduce the 5496-line command integration test file to a small registered facade and cohesive topical test files below 1000 lines where sensible; preserve every existing test/helper body, assertion, fixture and platform/debug attribute, retain all 120 Linux command test cases and platform-only declarations, preserve effective subprocess filters and root integration registration, change no production code or user-visible behavior, and document the test layout with faithful lifecycle evidence

## No-spec Rationale

Not applicable
