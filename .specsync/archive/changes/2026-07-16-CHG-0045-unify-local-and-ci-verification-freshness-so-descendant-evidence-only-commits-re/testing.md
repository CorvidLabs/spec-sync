---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: testing
---

# Testing

Focused unit tests SHALL create real Git histories and prove:

- evidence at `HEAD` is current with matching contract and workspace digests;
- one child commit containing only supported `state.json`, `verification.json`, and `verification-attempts.json` persistence is current;
- multiple supported-verification-persistence child commits are current;
- changing source, tests, SpecSync configuration, Trust policy, canonical specs, or the approved change contract makes evidence stale;
- a source change followed by a source revert remains stale;
- archive, hash, approval, task, `change.md`, sequence, lock/transaction, build, and cache paths are each stale despite broader volatility exclusions;
- a syntactically allowed state path with a malicious contract mutation is stale;
- a commit containing both supported verification evidence and a governed input is stale;
- a divergent or nonancestor evidence commit is stale;
- merges are inspected against every parent, with an allowed evidence-only merge passing only when every edge is unambiguous and allowed, and disallowed or ambiguous merges failing closed;
- missing commits and Git/path decoding failures fail closed; and
- toggling `CI`, `GITHUB_ACTIONS`, or workspace environment variables cannot change the result.

These focused and integration regressions are direct evidence for `REQ-change-013` and `REQ-change-016`.

CLI integration coverage SHALL verify that `specsync change status` reports `accept` exactly when strict `specsync change check` accepts the same persisted verification evidence. It SHALL also verify both surfaces return `verify`/failure after a governed-input child commit.

Native verification includes focused freshness tests, the complete unit and integration suite, format, type-check, Clippy with warnings denied, and release build. Lifecycle verification includes strict SpecSync at 100% coverage plus Trust doctor and explicit-range Trust verification. CHG42 through CHG44 are reverified only after CHG45 passes native gates. Hosted checks and closing approvals remain post-push boundaries and are not claimed by this artifact.
