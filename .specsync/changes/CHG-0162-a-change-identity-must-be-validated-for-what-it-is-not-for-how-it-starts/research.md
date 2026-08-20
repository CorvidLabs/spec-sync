# Research

## The prefix was doing no work

```rust
if id.starts_with("CHG-")
    && is_single_component
    && !id.contains(['/', '\\'])
    && !id.chars().any(char::is_control)
```

`CHG-` is four characters any caller can type, so the test proves neither well-formedness nor
provenance. What it did do is hard-reject every identity without an ordinal, from a function
that gates `find_change_dir` (`:16537`) and `validate_loaded_change` (`:16624`).

## Two real checks were missing

**No length bound at all.** Survivable only because every ID was minted as `CHG-NNNN-` over a
capped slug. Accept an arbitrary name and an unbounded one is a directory the process cannot
create. The longest ID in this repository's archive is 90 bytes; the ceiling is the 255-byte
filesystem component limit.

This is deliberately the ceiling rather than `MAX_SLUG_BYTES` (120): the slug cap bounds what
SpecSync *mints*, this bounds what it will *read*. An ID minted by a different version, or by
hand, must still load if it is legal.

**No reserved-name check.** `nul`, `con`, `com1` cannot be directory components on Windows.
Unreachable while every ID began `CHG-`; reachable the moment one does not. Reuses the shared
predicate rather than restating the list — the same decision as CHG-0161, for the same reason.

## What was already right

`Path::new(id).components()` rejects `.` and `..` for free: they yield `CurDir` and `ParentDir`,
not `Normal`. Verified by reading and now pinned by test rather than left to a reader to notice.
