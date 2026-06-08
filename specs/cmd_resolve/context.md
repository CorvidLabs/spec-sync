---
spec: cmd_resolve.spec.md
---

## Key Decisions

- **Network is opt-in**: with no flags the command only checks local file existence and lists cross-project refs. `--remote` enables registry fetches; `--verify` (which implies `--remote` at the CLI) enables deep content checks.
- **Two-phase remote checks**: phase 1 confirms a module exists in the remote registry; phase 2 (`--verify`) fetches the actual remote spec and inspects status, exports, and bidirectional deps.
- **Drift vs warning**: deprecated/removed/archived status and missing consumed exports are breaking `DRIFT` (exit 1); non-bidirectional deps, fetch failures, and parse failures are non-fatal `WARN`.
- **Cache to avoid re-fetching**: remote spec bodies are cached on disk under `.specsync-cache/remote-specs/` with a TTL (default 3600s); repo/path slashes are sanitized into the cache filename. TTL 0 disables caching.
- **Consumed-export discovery is table-driven**: `find_consumed_exports` scans the local spec's `### Consumes` table for rows whose module column matches the remote module, extracting backtick-wrapped identifiers.

## Files to Read First

- `src/commands/resolve.rs` — `cmd_resolve`, `verify_remote_specs`, `find_consumed_exports`, the `SpecCache` helper, and the inline unit tests.
- `src/registry.rs` — `fetch_remote_registry`, `fetch_remote_spec`, `parse_remote_spec`, `RemoteRegistry`, `RemoteSpec`.
- `src/validator.rs` — `is_cross_project_ref` / `parse_cross_project_ref` for ref classification.
- `src/main.rs` (Resolve arm) — where `remote || verify` is passed, making `--verify` imply `--remote`.

## Current Status

Stable and implemented. Unit tests cover the cache, `### Consumes` parsing, and deprecated-status detection; the registry/network orchestration is not yet covered end to end.

## Notes

- This is a command-layer module orchestrating `parser`, `registry`, `validator`, and `github`; it owns the dependency-resolution flow and the on-disk remote cache, but not the HTTP/registry primitives.
- `DriftIssue` is an internal enum (`#[allow(dead_code)]` on some variants used only for matching); it is not part of the public surface.
