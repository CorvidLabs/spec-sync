---
id: pin-the-trust-gate-to-1-2-1
state: draft
type: operations
base_commit: cddc39e478dcc1f111940a3cfb02134bba9804cc
---

# Pin the Trust gate to 1.2.1

## Intent

Pin the Trust gate to 1.2.1

## Affected Canonical Specs

- None

## Acceptance Criteria

- The Trust gate step in .github/workflows/trust.yml resolves CorvidLabs/trust@dd52a7a90ffbc1d2b1030e37007c18274f3d96bc (v1.2.1, the peeled commit of the annotated v1.2.1 tag) instead of fcc889f54d8b4892a81af463c5a0250e2be66fc5 (v1.2.0). The runner-local file:// SpecSync mirror still works unchanged: Trust downloads specsync-linux-x86_64.tar.gz and its .sha256 from specsync-download-base-url, revalidates the checksum, and gates on the pull request's own freshly built binary. specsync-version stays 6.0.0. Nested Augur and Attest install their prebuilt Linux binaries (augur-linux-x86_64, attest-linux-x86_64) instead of building Swift from source, and the hosted trust check passes.

## No-spec Rationale

This changes only the pinned ref of the Trust action in CI. No canonical spec text changes because no module contract, public API, or behavior is affected.
