---
change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
artifact: context
---

# Context

#965, from the Codex review of #963 (P2), deferred by Leif's call. #963 (#902) taught
`.github/scripts/archive-readiness.sh` to stop naming `specsync change ship` for a branch copy of a
change the remote default already archived, but only when a remote default ref is present. The
Archive jobs in `ci.yml` and `trust.yml` checked out the PR head at depth 1 and fetched nothing
else, so in CI no default ref existed and a stale copy was still told to ship.
REQ-cmd-change-029 says the script "that the Archive check runs SHALL NOT name `change ship`".

The fix checks out full history (`fetch-depth: 0`) in both Archive jobs, which also fetches the
default branch. A depth-1 fetch of the default branch is not enough: the script finds the commit
that added the archive with `log --diff-filter=A <default>` and asks `merge-base --is-ancestor`
whether HEAD contains it, and in shallow history a branch that already contains the archive (the
reopen-and-rearchive path) would read as a stale copy. The repository is about 70 MiB, and other
jobs here already check out full history. The script itself still never fetches.

`test-required-ci-gate.py` gains rule 4: every job that runs the archive gate on the checked-out
repository (no `root` input) checks out full history, in `ci.yml` and `trust.yml`. The packaged
Action consumer job gates a fixture under `runner.temp`, so it is out of the rule.

Chough: Leif's rule (relayed by t370 on 2026-11-13) is "make sure u use chough 0.6.0 for all
spec-sync changes". spec-sync's `.chough/` is still 0.5.x until spec-sync#1073 merges, so this
change is built now and goes through a Chough 0.6.0 session before review.
