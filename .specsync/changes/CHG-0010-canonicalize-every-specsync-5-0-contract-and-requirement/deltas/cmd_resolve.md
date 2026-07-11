## ADDED

### REQUIREMENT REQ-cmd-resolve-001

The resolve command SHALL list and optionally verify cross-project references without performing network access unless explicitly requested.

Acceptance Criteria
- `cmd_resolve(root, remote, verify, cache_ttl)` scans every spec's `depends_on`, classifying each ref as local (path) or cross-project (`owner/repo@module`).
- With no flags, local refs are checked by file existence and cross-project refs are listed only — **no network calls** are made.
- At the CLI, `--verify` implies `--remote` (`main.rs` passes `remote || verify`); registry-level checks always run before deep verification.
- `--remote` fetches each repo's registry once (de-duplicated per repo) and reports per ref: module present, module not in registry, registry fetch failed, or no registry.
- `--verify` deep-checks remote specs and reports: `DRIFT` for deprecated/removed/archived remote status, `DRIFT` for a consumed export missing from the remote spec, and `WARN` for non-bidirectional deps, fetch failures, and parse failures.
- The command exits 1 only on breaking drift (deprecated status or missing export); warnings alone exit 0. Unresolvable/local-missing refs are warnings.
- Remote spec content is cached under `.specsync-cache/remote-specs/` with the given TTL (default 3600s); cache filenames sanitize `/` in repo and path to `_`.
