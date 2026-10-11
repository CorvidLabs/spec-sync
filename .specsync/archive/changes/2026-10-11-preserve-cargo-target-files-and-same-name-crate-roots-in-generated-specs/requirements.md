# Requirements

### REQ-manifest-020

Cargo discovery SHALL preserve exact binary target files independently from source scan directories and retain package roots when target and package names agree.

Acceptance Criteria
- server declared at src/foo.rs retains src/foo.rs as its source path, while src remains a scan directory.
- A same-name package retains both its package source directory and exact binary target.
- Workspace target paths remain relative to the overall retained project.
