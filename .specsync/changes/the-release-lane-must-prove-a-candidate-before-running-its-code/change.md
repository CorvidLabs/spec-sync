---
id: the-release-lane-must-prove-a-candidate-before-running-its-code
state: implementing
type: bug_fix
base_commit: 4d12c7a567bcacca3f37543dce3e9a04c5ecc7a1
---

# The release lane must prove a candidate before running its code

## Intent

the release lane must prove a candidate before running its code

## Affected Canonical Specs

- None

## Acceptance Criteria

- Three open high-severity CodeQL cache-poisoning alerts sit in release.yml, and the lane has never executed, so the first time these steps run for real will be the 6.0 RC. In the validate job, cargo metadata runs the candidate's own manifests and build scripts before anything has established that the candidate is an ancestor of origin/main; the ancestor check is what makes the checkout trustworthy and it runs last. In the build jobs, rust-cache writes cache entries from a tree checked out at an operator-supplied candidate on a run carrying default-branch privileges, which is the actual poisoning vector. Done when: the ancestor proof runs before any cargo invocation; cache entries are restored but never saved from a candidate tree; and the checkout that authorize-release genuinely needs is kept with its reason recorded rather than removed on the mistaken belief that it is unused.

## No-spec Rationale

Workflow ordering and cache-save policy in the release lane; no module contract changes and no production source in scope
