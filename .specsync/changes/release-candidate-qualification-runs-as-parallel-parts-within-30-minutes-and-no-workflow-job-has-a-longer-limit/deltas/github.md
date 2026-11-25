## MODIFIED

### REQUIREMENT REQ-github-007

Release qualification SHALL bind Ubuntu and macOS results and final publication to one
immutable release-candidate commit, while ordinary product pull requests SHALL use Ubuntu as the
authoritative integration platform. Windows is not a 6.0 qualification or publication target.

Acceptance Criteria

- Ordinary development/product PRs do not schedule macOS or Windows integration jobs. A pull
  request that changes the release-candidate lane's own definition is not ordinary: it rehearses
  the lane on macOS (REQ-github-033).
- An RC branch is frozen by an immutable annotated `vX.Y.Z-rc.N` marker resolving to one full SHA.
- Two active tag rulesets let humans create new RC markers and final tags but forbid every actor,
  with no bypass, from updating or deleting either. Qualification validates exactly those two —
  `SpecSync immutable RC tags` over `refs/tags/v*.*.*-rc.*` and `SpecSync immutable final tags`
  over `refs/tags/v*.*.*` excluding the RC pattern — and fails closed on any broadening.
- Final-tag creation is not restricted to a release GitHub App, the final tag is created by the
  release workflow's own `GITHUB_TOKEN` rather than a separate release identity, and promotion is
  not behind a deployment-environment gate. Qualification states all three omissions on every run,
  including successful ones, and fails if that statement is ever empty; it never reports a
  protection it does not check.
- Permission to write a ref is granted to the promotion and publication jobs alone. The release
  workflow's default permissions stay read-only, so no other job in the lane can create, move, or
  delete a tag.
- Every required platform runs the same named Fledge RC lane at that exact SHA, as the same named
  parts (REQ-github-031).
- Changing candidate content requires a new RC marker and fresh platform evidence.
- Promotion fails closed unless Ubuntu and macOS are green for the unchanged candidate SHA.
- Windows is neither built nor qualified as of 6.0; the Action refuses a Windows runner.
- The final `vX.Y.Z` tag is created only after promotion succeeds and points to that same SHA.
- Release uploads independently reject mismatched marker, tag, checkout, evidence, or artifact SHA.
- Release-chain Actions and executables have independent immutable pins, and publication freshly
  revalidates tags, actual checkout, original platform evidence, and package hashes.

## ADDED

### REQUIREMENT REQ-github-031

Release-candidate qualification SHALL run each required platform's lane as parallel parts, four slices of the test set and one release build, and SHALL count a platform as qualified only when every part passed for the candidate and the slices together are the whole test set.

Acceptance Criteria
- Each part is its own job. A test slice runs the suite under cargo-nextest with `--partition slice:K/4`; the release-build part runs the release race guards and the release build.
- Each part writes a record bound to the candidate's tag and SHA that names its platform, part and outcome. A test slice's record also carries the number of tests in its slice and in the whole list, both from cargo-nextest's own listing.
- A platform's evidence says success only when its five part records are all present and successful, are bound to the same candidate and workflow revision, report the same whole-list count, and the four slice counts add up to it.
- A part that is missing, duplicated, unexpected, failed or bound to another candidate, or slice counts that do not add up, leave that platform's evidence unsuccessful, and the candidate gate fails.
- The platform evidence that promotion and publication validate keeps its form: one record for Ubuntu and one for macOS.
- No test is dropped, ignored or weakened to make a part faster, and the whole lane still runs locally as one command, `fledge lanes run release-candidate`.

### REQUIREMENT REQ-github-032

Every job in every workflow SHALL declare a time limit of at most 30 minutes. A job that outgrows its limit fails, and the limit is not raised past 30 to fit it.

Acceptance Criteria
- A contract test reads every workflow file and fails when a job declares no `timeout-minutes`, or one above 30.
- Each release-candidate part declares 30 minutes, on both platforms.
- The contract test runs in CI and in the local verify lane.

### REQUIREMENT REQ-github-033

The release-candidate lane SHALL be rehearsable without a candidate, as the same parts on the same runner images with the same pinned tools, commands and time limit, so that a lane that has grown is found before a release candidate is cut.

Acceptance Criteria
- The rehearsal starts by hand for any ref, once a week on the default branch, and for a pull request that changes the release workflow, the rehearsal workflow, the lane definition or the part script.
- A contract test fails when the rehearsal's parts, runner images, tool pins, commands or time limit differ from the release workflow's qualification.
- A rehearsal writes no candidate evidence and publishes no candidate gate.
- The release runbook has the releaser rehearse the commit before tagging a candidate from it.
