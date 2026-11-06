## ADDED

### REQUIREMENT REQ-change-134

The persistent read cache SHALL be reused only by the build that wrote it. Its build identity SHALL be the package version plus the build fingerprint of REQ-hash-cache-005. A cache written by another build SHALL NOT be loaded, so its passing workflow-history verdicts are recomputed live. That includes another build of the same version that shares the Git common directory, such as a sibling worktree, and a build from before this requirement.

#### Acceptance Criteria

- A persistent read cache stamped with the pre-#878 identity is not reused. The next read recomputes the verdict and rewrites the cache under this build's identity.
- Removing the fingerprint from the read-cache identity fails that test.
