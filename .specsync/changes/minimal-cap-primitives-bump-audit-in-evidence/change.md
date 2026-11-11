---
id: minimal-cap-primitives-bump-audit-in-evidence
state: implementing
type: bug_fix
base_commit: 9f85c34f3fe08ea4e28074c228d6003725e2c295
---

# Minimal cap-primitives bump, audit in evidence

## Intent

minimal cap-primitives bump, audit in evidence

## Affected Canonical Specs

- None

## Acceptance Criteria

- Cargo.lock's diff vs main is exactly cap-primitives' version and checksum; cargo audit exits 0 at the head; tasks.md ticks only completed items

## No-spec Rationale

redo of the lockfile edit per review: only cap-primitives' version+checksum change; cargo audit output is part of the evidence
