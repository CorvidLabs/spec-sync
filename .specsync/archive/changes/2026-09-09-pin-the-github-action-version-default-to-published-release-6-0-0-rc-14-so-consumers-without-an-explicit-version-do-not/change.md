---
id: pin-the-github-action-version-default-to-published-release-6-0-0-rc-14-so-consumers-without-an-explicit-version-do-not
state: archived
type: bug_fix
base_commit: d0da6075f8f383c474915dd8d2b97dfd9e8b776f
---

# Pin the GitHub Action version default to published release 6.0.0-rc.14 so consumers without an explicit version do not 404 on missing v6.0.0 (#628)

## Intent

Pin the GitHub Action version default to published release 6.0.0-rc.14 so consumers without an explicit version do not 404 on missing v6.0.0 (#628)

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- action.yml version default is 6.0.0-rc.14 (no v prefix); description names a published release candidate and does not imply stable 6.0.0 exists; site GitHub Action inputs table default matches; validate-release-version.py accepts Action/docs default 6.0.0-rc.14 during the pre-stable window while Cargo.toml remains 6.0.0; README immutable pin examples may stay at package version; python3 -S .github/scripts/validate-release-version.py passes; gh release view v6.0.0-rc.14 shows assets; specs/github REQ-github-002 covers the candidate-window pin to a published release with assets

## No-spec Rationale

Not applicable
