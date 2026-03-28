---
spec: registry.spec.md
---

## Tasks

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
- [x] `RemoteRegistry` struct with `has_spec()` lookup

## Gaps

- No caching — every `resolve --remote` re-fetches all remote registries
- Private repos are inaccessible (no auth token support)
- No validation of registry TOML structure (malformed files fail silently)

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: n/a
- **Dev**: pending
