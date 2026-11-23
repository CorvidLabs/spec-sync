---
spec: cmd_resolve.spec.md
---

## User Stories

- As a developer, I want `specsync resolve` to list every declared dependency and show which local refs exist on disk so that broken `depends_on` paths are obvious.
- As a developer with cross-project dependencies, I want `--remote` to fetch each referenced repo's registry and confirm the module is actually published so that I don't ship a dangling cross-project ref.
- As a maintainer, I want `--verify` to fetch the real remote spec and detect drift — deprecated/removed status and exports I consume that no longer exist — so that breaking upstream changes fail my build.
- As a maintainer, I want non-bidirectional dependencies surfaced as warnings (not failures) so that I'm informed without being blocked.
- As a CI operator, I want remote content cached with a configurable TTL so that repeated verify runs don't hammer GitHub.

## Acceptance Criteria

- `cmd_resolve(root, remote, verify, cache_ttl)` scans every spec's `depends_on`, classifying each ref as local (path) or cross-project (`owner/repo@module`).
- With no flags, local refs are checked by file existence and cross-project refs are listed only — **no network calls** are made.
- At the CLI, `--verify` implies `--remote` (`main.rs` passes `remote || verify`); registry-level checks always run before deep verification.
- `--remote` fetches each repo's registry once (de-duplicated per repo) and reports per ref: module present, module not in registry, registry fetch failed, or no registry.
- `--verify` deep-checks remote specs and reports: `DRIFT` for deprecated/removed/archived remote status, `DRIFT` for a consumed export missing from the remote spec, and `WARN` for non-bidirectional deps, fetch failures, and parse failures.
- The command exits 1 only on breaking drift (deprecated status or missing export); warnings alone exit 0. Unresolvable/local-missing refs are warnings.
- Remote spec content is cached under `.specsync-cache/remote-specs/` with the given TTL (default 3600s); cache filenames sanitize `/` in repo and path to `_`.

## Constraints

- Must not panic on expected error conditions — fetch/parse failures degrade to warnings and the scan continues.
- No network access unless `--remote` or `--verify` is set.
- Consumed exports are discovered by parsing the local spec's `### Consumes` table rows matching the remote module name; only backtick-wrapped identifiers are extracted.
- Cache TTL of 0 means entries are always treated as expired (no caching).

## Out of Scope

- Resolving transitive (multi-hop) dependencies — only directly declared refs are checked.
- Writing or fixing dependencies (the command is read-only aside from populating the cache).
- Interactive prompts and any GUI/web interface.

### REQ-cmd-resolve-001

The `cmd_resolve` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-cmd-resolve-002

The remote spec cache SHALL be read and written only in `remote-specs/` in the project's local state folder outside the work tree (REQ-util-006), under a name taken from a digest of the repository and spec path, so neither its place nor its names come from the repository. A `.specsync-cache/remote-specs/` in the work tree SHALL never be read. When no local state folder can be found, `resolve --remote` SHALL say so once on stderr and use the fetched content as it does with no cache.

Acceptance Criteria
- An entry committed under `.specsync-cache/remote-specs/` with forged content is never returned by the cache.
- A stored entry is found again under the local state folder.

### REQ-cmd-resolve-003

`resolve` SHALL read the local spec files it reads (8 MiB) only through the bounded project read of `util` (REQ-util-003), and the remote spec cache (8 MiB) only through the bounded read of its local state folder (REQ-util-006). A refused read, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.
A refused read of a local spec SHALL be reported with the path and the reason, never skipped silently with its `depends_on`.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list
- `resolve` with an oversized spec names it and the reason.

