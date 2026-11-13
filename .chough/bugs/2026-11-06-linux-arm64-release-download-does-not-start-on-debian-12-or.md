---
kind: bug
id: 2026-11-06-linux-arm64-release-download-does-not-start-on-debian-12-or
aliases:
  - BUG-linux-arm64-release-download-does-not-start-on-debian-12-or-ubuntu-22-0-23eb0fe0f3eb7815d6dc3284
title: Linux ARM64 release download does not start on Debian 12 or Ubuntu 22.04
status: verified
classification: product-defect
scope: blocking
impact: Every Linux ARM64 user below glibc 2.39, including GitHub's ubuntu-22.04-arm runners using the Action; the x86_64 gnu download has the same defect
criteria: [INSTALL-1, INSTALL-1.b, INSTALL-1.c]
requirements: [REQ-github-025]
cases:
  - TC-github-linux-arm64-glibc-download-starts-on-baseline
  - TC-github-linux-arm64-musl-download-starts-anywhere
  - TC-github-linux-baseline-lockstep
evidence:
  - OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-c9cfc037fd1400b5587015c3
  - OBS-tc-github-linux-arm64-musl-download-starts-anywhere-4accc748063adeedf526da3e
  - OBS-tc-github-linux-baseline-lockstep-8d3fbcb70a07759eebbe4095
  - OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453
  - OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b
  - OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991
receipts:
  - id: 2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline
    aliases: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-c9cfc037fd1400b5587015c3
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: failed (exit 1)."
    result: failed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64-docker
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:3cdbbaec56f5a6d0be5a952eee3e0a02a928b48772dfa45ae34d753563616608
    case: TC-github-linux-arm64-glibc-download-starts-on-baseline
    case_digest: 6c1458891f6ec19102613f5b61efd0e208d94a14c0a036f4bc247c882825e908
    observed: 2026-11-06T04:04:33Z
    duration_seconds: 8
  - id: 2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere
    aliases: OBS-tc-github-linux-arm64-musl-download-starts-anywhere-4accc748063adeedf526da3e
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: failed (exit 2)."
    result: failed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64-docker
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:9652fecf70fb8afa26301511bb86637958fb5a77d18aa98583812bae674d8e2b
    case: TC-github-linux-arm64-musl-download-starts-anywhere
    case_digest: 9b63920d3e9575e64918c797332f155b597e92179b987c70f28ce5d1993f1ec9
    observed: 2026-11-06T04:04:39Z
    duration_seconds: 0
  - id: 2026-11-06-tc-github-linux-baseline-lockstep
    aliases: OBS-tc-github-linux-baseline-lockstep-8d3fbcb70a07759eebbe4095
    claim: "Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: failed (exit 1)."
    result: failed
    command: python3 .github/scripts/test-linux-baseline.py
    revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
    environment: local-macos-arm64
    content: 977afd1b9b79ec04e6a39d27ccbd096a4f916606617a2869d2f31bb1569adf03
    fingerprint: sha256:860234f219219c7e69d7f6976d0dcdc51b7145943895691c2f1f94dba9dc2bb1
    case: TC-github-linux-baseline-lockstep
    case_digest: de0e92375a286f581f072544efb66b2b2df78cfb5dd821a50e0293aa9a788f03
    observed: 2026-11-06T04:04:17Z
    duration_seconds: 24
  - id: 2026-11-06-tc-github-linux-arm64-glibc-download-starts-on-baseline-fea482
    aliases: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-glibc-download-starts-on-baseline: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64.tar.gz --expect-version 6.2.0
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64-docker
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:d31f3335566bcc44fbc4d10389d049ae7bb8c957e3f2ee7c56215a736b6f7ee6
    case: TC-github-linux-arm64-glibc-download-starts-on-baseline
    case_digest: 6c1458891f6ec19102613f5b61efd0e208d94a14c0a036f4bc247c882825e908
    observed: 2026-11-06T04:58:03Z
    duration_seconds: 4
  - id: 2026-11-06-tc-github-linux-arm64-musl-download-starts-anywhere-65ac20
    aliases: OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b
    claim: "Ran python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0 for TC-github-linux-arm64-musl-download-starts-anywhere: passed (exit 0)."
    result: passed
    command: python3 -I .github/scripts/linux-baseline.py check --archive target/linux-baseline/specsync-linux-aarch64-musl.tar.gz --expect-version 6.2.0
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64-docker
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:89f47822b12c94514c628e202d2dd9d79bf49d4eb189b95189430d69a27b635a
    case: TC-github-linux-arm64-musl-download-starts-anywhere
    case_digest: 9b63920d3e9575e64918c797332f155b597e92179b987c70f28ce5d1993f1ec9
    observed: 2026-11-06T04:58:10Z
    duration_seconds: 5
  - id: 2026-11-06-tc-github-linux-baseline-lockstep-4afe7b
    aliases: OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991
    claim: "Ran python3 .github/scripts/test-linux-baseline.py for TC-github-linux-baseline-lockstep: passed (exit 0)."
    result: passed
    command: python3 .github/scripts/test-linux-baseline.py
    revision: 4547d9c0118b6a463a2344aa08ed0c17f580fb32
    environment: local-macos-arm64
    content: 40102202888fdf3bd6b11480d008fc0643523f192cfb58b692f16c3da48e70ae
    fingerprint: sha256:28ef48376243d93dc9b30cd4d800d7cd1a8abc3f9ce02b56e7c9bc1bb2241d43
    case: TC-github-linux-baseline-lockstep
    case_digest: de0e92375a286f581f072544efb66b2b2df78cfb5dd821a50e0293aa9a788f03
    observed: 2026-11-06T04:57:58Z
    duration_seconds: 5
environment: local-macos-arm64-docker
revision: 6fd90eebd2b7d67f5eac8239af205462bd44ce95
found_by: spec-sync#882 (independent audit D6)
related: []
created: 2026-11-06T04:05:14Z
updated: 2026-11-06T04:58:27Z
---
# Linux ARM64 release download does not start on Debian 12 or Ubuntu 22.04

**What happened:** The published specsync-linux-aarch64 (6.1.2 and 6.2.0) exits before --version with version GLIBC_2.39 not found; no static ARM64 fallback download exists
Actual: Loader refuses to start it: two weak Rust std symbols (pidfd_spawnp, pidfd_getpid) carry GLIBC_2.39 because the release links the gnu targets on ubuntu-latest (24.04)

**Steps:** Trigger: Run the checksum-verified published specsync-linux-aarch64 on glibc older than 2.39 (debian:12-slim 2.36, ubuntu:22.04 2.35)
1. Download v6.2.0 specsync-linux-aarch64.tar.gz and verify its sidecar
2. Place it under target/linux-baseline/ and run chough cases run TC-github-linux-arm64-glibc-download-starts-on-baseline --environment local-macos-arm64-docker

**Expected:** The prebuilt Linux download starts on the documented floor (glibc 2.35: Ubuntu 22.04, Debian 12 and newer) and a static musl download works below it

**Why it matters:** Every Linux ARM64 user below glibc 2.39, including GitHub's ubuntu-22.04-arm runners using the Action; the x86_64 gnu download has the same defect

**Fix:** Recorded before 0.6.0; the Log keeps its history.

## Log

- 2026-11-06T04:05:14Z · Reported (spec-sync#882 (independent audit D6)); evidence: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-c9cfc037fd1400b5587015c3, OBS-tc-github-linux-arm64-musl-download-starts-anywhere-4accc748063adeedf526da3e, OBS-tc-github-linux-baseline-lockstep-8d3fbcb70a07759eebbe4095
- 2026-11-06T04:58:27Z · status: open -> verified. Fixed at merged tree 4547d9c0: ARM64 glibc archive built natively in ubuntu:22.04 (Rust 1.89.0, cargo build --release --target aarch64-unknown-linux-gnu) passes on ubuntu:22.04 and debian:12-slim; ARM64 musl archive built in ubuntu:24.04 with musl-tools passes on ubuntu:22.04, debian:12-slim, debian:11-slim and alpine:3.22; lockstep test passes. x86_64 archives and the hosted runners are first exercised by the linux-baseline CI job on the archived tip.; evidence: OBS-tc-github-linux-arm64-glibc-download-starts-on-baseline-b49ab60d64c36c856935f453, OBS-tc-github-linux-arm64-musl-download-starts-anywhere-f8d8bed59851d38bba47323b, OBS-tc-github-linux-baseline-lockstep-0e1525ce22afcbc2fa4ac991
