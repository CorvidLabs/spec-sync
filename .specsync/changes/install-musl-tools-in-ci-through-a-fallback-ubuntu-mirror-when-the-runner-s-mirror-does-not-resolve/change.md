---
id: install-musl-tools-in-ci-through-a-fallback-ubuntu-mirror-when-the-runner-s-mirror-does-not-resolve
state: approved
type: bug_fix
base_commit: ad8544e9da79725f499b38ae4d4b3a052be5f807
---

# Install musl-tools in CI through a fallback Ubuntu mirror when the runner's mirror does not resolve

## Intent

Install musl-tools in CI through a fallback Ubuntu mirror when the runner's mirror does not resolve

## Affected Canonical Specs

- None

## Acceptance Criteria

- The ci.yml musl install step runs .github/scripts/install-musl-tools.sh and keeps its per-target env exports unchanged; with a stubbed apt-get whose runner mirror does not resolve, the script installs musl-tools through the fallback mirror list on the x86_64 mirror-list, arm64 ports and one-line sources layouts, leaves third-party sources alone, passes Acquire::Retries and Timeout to every apt-get call, and ends in ::error:: when every mirror fails or no Ubuntu source can be repointed; validate-action and the fledge musl-install-test task run that test

## No-spec Rationale

CI infrastructure only: the musl release-baseline jobs keep building and checking the same archives (REQ-github-025 unchanged); only how the runner obtains musl-tools changes, so no canonical spec or requirement changes
