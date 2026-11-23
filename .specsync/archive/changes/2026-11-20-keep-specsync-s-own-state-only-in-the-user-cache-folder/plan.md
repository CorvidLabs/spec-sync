---
change: keep-specsync-s-own-state-only-in-the-user-cache-folder
artifact: plan
---

# Plan

## Approach

Failing tests first, on `e3d508ac` with only the tests added. Then the fix in one commit.

## Out of scope

- **R5b.** An absolute hooks path spelled other than canonically is documented, not changed.
- **Directory listings.** `read_dir` (55 sites) and `WalkDir` (16) are not small, so the scans leave them out. This is listed.
- **Spawned commands other than Git**, and checking the markers themselves: listed.
- **HMAC keying** of the digests (the reviewer's option a): not chosen.
- **`-c safe.bareRepository=explicit`** on SpecSync's Git calls: listed for later.

## Steps

1. **Tests:**
   - CLI tests: an archive that ships `.git` with a forged cache; a project folder named `*`; a local `core.hooksPath` outside; a cache folder inside the project.
   - Unit tests: a journal under `.git/specsync`; a registry-mapped `.env`; the local state location.
   - The wider scans.
2. **`local_state`**, the location and the N2 refusal.
3. **`tracked_by_git`.**
4. **The registry replay rule.**
5. **The hooks scope.**
6. **The scan findings.**
7. **Records:** specs, CHANGELOG, docs, and the advisory additions.
8. **Checks:** the verify lane and the Linux container run, through one-lane.

## Risks

- **The first `check` after upgrading from the branch is cold again**, because state in `.git/specsync` is left alone.
- **CI that sets `HOME` to the checkout runs without a cache** and says so. Lifecycle commands there need `XDG_CACHE_HOME` set outside the project.

## Constraints consulted

- `util.3md`: state never lives where a repository can put a file.
- `change.3md`: a refusal names its recovery.
- Leif's option A bounds the scope.
