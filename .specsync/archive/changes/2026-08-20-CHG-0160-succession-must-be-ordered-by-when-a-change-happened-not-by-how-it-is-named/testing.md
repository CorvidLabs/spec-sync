---
change: CHG-0160-succession-must-be-ordered-by-when-a-change-happened-not-by-how-it-is-named
artifact: testing
---

# Testing

| Test | Discriminates | Proves |
|---|---|---|
| `supersedes_edges_sort_the_same_way_everywhere_at_five_digits` | yes | `approved_scope`'s lexicographic order is accepted by the strict-sort gate; on `origin/main` it is rejected with "must be strictly sorted by numeric sequence" |
| `a_predecessor_created_after_its_successor_is_refused` | yes | a predecessor whose name sorts first but which was created later is refused; the old guard saw only the name |
| `a_predecessor_created_before_its_successor_passes_the_ordering_guard` | control | the ordinary direction still clears the guard on both binaries |
| `changes_created_in_the_same_second_are_still_strictly_ordered` | new-unit | equal timestamps still yield a strict, irreflexive total order |

The first two are exercised through `validate_supersedes_semantics`, not through
`happens_after`, so they run on both binaries. A test calling a function that does not exist on
the old binary proves only that the function is new, which is not discrimination.

## Digest invariance

Measured, not assumed: 0 of 160 archived records carry a supersedes edge, and 0 of 160
`verification.json` carry semantic-succession evidence. Reordering either list therefore moves
no historical digest. The CHG-0068 golden vector remains the standing check.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-082 | The refusal test fails against a separate checkout of `origin/main` and passes here, proving ordering now asks when a change was created rather than what it is called; its control passes on both binaries, so succession did not simply become stricter. The five-digit sort test fails on `origin/main` with the numeric-sort message, proving the two orderings over one list now agree — including the one that feeds `scope_digest` |
