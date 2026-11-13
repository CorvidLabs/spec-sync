---
change: a-malformed-relay-in-a-correction-gets-the-repair-relays-recovery
artifact: context
---

# Context

Issue #973, a Codex P2 on #900 deferred by Leif's call. `load_correction_ledger` wraps a
deserialization failure as a bare `invalid correction ledger <path>: <error>` and never calls
`with_relayed_read_recovery`, so a malformed relay embedded in a correction gets no recovery,
unlike the same relay in `approvals.json` or the review files. REQ-change-relayed-decision-001
names the recovery: `repair-relays` for an active change, and `git checkout HEAD -- <path>` for an
archived one. The decider check #1014 added for embedded approvals
(`validate_correction_approval_relays`) refuses a relay naming someone other than its approval's
actor, also without a recovery; this change routes that refusal through the same recovery. #970,
which asked for that check, is closed as fixed by #1014. This builds on #998, which lets `repair-relays` reach the approvals a correction
embeds; before it, the active recovery would have named a command that cannot repair this file.
