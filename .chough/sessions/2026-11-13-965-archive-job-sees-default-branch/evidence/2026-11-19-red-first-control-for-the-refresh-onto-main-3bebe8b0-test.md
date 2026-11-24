---
kind: evidence
id: 2026-11-19-red-first-control-for-the-refresh-onto-main-3bebe8b0-test
aliases: []
claim: "Red-first control for the refresh onto main 3bebe8b0: test_every_archive_gate_job_checks_out_full_history fails against main's ci.yml and trust.yml (the Archive jobs check out one commit), the defect #965 reports; test_a_single_commit_checkout_before_the_archive_gate_fails passes there, as it tests the checker on a synthetic job."
result: failed
evidence_kind: product-test
revision: e816e662ca6499b57e5cacb0298e5f323db97872
content: eefd1f01777b5d272f6fcdc0d07cf4155c59637b94cd372fb71af18daa9d9c72
environment: "red control: main 3bebe8b0 plus only the new test-required-ci-gate.py"
criteria: [CI-2]
requirements: []
contracts: []
limitations: []
observed: 2026-11-19T07:33:31Z
actor:
  - agent: claude-code
    terminal: t4t
---
**Red-first control for the refresh onto main 3bebe8b0: test_every_archive_gate_job_checks_out_full_history fails against main's ci.yml and trust.yml (the Archive jobs check out one commit), the defect #965 reports; test_a_single_commit_checkout_before_the_archive_gate_fails passes there, as it tests the checker on a synthetic job.**

Recorded as failed in red control: main 3bebe8b0 plus only the new test-required-ci-gate.py at revision e816e662ca64, observed 2026-11-19T07:33:31Z.

## Key failing lines

- FAIL: test_every_archive_gate_job_checks_out_full_history; Ran 38 tests, FAILED (failures=1)
