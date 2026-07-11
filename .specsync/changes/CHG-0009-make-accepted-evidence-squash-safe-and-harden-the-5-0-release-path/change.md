---
id: CHG-0009-make-accepted-evidence-squash-safe-and-harden-the-5-0-release-path
state: accepted
type: bug_fix
base_commit: 884ad33b2158e9efca2f31d4798c1b6f27db8801
---

# Make accepted evidence squash-safe and harden the 5.0 release path

## Intent

Make accepted evidence squash-safe and harden the 5.0 release path

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- Post-squash accepted evidence remains valid for root and nested projects only when the accepted workspace is already integrated unchanged on the remote default branch; unintegrated or changed evidence fails closed; accepted workspaces archive after squash; digest evidence is collision-safe and file-kind-aware; release tags trigger only exact semantic versions; the Action defaults to the 5.0.0 binary; the repository lifecycle stamp is 5.0.0; the published crate excludes repository-only assets; all local and CI gates pass

## No-spec Rationale

Not applicable
