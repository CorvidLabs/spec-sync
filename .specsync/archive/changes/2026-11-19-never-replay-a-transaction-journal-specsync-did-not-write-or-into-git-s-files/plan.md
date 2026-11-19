---
change: never-replay-a-transaction-journal-specsync-did-not-write-or-into-git-s-files
artifact: plan
---

# Plan

## Approach

Failing tests first. Then the tracking check, the path rule on both sides of the journal, and the end of the legacy form.

## Out of scope

- Moving the journal into the Git directory, and keying the digest: both left out by Leif's decision.
- The other review findings (F2 to F9, R1, R2, G1), which are change 5.

## Steps

1. Unit tests in `src/change_tests/journal_replay_targets.rs`, and a CLI test through `change new` with a committed journal for `.git/config`. Run them on `cf528407` and keep the failures.
2. Add the tracking check, the path rule, the whole-journal refusal and the legacy refusal. Apply the rule where transactions are written.
3. Update the `change` spec, `CHANGELOG.md`, and a lesson in `change.3md`.
4. Run `change check --commit`, the verify lane, and the Linux container run.

## Risks

- **A 5.x run interrupted mid-write and then upgraded straight to 6.4.0** must recover by hand with the printed `.broken` command.
- **A transaction this build writes outside the rule** would now fail before writing instead of being journaled. The full suite finds any such path.
- **Each writing command runs one or two `git` calls** when a journal exists, and none when it does not.

## Constraints consulted

- `specs/change/change.3md`: a refusal is a result worth recording, and recovery text is a contract. The existing invalid-journal message and its `.broken` command are reused.
- `specs/util/util.3md`: enforce one rule where a path is written and where it is read back.
