---
change: record-relayed-human-decisions-on-approvals-and-reviews-as-claims-not-authentication
artifact: context
---

# Context

Issue #633, audit finding D7 (`_audits/specsync-chough-20261105/REPORT.md`, `scripts/repro.py approval-identity`). An approval records whatever `--actor` it is given, and editing `actor` afterwards is accepted, because the approval digest binds content and not identity. HI CHANGE-6.b promised more than that ("A coding agent can never grant that approval to itself"). README and REQ-change-046 already called reviewer and actor labels claims.

Leif chose to narrow the promise and to record relayed decisions (95-plan row S8). On 2026-11-05 at 20:19 MST, relayed by orc t18, he approved the intent ("Approve") and said "Keep it out" on whether the relay record belongs in the review digest. The `relayed_by` shape is shared with Chough (ch-d7, chough#123); Leif approved it from ch-d7's proposal section 1.

Ruled out: any signing or authentication inside SpecSync (that is attest's job and #633's residual), and putting the relay in the review digest (6.2.x would then report every relayed archive as corrupt).
