---
hi: 1
families: [SHIP]
---

# Delivering a change

## Intent

Once the scope is agreed, getting it delivered should be a short, ordered path with no ambiguity about what comes next. The contract edit is applied and checked against the real code, a person reviews the implementation against what was approved, and the whole package lands in a dated archive on the same pull request the code did. Nothing about that trail should ever need editing by hand — when evidence goes stale there is an audited way back, and the tool is honest that a recorded name is a claim, not proof of who did it.

SHIP-2.e was drafted by an agent for issue #633 (audit D7) and approved by Leif in the Herdr orc tab on 2026-11-05 at 20:19 MST, relayed by orc t18: "Approve", as proposed. Recorded by claude (ss-d7).

SHIP-1.c.3 was drafted by the agent for issue #893. Leif approved it as drafted in the Herdr orc tab on 2026-11-06 at about 19:35 MST ("3yes 3b-yes": the note and its JSON list, and emptying spec-sync's own `verification_commands`), relayed by orc t18. Recorded by the ss-triage Claude agent (t15w). Leif later narrowed the 3b policy choice through orc t18: empty only `verification_commands` and keep the 13 component modules and 3 strict commands, so this repository's note remains visible; the criterion text is unchanged.

SHIP-13, SHIP-13.a, SHIP-13.b, SHIP-13.c and SHIP-13.d were drafted by an agent for issues #657 and #688, from Leif's choices: for #657 on 2026-11-13 at about 04:04Z, "It may archive (Recommended)", meaning the archive records which change covered the shared file; for #688 on 2026-11-18, one sentence in each of the two archive refusals, naming the other change and saying it is blocked too. Leif approved the definition that carries them, with these five sentences marked as the agent's draft, in a Claude Code chat with his planning session (hc t4c) on 2026-11-19 at about 14:17Z: "Approve as written". They are the agent's wording, which he chose with those words; he did not write them. Relayed by hc t4c through the spec-sync lead agent (ss-lead, hc t58); recorded by the ss-builder Claude agent (hc t59).

## Criteria

- **SHIP-1**  Once the scope is approved, the agreed contract edit is applied for me rather than retyped by hand.
  - **SHIP-1.a**  The code is checked against the contract as it now stands, in the same step.
  - **SHIP-1.b**  That check covers the contracts this change could break, not every contract in the project.
  - **SHIP-1.c**  It compares contract to code; running the project's tests stays the pipeline's job.
    - **SHIP-1.c.1**  Every current contract gives the same answer about the verification commands I configure: the change check does not run them.
    - **SHIP-1.c.2**  If a current contract starts promising to run them again, a test tells me.
    - **SHIP-1.c.3**  When I have configured verification commands, the change check and the audit tell me they did not run them, so a pass is not read as my tests passing.
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
- **SHIP-13**  An accepted change does not go stale because a later change I am still working on edits a file they share.
  - **SHIP-13.a**  I am told which later change is covering that file for now.
  - **SHIP-13.b**  It can be archived while it is covered that way, and its archive says which change covered it.
  - **SHIP-13.c**  If that later change is sent back or closed instead of accepted, I am told the file has no accepted change behind it.
  - **SHIP-13.d**  When two changes each stop the other from being archived, each refusal names the other and says it is blocked too.
