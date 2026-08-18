---
change: CHG-0146-ci-must-run-the-product-lane-whenever-the-pull-request-touches-product-paths
artifact: context
---

# Context

`ci.yml` computed the lane from the whole pull request diff, then discarded that
answer whenever the TIP COMMIT alone looked archive-shaped:

    if grep -Eq '^(archive_only|legacy_archive_only|review_only)=true$' <<<"$child_output"; then
      output="$child_output"
    fi

and the whole-PR computation was then skipped entirely:

    if [[ -n "${output:-}" ]]; then
      :

`specsync change ship` always produces a lifecycle archive commit last. So the
override fired on EVERY pull request that used the lifecycle — which is every
product pull request in this repository.

Combined with `success|skipped) ;;` in the required aggregate, a green
`Required CI gate` came to mean "the product lane was deselected", not "the
product lane passed".

## Measured, on this repository

PR #629 changed nine production files — `src/change.rs`, `src/change_tests.rs`,
`src/commands/change.rs`, `src/commands/check.rs`, `src/commands/lifecycle.rs`,
`src/commands/report.rs`, `src/commands/stale.rs`, `src/git_utils.rs`,
`src/scoring.rs` — roughly six hundred lines. Its checks:

    pass      Required CI gate
    skipping  test  fmt  coverage  audit  spec-check

`gh api actions/runs?head_sha=` returned zero runs for all four commits on the
branch. No product gate executed anywhere on that pull request, and
`mergeStateStatus` was CLEAN. PR #567 (`60360ed2`) merged the same way earlier.

## Why it survived

`classify-ci-paths.sh` is thoroughly tested — 322 lines of test script. The
DECISION to override it with a tip-only answer lived in inline workflow YAML and
had no test at all. A well-tested component reached through untested wiring
reads exactly like a working system.
