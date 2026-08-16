---
id: CHG-0132-a-warm-hash-cache-must-not-drop-findings-because-skipping-re-validation-without
state: archived
type: bug_fix
base_commit: a38638fa386eae8db3dc039fc61648ca66b09397
---

# A warm hash cache must not drop findings, because skipping re-validation without replaying the previous result reports a passing spec that was never checked

## Intent

A warm hash cache must not drop findings, because skipping re-validation without replaying the previous result reports a passing spec that was never checked

## Affected Canonical Specs

- `hash_cache`
- `cmd_check`
- `commands`

## Acceptance Criteria

- Running check twice over an unchanged tree produces the same findings both times, in text and in JSON. A spec skipped as unchanged replays the findings recorded when it was last validated, so specs_checked counts it and its warnings are named. A genuinely clean spec still reports clean. Editing a spec or its source re-validates it and updates the stored result. --force and --no-cache behave as before, and the cache continues to skip re-validation rather than only re-extraction.

## No-spec Rationale

Not applicable
