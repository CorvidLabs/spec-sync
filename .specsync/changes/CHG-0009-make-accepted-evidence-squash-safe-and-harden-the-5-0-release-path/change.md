---
id: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
state: implementing
type: bug_fix
base_commit: 884ad33b2158e9efca2f31d4798c1b6f27db8801
---

# Make accepted evidence squash-safe and harden the 5.0 release path

## Intent

Make accepted evidence squash-safe and harden the 5.0 release path

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Post-squash accepted evidence remains valid only when the accepted workspace is already integrated unchanged on the remote default branch; unintegrated or changed evidence fails closed; accepted workspaces archive after squash; release tags trigger only exact semantic versions; the Action defaults to the 5.0.0 binary; all local and CI gates pass

## No-spec Rationale

Not applicable
