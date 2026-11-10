---
change: re-approval-accepts-an-unchanged-delta-body-that-already-carries-the-moved-canonical-text
artifact: context
---

# Context

Issue #975 (found on chough#163). Change B's MODIFIED body was written on top of change A's
amended text while A was still open. B was approved while main had the pre-A text, so the
approval recorded the old base. When A landed, the block moved. Re-approving B was refused
("the delta body has not been rewritten") although B's body already carried A's text. Only a
byte-only edit recovered it.

Leif chose on 2026-11-10 (relayed by t370): recover the old base text from git and re-approve
only when the delta still applies cleanly to the new base, with no new flags.
`approval_delta_bases` now collects each unchanged, unapplied item whose base moved. For
each, it walks the canonical file's history (newest first, at most 200 commits) for the
version whose block digest equals the recorded base. It then runs a line-level diff3-style
comparison of base, current and approved body. When every region the current text changed is
either unchanged there or already identical in the body, the approval records the current
base. Otherwise the existing refusal stands, as it does outside git or when no commit matches.
