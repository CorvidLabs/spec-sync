# Lesson bundle: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship

Material for folding this change's lessons into each affected module's lesson file.
Synthesise from what actually happened below; do not restate the change description.

## What this change was

- **Title**: The Archive check sees the default branch's history, so it never tells a stale copy of an upstream archive to ship
- **Kind**: BugFix
- **Specs**: github
- **Paths**: .github/workflows/ci.yml, .github/workflows/trust.yml, .github/scripts/test-required-ci-gate.py
- **Acceptance**: the Archive jobs in ci.yml and trust.yml check out full history before the archive gate, and test-required-ci-gate.py fails when a job that runs the gate on the repository checks out a single commit; the archive gate script itself still never fetches

## Evidence

- Verification commit: `122d5bf29cb90ff4c5c11c2059faebbcf51e6fed`
- Base commit: `18385ef5ead84f0b51d70a29a6fed8a3e5ca222a`
- Verified by: `specsync check --spec github --strict`

## From the change's context.md

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

## From the change's testing.md

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-029 (the Archive check never names ship for an upstream-archived copy) | `ArchiveGateHistoryTests` in `.github/scripts/test-required-ci-gate.py` (red before the fix, green after), plus the existing `.github/scripts/test-archive-readiness.sh` stale-copy case, which runs the script with a remote default present |

## How it runs

`fledge run ci-gate-test` (`python3 .github/scripts/test-required-ci-gate.py`) locally and in
hosted CI. The workflow runtime pin check (`validate-workflow-runtime-pins.py`) still passes.

## Where these lessons go

- `specs/github/github.3md`
