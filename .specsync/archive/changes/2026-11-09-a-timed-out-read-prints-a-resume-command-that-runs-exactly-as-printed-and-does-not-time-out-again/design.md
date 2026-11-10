---
change: a-timed-out-read-prints-a-resume-command-that-runs-exactly-as-printed-and-does-not-time-out-again
artifact: design
---

# Design

- **Quoting.** Every argument that is not made only of `[A-Za-z0-9_@%+=:,./-]` is single-quoted, with each `'` written as `'\''`. An empty argument becomes `''`. Plain arguments stay bare, so the common command reads as before.
- **Environment.** When `SPECSYNC_TIMEOUT` is set in the process environment, the command is prefixed with `SPECSYNC_TIMEOUT= `. An empty value parses as no deadline (`read_timeout_secs`), so the resumed run has none. Without the variable, there's no prefix.
- `--timeout <n>` and `--timeout=<n>` are still dropped. The report shape, exit 124 and the cache flush are unchanged.
