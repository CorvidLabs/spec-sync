---
change: CHG-0117-coverage-over-zero-source-files-must-report-that-nothing-was-measured-not-one-h
artifact: context
---

# Context

## What led here

CorvidLabs/spec-sync#562, found by sweeping affirmative-claim sites rather than from a
report. A project whose configured `source_dirs` contain no source files:

```
$ specsync coverage --require-coverage 100
File coverage: 0/0 (100%)
LOC coverage:  0/0 (100%)
exit 1
```

In a **single run**, the display reports 100% and the exit code reports failure.

## The codebase already knew

`compute_exit_code` carries the reasoning verbatim:

> A `--require-coverage` gate over zero source files is a vacuous pass: coverage is reported
> as 100% when there is nothing to measure (an empty or misconfigured `source_dirs`, or an
> over-broad `exclude_patterns`), silently satisfying the gate. Fail loud so a broken config
> cannot pass CI.

So the hazard was understood, written down, and defended — **in the gate**. The display was
never brought along. Without `--require-coverage` the run exits 0 and the only thing on
screen is `100%`.

## Why the display is the half that matters

The gate protects CI. The number protects nobody: it is what goes on a badge, into a
dashboard, or into a status report. A project with a misconfigured `source_dirs` — precisely
the scenario the gate comment names — records 100% coverage.

## Two more affirmative lines in the same output

`✓ All source files referenced by specs` and `✓ All source modules have spec directories`
are both true of an empty set and read as measurements. They are replaced with the
actionable cause rather than deleted, since "no source files were found" is a
misconfiguration the reader can act on.

## Tenth instance of one class

Same as #546, #547, #548, #549, #550, #551, #553, #558, #560: absence of evidence rendered
as evidence of absence.
