---
change: pass-safe-directory-to-specsync-s-isolated-git-when-the-user-s-own-git-trusts-a-repository-owned-by-another-user
artifact: context
---

# Context

Issue #909. SpecSync's isolated Git (`configured_git_command`) sets `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`, so it drops the user's `safe.directory`. A repository owned by another user, which the user's Git trusts, is refused with "unable to inspect changed paths" or "Git repository detection failed". Measured in containers (Git 2.32 to 2.39): `-c safe.directory` and `GIT_CONFIG_COUNT` are not honoured by upstream 2.35.2 to 2.37; a `GIT_CONFIG_GLOBAL` file holding only that entry works on all of them.

Definition: ss-lead/gates/git-environments-909-1072-1077-definition.md, section 2 and REQ-change-171. Leif's choice 2 in the tester's dialog of 2026-11-20 at about 00:45Z: "Yes, temp file (Recommended)".
