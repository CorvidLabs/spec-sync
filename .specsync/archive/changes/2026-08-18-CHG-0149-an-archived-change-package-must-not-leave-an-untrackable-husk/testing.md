# Testing

Every result below discriminates against a binary built from a **separate checkout** at
34ade838 (`scratchpad/base536`), never by reverting files in the working tree.

## Verb sweep against a husk

A dated archive directory holding only an empty `deltas/`:

| verb | unfixed | fixed |
|---|---|---|
| `change new` | hard-fail rc=1 | ok rc=0 |
| `change audit` | hard-fail rc=1 | ok |
| `change adopt` | hard-fail rc=1 | ok rc=0 |
| `check` | raw OS error as warning, rc=0 | ok rc=0 |
| `check --strict` | hard-fail rc=1 | ok |

## Controls — identical on both binaries

| fixture | both binaries |
|---|---|
| B. files present, no `state.json` | refused, `failed to read archived state` |
| C. legacy tombstone (`deltas/*.md`, no `state.json`) | tolerated |
| D. no archive directory at all | tolerated |

B is the vacuity control: a change that simply stopped reading `state.json` would flip it, and
it does not move.

## Unit tests

| test | unfixed | fixed |
|---|---|---|
| `archive_husk_of_empty_directories_is_skipped_by_enumeration` | FAILED | ok |
| `archive_husk_nested_below_an_empty_directory_is_still_a_husk` | FAILED | ok |
| `archive_directory_with_files_but_no_state_is_still_refused` | ok | ok |
| `archived_package_keeps_directories_that_hold_files_and_drops_the_rest` | n/a (new fn) | ok |

`dated_lifecycle_archive_missing_state_fails_global_enumeration` is pre-existing, asserts the
corruption path, and passes unchanged.

## Sandbox drills

Gate 050 is ≥044 and self-flips:

    unfixed  pass=4 fail=0 pending=3   verdict: FAIL
    fixed    pass=7 fail=0 pending=0   verdict: PASS

Pin 007 is <044 and does not self-flip. It asserts the raw-OS-error behaviour and must be
rewritten — not merely negated — because its recovery branch (`rmdir`, then `change new`
again) only has meaning while the bug exists.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-074 | Producer: gate 050 self-flips 4/0/3 to 7/0/0 — `empty_deltas` goes 1 to 0 and the checkout leaves no orphan, so `change new` and `change audit` both succeed where they hard-failed. Consumer: a twelve-verb sweep against a dated archive holding only an empty `deltas/` breaks 5 verbs on 34ade838 (`change new`, `change audit`, `change adopt`, `check`, `check --strict`) and 0 after. The discrimination line is the corruption fixture: a package holding `change.md` but no `state.json` is refused on **both** binaries, by both readers, so the allowance cannot be satisfied by ignoring corruption — `archive_directory_with_files_but_no_state_is_still_refused` passes on the unfixed source too, and the pre-existing `dated_lifecycle_archive_missing_state_fails_global_enumeration` is untouched. `archived_package_keeps_directories_that_hold_files_and_drops_the_rest` pins the other half of the prune: a directory holding a file at any depth survives. Full suite 2303 + 399 passed, 0 failed |
