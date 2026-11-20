---
change: prepare-specsync-6-4-0-package-metadata-and-release-handoff
artifact: context
---

# Context

Leif set what 6.4.0 holds and the order of its steps (Claude Code chat with his planning session hc t4c, 2026-11-19 about 15:49Z and 15:53Z, relayed through ss-lead hc t58). ss-lead assigned this preparation on 2026-11-20. Main is held until the `v6.4.0` tag. Leif approves this definition, reviews the PR and says yes to its merge. No tag, release, publish or workflow dispatch happens here.

- **6.4.0, not 6.3.1:** since `v6.3.0`, main adds a flag (`change approve --dry-run`, #1100) and changes an exit code (`change ship-status` exits 1 on a blocker, #1093). #1086 and #1088 change what `change check` writes: a history fragment, where it used to add a Change Log row and bump the version.
- **Merged since `v6.3.0`:** #1069 (the Action default to published 6.3.0), #1073 and #1078 (this repository's own Chough records), #1086 and #1088 (history fragments), #1093 (#654, #1038, #937) and #1100 (#1091).
- **Precedent:** #1068 prepared 6.3.0 and #1069 moved the published defaults after it. This change follows #1068.
