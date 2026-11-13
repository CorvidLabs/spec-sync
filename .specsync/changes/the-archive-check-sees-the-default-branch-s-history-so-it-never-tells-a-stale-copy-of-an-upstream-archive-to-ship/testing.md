---
change: the-archive-check-sees-the-default-branch-s-history-so-it-never-tells-a-stale-copy-of-an-upstream-archive-to-ship
artifact: testing
---

# Testing

## Requirement evidence

| Requirement | Evidence |
|---|---|
| REQ-cmd-change-029 (the Archive check never names ship for an upstream-archived copy) | `ArchiveGateHistoryTests` in `.github/scripts/test-required-ci-gate.py` (red before the fix, green after), plus the existing `.github/scripts/test-archive-readiness.sh` stale-copy case, which runs the script with a remote default present |

## How it runs

`fledge run ci-gate-test` (`python3 .github/scripts/test-required-ci-gate.py`) locally and in
hosted CI. The workflow runtime pin check (`validate-workflow-runtime-pins.py`) still passes.
