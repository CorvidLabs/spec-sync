---
change: CHG-0003-finalize-specsync-5-0-release-consistency-and-parallel-validation
artifact: context
---

# Context

The accepted 5.0 lifecycle is behaviorally complete, but the final PR run exposed a parallel-test collision in effective-contract validation. The validator names its scratch directory with only the process ID and a seconds-resolution timestamp, so concurrent validations in one process can share and delete the same files. A semantic audit also found stale v4/v5, agent-command, migration, comparison, and test-evidence wording that structural spec scoring cannot detect.

`specsync report` also flags `cmd_score` and `hooks` as freshness-aged. Their APIs remain fully documented, both score A, strict validation passes, and neither tracked source file changes in this PR. They are recorded as non-blocking pre-existing freshness maintenance rather than expanded into unrelated 5.0 code work.
