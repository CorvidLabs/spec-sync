---
change: drop-the-ci-yml-path-filters-so-every-pull-request-starts-ci-and-classify-alone-picks-its-lane
artifact: tasks
---

# Tasks

- [x] Capture CI-6 in `hi/ci.md` in Leif's words, with its source, and regenerate the `INTENT.md` index
- [x] Add `.github/scripts/test-ci-trigger-paths.py` and see it fail on the unmodified workflow
- [x] Remove the `paths:` lists from the `pull_request` and `push` triggers in `.github/workflows/ci.yml`
- [x] Run the new test through a fledge task in the `ci`, `verify` and `repo` lanes and from `validate-action` in `ci.yml`
- [x] Add REQ-github-026 and update the `specs/github/testing.md` row that relied on the path filters
- [x] Run the new test, `test-required-ci-gate.py`, `test-classify-ci-paths.sh`, `change audit --strict`, the pre-push gate and the verify lane
