---
change: close-the-low-items-left-after-the-hash-cache-advisory-red-tests-first
artifact: context
---

# Context

GHSA-7q5v-8jcc-rx54 was fixed in 6.4.0 (#1133, `3d57fa13`) and published on 2026-11-20. What it
left is all low severity:
- sec-review2's re-check of changes 7 and 8 left 13 items: W1, S1 to S5, L1 to L3, X1 to X3, I1.
- corvid-agent's review of #1133 added one (H1).
- #1135 holds the five round-2 Codex P2s on #1133 (C1 to C5).

Leif approved the 6.4.1 scope ("Approve (Recommended)", about 19:51Z on 2026-11-20, relayed by
t4c), with `security/6.4.1-scope.md` as the draft:
- **Fixes:** W1, S1, S2, S3, S5, L2, X1, X2, H1 and C1 to C5, plus the N2 nit.
- **Removals:** the five history clauses SpecSync forced into change 7's requirement texts.
- **Kept as documented:** S4, L1, L3, X3 and I1.
- **His choices:**
  1. A journal that does not record its project, including every 6.4.0 journal, is never
     replayed; lifecycle commands stop with its path.
  2. A typed path still follows links; only the FIFO wait and the size bound are fixed.
  3. The 64 MiB cache bound stays, and a bigger cache is refused.
  4. One change, one review, one re-check by sec-review2.

The red tests were written first (`e334bd8a`), and each failed on 6.4.0.
