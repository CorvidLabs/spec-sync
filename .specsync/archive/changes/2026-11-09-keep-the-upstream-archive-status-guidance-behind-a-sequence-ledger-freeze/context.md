---
change: keep-the-upstream-archive-status-guidance-behind-a-sequence-ledger-freeze
artifact: context
---

# Context

spec-sync#908, a review follow-up from #889. With an upstream archive and a frozen change-sequence ledger, JSON `next_action` names the freeze (it is checked first), but the status summary overrides the handoff unconditionally with `upstream_archive_handoff`, so `handoff.before_clearing` tells the operator to rebase. Rebasing does not clear a project-wide freeze.

Reproduced on main cd8cab5e by `a_sequence_ledger_freeze_outranks_the_upstream_archive_guidance`: the handoff leads with "already archived on `origin/main`" while `next_action` names the sequence ledger.
