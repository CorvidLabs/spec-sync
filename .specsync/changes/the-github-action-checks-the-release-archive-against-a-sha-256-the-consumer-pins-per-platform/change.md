---
id: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
state: draft
type: feature
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The GitHub Action checks the release archive against a SHA-256 the consumer pins per platform

## Intent

The GitHub Action checks the release archive against a SHA-256 the consumer pins per platform

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- with expected-sha256 set, a matching archive installs and a mismatched digest, a missing or duplicated platform line, or a malformed line fails before extraction naming the cause; with it empty the Action installs as before; the .sha256 sidecar is still checked

## No-spec Rationale

Not applicable
