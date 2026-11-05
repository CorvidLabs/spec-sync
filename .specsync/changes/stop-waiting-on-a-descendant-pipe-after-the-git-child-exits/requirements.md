---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: requirements
---

# Requirements

SPEED-1. No command should keep an agent waiting minutes.

SPEED-1.a. Status of an archived change, `change list` and `change status` return in under 2 seconds on Raven. A descendant holding a Git pipe must not be what stops `change status`.

SPEED-1.e. SpecSync must never wait on a descendant's open pipe after its child exits.

REQ-git-utils-008 and REQ-change-125 state that contract for the shared collector and the bounded Git runner. The 120 second deadline, the output cap, and the stdin-writer join stay as they are.
