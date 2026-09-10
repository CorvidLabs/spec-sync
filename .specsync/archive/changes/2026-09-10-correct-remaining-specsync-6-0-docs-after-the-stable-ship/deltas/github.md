## MODIFIED

### REQUIREMENT REQ-github-007

Release qualification SHALL bind Ubuntu and macOS results and final publication to one
immutable release-candidate commit, while ordinary product pull requests SHALL use Ubuntu as the
authoritative integration platform. Windows is not a 6.0 qualification or publication target.

Acceptance Criteria

- Ordinary development/product PRs do not schedule macOS or Windows integration jobs.
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
- Every required platform runs the same named Fledge RC lane at that exact SHA.
- Changing candidate content requires a new RC marker and fresh platform evidence.
- Promotion fails closed unless Ubuntu and macOS are green for the unchanged candidate SHA.
- Windows is neither built nor qualified as of 6.0; the Action refuses a Windows runner.
- The final `vX.Y.Z` tag is created only after promotion succeeds and points to that same SHA.
- Release uploads independently reject mismatched marker, tag, checkout, evidence, or artifact SHA.
- Release-chain Actions and executables have independent immutable pins, and publication freshly
  revalidates tags, actual checkout, original platform evidence, and package hashes.
