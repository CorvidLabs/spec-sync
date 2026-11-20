---
kind: evidence
id: 2026-11-20-on-linux-at-0187d498-cargo-clippy-all-targets-d-warnings
aliases: []
claim: "On Linux at 0187d498: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3126 passed, 0 failed, 2 ignored and integration 700 passed, 0 failed. The first Linux run at 5a449fb0 found two defects macOS could not show, both fixed before this run: fsync on the O_PATH folder handle failed every durable journal write (fixed in 1a5adcdd), and a clean cargo test writes no dep-info beside the binary, so the scans could not find their file list (fixed in 93e06a8e by reading the deps/ dep-info of the exact binary). Not run: x86_64, the Ubuntu image CI uses, hosted CI."
result: passed
evidence_kind: product-test
revision: 0187d49824c69d81cffd51d68b24bfd28afc6765
content: 2345cacabed0e5d381e05f4a2424e20aee042726356f24191ed679d284189a66
environment: Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 0187d498, as a normal user with jq, through one-lane
fingerprint: sha256:f9606bbc736b09e76201a6eb3fb4339416bf2edc1e7443b3642803e410cc291c
external: ext-8666b4
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements:
  - REQ-util-006
  - REQ-util-007
  - REQ-util-008
  - REQ-change-162
  - REQ-hooks-009
  - REQ-validator-052
contracts: []
limitations: []
observed: 2026-11-20T02:51:25Z
actor:
  - agent: claude-code
    terminal: t5r
---
**On Linux at 0187d498: cargo clippy --all-targets -D warnings passes; cargo test --no-fail-fast passes with unit 3126 passed, 0 failed, 2 ignored and integration 700 passed, 0 failed. The first Linux run at 5a449fb0 found two defects macOS could not show, both fixed before this run: fsync on the O_PATH folder handle failed every durable journal write (fixed in 1a5adcdd), and a clean cargo test writes no dep-info beside the binary, so the scans could not find their file list (fixed in 93e06a8e by reading the deps/ dep-info of the exact binary). Not run: x86_64, the Ubuntu image CI uses, hosted CI.**

Recorded as passed in Linux aarch64, local rust:1.89-bookworm container (Debian 12), a clean copy of 0187d498, as a normal user with jq, through one-lane at revision 0187d49824c6, observed 2026-11-20T02:51:25Z.
