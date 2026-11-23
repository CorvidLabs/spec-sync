---
change: never-write-a-project-file-through-a-symbolic-link
artifact: plan
---

# Plan

## Approach

Add the project file functions to `util` with their own tests, then move each write site onto them, one module at a time, with a failing test on `7b6e4779` before each.

## Out of scope

- The hash cache, fixed by the first change, and every writer the sweep read as already guarded.
- Reads that are not followed by a write, such as `hooks status`.
- Publishing the advisory and releasing.

## Steps

1. `util`: the functions and unit tests for a link out, a dangling link, a chain that leaves, a link that stays inside, a linked directory, a non-regular target.
2. `hooks`, `agents`, `mcp`, `registry` with `scaffold`, `cmd_resolve`: failing tests, then the move.
3. `generator` companions and the spec writers in `new`, `scaffold`, `add-spec`, `wizard`, `import`: failing tests, then the move.
4. `migrate`: failing tests, the preflight, then the move.
5. Specs, lessons, `CHANGELOG.md`, `cli.md`; `change check --commit`; the full verify lane locally and a Linux run in a local container.

## Risks

- A project that keeps one of these files as a link to a place outside the repository gets an error where it got a write. That is the point, and the message names the path.
- A link with an absolute target is refused even when it points inside the repository. Such a link does not survive a clone to another path.
- `register_module` and the companion functions change signature; every caller is in this repository.

## Constraints consulted

- `specs/hooks/hooks.3md`: installing is idempotent and appends, never overwrites. Unchanged.
- `specs/hash_cache/hash_cache.3md`, lesson from the first change: a path this tool creates is still a path the repository chose, and a check followed by a path write leaves the window the check was for.
