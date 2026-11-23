---
id: 2026-11-20-6-4-1-close-the-14-low-items-left-after-ghsa-7q5v-8jcc-rx54
title: "6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54"
type: journal
tags: [chough/journal, chough/shipped]
date: 2026-11-20
time: 2026-11-20T22:46:58Z
kind: shipped
links:
  - .chough/archive/sessions/2026-11-20-6-4-1-hardening.md
  - .specsync/archive/changes/2026-11-20-close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first/
record: .chough/archive/sessions/2026-11-20-6-4-1-hardening.md
actor:
  - agent: claude-code
    terminal: t5r
---
# 6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54

6.4.1: close the 14 low items left after GHSA-7q5v-8jcc-rx54 (sec-review2's re-check W1, S1, S2, S3, S5, L2, X1, X2; H1 from #1133's review; C1-C5 from #1135), fold in N2, and remove the five history clauses from change 7's requirement texts, red tests first

Why: 0xLeif decided "Scope approval of close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first": "Approve (Recommended)" (Claude Code question dialog with his planning session (hc t4c) on 2026-11-20 at about 19:51Z, answering the 6.4.1 scope question (security/6.4.1-scope.md: 14 fixes, W1, S1, S2, S3, S5, L2, X1, X2, H1 and C1-C5; 5 keeps, S4, L1, L3, X3 and I1; the N2 nit and the 5 clause removals; choices (1) a journal without its…

Verified: 6.4.1: the 18 red tests (W1 S1 S2 S3x2 S5 L2 X1x3 X2 H1 N2 C1-C5) failed on 6.4.0 and pass; full suites green (Linux 3162 unit, 741 integration), clippy clean, check --strict 100%

Read more: [the story](../../.chough/archive/sessions/2026-11-20-6-4-1-hardening.md) · [the archived change](../../.specsync/archive/changes/2026-11-20-close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first/)
