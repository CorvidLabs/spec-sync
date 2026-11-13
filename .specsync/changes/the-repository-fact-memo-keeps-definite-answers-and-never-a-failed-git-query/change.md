---
id: the-repository-fact-memo-keeps-definite-answers-and-never-a-failed-git-query
state: approved
type: bug_fix
base_commit: 18385ef5ead84f0b51d70a29a6fed8a3e5ca222a
---

# The repository-fact memo keeps definite answers and never a failed git query

## Intent

The repository-fact memo keeps definite answers and never a failed git query

## Affected Canonical Specs

- `git_utils`
- `change`

## Acceptance Criteria

- a git query that fails (for example rev-parse --verify HEAD exiting 128 before any commit) is not memoized and is asked again on the next read; an unset config key and an absent ref read through symbolic-ref -q or rev-parse --verify --quiet are memoized; eight_workflow_v2_changes_finalize_in_their_originating_prs_without_duplicate_product_ci stays within REQ-change-127's ceiling

## No-spec Rationale

REQ-git-utils-009 says a failed git query SHALL NOT be memoized and a failure is retried on the next read; note_repository_git memoized every exit code. An unset config key (exit 1) and an absent ref read with -q or --quiet (exit 1) are definite answers, not failures, which keeps REQ-change-127's spawn ceiling (#887)
