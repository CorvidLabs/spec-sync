---
change: qualify-the-6-1-2-release-candidate-on-macos-15-after-the-macos-14-brownout-and-ship-834-in-6-1-2
artifact: context
---

# Context

v6.1.2-rc.1 (tag object 520f1cc0, commit 8873c08d, the #822 merge) failed qualification in release run 41340279221. The Ubuntu qualify job passed; the macOS job was cancelled before any step because GitHub failed every macos-14 job during the scheduled brownout of 2026-11-01 14:00 UTC to 2026-11-02 00:00 UTC, and the image retires on 2026-11-02 (actions/runner-images#13518). A tag push runs the workflow from the tagged commit, so rc.1 can never requalify; it is left as is.

The owner chose to fix the runner pin on main and ship #834 (#768 case 1) in 6.1.2, since main already carries it and validate requires the candidate to be an ancestor of origin/main. rc.2 is tagged on the merge of this change.
