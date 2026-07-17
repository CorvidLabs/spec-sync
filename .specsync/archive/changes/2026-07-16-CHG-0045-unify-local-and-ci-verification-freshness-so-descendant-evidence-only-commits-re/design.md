---
change: CHG-0045-unify-local-and-ci-verification-freshness-so-descendant-evidence-only-commits-re
artifact: design
---

# Design

Introduce one shared verification-freshness predicate used by strict lifecycle validation and `summarize_change`. The predicate accepts the project root, change record, and loaded verification record and returns current only when all of these conditions hold:

1. verification passed;
2. the recorded commit is present and is `HEAD` or an ancestor of `HEAD`;
3. the recorded `contract_digest` matches the effective approved definition;
4. the recorded `workspace_digest` matches `project_input_digest(root)`; and
5. every path changed across every parent edge of every intervening commit is one of the three supported verification-persistence files under a canonical active-change ID; and
6. the persisted state and verification evidence remain mutually consistent with the loaded change and approved definition.

Enumerate every commit on the ancestry path from the verification commit, exclusive, through `HEAD`, inclusive. Do not use only `git diff <verified>..HEAD`: a net diff can hide a governed source change that a later commit reverts. For a single-parent commit, inspect its NUL-delimited changed-path set against that parent. For a merge, inspect the changed paths against every parent and fail closed if ancestry or parent-edge interpretation is ambiguous. Reject Git command failure, invalid UTF-8, non-portable paths, a missing commit, and nonancestor history.

The path parser accepts exactly `.specsync/changes/<canonical-change-id>/state.json`, `.specsync/changes/<canonical-change-id>/verification.json`, or `.specsync/changes/<canonical-change-id>/verification-attempts.json`. It validates the change ID with the existing canonical parser and rejects nested suffixes or aliases. It explicitly rejects `.specsync/archive/**`, approvals, tasks, `change.md`, deltas, sequence state, hash caches, lock/transaction files, specs, configuration, policy, source, tests, build products, and dependency caches. A path being absent from `project_input_digest` does not make it safe.

For every allowed path, load the referenced active change through normal validation. Its state ID must equal the canonical directory ID; its current verification and attempt ledger must parse and agree on the latest attempt; passed evidence must bind that record's effective approved definition; and the target change's state must be consistent with persisted verification. This prevents a syntactically allowed `state.json` mutation from laundering a changed contract.

Remove the environment parameter and CI branch from `verification_commit_is_current`. `change check` and `summarize_change` call the same complete predicate so local status, local strict checks, and hosted checks cannot disagree. Acceptance retains its exact-`HEAD` precondition because closing approval is a distinct immediate transition; CHG45 changes verification freshness reporting and validation, not acceptance authorization.

The canonical `REQ-change-013` and `REQ-change-016` text will state the environment-independent ancestry, per-parent history, exact-path, digest, and state/evidence-consistency rules. No release is claimed and CHG42 through CHG44 remain unaccepted until CHG45 is implemented and all four workspaces are reverified on the corrected implementation.
