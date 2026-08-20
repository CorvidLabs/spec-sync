---
change: retire-the-ordinal-and-keep-the-ledger-readable-forever
artifact: testing
---

# Testing

| Test | Proves |
|---|---|
| `ordinal_free_change_ids_do_not_block_numeric_sequence_validation` | an ID claiming no ordinal takes part in no numeric accounting and is not fatal to it |
| `a_non_canonical_ordinal_notation_still_fails_closed` | `CHG-123-too-short` and `CHG-09999-noncanonical-width` still refuse — the narrowing is a distinction, not a blanket skip |
| `two_archived_packages_sharing_an_ordinal_are_still_refused_until_acknowledged` | collision detection survives for the identities that still carry ordinals |
| `change_ordinals_identify_independently_allocated_workspaces` | control — the numeric gate is intact for the 11 archived collision members |

## End to end

With the new binary, in a fresh repository:

```
$ specsync change new "retire the ordinal at last"
retire-the-ordinal-at-last

$ specsync change new "retire the ordinal at last"
error: a change named `retire-the-ordinal-at-last` already exists
  .specsync/changes/retire-the-ordinal-at-last
  it is: retire the ordinal at last (draft)
```

No sequence ledger is created. This change's own package is the first slug-only one in this
repository.

## Corpus

0 of 164 archives change validity; one representative per risk class checked against a baseline
captured before any of the identity work.

## Suite

2345 unit + 405 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-086 | `change new` mints a slug and refuses a repeated description by name, verified end to end in a fresh repository. `ordinal_free_change_ids_do_not_block_numeric_sequence_validation` proves an ordinal-free identity is tolerated where it previously bricked creation repo-wide, and `a_non_canonical_ordinal_notation_still_fails_closed` proves that tolerance is a distinction rather than a blanket skip. `change_ordinals_identify_independently_allocated_workspaces` is the control keeping the numeric gate honest for the identities that still carry ordinals |
