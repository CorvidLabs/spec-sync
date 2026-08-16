---
change: CHG-0134-a-refused-reopen-must-restore-the-archive-it-un-archived-because-the-un-archive
artifact: testing
---

# Testing

    gate 048     before  pass=7 fail=0 pending=2   after  pass=9 fail=0 pending=0
    drill 008    unfixed FAIL "a refused reopen destroyed the archive again"
                 fixed   PASS
    full board   before  pass=42 fail=13           after  pass=43 fail=12

Drill 008 had to move in the same change and its own header said so: "Asserting
today's behavior; both halves must be updated if fixed." Only the transactional
half is inverted — the anchor-preflight half still reproduces and stays pinned.

The inverted section asserts three things, because "the archive survived" alone
is too weak:

  - the package is intact and no orphan exists
  - the refusal NAMES the restore, so a user whose reopen failed knows
  - the retry reproduces the SAME refusal; "an active change directory already
    exists" was the signature of the first attempt having consumed the archive

The new unit test's tail is the vacuity control: after committing the archive tip
and drifting the source, reopen SUCCEEDS and the archive is gone. Without that,
a fix that simply never un-archived would pass everything above.

Suite: fmt clean, clippy clean, 2276 unit (+1) + 374 integration, 0 failures.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-067 | Gate 048 goes 2 pending to 0 and drill 008's inverted section fails on an origin/main binary. The three assertions separate "archive survived" from "restore was reported" from "retry is idempotent" — the second is what a user actually needs, and the third is what proves nothing was consumed |
