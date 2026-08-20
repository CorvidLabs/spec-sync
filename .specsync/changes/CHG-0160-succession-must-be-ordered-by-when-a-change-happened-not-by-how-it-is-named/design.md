# Design

`succession_change_key` is replaced by `happens_after(later, earlier)`:

```rust
fn happens_after(later: &ChangeRecord, earlier: &ChangeRecord) -> bool {
    (later.created_at, later.id.as_str()) > (earlier.created_at, earlier.id.as_str())
}
```

`created_at` (`ChangeRecord`, `:706`) says what the ordinal was standing in for. The ID tiebreak
keeps the relation a strict total order when two changes share a timestamp, which matters
because callers enforce strict sorts.

**It costs no I/O the ordinal saved.** The objection recorded earlier — that a `created_at`
comparison needs the record loaded, which the call sites cannot do — turned out to be wrong for
every site that needs *chronology*. At `:7519` the predecessor is loaded on the line above; at
`:14722` and `:14842` both records are already in the `records` map.

## The three sorts become lexicographic

`:2447`, `:7460` and `:10888` need only *a* deterministic total order, not chronology — they
exist for canonical serialization and digest stability. Lexicographic by `predecessor_id` is
that, and it makes all four orderings agree with `approved_scope:9308`, which is the one that
feeds `scope_digest`. So the change removes the ordinal dependency **and** closes the five-digit
divergence in the same edit.

The strict-sort message changes accordingly: "strictly sorted by numeric sequence and full
predecessor ID" is no longer true, and a message describing an ordering the code does not use is
the kind of thing that later reads as a bug report.

## The three ordering comparisons become chronological

`:7519`, `:14722`, `:14842` ask a genuine happens-before question and now use `happens_after`.

`:14842` lives in `later_sequence_owner_covers_historical_input` and dies with the sequence
ledger in a later change. It is converted rather than left on the old key so that nothing in the
tree still derives time from a name in the interim.

## Discrimination

Against a separate checkout of `origin/main`:

```
supersedes_edges_sort_the_same_way_everywhere_at_five_digits         FAILED
a_predecessor_created_after_its_successor_is_refused                 FAILED
a_predecessor_created_before_its_successor_passes_the_ordering_guard passed
```

The refusal test is deliberately exercised through `validate_supersedes_semantics` rather than
through `happens_after` directly, so it compiles and runs on both binaries and therefore
actually discriminates. A test that calls a function which does not exist on the old binary
proves only that the function is new.

The third is the vacuity control: the ordinary direction must still clear the ordering guard, so
this is not "refuse everything". It asserts on the ordering error specifically, allowing a
failure at a later gate, because a control that demands overall success would be testing
unrelated machinery.
