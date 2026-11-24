---
change: check-verification-and-ship-refuse-a-branch-that-lost-canonical-requirements-no-approved-removed-delta-names
artifact: context
---

# Context

Issue #1081, moved from CorvidLabs/chough#299. In CorvidLabs/starling, PR #43 added REQ-StarlingCore-33 and merged with a `specs/StarlingCore/requirements.md` holding only that block: REQ-StarlingCore-1 to 28 were gone, no `## REMOVED` delta named them, and `specsync check --strict`, Chough's change check, the native verification and ship, and CI's verify lane all passed. A setup script had truncated the file before appending. It was repaired by hand in starling#46.

A change's approval binds only the blocks its delta names, and the strict check validates exports against code, not canonical requirements against their history, so nothing noticed the loss.

Leif's decision, 2026-11-18 (on the issue): build now, P1. The definition here is the one he approved on 2026-11-19 ("Approve as written", relayed by hc t4c through ss-lead t58), frozen in ss-lead/gates/1081-definition.md.

Constraints for whoever picks this up: compare against the merge-base of HEAD and the default branch, never the working tree's own history; fail closed when a default branch exists and Git cannot read it; account for every change on the branch, not only the one being checked, because refresh branches carry several.
