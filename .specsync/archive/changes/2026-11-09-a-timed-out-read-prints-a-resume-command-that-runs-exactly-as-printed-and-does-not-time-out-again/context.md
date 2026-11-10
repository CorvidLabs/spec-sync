---
change: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again
artifact: context
---

# Context

G-resume in Leif's burn-down (`_audits/specsync-triage-2026-11-06.md`): #923 and #953, two Codex P2s on #918 that were deferred. #918's read deadline (REQ-change-137, REQ-cmd-change-032) prints "the exact command that resumes the work". `resume_command_without_timeout` quotes an argument only when it is empty or contains whitespace, so `$`, `;`, `*` and `'` are printed bare. Pasting the command can then run something else (#923). The command also drops `--timeout` but leaves `SPECSYNC_TIMEOUT` in force, so in the same shell it times out again (#953). REQ-cmd-change-036, assigned by the orc. HI: SPEED-1.
