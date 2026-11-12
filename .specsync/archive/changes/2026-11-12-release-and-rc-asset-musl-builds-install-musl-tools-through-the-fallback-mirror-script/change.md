---
id: release-and-rc-asset-musl-builds-install-musl-tools-through-the-fallback-mirror-script
state: archived
type: bug_fix
base_commit: 4eed6e8dccc14ecf9fccf76b549d7e2fda8390e7
---

# Release and RC-asset musl builds install musl-tools through the fallback-mirror script

## Intent

Release and RC-asset musl builds install musl-tools through the fallback-mirror script

## Affected Canonical Specs

- None

## Acceptance Criteria

- The release.yml build job and the rc-assets.yml build job install musl-tools for their musl rows by running .github/scripts/install-musl-tools.sh, as ci.yml does, and keep their per-target env exports unchanged; no bare apt-get install of musl-tools remains in ci.yml, release.yml or rc-assets.yml, which test-install-musl-tools.sh asserts; no job in either release workflow runs augur or a CorvidLabs/trust gate, so none of them gets the ubuntu-24.04 pin

## No-spec Rationale

Release workflow infrastructure only: the release and RC-asset musl builds keep producing and checking the same archives (REQ-github-025 unchanged); only how those runners obtain musl-tools changes, matching ci.yml since #1061, so no canonical spec or requirement changes
