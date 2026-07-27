---
change: CHG-0067-fix-issue-467-by-deduplicating-identical-stage-zero-entries-from-overlapping-gi
artifact: testing
---

# Testing

## Requirement Evidence

- `REQ-change-042`: focused unit coverage exercises exact duplicate deduplication and independent
  fail-closed mode and object conflicts.

## Characterization

- Initialize a real Git repository with a tracked parent directory and
  `GIT_ATTRIBUTE_BATCH_PATHS + 1` exact tracked children.
- Inspect a candidate set containing the parent plus every exact child so Git returns identical
  stage-zero records through separate bounded pathspec batches.
- Assert inspection succeeds and retains exactly one mode and object ID per tracked child.

## Conflict Coverage

- Feed the stage-zero accumulator an identical path and object with a differing mode; assert the
  conflicting observation fails and does not replace the original pair.
- Feed the accumulator an identical path and mode with a differing object ID; assert the
  conflicting observation fails and does not replace the original pair.

## Required Gates

- `fledge run test -- git_candidate_inspection_ -- --nocapture`
- `fledge lanes run verify`
- `fledge spec check --strict`
- `fledge trust verify`
