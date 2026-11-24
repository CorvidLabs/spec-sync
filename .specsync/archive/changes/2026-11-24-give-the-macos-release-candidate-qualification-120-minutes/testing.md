---
change: give-the-macos-release-candidate-qualification-120-minutes
artifact: testing
---

# Testing

Run `python3 .github/scripts/validate-release-version.py`, `python3 .github/scripts/validate-workflow-runtime-pins.py`, `python3 .github/scripts/test-validate-release-candidate.py` and the required-gate tests on the edited tree. Parse `.github/workflows/release.yml` as YAML and read `jobs.qualify.timeout-minutes` back as 120, with every other job's limit unchanged.

The proof that 120 is enough is the `v6.4.2-rc.2` qualification itself, on Ubuntu and macOS.
