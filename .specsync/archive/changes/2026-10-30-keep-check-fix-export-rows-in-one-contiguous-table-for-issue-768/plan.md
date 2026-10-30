---
change: keep-check-fix-export-rows-in-one-contiguous-table-for-issue-768
artifact: plan
---

# Plan

## Approach
Select one contiguous table instead of independently selecting width and end offset.

## Out of scope
Generate binding, #813, #800, closed #809, lifecycle policy, security changes, and live services.

## Steps
1. Reproduce the #768 multi-table case with CLI regressions.
2. Update the existing module contract and implement a private scanner helper.
3. Run focused tests, formatting, full verification, strict coverage and trust.
4. Commit locally; request scope decision and publication approval separately.

## Risks
Table heuristics must preserve existing fence length rules and kind routing. Existing fix regressions exercise those paths.

## Constraints consulted
cmd_check.3md: deterministic local repairs, preservation of fenced examples, and no lifecycle coupling in check. No principles added.
