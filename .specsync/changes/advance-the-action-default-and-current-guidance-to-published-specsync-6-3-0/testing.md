---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-3-0
artifact: testing
---

# Testing

| Check | Expected |
|---|---|
| `validate-release-version.py` | "Validated release version 6.3.0 across current distribution surfaces", with the published default equal to the Cargo version |
| `test-validate-release-candidate.py` | OK |
| `validate-workflow-runtime-pins.py` | passes |
| `fledge lanes run pre-push` | pass |
| `git show v6.3.0:action.yml` | default `6.2.0` and the #886 403 fallback present, matching the docs |

The PR's CI runs the packaged Action consumer and the full lane. Omitted-input Action consumers on Linux and macOS at the merged wrapper commit come before any floating `v6` move, which is Leif's decision.
