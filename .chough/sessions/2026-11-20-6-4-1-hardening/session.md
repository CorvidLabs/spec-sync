---
kind: session
id: 2026-11-20-6-4-1-hardening
aliases: [6-4-1-hardening]
objective: "6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54 (sec-review2's re-check W1, S1, S2, S3, S5, L2, X1, X2; H1 from #1133's review; C1-C5 from #1135), fold in N2, and remove the five history clauses from change 7's requirement texts, red tests first"
intent_scope: ""
modules: [util, hooks, change, hash_cache]
stage: build
stage_status: active
iteration: 1
callers: []
native_change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
pr: ""
criteria: [SETUP-6, SETUP-7, SETUP-8, CHECK-1.f]
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
# 6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54 (sec-review2's re-check W1, S1, S2, S3, S5, L2, X1, X2; H1 from #1133's review; C1-C5 from #1135), fold in N2, and remove the five history clauses from change 7's requirement texts, red tests first

## Goal

6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54 (sec-review2's re-check W1, S1, S2, S3, S5, L2, X1, X2; H1 from #1133's review; C1-C5 from #1135), fold in N2, and remove the five history clauses from change 7's requirement texts, red tests first

## Where it stands

## Decisions and why

- 0xLeif decided "Scope approval of close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first": "Approve (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 19:51Z, answering the 6.4.1 scope question (security/6.4.1-scope.md: 14 fixes, W1, S1, S2, S3, S5, L2, X1, X2, H1 and C1-C5; 5 keeps, S4, L1, L3, X3 and I1; the N2 nit and the 5 clause removals; choices (1) a journal without its project is never replayed and lifecycle commands stop with its path, (2) a typed path still follows links and only the FIFO wait and size bound are fixed, (3) the 64 MiB bound stays and an oversized cache is refused, (4) one change, one review, one re-check by sec-review2) with 'Approve (Recommended)'. The red tests were committed first (e334bd8a), each failing on 6.4.0. Relayed by t4c over hc to t5r; a relayed claim, not authenticated); recorded by claude-code-t5r; a relayed claim, not authenticated; ran specsync change approve close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first as 0xLeif

## What was verified

## Open threads

## Timeline

- 2026-11-20T19:42:10Z · start · intent → intent · 6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54 (sec-review2's re-check W1, S1, S2, S3, S5, L2, X1, X2; H1 from #1133's review; C1-C5 from #1135), fold in N2, and remove the five history clauses from change 7's requirement texts, red tests first ^t-414f4e
  - revision: 3d57fa13f298a46e52416ab0407cb57d9dc67828
  - by: claude-code (hc t5r)
- 2026-11-20T19:42:10Z · advance · intent → design ·  ^t-881da1
  - revision: 3d57fa13f298a46e52416ab0407cb57d9dc67828
  - by: claude-code (hc t5r)
- 2026-11-20T19:42:10Z · advance · design → plan ·  ^t-43b543
  - revision: 3d57fa13f298a46e52416ab0407cb57d9dc67828
  - by: claude-code (hc t5r)
- 2026-11-20T19:55:24Z · gate-opened · plan → plan · Scope approval of close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first ^t-5242ab
  - native_change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
  - revision: e334bd8abd8f5a2343040d45ca1308cad9e33085
  - holds: plan
  - drives: approve
  - by: claude-code (hc t5r)
- 2026-11-20T19:55:24Z · direct-native-write · plan → plan · close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first's lifecycle records changed outside Chough: state.json changed; state stays draft ^t-647aa8
  - native_change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
  - revision: e334bd8abd8f5a2343040d45ca1308cad9e33085
  - direct_write:
    - change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
    - files:
      - state.json: changed
    - before_state: draft
    - after_state: draft
  - by: claude-code (hc t5r)
- 2026-11-20T19:55:24Z · gate-closed · plan → plan · Scope approval of close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first ^t-645237
  - native_change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
  - revision: e334bd8abd8f5a2343040d45ca1308cad9e33085
  - relayed_by:
    - decided_by: 0xLeif
    - words: Approve (Recommended)
    - where: Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 19:51Z, answering the 6.4.1 scope question (security/6.4.1-scope.md: 14 fixes, W1, S1, S2, S3, S5, L2, X1, X2, H1 and C1-C5; 5 keeps, S4, L1, L3, X3 and I1; the N2 nit and the 5 clause removals; choices (1) a journal without its project is never replayed and lifecycle commands stop with its path, (2) a typed path still follows links and only the FIFO wait and size bound are fixed, (3) the 64 MiB bound stays and an oversized cache is refused, (4) one change, one review, one re-check by sec-review2) with 'Approve (Recommended)'. The red tests were committed first (e334bd8a), each failing on 6.4.0. Relayed by t4c over hc to t5r; a relayed claim, not authenticated
    - recorded_by: claude-code-t5r
    - native: specsync change approve close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
    - actor: 0xLeif
  - by: claude-code (hc t5r)
- 2026-11-20T19:55:40Z · advance · plan → build · native:approved ^t-967505
  - native_change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
  - gate: native:approved
  - revision: e334bd8abd8f5a2343040d45ca1308cad9e33085
  - by: claude-code (hc t5r)
