---
id: CHG-0133-extract-the-change-module-s-tests-into-their-own-file-so-the-file-that-manufactu
state: implementing
type: refactor
base_commit: 15d2b20d11ad21a597007624c8ae6fded4429836
---

# Extract the change module's tests into their own file so the file that manufactures the sibling-site defect can be read, without altering a single test

## Intent

Extract the change module's tests into their own file so the file that manufactures the sibling-site defect can be read, without altering a single test

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- The change module's tests move to their own file with no test added, removed, renamed or edited: the test-function count and the passing count are identical before and after. The tests still reach every private item, because the module stays inline via a path attribute rather than becoming a sibling module. The test-only helpers and fault-injection hooks that production code paths reference remain where they are, since they are not test code that merely lives near production code. No product behaviour changes, and the drill board is unchanged.

## No-spec Rationale

Not applicable
