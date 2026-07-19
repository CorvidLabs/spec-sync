---
change: CHG-0054-trust-accepted-change-evidence-that-is-recorded-in-main-history-by-squash-merged
artifact: plan
---

# Plan

1. Add a recording-anchor fallback to accepted-transition authentication that trusts in-history
   commits recording the change as accepted with byte-identical evidence.
2. Cover the squash-merged refreshed-evidence path and the fail-closed no-matching-record path
   with regression tests.
3. Add canonical requirement REQ-change-037 and extend the canonical Invariants section.
4. Accept the change, then archive the four blocked collision-cluster changes with the fixed
   binary.
5. Run forced strict and the complete Trust lane.
