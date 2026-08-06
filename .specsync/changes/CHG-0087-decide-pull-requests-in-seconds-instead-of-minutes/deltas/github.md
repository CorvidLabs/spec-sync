## ADDED

### REQUIREMENT REQ-github-020

Continuous integration SHALL reject a pull request whose active change carries
verification evidence anchored to a commit that is not an ancestor of the head
under test, before running any job that builds or tests the project.

Acceptance Criteria

- The rejecting job requires no Rust toolchain and no build step.
- The check fails only when the recorded commit exists and is provably not an
  ancestor; missing files, unparseable JSON, absent commits and commits the
  repository does not contain are left to the authoritative audit.
- Jobs that build or test the project do not run when the check fails.
- The single required aggregate check still reports a conclusion, so a pull
  request is never left waiting on a check that cannot run.
