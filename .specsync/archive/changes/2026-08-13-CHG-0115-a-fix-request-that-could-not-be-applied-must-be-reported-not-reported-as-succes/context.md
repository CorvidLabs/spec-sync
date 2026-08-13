---
change: CHG-0115-a-fix-request-that-could-not-be-applied-must-be-reported-not-reported-as-succes
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#549. Two byte-identical projects, one with a non-writable spec:

```
$ specsync check --fix     # read-only spec
  ⚠ Undocumented export 'undocumented_one' from src/alpha.rs
exit 0

$ specsync check --fix     # writable twin
  ✓ specs/alpha/alpha.spec.md: added 1 export(s)
✓ Auto-added exports to 1 spec(s)
exit 0
```

Same exit code, opposite outcomes. The user asked for a mutation, none happened, and nothing
said so.

## Root cause

```rust
} else if let Ok(()) = fs::write(spec_file, &new_content) {
```

`if let Ok(())` discards the `Err` arm entirely. There was no failure path at all: the
caller took only a count of successes and used it to decide whether to print a message.

The read side had the same shape — `Err(_) => continue` skipped a spec `--fix` could not
read, without a word.

## Why it matters

`--fix` is the one verb whose entire purpose is to change files. An unwritable target is the
single outcome it must never hide. Read-only spec trees are not exotic: vendored specs, a CI
checkout with restrictive permissions, or a file left `444` by an editor all produce it.

`scaffold` on the same tree already got this right — `Failed to create …: Permission denied
(os error 13)`, exit 1 — so the correct shape existed a few files away.

## Eighth instance of one class

Same as #546, #547, #548, #550, #553, #558, #560: a result reported as success because the
thing that would have contradicted it was discarded rather than examined.
