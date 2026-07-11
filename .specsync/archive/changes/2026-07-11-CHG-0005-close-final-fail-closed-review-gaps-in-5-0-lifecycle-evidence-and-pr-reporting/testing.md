---
change: CHG-0005-close-final-fail-closed-review-gaps-in-5-0-lifecycle-evidence-and-pr-reporting
artifact: testing
---

# Testing

| Requirement | Evidence |
|---|---|
| REQ-change-013 | Unit regressions for mismatched/traversing persisted IDs, invalid persisted spec scopes, unreadable/malformed tombstone deltas, corrupt approval ledgers, and missing/stale/failed CI verification evidence |
| REQ-cmd-comment-001 | CLI integration regression proving an SDD-only failure appears in markdown and produces a failing status |

Focused regressions run first, followed by CI-style unit/integration suites, strict specs, release build/audit/docs/editor gates, executable lifecycle examples, and the GitHub matrix.

Local evidence: 1,542 unit tests and 194 integration tests pass; format, Clippy, type checking, release build, 221-dependency audit, 62/62 strict specs at 100% file/LOC coverage, 21 docs tests plus lint/build, and VS Code compile/package all pass. The lifecycle, concurrent-change, and five-epic examples pass, including five accepted/archived epics and six product tests. Two independent read-only reviews report no remaining blockers.
