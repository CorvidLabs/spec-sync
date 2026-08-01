## MODIFIED

### REQUIREMENT REQ-github-007

Release qualification SHALL bind Ubuntu, macOS, and Windows results and final publication to one
immutable release-candidate commit, while ordinary product pull requests SHALL use Ubuntu as the
authoritative integration platform.

Acceptance Criteria

- Ordinary development/product PRs do not schedule macOS or Windows integration jobs.
- An RC branch is frozen by an immutable annotated `vX.Y.Z-rc.N` marker resolving to one full SHA.
- Active tag rulesets let humans create new RC markers but forbid their update/deletion, allow only
  a dedicated release GitHub App to create final tags, and forbid every actor from updating or
  deleting final tags. Its private key is available only to the protected `release` environment's
  promotion job, which mints a short-lived token scoped to the repository.
- Every required platform runs the same named Fledge RC lane at that exact SHA.
- Changing candidate content requires a new RC marker and fresh platform evidence.
- Promotion fails closed unless Ubuntu, macOS, and Windows are green for the unchanged candidate SHA.
- The final `vX.Y.Z` tag is created only after promotion succeeds and points to that same SHA.
- Release uploads independently reject mismatched marker, tag, checkout, evidence, or artifact SHA.
- Release-chain Actions and executables have independent immutable pins, and publication freshly
  revalidates tags, actual checkout, original platform evidence, and package hashes.
