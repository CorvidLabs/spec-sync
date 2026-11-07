---
change: repair-a-damaged-relayed-decision-with-one-command-whether-or-not-it-is-committed
artifact: context
---

# Context

Issue #904, from a Codex thread on #900 (comment 4489111584 thread `PRRT_kwDORqhp_860YaI6`). #900 refuses a damaged `relayed_by` on read and every refusal names `git checkout HEAD -- <file>`. `change check --commit` commits `approvals.json` before review, but `change review` writes `review.json` and `review-attempts.json` afterwards, and nothing may be committed between review and ship. In that window `HEAD` holds neither file, so the named command fails. After a block, a fix and a new review, `HEAD` holds an older `review-attempts.json`, and `git checkout HEAD` would throw away the uncommitted attempts written since, one of which can be a block (REQ-change-046).

Leif approved CHANGE-13.e, 13.f and 13.g as drafted, the `specsync change repair-relays` command, and removing only the damaged relay when no copy survives, on 2026-11-06 between 06:10 and 06:19 MST in the Herdr orc tab, relayed by orc t18: "Ok yes I approve all".

Ruled out: refusing when no copy survives (the change stays unreadable, and the only way out deletes review files or hand-edits them); `git checkout` as the active-change recovery (fails in the window, and can erase newer attempts later); recording repairs in a history file (no gate reads a relay, so output is enough).

This branch is stacked on #900 and ships after it. If #900's REQ-change-relayed-decision-001 text or Public API rows change before it lands, these MODIFIED deltas are rebased onto them (CHANGE-11) and re-approved.
