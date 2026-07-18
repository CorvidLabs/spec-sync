---
change: CHG-0048-prepare-the-specsync-5-1-1-stabilization-release-from-merged-pr-387-bump-accur
artifact: testing
---

# Testing

Map `REQ-github-002`, `REQ-github-003`, and the change-level release acceptance criteria to deterministic repository
checks, hosted release evidence, and post-publication installation smoke tests.

- **Version consistency:** assert the current package, lockfile, changelog, Action default, README pin,
  Action docs and examples, packaged consumer, and Trust pin are 5.1.1; scan the diff to prove
  archived and historical version references were not bulk-rewritten.
- **REQ-github-002:** parse maintained Action/workflow YAML with Psych, require protected settings
  under each step's `with` mapping, and run the packaged Action against the runner-local
  candidate mirror; after publication, run pinned `@v5.1.1` and floating `@v5` consumers on Linux,
  macOS, and Windows and compare their resolved commit/version.
- **REQ-github-003:** assert all three `setup-bun` call sites specify `bun-version: 1.3.14`; run
  frozen installs plus site tests/lint/build and extension compile/package; require the release PR
  and integrated-main site, extension, and Pages jobs to pass with that pin.
- **Publication gates:** run `fledge release 5.1.1 --dry-run --json`, `specsync check --strict`, the
  full fledge verification lanes, `fledge trust verify`, CodeQL, and the hosted required gate;
  confirm the release tag is integrated into main and matches Cargo.
- **Persisted lifecycle evidence:** configure `specsync change verify` to run both deterministic
  release guards after `cargo test`, without Python site packages, so `REQ-github-002` and
  `REQ-github-003` cannot be marked verified from Rust tests alone.
- **Distribution parity:** verify every GitHub checksum byte-for-byte, perform an exact clean Cargo
  install, validate Homebrew URLs/checksums, and run formula install/test on supported hosts.
- **Fail-closed promotion:** exercise dry-run/preflight failures before publication; inspect remote refs
  before and after each promotion; prove a failed pinned smoke test leaves `v5` and Homebrew
  unchanged. Do not simulate destructive immutable-tag replacement.

The release candidate is not complete merely because local tests pass. Exact-head hosted evidence,
closing approval, post-merge main verification, and post-publication smoke evidence are separate
required checkpoints.
