---
change: CHG-0024-stabilize-specsync-5-lifecycle-integrity-and-strict-validation-for-5-0-2
artifact: research
---

# Research

Both active `CHG-0016` branches were created from commit `60bd655`, where `CHG-0015` was the highest visible record. The local allocator therefore returned `0016` on both branches. Git merged the directories cleanly because their slugs differed. The existing archived `CHG-0016` makes the collision group contain three immutable historical records.

A single repository-backed sequence claim containing both the numeric value and full claimed ID gives Git a deterministic coordination point: parallel branches write different content from the same base and must reconcile after updating from the default branch. Strict validation remains the enforcement backstop. Historical collisions cannot be renamed without invalidating accepted definitions, approvals, verification evidence, and canonical changelog references, so an exact immutable baseline is safer than rewriting history.

Process recursion cannot be detected reliably by inspecting only the top-level command because Fledge lanes and scripts can re-enter SpecSync indirectly. A verification-context environment marker inherited by descendants detects both direct and indirect re-entry at the next SpecSync boundary. Direct commands can additionally fail before spawning.

The supported retry state already permits `verifying`; preserving every attempt in an append-only ledger removes destructive overwrite ambiguity. The latest evidence remains in `verification.json` for compatibility.

Registry-backed application needs one canonical resolver shared by spec and companion paths. Coverage should recognize configured static content without asking export extraction to invent symbols. Companion-marker checks should compare exact generated lines outside fenced examples so ordinary prose discussing placeholders remains valid.
