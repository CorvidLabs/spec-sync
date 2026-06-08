---
spec: cmd_resolve.spec.md
---

## Tasks

- [ ] Add end-to-end CLI tests for `--remote` / `--verify` against a stubbed registry (currently only helper-level unit tests exist)

## Done

- [x] Implement local vs cross-project dependency classification
- [x] `--remote` registry fetch + per-repo de-duplication and presence checks
- [x] `--verify` deep content verification: deprecated-status, missing-export, bidirectional, and fetch/parse drift issues
- [x] Exit 1 on breaking drift; warnings exit 0
- [x] File-based remote-spec cache with TTL (`SpecCache`) and slash sanitization
- [x] Unit tests for cache roundtrip/miss/expired, path sanitization, `### Consumes` parsing, and deprecated-status detection

## Gaps

- Coverage is helper-level only (`find_consumed_exports`, `SpecCache`, `RemoteSpec`); the `cmd_resolve` orchestration and `verify_remote_specs` network paths are not exercised end to end.

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
