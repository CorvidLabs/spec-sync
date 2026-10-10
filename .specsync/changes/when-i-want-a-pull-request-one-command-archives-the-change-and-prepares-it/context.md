---
change: when-i-want-a-pull-request-one-command-archives-the-change-and-prepares-it
artifact: context
---

# Context

`change ship` archives and commits, then tells the person to push and open the pull request themselves. That second half is where pull requests land with the change still open, or with the archive commit still local.

`change pr` does the rest: archive when the change is ready, push the branch, and open the pull request. An existing pull request for the branch is reused. The command stops before merge.
