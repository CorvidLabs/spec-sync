---
id: CHG-0082-let-documentation-only-pull-requests-reach-the-required-ci-gate
state: archived
type: bug_fix
base_commit: 2f0667477708bfdffcb5b242e8f54df8e0d751a8
---

# Let documentation-only pull requests reach the required CI gate

## Intent

Let documentation-only pull requests reach the required CI gate

## Affected Canonical Specs

- `github`

## Acceptance Criteria

- A pull request touching only docs/** or top-level *.md triggers the CI workflow, so the required Required CI gate context reports instead of waiting forever; classify still decides what work runs; test-classify-ci-paths.sh and validate-workflow-runtime-pins.py pass; a docs-only diff classifies without disturbing archive_only or review_only

## No-spec Rationale

Not applicable
