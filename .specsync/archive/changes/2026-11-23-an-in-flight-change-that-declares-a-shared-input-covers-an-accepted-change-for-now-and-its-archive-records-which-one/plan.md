---
change: an-in-flight-change-that-declares-a-shared-input-covers-an-accepted-change-for-now-and-its-archive-records-which-one
artifact: plan
---

# Plan

## Approach

Let an in-flight later change that declares a changed input's module and path cover it for now, report that as a warning, let the accepted change archive with a record of the coverer, read that record back on the archive, and add #688's sentence to each of the two archive refusals.

## Out of scope

Downgrading every stale input to a warning; any change to an archive's own evidence, which stays authenticated history.

## Steps

1. Red tests: provisional coverage, the draft and undeclared-module cases, successor and retirement refusals, reopen, archive through `change archive` (workflow v1) and `change finalize` (workflow v2), the sent-back coverer after the archive, and the two refusals.
2. The in-flight coverer in the accepted-input walk; `provisionally-covered` evidence and reason.
3. Audit warning; doctor, successor, retirement and reopen decisions.
4. `coverage.json` at archive and finalize; status, show and `audit --archives` read it.
5. #688's two sentences.
6. Docs, CHANGELOG; the released-6.3.0 compatibility run; full suite, `change audit --strict`.

## Risks

A new validity value reaches status JSON; a consumer that expects `exact` or `successor-covered` for an accepted change sees `provisionally-covered`. Released 6.3.0 still calls such a change stale, so a repository that pins SpecSync moves its pin with this release.

## Constraints consulted

The accepted-input currency rules (successor coverage, REQ-change-012, REQ-change-024, REQ-change-119, REQ-change-135), the archive preflight, reopen's refusal of exact or successor-covered evidence, retirement replacements, and Leif's rule 5 (narrow a dependency rather than add a rule).
