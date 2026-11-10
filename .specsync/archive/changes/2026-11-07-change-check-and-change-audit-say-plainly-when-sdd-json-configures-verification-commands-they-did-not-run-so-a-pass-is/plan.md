---
change: change-check-and-change-audit-say-plainly-when-sdd-json-configures-verification-commands-they-did-not-run-so-a-pass-is
artifact: plan
---

# Plan

## Approach

Add one command-layer presentation helper for the ordered, deduplicated configured-command list. Reuse it for the single text note and both JSON projections. Keep verification and lifecycle policy untouched. Empty only this repository's top-level list, update the contract and user documentation, and adjust only integration assertions incompatible with the additive note.

## Out of scope

Running configured commands, changing which commands verification selects, changing verdicts or lifecycle state, changing `change check --commit --json`'s missing success document, and emptying component or strict lists. No new HI or requirement IDs beyond the reserved SHIP-1.c.3 and REQ-cmd-change-unrun-commands-001.

## Steps

1. Implement the ordered/deduplicated helper and shared text note on plain check, committed check, and audit; add the two JSON keys.
2. Empty only `verification_commands`; update site configuration, ADOPTING, the cmd_change contract and companions, HI decision provenance, INTENT, and CHANGELOG.
3. Run the committed discriminators and the whole integration suite, fixing only assertions that require empty stderr where the note is now specified.
4. Run strict spec checks, `fledge lanes run verify`, and `fledge trust verify`; record Chough case evidence.
5. Commit and run `change check --commit`, request Leif's implementation review through orc t18, then review and ship without an intervening commit.

## Risks

Additive stderr can break fixtures that incorrectly require silence; the full integration suite is required. Duplicate commands across lists must not inflate N; stable deduplication and exact JSON ordering are pinned. Malformed or missing policy must preserve existing behavior rather than being hidden by the presentation helper.

## Constraints consulted

`specs/cmd_change/cmd_change.3md` z=4 requires disclosure on both the plain and committed output paths with structured parity and no digest in text. z=1 keeps lifecycle policy out of the command adapter. `specs/change/change.3md` z=15 confirms configured commands are never executed. No principles file is configured.

## Estimate and ownership

One small implementation increment plus one integration-suite compatibility increment. Main uncertainty is the number of fixtures with empty-stderr assertions. `chough timing` has too few samples for a history range (one bug fix, zero human gates), so no time range is claimed. One agent owns all declared files; no parallel writers.
