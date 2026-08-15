---
change: CHG-0129-a-ruby-method-below-private-must-never-be-extracted-as-an-export-because-an-ass
artifact: testing
---

# Testing

Sandbox gate 063 is the judge:

    before (origin/main)   pass=? fail=0 pending=2   FAIL
    after  (this change)   pass=6 fail=0 pending=0   PASS

**Drill 039 had to move in the same change.** Its Ruby section pinned #479's
BUGGY behaviour as expected, so the fix turned it red — and it said so itself:

    FAIL: #479 appears FIXED: no method leaks past `private`.
          Invert this section and close the issue.

with `# INVERT THIS SECTION when #479 is fixed` in the source above it. Whoever
wrote that pin knew it would one day be wrong and left instructions. Both of its
assertions are now inverted, and the inversion was proved to discriminate:

    unfixed product binary   039 pass=46 fail=2  (both inverted assertions fire)
    fixed product binary     039 pass=48 fail=0

An inverted pin that cannot fail is worse than no pin, so the unfixed board is
the half that matters.

Controls: the statement-form conditional, which never triggered the desync,
behaves exactly as before; public methods above `private` are still extracted.

Suite: fmt clean, clippy clean, 2253 unit + 367 integration, 0 failures.

Residual, disclosed and out of scope: a `def` whose default parameter contains
an inline `if … end` is still mis-tracked. Identical before and after; a
narrower case than the one filed.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-exports-009 | Gate 063 flips 2 pending to 0. Drill 039's second assertion is the one that matters: documenting the leaked method is now an orphan error, so the tool no longer accepts a private method as contract when a user silences the warning the obvious way |
