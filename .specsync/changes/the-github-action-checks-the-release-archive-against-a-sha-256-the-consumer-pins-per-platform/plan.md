---
change: the-github-action-checks-the-release-archive-against-a-sha-256-the-consumer-pins-per-platform
artifact: plan
---

# Plan

## Approach

Add the input and a small checked script, and document it on the Action page.

## Out of scope

Signing or immutable GitHub Releases (the issue's other option).

## Steps

1. Red: the script test with no script.
2. The script, the `action.yml` wiring, the Fledge task and CI step, and the docs.
3. The script test, the action safety tests in `tests/integration/config.rs`, and the workflow
   runtime pin check.

## Risks

A consumer who pins one platform and adds a runner type gets a clear refusal naming the missing
platform line. This is intended: fail closed.

## Constraints consulted

`action.yml` never evaluates inputs in the shell (config.rs safety test).
