## ADDED

### REQUIREMENT REQ-hash-cache-005

A validation snapshot SHALL be replayed only by the build that recorded it. The snapshot input digest SHALL frame a fingerprint of every source and manifest the binary was compiled from: the files under `src/`, plus `Cargo.toml`, `Cargo.lock` and `build.rs`. A snapshot recorded by any other build SHALL be re-validated by a default `specsync check` instead of replayed, including one that reports the same package version and one written before this requirement.

#### Acceptance Criteria

- A clean snapshot written under the pre-#878 digest, for a spec whose `requirements.md` repeats an ID, is not replayable, and this build reports the duplicate.
- A clean snapshot under another build fingerprint at the same package version is not replayable, while the same snapshot under this build's fingerprint is.
- Removing the fingerprint from the digest fails these tests.
