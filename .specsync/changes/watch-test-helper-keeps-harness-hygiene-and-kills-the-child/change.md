---
id: watch-test-helper-keeps-harness-hygiene-and-kills-the-child
state: implementing
type: bug_fix
base_commit: 184bf8893a20289a2e044a137a11d4828a558761
---

# Watch test helper keeps harness hygiene and kills the child

## Intent

watch test helper keeps harness hygiene and kills the child

## Affected Canonical Specs

- None

## Acceptance Criteria

- watch_until inherits the specsync() env hygiene and kills the child on assert/panic paths too

## No-spec Rationale

test-harness hardening from review; no contract change
