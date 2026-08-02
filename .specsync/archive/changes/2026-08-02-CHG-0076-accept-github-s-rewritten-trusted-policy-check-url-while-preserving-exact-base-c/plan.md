---
change: CHG-0076-accept-github-s-rewritten-trusted-policy-check-url-while-preserving-exact-base-c
artifact: plan
---

# Plan

1. Add fixtures reproducing GitHub's rewritten check URL and a PR tip that advanced after the parent
   policy run.
2. Refactor trusted-run discovery to authenticate bounded Actions API results instead of parsing the
   check details URL.
3. Add negative fixtures for every provenance mismatch and ambiguity.
4. Run the focused verifier tests, lifecycle workflow checks, strict spec check, and the PR's
   lightweight archive gate.
