---
spec: registry.spec.md
---

## Tasks

## Post-5.0 Roadmap

- [ ] Support authenticated GitHub API requests for private repo registries
- [ ] Add registry caching to avoid re-fetching on every resolve
- [ ] Support non-GitHub hosts (GitLab, Bitbucket raw content URLs)
- [ ] Add `specsync registry publish` command to push registry to a package index

## Done

- [x] Local registry generation from specs directory
- [x] Remote registry fetching from GitHub raw URLs
- [x] Zero-dependency TOML parsing
- [x] Template file exclusion
- [x] Module name extraction from frontmatter
- [x] Alphabetically sorted output
- [x] `RemoteRegistry` struct with `has_spec()` and `spec_path()` lookup
- [x] `register_module` — idempotent append of a module entry to an existing registry
- [x] Cross-repo content verification: `fetch_remote_spec`, `parse_remote_spec`, `RemoteSpec`
- [x] Tolerate inert 5.0.1 registry.toml stubs via `load_local_registry`
- [x] `register_module` goes through `util`'s project file functions and reports a refused registry as an error (REQ-registry-003). Evidence: `register_module_refuses_a_registry_linked_outside_the_project`, `register_module_appends_once_to_a_regular_registry`.
- [x] Read project files only through the bounded project read of `util` (REQ-registry-004). Evidence: `tests/integration/project_reads.rs`.
- [x] Walk or list configured folders only through the confined walk, and keep the read cache out of the Git directory (REQ-registry-005).

## Gaps

- No caching — every `resolve --remote` re-fetches all remote registries
- Private repos are inaccessible (no auth token support)
- No validation of registry TOML structure (malformed files fail silently)

## Review Status

Per-module role sign-offs were not collected. Release approval is governed by digest-bound change approvals and required CI; this note is informational and is not a release gate.
