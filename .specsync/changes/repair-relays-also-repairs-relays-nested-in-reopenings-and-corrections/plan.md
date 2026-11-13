---
change: repair-relays-also-repairs-relays-nested-in-reopenings-and-corrections
artifact: plan
---

# Plan

## Approach

Generalize the ledger walk in `repair_relayed_decisions` from one array key per file to a list of
record locations, add `corrections.json` to the files it reads, and name nested places in the
text and JSON output.

## Out of scope

Routing a damaged correction relay's read refusal to `repair-relays` (#973) and checking an
embedded relay's decider on read (#970) are separate bug fixes that follow this one.

## Steps

1. Failing unit tests: a damaged relay in a reopening's `superseded_approval` is put back from
   `HEAD`; one in a correction's embedded approval with no committed copy is removed; the other
   records stay; the text place and JSON `within` name it.
2. The record-location walk and `corrections.json`.
3. The printer's place names, and the site's `repair-relays` paragraph.

## Risks

A malformed nested location (a reopening that is not an object) must be refused before anything
is written, as a malformed top-level record is today.

## Constraints consulted

REQ-change-046 (no blocking review is erased): no record is removed, only a damaged relay.
