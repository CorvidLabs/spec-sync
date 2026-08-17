---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: research
---

# Research

## The pin drill a number-grep does not find

`grep -l "#533" drills/*.sh` returns 036, 038, 051, 054, 061 — and **not** 037, which is the
pin. Drill 037 describes this bug in full and cites #433 and #523 instead of #533.

That is the second time in this release a systematic-looking check missed a pin: the same
grep-by-issue-number approach missed drill 034 for #529, and the whole-board run caught it
only after the product PR had merged.

The reliable check is not "which drills mention this issue" but "which drills change state
when this behaviour changes", and only a board run answers that.

## Errors hidden behind errors

Inverting 037 surfaced three of its own fixture defects in sequence, each visible only once
the one in front of it was removed:

    missing acceptance criterion  ->  incomplete interview  ->  incomplete artifacts

All three were always true. `audit --strict` inspects every change in the workspace, not only
the ones the drill approves, and the high-water error fired first and masked them for months.

This is why the inverted `late_gate` classifies rather than counts: a refusal carrying the
high-water diagnostic is a #533 regression; any other refusal is the drill's own fixture. The
same failing exit code, two opposite conclusions.

## A self-inflicted instance of the bug under repair

While building the CLI discrimination I wrote a loop referencing fixture directories that did
not exist. The `cd` failed, the loop body ran in the real repository, and `specsync change new`
plus `git add -A` staged a bogus workspace and a ledger rewrite from 115 down to 1 — precisely
the regression this change prevents.

Nothing was committed and it was reverted cleanly. Recorded because the cause is worth keeping:
the rule "never run lifecycle verbs outside a throwaway fixture" was enforced on four
subagents by instruction and on myself by habit. Instructions do not survive a failed `cd`.
The harness now carries a `case "$PWD"` guard that refuses to run anywhere but a fixture path.
