---
change: release-specsync-6-1-0
artifact: context
---

# Context

Published SpecSync is 6.0.1. Main at efe441d has seven commits after that tag: optional module 3md files, lesson briefings, `change pr`, archive readiness, a fail-closed broken policy, finalize committing the archive move, the source split, and a handoff that does not claim a commit. Corvid-agent recommended 6.1.0 because the 3md support is new behavior a 6.0.1 user can use. The source split is not the reason for the minor.

This change only prepares that version. It does not push a tag, promote a candidate, or publish crates.io or Homebrew. Floating `v6` stays on the 6.0.1 commit until a person moves it after promotion. The immutable `@v6.0.0` tag stays on `6.0.0-rc.14`. Pull request #800 stays out of this release.
