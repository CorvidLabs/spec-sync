---
id: one-change-whose-records-fail-validation-no-longer-decides-another-change-s-terminal-evidence
state: approved
type: bug_fix
base_commit: 2cc0897fc00eee7db2b07f5134ba05639af39018
---

# One change whose records fail validation no longer decides another change's terminal evidence

## Intent

one change whose records fail validation no longer decides another change's terminal evidence

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- With another change whose history fails validation (open and archived at one commit, as in #1104), an archived change whose own package and history authenticate reads `authenticated-history`, and an accepted change whose delivery inputs are unchanged reads `exact`, in `change status`, `change show`, `change list` and `change ship-status`, with and without a warm read cache.
- The broken change itself still reports its error in `change status` and `change show` (exit 1); `change audit --archives` still exits 1, naming the change when it is only an archive, and `change audit` still exits 1 with its error when the change is active.
- An accepted change whose changed input a broken change would cover reads `stale`, and the reason names that change and its failure; the same holds for a candidate successor whose `state.json` does not read, when the judged change is searching for a successor.
- A retired change whose replacement fails validation still reports that failure in its own status.
- `change check`, `change review`, `change finalize`, `change ship` and `change archive` refuse and pass exactly as before, with and without a broken change in the roster.

## No-spec Rationale

No requirement text changes. REQ-change-119 computes an archived change's terminal evidence for that
change alone; REQ-change-012 has an archive enter current-input recursion only when selected as a
successor; REQ-change-024 requires a selected candidate successor to be valid. `terminal_evidence_summary`
read the whole roster first and gave its first error to every change, so one change whose history
fails validation made every other archived change read `corrupt-history` and every open one `stale`
with that change's reason (#1104). Each verdict now reads only its own records and the candidates it
selects; a failure among those still counts.
