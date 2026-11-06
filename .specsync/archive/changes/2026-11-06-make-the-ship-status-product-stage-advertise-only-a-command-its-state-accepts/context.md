---
change: make-the-ship-status-product-stage-advertise-only-a-command-its-state-accepts
artifact: context
---

# Context

spec-sync#690: `ship-status` advertised `specsync change check <id> --commit` for an `accepted` change whose verification was no longer current, and `check` refuses `accepted` ("expected approved or implementing or verifying").

On main cd8cab5e the top-level `ship_next` is already safe: #534's narrowing falls back to the lifecycle next step for draft, accepted and archived. The `product_tip` stage in `ship_stages` (`src/commands/change.rs`) still prints `check --commit` whenever the stage is not done, whatever the state. A unit test over every state shows draft, accepted and archived get a command their guard refuses.

Leif chose the fix 2026-11-05 22:35 ("Take it"): name `change reopen` for accepted. Archived has the same guard (reopen admits accepted and archived), and draft needs `change approve`, so the definition covers all three so the test can hold for every state; this is flagged in the approval request.
