---
kind: session
id: 2026-11-20-state-outside-the-work-tree
aliases: [state-outside-the-work-tree]
objective: SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)
intent_scope: SETUP
modules: [util, hash_cache, change, cmd_check, rehash, cmd_resolve, hooks, validator]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
pr: ""
criteria: [SETUP-6, SETUP-7, CHECK-1.f]
requirements: []
cases: []
design: []
files: []
observations: []
bugs: []
blockers: []
waiting_on_human: []
run_consents: []
next: Continue build
---
# SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)

## Goal

SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54)

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree": "a, all yes.. im back" (Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 00:38Z, answering item 1 as t4c put it at about 21:09Z on 2026-11-19 with option A: 'one more change (change 6): move the hash cache, journal and remote cache out of the work tree; close the scan blind spots, the hooks-folder case and the enclosing-repo case; leave the paths you type yourself (--template) as they are, and document the low-rated items. Then this same reviewer re-checks its own findings, rather than a third full review. [...] (fold [change 5's review] into change 6 if you pick A)', and that the class fix moves them 'into the Git directory, where a repo can't commit files, or into your user cache folder when there's no Git'; t4c mapped it to review2's F1, F2, F3, F4, F6 and F7 fixed, F5 kept and documented, F8, F9 and F10 documented; relayed by t4c over hc to t5r); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve keep-specsync-s-own-caches-and-journal-outside-the-work-tree as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T00:50:57Z · start · intent → intent · SpecSync's hash cache, transaction journal and remote spec cache live outside the work tree, so a repository can never ship one; the scans see every production read and write; the hooks folder and an enclosing work tree cannot lead a write out (F1-F4, F6, F7 of the second review, private, with GHSA-7q5v-8jcc-rx54) ^t-afb3b0
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:40Z · advance · intent → design ·  ^t-d4f09c
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:50Z · advance · design → plan ·  ^t-109984
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
- 2026-11-20T00:55:50Z · gate-opened · plan → plan · Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-3dc27e
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T00:56:19Z · gate-closed · plan → plan · Scope approval of keep-specsync-s-own-caches-and-journal-outside-the-work-tree ^t-89692a
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - relayed_by:
    - decided_by: 0xLeif
    - words: a, all yes.. im back
    - where: Claude Code chat message to his planning session (hc t4c) on 2026-11-20 at about 00:38Z, answering item 1 as t4c put it at about 21:09Z on 2026-11-19 with option A: 'one more change (change 6): move the hash cache, journal and remote cache out of the work tree; close the scan blind spots, the hooks-folder case and the enclosing-repo case; leave the paths you type yourself (--template) as they are, and document the low-rated items. Then this same reviewer re-checks its own findings, rather than a third full review. [...] (fold [change 5's review] into change 6 if you pick A)', and that the class fix moves them 'into the Git directory, where a repo can't commit files, or into your user cache folder when there's no Git'; t4c mapped it to review2's F1, F2, F3, F4, F6 and F7 fixed, F5 kept and documented, F8, F9 and F10 documented; relayed by t4c over hc to t5r
    - recorded_by: claude-code-t5r
    - native: specsync change approve keep-specsync-s-own-caches-and-journal-outside-the-work-tree
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T00:56:57Z · advance · plan → build · native:approved ^t-ef6ef1
  - native_change: keep-specsync-s-own-caches-and-journal-outside-the-work-tree
  - gate: native:approved
  - revision: a2a508244b3be174a57f1b681eba5c62cfb76661
  - by: claude-code (hc t5r)
