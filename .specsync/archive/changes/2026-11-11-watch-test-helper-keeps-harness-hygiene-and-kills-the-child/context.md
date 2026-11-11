---
change: watch-test-helper-keeps-harness-hygiene-and-kills-the-child
artifact: context
---

# Context

corvid-agent on #1053: watch_until dropped the harness env hygiene (SPECSYNC_*, GITHUB_*) and leaked the child on panic paths. helpers.rs gains specsync_std() as the single hygiene source (specsync() wraps it); watch_until uses it and a KillOnDrop guard.
