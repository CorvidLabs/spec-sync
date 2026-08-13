## ADDED

### REQUIREMENT REQ-validator-012

Source discovery SHALL skip a symlinked entry rather than failing the traversal.

Acceptance Criteria
- A symlinked entry met during source detection or the coverage walk is recorded and skipped, and discovery continues.
- The entry is never traversed, so content beyond a link that leaves the project root is never read.
- A symlink that is a configured source directory entry remains a hard failure, because silently skipping an explicitly configured tree would drop everything the author asked to measure.
- Symlinks in the spec tree remain a hard failure.
- Recorded paths are deduplicated and normalized to forward slashes.
