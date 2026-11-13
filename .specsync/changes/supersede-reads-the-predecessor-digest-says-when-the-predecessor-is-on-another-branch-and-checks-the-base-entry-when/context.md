---
change: supersede-reads-the-predecessor-digest-says-when-the-predecessor-is-on-another-branch-and-checks-the-base-entry-when
artifact: context
---

# Context

`change supersede` was unusable beyond the narrowest case (#423): its 64-hex entry digest had to
be fished out of the predecessor's `verification.json`; a predecessor accepted on another branch
was reported only as "not found"; and when a later edit moved the predecessor's entry before the
successor's base, the obligation was recorded and later failed silently at coverage time.

Leif approved building it at 2026-11-13T03:49Z ("#423 easier supersede", feature set 2), and
t370 relayed at 04:04Z that it goes ahead as designed (coordinator chat, Claude Code hc t370).
