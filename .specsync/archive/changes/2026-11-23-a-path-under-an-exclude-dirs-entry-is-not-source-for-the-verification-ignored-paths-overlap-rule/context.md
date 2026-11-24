---
change: a-path-under-an-exclude-dirs-entry-is-not-source-for-the-verification-ignored-paths-overlap-rule
artifact: context
---

# Context

Issue #1083, from CorvidLabs/chough#251 and #279. CorvidLabs/NFTMix keeps its specified code at the repository root (`source_dirs = ["."]`), so there is no code folder to narrow to. Adding `.agents` and `.chough` to `exclude_dirs` restores `--require-coverage 100`, but listing Chough's folders in `verification_ignored_paths` is still refused: "verification_ignored_paths entry `.chough/sessions/` overlaps source_dirs `.`". The Chough tester reproduced it on NFTMix e60625a.

Leif's decision, 2026-11-18, relayed by hc t4c and posted on the issue: "build now. A path under an `exclude_dirs` entry counts as outside `source_dirs` for the `verification_ignored_paths` overlap rule. Failing test first." He approved the definition (ss-lead/gates/1083-definition.md) "Approve as written" on 2026-11-19 at about 14:19Z, relayed by hc t4c through ss-lead hc t58.
