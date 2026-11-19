---
change: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
artifact: plan
---

# Plan

## Approach

Narrow what a terminal-evidence verdict reads instead of adding a rule: load the roster one change
at a time in `terminal_evidence_summary`'s three paths, give a broken change its own failure, judge
every other change with the changes that loaded, and name a failed candidate when a successor search
for a changed input finds no cover.

## Out of scope

- The commands that write: `check`, `review`, `finalize`, `ship`, `archive` and the archive
  preflight keep their roster loads and refusals.
- A warning about other broken changes in a change's status (Leif chose none).
- What `change audit` reports; it reports the broken change as today.

## Steps

1. Failing tests: tester2's S1 and S2 as unit fixtures in `src/change_tests/terminal_evidence_isolation.rs`,
   read through the scoped and the unscoped path; the dependency cases; an integration test through
   the binary.
2. Load the roster per change for terminal evidence, in `src/change/validate_loaded_change.rs`.
3. Use it in the three paths of `terminal_evidence_summary`, and name a failed candidate in a
   successor search, in `src/change/consider_accepted_evidence_anchor.rs`.
4. CHANGELOG Fixed line.
5. `fledge lanes run verify`, then the pre-push gate.

## Risks

- A failure that should count is dropped. Held by the dependency tests: a broken candidate
  successor, an unreadable `state.json` during a search, a retirement whose replacement is broken.
- A command that writes changes its behaviour. Held by running check, review and finalize on the
  S1 fixture with the same exit codes and messages as on main.

## Constraints consulted

- REQ-change-119, REQ-change-012 and REQ-change-024, which already say what a verdict depends on.
- Leif's rule 5: prefer narrowing a dependency over adding a rule.
- The read cache of REQ-change-119: every read stays identical to a live read.
