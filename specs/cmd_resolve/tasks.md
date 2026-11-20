---
spec: cmd_resolve.spec.md
---

## Tasks

## Post-5.0 Test Debt

- [ ] Add end-to-end CLI tests for `--remote` / `--verify` against a stubbed registry (currently only helper-level unit tests exist)

## Done

- [x] Implement local vs cross-project dependency classification
- [x] `--remote` registry fetch + per-repo de-duplication and presence checks
- [x] `--verify` deep content verification: deprecated-status, missing-export, bidirectional, and fetch/parse drift issues
- [x] Exit 1 on breaking drift; warnings exit 0
- [x] File-based remote-spec cache with TTL (`SpecCache`) and slash sanitization
- [x] Unit tests for cache roundtrip/miss/expired, path sanitization, `### Consumes` parsing, and deprecated-status detection
- [x] Never read or write the remote spec cache through a symbolic link; skip it with one warning (REQ-cmd-resolve-002). Evidence: `spec_cache_never_writes_or_reads_through_a_link`, `spec_cache_creates_nothing_through_a_dangling_link_or_a_linked_directory`.
- [x] Read project files only through the bounded project read of `util` (REQ-cmd-resolve-003). Evidence: `tests/integration/project_reads.rs`.
- [x] Keep the remote spec cache in the local state folder under digest names (REQ-cmd-resolve-002, REQ-cmd-resolve-003). Evidence: `spec_cache_never_returns_an_entry_the_work_tree_holds`.

## Gaps

- Coverage is helper-level only (`find_consumed_exports`, `SpecCache`, `RemoteSpec`); the `cmd_resolve` orchestration and `verify_remote_specs` network paths are not exercised end to end.

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
