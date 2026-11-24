---
change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
artifact: requirements
---

# Requirements

CHANGE-17.d: both `change audit` and `check --strict` report an unexplained loss of a canonical requirement, so projects running either command catch it.

REQ-cmd-check-004 says strict check runs the shared comparison on a Git project that has enabled SDD. This change says where "enabled" is read: in the working tree, or at the trusted comparison base. A branch cannot leave the comparison by disabling or deleting the policy its base has enabled.

A project that never enabled SDD, or whose comparison base has it disabled, keeps drift-only strict check. Plain `check` is unchanged. REQ-change-archived-requirement-loss-001 and REQ-change-169 are unchanged: the comparison itself, its reference, its removal accounting and its bounded reads stay as they are.
