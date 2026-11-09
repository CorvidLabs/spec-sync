---
hi: 1
families: [CI]
---

# Proving it in the pipeline

## Intent

The local check only matters if the same check stands between a drifted contract and the main branch. Dropping it into a pipeline should take one step with nothing to build and a version I can pin. When it finds something, it should say so where the conversation already is — on the pull request — rather than buried in a build log nobody opens.

Approved by Leif in the Herdr orc tab on 2026-11-05 20:30 MST (relayed by orc t18, choice "Approve") for issue #886, after a read-only token made the Action's comment step fail a passing check with HTTP 403. CI-3.b is that intent, worded as proposed and approved.

CI-6 is Leif's own sentence, captured as he gave it ("Capture it, fix CI") in the Herdr orc tab on 2026-11-06 (relayed by orc t18). He wrote it after PR #941, which changed only `.chough/` records, never started CI and so could not get the required check.

CI-7 is captured from Leif's answer in the Herdr orc tab on 2026-11-06 ("Broad", relayed by orc t18), choosing the broader of two wordings the agent proposed for issue #977, a unit test from #969 that could fail when two reviews landed in the same second. Recorded by the ss-977 Claude agent.

CI-9 and CI-10 are captured from Leif's answer in the Herdr orc tab on 2026-11-06 ("Both A", relayed by orc t18). He chose the first of two wordings the agent proposed for each of #798 and #799. In #798, `Record attestation` skipped on every push to main because its implicit `success()` saw the PR-only review job skipped upstream. In #799, that review job's pull-request summary had no row for the preflight or the lifecycle gate, and called every skipped check "Not selected". Recorded by the ss-linux-arm64 Claude agent.

## Criteria

- **CI-1**  The same check I run locally runs in my pipeline as a ready-made step, with nothing to build.
  - **CI-1.a**  I can pin the exact version of the step and of the binary it fetches.
  - **CI-1.b**  The fetched binary is verified against a published checksum before it is allowed to run.
- **CI-2**  The pipeline validates from scratch rather than trusting a cache nobody committed.
- **CI-3**  Drift shows up as a comment on the pull request, where the discussion already is.
  - **CI-3.a**  Re-running updates that comment instead of adding another one.
  - **CI-3.b**  A comment that cannot be posted never turns a passing check red; I am told why and can still read the result.
- **CI-4**  A branch's export drift is reported name by name, limited to the modules whose files it touched.
  - **CI-4.a**  The comparison works out the branch's real base on its own rather than making me name it.
- **CI-5**  A contract with drift can raise an issue, so the gap does not vanish with the build log.
  - **CI-5.a**  An issue reference already written into a contract can be checked as real.
- **CI-6**  A pull request that only changes Chough records still gets the checks the branch requires.
- **CI-7**  A failing test always means something is actually wrong, never that it ran at an unlucky moment.
- **CI-9**  Every commit on main that passes its required checks gets its attestation recorded.
- **CI-10**  The summary on a pull request never looks green when a check that gates it failed or was blocked.
