---
hi: 1
families: [CI]
---

# Proving it in the pipeline

## Intent

The local check only matters if the same check stands between a drifted contract and the main branch. Dropping it into a pipeline should take one step with nothing to build and a version I can pin. When it finds something, it should say so where the conversation already is — on the pull request — rather than buried in a build log nobody opens.

## Criteria

- **CI-1**  The same check I run locally runs in my pipeline as a ready-made step, with nothing to build.
  - **CI-1.a**  I can pin the exact version of the step and of the binary it fetches.
  - **CI-1.b**  The fetched binary is verified against a published checksum before it is allowed to run.
- **CI-2**  The pipeline validates from scratch rather than trusting a cache nobody committed.
- **CI-3**  Drift shows up as a comment on the pull request, where the discussion already is.
  - **CI-3.a**  Re-running updates that comment instead of adding another one.
- **CI-4**  A branch's export drift is reported name by name, limited to the modules whose files it touched.
  - **CI-4.a**  The comparison works out the branch's real base on its own rather than making me name it.
- **CI-5**  A contract with drift can raise an issue, so the gap does not vanish with the build log.
  - **CI-5.a**  An issue reference already written into a contract can be checked as real.
