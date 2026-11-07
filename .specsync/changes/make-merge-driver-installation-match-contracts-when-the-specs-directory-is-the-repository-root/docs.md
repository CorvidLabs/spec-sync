---
change: make-merge-driver-installation-match-contracts-when-the-specs-directory-is-the-repository-root
artifact: docs
---

# Docs

Update `specs/cmd_merge/cmd_merge.spec.md` so installation explicitly covers a repository-root specs directory, and add the orc-assigned requirement to `specs/cmd_merge/requirements.md` through change materialization. Increment the spec version and add its change-log entry. Mark the completed module task and append a later `cmd_merge.3md` lesson describing the root-pattern failure and the `git check-attr` proof.

No site or CLI grammar change is needed: this restores the documented behavior of the existing `merge --install` command.
