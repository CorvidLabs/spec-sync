---
hi: 1
families: [SHIP]
---

# Delivering a change

## Intent

Once the scope is agreed, getting it delivered should be a short, ordered path with no ambiguity about what comes next. The contract edit is applied and checked against the real code, a person reviews the implementation against what was approved, and the whole package lands in a dated archive on the same pull request the code did. Nothing about that trail should ever need editing by hand — when evidence goes stale there is an audited way back, and the tool is honest that a recorded name is a claim, not proof of who did it.

## Criteria

- **SHIP-1**  Once the scope is approved, the agreed contract edit is applied for me rather than retyped by hand.
  - **SHIP-1.a**  The code is checked against the contract as it now stands, in the same step.
  - **SHIP-1.b**  That check covers the contracts this change could break, not every contract in the project.
  - **SHIP-1.c**  It compares contract to code; running the project's tests stays the pipeline's job.
  - **SHIP-1.d**  I can commit the verified result in the same step, so the evidence matches the tree it describes.
- **SHIP-2**  A person reviews the implementation against what was approved before it can ship.
  - **SHIP-2.a**  That review is recorded with the change rather than left in a chat thread.
  - **SHIP-2.b**  Editing the code after a review makes that review stale instead of letting it stand.
  - **SHIP-2.c**  A recorded name reads as a claim rather than proof of who did the work.
  - **SHIP-2.d**  A reviewer can block as plainly as they can pass.
- **SHIP-3**  Finalizing moves the whole package into a dated archive on the same pull request, so the trail travels with the merge.
  - **SHIP-3.a**  Finalizing needs verification and review that are both still current.
  - **SHIP-3.b**  I am warned plainly that merging before finalizing strands the evidence.
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
