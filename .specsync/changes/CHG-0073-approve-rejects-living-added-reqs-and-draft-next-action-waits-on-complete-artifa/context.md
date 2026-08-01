---
change: CHG-0073-approve-rejects-living-added-reqs-and-draft-next-action-waits-on-complete-artifa
artifact: context
---

# Context

Sandbox dogfood (issues #14 and #16) showed two agent-facing lifecycle gaps on SpecSync 6.0:

1. Draft next_action recommended change approve while selected artifacts still contained incomplete HTML TODO comment stubs (artifacts_complete=false).
2. ADDED of a requirement ID already present in living requirements.md only failed at materialize/check, so agents discovered the error after definition approval.

This change tightens both gates in the change domain and surfaces the completeness guidance from the change command adapter.
