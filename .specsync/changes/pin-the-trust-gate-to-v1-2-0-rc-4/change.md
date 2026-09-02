---
id: pin-the-trust-gate-to-v1-2-0-rc-4
state: implementing
type: bug_fix
base_commit: ac796b8eadd3092283093bbea331ec2d3494b527
---

# Pin the Trust gate to v1.2.0-rc.4

## Intent

pin the Trust gate to v1.2.0-rc.4

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- The Trust workflow pins CorvidLabs/trust to e0272543 (v1.2.0-rc.4) while specsync-version stays 6.0.0 against the runner-local file:// mirror.

## No-spec Rationale

Trust action pin only (v1.1.1 → v1.2.0-rc.4); the dogfood 6.0.0 file:// mirror is unchanged
