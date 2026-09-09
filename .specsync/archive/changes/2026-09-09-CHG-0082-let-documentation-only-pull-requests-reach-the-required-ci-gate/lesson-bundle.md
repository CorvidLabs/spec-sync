# Lesson bundle — CHG-0082-let-documentation-only-pull-requests-reach-the-required-ci-gate

Material for folding this change's lessons into the affected specs' `context.md`.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: Let documentation-only pull requests reach the required CI gate
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, .specsync
- **Acceptance**: A pull request touching only docs/** or top-level *.md triggers the CI workflow, so the required Required CI gate context reports instead of waiting forever; classify still decides what work runs; test-classify-ci-paths.sh and validate-workflow-runtime-pins.py pass; a docs-only diff classifies without disturbing archive_only or review_only

## Evidence

- Verification commit: `8aa2fee450f471ead2396094711e6ec73ca4ec6c`
- Base commit: `2f0667477708bfdffcb5b242e8f54df8e0d751a8`
- Verified by: `specsync check --spec github`

## From the change's context.md

# context

`Required CI gate` became a required status check, which was correct — without it
the gates were advisory and three pull requests merged red. But it means every
mergeable path must be able to reach the gate, and `docs/**` could not: the CI
workflow's path filter excluded it, so a documentation-only pull request never
triggered CI and the gate sat "Expected — waiting for status to be reported"
forever. PR #504 was the first to hit it.

Tried and rejected: teaching `classify` a documentation bucket so docs pull
requests skip the matrix. Cheaper, but path classification is load-bearing for
archive and review detection, and a wrong bucket there is worse than a slow docs
build. Deferred with its own fixture.

## From the change's testing.md

# testing


## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-github-006` | `bash .github/scripts/test-classify-ci-paths.sh` passes; `python3 -S .github/scripts/validate-workflow-runtime-pins.py` passes; a docs-only diff piped through `classify-ci-paths.sh` yields `full=true`, `archive_only=false`, `review_only=false`. |

## Where these lessons go

- `specs/github/context.md`
