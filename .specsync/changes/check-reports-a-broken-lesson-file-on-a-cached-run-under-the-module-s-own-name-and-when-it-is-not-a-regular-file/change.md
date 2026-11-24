---
id: check-reports-a-broken-lesson-file-on-a-cached-run-under-the-module-s-own-name-and-when-it-is-not-a-regular-file
state: approved
type: bug_fix
base_commit: 24d553f6912b7594e8664e629eba5092afd6cbe2
---

# Check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file

## Intent

Check reports a broken lesson file on a cached run, under the module's own name, and when it is not a regular file

## Affected Canonical Specs

- `cmd_check`
- `hash_cache`
- `validator`

## Acceptance Criteria

- After a clean check, a plain check reports a module's lesson file that was added broken or broken since: every .3md entry beside a spec is an input of its cached verdict and of the changed-spec classification (REQ-hash-cache-011). check reads the lesson file the lifecycle reads, <module>.3md named for the spec's module whatever the spec file is called. A <module>.3md that is not a regular file, such as a directory, is reported as one that cannot be read. change check --strict still fails on the warning, as on any other; REQ-cmd-check-023 and the changelog say that instead of saying lifecycle commands never fail. Each of the three fixes has a test that fails without it, and the full native Verify lane passes.

## No-spec Rationale

Not applicable
