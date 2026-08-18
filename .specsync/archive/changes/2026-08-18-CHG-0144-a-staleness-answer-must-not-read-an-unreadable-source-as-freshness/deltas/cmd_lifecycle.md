## ADDED

### REQUIREMENT REQ-cmd-lifecycle-004

The no-stale guard SHALL fail when a spec cites a source file that no longer exists.

Acceptance Criteria
- A deleted cited file fails the guard regardless of the configured threshold, since it measures a single commit and would otherwise pass on any threshold above one.
- The failure names the file and states that it is still cited by the spec.
