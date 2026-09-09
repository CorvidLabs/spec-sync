---
change: CHG-0082-let-documentation-only-pull-requests-reach-the-required-ci-gate
artifact: testing
---

# testing


## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-006` | `bash .github/scripts/test-classify-ci-paths.sh` passes; `python3 -S .github/scripts/validate-workflow-runtime-pins.py` passes; a docs-only diff piped through `classify-ci-paths.sh` yields `full=true`, `archive_only=false`, `review_only=false`. |
