---
change: advance-the-action-default-and-current-guidance-to-published-specsync-6-4-1
artifact: testing
---

# Testing

- `python3 -I .github/scripts/validate-release-version.py`, `validate-workflow-runtime-pins.py`, and `test-validate-release-candidate.py` (with Git's global configuration off for that one command on a tag-signing Mac, #1110).
- `fledge lanes run pre-push`; `specsync change audit --strict` with origin/main fetched.
- A scripted check that every line changed is one #1136 changed for 6.4.0, or one of its two sentences, and that the history lines are untouched.
