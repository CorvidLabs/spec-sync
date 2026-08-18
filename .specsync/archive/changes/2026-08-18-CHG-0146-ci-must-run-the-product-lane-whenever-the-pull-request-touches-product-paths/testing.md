---
change: CHG-0146-ci-must-run-the-product-lane-whenever-the-pull-request-touches-product-paths
artifact: testing
---

# Testing

## Discrimination

The previous behaviour was reconstructed by deleting the new rule from the
selector, then both were given identical input:

| input | previous | this change |
|---|---|---|
| PR touches `src/main.rs`, tip is an archive move | `full=false` — lane skipped | `full=true` — lane runs |

## Controls

| case | result |
|---|---|
| genuinely archive-only PR, archive tip | narrows to `archive_only=true` — unchanged routing |
| no tip candidate | whole-PR answer stands |

Both pass before and after, so the change cannot be satisfied by always running
the full lane.

## Existing harness

`.github/scripts/test-classify-ci-paths.sh` — passes, including the three new
cases: "classify-ci-paths tests passed".

## Verification of the fix in situ

The pull request carrying this change is itself the test: it touches
`.github/` and its tip will be a lifecycle archive commit. Under the previous
rule its product lane would be skipped.
