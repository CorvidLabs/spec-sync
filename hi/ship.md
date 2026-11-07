---
hi: 1
families: [SHIP]
---

# Delivering a change

## Intent

Once the scope is agreed, getting it delivered should be a short, ordered path with no ambiguity about what comes next. The contract edit is applied and checked against the real code, a person reviews the implementation against what was approved, and the whole package lands in a dated archive on the same pull request the code did. Nothing about that trail should ever need editing by hand — when evidence goes stale there is an audited way back, and the tool is honest that a recorded name is a claim, not proof of who did it.

SHIP-2.e was drafted by an agent for issue #633 (audit D7) and approved by Leif in the Herdr orc tab on 2026-11-05 at 20:19 MST, relayed by orc t18: "Approve", as proposed. Recorded by claude (ss-d7).

## Criteria

- **SHIP-1**  Once the scope is approved, the agreed contract edit is applied for me rather than retyped by hand.
  - **SHIP-1.a**  The code is checked against the contract as it now stands, in the same step.
  - **SHIP-1.b**  That check covers the contracts this change could break, not every contract in the project.
  - **SHIP-1.c**  It compares contract to code; running the project's tests stays the pipeline's job.
    - **SHIP-1.c.1**  Every current contract gives the same answer about the verification commands I configure: the change check does not run them.
    - **SHIP-1.c.2**  If a current contract starts promising to run them again, a test tells me.
  - **SHIP-1.d**  I can commit the verified result in the same step, so the evidence matches the tree it describes.
- **SHIP-2**  A person reviews the implementation against what was approved before it can ship.
  - **SHIP-2.a**  That review is recorded with the change rather than left in a chat thread.
  - **SHIP-2.b**  Editing the code after a review makes that review stale instead of letting it stand.
  - **SHIP-2.c**  A recorded name reads as a claim rather than proof of who did the work.
  - **SHIP-2.d**  A reviewer can block as plainly as they can pass.
  - **SHIP-2.e**  A review an agent records for a person can carry the same record, shown the same way.
- **SHIP-3**  Finalizing moves the whole package into a dated archive on the same pull request, so the trail travels with the merge.
  - **SHIP-3.a**  Finalizing needs verification and review that are both still current.
  - **SHIP-3.b**  I am warned plainly that merging before finalizing strands the evidence.
  - **SHIP-3.c**  When the tool cannot tell whether my change was already archived on the main branch, review and finalize stop and tell me how to find out, instead of archiving it a second time.
- **SHIP-4**  I can ask whether a change is ready to ship.
  - **SHIP-4.a**  The answer names the stage I am on and the one thing holding it.
  - **SHIP-4.b**  Readiness reflects what my pipeline actually reports rather than a guess.
- **SHIP-5**  One command takes a ready change the rest of the way.
  - **SHIP-5.a**  A change that is not ready is refused rather than half-shipped.
  - **SHIP-5.b**  I can ask for the same readiness report without anything being finalized.
- **SHIP-6**  When closed evidence goes stale there is an audited way to reopen it, rather than a temptation to edit the record by hand.
  - **SHIP-6.a**  Reopening asks who is authorizing it and why.
  - **SHIP-6.b**  That reason stays in the change's history for whoever reads it later.
- **SHIP-7**  I can audit the live changes and current contracts for coherence without rewalking the archives.
- **SHIP-8**  When I look at a pull request, I can tell whether its change is still open or already archived.
  - **SHIP-8.a**  A pull request that still has an open change does not look finished.
  - **SHIP-8.b**  A pull request whose changes are all archived says it is ready to merge once the checks are green.
- **SHIP-9**  When it is time to archive, one command archives the change, commits that archive, and tells me that merging is the step that is left.
  - **SHIP-9.a**  When it is not time, that same place tells me the one reason and the one command.
- **SHIP-10**  When more than one change is open, I am told which one to archive first.
- **SHIP-11**  When I want a pull request, one command archives the change and prepares the pull request.
  - **SHIP-11.a**  If the pull request is already open, that command leaves that pull request in place.
  - **SHIP-11.b**  That command does not merge the pull request.
- **SHIP-12**  When I start a change, no change someone merged before me is still open and standing in my way.
  - **SHIP-12.a**  A change closed long after it merged says so, rather than looking as if it had been checked at the time.
  - **SHIP-12.b**  A change closed late never counts as checked, for itself or for any change that comes after it.
  - **SHIP-12.c**  I can close such a change in one step, even when its old evidence can no longer be checked.
  - **SHIP-12.d**  Closing late asks who is authorizing it and why, and that reason stays in the change's history.
  - **SHIP-12.e**  A change whose pull request has not merged cannot be closed late.
  - **SHIP-12.f**  I can close several at once, and each one is recorded as if I had closed it alone.
  - **SHIP-12.g**  A change closed late does not trip up an older version of SpecSync, and that older version never counts it as checked.
