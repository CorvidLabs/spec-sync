---
change: release-specsync-6-1-0
artifact: testing
---

# Testing

REQ-github-002 is checked by `python3 -S .github/scripts/validate-release-version.py`, which requires the Action default, the consumer pins, the README and site pins, the SECURITY major ref, and the changelog link to match Cargo.toml. Also run `python3 -S .github/scripts/validate-workflow-runtime-pins.py` and `python3 .github/scripts/test-validate-release-candidate.py`.
