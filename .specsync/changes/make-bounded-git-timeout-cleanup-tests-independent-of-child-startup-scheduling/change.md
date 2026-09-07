---
id: make-bounded-git-timeout-cleanup-tests-independent-of-child-startup-scheduling
state: implementing
type: feature
base_commit: 4908a238a0030a451b3647c2a708403704545157
---

# Make bounded Git timeout cleanup tests independent of child startup scheduling

## Intent

Make bounded Git timeout cleanup tests independent of child startup scheduling

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Timeout cleanup regression obtains the spawned child PID in the parent without a child-written readiness file; it still asserts deadline failure and child termination/reaping with a blocked stdin payload; deterministic delayed-start and cleanup negative controls distinguish the race from cleanup defects; repeated targeted runs and required verification pass without changing production deadlines or release gates.

## No-spec Rationale

Not applicable
