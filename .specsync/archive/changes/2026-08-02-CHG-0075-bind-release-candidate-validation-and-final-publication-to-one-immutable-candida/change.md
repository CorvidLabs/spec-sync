---
id: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
state: archived
type: operations
base_commit: 6a0956fbe53669aa9a7bc564fd472e0952a70f2a
---

# Bind release-candidate validation and final publication to one immutable candidate SHA across Ubuntu macOS and Windows

## Intent

Bind release-candidate validation and final publication to one immutable candidate SHA across Ubuntu macOS and Windows

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- Ordinary development and product pull requests use Ubuntu as the only integration platform; an RC branch is frozen by an immutable vX.Y.Z-rc.N marker bound to one candidate commit; Ubuntu macOS and Windows run the same named release-candidate Fledge lane against that exact commit and publish SHA-bound evidence; changing the candidate requires a new RC marker; final vX.Y.Z tag creation and release uploads are refused unless every required platform result is green for that unchanged candidate commit, and the final tag is created only after the cross-platform gate succeeds.

## No-spec Rationale

Not applicable
