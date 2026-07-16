---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: requirements
---

# Requirements

## REQ-CHG-0045-001 — Environment-independent freshness

SpecSync SHALL use one verification-freshness decision in local checks, hosted checks, and lifecycle summaries.

Acceptance criteria:

- The result is independent of `CI`, `GITHUB_ACTIONS`, and `GITHUB_WORKSPACE` environment variables.
- A verification commit equal to `HEAD` remains current when its contract and workspace digests match.
- A verification commit that is an ancestor of `HEAD` remains current only when every parent edge of every intervening commit changes exclusively supported verification-persistence paths.
- `summarize_change`, strict `change check`, and the default check path use the same predicate.

## REQ-CHG-0045-002 — Mandatory digest and history boundaries

Ancestor status alone SHALL NOT make verification evidence current.

Acceptance criteria:

- `contract_digest` must match the effective approved definition.
- `workspace_digest` must match the exact current project-input digest.
- A source, test, configuration, policy, canonical-spec, or other governed-input change invalidates evidence even if a later commit reverts it.
- A missing, malformed, divergent, or nonancestor verification commit fails closed.
- Git history/diff failure, non-UTF-8 path output, non-portable intervening paths, and ambiguous merge ancestry fail closed.

## REQ-CHG-0045-003 — Bounded evidence-only descendants

Persisting supported lifecycle evidence SHALL NOT invalidate the verification that produced it.

Acceptance criteria:

- One supported-verification-persistence child commit remains current locally and in CI.
- Multiple supported-verification-persistence child commits remain current locally and in CI.
- Allowed paths are exactly `.specsync/changes/<canonical-change-id>/state.json`, `verification.json`, and `verification-attempts.json`, with a canonical portable change ID and no nested suffix.
- Archive paths, approvals, tasks, `change.md`, sequence state, hashes, lock/transaction files, specs, configuration, policy, source, tests, build products, and caches are rejected.
- Every referenced state/evidence set parses and remains internally consistent with its active change and effective approved definition; a malicious state contract mutation fails closed.
- Merge commits are checked against every parent, and any disallowed parent-edge change makes the evidence stale.
- The exact-`HEAD` requirement for acceptance and closing approval remains unchanged.
