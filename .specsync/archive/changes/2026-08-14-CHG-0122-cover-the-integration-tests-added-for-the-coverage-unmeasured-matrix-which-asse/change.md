---
id: CHG-0122-cover-the-integration-tests-added-for-the-coverage-unmeasured-matrix-which-asse
state: archived
type: bug_fix
base_commit: 62cb051178e29c642b29df39f9a0daefbd6ba888
---

# Cover the integration tests added for the coverage-unmeasured matrix, which assert behaviour already specified by CHG-0121 and introduce no new spec text

## Intent

Cover the integration tests added for the coverage-unmeasured matrix, which assert behaviour already specified by CHG-0121 and introduce no new spec text

## Affected Canonical Specs

- `output`

## Acceptance Criteria

- The three test files are covered by a change so the lifecycle gate passes, without asserting any behaviour beyond what CHG-0121 already specified. The matrix runs every coverage-reporting command through every format plus both MCP surfaces and asserts none reports a percentage for an unmeasurable tree, and that a healthy project is unchanged across the same matrix. No module gains, loses, or alters a requirement.

## No-spec Rationale

These files are the regression for CHG-0121's requirements, not new behaviour: tests/integration/coverage_unmeasured.rs is the every-command-every-format matrix asserting REQ-output-005, REQ-mcp-004, REQ-cmd-check-009, REQ-cmd-coverage-002, REQ-cmd-report-002, REQ-cmd-deps-002 and REQ-comment-003, and the other two files register and adjust it. No module's specified behaviour changes, so there is no spec text to write.
