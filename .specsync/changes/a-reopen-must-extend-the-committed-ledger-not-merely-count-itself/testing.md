---
change: a-reopen-must-extend-the-committed-ledger-not-merely-count-itself
artifact: testing
---

# Testing

`a_reopened_change_closes_again_without_reopening_what_the_bound_refuses` fails on **two**
binaries: the current one, where a reopened change cannot finalize, and a build with #660's
stage-D conjunct simply deleted — which is what a fake fix looks like here, and which reopens the
laundering the test also checks.

Two vacuity controls pass on all three binaries, so the repair is not "admit everything".

| Check | Result |
|---|---|
| drill 049 reopen→close | `pass=12 fail=0 pending=0` (was `11/1/1`) |
| drill 013 batch correct-owner | PASSED (was FAIL) |
| drill 069 anchor re-introduction | `pass=5 fail=0` — unchanged |
| full board | 41/17 → 51/7 |
| suite | 2346 unit + 405 integration, 0 failed |

## Vectors, with and without a forged generation

All six combinations refused. The three that report `authenticated-history` on `7cbe820e` —
v4, v5 and v6 with a forged `ReopenRecord` — report `corrupt-history` here.

## Requirement evidence

| Requirement | Evidence |
|-------------|----------|
| REQ-change-088 | The three laundering vectors are refused with and without a forged generation, measured on a fixture built by the unrepaired binary, where the forged-generation variants currently authenticate. Drills 013 and 049 return to green, bisected against a pre-#660 binary so the regression is attributed rather than assumed. The discriminating test fails on the naive revert as well as on the current binary, so it cannot be satisfied by deleting the conjunct that caused the regression |
