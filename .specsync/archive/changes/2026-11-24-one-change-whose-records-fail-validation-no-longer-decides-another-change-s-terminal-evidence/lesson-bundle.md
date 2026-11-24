# Lesson bundle: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: One change whose records fail validation no longer decides another change's terminal evidence
- **Kind**: BugFix
- **Specs**: change
- **Paths**: src/change/consider_accepted_evidence_anchor.rs, src/change/validate_loaded_change.rs, src/change_tests.rs, src/change_tests/terminal_evidence_isolation.rs, tests/integration.rs, tests/integration/, CHANGELOG.md
- **Acceptance**: With another change whose history fails validation (open and archived at one commit, as in #1104), an archived change whose own package and history authenticate reads `authenticated-history`, and an accepted change whose delivery inputs are unchanged reads `exact`, in `change status`, `change show`, `change list` and `change ship-status`, with and without a warm read cache.
- **Acceptance**: The broken change itself still reports its error in `change status` and `change show` (exit 1); `change audit --archives` still exits 1, naming the change when it is only an archive, and `change audit` still exits 1 with its error when the change is active.
- **Acceptance**: An accepted change whose changed input a broken change would cover reads `stale`, and the reason names that change and its failure; the same holds for a candidate successor whose `state.json` does not read, when the judged change is searching for a successor.
- **Acceptance**: A retired change whose replacement fails validation still reports that failure in its own status.
- **Acceptance**: `change check`, `change review`, `change finalize`, `change ship` and `change archive` refuse and pass exactly as before, with and without a broken change in the roster.

## Evidence

- Verification commit: `a8d3b46098e9099104c40147032d5120a69cd8f2`
- Base commit: `2cc0897fc00eee7db2b07f5134ba05639af39018`
- Verified by: `specsync check --spec change`

## From the change's context.md

# Context

Issue #1104. `terminal_evidence_summary` (`src/change/consider_accepted_evidence_anchor.rs`) answers
"is this change's evidence still good?" for `change status`, `change show`, `change list` and
ship-status. In each of its three paths it first loads the whole roster with
`list_all_changes_checked`, which validates every change, the history of every archive included. If
any one change fails, that error becomes every change's verdict: `corrupt-history` for each archived
change and `stale` for each open one, with the broken change's reason.

tester2 reproduced it on SpecSync 6.3.0 and on a copy of starling (`tester2/prs/g298/notes.md`, S1
and S2): a merge left one change both open and archived at one commit, and 8 to 13 other archived
changes then read `corrupt-history` with that change's reason, their own evidence intact. `check`,
`review` and `finalize` were never stopped by it.

Leif approved the definition as written on 2026-11-19 at about 15:12Z ("approve 1104 as written"),
relayed by his planning session (hc t4c) through ss-lead (hc t58), with its three choices as
recommended: a bug fix with no new requirement text, an unreadable `state.json` counting against a
successor search only, and no warning line in other changes' status. Frozen copy:
`ss-lead/gates/1104-definition.md`.

## From the change's design.md

# Design

Each verdict reads only the records it depends on (REQ-change-119, REQ-change-012, REQ-change-024):

| Change being judged | What its verdict reads | A failure there |
|---|---|---|
| Archived | its own package and its own history | its own verdict, as today |
| Accepted, every delivery input unchanged | its own package and history | its own verdict, as today |
| Accepted, an input changed | the above, and every candidate successor for that input | counts: the change is `stale` and the reason names the candidate and its failure |
| Retired | its retirement receipt and its replacement | its own refusal, as today |

The roster is loaded one change at a time: a change whose records fail validation keeps its own
failure, and every other change is judged with the changes that loaded. When a successor search for
a changed input finds no cover, a candidate that failed to load is named with its failure: a
change whose loaded record declares the input, or one whose `state.json` does not read at all. An
archived change, and an accepted change with unchanged inputs, never search.

`check`, `review`, `finalize`, `ship` and `archive` load the roster themselves and are not changed.

## From the change's testing.md

# Testing

Failing first: on main, the S1 and S2 fixtures read `corrupt-history` for a change whose own
evidence is intact, as tester2 found.

- `src/change_tests/terminal_evidence_isolation.rs`
  - S1: a change archived before another change is left open and archived at one commit, then
    cleaned up. It reads `authenticated-history`, through the scoped read (warm cache) and the
    unscoped one.
  - S2: a change created and archived after that commit and its cleanup. The same.
  - An accepted change whose inputs are unchanged reads `exact` beside the broken change.
  - The broken change's own status still refuses, naming its failure.
  - A broken change that declares an accepted change's changed input: `stale`, naming it.
  - An unreadable `state.json`: counted in a successor search, not otherwise.
  - A retired change whose replacement fails: its own refusal, as today.
- `tests/integration/`: through the binary on the S1 fixture, `change status`, `change list`,
  `change ship-status` and `change audit --archives`; `check`, `review` and `finalize` of a third
  change, with main's exit codes and messages.
- The whole suite, since every status read goes through `terminal_evidence_summary`.

## Where these lessons go

- `specs/change/change.3md`
