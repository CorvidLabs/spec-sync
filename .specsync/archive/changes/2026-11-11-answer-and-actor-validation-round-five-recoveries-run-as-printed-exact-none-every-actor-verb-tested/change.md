---
id: answer-and-actor-validation-round-five-recoveries-run-as-printed-exact-none-every-actor-verb-tested
state: archived
type: bug_fix
base_commit: 1c07640aa5960c97c0d7fe9b7edb49aa42b8556b
---

# Answer and actor validation round five: recoveries run as printed, exact none, every actor verb tested

## Intent

answer and actor validation round five: recoveries run as printed, exact none, every actor verb tested

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- owner recovery scaffolds a configured owner with no spec before declaring it; empty constraints keep their none guidance; an exact none is refused as ambiguous when something is named none; registered-but-missing specs and unreadable registries get their own messages; zero-width and directional-mark characters are refused in actors; reopen, retire, correct, correct-owner, reconcile-history and the doctor late close each have a bad-actor test; REQ-change-150, invariant 56 and the lesson match

## No-spec Rationale

Not applicable
