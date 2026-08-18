# Testing

## Before

    ERROR: test_release_reconstruction_requires_actual_pull_request_event
    ValueError: substring not found
    FAILED (errors=1)
    ##[error]Process completed with exit code 1

CI effect on PR #638: `Classify changed paths` failed, and because every other job depends on
it, `audit`, `coverage`, `fmt`, `site`, `spec-check`, `test`, `validate-action`,
`vscode-extension`, `Lifecycle gate` and `Packaged GitHub Action consumer` all reported
`skipping`. `SpecSync implementation ready` and `Required CI gate` then failed.

That cascade is worth noting on its own: a single orphaned string anchor silently converted ten
gates into `skipping`, which is the same shape as #626 — a gate that did not run reading as a
gate that did not object.

## Anchor enumeration

Every `release.index(...)` literal in the file, resolved against the post-CHG-0150 workflow:

    MISSING ANCHOR: '          export CHECKS_FILE="$checks_file"'
    1 of 20 orphaned

The nineteen survivors are the control. If the enumeration had reported all twenty missing, the
conclusion would have been that the file no longer matches the workflow at all rather than that
one block was removed.

Checked explicitly because CHG-0150 edited the `resolve` job: `assertNotIn('mode="release"')`
and `assertNotIn("needs.resolve.outputs.mode == 'release'")` both still hold — the added mode
is `dry-run`.

## After

    Ran 49 tests in 8.495s
    OK

    classify-ci-paths tests passed

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| n/a — `--no-spec-change` | The suite goes from `FAILED (errors=1)` with the run aborting to `Ran 49 tests … OK`, and the anchor enumeration shows exactly one orphan of twenty, so the removal is targeted rather than a suppression. The nineteen resolving anchors are the control that the file still describes the workflow it reads |
