---
change: CHG-0121-coverage-over-zero-source-files-must-report-nothing-measured-everywhere-replac
artifact: design
---

# Design

Delete `coverage_percent` and `loc_coverage_percent` from `CoverageReport`.
Replace them with accessors that cannot express a percentage that was never
measured:

    pub fn file_coverage_percent(&self) -> Option<usize>
    pub fn loc_coverage_percent(&self) -> Option<usize>

`None` when the denominator is zero.

The point is not that `Option` is tidier. It is that removing the fields turns
every one of the twenty call sites into a compile error, so the change cannot be
partially applied. A renderer that wants a number must now say, in code, what it
prints when there is no number. That is the property #575 lacked: nothing failed
when eight sites were left behind, because nothing forced them to be visited.

This has already been demonstrated. A concurrent fix on another branch added a
new hand-rolled CSV renderer that read `coverage.coverage_percent` directly. On
this change's type it does not compile:

    error[E0609]: no field `coverage_percent` on type `&CoverageReport`

Under the old type it would have compiled and silently shipped the 100.

Rendering, per surface:
- text: the existing `0/0 (no source files to measure)` wording, unchanged
- JSON: `null`, not `0` and not `100` — a consumer must be able to tell "nothing
  to measure" from "measured, and it is zero"
- `--require-coverage`: fails closed. A gate cannot pass on a measurement that
  was never taken.

Out of scope: changing what counts toward the denominator, and the coverage
figures of any project that has source files. Those are unchanged by
construction, and the healthy-project control proves it.
