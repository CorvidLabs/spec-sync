---
change: let-the-release-validator-accept-the-action-archive-gate-in-ci-and-docs-before-its-release
artifact: testing
---

# Testing

- `python3 .github/scripts/validate-release-version.py` passes on the tree.
- `python3 .github/scripts/test-validate-release-candidate.py` passes, including
  `test_archive_gate_main_ref_expires_once_a_newer_release_is_published`,
  `test_main_ref_without_archive_gate_inputs_is_rejected`,
  `test_ci_still_requires_candidate_binary` and `test_ci_candidate_mirror_remains_required`.
