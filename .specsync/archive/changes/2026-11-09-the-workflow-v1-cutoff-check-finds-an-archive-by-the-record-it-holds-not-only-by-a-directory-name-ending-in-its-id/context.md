---
change: the-workflow-v1-cutoff-check-finds-an-archive-by-the-record-it-holds-not-only-by-a-directory-name-ending-in-its-id
artifact: context
---

# Context

Issue #940, a Codex P2 on #918 that Leif deferred and then scheduled as its own fix. #918 (REQ-change-136) decides whether a workflow-v1 change existed at the trusted pre-v2 cutoff from the cutoff commit's tree. Its candidates are the active state path and the cutoff archive entries ending in `-<id>`. The history walk it replaced went through `workflow_version_state_paths`, which also includes the archived record's own resolved package path from `find_change_dir`, which finds an archive by the id its `state.json` declares. So a workflow-v1 archive moved before the cutoff to a directory such as `renamed-archive` was accepted before #918 and is refused after it. That is a false refusal, never a false pass.

Leif captured SPEED-1.a.2 for it on 2026-11-06 ("Capture both", relayed by orc t18): "Making a read faster never turns an archived change whose history is intact into a refusal." The orc assigned REQ-change-146.
