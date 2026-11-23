---
change: keep-writes-reads-and-caches-a-repository-can-steer-inside-its-git-work-tree
artifact: plan
---

# Plan

## Approach

Failing tests first, then the fix in one commit. The confinement moves to the Git work tree. The writers, readers and walks a repository can steer go through the confined helpers. A source scan keeps new writes there.

## Out of scope

- Moving the hash cache into the Git directory, or keying its digests. A tracked cache is simply not used.
- Windows short names such as `GIT~1`: Windows is neither built nor published (#735).
- The plain `check` discovery and the MCP server's own confinement, which already refused these inputs.

## Steps

1. Add tests through the CLI in `tests/integration/project_writes.rs` and `tests/integration/repository_inputs.rs`, plus two `hash_cache` unit tests. Run them on the base with only the tests added, and keep the failures.
2. Build the work-tree confinement, the absolute-link and Git-directory refusals, and the safe write open. Add the new helpers and route every writer through them.
3. Add the tracked-cache refusal, `prune`, the directory rule, the confined spec walk, the changelog bound, and the import confinement.
4. Add the write scan and widen the read scan.
5. Update the specs: requirements, invariants, testing, tasks and lessons. Add a `CHANGELOG.md` Security entry.
6. Run `change check --commit`, the verify lane, and the Linux container run.

## Risks

- **A project whose `specs_dir`, `source_dirs` or `schema_dir` points outside it** now fails to load its config. The message names the field.
- **A local absolute link**, such as `CLAUDE.md -> /Users/me/proj/AGENTS.md`, is now refused as an absolute link.
- **A committed hash cache** stops being used, with one warning on each `check` until it is removed from Git.
- **Each process asks Git once per project root** for the work tree.

## Constraints consulted

- `specs/util/util.3md`: stay inside the root by construction, and enforce one rule wherever a path is read, written, walked or replayed.
- `specs/change/change.3md`: a refusal is a result worth recording, and it names its recovery.
- Leif, 2026-11-19 at about 04:53Z: a cache never needs a link, so both caches stay strict.
