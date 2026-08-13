---
change: CHG-0109-a-symlink-under-a-source-directory-must-be-skipped-and-disclosed-never-abort-di
artifact: plan
---

# Plan

Implementation and the full suite ran **before** `change new`, per #542: delivery scope
freezes at the interview, and blast radius only becomes visible at compile, test, and
verification time. The declared scope is measured, not estimated — eight files, of which
four changed only because `CoverageReport` gained a field.

## Sequence

1. Add `skipped_links` to `CoverageTraversalBudget` with `record_skipped_link`.
2. Convert the three discovery sites from `return Err` to record-and-continue; leave the
   configured-source-dir and spec-tree sites fatal.
3. Add `CoverageReport::skipped_links`; populate it from the budget.
4. Disclose on all three channels — text, markdown, JSON.
5. Gate under `--strict` in **both** exit paths.
6. Verify: `cargo test --no-run` for discovery, then fmt, clippy, full suite.
7. Pin as sandbox drill 040 assertions, and prove they discriminate against a pre-fix
   binary before trusting them.

## Ordering learned the hard way

Step 4 was written after step 2 rather than with it, and the intermediate state — run
completes, exclusion invisible — is precisely the defect this change exists to prevent.
Steps 2 and 4 are one change and should be made together.

## Rollout

A repository with no symlinks under its source directories sees no behavioural change at
all: no new output, no new exit codes. A repository that has them stops failing outright,
gains a disclosure line, and gains a `--strict` failure it did not previously reach because
the command never got that far.
