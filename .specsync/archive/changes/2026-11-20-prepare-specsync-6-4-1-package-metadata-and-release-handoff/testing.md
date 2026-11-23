---
change: prepare-specsync-6-4-1-package-metadata-and-release-handoff
artifact: testing
---

# Testing

- `python3 -I .github/scripts/validate-release-version.py` reports 6.4.1; `validate-workflow-runtime-pins.py` and `test-validate-release-candidate.py` pass (the last with Git's global configuration off for that one command on a tag-signing Mac, #1110).
- `fledge lanes run pre-push` and `fledge lanes run verify`; `specsync change audit --strict` with origin/main fetched.
