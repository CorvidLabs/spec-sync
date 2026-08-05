---
change: CHG-0083-let-finalize-work-in-a-repository-that-has-archived-a-change
artifact: testing
---

# Testing

`a_directory_candidate_admits_the_files_it_expands_to` builds a repository with
an archived change directory, supplies that directory as a candidate, and
asserts evidence collection succeeds. It fails on the unfixed code with the
exact production message.

Only the index guard was reachable from that test. Fixing it surfaced the
visibility guard, which is how the remaining two were found. A single-symptom
fix would have left three.

Drill 036 covers the lifecycle end to end but starts from an empty repository,
so it cannot reach this. It needs a pre-populated archive — tracked above.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| `REQ-change-051` | `a_directory_candidate_admits_the_files_it_expands_to` supplies an archived-change directory as a candidate and asserts evidence collection succeeds; it fails on the unfixed code with the exact production message `Git returned an out-of-scope index path`. Only the index guard is reachable from that test, so the modified, visibility and fsmonitor guards were converted to the same `candidate_scope_admits` helper and verified by inspection to apply identical semantics. Separator-boundary comparison keeps `a/bc` from admitting `a/b`. Confirmed against this repository, which holds 82 archived changes: `finalize` no longer reports an out-of-scope path. |

## Follow-up

Follow-up, tracked outside this change: drill 036 lives in the spec-sync-sandbox
repository and needs a second scenario running the lifecycle in a repository that
already has archived changes. Every fixture in both suites builds from an empty
temp repository, so no existing test can reach this defect class.
