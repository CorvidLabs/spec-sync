---
change: CHG-0140-a-stale-sequence-ledger-must-not-be-committed-backwards
artifact: context
---

# Context

`change check --commit` committed `.specsync/change-sequence.json` **backwards**, lowering the
high-water mark that guarantees change IDs are unique, and reported success while doing it.

Reproduced through the real CLI, unfixed binary:

    HEAD ledger = 3, working tree = 1
    specsync change check <id> --commit
      rc=0
      HEAD ledger AFTER = 1

Exit zero. "Verified, committed, and consistent with the committed tree." Mark regressed 3 to 1.

That matches the live evidence in the issue exactly — commit `6576162d` on
`0xleif/finalize-chg0101`, whose parent was `origin/main` carrying sequence 103, recording:

    -  "sequence": 103,
    +  "sequence": 101,

## Mechanism

1. `change new` mints CHG-N and writes the ledger into the **working tree only**. Nothing
   commits it; the workspace stays untracked.
2. Shared `main` advances past N and the branch is brought up to date, so HEAD carries the
   higher mark while the author's uncommitted copy still claims N.
3. `git_commit_all` runs `git add -A` with no high-water validation, so the lifecycle's own
   `chore(lifecycle): materialize` commit records the regression.

The allocation-time floor (#523, `maximum_observed_sequence`) cannot help. The value was
correct when written. It went stale afterwards, and nothing floors the *write*.

## Why the damage is invisible until much later

Drill 037 measured the blast radius before this change. Over a regressed ledger:

    change check (re-run)   green — the command that committed it cannot see it
    change status/list/show green
    change ship-status      green
    change review           green
    specsync check --strict green
    change audit --strict   REFUSES, with a message naming neither the command nor the file
    change finalize         REFUSES
    change ship             REFUSES
    change new              REFUSES

So the author commits successfully, re-checks successfully, and is stopped much later by a
diagnostic that does not say what caused it or where.

## Sibling sites

`git_commit_all` has three callers — materialize, verification evidence, and archive — and
each stages `-A`. The report named the materialize path. Fixing only that would leave two
commit paths able to commit the same regression, which is the pattern this codebase has hit
eight times. The floor therefore lives in `git_commit_all` itself.

## Ruled out

Refusing the commit. The author did nothing wrong: their branch sat while `main` moved.
Blocking them punishes the wrong person for a race they cannot observe, and leaves them with
no obvious remedy. Raising and disclosing repairs the state and says so.
