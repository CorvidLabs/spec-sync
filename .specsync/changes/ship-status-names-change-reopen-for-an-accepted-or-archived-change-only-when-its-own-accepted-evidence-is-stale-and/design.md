---
change: ship-status-names-change-reopen-for-an-accepted-or-archived-change-only-when-its-own-accepted-evidence-is-stale-and
artifact: design
---

# Design

- `change::AcceptedEvidenceState` is `Current`, `Stale` or `Unreadable`, and
  `change::accepted_evidence_state(root, record)` (`pub(crate)`) returns it. `Unreadable` means
  `load_verification` fails or the verdict is `corrupt-history`. `Current` means `exact`,
  `successor-covered`, `authenticated-history` or `historical-transition-unproven`. `Stale`
  means `stale`.
- `ship_status_report` computes it once for an accepted or archived record and uses it for
  `verification_current` there. `ship_stages` and `product_stage_action` take it, so an
  `Unreadable` change gets the restore step and a `Stale` one keeps the reopen step.
