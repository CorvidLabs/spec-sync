---
change: change-review-refuses-to-pass-a-change-finalize-would-refuse-on-ownership
artifact: context
---

# Context

Issue #1002. The Archive job, and the packaged Action's archive gate, run
`.github/scripts/archive-readiness.sh` with no SpecSync binary. It names `specsync change
ship <id>` for a change whose passing review is at HEAD. It cannot prove canonical
ownership, so it could name ship for a change ship refuses on unowned production source
(#999 fixed only the Rust side).

Leif chose option (b) on 2026-11-10 (relayed by coordinator t370): `change review` refuses
a passing verdict while `ownership_preflight_blocker` reports a blocker. A passing review at
HEAD then implies ship would not refuse on ownership, and the script's existing proof
(passing review at HEAD) is enough. A blocking verdict is still recorded.
`archive_candidate_can_ship` keeps its ownership check for reviews recorded before this.
The script is unchanged.

This also helps #1000: an over-wide scope is caught at review, before a human records a
pass, rather than only at ship.
