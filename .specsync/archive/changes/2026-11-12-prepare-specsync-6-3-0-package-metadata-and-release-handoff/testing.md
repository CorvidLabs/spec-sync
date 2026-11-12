---
change: prepare-specsync-6-3-0-package-metadata-and-release-handoff
artifact: testing
---

# Testing

| Check | Expected |
|---|---|
| `python3 .github/scripts/validate-release-version.py` | "Validated release version 6.3.0 across current distribution surfaces" |
| `python3 .github/scripts/validate-workflow-runtime-pins.py` | passes |
| `python3 .github/scripts/test-validate-release-candidate.py` | OK |
| `fledge lanes run pre-push` | pass |
| Review of the diff | the `action.yml` default, docs default row, `PUBLISHED_ACTION_DEFAULT` and current-install guidance unchanged; prose about the immutable `@v6.2.0` tag unchanged |

Hosted CI on the archived head runs the full lane. The release workflows run only on Leif's RC tag and dispatch.
