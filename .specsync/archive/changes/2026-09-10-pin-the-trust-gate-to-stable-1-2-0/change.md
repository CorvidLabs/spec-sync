---
id: pin-the-trust-gate-to-stable-1-2-0
state: archived
type: operations
base_commit: 15e53cab4ea1271f6c566f8372f4b584c94a07b8
---

# Pin the Trust gate to stable 1.2.0

## Intent

Pin the Trust gate to stable 1.2.0

## Affected Canonical Specs

- None

## Acceptance Criteria

- The Trust gate step in .github/workflows/trust.yml resolves CorvidLabs/trust@fcc889f54d8b4892a81af463c5a0250e2be66fc5 (v1.2.0, the peeled stable release commit) instead of e0272543 (v1.2.0-rc.4). The runner-local file:// SpecSync mirror still works unchanged: Trust downloads specsync-linux-x86_64.tar.gz and its .sha256 from specsync-download-base-url, revalidates the checksum, and gates on the pull request's own freshly built binary rather than a published release. specsync-version stays 6.0.0. The hosted trust check passes.

## No-spec Rationale

This changes only the pinned ref of the Trust action in CI. No canonical spec text changes because no module contract, public API, or behavior is affected.
