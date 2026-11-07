---
hi: 1
families: [RECOVER]
---

# Recover

## Intent

<!-- What is this for, and what should it feel like? Write it as a person. -->

## Criteria

- **RECOVER-1**  I can reopen an inconsistent finalization already anchored in Git without losing its evidence, and closing it again requires fresh verification and scoped review.
- **RECOVER-2**  I can reconcile historically drifted committed receipts without losing their original evidence or pretending their old file transitions were proven, and closure requires fresh verification and scoped review.
- **RECOVER-3**  One command tells me everything wrong with the changes in my repository, and how to fix each one.
  - **RECOVER-3.a**  It finds changes merged but never archived, accepted changes whose files changed since, finalizes that stopped halfway, verification that went stale, and records that another version of SpecSync wrote.
  - **RECOVER-3.b**  I can see every fix it would make before it changes anything.
  - **RECOVER-3.c**  It makes only the fixes that need no decision on its own; a fix that needs a decision asks who is deciding and why.
  - **RECOVER-3.d**  When it cannot check something, it says so and still reports everything else.
