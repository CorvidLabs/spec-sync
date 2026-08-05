## ADDED

### REQUIREMENT REQ-change-050

SpecSync SHALL leave a newly initialised project able to complete its own lifecycle, and
SHALL treat an active-change directory that contains no `state.json` as not an active change
in this working tree rather than as corruption.

Acceptance Criteria

- `init` detects a verification command for Cargo, bun, Swift, fledge, Go, Python and npm
  projects, and when none is detected warns at init time naming `.specsync/sdd.json` and an
  example command.
- A change directory with no `state.json` is skipped by active-change discovery, so
  `change new` succeeds on a branch that does not contain an earlier change.
- Every other read error, including an unreadable or malformed `state.json`, still fails closed.
- Verification exposes a lock-free body so a caller already holding the project lock can
  re-run it without deadlocking on the non-reentrant lock.
