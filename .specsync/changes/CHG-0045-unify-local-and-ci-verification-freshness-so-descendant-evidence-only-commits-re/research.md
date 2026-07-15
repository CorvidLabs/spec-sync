---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: research
---

# Research

The current behavior has three separate pieces:

- `verification_commit_is_current(root, evidence, allow_ancestor)` compares exact `HEAD` locally but uses `git merge-base --is-ancestor` in CI.
- strict lifecycle checking supplies `is_ci_project(root)` as `allow_ancestor` and independently checks definition and project-input digests.
- `summarize_change` duplicates passed/digest checks and always compares the evidence commit directly to `HEAD`.

`project_input_digest` already excludes broad SpecSync change/archive workspaces and volatile build inputs so writing supported verification evidence does not alter the tested input digest. That exclusion is necessary but far too broad for ancestry: approvals, tasks, definitions, archive evidence, hashes, sequence state, and caches must not become permissible merely because the workspace digest omits them. The only supported persistence from `change verify` is the active change's `state.json`, `verification.json`, and `verification-attempts.json`.

A single net diff from the verification commit to `HEAD` is also insufficient. A governed source change followed by a revert produces an unchanged final tree but proves that an unverified delivery state existed in the intervening history. The implementation must enumerate every intervening commit and inspect each parent edge with NUL-delimited output. Merge commits require all parent edges to satisfy the same exact allowlist or must fail closed when ancestry is ambiguous.

The existing unit test explicitly asserts the local/CI mismatch. It should be replaced by parity tests around the complete predicate. Existing canonical change-ID parsing, digest framing, portable path validation, state/evidence loaders, and Git helper patterns can be reused. The acceptance path's exact commit comparison is intentionally separate and must remain unchanged.
