---
change: keep-strict-check-s-lost-requirements-comparison-when-a-branch-turns-sdd-off
artifact: docs
---

# Docs

Say in the cmd_check specification and its testing companion that strict check reads SDD enablement from the working tree or the trusted comparison base, and that a branch disabling or deleting the policy is still compared. Record the lesson in `specs/cmd_check/cmd_check.3md` and the helper in the change specification's Public API table. The published site pages do not describe the opt-out in more detail than "a project that has enabled SDD", so they need no edit.
