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
- Runtime-pin and protected-policy tests cover every changed workflow/action.

## Completion

- Run targeted validator/workflow tests while iterating.
- Run the full repository and release validation once on the reviewed final candidate.
- Exercise one intentional platform failure and one all-green promotion in a non-release fixture.
