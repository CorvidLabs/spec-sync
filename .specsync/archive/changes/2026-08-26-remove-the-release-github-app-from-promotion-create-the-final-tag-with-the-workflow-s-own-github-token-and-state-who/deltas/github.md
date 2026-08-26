# Release promotion identity scope delta

## MODIFIED

### SPEC SECTION Invariants

Every path that can be merged can reach the required CI gate; a path the CI
workflow cannot trigger can never report the gate and blocks its pull request.

Release qualification verifies exactly the tag protections this repository actually has, and names
every protection it does not verify on every run, green runs included. A gate that demands an
unprovisioned policy fails on every candidate and therefore verifies nothing — it is not a safe
default, because the protections that DO exist are never reached. Dropping a check from the gate is
permitted; dropping it silently is not. The tag protections that remain admit no bypass actor and
no broadening.

Release authority is stated wherever it is exercised. The final tag is created by the release
workflow's own token under a permission scoped to the single job that writes it, so the authority
to run the release lane is the authority to create a release tag; that equivalence is announced by
every run and recorded at the job itself, never left to be inferred from a green result. A named
deployment environment that does not exist is not a gate — GitHub materializes it unprotected on
first use — so the workflow names no environment rather than publish a gate that gates nothing.

### SPEC SECTION Error Cases

| Condition | Behavior |
|-----------|----------|
| No git remote configured | `detect_repo` returns `None` |
| Neither config repo nor git remote | `resolve_repo` returns `Err` |
| No `GITHUB_TOKEN` | Read, list, and verification paths return an actionable error without consulting `gh` credentials |
| Issue does not exist (404) | Returns not-found only after repository access is revalidated within the operation deadline |
| Network timeout | `fetch_issue_api` returns `Err` after 10 seconds |
| `gh` CLI not authenticated | `gh_is_available` returns `false` |
| Repository missing or inaccessible | Verification is inconclusive; never reported as issue not_found |
| More than 100 unique issue IDs | Verification fails before provider access |
| Duplicate issue IDs across specs | Provider is queried once per unique issue in the batch |
| Malformed REST response | Strict issue verification records an inconclusive provider error; never successful empty verification |
| Direct issue-detail response contains a `pull_request` marker | Returns a provider error; verification/import does not treat the pull request as an issue |
| Issue-list entry has `pull_request: null` or another non-object marker | Entire listing fails as malformed provider data; null cannot masquerade as an ordinary issue |
| Raw issue or pull-request item is closed, has malformed fields, or has mismatched repository/resource/number URL identity | Entire listing fails before pull-request filtering |
| Duplicate raw item identity within or across list pages, including pull requests | Entire listing fails; filtered pull requests cannot hide duplicates |
| Issue-list page contains 101 or more provider entries | Entire listing fails before parsing any entry, even if overflow entries are pull requests or malformed |
| Issue listing still has a next page after 100 pages | Entire listing fails instead of truncating |
| Archive child changes code/spec/tests or rewrites immutable package evidence | Archive-only validation fails; it never skips the product matrix on an unproven diff |
| Release commit lacks a successful merge-bound archive check | Release validation fails before building artifacts |
| RC marker is lightweight, malformed, moved, or has conflicting workflow history | Qualification and promotion fail; a fresh annotated RC marker is required |
| Platform evidence is missing, unsuccessful, or bound to another tag/SHA | Final tag creation and release upload are refused |
| Either immutable tag ruleset (RC or final) is absent, inactive, incomplete, broadened, or grants any bypass actor | RC qualification fails before any release job runs |
| Ruleset validation reports no unenforced tag protections | Qualification fails rather than imply that App-only final-tag creation, a separate release identity, or a deployment-environment approval was verified |
| Promotion has no token available to create the final tag | The empty-token guard refuses promotion before any tag is written or pushed |
| A release job other than promotion or publication attempts to write a ref | The workflow's read-only default permissions deny it; write is granted per job, never workflow-wide |
| Metadata ancestry contains code, a merge-only side parent, exceeds 32 first parents, or has foreign/malformed/unsuccessful evidence | Reuse stops and the archive/Trust gate fails closed instead of borrowing older checks |

### REQUIREMENT REQ-github-007

Release qualification SHALL bind Ubuntu, macOS, and Windows results and final publication to one
immutable release-candidate commit, while ordinary product pull requests SHALL use Ubuntu as the
authoritative integration platform.

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
- Promotion fails closed unless Ubuntu, macOS, and Windows are green for the unchanged candidate SHA.
- The final `vX.Y.Z` tag is created only after promotion succeeds and points to that same SHA.
- Release uploads independently reject mismatched marker, tag, checkout, evidence, or artifact SHA.
- Release-chain Actions and executables have independent immutable pins, and publication freshly
  revalidates tags, actual checkout, original platform evidence, and package hashes.
