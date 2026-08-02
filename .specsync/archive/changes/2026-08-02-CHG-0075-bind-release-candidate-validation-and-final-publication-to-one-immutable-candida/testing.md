---
change: CHG-0075-bind-release-candidate-validation-and-final-publication-to-one-immutable-candida
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-007` | Workflow fixtures and validator tests prove exact-SHA binding, all-platform success, candidate-change invalidation, and pre-tag promotion ordering. |

## Characterization

- Show that ordinary PR classification currently schedules macOS and Windows.
- Show that the current release workflow starts from an already-created final `v*` tag.

## Focused regressions

- Ordinary product paths select Ubuntu and do not schedule macOS/Windows.
- `vX.Y.Z-rc.N` resolves to one full commit SHA and rejects malformed, lightweight, moved, missing,
  or version-mismatched markers.
- Ubuntu, macOS, and Windows invoke the same named Fledge RC lane at that exact SHA.
- Missing, failed, cancelled, stale, duplicate, or wrong-SHA platform results block promotion.
- A changed candidate requires a new RC marker and cannot reuse earlier green evidence.
- Final-tag creation happens only after the exact-SHA gate passes.
- Upload refuses a final tag, checkout, artifact manifest, or RC marker bound to any other SHA.
- Release provenance accepts the actual post-merge archive-binding `pull_request` event and rejects
  every unrelated event, path, or SHA.
- Runtime-pin and protected-policy tests cover every changed workflow/action.
- Ruleset fixtures require RC and final update/deletion protection without bypasses, plus a distinct
  final-creation policy with only the configured release GitHub App integration; broader
  human/admin actors, wrong App ids, overlapping powers, and non-audited bypass modes fail.
- Workflow source assertions reject stable-tag push triggers, mutable Action refs, missing
  concurrency/rerun controls, promotion outside the protected `release` environment or without the
  repository-scoped release App token, and publication without fresh tag/HEAD/original-evidence
  validation.
- Live private-sandbox dogfood created `v9999.0.0-rc.20260801` once, then confirmed that a human
  final-tag push, RC force-move, and RC deletion are each rejected. The repaired sandbox now has
  distinct final-creation, final-immutability, and RC-immutability policies plus a `release`
  environment restricted to `main` with administrator bypass disabled; their exact GitHub API
  responses pass the production
  validator. A dedicated App-authored promotion remains required before activation.
- Regression assertions require newest-check-first authorization, isolated check-writing
  validation, explicit-404-only release absence, overwrite refusal, and fail-closed API errors.

## Completion

- Run targeted validator/workflow tests while iterating.
- Run the full repository and release validation once on the reviewed final candidate.
- After the dedicated release App is installed, exercise one intentional platform failure and one
  all-green App-authored promotion in the private sandbox before the first production release.
