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

The remote spec cache under `.specsync-cache/remote-specs/` SHALL never be read or written through a symbolic link, whether or not the link stays inside the project: a cache never needs one. When a cache file, or a directory on the way to it, is a symbolic link or not a regular file and a directory, `resolve --remote` SHALL skip the cache for that entry, SHALL say so once on stderr naming the path, and SHALL use the fetched content as it does with no cache.

Acceptance Criteria
- A symbolic link at the cache path of a dependency, to a file outside the project: storing the fetched spec leaves that file unchanged, and reading the cache returns nothing.
- A dangling link at that path creates nothing at its target.
- `.specsync-cache` linked to a directory outside the project: nothing is created there.
- A regular cache file is stored and read back as before.

### REQ-cmd-resolve-003

`resolve` SHALL read the local spec files it reads (8 MiB) and the remote spec cache (8 MiB, still refusing any link as REQ-cmd-resolve-002 requires) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound.

Acceptance Criteria
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

