---
change: correct-the-claim-that-nothing-writes-the-change-sequence-ledger-in-the-diagnostic-that-started-it-and-in-the-two
artifact: testing
---

# Testing

## What is and is not verifiable here

No behaviour changes. One diagnostic string and two prose files.

**No test is added, deliberately.** The only assertion available would pin the new wording of an
error message, which would fail against unfixed `main` — satisfying the letter of the discrimination
protocol while proving nothing about correctness. A string-equality test on a diagnostic does not
establish that the diagnostic is *true*; it establishes that nobody has edited it. That is precisely
the false comfort this change is about, and adding one would be the defect wearing a test's clothes.

**Honest label: no DISCRIMINATOR exists for this change, and none should.** What would have caught
the original error is not a test but a reader checking a claim against the function that implements
it — which is what the #714 audit did.

## What was checked instead

| check | result |
|---|---|
| No live file claims the ledger is unwritten, read-only, or frozen | `grep -rn 'read-only history\|[Nn]othing writes'` over tracked files returns only `specs/change/context.md` (owned by the concurrent #714 audit) and archived records under `.specsync/archive/`, which are immutable evidence and correctly untouched |
| No test pinned the old wording | `grep -rn 'nothing writes this file\|repaired by allocating' src/ tests/` returns nothing but the source line itself — part of why the claim survived |
| The claim being corrected is actually false | `floor_sequence_ledger_to_committed` at `src/change.rs:1869` calls `write_json(&root.join(SEQUENCE_PATH), …)`; caller at `src/commands/change.rs:2865` is on the commit path |
| The replacement is true | Nothing calls an allocation path; #665 removed `maximum_observed_sequence` and `remote_sequence_high_water`, and #732 retired the last spec text describing them |

Full suite, `cargo fmt --check`, and `cargo clippy -- -D warnings` all run unchanged, since the
edit is inside a `format!` string literal and two Markdown files.
