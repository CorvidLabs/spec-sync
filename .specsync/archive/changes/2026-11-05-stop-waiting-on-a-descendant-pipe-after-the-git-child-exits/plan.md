---
change: stop-waiting-on-a-descendant-pipe-after-the-git-child-exits
artifact: plan
---

# Plan

## Approach

Collect a Git child's stdout and stderr on pipes this process owns, and return when that child exits. Readers wake from a self-pipe instead of waiting for end-of-file, so a grandchild that still holds the pipe cannot stall the caller. Status checks that only need an exit code point both streams at the null device, so a grandchild cannot keep the caller's own stdout or stderr open. SPEED-1, SPEED-1.a, and SPEED-1.e.

## Out of scope

Killing `git gc --auto` or other descendants. Adding a new wall-clock budget. Changing the 120 second Git deadline, the output cap, or the stdin-writer join. Cargo verification children. The `gh` CLI.

## Steps

1. Add a failing test that returns only if the read finishes while a grandchild still holds the pipe.
2. Stop the bounded Git reader and every production Git `.output()` / `.status()` on that same rule.
3. Keep the existing deadline, cap, and stdin-writer tests unchanged.
4. Record the contract on the git_utils and change specs.

## Risks

A reader that returns before the child's last bytes are drained would drop real Git output. The drain happens before the stop flag is honored, and the tests require the child's payload bytes.

Nulling stderr on status checks hides Git diagnostics that used to appear on the terminal for quiet commands (`diff --quiet`, `merge-base --is-ancestor`, `cat-file -e`). Those callers decide from the exit code.

## Constraints consulted

none. The existing module lessons do not choose how a pipe is closed. SPEED-1, SPEED-1.a, and SPEED-1.e are the intent this change carries.
