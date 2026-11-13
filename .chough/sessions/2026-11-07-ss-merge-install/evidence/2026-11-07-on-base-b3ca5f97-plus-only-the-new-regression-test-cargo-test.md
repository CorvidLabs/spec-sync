---
kind: evidence
id: 2026-11-07-on-base-b3ca5f97-plus-only-the-new-regression-test-cargo-test
aliases: [OBS-product-test-0677784587f48c2115ff3f2c]
claim: "On base b3ca5f97 plus only the new regression test, cargo test --bin specsync root_specs_dir_matches_contracts failed at root.spec.md with merge: unspecified; 1 test failed and 2898 were filtered out."
result: failed
evidence_kind: product-test
method: automated
revision: b3ca5f97aa9c07b09ab6bfd8d7eece28f2c30f79
content: 7a71b6c5f6022b7dee3c3daaf072d91d0aa14629cf6be890ed1cd7b00e083763
environment: local-macos-arm64
criteria: [MERGE-1]
requirements: [REQ-cmd-merge-003]
contracts: [specs/cmd_merge/cmd_merge.spec.md]
limitations:
  - Red-first evidence before the implementation fix; linked to existing REQ-cmd-merge-003 because REQ-cmd-merge-004 is not yet approved and materialized.
observed: 2026-11-07T03:27:35Z
---
**On base b3ca5f97 plus only the new regression test, cargo test --bin specsync root_specs_dir_matches_contracts failed at root.spec.md with merge: unspecified; 1 test failed and 2898 were filtered out.**

Recorded as failed in local-macos-arm64 at revision b3ca5f97aa9c, observed 2026-11-07T03:27:35Z.
