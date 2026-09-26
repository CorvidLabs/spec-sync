---
change: required-ci-gate-fails-when-the-lifecycle-gate-fails
artifact: research
---

# Research

## What each job runs on

Read from `.github/workflows/ci.yml` on `cddc39e4`:

| Job | Needs | `if:` |
|---|---|---|
| `classify` | none | none |
| `preflight` | none | none |
| `lifecycle-gate` | classify, preflight | none |
| `test`, `audit`, `coverage` | classify, lifecycle-gate | `full` |
| `spec-check` | classify, lifecycle-gate | not `archive_only`, not `review_only` |
| `fmt`, `hi-check`, `action-consumer` | classify | `full` |
| `validate-action` | classify | not `archive_only`, not `review_only` |
| `site` | classify | `full` or `site` |
| `vscode-extension` | classify | `full` or `vscode` |
| `corvid-pet` | 13 jobs | `always()`, pull request, `review_required` |
| `implementation-gate` | 12 jobs (no preflight, no lifecycle-gate) | `always()` |
| `ci-gate` | classify, implementation-gate | `always()` |
| `attest` | ci-gate | ci-gate success, push to `main` |

A job whose `if:` has no status function gets an implicit `success()`, so it is skipped
whenever a dependency did not succeed.

## Observed check runs

| Commit | Lifecycle gate | test/audit/coverage/spec-check | Implementation ready | Required CI gate |
|---|---|---|---|---|
| #795 `bb1d80f2` (unapproved draft) | failure | skipped | success | success |
| #795 `f50bfdd5` (product tip) | success | success | success | success |
| #795 `96f948d9` (archive tip) | success | success | success | success |
| #790 head (specs/lifecycle-only) | success | test/audit/coverage skipped, spec-check success | success | success |
| #791 head (archive tip) | success | success | success | success |

The first row is #796. The others show that `preflight` and `lifecycle-gate` succeed on every
lane a lifecycle pull request passes through, so requiring them costs nothing.

## Classify lanes

From `.github/scripts/classify-ci-paths.sh` and `select-ci-lane.sh`: `full` for product paths,
workflows, scripts and anything unrecognized (so `docs/**` and `*.md` run the full lane); `site`
or `vscode` alone for those trees; nothing selected for `specs/**` and `.specsync/changes/**`;
`archive_only` for a proven one-change workflow-v2 archive move; `legacy_archive_only` together
with `full` for workflow-v1; and `review_only` for a review-only tip. Pushes classify from a
name-only diff, so they are never archive-only or review-only. `review_required` is computed on
every event, but `corvid-pet` also requires `pull_request`.

## `attest` never runs on `main`

The last 15 pushes to `main` all show `Record attestation: skipped`, including runs where
`Required CI gate` succeeded. This is the transitive `success()` above and a separate defect.
