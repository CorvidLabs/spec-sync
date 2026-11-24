---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-2
artifact: testing
---

# Testing

- `python3 -I .github/scripts/validate-release-version.py`, `validate-workflow-runtime-pins.py`, and `test-validate-release-candidate.py`.
- `fledge lanes run pre-push`; `specsync change audit --strict` with origin/main fetched.
- A scripted check that every line changed is one #1142 changed for 6.4.1, one of the three "after publication" sentences, the CHANGELOG entry or the release handoff, and that the history lines are untouched.
- The pull request's own Action checks run this wrapper commit with its new default on the hosted runners.
