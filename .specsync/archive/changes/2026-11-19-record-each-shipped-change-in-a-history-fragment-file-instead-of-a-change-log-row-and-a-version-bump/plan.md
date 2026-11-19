---
change: record-each-shipped-change-in-a-history-fragment-file-instead-of-a-change-log-row-and-a-version-bump
artifact: plan
---

# Plan

## Approach

Record each materialized change in its own history fragment instead of a Change Log row and a `version:` bump, keep a row written by an earlier SpecSync as a valid record, validate fragments in `check`, and render the history with a new `specsync history` command. The failing tests come first.

## Out of scope

- `requirements.md` appends from two ADDED deltas to one module, which still conflict in plain git.
- Converting existing Change Log rows into fragments, or rewriting any row.
- The generated agent instructions that tell a hand workflow to bump `version:` and add a row.
- Gating released 6.3.0 out of a repository that holds fragments.
- No tag, release or publish.

## Steps

1. Write the failing tests (`tests/integration/spec_history.rs`) and record how they fail on 65864de8.
2. After scope approval: the fragment helpers and the recorded predicate (REQ-change-159, REQ-change-160), and the materializer change.
3. The canonical-file enumerations, send-back's reads and advice.
4. Fragment validation in `check` and the snapshot digest (REQ-validator-048, REQ-hash-cache-006).
5. `specsync history` and its grammar and routing (REQ-cmd-change-037, REQ-cli-args-026, REQ-cli-014).
6. Update the tests that asserted a bump or a row, the specs' Public API and invariants, a `change.3md` lesson, the site docs, AGENTS.md's quick reference and CHANGELOG.
7. `change check --commit`, the verify lane and pre-push.

## Risks

- **Mixed binaries.** A released 6.3.0 `change check` adds a row and a bump to a change this build recorded by fragment. `check` reports the double record, and the docs say every machine running change commands needs this release.
- **Hidden evidence uses.** Something else may read the row as evidence. The full suite runs after the change.

## Constraints consulted

- MERGE-1 and MERGE-4.
- #741: the record of a change in a module is the evidence that its materialization ran.
- #843: the stale MODIFIED refusal is unchanged.
