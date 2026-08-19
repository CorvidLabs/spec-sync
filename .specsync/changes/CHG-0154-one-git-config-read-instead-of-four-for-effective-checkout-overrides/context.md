# Context

`effective_checkout_overrides_uncached` asked Git for four `core` keys with four separate
`git config --get` invocations — `core.autocrlf`, `core.eol`, `core.symlinks`, `core.filemode`.

Each one is a process spawn, measured at **~15 ms** on this hardware. Instrumented across one
test-suite run, the lifecycle issued **15,359 `git config` spawns**; asking for the four keys in
a single `--get-regexp` takes that to **3,842**.

## How this surfaced

While measuring why `change check --commit` takes 39 minutes on this repository. That
investigation produced two findings and only one of them was real:

- The verification running twice is **not** a defect — it is load-bearing, because committing
  can change what a verification command observes. Withdrawn, with the reproduction recorded on
  #644.
- The subprocess cost is real, and it is not test-only: every spec-sync command that inspects
  the checkout pays it.

## What this is not

Not a cache. Every call still spawns; the saving is asking once for four answers, never
remembering an answer. A cached value would change behaviour — a configuration edited between
two reads must still be observed — and that is the failure mode this deliberately avoids.
