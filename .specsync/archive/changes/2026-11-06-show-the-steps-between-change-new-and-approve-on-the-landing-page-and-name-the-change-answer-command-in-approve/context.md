---
change: show-the-steps-between-change-new-and-approve-on-the-landing-page-and-name-the-change-answer-command-in-approve
artifact: context
---

# Context

Audit finding D11 (#883, sealed audit of 2026-11-05 at 40dcf33a and 6.1.2 d08d2746). The landing page `site/src/content/docs/index.md` (Quick Start, lines 48-54) goes from `specsync change new "Add auth" ... --json` straight to `specsync change approve add-auth --actor "Ada"`. That approve refuses with `error: at least one acceptance criterion is required`. The guard is right; the snippet leaves out the interview and the selected artifacts. Reproduced locally with 6.2.0: after the criterion is answered, the next refusal is `error: the deterministic interview is incomplete`. Neither refusal names a command; only `change status` does.

Leif approved the intent on 2026-11-05 at 20:22 MST (orc tab, relayed by orc t18, "Approve all"). DOCS-3 and DOCS-3.a are this change. DOCS-3.a is a small product change to those two refusals. The landing page is also classified as a full-lane CI path, so editing it reruns its test.

This branch sits on #881's archive tip (603a1356, PR #898), because the shared harness is #881's `tests/integration/documented_workflows.rs`. origin/main is merged in once #898 lands.

REQ-change-133 comes from the orc's REQ-change registry (127 #861, 128 #890, 129 #889, 131-132 S6). Neither main nor any open PR branch had an id of 130 or above.

The artifact refusal (`artifact is incomplete: <path>`) and the delta refusal (`semantic delta modules must exactly match affected specs (missing: auth ...)`) already name what to edit. They are file edits, not commands, and are unchanged.
