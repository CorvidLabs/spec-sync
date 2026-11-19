---
change: never-read-a-project-file-through-a-link-that-leaves-the-project-a-non-regular-file-or-past-a-size-bound
artifact: plan
---

# Plan

## Approach

Add the bound to `util`'s project read and make every in-scope read go through it, with failing tests first and a source scan that keeps it that way.

## Out of scope

- Paths named on the command line (`import --from-dir`, `change correct-owner --manifest`).
- The repository's own `.git`, including the pre-commit hook in the Git hooks directory.
- The files Git hands the merge driver.
- Git output and MCP stdin.
- The confined, bounded readers that already exist (`watch`, coverage, the Gradle manifest reader, MCP, the issue snapshot).
- Chough's own unbounded reads, which are a Chough fix.

## Steps

1. Write the failing tests on `3a02a5d5`, without `/dev/zero`:
   - `util` unit tests;
   - integration tests with FIFOs (under a timeout), outside links and sparse over-bound files at `sdd.json`, the journal, `config.toml`, `.specsyncignore`, `requirements.md`, `hashes.json`, the MCP lock, `agent-artifacts.json`, `tasks.md` and `CLAUDE.md`;
   - the source scan.
2. Add the bound and the two read functions to `util`.
3. Move the sites module by module: `change`, `cmd_change`, `config`, `ignore`, `registry`, `mcp`, `agents`, `hooks`, `hash_cache`, `exports`, `deps`, `schema`, `validator`, then the spec readers.
4. Update the 31 specs, `CHANGELOG.md`, `cli.md` and the lessons.
5. Run `change check --commit`, the full verify lane, and a Linux run in a local container.

## Risks

- A project file over its bound stops being read. Projects with a source file over 8 MiB already hit the coverage bound; a policy, config or change record over 4 MiB is far past any real one (the largest in spec-sync is 274 KB).
- Functions gain a `root` argument, which touches many call sites and tests.
- A refused read follows that read's existing error handling. Where that is silent today, a refused file is treated as missing, as an unreadable one already is.

## Constraints consulted

- `specs/util/util.3md`, the lesson from change 2: stay inside the root by construction, through one handle on the project root, and follow a link that stays inside.
- `specs/hash_cache/hash_cache.3md`: the cache is conservative on uncertainty, so a refused cache read is an empty cache, and a refused hash is a change.
