---
change: run-the-archive-gate-before-every-ci-job-so-nothing-runs-or-merges-until-every-change-is-archived
artifact: plan
---

# Plan

## Approach

Make `Archive` the root of the CI and Trust job graphs, run the full lane on the archived tip,
expose the same gate in the GitHub Action, and rewrite ship guidance for the new order.

## Out of scope

Changing `archive-readiness.sh` semantics, the archive sentence, release workflows, and the
per-repository rollout PRs (separate pull requests in each repository).

## Steps

1. `action.yml`: `require-archived`, `archive-gate-only`, `archived` output; gate step first.
2. `ci.yml`: Archive first, dogfooding `archive-gate-only`; classify/preflight need it; the
   implementation gate requires it; classify uses the whole-PR answer only; corvid-pet runs
   on every full PR; action-consumer exercises pass and block.
3. `trust.yml`: `trust` needs an Archive job.
4. Remove `select-ci-lane.sh` and its tests; extend `test-required-ci-gate.py`.
5. `change ship-status`/`ship` guidance, check-runs on HEAD first; `Archive` is trust-relevant.
6. Agent skills/commands, AGENTS.md, docs and site describe the local verify → review → ship flow.

## Risks

A failure on the archived tip is found after archival; recovery is a fix commit on top with
specs in sync. Consumers on `@v6.1.1` silently skip the gate, so docs say to pin a newer ref.

## Constraints consulted

`change status`/`ship` never say to merge while a change is open; the Archive sentence stays
identical in `archive-readiness.sh` and `pull_request_archive_sentence`; no new `gh` paths.
