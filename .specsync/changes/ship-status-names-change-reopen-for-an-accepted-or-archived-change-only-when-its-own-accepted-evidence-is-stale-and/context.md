---
change: ship-status-names-change-reopen-for-an-accepted-or-archived-change-only-when-its-own-accepted-evidence-is-stale-and
artifact: context
---

# Context

#922, two Codex P2s on #916, deferred by Leif's call. Since #916, an accepted or archived change
whose whole-project verification digest was no longer current got the product-stage action
`specsync change reopen <id> ...`. `reopen_change` compares the change-scoped acceptance inputs:
when only an unrelated project file drifted, it finds them unchanged and refuses. When
`verification.json` is missing or malformed, it fails at `load_verification`. In both cases
ship-status advertised a command that refuses, which REQ-cmd-change-033 forbids.

Now the product evidence of an accepted or archived change is its change-scoped accepted
evidence: `change::accepted_evidence_state`, from the terminal-evidence verdict `reopen` relies
on, plus whether `verification.json` loads. Current evidence completes the product stage.
Stale evidence still names `reopen`. Unreadable evidence, or a corrupt archive, names the restore
from Git history. Verifying and earlier states keep the whole-project digest.

Chough: spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges; this change goes through
a Chough 0.6.0 session before review (Leif's rule, relayed by t370 on 2026-11-13).
