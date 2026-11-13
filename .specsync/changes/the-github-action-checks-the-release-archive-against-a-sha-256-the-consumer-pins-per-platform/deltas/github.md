---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: delta
module: github
---

## ADDED

### REQUIREMENT REQ-github-031

The GitHub Action SHALL accept an `expected-sha256` input that holds one `<platform>=<sha256>`
line per platform, such as `linux-x86_64=<64 hex>`. When the input is set, the Action SHALL
refuse a downloaded release archive before extracting it when the line for its platform is
missing, appears more than once, is malformed, or names a different SHA-256 than the archive has.
The release's own `.sha256` sidecar SHALL still be checked. With the input empty, the Action
SHALL install exactly as before (#476).

#### Acceptance Criteria

- A matching digest installs; a different digest, a platform with no line, a platform named twice
  and a malformed line each fail before extraction, and the message names the cause.
- Surrounding spaces and other platforms' lines are ignored.
- `.github/scripts/test-verify-release-digest.sh` covers each case, through Fledge
  (`release-digest-test`) and CI.
