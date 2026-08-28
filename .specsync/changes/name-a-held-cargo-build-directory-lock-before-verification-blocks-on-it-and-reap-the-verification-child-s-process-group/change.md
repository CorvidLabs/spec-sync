---
id: name-a-held-cargo-build-directory-lock-before-verification-blocks-on-it-and-reap-the-verification-child-s-process-group
state: verifying
type: bug_fix
base_commit: d6f266a4fd683246469eb15a8f632061dd5cfbb4
---

# Name a held Cargo build-directory lock before verification blocks on it, and reap the verification child's process group

## Intent

Name a held Cargo build-directory lock before verification blocks on it, and reap the verification child's process group

## Affected Canonical Specs

- `change`

## Acceptance Criteria

- A verification command that is about to wait on a held Cargo build-directory lock prints one line naming that lock and saying the run is blocked rather than compiling, derived from a non-blocking exclusive acquisition and never from elapsed time
- A Cargo build directory that cannot be derived exactly from the argv and process environment produces no notice at all, and a command that takes no build lock is never probed
- On Unix every verification child leads its own process group, so an interrupted parent can end the whole group instead of orphaning cargo on the lock

## No-spec Rationale

Not applicable
