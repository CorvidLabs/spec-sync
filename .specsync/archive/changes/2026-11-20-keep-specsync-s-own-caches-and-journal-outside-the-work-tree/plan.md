---
change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
artifact: plan
---

# Plan

## Approach

Failing tests first, on `a2a50824` with only the tests added. Then the fix in one commit.

## Out of scope

- **`--template` and `--manifest` (F5):** the person types these paths, and they are read as given. This is documented.
- **The Git directory by device and inode, and HFS+ ignorable code points (F8):** documented.
- **Check-then-open races (F9):** they need local access. Documented.
- **`fledge lanes run pre-push` from the change push path (F10):** this is by design, and it is documented as running the repository's commands.
- **Renaming `src/validator/tests.rs`:** the dep-info list makes the name irrelevant to the scans.
- **The lifecycle lock:** it stays at `.specsync/change.lock`. It is only opened and locked, never read.

## Steps

1. **Tests.**
   - `tests/integration/local_state.rs`:
     - a forged cache in an unpacked tree;
     - a cache committed in another letter case, on a case-insensitive disk;
     - an in-tree journal;
     - the hooks folder;
     - an enclosing work tree.
   - Unit tests in `hash_cache`, `change_tests`, `resolve` and `util`.
   - The widened scans.
   - Run them all on the base and keep the failures.
2. **`local_state`**, then the hash cache, the journal and the remote cache moved onto it.
3. **Confinement:** the enclosing-work-tree rule.
4. **Hooks:** the hooks folder handle.
5. **Scans:** the dep-info list, the lexer, the wider call lists and the Git scan. Then fix or mark what they find.
6. **Records:** specs, lesson, `CHANGELOG.md`, and the advisory additions.
7. **Checks:** `change check`, the verify lane, and the Linux container run, all through one-lane.

## Risks

- **The first `check` after upgrading runs cold.** The old `.specsync/hashes.json` stays where it was, unused.
- **A journal an older SpecSync left after a crash** stops lifecycle commands until it is checked and deleted.
- **A subproject not yet added to Git**, inside a monorepo, is confined to itself until it is added.
- **A hooks folder reached through a link** (`.githooks -> tools/hooks`) is refused.

## Constraints consulted

- `util.3md`: stay inside by construction.
- `change.3md`: a refusal names its recovery.
- Leif's option A bounds the scope.
