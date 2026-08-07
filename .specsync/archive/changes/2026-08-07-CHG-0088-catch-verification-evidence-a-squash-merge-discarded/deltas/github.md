## MODIFIED

### REQUIREMENT REQ-github-020

Continuous integration SHALL reject a pull request whose active change carries
verification evidence anchored to a commit that is absent from the repository or
not an ancestor of the head under test, before running any job that builds or
tests the project.

Acceptance Criteria

- The rejecting job requires no Rust toolchain and no build step.
- A recorded commit the repository does not contain fails the check: the job
  fetches full history (`fetch-depth: 0`), so absence means the recorded tip was
  orphaned (squash merge of an unfinalized change, rebase, amend, or force-push).
- A recorded commit present but not an ancestor of the head fails the check.
- Missing evidence files, unparseable JSON, and absent or malformed commit
  fields are left to the authoritative audit rather than failing here.
- Jobs that build or test the project do not run when the check fails.
- The single required aggregate check still reports a conclusion, so a pull
  request is never left waiting on a check that cannot run.
