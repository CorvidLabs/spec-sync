---
spec: hash_cache.spec.md
---

## User Stories

- As a developer on a large spec set, I want `specsync check` to re-validate only the specs that actually changed so that validation stays fast
- As a developer, I want changes to a spec's backing source files (`files:` in frontmatter) to also trigger re-validation so that drift between code and spec is caught
- As a developer, I want companion file edits (requirements/context/tasks/testing/design) to mark the parent spec as changed so that companion-aware checks re-run
- As a CI operator, I want the cache to be self-healing on first run or corruption so that a missing/invalid cache never blocks validation — it just re-checks everything
- As a maintainer, I want stale cache entries for deleted files pruned automatically so that the cache file does not grow without bound

## Acceptance Criteria

- Cache persists to `{root}/.specsync/hashes.json` (pretty JSON); the `.specsync/` directory is created on `save`
- `HashCache::load` returns an empty cache when the file is missing or unparseable (treating all files as changed)
- `hash_file` computes SHA-256 in 8KB chunks and returns a hex digest, or `None` if the file cannot be read
- `is_changed` returns true when a path is new or its current hash differs from the cached value; unreadable files are treated as changed
- Path keys are normalized to forward slashes (`normalize_rel`) for cross-platform consistency
- `classify_changes` reports `ChangeKind::Spec`, `Requirements`, `Companion`, and/or `Source` for one spec; the requirements companion maps to `Requirements`, while context/tasks/testing/design map to `Companion`
- Companion detection covers both the plain convention (`requirements.md`, `context.md`, `tasks.md`, `testing.md`, `design.md`) and legacy `{module}.<suffix>` prefixed names
- `classify_all_changes` / `filter_unchanged` return only specs that have detected changes
- `update_cache` re-hashes every spec, its companions, and its source files after validation, then prunes deleted entries
- `extract_frontmatter_files` extracts the `files:` list with lightweight string matching, not the full YAML parser

## Constraints

- Hashing must stream in chunks (8KB) to avoid loading large files fully into memory
- Cache corruption or absence must never error — fall back to "everything changed"
- Path normalization is mandatory so cache keys are stable across OSes
- Companion-name detection must support both current and legacy naming without false positives
- `extract_frontmatter_files` must not pull in the full parser (kept cheap for the hot path)

## Out of Scope

- Hashing strategies other than SHA-256
- Tracking changes to files not referenced by a spec (spec, its companions, and its `files:` only)
- Cross-run change history or diffing (only the last-known hash and last validation snapshot are stored)
- Watching the filesystem for live changes (see the `watch` module)
- Validating spec content (only change detection lives here)

### REQ-hash-cache-001

The `hash_cache` module SHALL preserve truthful user-visible behavior for the pre-6.0 product fixes landed in this change.

Acceptance Criteria
- Related tests remain green.
- No intentional regression of SpecSync 6.0 lifecycle verbs.

### REQ-hash-cache-002

A change classification SHALL record whether the cache held a prior hash for the spec, so
that a change can be told apart from an absence of evidence.

Acceptance Criteria
- An absent cache entry continues to select the spec for re-validation.
- An absent cache entry is not reported as a change.
- A companion change observed against a known baseline is still reported.
- The cache's own frontmatter `files:` extraction resolves a quoted entry to the same path the parser resolves.

### REQ-hash-cache-003

The hash cache SHALL store the last validation snapshot for a spec and SHALL refuse to
replay it when any bound input has changed.

Acceptance Criteria
- `record_current_validation_snapshot` writes errors, warnings, and notices bound to the current input digest.
- `replayable_validation_snapshot` returns that snapshot only when the schema, integrity digest, and every validation input still match.
- A cache written before snapshots existed loads, but replay returns `None` so the caller re-validates.

### REQ-hash-cache-004

A cached spec's validation result SHALL be stored and replayed.

Acceptance Criteria
- The result recorded when a spec was last validated survives in the cache alongside its hash.
- A spec skipped as unchanged replays that result rather than contributing nothing.
- Editing a spec or any file it maps re-validates it and overwrites the stored result.
- The cache continues to skip re-validation, not merely re-extraction; the optimisation is preserved and the verdict is not lost.

### REQ-hash-cache-005

A validation snapshot SHALL be replayed only by the build that recorded it. The snapshot input digest SHALL frame a fingerprint of every source and manifest the binary was compiled from: the files under `src/`, plus `Cargo.toml`, `Cargo.lock` and `build.rs`. A snapshot recorded by any other build SHALL be re-validated by a default `specsync check` instead of replayed, including one that reports the same package version and one written before this requirement.

#### Acceptance Criteria

- A clean snapshot written under the pre-#878 digest, for a spec whose `requirements.md` repeats an ID, is not replayable, and this build reports the duplicate.
- A clean snapshot under another build fingerprint at the same package version is not replayable, while the same snapshot under this build's fingerprint is.
- Removing the fingerprint from the digest fails these tests.

### REQ-hash-cache-006

A validation snapshot SHALL be bound to the spec's history fragments: the snapshot input digest SHALL frame the name, type and content of every entry in the spec's `history/` directory, and the type of the directory itself, so adding, editing, removing or replacing a fragment, including swapping it for a symlink to the same bytes, re-validates the spec instead of replaying the earlier verdict.

Acceptance Criteria
- A snapshot recorded before a fragment was added, edited or removed is not replayable; with no fragment change it is.
- A snapshot recorded before a fragment was swapped for a symlink to a copy of the same bytes is not replayable.

### REQ-hash-cache-007

The hash cache SHALL never be read or written through a symbolic link, and SHALL be written only inside the project's own `.specsync` directory. `HashCache::save` SHALL refuse, writing nothing, when `.specsync` is a symbolic link or not a directory, or when `.specsync/hashes.json` is a symbolic link or anything other than a regular file. Otherwise it SHALL write the cache to a new file in `.specsync` and rename that file over `.specsync/hashes.json`. `HashCache::load` SHALL return an empty cache for the same locations, without reading what a link points to. `HashCache::refusal` SHALL say which path is refused and what it is, and SHALL return nothing for a usable or absent cache.
`save` SHALL also refuse, writing nothing and leaving the cache already there, a cache whose JSON is larger than the bound `load` reads (64 MiB).

Acceptance Criteria
- With `.specsync/hashes.json` a symbolic link to an existing file outside the project, `save` returns an error, the link is still a link and that file's bytes are unchanged; `load` returns an empty cache even when that file holds a valid cache.
- With that link dangling, `save` returns an error and nothing is created at its target.
- With `.specsync` a symbolic link to a directory outside the project, `save` returns an error and that directory gains no entry; `load` returns an empty cache even when that directory holds a valid `hashes.json`.
- With `.specsync/hashes.json` a directory, `save` returns an error and `load` returns an empty cache.
- With a regular `.specsync/hashes.json`, `save` replaces it by rename: a second hard link to the old file keeps the old bytes, the new cache round-trips through `load`, and `.specsync` holds no file besides `hashes.json`.
- `refusal` names `.specsync/hashes.json` or `.specsync` in each refused case and returns nothing for a regular or absent cache.
- A cache whose JSON is over 64 MiB is refused, and the cache already there is unchanged.

### REQ-hash-cache-008

The hash cache SHALL read `.specsync/hashes.json` (64 MiB, still refusing any link as REQ-hash-cache-007 requires) and every file it hashes or reads to classify a spec (8 MiB) only through the bounded project read of `util` (REQ-util-003), with a bound of the one named for each file. A read refused there, because the file leaves the project through a symbolic link, is a dangling link, is not a regular file, or is larger than the bound, SHALL be handled as that read's other read errors are, with a message that names the path and the reason, and SHALL never wait on the file or read past the bound. A file it cannot hash is treated as changed, as before.

Acceptance Criteria
- With a spec's `requirements.md` a FIFO, `check` finishes without waiting on it.
- With `.specsync/hashes.json` one byte over 64 MiB, it loads as an empty cache without reading it whole.
- `tests/integration/project_reads.rs` `no_project_file_is_read_outside_the_bounded_read` finds no direct file read in this module's non-test source outside the allowed list

### REQ-hash-cache-009

The hash cache SHALL be read and saved only as `hashes.json` in the project's local state folder outside the work tree (REQ-util-006). A `.specsync/hashes.json` in the work tree, in any letter case and whether or not Git tracks it, SHALL never be read or saved, so a cache that came with a repository (a clone, an unpacked archive, or a checkout where Git cannot answer) can never replay a stored pass. When no local state folder can be found, `load` SHALL return an empty cache, and `save` and `refusal` SHALL give the reason.

Acceptance Criteria
- A forged pass written to `.specsync/hashes.json` in a tree with no Git, and committed as `.specsync/Hashes.json` on a case-insensitive disk, is never replayed.
- `save` writes into the local state folder and leaves the work tree's `.specsync` unchanged.

### REQ-hash-cache-010

`prune` SHALL keep an entry only for a regular file the project rule reaches, and SHALL drop a key that is absolute or holds `..` without looking at the path it names.

Acceptance Criteria
- An absolute key and a `..` key naming an existing outside file are dropped; a key for a project file is kept.

