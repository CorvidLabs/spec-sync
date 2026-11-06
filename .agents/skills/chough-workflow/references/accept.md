# Acceptance and native closeout

Acceptance is where the human decides whether the implementation is what they
wanted. Agent review, a passing command, scope approval or an actor label never
stands in for that decision.

## Prepare the review

Follow this order, so that one verification covers both the code and Chough's
records. Follow native status where it names a different next action.

1. When the delta adds or modifies requirements, run `specsync change check <id>`
   first, so the requirement text that cases link is materialized (cases already
   link it from the approved delta while the checkpoint records the change). It
   refuses while `tasks.md` has an unchecked box, so the cases in step 2 are never a
   box there.
2. Run the affected cases: `chough cases run <ID> --environment <env>`, plus the
   project lanes.
3. Record the gate: `chough checkpoint --awaiting-human "Implementation review of <id>"`.
4. Commit the source together with `.chough/`.
5. Run `specsync change check <id> --commit` (stage new source files first).
6. Push the branch, open or update the pull request, and observe hosted CI at the
   real tip. Report product tests, structural checks, native lifecycle state,
   provenance and platforms separately.
7. Present to the human: what changed, the current verification, the relevant case
   observations with freshness, open bugs and limits, and what was not checked.

Between the check in step 5 and finalize, nothing writes to `.chough/` and nothing is
committed. Committing Chough records or SpecSync bookkeeping does not make Chough
observations stale, but SpecSync's verification digest covers every non-ignored file,
including `.chough/`. SpecSync 6.1.2 ships an opt-in verification exclusion,
`verification_ignored_paths`, but Chough's own repository has not adopted it: its
entries are directory prefixes, so covering `.chough/session.json` would also hide
`.chough/cases/`, whose case definitions must stay verified. Unless a project lists
only directories that are safe to leave out, any Chough write in that window forces
another `change check`.

If native archive checks are red because review has not happened, say exactly that.
Do not claim all checks are green and do not bypass the gate.

## After the human decides

Only after a real implementation acceptance decision, the current agent records
it and closes natively, following native status rather than a remembered sequence.
The user reviews the implementation or PR; the agent handles these internal commands.
Use the acting project agent account for the native record and retain the actual
acceptance source separately. The actor label does not create that decision:

```sh
specsync change review <id> --reviewer "<agent-account>"
specsync change ship <id>        # or: specsync change finalize <id>
```

Do not commit between review and finalize. Only then clear the gate with `chough
checkpoint --clear-awaiting-human` and run `chough stage advance`, which moves accept
to done; it is refused until native state is `archived`. Record the actual acceptance
source in the checkpoint after archival, preserving the verification window above.
Push the archive commit and
the checkpoint on the same pull request, wait for required checks, then merge through
the repository's normal process. If several changes share a pull request, review and
finalize them one at a time. If evidence went stale, use the recovery action that
`specsync change status` names. Finishing never enters release.

## Compounding

Lessons can be recorded at any stage. Put them in the module's native lesson source
(`<module>.3md` later plane when present, otherwise the Lessons section of
`context.md`) with the source, the scope where it applies and the future action it
changes. `chough next` points to that source; Chough keeps no lesson store of its own.

Each lesson carries a certainty label:

- `observation`: something seen once, with what was run and where.
- `hypothesis`: a likely explanation not yet tested. It is a prompt to check, not a
  rule.
- `verified`: supported by a test or a repeated observation.

Each lesson names its source (an observation ID, a bug ID or a commit) and its limits
(where it applies and where it was not checked). A project decision, meaning a
checkpoint decision, a `docs/DECISIONS.md` entry or an approved contract, always
outranks a lesson. A lesson never changes a decision; if a lesson argues against one,
raise it with the human. A lesson that changes behavior or policy may need its own
native change. Write lessons from a change during closeout, inside the open native
change (see [librarian.md](librarian.md)). Archival does not mean no bug remains;
open bugs stay in the backlog.
